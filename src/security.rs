//! Trusted ownership, restrictive permissions and bounded protected-file reads.
use crate::payload;
use std::{io::Read, path::Path};
use windows::{
    Win32::{
        Foundation::{HLOCAL, LocalFree},
        Security::Authorization::{
            ConvertSidToStringSidW, ConvertStringSidToSidW, GetNamedSecurityInfoW, SE_FILE_OBJECT,
            SetNamedSecurityInfoW,
        },
        Security::{
            ACCESS_ALLOWED_ACE, ACE_HEADER, DACL_SECURITY_INFORMATION, GetAce,
            OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID,
        },
    },
    core::PCWSTR,
};
#[allow(unsafe_code)]
fn sid_text(sid: PSID) -> Result<String, String> {
    let mut text = windows::core::PWSTR::null();
    // SAFETY: SID belongs to a live security descriptor; API allocates terminated text.
    unsafe { ConvertSidToStringSidW(sid, &mut text) }.map_err(|e| e.to_string())?;
    let result = unsafe { text.to_string() }.map_err(|e| e.to_string());
    // SAFETY: sole allocation returned by SID conversion.
    unsafe {
        LocalFree(Some(HLOCAL(text.0.cast())));
    }
    result
}
/// Set trusted ownership while retaining the destination parent's inherited read access.
#[allow(unsafe_code)]
pub(crate) fn set_administrator_owner(path: &Path) -> Result<(), String> {
    payload::no_reparse(path)?;
    let admin: Vec<_> = "S-1-5-32-544".encode_utf16().chain(Some(0)).collect();
    let wide: Vec<_> = path
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut sid = PSID::default();
    // SAFETY: trusted terminated SID, API-owned allocation, terminated existing path.
    unsafe { ConvertStringSidToSidW(PCWSTR(admin.as_ptr()), &mut sid) }
        .map_err(|e| e.to_string())?;
    let result = unsafe {
        SetNamedSecurityInfoW(
            PCWSTR(wide.as_ptr()),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION,
            Some(sid),
            None,
            None,
            None,
        )
    }
    .ok()
    .map_err(|e| e.to_string());
    unsafe {
        LocalFree(Some(HLOCAL(sid.0)));
    }
    result
}
/// Reject untrusted ownership or any non-administrator write-capable allow ACE.
#[allow(unsafe_code)]
pub(crate) fn verify(path: &Path) -> Result<(), String> {
    verify_owner(path, false)
}
/// Observe an entry without following a link, including dangling retention markers.
pub(crate) fn entry_exists(path: &Path) -> Result<bool, String> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.to_string()),
    }
}
/// Read an optional protected regular file. Only metadata NotFound means absence;
/// a failed trust check, open or read must remain an error, including disappearance
/// after metadata inspection. Callers retain their own bounds and diagnostics.
pub(crate) fn read_optional(
    path: &Path,
    limit: usize,
    not_file: &str,
    too_large: &str,
) -> Result<Option<Vec<u8>>, String> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
        Ok(metadata) if !metadata.is_file() => return Err(not_file.into()),
        Ok(_) => {}
    }
    verify(path)?;
    let input = std::fs::File::open(path).map_err(|e| e.to_string())?;
    read_bounded(input, limit, too_large).map(Some)
}

