//! Read-only, installation-only inspection of the current selected VM disk.
//! KVP and VM names are deliberately excluded: neither proves current disk OS.
use super::{TIMEOUT, inspect, settings};
use crate::{
    guest_compatibility::{WindowsIdentity, require_supported, unknown},
    model::{Power, Target},
    windows_wmi::Session,
};
use std::{path::Path, time::Duration};
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        Storage::Vhd::*,
        System::Registry::*,
    },
    core::{PCWSTR, PWSTR},
};

const BASIC: &str = "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7";
const EFI: &str = "c12a7328-f81f-11d2-ba4b-00a0c93ec93b";
const MSR: &str = "e3c9e316-0b5c-4db8-817d-f92df00215ae";
const RECOVERY: &str = "de94bba4-06d1-4d40-a16a-bfd50179d6ac";

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

fn disk_path(t: &Target) -> Result<String, String> {
    if inspect(&t.vm_id)?.power != Power::Off {
        return Err(unknown());
    }
    let session = Session::new(TIMEOUT)?;
    let cfg = settings(&session, &t.vm_id)?;
    let resources = session.related(&cfg, "Msvm_StorageAllocationSettingData")?;
    let mut disks = Vec::new();
    for resource in resources {
        match resource.number("ResourceType")? {
            31 => {
                if resource.string("ResourceSubType")? != "Microsoft:Hyper-V:Virtual Hard Disk" {
                    return Err(unknown());
                }
                let paths = resource.strings("HostResource")?;
                let [path] = paths.as_slice() else {
                    return Err(unknown());
                };
                disks.push(path.clone());
            }
            // Bootable optical media would make the next guest boot ambiguous.
            16 if !resource.strings("HostResource")?.is_empty() => return Err(unknown()),
            _ => {}
        }
    }
    let [path] = disks.as_slice() else {
        return Err(unknown());
    };
    if path.contains('\0') || !Path::new(path).is_absolute() {
        return Err(unknown());
    }
    crate::payload::no_reparse(Path::new(path))?;
    Ok(path.clone())
}

struct Disk(HANDLE);
#[allow(unsafe_code)]
impl Disk {
    fn open(path: &str) -> Result<Self, String> {
        let path = wide(path);
        let storage = VIRTUAL_STORAGE_TYPE {
            DeviceId: VIRTUAL_STORAGE_TYPE_DEVICE_UNKNOWN,
            VendorId: VIRTUAL_STORAGE_TYPE_VENDOR_MICROSOFT,
        };
        let parameters = OPEN_VIRTUAL_DISK_PARAMETERS {
            Version: OPEN_VIRTUAL_DISK_VERSION_2,
            Anonymous: OPEN_VIRTUAL_DISK_PARAMETERS_0 {
                Version2: OPEN_VIRTUAL_DISK_PARAMETERS_0_1 {
                    ReadOnly: true.into(),
                    ..Default::default()
                },
            },
        };
        let mut handle = HANDLE::default();
        // SAFETY: initialized parameters, terminated path and writable handle.
        // Request no write access. Lifetime is owned by Disk, including failures.
        unsafe {
            OpenVirtualDisk(
                &storage,
                PCWSTR(path.as_ptr()),
                VIRTUAL_DISK_ACCESS_NONE,
                OPEN_VIRTUAL_DISK_FLAG_NONE,
                Some(&parameters),
                &mut handle,
            )
        }
        .ok()
        .map_err(|_| unknown())?;
        let disk = Self(handle);
        let parameters = ATTACH_VIRTUAL_DISK_PARAMETERS {
            Version: ATTACH_VIRTUAL_DISK_VERSION_1,
            ..Default::default()
        };
        // SAFETY: live owned VHD handle and initialized parameters. No permanent
        // lifetime: closing this handle removes the temporary read-only surface,
        // even if the operation fails. Never assign drive letters or unlock it.
        unsafe {
            AttachVirtualDisk(
                disk.0,
                None,
                ATTACH_VIRTUAL_DISK_FLAG_READ_ONLY | ATTACH_VIRTUAL_DISK_FLAG_NO_DRIVE_LETTER,
                0,
                Some(&parameters),
                None,
            )
        }
        .ok()
        .map_err(|_| unknown())?;
        Ok(disk)
    }

