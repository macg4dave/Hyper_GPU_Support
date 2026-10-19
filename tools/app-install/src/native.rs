//! Narrow Windows identity, known-folder and credential operations.
#![allow(unsafe_code)]

use std::path::{Path, PathBuf};
use windows::{
    Win32::{
        Foundation::{CloseHandle, ERROR_CANCELLED, ERROR_NOT_FOUND, HANDLE, WAIT_OBJECT_0},
        Security::{
            Credentials::{CRED_TYPE_GENERIC, CREDENTIALW, CredDeleteW, CredEnumerateW, CredFree},
            GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
        },
        Storage::FileSystem::{FILE_ATTRIBUTE_READONLY, GetFileAttributesW, SetFileAttributesW},
        System::{
            Com::CoTaskMemFree,
            Threading::{
                GetCurrentProcess, GetExitCodeProcess, INFINITE, OpenProcessToken,
                WaitForSingleObject,
            },
        },
        UI::Shell::{
            FOLDERID_LocalAppData, FOLDERID_ProgramData, FOLDERID_ProgramFiles, FOLDERID_System,
            KF_FLAG_DEFAULT, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, SHGetKnownFolderPath,
            ShellExecuteExW,
        },
        UI::WindowsAndMessaging::SW_SHOWNORMAL,
    },
    core::{GUID, HRESULT, PCWSTR, w},
};

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

/// Ask the Windows shell to open Setup normally. Its bootstrapper owns UAC and
/// preserves the original user for Inno's ExecAsOriginalUser callbacks.
pub(super) fn run_setup(executable: &Path) -> Result<u32, String> {
    let file = wide(
        executable
            .to_str()
            .ok_or("installer path is not valid Unicode")?,
    );
    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: w!("open"),
        lpFile: PCWSTR(file.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    // SAFETY: terminated path lives through the call; structure has the required
    // size and Windows returns a process handle owned by this function.
    unsafe { ShellExecuteExW(&mut info) }.map_err(|error| {
        if error.code() == HRESULT::from_win32(ERROR_CANCELLED.0) {
            "administrator approval was declined; installation was cancelled".into()
        } else {
            format!("start Setup through Windows shell: {error}")
        }
    })?;
    if info.hProcess.is_invalid() {
        return Err("Windows returned no Setup process handle".into());
    }
    let mut code = 0;
    // SAFETY: owned valid process handle; output points to an initialized u32.
    let result = unsafe {
        if WaitForSingleObject(info.hProcess, INFINITE) == WAIT_OBJECT_0 {
            GetExitCodeProcess(info.hProcess, &mut code).map_err(|error| error.to_string())
        } else {
            Err(format!(
                "wait for Setup: {}",
                std::io::Error::last_os_error()
            ))
        }
    };
    // SAFETY: process handle is owned here and is closed once after waiting.
    let closed = unsafe { CloseHandle(info.hProcess) }.map_err(|error| error.to_string());
    result?;
    closed?;
    Ok(code)
}

pub(super) enum Folder {
    ProgramFiles,
    ProgramData,
    LocalAppData,
    System,
}

/// Resolve fixed Windows folders without trusting caller-supplied environment paths.
pub(super) fn folder(folder: Folder) -> Result<PathBuf, String> {
    let id: &GUID = match folder {
        Folder::ProgramFiles => &FOLDERID_ProgramFiles,
        Folder::ProgramData => &FOLDERID_ProgramData,
        Folder::LocalAppData => &FOLDERID_LocalAppData,
        Folder::System => &FOLDERID_System,
    };
    // SAFETY: fixed known-folder GUID, current-user token, returned allocation
    // stays live through conversion and is released exactly once with CoTaskMemFree.
    let path = unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None) }
        .map_err(|error| format!("resolve Windows known folder: {error}"))?;
    // SAFETY: successful SHGetKnownFolderPath returned a terminated UTF-16 string.
    let text = unsafe { path.to_string() }.map_err(|error| error.to_string());
    // SAFETY: this is the allocation returned above; conversion no longer borrows it.
    unsafe { CoTaskMemFree(Some(path.0.cast())) };
    text.map(PathBuf::from)
}

