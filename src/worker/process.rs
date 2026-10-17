//! Fixed worker/setup launch, trusted artifacts and retained process identity.
use super::runner;
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        Security::Cryptography::{BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom},
        UI::{
            Shell::{
                SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW,
            },
            WindowsAndMessaging::SW_HIDE,
        },
    },
    core::PCWSTR,
};

pub(crate) struct OwnedProcess(pub(crate) HANDLE);
impl Drop for OwnedProcess {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: owns only its retained process handle; never terminates the process.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
pub(crate) fn valid_session(session: &str) -> bool {
    session.len() == 32
        && session
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
pub(crate) fn pipe_name(session: &str) -> Result<String, String> {
    if !valid_session(session) {
        return Err("invalid worker session identity".into());
    }
    Ok(format!(r"\\.\pipe\HyperGpuSupport.Operation.{session}"))
}
#[allow(unsafe_code)]
pub(crate) fn random_session() -> Result<String, String> {
    let mut bytes = [0_u8; 16];
    // SAFETY: fixed initialized output; uses Windows system-preferred cryptographic RNG.
    unsafe { BCryptGenRandom(None, &mut bytes, BCRYPT_USE_SYSTEM_PREFERRED_RNG) }
        .ok()
        .map_err(|e| e.to_string())?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
pub(super) fn trusted_executable() -> Result<std::path::PathBuf, String> {
    crate::diagnostics::run(
        "worker-availability",
        "fixed-product",
        trusted_executable_inner,
    )
}

fn trusted_executable_inner() -> Result<std::path::PathBuf, String> {
    let enrollment = runner::enrollment()?;
    runner::verify_artifacts(&enrollment)?;
    let path = runner::install_directory()?.join("hyper-gpu-support.exe");
    crate::security::verify(&path)?;
    if crate::payload::hash_file(&path)?
        != *enrollment
            .artifacts
            .get("hyper-gpu-support.exe")
            .ok_or("restricted worker is not installed; update the product installation")?
    {
        return Err("installed worker artifact changed".into());
    }
    Ok(path)
}
/// Report whether the protected, enrolled worker artifact is currently usable.
pub fn available() -> Result<(), String> {
    trusted_executable().map(|_| ())
}

#[allow(unsafe_code)]
pub(super) fn launch(session: &str) -> Result<OwnedProcess, String> {
    let executable = trusted_executable()?;
    launch_fixed(&executable, session, "--internal-worker")
}

pub(crate) fn launch_setup(session: &str) -> Result<OwnedProcess, String> {
    let executable = crate::setup::source_executable()?;
    launch_fixed(&executable, session, "--internal-setup")
}

#[allow(unsafe_code)]
fn launch_fixed(
    executable: &std::path::Path,
    session: &str,
    mode: &str,
) -> Result<OwnedProcess, String> {
    if !valid_session(session) || !matches!(mode, "--internal-worker" | "--internal-setup") {
        return Err("invalid fixed worker launch".into());
    }
    let verb = wide("runas");
    let path: Vec<u16> = executable
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let parameters = wide(&format!("{mode} {session} {}", std::process::id()));
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(path.as_ptr()),
        lpParameters: PCWSTR(parameters.as_ptr()),
        nShow: SW_HIDE.0,
        ..Default::default()
    };
    crate::work_counts::add(crate::work_counts::Counter::ProcessStarts, 1);
    // SAFETY: fixed installed-worker or same-package setup executable; callers
    // select one fixed internal mode and bounded numeric/session arguments.
    unsafe { ShellExecuteExW(&mut info) }
        .map_err(|e| format!("worker elevation refused or unavailable: {e}"))?;
    if info.hProcess.is_invalid() {
        return Err("worker launch returned no process identity".into());
    }
    Ok(OwnedProcess(info.hProcess))
}

#[cfg(test)]
mod tests {
    use super::super::protocol::Request;
    use super::*;

    #[test]
    fn restricted_session_refuses_paths_and_arbitrary_modes() {
        for value in ["", "../escape", "a".repeat(33).as_str(), "123"] {
            assert!(pipe_name(value).is_err());
        }
        assert!(pipe_name("0123456789abcdef0123456789abcdef").is_ok());
        assert!(
            serde_json::from_str::<Request>(
                r#"{"schema":1,"session":"x","command":{"Shell":"whoami"},"credential":null}"#
            )
            .is_err()
        );
    }
}
