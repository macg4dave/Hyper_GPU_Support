//! Bounded, process-bound transport for one explicitly launched elevated worker.
use super::{
    CLIENT_PIPE_ACCESS, CLIENT_PIPE_SQOS, FRAME_LIMIT, PIPE_BUFFER, frame_error,
    verify_pipe_client, verify_pipe_owner, wide_null, windows_error,
};
use std::{
    fs::{File, OpenOptions},
    io,
    os::windows::{
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle},
    },
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::{
            CloseHandle, ERROR_IO_PENDING, ERROR_PIPE_CONNECTED, HANDLE, HLOCAL, LocalFree,
        },
        Security::{
            Authorization::{
                ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
            },
            PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES,
        },
        Storage::FileSystem::{
            FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED, PIPE_ACCESS_DUPLEX, ReadFile,
            WriteFile,
        },
        System::{
            IO::{CancelIoEx, GetOverlappedResultEx, OVERLAPPED},
            Pipes::{
                ConnectNamedPipe, CreateNamedPipeW, GetNamedPipeClientProcessId,
                GetNamedPipeServerProcessId, PIPE_REJECT_REMOTE_CLIENTS,
            },
            Threading::CreateEventW,
        },
    },
    core::PCWSTR,
};

/// File ownership plus one common bounded operation deadline.
pub(crate) struct Pipe {
    file: File,
}
struct Transfer {
    event: HANDLE,
    overlapped: Box<OVERLAPPED>,
    bytes: Vec<u8>,
}
impl Drop for Transfer {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.bytes.zeroize();
        // SAFETY: event is sole owned handle; transfer is dropped only after completion.
        unsafe {
            let _ = CloseHandle(self.event);
        }
    }
}
impl Transfer {
    #[allow(unsafe_code)]
    fn new(bytes: Vec<u8>) -> io::Result<Self> {
        // SAFETY: unnamed private event; returned handle is owned by this transfer.
        let event =
            unsafe { CreateEventW(None, true, false, PCWSTR::null()) }.map_err(windows_error)?;
        let overlapped = Box::new(OVERLAPPED {
            hEvent: event,
            ..Default::default()
        });
        Ok(Self {
            event,
            overlapped,
            bytes,
        })
    }
}
#[allow(unsafe_code)]
fn complete(
    handle: HANDLE,
    transfer: &mut Transfer,
    result: windows::core::Result<()>,
    timeout: Duration,
) -> io::Result<usize> {
    if let Err(error) = result
        && error.code() != ERROR_IO_PENDING.to_hresult()
    {
        return Err(windows_error(error));
    }
    let mut count = 0;
    // SAFETY: boxed OVERLAPPED/event and heap buffer remain alive until kernel completion.
    unsafe {
        GetOverlappedResultEx(
            handle,
            &*transfer.overlapped,
            &mut count,
            timeout.as_millis().min(u128::from(u32::MAX)) as u32,
            false,
        )
    }
    .map_err(windows_error)?;
    Ok(count as usize)
}
#[allow(unsafe_code)]
fn perform(
    handle: HANDLE,
    mut transfer: Transfer,
    operation: impl FnOnce(&mut Transfer) -> windows::core::Result<()>,
    timeout: Duration,
) -> io::Result<(Transfer, usize)> {
    let result = operation(&mut transfer);
    match complete(handle, &mut transfer, result, timeout) {
        Ok(count) => Ok((transfer, count)),
        Err(primary) => {
            // SAFETY: exact still-live OVERLAPPED; cancel only this transfer.
            unsafe {
                let _ = CancelIoEx(handle, Some(&*transfer.overlapped));
            }
            let mut count = 0;
            // Cancellation completion must be drained before Rust drops the backing allocations.
            // SAFETY: transfer remains fully alive during this bounded drain.
            let drained = unsafe {
                GetOverlappedResultEx(handle, &*transfer.overlapped, &mut count, 5000, false)
            };
            if let Err(error) = drained
                && (error.code()
                    == windows::core::HRESULT::from_win32(
                        windows::Win32::Foundation::WAIT_TIMEOUT.0,
                    )
                    || error.code() == windows::Win32::Foundation::ERROR_IO_INCOMPLETE.to_hresult())
            {
                // The OS still owns pointers. Leak these bounded allocations instead of
                // freeing pending kernel buffers. Worker watchdog/process exit contains them.
                std::mem::forget(transfer);
            }
            Err(primary)
        }
    }
}
fn remaining(deadline: Instant) -> io::Result<Duration> {
    deadline
        .checked_duration_since(Instant::now())
        .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "worker transport deadline expired"))
}
impl Pipe {
    #[allow(unsafe_code)]
    pub(crate) fn accept(name: &str, sid: &str, peer_pid: u32) -> io::Result<Self> {
        let descriptor_text = wide_null(&format!(
            "O:BAG:BAD:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;0x0012008b;;;{sid})"
        ));
        let mut descriptor = PSECURITY_DESCRIPTOR::default();
        // SAFETY: terminated SDDL, API owns initialized descriptor allocation.
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                PCWSTR(descriptor_text.as_ptr()),
                SDDL_REVISION_1,
                &mut descriptor,
                None,
            )
        }
        .map_err(windows_error)?;
        let attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor.0,
            bInheritHandle: false.into(),
        };
        let name = wide_null(name);
        // SAFETY: descriptor/name live during call; first local instance, overlapped IO.
        let handle = unsafe {
            CreateNamedPipeW(
                PCWSTR(name.as_ptr()),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE | FILE_FLAG_OVERLAPPED,
                PIPE_REJECT_REMOTE_CLIENTS,
                1,
                PIPE_BUFFER,
                PIPE_BUFFER,
                0,
                Some(&attributes),
            )
        };
        unsafe {
            LocalFree(Some(HLOCAL(descriptor.0)));
        }
        if handle.is_invalid() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: checked owned handle transfers once to File.
        let pipe = Self {
            file: unsafe { File::from_raw_handle(handle.0) },
        };
        let mut transfer = Transfer::new(Vec::new())?;
        // SAFETY: stable OVERLAPPED retained through completion/drain.
        let started = unsafe { ConnectNamedPipe(handle, Some(&mut *transfer.overlapped)) };
        if !started
            .as_ref()
            .is_err_and(|error| error.code() == ERROR_PIPE_CONNECTED.to_hresult())
        {
            perform(handle, transfer, |_| started, Duration::from_secs(30))?;
        }
        let mut actual = 0;
        unsafe { GetNamedPipeClientProcessId(handle, &mut actual) }.map_err(windows_error)?;
        if actual != peer_pid {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "worker client process mismatch",
            ));
        }
        // Token impersonation needs a received message, so SID is checked after read.
        Ok(pipe)
    }
    #[allow(unsafe_code)]
    pub(crate) fn connect(name: &str, sid: &str, peer_pid: u32) -> io::Result<Self> {
        let deadline = Instant::now() + Duration::from_secs(30);
        let file = loop {
            match OpenOptions::new()
                .access_mode(CLIENT_PIPE_ACCESS)
                .custom_flags(FILE_FLAG_OVERLAPPED.0)
                .security_qos_flags(CLIENT_PIPE_SQOS)
                .open(name)
            {
                Ok(file) => break file,
                Err(error) => {
                    if Instant::now() >= deadline {
                        return Err(error);
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
            }
        };
        let handle = HANDLE(file.as_raw_handle());
        verify_pipe_owner(handle, sid)?;
        let mut actual = 0;
        // SAFETY: live connected pipe and initialized output counter.
        unsafe { GetNamedPipeServerProcessId(handle, &mut actual) }.map_err(windows_error)?;
        if actual != peer_pid {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "worker server process mismatch",
            ));
        }
        Ok(Self { file })
    }
    pub(crate) fn authenticate_client(&self, sid: &str) -> io::Result<()> {
        verify_pipe_client(HANDLE(self.file.as_raw_handle()), sid)
    }
    #[allow(unsafe_code)]
    fn read_exact(&self, length: usize, deadline: Instant) -> io::Result<Vec<u8>> {
        let handle = HANDLE(self.file.as_raw_handle());
        let mut output = zeroize::Zeroizing::new(Vec::with_capacity(length));
        while output.len() < length {
            let transfer = Transfer::new(vec![0; length - output.len()])?;
            let (transfer, count) = perform(
                handle,
                transfer,
                |transfer| {
                    // SAFETY: byte buffer and boxed OVERLAPPED outlive the overlapped operation.
                    unsafe {
                        ReadFile(
                            handle,
                            Some(&mut transfer.bytes),
                            None,
                            Some(&mut *transfer.overlapped),
                        )
                    }
                },
                remaining(deadline)?,
            )?;
            if count == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "worker pipe closed",
                ));
            }
            output.extend_from_slice(
                transfer
                    .bytes
                    .get(..count)
                    .ok_or_else(|| io::Error::other("invalid transferred byte count"))?,
            );
        }
        Ok(output.to_vec())
    }
    pub(crate) fn receive(&self, timeout: Duration) -> io::Result<Vec<u8>> {
        let deadline = Instant::now() + timeout;
        let header = self.read_exact(4, deadline)?;
        let length = u32::from_le_bytes(
            header
                .as_slice()
                .try_into()
                .map_err(|_| frame_error(crate::runner::FrameError))?,
        ) as usize;
        if length > FRAME_LIMIT {
            return Err(frame_error(crate::runner::FrameError));
        }
        self.read_exact(length, deadline)
    }
    #[allow(unsafe_code)]
    pub(crate) fn send(&self, bytes: &[u8]) -> io::Result<()> {
        if bytes.len() > FRAME_LIMIT {
            return Err(frame_error(crate::runner::FrameError));
        }
        let mut frame = zeroize::Zeroizing::new((bytes.len() as u32).to_le_bytes().to_vec());
        frame.extend_from_slice(bytes);
        let deadline = Instant::now() + Duration::from_secs(15);
        let handle = HANDLE(self.file.as_raw_handle());
        let mut offset = 0;
        while offset < frame.len() {
            let transfer = Transfer::new(frame[offset..].to_vec())?;
            let (_, count) = perform(
                handle,
                transfer,
                |transfer| {
                    // SAFETY: stable owned buffer and boxed OVERLAPPED retained until completion.
                    unsafe {
                        WriteFile(
                            handle,
                            Some(&transfer.bytes),
                            None,
                            Some(&mut *transfer.overlapped),
                        )
                    }
                },
                remaining(deadline)?,
            )?;
            if count == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "worker pipe write stalled",
                ));
            }
            offset += count;
        }
        Ok(())
    }
}
