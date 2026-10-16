//! Protected product paths, ACLs, atomic JSON publication and the common operation lock.
use super::policy::enrollment;
use crate::{payload, windows_pipe};
use serde::Serialize;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use windows::{
    Win32::{
        Foundation::{HLOCAL, LocalFree},
        Security::Authorization::{
            ConvertStringSecurityDescriptorToSecurityDescriptorW, ConvertStringSidToSidW,
            SDDL_REVISION_1, SE_FILE_OBJECT, SetNamedSecurityInfoW,
        },
        Security::{
            DACL_SECURITY_INFORMATION, GetSecurityDescriptorDacl, OWNER_SECURITY_INFORMATION,
            PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID,
        },
        System::Com::CoTaskMemFree,
        UI::Shell::{
            FOLDERID_ProgramData, FOLDERID_ProgramFiles, KNOWN_FOLDER_FLAG, SHGetKnownFolderPath,
        },
    },
    core::PCWSTR,
};

#[allow(unsafe_code)]
pub(crate) fn folder(id: &windows::core::GUID) -> Result<PathBuf, String> {
    // SAFETY: known-folder GUID, current token, API-owned returned string.
    let p = unsafe { SHGetKnownFolderPath(id, KNOWN_FOLDER_FLAG(0), None) }
        .map_err(|e| e.to_string())?;
    // SAFETY: terminated result valid until CoTaskMemFree.
    let text = unsafe { p.to_string() }.map_err(|e| e.to_string());
    unsafe {
        CoTaskMemFree(Some(p.0.cast()));
    }
    text.map(PathBuf::from)
}
/// Discovered protected product installation path.
pub fn install_directory() -> Result<PathBuf, String> {
    Ok(folder(&FOLDERID_ProgramFiles)?.join("HyperGpuSupportProduct"))
}
/// Discovered administrator-owned product state path.
pub fn data_directory() -> Result<PathBuf, String> {
    Ok(folder(&FOLDERID_ProgramData)?.join("HyperGpuSupportProduct"))
}
#[allow(unsafe_code)]
pub(crate) fn protect(path: &Path, client: &str) -> Result<(), String> {
    payload::no_reparse(path)?;
    let sddl: Vec<_> = format!("D:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)(A;OICI;FRFX;;;{client})")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut descriptor = PSECURITY_DESCRIPTOR::default();
    // SAFETY: terminated SDDL and writable LocalAlloc descriptor output.
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            PCWSTR(sddl.as_ptr()),
            SDDL_REVISION_1,
            &mut descriptor,
            None,
        )
    }
    .map_err(|e| e.to_string())?;
    let mut owner = PSID::default();
    let admin: Vec<_> = "S-1-5-32-544".encode_utf16().chain(Some(0)).collect();
    // SAFETY: terminated trusted SID constant and LocalAlloc output.
    if let Err(e) = unsafe { ConvertStringSidToSidW(PCWSTR(admin.as_ptr()), &mut owner) } {
        unsafe {
            LocalFree(Some(HLOCAL(descriptor.0)));
        }
        return Err(e.to_string());
    }
    let result = (|| {
        let mut present = windows::core::BOOL(0);
        let mut defaulted = windows::core::BOOL(0);
        let mut acl = std::ptr::null_mut();
        // SAFETY: live security descriptor, writable ACL outputs.
        unsafe { GetSecurityDescriptorDacl(descriptor, &mut present, &mut acl, &mut defaulted) }
            .map_err(|e| e.to_string())?;
        let text: Vec<_> = path
            .as_os_str()
            .to_string_lossy()
            .encode_utf16()
            .chain(Some(0))
            .collect();
        // SAFETY: live ACL and terminated existing path; installs a protected DACL.
        unsafe {
            SetNamedSecurityInfoW(
                PCWSTR(text.as_ptr()),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION
                    | DACL_SECURITY_INFORMATION
                    | PROTECTED_DACL_SECURITY_INFORMATION,
                Some(owner),
                None,
                Some(acl),
                None,
            )
        }
        .ok()
        .map_err(|e| e.to_string())
    })();
    // SAFETY: release sole SDDL allocation after ACL is copied.
    unsafe {
        LocalFree(Some(HLOCAL(descriptor.0)));
        LocalFree(Some(HLOCAL(owner.0)));
    }
    result?;
    crate::security::verify(path)
}
pub(crate) fn atomic_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    crate::security::verify(path.parent().ok_or("state directory missing")?)?;
    let temp = path.with_extension("partial");
    remove_protected_leaf(&temp)?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let client = enrollment()
        .map(|e| e.client_sid)
        .or_else(|_| windows_pipe::current_user_sid_string().map_err(|e| e.to_string()))?;
    let mut file = crate::security::create_file(&temp, &client)?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())?;
    drop(file);
    crate::security::verify(&temp)?;
    if path.exists() {
        crate::security::verify(path)?;
    }
    crate::guest::replace_file(&temp, path)
}
pub(crate) fn operation_lock() -> Result<fs::File, String> {
    use std::os::windows::fs::OpenOptionsExt;
    let data = data_directory()?;
    crate::security::verify(&data)?;
    let path = data.join("operation.lock");
    match fs::symlink_metadata(&path) {
        Ok(_) => {
            crate::security::verify(&path)?;
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .share_mode(0)
                .open(path)
                .map_err(|e| format!("another host operation is active: {e}"))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let client = enrollment()
                .map(|e| e.client_sid)
                .or_else(|_| windows_pipe::current_user_sid_string().map_err(|e| e.to_string()))?;
            crate::security::create_file(&path, &client)
        }
        Err(error) => Err(error.to_string()),
    }
}
pub(super) fn remove_protected_leaf(path: &Path) -> Result<(), String> {
    if path.exists() {
        crate::security::verify(path)?;
        if !fs::metadata(path).map_err(|e| e.to_string())?.is_file() {
            return Err("protected temporary path is not a file".into());
        }
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
