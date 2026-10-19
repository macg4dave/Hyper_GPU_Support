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
    prompt: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    if let Some(credential) = stored? {
        return Ok(credential);
    }
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
    match unsafe { CredDeleteW(PCWSTR(key.as_ptr()), CRED_TYPE_GENERIC, None) } {
        Ok(()) => Ok(()),
        Err(e) if e.code() == windows::Win32::Foundation::ERROR_NOT_FOUND.to_hresult() => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn prompt_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

/// Native credential-dialog text limited to `units` UTF-16 units including the terminator.
/// Catalogue wording is translatable, so overlong text is shortened at a character boundary
/// rather than making the native prompt fail.
fn prompt_wide_bounded(text: &str, units: usize) -> Vec<u16> {
    let mut wide = Vec::with_capacity(units);
    for c in text.chars() {
        if wide.len() + c.len_utf16() >= units {
            break;
        }
        let mut buffer = [0u16; 2];
        wide.extend_from_slice(c.encode_utf16(&mut buffer));
    }
    wide.push(0);
    wide
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
    prompt_if_needed(read(vm), || prompt_native(vm, name, parent, false))
}

/// Manage the selected VM's opt-in vault entry without connecting to the guest.
/// Cancel leaves the old entry intact; replacement always opens fresh input.
#[allow(unsafe_code)]
pub fn manage_named(
    vm: &str,
    name: &str,
    parent: windows::Win32::Foundation::HWND,
) -> Result<bool, String> {
    use windows::Win32::UI::WindowsAndMessaging::{
        IDCANCEL, IDNO, IDYES, MB_ICONQUESTION, MB_YESNOCANCEL, MessageBoxW,
    };
    key(vm)?;
    // Surface vault failure before offering a mutation. Do not prefill passwords.
    drop(read(vm)?);
    let name: String = name.chars().filter(|c| !c.is_control()).take(256).collect();
    let message = prompt_wide(&format!(
        "Guest credentials for {name}\nVM: {vm}\n\nYes: enter replacement credentials for the guest Windows account, not the host account. Remember must be checked to save the replacement; leaving it unchecked removes the previous saved entry after successful input.\nNo: forget this VM's saved credentials.\nCancel: retain the existing entry.\n\nNo guest connection or VM changes will be performed."
    ));
    let caption = prompt_wide("Manage guest credentials");
    // SAFETY: bounded, terminated strings and an optional native owner.
    match unsafe {
        MessageBoxW(
            Some(parent),
            PCWSTR(message.as_ptr()),
            PCWSTR(caption.as_ptr()),
            MB_YESNOCANCEL | MB_ICONQUESTION,
        )
    } {
        IDYES => {
            drop(prompt_native(vm, &name, parent, true)?);
        }
        IDNO => forget(vm)?,
        IDCANCEL => return Err(PROMPT_CANCELLED.into()),
        _ => return Err("Guest credential settings could not be opened.".into()),
    }
    Ok(read(vm)?.is_some())
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
fn with_prompt_owner<T>(
    parent: windows::Win32::Foundation::HWND,
    show: impl FnOnce(windows::Win32::Foundation::HWND) -> Result<T, String>,
) -> Result<T, String> {
    use windows::Win32::{
        Foundation::{HWND, RECT},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, GWL_EXSTYLE, GetDesktopWindow, GetForegroundWindow,
            GetWindowLongPtrW, GetWindowRect, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
            SetWindowPos, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
        },
    };

    // CredUI inherits its owner's topmost status. Keep this hidden window on the
    // prompt thread for the modal call, without making the application topmost.
    struct PromptOwner(HWND);
    impl Drop for PromptOwner {
        fn drop(&mut self) {
            // SAFETY: this guard owns the hidden window on its creating thread;
            // CredUI has returned before the guard is released.
            if let Err(error) = unsafe { DestroyWindow(self.0) } {
                eprintln!("Guest sign-in owner cleanup failed: {error}");
            }
        }
    }
    let parent = (!parent.is_invalid()).then_some(parent);
    // Use foreground geometry only for placement; it grants no ownership or authority.
    // SAFETY: these queries borrow existing windows without changing their state.
    let placement = parent.unwrap_or_else(|| unsafe { GetForegroundWindow() });
    let placement = if placement.is_invalid() {
        // SAFETY: borrow the desktop window for default dialog placement.
        unsafe { GetDesktopWindow() }
    } else {
        placement
    };
    let mut bounds = RECT::default();
    // SAFETY: borrowed window and writable RECT valid for the synchronous query.
    unsafe { GetWindowRect(placement, &mut bounds) }
        .map_err(|error| format!("Guest sign-in placement failed: {error}"))?;
    let width = bounds
        .right
        .checked_sub(bounds.left)
        .filter(|width| *width >= 0)
        .ok_or("Guest sign-in owner width is invalid.")?;
    let height = bounds
        .bottom
        .checked_sub(bounds.top)
        .filter(|height| *height >= 0)
        .ok_or("Guest sign-in owner height is invalid.")?;
    // SAFETY: predefined STATIC class, no visible style or custom procedure, finite
    // rectangle and optional borrowed owner. The returned window is owned by the guard.
    let owner = PromptOwner(
        unsafe {
            CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                windows::core::w!("STATIC"),
                windows::core::w!(""),
                WS_POPUP,
                bounds.left,
                bounds.top,
                width,
                height,
                parent,
                None,
                None,
                None,
            )
        }
        .map_err(|error| format!("Guest sign-in owner could not be created: {error}"))?,
    );
    // SAFETY: owned live window; establish its z-order without showing, moving,
    // resizing or activating it. Its owned credential dialog inherits topmost.
    unsafe {
        SetWindowPos(
            owner.0,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
        )
    }
    .map_err(|error| format!("Guest sign-in could not be made topmost: {error}"))?;
    // SAFETY: query the extended style of this same owned live window. Do not
    // open credential input if the required topmost ownership was not established.
    if unsafe { GetWindowLongPtrW(owner.0, GWL_EXSTYLE) } & WS_EX_TOPMOST.0 as isize == 0 {
        return Err("Guest sign-in owner is not topmost.".into());
    }
    show(owner.0)
}

