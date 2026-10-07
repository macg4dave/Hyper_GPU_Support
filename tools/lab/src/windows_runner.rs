//! Authenticated, local-only Windows named-pipe transport for the fixed runner.
//!
//! The server impersonates the connected client to verify its token SID. The
//! client verifies the pipe object's enrolled owner SID without requiring access
//! to the server process. The pipe also rejects remote clients and permits only
//! one server instance.

use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{
    CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_PIPE_CONNECTED, HANDLE, HLOCAL, LocalFree,
};
use windows::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
    ConvertStringSidToSidW, GetSecurityInfo, SDDL_REVISION_1, SE_KERNEL_OBJECT,
};
use windows::Win32::Security::{
    EqualSid, GetLengthSid, GetTokenInformation, OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
    PSID, RevertToSelf, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER, TokenUser,
};
use windows::Win32::Storage::FileSystem::{
    FILE_CREATE_PIPE_INSTANCE, FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_GENERIC_READ, FILE_WRITE_DATA,
    PIPE_ACCESS_DUPLEX, SECURITY_IDENTIFICATION, SECURITY_SQOS_PRESENT,
};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, ImpersonateNamedPipeClient, PIPE_REJECT_REMOTE_CLIENTS,
    PeekNamedPipe,
};
use windows::Win32::System::TaskScheduler::{
    ITaskService, TASK_STATE_READY, TASK_STATE_RUNNING, TaskScheduler,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentThread, OpenProcessToken, OpenThreadToken,
};
use windows::Win32::System::Variant::VARIANT;
use windows::core::BSTR;
use windows::core::{PCWSTR, PWSTR};

use crate::runner::{FRAME_LIMIT, FrameError, read_frame, write_frame};

const PIPE_BUFFER: u32 = 64 * 1024;
const CLIENT_PIPE_ACCESS: u32 = FILE_GENERIC_READ.0 | FILE_WRITE_DATA.0;
const CLIENT_PIPE_SQOS: u32 = SECURITY_SQOS_PRESENT.0 | SECURITY_IDENTIFICATION.0;
const _: () = assert!(CLIENT_PIPE_ACCESS & FILE_CREATE_PIPE_INSTANCE.0 == 0);

struct ComApartment;

impl ComApartment {
    #[allow(unsafe_code)]
    fn initialize() -> io::Result<Self> {
        // SAFETY: initializes COM for this thread once for the duration of the guard.
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
            .ok()
            .map_err(windows_error)?;
        Ok(Self)
    }
}

impl Drop for ComApartment {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: paired with this guard's successful CoInitializeEx call.
        unsafe { CoUninitialize() };
    }
}

/// Wait until an enrolled scheduled task is ready for a new one-shot invocation.
///
/// This closes the interval in which a completed pipe response can precede Task
/// Scheduler observing process exit. Triggering during that interval would be
/// ignored by the runner task's `TASK_INSTANCES_IGNORE_NEW` policy.
///
/// # Errors
/// Returns native COM/Task Scheduler errors, an unexpected task state, or a
/// timeout while the preceding invocation is still running.
#[allow(unsafe_code)]
pub fn wait_for_task_ready(task_path: &str, task_name: &str, timeout: Duration) -> io::Result<()> {
    let _apartment = ComApartment::initialize()?;
    // SAFETY: COM is initialized on this thread; no aggregation is requested.
    let service: ITaskService =
        unsafe { CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER) }
            .map_err(windows_error)?;
    let empty = VARIANT::default();
    // SAFETY: empty variants request a connection using the current local token.
    unsafe { service.Connect(&empty, &empty, &empty, &empty) }.map_err(windows_error)?;
    let folder_path = task_path.trim_end_matches('\\');
    // SAFETY: BSTR arguments remain valid for each synchronous COM call.
    let folder = unsafe { service.GetFolder(&BSTR::from(folder_path)) }.map_err(windows_error)?;
    let task = unsafe { folder.GetTask(&BSTR::from(task_name)) }.map_err(windows_error)?;
    let deadline = Instant::now() + timeout;
    loop {
        // SAFETY: the registered-task interface remains valid in this apartment.
        match unsafe { task.State() }.map_err(windows_error)? {
            TASK_STATE_READY => return Ok(()),
            TASK_STATE_RUNNING if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(25));
            }
            TASK_STATE_RUNNING => {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "previous runner task invocation did not become ready",
                ));
            }
            _ => {
                return Err(io::Error::other(
                    "runner task is not enabled and ready for invocation",
                ));
            }
        }
    }
}

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
        let request = read_frame(&mut pipe).map_err(frame_error)?;
        verify_pipe_client(handle, expected_client_sid)?;
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
        match OpenOptions::new()
            .access_mode(CLIENT_PIPE_ACCESS)
            .security_qos_flags(CLIENT_PIPE_SQOS)
            .open(pipe_name)
        {
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
    verify_pipe_owner(handle, expected_server_sid)?;
    write_frame(&mut pipe, request).map_err(frame_error)?;
    wait_for_frame(&mut pipe, handle, deadline)
}

