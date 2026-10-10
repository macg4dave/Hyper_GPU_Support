//! Bounded child processes and independent worker deadlines.
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
/// Captured process result with bounded streams.
pub struct ProcessOutput {
    /// Exit status.
    pub exit_code: Option<i32>,
    /// Bounded stdout.
    pub stdout: String,
    /// Bounded stderr.
    pub stderr: String,
}

const PROCESS_OUTPUT_LIMIT: usize = 1024 * 1024;
// Fixed helpers stream payload bytes; they are not general workload launchers.
// Bound aggregate descendants before resume, leaving room for Windows/UI work.
const CHILD_JOB_MEMORY_LIMIT: usize = 1024 * 1024 * 1024;
const CHILD_JOB_PROCESS_LIMIT: u32 = 4;
const CHILD_JOB_CPU_RATE: u32 = 2500; // 25% in Windows' 1/100-percent units.

struct BackgroundWork(bool);
#[allow(unsafe_code)]
impl BackgroundWork {
    fn start() -> Result<Self, String> {
        use windows::Win32::System::Threading::{
            GetCurrentThread, SetThreadPriority, THREAD_MODE_BACKGROUND_BEGIN,
        };
        // SAFETY: modifies only the calling worker thread; no borrowed handles.
        unsafe { SetThreadPriority(GetCurrentThread(), THREAD_MODE_BACKGROUND_BEGIN) }
            .map_err(|e| format!("cannot enter background payload processing: {e}"))?;
        Ok(Self(true))
    }
    fn finish(&mut self) -> Result<(), String> {
        use windows::Win32::System::Threading::{
            GetCurrentThread, SetThreadPriority, THREAD_MODE_BACKGROUND_END,
        };
        if self.0 {
            // SAFETY: same synchronous thread that successfully entered background mode.
            unsafe { SetThreadPriority(GetCurrentThread(), THREAD_MODE_BACKGROUND_END) }
                .map_err(|e| format!("cannot restore payload scheduling priority: {e}"))?;
            self.0 = false;
        }
        Ok(())
    }
}
impl Drop for BackgroundWork {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}
pub(crate) fn background_work<T>(work: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let mut guard = BackgroundWork::start()?;
    let result = work();
    match (result, guard.finish()) {
        (result, Ok(())) => result,
        (Ok(_), Err(recovery)) => Err(recovery),
        (Err(primary), Err(recovery)) => Err(format!("{primary}; {recovery}")),
    }
}
/// Independent worker deadline. Abrupt exit closes OS-owned kill-job handles even
/// when WMI/input or remoting blocks; no Rust destructor is required for containment.
pub struct WorkerDeadline {
    cancel: std::sync::mpsc::Sender<()>,
    thread: Option<thread::JoinHandle<()>>,
}
impl WorkerDeadline {
    /// Start the worker's hard watchdog before any blocking guest operations.
    #[must_use]
    pub fn start(timeout: Duration) -> Self {
        Self::with_expiry(timeout, || std::process::exit(6))
    }
    fn with_expiry(timeout: Duration, expire: impl FnOnce() + Send + 'static) -> Self {
        let (cancel, receiver) = std::sync::mpsc::channel();
        let thread = thread::spawn(move || {
            if matches!(
                receiver.recv_timeout(timeout),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout)
            ) {
                expire();
            }
        });
        Self {
            cancel,
            thread: Some(thread),
        }
    }
}
impl Drop for WorkerDeadline {
    fn drop(&mut self) {
        let _ = self.cancel.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[allow(unsafe_code)]
fn resume_suspended_child(id: u32) -> Result<(), String> {
    use windows::Win32::{
        Foundation::{CloseHandle, ERROR_NO_MORE_FILES},
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First,
                Thread32Next,
            },
            Threading::{OpenThread, ResumeThread, THREAD_SUSPEND_RESUME},
        },
    };
    // SAFETY: read-only snapshot; handle is owned locally.
    let snapshot =
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) }.map_err(|e| e.to_string())?;
    let result = (|| {
        let mut entry = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };
        // SAFETY: correctly sized writable entry and live snapshot.
        unsafe { Thread32First(snapshot, &mut entry) }.map_err(|e| e.to_string())?;
        loop {
            if entry.th32OwnerProcessID == id {
                // The process was created suspended, so this is its only initial
                // thread; no user code or descendants have run before job assignment.
                // SAFETY: thread belongs to our live suspended child.
                let handle =
                    unsafe { OpenThread(THREAD_SUSPEND_RESUME, false, entry.th32ThreadID) }
                        .map_err(|e| e.to_string())?;
                // SAFETY: owned live suspended thread, now inside the kill job.
                let previous = unsafe { ResumeThread(handle) };
                // SAFETY: close this function's owned thread handle exactly once.
                unsafe { CloseHandle(handle) }.map_err(|e| e.to_string())?;
                if previous != 1 {
                    return Err("unexpected child suspend count".into());
                }
                return Ok(());
            }
            // SAFETY: same live snapshot and correctly sized writable entry.
            match unsafe { Thread32Next(snapshot, &mut entry) } {
                Ok(()) => {}
                Err(e) if e.code() == windows::core::HRESULT::from_win32(ERROR_NO_MORE_FILES.0) => {
                    return Err("suspended child thread missing".into());
                }
                Err(e) => return Err(e.to_string()),
            }
        }
    })();
    // SAFETY: release the snapshot after enumeration on every path.
    let close = unsafe { CloseHandle(snapshot) }.map_err(|e| e.to_string());
    result.and(close)
}

