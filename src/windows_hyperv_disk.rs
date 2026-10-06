//! Native Virtual Disk identity and protected parent/child filesystem guards.

use crate::{config::ProjectConfiguration, runner::embedded_policy};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::Read,
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    path::{Component, Path},
    time::Instant,
};
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        Storage::{
            FileSystem::{
                FILE_ATTRIBUTE_READONLY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS,
                FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES, FILE_SHARE_READ,
                FILE_SHARE_WRITE,
            },
            Vhd::*,
        },
    },
    core::{BOOL, PCWSTR},
};

type Result<T> = std::result::Result<T, String>;

/// Hold each ancestor directory without delete sharing. This prevents replacement
/// or junction insertion between path validation and the fixed child effects.
pub(crate) struct DirectoryGuard {
    _handles: Vec<File>,
}
impl DirectoryGuard {
    pub(crate) fn acquire(paths: &[&Path]) -> Result<Self> {
        let mut handles = Vec::new();
        let mut opened = std::collections::BTreeSet::new();
        for path in paths {
            if !path.is_absolute()
                || path
                    .components()
                    .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
            {
                return Err("unsafe disk guard path".into());
            }
            let directory = path.parent().ok_or("disk has no containing directory")?;
            let mut prefix = std::path::PathBuf::new();
            for component in directory.components() {
                prefix.push(component);
                if matches!(component, Component::Prefix(_)) || !opened.insert(prefix.clone()) {
                    continue;
                }
                let file = OpenOptions::new()
                    .access_mode(FILE_READ_ATTRIBUTES.0)
                    .share_mode(FILE_SHARE_READ.0 | FILE_SHARE_WRITE.0)
                    .custom_flags(FILE_FLAG_BACKUP_SEMANTICS.0 | FILE_FLAG_OPEN_REPARSE_POINT.0)
                    .open(&prefix)
                    .map_err(|e| format!("lock disk ancestor {}: {e}", prefix.display()))?;
                let m = file.metadata().map_err(|e| e.to_string())?;
                if !m.is_dir() || m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0 {
                    return Err(format!("unsafe disk ancestor {}", prefix.display()));
                }
                handles.push(file);
            }
        }
        Ok(Self { _handles: handles })
    }
}
fn wide(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    path.as_os_str().encode_wide().chain(Some(0)).collect()
}
struct Disk(HANDLE);
#[allow(unsafe_code)]
impl Drop for Disk {
    fn drop(&mut self) {
        // SAFETY: this handle is exclusively owned from a successful disk API call.
        let _ = unsafe { CloseHandle(self.0) };
    }
}
#[allow(unsafe_code)]
impl Disk {
    fn open(path: &Path) -> Result<Self> {
        let storage = VIRTUAL_STORAGE_TYPE {
            DeviceId: VIRTUAL_STORAGE_TYPE_DEVICE_VHDX,
            VendorId: VIRTUAL_STORAGE_TYPE_VENDOR_MICROSOFT,
        };
        let params = OPEN_VIRTUAL_DISK_PARAMETERS {
            Version: OPEN_VIRTUAL_DISK_VERSION_2,
            Anonymous: OPEN_VIRTUAL_DISK_PARAMETERS_0 {
                Version2: OPEN_VIRTUAL_DISK_PARAMETERS_0_1 {
                    GetInfoOnly: BOOL(1),
                    ReadOnly: BOOL(1),
                    ..Default::default()
                },
            },
        };
        let mut handle = HANDLE::default();
        // SAFETY: typed VHDX parameters match version 2; terminated path and output live.
        unsafe {
            OpenVirtualDisk(
                &storage,
                PCWSTR(wide(path).as_ptr()),
                VIRTUAL_DISK_ACCESS_NONE,
                OPEN_VIRTUAL_DISK_FLAG_NONE,
                Some(&params),
                &mut handle,
            )
        }
        .ok()
        .map_err(|e| format!("OpenVirtualDisk {}: {e}", path.display()))?;
        Ok(Self(handle))
    }
    fn info(&self, version: GET_VIRTUAL_DISK_INFO_VERSION) -> Result<GET_VIRTUAL_DISK_INFO> {
        let mut value = GET_VIRTUAL_DISK_INFO {
            Version: version,
            ..Default::default()
        };
        let mut bytes = std::mem::size_of_val(&value) as u32;
        // SAFETY: valid disk handle and correctly sized aligned output for scalar version.
        unsafe { GetVirtualDiskInformation(self.0, &mut bytes, &mut value, None) }
            .ok()
            .map_err(|e| format!("GetVirtualDiskInformation {}: {e}", version.0))?;
        Ok(value)
    }
    fn subtype(&self) -> Result<u32> {
        let value = self.info(GET_VIRTUAL_DISK_INFO_PROVIDER_SUBTYPE)?;
        // SAFETY: successful query with PROVIDER_SUBTYPE selects this union arm.
        Ok(unsafe { value.Anonymous.ProviderSubtype })
    }
    fn loaded(&self) -> Result<bool> {
        let value = self.info(GET_VIRTUAL_DISK_INFO_IS_LOADED)?;
        // SAFETY: successful query with IS_LOADED selects this arm.
        Ok(unsafe { value.Anonymous.IsLoaded }.as_bool())
    }
    fn parent(&self) -> Result<String> {
        // DWORD-aligned header and enough bounded storage for a Windows NT path.
        let mut storage = vec![0u64; 8196];
        let capacity = std::mem::size_of_val(storage.as_slice());
        let info = storage.as_mut_ptr().cast::<GET_VIRTUAL_DISK_INFO>();
        // SAFETY: aligned initialized allocation bigger than header, selected version.
        unsafe {
            (*info).Version = GET_VIRTUAL_DISK_INFO_PARENT_LOCATION;
        }
        let mut bytes = capacity as u32;
        // SAFETY: valid handle and writable output allocation of advertised size.
        unsafe { GetVirtualDiskInformation(self.0, &mut bytes, info, None) }
            .ok()
            .map_err(|e| format!("read differencing parent: {e}"))?;
        if bytes as usize > capacity {
            return Err("VHD parent result exceeds allocation".into());
        }
        // SAFETY: selected PARENT_LOCATION; locate inline string inside owned buffer.
        let parent = unsafe { &(*info).Anonymous.ParentLocation };
        if !parent.ParentResolved.as_bool() {
            return Err("VHD parent cannot be resolved".into());
        }
        let pointer = parent.ParentLocationBuffer.as_ptr();
        let offset = pointer as usize - storage.as_ptr() as usize;
        if offset >= capacity {
            return Err("invalid VHD parent string offset".into());
        }
        // SAFETY: pointer and bounded length stay inside the initialized allocation.
        let text = unsafe { std::slice::from_raw_parts(pointer, (capacity - offset) / 2) };
        let end = text
            .iter()
            .position(|u| *u == 0)
            .ok_or("unterminated VHD parent path")?;
        String::from_utf16(&text[..end]).map_err(|_| "invalid VHD parent UTF-16".into())
    }
}

