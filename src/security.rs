//! Trusted ownership and restrictive administrator/SYSTEM file permissions.
use crate::payload;
use std::path::Path;
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
/// Windows destinations may be owned by TrustedInstaller as well as SYSTEM/admins.
pub(crate) fn verify_system(path: &Path) -> Result<(), String> {
    verify_owner(path, true)
}
fn trusted(sid: &str, system: bool) -> bool {
    matches!(sid, "S-1-5-18" | "S-1-5-32-544")
        || (system && sid == "S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464")
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
        std::fs::remove_file(&path).unwrap();
        assert!(checked.is_err());
    }
}
