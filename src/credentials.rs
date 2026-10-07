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
    if let Some(c) = read(vm)? {
        return Ok(c);
    }
    let caption = prompt_wide("Hyper GPU Support — guest credentials");
    let message = prompt_wide(
        "Guest administrator credentials. Remember saves them in your Windows Credential Manager for this VM only.",
    );
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
    unsafe {
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
    }
    .ok()
    .map_err(|_| "guest credential dialog cancelled or failed")?;
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
    if remember.as_bool() {
        store(vm, &credential)?;
    }
    Ok(credential)
}