/// Elevated helpers never search a caller-controlled working directory or PATH.
pub(super) fn system_executable(name: &str) -> Result<PathBuf, String> {
    let system = folder(Folder::System)?;
    match name {
        "powershell.exe" => Ok(system.join(r"WindowsPowerShell\v1.0\powershell.exe")),
        "tasklist.exe" | "takeown.exe" | "icacls.exe" => Ok(system.join(name)),
        _ => Err(format!("unsupported system helper {name}")),
    }
}

pub(super) fn clear_readonly(path: &Path) -> Result<(), String> {
    let path = wide(path.to_str().ok_or("cleanup path is not valid Unicode")?);
    // SAFETY: terminated path outlives each call; attributes are a value bitmask.
    let attributes = unsafe { GetFileAttributesW(PCWSTR(path.as_ptr())) };
    if attributes == u32::MAX {
        return Err(format!(
            "read file attributes: {}",
            std::io::Error::last_os_error()
        ));
    }
    // SAFETY: same valid path, preserving every attribute except readonly.
    unsafe {
        SetFileAttributesW(
            PCWSTR(path.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(
                attributes & !FILE_ATTRIBUTE_READONLY.0,
            ),
        )
    }
    .map_err(|error| format!("clear readonly attribute: {error}"))
}

/// Whether this process token is elevated.
pub fn is_elevated() -> Result<bool, String> {
    let mut token = HANDLE::default();
    // SAFETY: pseudo-handle for this process and a writable token output.
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }
        .map_err(|e| e.to_string())?;
    let mut elevation = TOKEN_ELEVATION::default();
    let mut length = 0;
    // SAFETY: live token; output buffer is exactly one TOKEN_ELEVATION.
    let result = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            Some((&raw mut elevation).cast()),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut length,
        )
    };
    // SAFETY: token was opened above and is closed exactly once.
    let closed = unsafe { CloseHandle(token) };
    result.map_err(|e| e.to_string())?;
    closed.map_err(|error| error.to_string())?;
    Ok(elevation.TokenIsElevated != 0)
}

/// Delete the current user's generic vault credentials under `prefix`.
pub fn delete_credentials(prefix: &str) -> Result<Vec<String>, String> {
    let filter = wide(&format!("{prefix}*"));
    let mut count = 0;
    let mut list: *mut *mut CREDENTIALW = std::ptr::null_mut();
    // SAFETY: terminated filter; Windows allocates the list, freed below.
    if let Err(error) =
        unsafe { CredEnumerateW(PCWSTR(filter.as_ptr()), None, &mut count, &mut list) }
    {
        return if error.code() == HRESULT::from_win32(ERROR_NOT_FOUND.0) {
            Ok(vec![])
        } else {
            Err(format!("credential listing failed: {error}"))
        };
    }
    let mut targets = Ok(Vec::new());
    // SAFETY: `count` valid entries; target names are terminated strings.
    unsafe {
        for index in 0..count as usize {
            let credential = &**list.add(index);
            if credential.Type == CRED_TYPE_GENERIC {
                match credential.TargetName.to_string() {
                    Ok(name) => {
                        if let Ok(targets) = &mut targets {
                            targets.push(name);
                        }
                    }
                    Err(error) => {
                        targets = Err(format!("read remembered credential name: {error}"));
                        break;
                    }
                }
            }
        }
        CredFree(list.cast());
    }
    let targets = targets?;
    for target in &targets {
        let name = wide(target);
        // SAFETY: terminated target name and fixed generic credential type.
        unsafe { CredDeleteW(PCWSTR(name.as_ptr()), CRED_TYPE_GENERIC, None) }
            .map_err(|e| format!("delete credential {target}: {e}"))?;
    }
    Ok(targets)
}