/// Native OS version as reported by WMI, without inventing an unavailable UBR.
/// # Errors
/// Native OS query failures remain explicit.
/// # Errors
/// Preserves native token failures instead of probing privileged APIs unelevated.
#[allow(unsafe_code)]
pub fn is_elevated() -> Result<bool, String> {
    use windows::Win32::{
        Foundation::{CloseHandle, HANDLE},
        Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation},
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    let mut token = HANDLE::default();
    // SAFETY: current process handle is valid; token receives owned handle.
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }
        .map_err(|e| e.to_string())?;
    let mut elevation = TOKEN_ELEVATION::default();
    let mut length = 0;
    // SAFETY: correctly sized and aligned writable TOKEN_ELEVATION buffer.
    let result = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            Some((&mut elevation as *mut TOKEN_ELEVATION).cast()),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut length,
        )
    };
    // SAFETY: token is the handle owned by this function.
    let close = unsafe { CloseHandle(token) };
    result.map_err(|e| e.to_string())?;
    close.map_err(|e| e.to_string())?;
    Ok(elevation.TokenIsElevated != 0)
}

struct KillJob(windows::Win32::Foundation::HANDLE);
#[allow(unsafe_code)]
impl KillJob {
    fn create() -> Result<Self, String> {
        use windows::Win32::System::JobObjects::{
            CreateJobObjectW, JOB_OBJECT_CPU_RATE_CONTROL_ENABLE,
            JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
            JOB_OBJECT_LIMIT_JOB_MEMORY, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_CPU_RATE_CONTROL_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JobObjectCpuRateControlInformation, JobObjectExtendedLimitInformation,
            SetInformationJobObject,
        };
        // SAFETY: unnamed non-inheritable job with no external pointers.
        let job = Self(unsafe { CreateJobObjectW(None, None) }.map_err(|e| e.to_string())?);
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            | JOB_OBJECT_LIMIT_JOB_MEMORY
            | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
        info.BasicLimitInformation.ActiveProcessLimit = CHILD_JOB_PROCESS_LIMIT;
        info.JobMemoryLimit = CHILD_JOB_MEMORY_LIMIT;
        // SAFETY: valid owned job and correctly sized information structure.
        unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&info) as u32,
            )
        }
        .map_err(|e| e.to_string())?;
        let mut cpu = JOBOBJECT_CPU_RATE_CONTROL_INFORMATION {
            ControlFlags: JOB_OBJECT_CPU_RATE_CONTROL_ENABLE | JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP,
            ..Default::default()
        };
        cpu.Anonymous.CpuRate = CHILD_JOB_CPU_RATE;
        // SAFETY: valid owned job; enabled hard cap selects the CpuRate union arm.
        unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectCpuRateControlInformation,
                (&cpu as *const JOBOBJECT_CPU_RATE_CONTROL_INFORMATION).cast(),
                std::mem::size_of_val(&cpu) as u32,
            )
        }
        .map_err(|e| format!("cannot enforce child CPU limit: {e}"))?;
        Ok(job)
    }
    fn attach(child: &std::process::Child) -> Result<Self, String> {
        use std::os::windows::io::AsRawHandle;
        use windows::Win32::{Foundation::HANDLE, System::JobObjects::AssignProcessToJobObject};
        let job = Self::create()?;
        // SAFETY: child owns this live process handle. Job cannot outlive ownership.
        unsafe { AssignProcessToJobObject(job.0, HANDLE(child.as_raw_handle())) }
            .map_err(|e| e.to_string())?;
        Ok(job)
    }
}
#[allow(unsafe_code)]
impl Drop for KillJob {
    fn drop(&mut self) {
        // SAFETY: close owned job once; KILL_ON_JOB_CLOSE terminates remaining children.
        let _ = unsafe { windows::Win32::Foundation::CloseHandle(self.0) };
    }
}