#[allow(unsafe_code)]
fn prompt_native(
    vm: &str,
    name: &str,
    parent: windows::Win32::Foundation::HWND,
    replacing: bool,
) -> Result<Credential, String> {
    with_prompt_owner(parent, |parent| {
        // CREDUI_MAX_CAPTION_LENGTH and CREDUI_MAX_MESSAGE_LENGTH.
        let caption = prompt_wide_bounded(crate::ui_text::credentials::native::caption(), 128);
        let message = prompt_wide_bounded(guest_prompt_message(vm, name).as_str(), 1024);
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
            password: String::from_utf16(&password[..password_end])
                .map_err(|_| "invalid password")?,
        };
        finish_input(
            Ok((credential, remember.as_bool())),
            replacing,
            |credential| store(vm, credential),
            || forget(vm),
        )
    })
}

fn finish_input<T>(
    input: Result<(T, bool), String>,
    replacing: bool,
    store: impl FnOnce(&T) -> Result<(), String>,
    forget: impl FnOnce() -> Result<(), String>,
) -> Result<T, String> {
    let (credential, remember) = input?;
    if remember {
        store(&credential)?;
    } else if replacing {
        forget()?;
    }
    Ok(credential)
}

#[cfg(test)]
fn retain_if_requested(
    credential: Credential,
    remember: bool,
    store: impl FnOnce(&Credential) -> Result<(), String>,
) -> Result<Credential, String> {
    finish_input(Ok((credential, remember)), false, store, || Ok(()))
}

fn guest_prompt_message(_vm: &str, name: &str) -> crate::ui_text::UiMessage {
    let name: String = name.chars().filter(|c| !c.is_control()).take(256).collect();
    crate::ui_text::credentials::guest_sign_in::message(&name)
}

#[cfg(test)]
mod prompt_tests {
    use super::*;

    #[test]
    fn native_prompt_owner_is_topmost_and_preserves_success_or_cancellation() {
        // Only hidden native owner windows are created: no credential prompt,
        // vault access, guest connection or VM operation occurs in this test.
        for result in [Ok(7), Err(PROMPT_CANCELLED.to_owned())] {
            let mut calls = 0;
            let returned =
                with_prompt_owner(windows::Win32::Foundation::HWND::default(), |owner| {
                    assert!(!owner.is_invalid());
                    calls += 1;
                    result.clone()
                });
            assert_eq!(returned, result);
            assert_eq!(calls, 1);
        }
    }

