//! GUI subsystem packaging with optional parent-console attachment for the CLI.
//! Redirected handles survive attachment. No console is created for GUI/workers.

/// Attach an explicit CLI invocation to its parent console, if one exists.
/// Pipe/file handles supplied by the parent are preserved independently per stream.
/// A headless caller with redirected streams does not need a console.
#[allow(unsafe_code)]
pub fn attach_parent() {
    use windows::Win32::{
        Storage::FileSystem::{FILE_TYPE_UNKNOWN, GetFileType},
        System::Console::{
            ATTACH_PARENT_PROCESS, AttachConsole, GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE,
            STD_OUTPUT_HANDLE, SetStdHandle,
        },
    };
    // SAFETY: borrowed process standard handles are probed and restored, never closed.
    // https://learn.microsoft.com/en-us/windows/console/getstdhandle
    unsafe {
        let handles = [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE].map(|kind| {
            let saved = GetStdHandle(kind)
                .ok()
                .filter(|handle| !handle.is_invalid() && GetFileType(*handle) != FILE_TYPE_UNKNOWN);
            (kind, saved)
        });
        if AttachConsole(ATTACH_PARENT_PROCESS).is_ok() {
            for (kind, handle) in handles {
                if let Some(handle) = handle {
                    let _ = SetStdHandle(kind, handle);
                }
            }
        }
    }
}