/// Failure from a contained child process.
pub enum BoundedProcessError {
    /// Process could not be launched.
    Launch {
        /// Standard I/O error category.
        kind: std::io::ErrorKind,
        /// Native operating-system error code.
        code: Option<i32>,
    },
    /// Hard execution budget exceeded.
    Timeout,
    /// Captured output exceeded the safety bound.
    OutputTooLarge,
    /// Other bounded execution failure.
    Execution(String),
}
impl std::fmt::Display for BoundedProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Launch { kind, code } => {
                write!(f, "process launch failed: {kind} (OS code {code:?})")
            }
            Self::Timeout => {
                f.write_str("process deadline expired; process and descendants terminated")
            }
            Self::OutputTooLarge => f.write_str("process output limit exceeded"),
            Self::Execution(message) => f.write_str(message),
        }
    }
}
impl From<String> for BoundedProcessError {
    fn from(message: String) -> Self {
        Self::Execution(message)
    }
}
impl From<&str> for BoundedProcessError {
    fn from(message: &str) -> Self {
        Self::Execution(message.into())
    }
}

fn drain(mut reader: impl Read, limit: usize) -> Result<Vec<u8>, BoundedProcessError> {
    let mut bytes = Vec::new();
    let mut buf = [0; 1024];
    loop {
        let count = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if count == 0 {
            return Ok(bytes);
        }
        if bytes.len() + count > limit {
            return Err(BoundedProcessError::OutputTooLarge);
        }
        bytes.extend_from_slice(&buf[..count]);
    }
}

/// Fixed bridge control records are consumed independently of diagnostic output.
/// Implementations must not block while checking deadlines or holding state locks.
pub(crate) trait StderrMonitor: Send + Sync {
    fn line(&self, line: &[u8]) -> Result<bool, String>;
    fn check(&self, now: Instant) -> Result<(), String>;
    fn finish(&self) -> Result<(), String>;
}

fn drain_monitored(
    mut reader: impl Read,
    limit: usize,
    monitor: &dyn StderrMonitor,
) -> Result<Vec<u8>, BoundedProcessError> {
    let mut bytes = Vec::new();
    let mut line = Vec::new();
    let mut buf = [0; 1024];
    loop {
        let count = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if count == 0 {
            if !line.is_empty() {
                capture_monitored_line(&line, &mut bytes, limit, monitor)?;
            }
            return Ok(bytes);
        }
        for byte in &buf[..count] {
            line.push(*byte);
            // Bound partial records too, without an unbounded read_until buffer.
            if line.len() > 4096 {
                return Err(BoundedProcessError::OutputTooLarge);
            }
            if *byte == b'\n' {
                capture_monitored_line(&line, &mut bytes, limit, monitor)?;
                line.clear();
            }
        }
    }
}

fn capture_monitored_line(
    line: &[u8],
    bytes: &mut Vec<u8>,
    limit: usize,
    monitor: &dyn StderrMonitor,
) -> Result<(), BoundedProcessError> {
    if !monitor.line(line)? {
        if line.len() > limit.saturating_sub(bytes.len()) {
            return Err(BoundedProcessError::OutputTooLarge);
        }
        bytes.extend_from_slice(line);
    }
    Ok(())
}
/// Execute a fixed process with bounded streams, deadline and descendant cleanup.
/// # Errors
/// Launch, timeout, output overflow and malformed UTF-8 remain explicit failures.
pub fn bounded_process(command: Command, timeout: Duration) -> Result<ProcessOutput, String> {
    bounded_process_with_limit(command, timeout, PROCESS_OUTPUT_LIMIT, &[])
        .map_err(|e| e.to_string())
}