fn read_bounded(input: impl Read, limit: usize, too_large: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    input
        .take(limit.saturating_add(1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err(too_large.into());
    }
    Ok(bytes)
}
/// Restrict cleanup/metadata leaves to protected regular files with one link.
#[allow(unsafe_code)]
pub(crate) fn verify_single_file(path: &Path) -> Result<(), String> {
    use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
    use windows::Win32::{
        Foundation::HANDLE,
        Storage::FileSystem::{BY_HANDLE_FILE_INFORMATION, GetFileInformationByHandle},
    };
    verify(path)?;
    if !std::fs::symlink_metadata(path)
        .map_err(|e| e.to_string())?
        .is_file()
    {
        return Err("protected leaf is not a regular file".into());
    }
    let file = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(path)
        .map_err(|e| e.to_string())?;
    let mut information = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: live owned file handle and writable fixed-size information structure.
    unsafe { GetFileInformationByHandle(HANDLE(file.as_raw_handle()), &mut information) }
        .map_err(|e| e.to_string())?;
    if information.nNumberOfLinks != 1 {
        return Err("protected hard link refused".into());
    }
    Ok(())
}
/// Windows destinations may be owned by TrustedInstaller as well as SYSTEM/admins.
pub(crate) fn verify_system(path: &Path) -> Result<(), String> {
    verify_owner(path, true)
}
fn trusted(sid: &str, system: bool) -> bool {
    matches!(sid, "S-1-5-18" | "S-1-5-32-544")
        || (system && sid == "S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464")
}
/// Create a new file with trusted ownership at creation, inheriting only the
/// explicit protected DACL. No caller-owned or inherited-writer interval.
#[allow(unsafe_code)]
pub(crate) fn create_file(path: &Path, client: &str) -> Result<std::fs::File, String> {
    use std::os::windows::io::FromRawHandle;
    use windows::Win32::{
        Foundation::{GENERIC_READ, GENERIC_WRITE},
        Storage::FileSystem::{CREATE_NEW, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_MODE},
    };
    verify(path.parent().ok_or("protected file parent missing")?)?;
    let sddl: Vec<u16> = format!("O:BAG:BAD:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FR;;;{client})")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let wide: Vec<u16> = path
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut descriptor = PSECURITY_DESCRIPTOR::default();
    // SAFETY: fixed terminated descriptor; owner is assigned atomically by CreateFile.
    unsafe { windows::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW(PCWSTR(sddl.as_ptr()), windows::Win32::Security::Authorization::SDDL_REVISION_1, &mut descriptor, None) }.map_err(|e| e.to_string())?;
    let attributes = windows::Win32::Security::SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<windows::Win32::Security::SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: false.into(),
    };
    // SAFETY: descriptor/path live through call; exclusive new file cannot replace an existing object.
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            GENERIC_READ.0 | GENERIC_WRITE.0,
            FILE_SHARE_MODE(0),
            Some(&attributes),
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    };
    unsafe {
        LocalFree(Some(HLOCAL(descriptor.0)));
    }
    let handle = handle.map_err(|e| e.to_string())?;
    // SAFETY: successful newly created handle transfers once into File ownership.
    Ok(unsafe { std::fs::File::from_raw_handle(handle.0) })
}
/// Create a new protected directory with its owner and DACL installed atomically.
#[allow(unsafe_code)]
pub(crate) fn create_directory(path: &Path, client: &str) -> Result<(), String> {
    payload::no_reparse(path.parent().ok_or("protected directory parent missing")?)?;
    let sddl: Vec<u16> =
        format!("O:BAG:BAD:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)(A;OICI;FRFX;;;{client})")
            .encode_utf16()
            .chain(Some(0))
            .collect();
    let wide: Vec<u16> = path
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut descriptor = PSECURITY_DESCRIPTOR::default();
    // SAFETY: terminated SDDL and initialized API-owned output descriptor.
    unsafe { windows::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW(PCWSTR(sddl.as_ptr()), windows::Win32::Security::Authorization::SDDL_REVISION_1, &mut descriptor, None) }.map_err(|e| e.to_string())?;
    let attributes = windows::Win32::Security::SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<windows::Win32::Security::SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: false.into(),
    };
    // SAFETY: live descriptor and terminated destination; fails rather than replacing an existing object.
    let result = unsafe {
        windows::Win32::Storage::FileSystem::CreateDirectoryW(
            PCWSTR(wide.as_ptr()),
            Some(&attributes),
        )
    }
    .map_err(|e| e.to_string());
    // SAFETY: descriptor is solely owned here and CreateDirectory copies it.
    unsafe {
        LocalFree(Some(HLOCAL(descriptor.0)));
    }
    result?;
    verify(path)
}
#[allow(unsafe_code)]
fn verify_owner(path: &Path, system: bool) -> Result<(), String> {
    payload::no_reparse(path)?;
    let wide: Vec<_> = path
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut descriptor = PSECURITY_DESCRIPTOR::default();
    let mut owner = PSID::default();
    let mut acl = std::ptr::null_mut();
    // SAFETY: terminated existing path and initialized descriptor/owner/ACL outputs.
    unsafe {
        GetNamedSecurityInfoW(
            PCWSTR(wide.as_ptr()),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            Some(&mut owner),
            None,
            Some(&mut acl),
            None,
            &mut descriptor,
        )
    }
    .ok()
    .map_err(|e| e.to_string())?;
    let result = (|| {
        if !trusted(&sid_text(owner)?, system) {
            return Err("protected product path is not owned by SYSTEM/Administrators".into());
        }
        if acl.is_null() {
            return Err("protected product path has a null DACL".into());
        }
        // SAFETY: ACL remains owned by the live descriptor; GetAce validates indexes.
        for index in 0..u32::from(unsafe { (*acl).AceCount }) {
            let mut pointer = std::ptr::null_mut();
            unsafe { GetAce(acl, index, &mut pointer) }.map_err(|e| e.to_string())?;
            let header = unsafe { &*pointer.cast::<ACE_HEADER>() };
            // Inherit-only rules do not grant rights on this object.
            if header.AceFlags & 0x08 != 0 {
                continue;
            }
            if header.AceType == 0 {
                if usize::from(header.AceSize) < std::mem::size_of::<ACCESS_ALLOWED_ACE>() {
                    return Err("malformed product ACL".into());
                }
                let ace = unsafe { &*pointer.cast::<ACCESS_ALLOWED_ACE>() };
                let sid = PSID((&ace.SidStart as *const u32).cast_mut().cast());
                // Write data/append/EA/attributes/delete child/delete/DAC/owner and generic write/all.
                if ace.Mask & 0x500d_0156 != 0 && !trusted(&sid_text(sid)?, system) {
                    return Err(
                        "protected product path grants non-administrator write access".into(),
                    );
                }
            } else if header.AceType != 1 && header.AceType != 2 {
                return Err("unsupported product ACL entry".into());
            }
        }
        Ok(())
    })();
    // SAFETY: sole GetNamedSecurityInfo allocation, released after all borrowed references.
    unsafe {
        LocalFree(Some(HLOCAL(descriptor.0)));
    }
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn protected_read_accepts_exact_bounds_and_stops_after_one_excess_byte() {
        use std::io::Cursor;
        for limit in [0, 1, 65536, crate::runner::FRAME_LIMIT] {
            let expected = vec![b'x'; limit];
            assert_eq!(
                super::read_bounded(Cursor::new(&expected), limit, "oversized").unwrap(),
                expected
            );
            let mut oversized = Cursor::new(vec![b'x'; limit + 32]);
            assert_eq!(
                super::read_bounded(&mut oversized, limit, "record-specific size failure")
                    .unwrap_err(),
                "record-specific size failure"
            );
            assert_eq!(oversized.position(), limit as u64 + 1);
        }
    }

    #[test]
    fn protected_read_preserves_a_read_failure_after_partial_input() {
        struct FailedRead;
        impl std::io::Read for FailedRead {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other(
                    "read interrupted after partial input",
                ))
            }
        }
        use std::io::Read;
        let input = std::io::Cursor::new(b"partial record").chain(FailedRead);
        assert_eq!(
            super::read_bounded(input, 65536, "oversized").unwrap_err(),
            "read interrupted after partial input"
        );
    }

    #[test]
    fn optional_protected_read_distinguishes_absence_from_a_non_file() {
        let root = std::env::temp_dir();
        let missing = root.join(format!("hyper-gpu-absent-read-{}", std::process::id()));
        assert!(!missing.exists());
        assert_eq!(
            super::read_optional(&missing, 65536, "not a regular file", "oversized").unwrap(),
            None
        );
        assert_eq!(
            super::read_optional(&root, 65536, "not a regular file", "oversized").unwrap_err(),
            "not a regular file"
        );
    }

    #[test]
    fn windows_owner_exception_does_not_expand_private_state_ownership() {
        let installer = "S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464";
        assert!(super::trusted(installer, true));
        assert!(!super::trusted(installer, false));
        assert!(!super::trusted("S-1-5-32-545", true));
    }
    #[test]
    fn operator_writable_temporary_file_cannot_be_protected_state() {
        let path =
            std::env::temp_dir().join(format!("hyper-gpu-untrusted-{}.json", std::process::id()));
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        let checked = super::verify(&path);
        // Trust is checked before bytes or their size can be accepted.
        let read = super::read_optional(&path, 0, "not a regular file", "oversized");
        std::fs::remove_file(&path).unwrap();
        assert!(checked.is_err());
        assert_eq!(read.unwrap_err(), checked.unwrap_err());
    }
}