pub(crate) fn no_reparse(path: &Path, allow_missing_leaf: bool) -> Result<()> {
    if !path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return Err("unsafe VHD path".into());
    }
    let mut prefix = std::path::PathBuf::new();
    for component in path.components() {
        prefix.push(component);
        if matches!(component, Component::Prefix(_)) {
            continue;
        }
        match std::fs::symlink_metadata(&prefix) {
            Ok(m) if m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0 => {
                return Err(format!("reparse path refused: {}", prefix.display()));
            }
            Ok(_) => (),
            Err(e)
                if allow_missing_leaf
                    && prefix == path
                    && e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("VHD path guard {}: {e}", prefix.display())),
        }
    }
    Ok(())
}

pub(crate) struct ParentGuard {
    _file: File,
    pub(crate) hash: String,
}
impl ParentGuard {
    pub(crate) fn verify(
        project: &ProjectConfiguration,
        hash: bool,
        reset: bool,
        deadline: Instant,
    ) -> Result<Self> {
        let policy = embedded_policy().map_err(|e| e.to_string())?;
        no_reparse(&project.slot.parent_path, false)?;
        let mut file = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ.0)
            .open(&project.slot.parent_path)
            .map_err(|e| format!("open protected parent: {e}"))?;
        let metadata = file.metadata().map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.file_attributes() & FILE_ATTRIBUTE_READONLY.0 == 0 {
            return Err("parent protection mismatch".into());
        }
        let disk = Disk::open(&project.slot.parent_path)?;
        if disk.subtype()? != 3 || (reset && disk.loaded()?) {
            return Err("parent must be an unattached dynamic VHDX".into());
        }
        let mut actual = policy.parent_sha256().to_owned();
        if hash {
            let mut digest = Sha256::new();
            let mut buffer = vec![0u8; 1024 * 1024];
            loop {
                if Instant::now() >= deadline {
                    return Err("parent hash deadline expired".into());
                }
                let count = file
                    .read(&mut buffer)
                    .map_err(|e| format!("hash protected parent: {e}"))?;
                if count == 0 {
                    break;
                }
                digest.update(&buffer[..count]);
            }
            actual = digest
                .finalize()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            if actual != policy.parent_sha256() {
                return Err("parent hash mismatch".into());
            }
        }
        Ok(Self {
            _file: file,
            hash: actual,
        })
    }
}
pub(crate) fn verify_child(project: &ProjectConfiguration, allow_missing: bool) -> Result<()> {
    no_reparse(&project.slot.child_path, allow_missing)?;
    if allow_missing
        && !project
            .slot
            .child_path
            .try_exists()
            .map_err(|e| e.to_string())?
    {
        return Ok(());
    }
    let disk = Disk::open(&project.slot.child_path)?;
    if disk.subtype()? != 4
        || !disk
            .parent()?
            .eq_ignore_ascii_case(&project.slot.parent_path.to_string_lossy())
    {
        return Err("child differencing chain mismatch".into());
    }
    Ok(())
}

#[cfg(feature = "dev-harness")]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tools/test-harness/reset_disk.rs"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ancestor_guards_hold_identity_and_missing_leaf_does_not_hide_missing_ancestor() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("local/test-output")
            .join(format!("disk-guard-{}", std::process::id()));
        let directory = root.join("original");
        std::fs::create_dir_all(&directory).unwrap();
        // The developer checkout itself may be reached through a junction. Test
        // native guards on the resolved test directory, as product pins require.
        let root = std::fs::canonicalize(&root).unwrap();
        let directory = root.join("original");
        let leaf = directory.join("child.vhdx");
        assert_eq!(no_reparse(&leaf, true), Ok(()), "{}", leaf.display());
        assert!(no_reparse(&leaf, false).is_err());
        assert!(no_reparse(&root.join("absent/child.vhdx"), true).is_err());
        assert!(DirectoryGuard::acquire(&[Path::new("relative/child.vhdx")]).is_err());
        let guard = DirectoryGuard::acquire(&[&leaf]).unwrap();
        assert!(std::fs::rename(&directory, root.join("renamed")).is_err());
        drop(guard);
        std::fs::rename(&directory, root.join("renamed")).unwrap();
        std::fs::write(root.join("file"), "not a directory").unwrap();
        assert!(DirectoryGuard::acquire(&[&root.join("file/child.vhdx")]).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