/// Same suspended launch/kill-job supervisor with a caller's fixed stream limit.
pub fn bounded_process_with_limit(
    command: Command,
    timeout: Duration,
    limit: usize,
    input: &[u8],
) -> Result<ProcessOutput, BoundedProcessError> {
    supervise(command, timeout, limit, input, None)
}

pub(crate) fn bounded_process_with_monitor(
    command: Command,
    timeout: Duration,
    limit: usize,
    input: &[u8],
    monitor: Arc<dyn StderrMonitor>,
) -> Result<ProcessOutput, BoundedProcessError> {
    supervise(command, timeout, limit, input, Some(monitor))
}

fn supervise(
    mut command: Command,
    timeout: Duration,
    limit: usize,
    input: &[u8],
    monitor: Option<Arc<dyn StderrMonitor>>,
) -> Result<ProcessOutput, BoundedProcessError> {
    let end = Instant::now() + timeout;
    use std::os::windows::process::CommandExt;
    command.creation_flags(
        windows::Win32::System::Threading::CREATE_SUSPENDED.0
            | windows::Win32::System::Threading::CREATE_NO_WINDOW.0,
    );
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| BoundedProcessError::Launch {
            kind: e.kind(),
            code: e.raw_os_error(),
        })?;
    let job = match KillJob::attach(&child) {
        Ok(job) => job,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e.into());
        }
    };
    if let Err(error) = resume_suspended_child(child.id()) {
        drop(job);
        let _ = child.kill();
        let _ = child.wait();
        return Err(error.into());
    }
    let mut stdin = child.stdin.take().ok_or("missing stdin")?;
    let input = zeroize::Zeroizing::new(input.to_vec());
    let writer = thread::spawn(move || stdin.write_all(&input));
    let out = child.stdout.take().ok_or("missing stdout")?;
    let err = child.stderr.take().ok_or("missing stderr")?;
    let (sender, receiver) = std::sync::mpsc::channel();
    let sender2 = sender.clone();
    let output_thread = thread::spawn(move || {
        let result = drain(out, limit);
        let _ = sender.send((true, result));
    });
    let stderr_monitor = monitor.clone();
    let error_thread = thread::spawn(move || {
        let result = match stderr_monitor {
            Some(monitor) => drain_monitored(err, limit, &*monitor),
            None => drain(err, limit),
        };
        let _ = sender2.send((false, result));
    });
    let mut stdout = None;
    let mut stderr = None;
    let mut failure = None;
    let status = loop {
        while let Ok((is_out, result)) = receiver.try_recv() {
            match result {
                Ok(bytes) => {
                    if is_out {
                        stdout = Some(bytes);
                    } else {
                        stderr = Some(bytes);
                    }
                }
                Err(e) => failure = Some(e),
            }
        }
        if failure.is_some() || Instant::now() >= end {
            break None;
        }
        if let Some(monitor) = &monitor
            && let Err(error) = monitor.check(Instant::now())
        {
            failure = Some(BoundedProcessError::Execution(error));
            break None;
        }
        match child.try_wait() {
            Ok(Some(s)) => break Some(s),
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(e) => {
                failure = Some(BoundedProcessError::Execution(e.to_string()));
                break None;
            }
        }
    };
    // Terminate descendants even if the original process exited while they held streams.
    drop(job);
    if status.is_none() {
        let _ = child.kill();
    }
    child.wait().map_err(|e| e.to_string())?;
    let write_result = writer.join().map_err(|_| "stdin writer panicked")?;
    output_thread.join().map_err(|_| "stdout reader panicked")?;
    error_thread.join().map_err(|_| "stderr reader panicked")?;
    for (is_out, result) in receiver {
        match result {
            Ok(bytes) => {
                if is_out {
                    stdout = Some(bytes);
                } else {
                    stderr = Some(bytes);
                }
            }
            Err(e) => {
                if failure.is_none() {
                    failure = Some(e);
                }
            }
        }
    }
    if let Some(e) = failure {
        return Err(e);
    }
    let status = status.ok_or(BoundedProcessError::Timeout)?;
    // A child can reject the command before consuming input. Its failure status
    // and bounded diagnostics explain that refusal; a broken stdin pipe must not
    // hide them. A successful child must still have accepted the complete input.
    if status.success() {
        write_result.map_err(|e| e.to_string())?;
        if let Some(monitor) = &monitor {
            monitor.finish()?;
        }
    }
    Ok(ProcessOutput {
        exit_code: status.code(),
        stdout: String::from_utf8(stdout.ok_or("missing stdout result")?)
            .map_err(|_| "non-UTF-8 stdout")?,
        stderr: String::from_utf8(stderr.ok_or("missing stderr result")?)
            .map_err(|_| "non-UTF-8 stderr")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(unsafe_code)]
    fn background_payload_work_restores_priority_after_success_and_failure() {
        use windows::Win32::System::Threading::{GetCurrentThread, GetThreadPriority};
        // SAFETY: borrowed current thread pseudo-handle; read-only priority query.
        let original = unsafe { GetThreadPriority(GetCurrentThread()) };
        assert_eq!(background_work(|| Ok(17)).unwrap(), 17);
        assert_eq!(unsafe { GetThreadPriority(GetCurrentThread()) }, original);
        assert_eq!(
            background_work::<()>(|| Err("fixed failure".into())).unwrap_err(),
            "fixed failure"
        );
        assert_eq!(unsafe { GetThreadPriority(GetCurrentThread()) }, original);
    }

    #[test]
    #[allow(unsafe_code)]
    fn child_job_resource_limits_are_installed_before_launch() {
        use windows::Win32::System::JobObjects::{
            JOB_OBJECT_LIMIT_ACTIVE_PROCESS, JOB_OBJECT_LIMIT_JOB_MEMORY,
            JOBOBJECT_CPU_RATE_CONTROL_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JobObjectCpuRateControlInformation, JobObjectExtendedLimitInformation,
            QueryInformationJobObject,
        };
        let job = KillJob::create().unwrap();
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        let mut cpu = JOBOBJECT_CPU_RATE_CONTROL_INFORMATION::default();
        // SAFETY: owned empty job and correctly sized writable native buffers.
        unsafe {
            QueryInformationJobObject(
                Some(job.0),
                JobObjectExtendedLimitInformation,
                (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&limits) as u32,
                None,
            )
            .unwrap();
            QueryInformationJobObject(
                Some(job.0),
                JobObjectCpuRateControlInformation,
                (&mut cpu as *mut JOBOBJECT_CPU_RATE_CONTROL_INFORMATION).cast(),
                std::mem::size_of_val(&cpu) as u32,
                None,
            )
            .unwrap();
        }
        assert!(
            limits
                .BasicLimitInformation
                .LimitFlags
                .contains(JOB_OBJECT_LIMIT_JOB_MEMORY | JOB_OBJECT_LIMIT_ACTIVE_PROCESS)
        );
        assert_eq!(limits.JobMemoryLimit, CHILD_JOB_MEMORY_LIMIT);
        assert_eq!(
            limits.BasicLimitInformation.ActiveProcessLimit,
            CHILD_JOB_PROCESS_LIMIT
        );
        // SAFETY: configured CPU rate control selects the CpuRate arm.
        assert_eq!(unsafe { cpu.Anonymous.CpuRate }, CHILD_JOB_CPU_RATE);
    }

    #[test]
    fn powershell_reads_large_stdin_without_command_line_interpretation() {
        let mut command =
            Command::new(crate::windows_paths::windows_powershell_executable().unwrap());
        command.args([
            "-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
            "\n$ErrorActionPreference = 'Stop'\n$value = [Console]::In.ReadToEnd()\n[Console]::Out.Write($value.Length)\n",
        ]);
        let result =
            bounded_process_with_limit(command, Duration::from_secs(15), 1024, &vec![b'x'; 32768])
                .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(result.exit_code, Some(0), "{}", result.stderr);
        assert_eq!(result.stdout.trim(), "32768");
    }

    #[test]
    fn early_child_failure_retains_its_exit_status_and_diagnostics() {
        let mut command =
            Command::new(crate::windows_paths::windows_powershell_executable().unwrap());
        command.args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "[Console]::Error.Write('fixed failure'); exit 7",
        ]);
        let result = bounded_process_with_limit(
            command,
            Duration::from_secs(15),
            1024,
            &vec![b'x'; 1048576],
        )
        .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(result.exit_code, Some(7));
        assert_eq!(result.stderr, "fixed failure");
    }
}
