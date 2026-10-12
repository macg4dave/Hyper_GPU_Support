//! Per-user, per-VM Windows Credential Manager integration.
use serde::{Deserialize, Serialize};
use windows::Win32::Security::Credentials::{
    CREDUI_FLAGS, CREDUI_FLAGS_ALWAYS_SHOW_UI, CREDUI_FLAGS_DO_NOT_PERSIST,
    CREDUI_FLAGS_GENERIC_CREDENTIALS, CREDUI_FLAGS_SHOW_SAVE_CHECK_BOX, CREDUI_INFOW,
    CredUIPromptForCredentialsW,
};
use windows::{
    Win32::Security::Credentials::{
        CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC, CREDENTIALW, CredDeleteW, CredFree,
        CredReadW, CredWriteW,
    },
    core::{PCWSTR, PWSTR},
};
use zeroize::Zeroize;

const PROMPT_CANCELLED: &str =
    "Guest credential request cancelled. No guest operation was started.";

/// Distinguish local cancellation from a credential or operation failure.
pub fn is_prompt_cancelled(error: &str) -> bool {
    error == PROMPT_CANCELLED
}

fn prompt_if_needed<T>(
    stored: Result<Option<T>, String>,
    explain: impl FnOnce() -> Result<(), String>,
    prompt: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    if let Some(credential) = stored? {
        return Ok(credential);
    }
    explain()?;
    prompt()
}

/// Ephemeral guest credentials. Debug intentionally omits credential data entirely.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Credential {
    /// Guest account name.
    pub username: String,
    /// Guest password, zeroized on drop.
    pub password: String,
}
impl Drop for Credential {
    fn drop(&mut self) {
        self.password.zeroize();
    }
}
fn key(vm: &str) -> Result<Vec<u16>, String> {
    if vm.len() != 36 || !vm.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-') {
        return Err("invalid credential VM identity".into());
    }
    Ok(format!("HyperGpuSupport/guest/{}", vm.to_ascii_lowercase())
        .encode_utf16()
        .chain(Some(0))
        .collect())
}
/// Read an opt-in credential from the current user's Windows vault.
#[allow(unsafe_code)]
pub fn read(vm: &str) -> Result<Option<Credential>, String> {
    let key = key(vm)?;
    let mut raw = std::ptr::null_mut();
    // SAFETY: terminated target and initialized output pointer owned by CredFree.
    if let Err(e) = unsafe { CredReadW(PCWSTR(key.as_ptr()), CRED_TYPE_GENERIC, None, &mut raw) } {
        if e.code() == windows::Win32::Foundation::ERROR_NOT_FOUND.to_hresult() {
            return Ok(None);
        }
        return Err(e.to_string());
    }
    let result = (|| {
        // SAFETY: successful CredReadW returns a live CREDENTIALW and bounded blob.
        let c = unsafe { &*raw };
        if c.CredentialBlobSize > 5120 || c.CredentialBlobSize % 2 != 0 {
            return Err("invalid stored credential".into());
        }
        let bytes = if c.CredentialBlobSize == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(c.CredentialBlob, c.CredentialBlobSize as usize) }
        };
        let utf16 = zeroize::Zeroizing::new(
            bytes
                .chunks_exact(2)
                .map(|p| u16::from_le_bytes([p[0], p[1]]))
                .collect::<Vec<_>>(),
        );
        // SAFETY: Username is terminated and remains owned by the returned credential.
        Ok(Some(Credential {
            username: unsafe { c.UserName.to_string() }.map_err(|e| e.to_string())?,
            password: String::from_utf16(&utf16).map_err(|_| "invalid credential encoding")?,
        }))
    })();
    // SAFETY: sole successful CredReadW allocation, released after copying.
    unsafe {
        if (*raw).CredentialBlobSize > 0 {
            std::slice::from_raw_parts_mut(
                (*raw).CredentialBlob,
                (*raw).CredentialBlobSize as usize,
            )
            .zeroize();
        }
        CredFree(raw.cast());
    }
    result
}
/// Store only after an explicit user opt-in.
#[allow(unsafe_code)]
pub fn store(vm: &str, credential: &Credential) -> Result<(), String> {
    let mut key = key(vm)?;
    let mut user: Vec<_> = credential.username.encode_utf16().chain(Some(0)).collect();
    if credential.username.contains('\0') || credential.password.len() > 2560 {
        return Err("invalid guest credential".into());
    }
    let mut blob = zeroize::Zeroizing::new(
        credential
            .password
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>(),
    );
    let native = CREDENTIALW {
        Type: CRED_TYPE_GENERIC,
        TargetName: PWSTR(key.as_mut_ptr()),
        UserName: PWSTR(user.as_mut_ptr()),
        CredentialBlobSize: blob.len() as u32,
        CredentialBlob: blob.as_mut_ptr(),
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        ..Default::default()
    };
    // SAFETY: live terminated strings and finite blob; CredWriteW copies all inputs.
    unsafe { CredWriteW(&native, 0) }.map_err(|e| e.to_string())
}
/// Remove the current user's stored credential for one VM.
#[allow(unsafe_code)]
pub fn forget(vm: &str) -> Result<(), String> {
    let key = key(vm)?;
    // SAFETY: terminated target, fixed generic credential type.
    unsafe { CredDeleteW(PCWSTR(key.as_ptr()), CRED_TYPE_GENERIC, None) }.map_err(|e| e.to_string())
}

