//! Authenticated, local-only Windows named-pipe transport for the fixed runner.
//!
//! Both endpoints compare the peer process's user SID with their own. This
//! permits a highest-privilege scheduled task owned by the enrolled interactive
//! user without trusting another user in the same session. The pipe also rejects
//! remote clients and permits only one server instance.

use std::fs::{File, OpenOptions};
use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{
    CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_PIPE_CONNECTED, HANDLE, HLOCAL, LocalFree,
};
use windows::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
    ConvertStringSidToSidW, SDDL_REVISION_1,
};
use windows::Win32::Security::{
    EqualSid, GetLengthSid, GetTokenInformation, PSECURITY_DESCRIPTOR, PSID, SECURITY_ATTRIBUTES,
    TOKEN_QUERY, TOKEN_USER, TokenUser,
};
use windows::Win32::Storage::FileSystem::{FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, GetNamedPipeClientProcessId, GetNamedPipeServerProcessId,
    PIPE_REJECT_REMOTE_CLIENTS,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::core::{PCWSTR, PWSTR};

use crate::runner::{FrameError, read_frame, write_frame};

const PIPE_BUFFER: u32 = 64 * 1024;

/// Serve exactly one authenticated request and response, then close the pipe.
///
/// The supplied handler runs only after the client SID is verified. A single
/// instance plus the runner's filesystem lock prevents concurrent effects.
///
/// # Errors
/// Returns native pipe/security errors, a bounded framing error, or the handler's
/// error converted to [`io::Error`].
#[allow(unsafe_code)]
pub fn serve_one(
    pipe_name: &str,
    expected_client_sid: &str,
    pipe_sddl: &str,
    handler: impl FnOnce(&[u8]) -> io::Result<Vec<u8>>,
) -> io::Result<()> {
    let wide = wide_null(pipe_name);
    let sddl = wide_null(pipe_sddl);
    let mut descriptor = PSECURITY_DESCRIPTOR::default();
    // SAFETY: `sddl` is NUL-terminated and descriptor receives LocalAlloc memory.
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            PCWSTR(sddl.as_ptr()),
            SDDL_REVISION_1,
            &mut descriptor,
            None,
        )
    }
    .map_err(windows_error)?;
    let security = SECURITY_ATTRIBUTES {
        nLength: u32::try_from(std::mem::size_of::<SECURITY_ATTRIBUTES>())
            .expect("SECURITY_ATTRIBUTES size fits u32"),
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: false.into(),
    };
    // SAFETY: `wide` and `security` remain valid for the call; the returned
    // owned handle is checked before use.
    let handle = unsafe {
        CreateNamedPipeW(
            PCWSTR(wide.as_ptr()),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_REJECT_REMOTE_CLIENTS,
            1,
            PIPE_BUFFER,
            PIPE_BUFFER,
            0,
            Some(&raw const security),
        )
    };
    // SAFETY: descriptor was allocated by the SDDL conversion API.
    let _ = unsafe { LocalFree(Some(HLOCAL(descriptor.0))) };
    if handle.is_invalid() {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `handle` is a valid newly-created pipe. ERROR_PIPE_CONNECTED is a
    // successful early-client race documented by ConnectNamedPipe.
    if let Err(error) = unsafe { ConnectNamedPipe(handle, None) }
        && error.code() != ERROR_PIPE_CONNECTED.to_hresult()
    {
        // SAFETY: the checked handle is owned here and has not been transferred.
        let _ = unsafe { CloseHandle(handle) };
        return Err(io::Error::from_raw_os_error(error.code().0));
    }
    // SAFETY: ownership of the valid connected pipe moves into exactly one File.
    let mut pipe = unsafe { File::from_raw_handle(handle.0) };
    (|| {
        let mut client_pid = 0;
        // SAFETY: connected pipe handle and valid writable PID pointer.
        unsafe { GetNamedPipeClientProcessId(handle, &mut client_pid) }.map_err(windows_error)?;
        verify_process_user(client_pid, expected_client_sid)?;
        let request = read_frame(&mut pipe).map_err(frame_error)?;
        let response = handler(&request)?;
        write_frame(&mut pipe, &response).map_err(frame_error)
    })()
}

/// Connect to the runner, authenticate its process SID, and exchange one frame.
///
/// # Errors
/// Returns the last native open/security error after `timeout`, or a bounded
/// framing error.
#[allow(unsafe_code)]
pub fn transact(
    pipe_name: &str,
    expected_server_sid: &str,
    request: &[u8],
    timeout: Duration,
) -> io::Result<Vec<u8>> {
    let deadline = Instant::now() + timeout;
    let mut last_error;
    let mut pipe = loop {
        match OpenOptions::new().read(true).write(true).open(pipe_name) {
            Ok(pipe) => break pipe,
            Err(error) => {
                last_error = error;
                if Instant::now() >= deadline {
                    return Err(last_error);
                }
                thread::sleep(Duration::from_millis(25));
            }
        }
    };
    let handle = HANDLE(pipe.as_raw_handle());
    let mut server_pid = 0;
    // SAFETY: open pipe handle and a valid writable PID pointer.
    unsafe { GetNamedPipeServerProcessId(handle, &mut server_pid) }.map_err(windows_error)?;
    verify_process_user(server_pid, expected_server_sid)?;
    write_frame(&mut pipe, request).map_err(frame_error)?;
    read_frame(&mut pipe).map_err(frame_error)
}

