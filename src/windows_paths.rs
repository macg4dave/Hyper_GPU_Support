//! Discovery of trusted executables and directories owned by Windows.

use std::ffi::OsString;
use std::io;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use windows::Win32::System::SystemInformation::GetWindowsDirectoryW;

const INITIAL_WINDOWS_DIRECTORY_CAPACITY: usize = 260;

/// Return the Windows installation directory reported by the operating system.
///
/// # Errors
/// Returns the last Windows error when the directory cannot be queried or an
/// invalid length is returned.
pub fn windows_directory() -> io::Result<PathBuf> {
    let mut buffer = vec![0_u16; INITIAL_WINDOWS_DIRECTORY_CAPACITY];
    loop {
        #[allow(unsafe_code)]
        // SAFETY: `buffer` is a live, writable UTF-16 slice for the duration of
        // the call. The returned length is checked before reading the slice.
        let length = unsafe { GetWindowsDirectoryW(Some(&mut buffer)) } as usize;
        if length == 0 {
            return Err(io::Error::last_os_error());
        }
        if length < buffer.len() {
            buffer.truncate(length);
            return Ok(PathBuf::from(OsString::from_wide(&buffer)));
        }
        let required = length
            .checked_add(1)
            .ok_or_else(|| io::Error::other("Windows directory length overflow"))?;
        buffer.resize(required, 0);
    }
}

/// Return the inbox Task Scheduler command-line executable.
///
/// # Errors
/// Returns an error when the Windows directory cannot be queried.
pub fn task_scheduler_executable() -> io::Result<PathBuf> {
    Ok(windows_directory()?.join("System32").join("schtasks.exe"))
}

/// Return the inbox Windows PowerShell executable.
///
/// # Errors
/// Returns an error when the Windows directory cannot be queried.
pub fn windows_powershell_executable() -> io::Result<PathBuf> {
    Ok(windows_directory()?
        .join("System32")
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe"))
}

#[cfg(test)]
mod tests {
    use super::{task_scheduler_executable, windows_directory, windows_powershell_executable};

    #[test]
    fn discovers_absolute_windows_owned_paths() {
        let windows = windows_directory().unwrap();
        assert!(windows.is_absolute());
        assert_eq!(
            windows_powershell_executable().unwrap(),
            windows
                .join("System32")
                .join("WindowsPowerShell")
                .join("v1.0")
                .join("powershell.exe")
        );
        assert_eq!(
            task_scheduler_executable().unwrap(),
            windows.join("System32").join("schtasks.exe")
        );
    }
}