#[allow(unsafe_code)]
fn wait_for_frame(pipe: &mut File, handle: HANDLE, deadline: Instant) -> io::Result<Vec<u8>> {
    wait_for_available(handle, 4, deadline)?;
    let mut header = [0_u8; 4];
    pipe.read_exact(&mut header)?;
    let frame_length = u32::from_le_bytes(header) as usize;
    if frame_length > FRAME_LIMIT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "runner protocol frame exceeds limit",
        ));
    }
    wait_for_available(handle, frame_length, deadline)?;
    let mut payload = vec![0_u8; frame_length];
    pipe.read_exact(&mut payload)?;
    Ok(payload)
}

#[allow(unsafe_code)]
fn wait_for_available(handle: HANDLE, required: usize, deadline: Instant) -> io::Result<()> {
    if required == 0 {
        return Ok(());
    }
    loop {
        let mut available = 0;
        // SAFETY: `handle` remains owned by the caller and `available` is a valid
        // writable counter for this non-consuming query.
        unsafe { PeekNamedPipe(handle, None, 0, None, Some(&raw mut available), None) }
            .map_err(windows_error)?;
        if available as usize >= required {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "runner response deadline exceeded",
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[allow(unsafe_code)]
fn verify_pipe_client(pipe: HANDLE, expected_sid: &str) -> io::Result<()> {
    // SAFETY: the connected pipe has received the bounded request whose client
    // security context is selected for impersonation.
    unsafe { ImpersonateNamedPipeClient(pipe) }.map_err(windows_error)?;
    let verification = (|| {
        let mut token = HANDLE::default();
        // SAFETY: the current thread impersonates the connected client and the
        // output receives an owned query-only token handle.
        unsafe { OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, true, &mut token) }
            .map_err(windows_error)?;
        let client_sid = token_user_sid(token);
        // SAFETY: token ownership was returned by OpenThreadToken.
        let _ = unsafe { CloseHandle(token) };
        compare_sid(
            &client_sid?,
            expected_sid,
            "runner pipe client user SID mismatch",
        )
    })();
    // SAFETY: the thread successfully impersonated this pipe client above. The
    // Windows contract requires process termination if the original token cannot
    // be restored; continuing under an untrusted client identity is forbidden.
    if unsafe { RevertToSelf() }.is_err() {
        std::process::abort();
    }
    verification
}

#[allow(unsafe_code)]
fn verify_pipe_owner(pipe: HANDLE, expected_sid: &str) -> io::Result<()> {
    let expected = string_sid(expected_sid)?;
    let mut owner = PSID::default();
    let mut descriptor = PSECURITY_DESCRIPTOR::default();
    // SAFETY: pipe is an open kernel handle and output pointers are writable.
    let status = unsafe {
        GetSecurityInfo(
            pipe,
            SE_KERNEL_OBJECT,
            OWNER_SECURITY_INFORMATION,
            Some(&raw mut owner),
            None,
            None,
            None,
            Some(&raw mut descriptor),
        )
    };
    if status.0 != 0 {
        return Err(io::Error::from_raw_os_error(status.0 as i32));
    }
    // SAFETY: owner points inside descriptor and expected is a complete SID.
    let equal = unsafe { EqualSid(owner, PSID(expected.as_ptr().cast_mut().cast())) };
    // SAFETY: descriptor was allocated by GetSecurityInfo.
    let _ = unsafe { LocalFree(Some(HLOCAL(descriptor.0))) };
    equal.map_err(|_| {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            "runner pipe owner SID mismatch",
        )
    })
}

#[allow(unsafe_code)]
fn compare_sid(actual: &[u8], expected: &str, mismatch: &'static str) -> io::Result<()> {
    let expected = string_sid(expected)?;
    // SAFETY: both slices hold complete SID byte strings for this call.
    unsafe {
        EqualSid(
            PSID(actual.as_ptr().cast_mut().cast()),
            PSID(expected.as_ptr().cast_mut().cast()),
        )
    }
    .map_err(|_| io::Error::new(io::ErrorKind::PermissionDenied, mismatch))
}

/// Return the current process user SID in canonical string form.
///
/// # Errors
/// Preserves Windows token and SID conversion failures.
#[allow(unsafe_code)]
pub fn current_user_sid_string() -> io::Result<String> {
    let mut token = HANDLE::default();
    // SAFETY: the current-process pseudo-handle is valid and token receives ownership.
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }
        .map_err(windows_error)?;
    let bytes = token_user_sid(token);
    // SAFETY: token ownership was returned by OpenProcessToken.
    let _ = unsafe { CloseHandle(token) };
    let bytes = bytes?;
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
fn token_user_sid(token: HANDLE) -> io::Result<Vec<u8>> {
    (|| {
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
    })()
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
    use std::fs::File;
    use std::io::Write;
    use std::os::windows::io::{AsRawHandle, FromRawHandle};
    use std::time::Duration;

    use super::{PIPE_BUFFER, current_user_sid_string, serve_one, transact, wait_for_frame};
    use crate::runner::{FRAME_LIMIT, write_frame};
    use windows::Win32::Foundation::{CloseHandle, ERROR_NO_TOKEN, HANDLE};
    use windows::Win32::Security::TOKEN_QUERY;
    use windows::Win32::System::Pipes::CreatePipe;
    use windows::Win32::System::Threading::{GetCurrentThread, OpenThreadToken};

    #[allow(unsafe_code)]
    fn assert_not_impersonating() {
        let mut token = HANDLE::default();
        // SAFETY: the output pointer is writable and the pseudo-handle is valid.
        match unsafe { OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, true, &mut token) } {
            Err(error) => assert_eq!(error.code(), ERROR_NO_TOKEN.to_hresult()),
            Ok(()) => {
                // SAFETY: an unexpected success returned an owned token handle.
                let _ = unsafe { CloseHandle(token) };
                panic!("handler still has an impersonation token");
            }
        }
    }

    #[test]
    fn authenticated_local_pipe_round_trip() {
        let pipe = format!(
            r"\\.\pipe\HyperGpuSupport.Runner.test.{}",
            std::process::id()
        );
        let server_pipe = pipe.clone();
        let sid = current_user_sid_string().unwrap();
        let server_sid = sid.clone();
        let sddl = format!("O:{sid}D:P(A;;0x0012008b;;;{sid})");
        let server = std::thread::spawn(move || {
            serve_one(&server_pipe, &server_sid, &sddl, |request| {
                assert_not_impersonating();
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

    #[test]
    fn server_rejects_wrong_client_sid_before_handler() {
        let pipe = format!(
            r"\\.\pipe\HyperGpuSupport.Runner.client-test.{}",
            std::process::id()
        );
        let server_pipe = pipe.clone();
        let sid = current_user_sid_string().unwrap();
        let server_sid = sid.clone();
        let sddl = format!("O:{sid}D:P(A;;0x0012008b;;;{sid})");
        let server = std::thread::spawn(move || {
            serve_one(&server_pipe, "S-1-5-18", &sddl, |_| {
                panic!("handler ran for wrong client SID")
            })
        });
        assert!(transact(&pipe, &server_sid, b"request", Duration::from_secs(5)).is_err());
        let error = server.join().unwrap().unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn client_rejects_pipe_owned_by_another_sid() {
        let pipe = format!(
            r"\\.\pipe\HyperGpuSupport.Runner.owner-test.{}",
            std::process::id()
        );
        let server_pipe = pipe.clone();
        let sid = current_user_sid_string().unwrap();
        let client_sid = sid.clone();
        let sddl = format!("O:{sid}D:P(A;;0x0012008b;;;{sid})");
        let server = std::thread::spawn(move || {
            serve_one(&server_pipe, &client_sid, &sddl, |_| Ok(Vec::new()))
        });
        let error = transact(&pipe, "S-1-5-18", b"request", Duration::from_secs(5)).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(server.join().unwrap().is_err());
    }

    #[test]
    fn client_access_excludes_server_instance_creation() {
        assert_eq!(
            super::CLIENT_PIPE_ACCESS & super::FILE_CREATE_PIPE_INSTANCE.0,
            0
        );
    }

    #[allow(unsafe_code)]
    fn anonymous_pipe() -> (File, File) {
        let mut read = HANDLE::default();
        let mut write = HANDLE::default();
        // SAFETY: output pointers are writable and successful handles move into
        // exactly one owning File each.
        unsafe { CreatePipe(&raw mut read, &raw mut write, None, PIPE_BUFFER) }.unwrap();
        // SAFETY: CreatePipe returned two distinct owned handles.
        unsafe {
            (
                File::from_raw_handle(read.0),
                File::from_raw_handle(write.0),
            )
        }
    }

    #[test]
    fn bounded_reader_deadline_is_enforced_on_ready_pipe() {
        let (mut reader, _writer) = anonymous_pipe();
        let handle = HANDLE(reader.as_raw_handle());
        let started = std::time::Instant::now();
        let error =
            wait_for_frame(&mut reader, handle, started + Duration::from_millis(100)).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn bounded_reader_accepts_partial_header_and_payload() {
        let (mut reader, mut writer) = anonymous_pipe();
        let payload = b"partial-payload".to_vec();
        let expected = payload.clone();
        let writer = std::thread::spawn(move || {
            let header = (payload.len() as u32).to_le_bytes();
            writer.write_all(&header[..2]).unwrap();
            std::thread::sleep(Duration::from_millis(20));
            writer.write_all(&header[2..]).unwrap();
            writer.write_all(&payload[..3]).unwrap();
            std::thread::sleep(Duration::from_millis(20));
            writer.write_all(&payload[3..]).unwrap();
        });
        let handle = HANDLE(reader.as_raw_handle());
        assert_eq!(
            wait_for_frame(
                &mut reader,
                handle,
                std::time::Instant::now() + Duration::from_secs(1)
            )
            .unwrap(),
            expected
        );
        writer.join().unwrap();
    }

    #[test]
    fn bounded_reader_accepts_maximum_frame_without_backpressure_deadlock() {
        let (mut reader, mut writer) = anonymous_pipe();
        let expected = vec![0x5a; FRAME_LIMIT];
        let payload = expected.clone();
        let writer = std::thread::spawn(move || write_frame(&mut writer, &payload).unwrap());
        let handle = HANDLE(reader.as_raw_handle());
        assert_eq!(
            wait_for_frame(
                &mut reader,
                handle,
                std::time::Instant::now() + Duration::from_secs(2)
            )
            .unwrap(),
            expected
        );
        writer.join().unwrap();
    }

    #[test]
    fn bounded_reader_rejects_oversized_declaration() {
        let (mut reader, mut writer) = anonymous_pipe();
        writer
            .write_all(&((FRAME_LIMIT as u32) + 1).to_le_bytes())
            .unwrap();
        let handle = HANDLE(reader.as_raw_handle());
        let error = wait_for_frame(
            &mut reader,
            handle,
            std::time::Instant::now() + Duration::from_secs(1),
        )
        .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn bounded_reader_accepts_empty_frame_after_writer_closes() {
        let (mut reader, mut writer) = anonymous_pipe();
        write_frame(&mut writer, &[]).unwrap();
        drop(writer);
        let handle = HANDLE(reader.as_raw_handle());
        assert!(
            wait_for_frame(
                &mut reader,
                handle,
                std::time::Instant::now() + Duration::from_secs(1)
            )
            .unwrap()
            .is_empty()
        );
    }
}