fn prompt_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
/// Obtain VM credentials through the native Windows dialog with explicit optional storage.
/// Credentials are scoped to the current user and VM and never passed in command arguments.
#[allow(unsafe_code)]
pub fn prompt(vm: &str, parent: windows::Win32::Foundation::HWND) -> Result<Credential, String> {
    prompt_named(vm, vm, parent)
}

/// Prompt for one VM's guest account, naming it without changing its GUID vault key.
#[allow(unsafe_code)]
pub fn prompt_named(
    vm: &str,
    name: &str,
    parent: windows::Win32::Foundation::HWND,
) -> Result<Credential, String> {
    prompt_if_needed(
        read(vm),
        || explain_guest_sign_in(vm, name, parent),
        || prompt_native(vm, name, parent),
    )
}

#[allow(unsafe_code)]
fn explain_guest_sign_in(
    vm: &str,
    name: &str,
    parent: windows::Win32::Foundation::HWND,
) -> Result<(), String> {
    use windows::Win32::UI::WindowsAndMessaging::{
        IDCANCEL, IDOK, MB_ICONINFORMATION, MB_OKCANCEL, MessageBoxW,
    };
    let caption = prompt_wide("Sign in to your virtual machine");
    let message = prompt_wide(&format!(
        "{}\n\nChoose OK to enter your details. Cancel returns to the app.",
        guest_prompt_message(vm, name),
    ));
    // SAFETY: live terminated strings, optional dialog owner; no credentials are collected here.
    let result = unsafe {
        MessageBoxW(
            Some(parent),
            PCWSTR(message.as_ptr()),
            PCWSTR(caption.as_ptr()),
            MB_OKCANCEL | MB_ICONINFORMATION,
        )
    };
    match result {
        IDOK => Ok(()),
        IDCANCEL => Err(PROMPT_CANCELLED.into()),
        _ => Err("Guest sign-in explanation could not be opened.".into()),
    }
}

fn prompt_status(status: windows::Win32::Foundation::WIN32_ERROR) -> Result<(), String> {
    if status == windows::Win32::Foundation::ERROR_CANCELLED {
        return Err(PROMPT_CANCELLED.into());
    }
    status
        .ok()
        .map_err(|_| "Guest credential dialog failed.".into())
}

#[allow(unsafe_code)]
fn prompt_native(
    vm: &str,
    name: &str,
    parent: windows::Win32::Foundation::HWND,
) -> Result<Credential, String> {
    let caption = prompt_wide("Hyper GPU Support - Sign in");
    let message = prompt_wide(&guest_prompt_message(vm, name));
    let target = prompt_wide(&format!("HyperGpuSupport/guest/{vm}"));
    let info = CREDUI_INFOW {
        cbSize: std::mem::size_of::<CREDUI_INFOW>() as u32,
        hwndParent: parent,
        pszCaptionText: PCWSTR(caption.as_ptr()),
        pszMessageText: PCWSTR(message.as_ptr()),
        ..Default::default()
    };
    let mut username = [0u16; 514];
    let mut password = zeroize::Zeroizing::new([0u16; 256]);
    let mut remember = windows::core::BOOL(0);
    // SAFETY: live dialog owner, bounded mutable credential buffers, no automatic persistence.
    let status = unsafe {
        CredUIPromptForCredentialsW(
            Some(&info),
            PCWSTR(target.as_ptr()),
            None,
            0,
            &mut username,
            &mut password[..],
            Some(&mut remember),
            CREDUI_FLAGS(
                CREDUI_FLAGS_GENERIC_CREDENTIALS.0
                    | CREDUI_FLAGS_ALWAYS_SHOW_UI.0
                    | CREDUI_FLAGS_DO_NOT_PERSIST.0
                    | CREDUI_FLAGS_SHOW_SAVE_CHECK_BOX.0,
            ),
        )
    };
    prompt_status(status)?;
    let user_end = username
        .iter()
        .position(|c| *c == 0)
        .unwrap_or(username.len());
    let password_end = password
        .iter()
        .position(|c| *c == 0)
        .unwrap_or(password.len());
    let credential = Credential {
        username: String::from_utf16(&username[..user_end]).map_err(|_| "invalid username")?,
        password: String::from_utf16(&password[..password_end]).map_err(|_| "invalid password")?,
    };
    retain_if_requested(credential, remember.as_bool(), |credential| {
        store(vm, credential)
    })
}

