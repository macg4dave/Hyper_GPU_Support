//! Windows UAC relaunch, elevation check and remembered-credential cleanup.
#![allow(unsafe_code)]

use std::path::Path;
use windows::{
    Win32::{
        Foundation::{CloseHandle, ERROR_CANCELLED, ERROR_NOT_FOUND, HANDLE},
        Security::{
            Credentials::{CRED_TYPE_GENERIC, CREDENTIALW, CredDeleteW, CredEnumerateW, CredFree},
            GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
        },
        System::Threading::{
            GetCurrentProcess, GetExitCodeProcess, INFINITE, OpenProcessToken, WaitForSingleObject,
        },
        UI::{
            Shell::{SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW},
            WindowsAndMessaging::SW_SHOWNORMAL,
        },
    },
    core::{HRESULT, PCWSTR, w},
};

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

/// Relaunch `exe` through the UAC consent prompt and wait for its exit code.
pub fn run_elevated(exe: &Path, args: &[String]) -> Result<u32, String> {
    let file = wide(exe.to_str().ok_or("executable path is not valid Unicode")?);
    let parameters = wide(&super::native_command_line(args));
    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: w!("runas"),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(parameters.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    // SAFETY: terminated strings outlive the call; Windows fills hProcess.
    if let Err(error) = unsafe { ShellExecuteExW(&mut info) } {
        return Err(if error.code() == HRESULT::from_win32(ERROR_CANCELLED.0) {
            "administrator approval was declined; nothing was changed".into()
        } else {
            format!("UAC launch failed: {error}")
        });
    }
    let process = info.hProcess;
    if process.is_invalid() {
        return Err("UAC launch returned no process handle".into());
    }
    let mut code = 0;
    // SAFETY: owned process handle from SEE_MASK_NOCLOSEPROCESS, closed once.
    let result = unsafe {
        WaitForSingleObject(process, INFINITE);
        let result = GetExitCodeProcess(process, &mut code);
        let _ = CloseHandle(process);
        result
    };
    result.map_err(|e| e.to_string())?;
    Ok(code)
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
        let result = GetTokenInformation(
            token,
            TokenElevation,
            Some((&raw mut elevation).cast()),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut length,
        );
        let _ = CloseHandle(token);
        result
    };
    result.map_err(|e| e.to_string())?;
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
    let mut targets = Vec::new();
    // SAFETY: `count` valid entries; target names are terminated strings.
    unsafe {
        for index in 0..count as usize {
            let credential = &**list.add(index);
            if credential.Type == CRED_TYPE_GENERIC
                && let Ok(name) = credential.TargetName.to_string()
            {
                targets.push(name);
            }
        }
        CredFree(list.cast());
    }
    for target in &targets {
        let name = wide(target);
        // SAFETY: terminated target name and fixed generic credential type.
        unsafe { CredDeleteW(PCWSTR(name.as_ptr()), CRED_TYPE_GENERIC, None) }
            .map_err(|e| format!("delete credential {target}: {e}"))?;
    }
    Ok(targets)
}