#[allow(unsafe_code)]
fn verify_process_user(peer_pid: u32, expected_sid: &str) -> io::Result<()> {
    // SAFETY: access is query-only and PID came from the kernel pipe endpoint.
    let peer = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, peer_pid) }
        .map_err(windows_error)?;
    let peer_sid = process_user_sid(peer);
    // SAFETY: `peer` is an owned real process handle.
    let _ = unsafe { CloseHandle(peer) };
    let peer_sid = peer_sid?;
    let expected = string_sid(expected_sid)?;
    // SAFETY: both slices hold complete SID byte strings for this call.
    let equal = unsafe {
        EqualSid(
            PSID(peer_sid.as_ptr().cast_mut().cast()),
            PSID(expected.as_ptr().cast_mut().cast()),
        )
    };
    if equal.is_err() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "runner pipe peer user SID mismatch",
        ));
    }
    Ok(())
}

/// Return the current process user SID in canonical string form.
///
/// # Errors
/// Preserves Windows token and SID conversion failures.
#[allow(unsafe_code)]
pub fn current_user_sid_string() -> io::Result<String> {
    // SAFETY: pseudo-handle is valid for the current process and must not close.
    let bytes = process_user_sid(unsafe { GetCurrentProcess() })?;
    let mut text = PWSTR::null();
    // SAFETY: `bytes` contains a validated token SID and text receives LocalAlloc memory.
    unsafe { ConvertSidToStringSidW(PSID(bytes.as_ptr().cast_mut().cast()), &mut text) }
        .map_err(windows_error)?;
    // SAFETY: conversion returns a NUL-terminated string.
    let value = unsafe { text.to_string() }
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
    // SAFETY: text was allocated by ConvertSidToStringSidW.
    let _ = unsafe { LocalFree(Some(HLOCAL(text.0.cast()))) };
    value
}

#[allow(unsafe_code)]
fn string_sid(value: &str) -> io::Result<Vec<u8>> {
    let wide = wide_null(value);
    let mut sid = PSID::default();
    // SAFETY: `wide` is NUL-terminated and sid receives LocalAlloc memory.
    unsafe { ConvertStringSidToSidW(PCWSTR(wide.as_ptr()), &mut sid) }.map_err(windows_error)?;
    // SAFETY: successful conversion produced a valid SID.
    let length = unsafe { GetLengthSid(sid) } as usize;
    // SAFETY: converted SID is valid for `length` bytes until LocalFree.
    let bytes = unsafe { std::slice::from_raw_parts(sid.0.cast::<u8>(), length) }.to_vec();
    // SAFETY: sid was allocated by ConvertStringSidToSidW.
    let _ = unsafe { LocalFree(Some(HLOCAL(sid.0))) };
    Ok(bytes)
}

#[allow(unsafe_code)]
fn process_user_sid(process: HANDLE) -> io::Result<Vec<u8>> {
    let mut token = HANDLE::default();
    // SAFETY: process is a queryable process handle; token receives ownership.
    unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) }.map_err(windows_error)?;
    let result = (|| {
        let mut required = 0;
        // SAFETY: null first query obtains the exact required byte count.
        let first = unsafe { GetTokenInformation(token, TokenUser, None, 0, &mut required) };
        if first.is_ok()
            || io::Error::last_os_error().raw_os_error() != Some(ERROR_INSUFFICIENT_BUFFER.0 as i32)
        {
            return Err(io::Error::last_os_error());
        }
        let words = usize::try_from(required)
            .expect("token length fits usize")
            .div_ceil(std::mem::size_of::<usize>());
        let mut buffer = vec![0_usize; words];
        // SAFETY: aligned buffer has the exact reported byte capacity.
        unsafe {
            GetTokenInformation(
                token,
                TokenUser,
                Some(buffer.as_mut_ptr().cast()),
                required,
                &mut required,
            )
        }
        .map_err(windows_error)?;
        // SAFETY: successful TokenUser query initialized TOKEN_USER and its SID.
        let sid = unsafe { (*(buffer.as_ptr().cast::<TOKEN_USER>())).User.Sid };
        // SAFETY: the SID came from a successful token query.
        let length = unsafe { GetLengthSid(sid) } as usize;
        // SAFETY: SID is valid for `length` bytes while buffer remains alive.
        Ok(unsafe { std::slice::from_raw_parts(sid.0.cast::<u8>(), length) }.to_vec())
    })();
    // SAFETY: token is an owned handle created by OpenProcessToken.
    let _ = unsafe { CloseHandle(token) };
    result
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn windows_error(error: windows::core::Error) -> io::Error {
    io::Error::other(format!("Windows error {:#010x}: {error}", error.code().0))
}

fn frame_error(error: FrameError) -> io::Error {
    match error {
        FrameError::Io(error) => error,
        FrameError::Protocol(error) => io::Error::new(io::ErrorKind::InvalidData, error),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{current_user_sid_string, serve_one, transact};

    #[test]
    fn authenticated_local_pipe_round_trip() {
        let pipe = format!(
            r"\\.\pipe\HyperGpuSupport.Runner.test.{}",
            std::process::id()
        );
        let server_pipe = pipe.clone();
        let sid = current_user_sid_string().unwrap();
        let server_sid = sid.clone();
        let sddl = format!("D:P(A;;GA;;;{sid})");
        let server = std::thread::spawn(move || {
            serve_one(&server_pipe, &server_sid, &sddl, |request| {
                assert_eq!(request, b"request");
                Ok(b"response".to_vec())
            })
        });
        assert_eq!(
            transact(&pipe, &sid, b"request", Duration::from_secs(5)).unwrap(),
            b"response"
        );
        server.join().unwrap().unwrap();
    }
}