    fn number(&self) -> Result<u32, String> {
        let mut path = [0u16; 1024];
        let mut bytes = std::mem::size_of_val(&path) as u32;
        // SAFETY: live disk handle, writable buffer and its byte capacity.
        unsafe { GetVirtualDiskPhysicalPath(self.0, &mut bytes, PWSTR(path.as_mut_ptr())) }
            .ok()
            .map_err(|_| unknown())?;
        let end = path.iter().position(|c| *c == 0).ok_or_else(unknown)?;
        let path = String::from_utf16(&path[..end]).map_err(|_| unknown())?;
        path.to_ascii_lowercase()
            .strip_prefix(r"\\.\physicaldrive")
            .ok_or_else(unknown)?
            .parse()
            .map_err(|_| unknown())
    }
}
#[allow(unsafe_code)]
impl Drop for Disk {
    fn drop(&mut self) {
        // SAFETY: exclusively owned successful OpenVirtualDisk handle. The
        // non-permanent attachment cannot outlive the final handle close.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

struct Hive(HKEY);
pub(crate) fn local_client_installation() -> Result<bool, String> {
    // Borrow the predefined key without passing it through the owning Hive type.
    Ok(registry_string(
        HKEY_LOCAL_MACHINE,
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
        "InstallationType",
    )? == "Client")
}

#[allow(unsafe_code)]
fn registry_string(key: HKEY, subkey: &str, name: &str) -> Result<String, String> {
    let subkey = wide(subkey);
    let name = wide(name);
    let mut data = [0u16; 128];
    let mut bytes = std::mem::size_of_val(&data) as u32;
    // SAFETY: live borrowed key, terminated names, bounded writable UTF-16 buffer.
    unsafe {
        RegGetValueW(
            key,
            PCWSTR(subkey.as_ptr()),
            PCWSTR(name.as_ptr()),
            RRF_RT_REG_SZ | RRF_NOEXPAND,
            None,
            Some(data.as_mut_ptr().cast()),
            Some(&mut bytes),
        )
    }
    .ok()
    .map_err(|_| unknown())?;
    let end = data.iter().position(|c| *c == 0).ok_or_else(unknown)?;
    String::from_utf16(&data[..end]).map_err(|_| unknown())
}

#[allow(unsafe_code)]
impl Hive {
    fn open(path: &Path) -> Result<Self, String> {
        // Explicit known ancestors only: never follow a guest junction into the
        // host registry. The entire backing disk is already attached read-only.
        for ancestor in path.ancestors().take(5) {
            use std::os::windows::fs::MetadataExt;
            if std::fs::symlink_metadata(ancestor)
                .map_err(|_| unknown())?
                .file_attributes()
                & 0x400
                != 0
            {
                return Err(unknown());
            }
        }
        let path = wide(path.to_str().ok_or_else(unknown)?);
        let mut key = HKEY::default();
        // SAFETY: terminated hive path, writable key output. Private process
        // hive, KEY_READ only, no global registry namespace or write access.
        unsafe {
            RegLoadAppKeyW(
                PCWSTR(path.as_ptr()),
                &mut key,
                KEY_READ.0,
                REG_PROCESS_APPKEY,
                None,
            )
        }
        .ok()
        .map_err(|_| unknown())?;
        Ok(Self(key))
    }

    fn string(&self, name: &str) -> Result<String, String> {
        registry_string(self.0, r"Microsoft\Windows NT\CurrentVersion", name)
    }

    fn dword(&self, name: &str) -> Result<u32, String> {
        let subkey = wide(r"Microsoft\Windows NT\CurrentVersion");
        let name = wide(name);
        let mut value = 0u32;
        let mut bytes = 4u32;
        // SAFETY: live hive, terminated names and exactly four writable bytes;
        // RRF_RT_REG_DWORD rejects other registry representations.
        unsafe {
            RegGetValueW(
                self.0,
                PCWSTR(subkey.as_ptr()),
                PCWSTR(name.as_ptr()),
                RRF_RT_REG_DWORD,
                None,
                Some((&mut value as *mut u32).cast()),
                Some(&mut bytes),
            )
        }
        .ok()
        .map_err(|_| unknown())?;
        if bytes != 4 {
            return Err(unknown());
        }
        Ok(value)
    }
}
#[allow(unsafe_code)]
impl Drop for Hive {
    fn drop(&mut self) {
        // SAFETY: sole process-private key. Final close unloads the private hive.
        let _ = unsafe { RegCloseKey(self.0) };
    }
}

fn windows_volume(session: &Session, number: u32) -> Result<String, String> {
    let partitions = session.query(&format!(
        "SELECT * FROM MSFT_Partition WHERE DiskNumber={number}"
    ))?;
    let observations = partitions
        .into_iter()
        .map(|partition| {
            let kind = partition
                .string("GptType")?
                .trim_matches(['{', '}'])
                .to_ascii_lowercase();
            let paths = if kind == BASIC {
                partition.strings("AccessPaths")?
            } else {
                Vec::new()
            };
            Ok((kind, paths))
        })
        .collect::<Result<Vec<_>, String>>()?;
    select_windows_volume(observations)
}

fn select_windows_volume(partitions: Vec<(String, Vec<String>)>) -> Result<String, String> {
    let mut counts = [0u32; 4];
    let mut volume = None;
    for (kind, paths) in partitions {
        let index = match kind.as_str() {
            BASIC => 0,
            EFI => 1,
            MSR => 2,
            RECOVERY => 3,
            _ => return Err(unknown()),
        };
        counts[index] += 1;
        if index == 0 {
            let paths: Vec<_> = paths
                .into_iter()
                .filter(|p| p.starts_with(r"\\?\Volume{") && p.ends_with("}\\"))
                .collect();
            let [path] = paths.as_slice() else {
                return Err(unknown());
            };
            // Do not accept arbitrary provider strings as a host file path.
            let guid = &path[11..path.len() - 2];
            if windows::core::GUID::try_from(guid).is_err() {
                return Err(unknown());
            }
            volume = Some(path.clone());
        }
    }
    if counts[0] != 1 || counts[1] != 1 || counts[2] > 1 || counts[3] > 1 {
        return Err(unknown());
    }
    volume.ok_or_else(unknown)
}

pub(crate) fn check_offline(t: &Target) -> Result<(), String> {
    let changed = || {
        crate::reporting::ComponentFailure {
            stage: None,
            component: crate::reporting::Component::GuestCompatibility,
            cause: crate::reporting::FailureCause::ReadbackMismatch,
            code: None,
        }
        .into_error()
    };
    let observe = || {
        let path = disk_path(t)?;
        let disk = Disk::open(&path)?;
        let session =
            Session::in_namespace(r"ROOT\Microsoft\Windows\Storage", Duration::from_secs(30))?;
        let volume = windows_volume(&session, disk.number()?)?;
        let hive = Hive::open(&Path::new(&volume).join(r"Windows\System32\config\SOFTWARE"))?;
        let identity = WindowsIdentity {
            major: hive.dword("CurrentMajorVersionNumber")?,
            minor: hive.dword("CurrentMinorVersionNumber")?,
            build: hive
                .string("CurrentBuildNumber")?
                .parse()
                .map_err(|_| unknown())?,
            workstation: hive.string("InstallationType")? == "Client",
        };
        // Catch VM power/configuration changes while its disk was inspected.
        if disk_path(t).map_err(|_| changed())? != path {
            return Err(changed());
        }
        Ok(identity)
    };
    // Do not expose guest-controlled registry strings, provider prose or paths.
    let identity = observe().map_err(|error: String| {
        if matches!(
            crate::reporting::ComponentFailure::decode(&error),
            Some(crate::reporting::ComponentFailure {
                component: crate::reporting::Component::GuestCompatibility,
                cause: crate::reporting::FailureCause::ReadbackMismatch,
                ..
            })
        ) {
            error
        } else {
            unknown()
        }
    })?;
    require_supported(&identity)
}

#[cfg(test)]
mod tests {
    use super::*;
    const VOLUME: &str = r"\\?\Volume{11111111-1111-1111-1111-111111111111}\";

    fn layout() -> Vec<(String, Vec<String>)> {
        vec![
            (EFI.into(), vec![]),
            (MSR.into(), vec![]),
            (BASIC.into(), vec![VOLUME.into()]),
            (RECOVERY.into(), vec![]),
        ]
    }

    #[test]
    fn offline_layout_requires_one_unambiguous_windows_candidate() {
        assert_eq!(select_windows_volume(layout()).unwrap(), VOLUME);
        for invalid in 0..7 {
            let mut partitions = layout();
            match invalid {
                0 => partitions.push((BASIC.into(), vec![VOLUME.into()])),
                1 => partitions[2].1.clear(), // Locked/unavailable volume is not proof.
                2 => partitions[2].1 = vec![r"C:\".into()],
                3 => partitions[2].1 = vec![r"\\?\Volume{..}\".into()],
                4 => {
                    partitions.remove(0);
                }
                5 => partitions.push(("linux-or-unrecognised-partition".into(), vec![])),
                _ => partitions[2].1.push(VOLUME.into()),
            }
            assert!(
                select_windows_volume(partitions).is_err(),
                "layout {invalid}"
            );
        }
        assert!(select_windows_volume(vec![]).is_err());
    }
}