fn retain_if_requested(
    credential: Credential,
    remember: bool,
    store: impl FnOnce(&Credential) -> Result<(), String>,
) -> Result<Credential, String> {
    if remember {
        store(&credential)?;
    }
    Ok(credential)
}

fn guest_prompt_message(_vm: &str, name: &str) -> String {
    let name: String = name.chars().filter(|c| !c.is_control()).take(256).collect();
    format!(
        "Sign in to {name}\n\nEnter the Windows administrator username and password you use inside this virtual machine. The app uses them for the guest steps you approve in review.\n\nRemember is optional. Leave it unchecked to use your details just this once; selecting it saves them securely on this computer for this VM."
    )
}

#[cfg(test)]
mod prompt_tests {
    use super::*;

    #[test]
    fn prompt_names_the_guest_and_explains_which_account_is_needed() {
        let text = super::guest_prompt_message("vm-guid", "test-001");
        for expected in [
            "test-001",
            "username and password",
            "Windows administrator",
            "guest steps you approve in review",
            "Remember is optional. Leave it unchecked",
        ] {
            assert!(text.contains(expected));
        }
        assert!(!text.contains("vm-guid"));
        assert!(!text.contains("set up GPU support"));
        assert!(text.len() < 400);
    }

    #[test]
    fn explanation_precedes_input_and_guest_work_for_the_bound_vm() {
        let events = std::cell::RefCell::new(Vec::new());
        let vm = "reviewed-vm";
        let credential = prompt_if_needed(
            Ok(None),
            || {
                events.borrow_mut().push(("explain", vm));
                Ok(())
            },
            || {
                events.borrow_mut().push(("prompt", vm));
                Ok("ephemeral-input")
            },
        )
        .unwrap();
        assert_eq!(credential, "ephemeral-input");
        events.borrow_mut().push(("guest-work", vm));
        assert_eq!(
            *events.borrow(),
            [("explain", vm), ("prompt", vm), ("guest-work", vm)]
        );
    }

    #[test]
    fn cancellation_or_explanation_failure_prevents_input_and_guest_work() {
        for error in [PROMPT_CANCELLED, "Explanation unavailable"] {
            let result = prompt_if_needed::<()>(
                Ok(None),
                || Err(error.into()),
                || panic!("Input must not open after cancelled/failed explanation"),
            );
            let mut guest_work = false;
            let result = result.map(|()| guest_work = true);
            assert_eq!(result.unwrap_err(), error);
            assert!(!guest_work);
        }
    }

    #[test]
    fn stored_opt_in_skips_both_panels_and_vault_errors_do_not_prompt() {
        let stored = prompt_if_needed(
            Ok(Some("stored-input")),
            || panic!("No explanation without a prompt"),
            || panic!("No prompt when stored input is available"),
        );
        assert_eq!(stored.unwrap(), "stored-input");
        let error = prompt_if_needed::<()>(
            Err("Vault unavailable".into()),
            || panic!("Vault failure must be reported"),
            || panic!("Vault failure must not fall back to input"),
        );
        assert_eq!(error.unwrap_err(), "Vault unavailable");
    }

    #[test]
    fn native_cancel_is_distinct_from_dialog_failure_and_stops_guest_work() {
        use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_CANCELLED, ERROR_SUCCESS};
        let result = prompt_if_needed(Ok(None), || Ok(()), || prompt_status(ERROR_CANCELLED));
        let mut guest_work = false;
        let result = result.map(|()| guest_work = true);
        assert!(is_prompt_cancelled(&result.unwrap_err()));
        assert!(!guest_work);
        assert!(!is_prompt_cancelled(
            &prompt_status(ERROR_ACCESS_DENIED).unwrap_err()
        ));
        assert!(prompt_status(ERROR_SUCCESS).is_ok());
    }

    fn test_credential() -> Credential {
        Credential {
            username: "test-account".into(),
            password: "test-input".into(),
        }
    }

    #[test]
    fn remembering_requires_explicit_opt_in_and_reports_storage_failure() {
        let ephemeral = retain_if_requested(test_credential(), false, |_| {
            panic!("Unchecked Remember must not write to the vault")
        })
        .unwrap();
        assert_eq!(ephemeral.username, "test-account");
        let mut writes = 0;
        let retained = retain_if_requested(test_credential(), true, |credential| {
            assert_eq!(credential.username, "test-account");
            writes += 1;
            Ok(())
        });
        assert!(retained.is_ok());
        assert_eq!(writes, 1);
        let failed =
            retain_if_requested(test_credential(), true, |_| Err("Vault unavailable".into()));
        assert_eq!(failed.err().unwrap(), "Vault unavailable");
    }

    #[test]
    fn vm_display_name_cannot_inject_lines_or_terminate_native_text() {
        let text = guest_prompt_message("bound-id", "guest\0\n\tname");
        assert!(text.starts_with("Sign in to guestname\n"));
        assert!(!text.contains("bound-id"));
        assert!(!text.contains('\0'));
    }
}