    #[test]
    fn native_prompt_text_is_bounded_at_a_character_boundary() {
        assert_eq!(prompt_wide_bounded("ab", 8), [97, 98, 0]);
        let long = "x".repeat(2000);
        assert_eq!(prompt_wide_bounded(&long, 1024).len(), 1024);
        // A surrogate pair is never split: '😀' needs two units, leaving room only for 'a'.
        assert_eq!(prompt_wide_bounded("a😀", 3), [97, 0]);
        assert_eq!(*prompt_wide_bounded(&long, 128).last().unwrap(), 0);
    }

    #[test]
    fn settings_input_replaces_or_forgets_only_after_successful_input() {
        for remember in [false, true] {
            let events = std::cell::RefCell::new(Vec::new());
            finish_input(
                Ok(("replacement", remember)),
                true,
                |value| {
                    assert_eq!(*value, "replacement");
                    events.borrow_mut().push("replace");
                    Ok(())
                },
                || {
                    events.borrow_mut().push("forget");
                    Ok(())
                },
            )
            .unwrap();
            assert_eq!(
                *events.borrow(),
                [if remember { "replace" } else { "forget" }]
            );
        }
        for error in [PROMPT_CANCELLED, "Vault unavailable"] {
            assert_eq!(
                finish_input::<()>(
                    Err(error.into()),
                    true,
                    |_| panic!("Cancelled input must retain entry"),
                    || panic!("Cancelled input must retain entry")
                )
                .unwrap_err(),
                error
            );
        }
        for remember in [false, true] {
            let result = finish_input(
                Ok(((), remember)),
                true,
                |_| Err("Vault write failed".into()),
                || Err("Vault delete failed".into()),
            );
            assert_eq!(
                result.unwrap_err(),
                if remember {
                    "Vault write failed"
                } else {
                    "Vault delete failed"
                }
            );
        }
    }

    #[test]
    fn native_name_bound_and_control_filter_precede_catalogue_insertion() {
        let prepared = "😀".repeat(256);
        let input = format!("\0\n\t{}", "😀".repeat(257));
        let actual = guest_prompt_message("bound-id", &input);
        let expected = crate::ui_text::credentials::guest_sign_in::message(&prepared);
        assert_eq!(actual.as_str(), expected.as_str());
        assert!(!actual.as_str().contains('\0'));
    }

    #[test]
    fn input_opens_directly_before_guest_work_for_the_bound_vm() {
        let events = std::cell::RefCell::new(Vec::new());
        let vm = "reviewed-vm";
        let credential = prompt_if_needed(Ok(None), || {
            events.borrow_mut().push(("prompt", vm));
            Ok("ephemeral-input")
        })
        .unwrap();
        assert_eq!(credential, "ephemeral-input");
        events.borrow_mut().push(("guest-work", vm));
        assert_eq!(*events.borrow(), [("prompt", vm), ("guest-work", vm)]);
    }

    #[test]
    fn cancelled_or_failed_input_prevents_guest_work() {
        for error in [PROMPT_CANCELLED, "Guest credential dialog failed."] {
            let mut prompts = 0;
            let result = prompt_if_needed::<()>(Ok(None), || {
                prompts += 1;
                Err(error.into())
            });
            let mut guest_work = false;
            let result = result.map(|()| guest_work = true);
            assert_eq!(result.unwrap_err(), error);
            assert_eq!(prompts, 1);
            assert!(!guest_work);
            let edited_label = String::from_iter(['λ', '\n', '{', '}']);
            assert!(!is_prompt_cancelled(&edited_label));
            assert!(is_prompt_cancelled(PROMPT_CANCELLED));
        }
    }

    #[test]
    fn stored_opt_in_skips_input_and_vault_errors_do_not_prompt() {
        let stored = prompt_if_needed(Ok(Some("stored-input")), || {
            panic!("No prompt when stored input is available")
        });
        assert_eq!(stored.unwrap(), "stored-input");
        let error = prompt_if_needed::<()>(Err("Vault unavailable".into()), || {
            panic!("Vault failure must not fall back to input")
        });
        assert_eq!(error.unwrap_err(), "Vault unavailable");
    }

    #[test]
    fn native_cancel_is_distinct_from_dialog_failure_and_stops_guest_work() {
        use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_CANCELLED, ERROR_SUCCESS};
        let result = prompt_if_needed(Ok(None), || prompt_status(ERROR_CANCELLED));
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
        let text = guest_prompt_message("bound-id", "guest\0\n\tname").into_string();
        assert!(text.contains("guestname"));
        assert!(!text.contains("bound-id"));
        assert!(!text.contains('\0'));
    }
}
