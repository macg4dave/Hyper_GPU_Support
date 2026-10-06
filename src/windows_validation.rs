//! Native guest validation and a fixed, bounded PowerShell Direct host transport.

use crate::{
    config::ProjectConfiguration,
    guest::GuestCredential,
    validation::{
        self, InputIdentity, PnpSample, ProcessOutput, ValidationAdapter, ValidationReport,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

const PROCESS_OUTPUT_LIMIT: usize = 4096;

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
pub fn os_build() -> Result<String, String> {
    crate::windows_driver_environment::operating_system_version().map_err(|e| e.to_string())
}

/// Deliberately inspect the current token before any known-admin transport operation.
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
    fn attach(child: &std::process::Child) -> Result<Self, String> {
        use std::os::windows::io::AsRawHandle;
        use windows::Win32::{
            Foundation::HANDLE,
            System::JobObjects::{
                AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
                JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
                SetInformationJobObject,
            },
        };
        // SAFETY: unnamed non-inheritable job with no external pointers.
        let job = Self(unsafe { CreateJobObjectW(None, None) }.map_err(|e| e.to_string())?);
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
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

fn drain(mut reader: impl Read) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut buf = [0; 1024];
    loop {
        let count = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if count == 0 {
            return Ok(bytes);
        }
        if bytes.len() + count > PROCESS_OUTPUT_LIMIT {
            return Err("process output limit exceeded".into());
        }
        bytes.extend_from_slice(&buf[..count]);
    }
}
/// Execute a fixed process with bounded streams, deadline and descendant cleanup.
/// # Errors
/// Launch, timeout, output overflow and malformed UTF-8 remain explicit failures.
pub(crate) fn bounded_process(
    mut command: Command,
    timeout: Duration,
) -> Result<ProcessOutput, String> {
    let end = Instant::now() + timeout;
    use std::os::windows::process::CommandExt;
    command.creation_flags(
        windows::Win32::System::Threading::CREATE_SUSPENDED.0
            | windows::Win32::System::Threading::CREATE_NO_WINDOW.0,
    );
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let job = match KillJob::attach(&child) {
        Ok(job) => job,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e);
        }
    };
    if let Err(error) = resume_suspended_child(child.id()) {
        drop(job);
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    let out = child.stdout.take().ok_or("missing stdout")?;
    let err = child.stderr.take().ok_or("missing stderr")?;
    let (sender, receiver) = std::sync::mpsc::channel();
    let sender2 = sender.clone();
    let output_thread = thread::spawn(move || {
        let result = drain(out);
        let _ = sender.send((true, result));
    });
    let error_thread = thread::spawn(move || {
        let result = drain(err);
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
        match child.try_wait() {
            Ok(Some(s)) => break Some(s),
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(e) => {
                failure = Some(e.to_string());
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
            Err(e) => failure = Some(e),
        }
    }
    if let Some(e) = failure {
        return Err(e);
    }
    let status = status.ok_or("process deadline expired; process and descendants terminated")?;
    Ok(ProcessOutput {
        exit_code: status.code(),
        stdout: String::from_utf8(stdout.ok_or("missing stdout result")?)
            .map_err(|_| "non-UTF-8 stdout")?,
        stderr: String::from_utf8(stderr.ok_or("missing stderr result")?)
            .map_err(|_| "non-UTF-8 stderr")?,
    })
}

/// Local effects used only by the explicitly launched validation worker.
pub struct LocalValidationAdapter {
    root: PathBuf,
    started: Instant,
}
impl LocalValidationAdapter {
    /// Bind to the fixed protected artifact directory.
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            started: Instant::now(),
        }
    }
}
impl ValidationAdapter for LocalValidationAdapter {
    fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
    fn wait(&mut self, duration: Duration) {
        thread::sleep(duration);
    }
    fn sample(&mut self) -> Result<PnpSample, String> {
        let (id, name, problem) = crate::windows_driver_environment::virtual_render_sample(
            Instant::now() + Duration::from_secs(5),
        )
        .map_err(|e| e.to_string())?;
        Ok(PnpSample {
            elapsed_ms: self.elapsed().as_millis() as u64,
            device_id: id,
            name,
            problem,
        })
    }
    fn run(&mut self, name: &str, timeout: Duration) -> Result<ProcessOutput, String> {
        let (path, args) = match name {
            "nvidia-smi" => (
                crate::windows_paths::windows_directory()
                    .map_err(|e| e.to_string())?
                    .join("System32\\nvidia-smi.exe"),
                vec![
                    "--query-gpu=name,driver_version,uuid",
                    "--format=csv,noheader",
                ],
            ),
            "d3d11" => (self.root.join("d3d11-probe.exe"), vec![]),
            "d3d12" => (self.root.join("d3d12-probe.exe"), vec![]),
            "cuda-identity" => (self.root.join("cuda-identity.exe"), vec!["--guest-compute"]),
            "cuda" => (self.root.join("vectorAddDrv.exe"), vec!["--device=0"]),
            _ => return Err("unknown fixed workload".into()),
        };
        if validation::unsafe_path(&path)? {
            return Err("unsafe workload path".into());
        }
        let mut command = Command::new(path);
        command.args(args).current_dir(&self.root);
        bounded_process(command, timeout)
    }
}

/// Worker request contains input identities only. Configuration is embedded;
/// it cannot select arbitrary paths, programs, arguments or commands.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerRequest {
    /// Complete nine-file artifact inventory.
    pub inputs: Vec<InputIdentity>,
}

/// Host validation preflight, safe to call without elevation or credentials.
/// # Errors
/// Missing/unsafe artifacts fail before any session or guest mutation.
pub fn collect_inputs(
    project: &ProjectConfiguration,
) -> Result<Vec<(PathBuf, InputIdentity)>, String> {
    let mut result = Vec::new();
    for name in validation::INPUT_NAMES {
        let directory = if name.ends_with(".dll") {
            project.validation.crt_directory.clone()
        } else if name.starts_with("vectorAdd") {
            project.validation.cuda_directory.clone()
        } else {
            project.runner.artifacts.directory.clone()
        };
        let root_item = std::fs::symlink_metadata(&directory)
            .map_err(|e| format!("missing prerequisite directory for {name}: {e}"))?;
        use std::os::windows::fs::MetadataExt;
        if !root_item.is_dir() || root_item.file_attributes() & 0x400 != 0 {
            return Err("unsafe prerequisite root".into());
        }
        // Resolve operator-selected roots once (the repository's container is a
        // junction), then forbid reparse inputs within those resolved roots.
        let root = directory.canonicalize().map_err(|e| e.to_string())?;
        let path = root.join(name);
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|e| format!("missing prerequisite {name}: {e}"))?;
        if !metadata.is_file()
            || validation::unsafe_path_with_anchor(&path, &root)?
            || metadata.len() == 0
            || metadata.len() > 64 * 1024 * 1024
        {
            return Err(format!("unsafe prerequisite {name}"));
        }
        let hash = crate::guest::hash_file(&path).map_err(|e| e.to_string())?;
        result.push((
            path,
            InputIdentity {
                name: name.into(),
                bytes: metadata.len(),
                sha256: hash,
            },
        ));
    }
    Ok(result)
}

#[derive(Serialize)]
struct HostRequest<'a> {
    vm_id: &'a str,
    vm_name: &'a str,
    child_path: &'a Path,
    parent_path: &'a Path,
    gpu_interface: &'a str,
    computer_name: &'a str,
    machine_guid: &'a str,
    root: PathBuf,
    inputs: &'a [(PathBuf, InputIdentity)],
    worker_json: String,
    username: &'a str,
    password: &'a str,
    timeout_ms: u64,
}

/// Run only the configured fixed worker via PowerShell Direct.
/// # Errors
/// Requires an elevated development token, verified target and verified inputs.
/// A transport timeout does not claim the remote worker has stopped; its own
/// finite watchdog and process jobs bound remote execution independently.
pub fn validate_guest(
    project: &ProjectConfiguration,
    credential: &GuestCredential,
    inputs: &[(PathBuf, InputIdentity)],
) -> Result<ValidationReport, String> {
    if project != &ProjectConfiguration::embedded().map_err(|e| e.to_string())? {
        return Err("validation target/configuration differs from compiled authorization".into());
    }
    if collect_inputs(project)? != inputs {
        return Err("validation inputs differ from fresh configured sources".into());
    }
    if !is_elevated()? {
        return Err("validation transport requires an elevated development token".into());
    }
    use std::os::windows::fs::OpenOptionsExt;
    let _lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .share_mode(0)
        .open(project.runner.data_directory.join("state\\operation.lock"))
        .map_err(|e| format!("validation operation lock: {e}"))?;
    let identities = inputs.iter().map(|(_, i)| i.clone()).collect();
    let request = HostRequest {
        vm_id: &project.slot.vm_id,
        vm_name: &project.slot.vm_name,
        child_path: &project.slot.child_path,
        parent_path: &project.slot.parent_path,
        gpu_interface: &project.slot.gpu_interface,
        computer_name: &project.guest.computer_name,
        machine_guid: &project.guest.machine_guid,
        root: project.guest.staging_root.join("Validation"),
        inputs,
        worker_json: serde_json::to_string(&WorkerRequest { inputs: identities })
            .map_err(|e| e.to_string())?,
        username: credential.username(),
        password: credential.password(),
        timeout_ms: project.validation.worker_timeout_seconds * 1000,
    };
    let payload = Zeroizing::new(serde_json::to_vec(&request).map_err(|e| e.to_string())?);
    if payload.len() > 16 * 1024 {
        return Err("validation request exceeds bound".into());
    }
    let script =
        crate::windows_guest::expand_staging_script(include_str!("validation_transport.ps1"));
    let output = crate::windows_guest::run_script(
        &project.guest.powershell_path,
        &script,
        payload,
        Duration::from_secs(project.validation.worker_timeout_seconds)
            + project.guest.transfer_timeout
            + project.guest.session_timeout,
    )
    .map_err(|e| e.to_string())?;
    let report: ValidationReport =
        serde_json::from_str(output.trim()).map_err(|_| "invalid validation response")?;
    verify_report(&report, project, inputs)?;
    Ok(report)
}

fn verify_report(
    report: &ValidationReport,
    project: &ProjectConfiguration,
    inputs: &[(PathBuf, InputIdentity)],
) -> Result<(), String> {
    if report.schema != 1
        || report.vm_id != project.slot.vm_id
        || report.architecture != "x64"
        || report.guest_build.is_empty()
        || report.checks.len() != validation::CHECK_NAMES.len()
        || report
            .checks
            .iter()
            .zip(validation::CHECK_NAMES)
            .any(|(c, n)| c.name != n)
        || report.inputs != inputs.iter().map(|(_, i)| i.clone()).collect::<Vec<_>>()
    {
        return Err("validation response identity mismatch".into());
    }
    crate::validation::verify_report_evidence(report, project)?;
    Ok(())
}

/// Public validate path: preflight first, credentials locally, then the fixed adapter.
/// # Errors
/// Configuration and terminal failures preserve their native error details.
pub fn validate_command() -> Result<ValidationReport, String> {
    use std::io::Write;
    let project = ProjectConfiguration::embedded().map_err(|e| e.to_string())?;
    let mut report = ValidationReport::new(&project);
    let inputs = match collect_inputs(&project) {
        Ok(inputs) => inputs,
        Err(e) => {
            report.block(e);
            return Ok(report);
        }
    };
    if !is_elevated()? {
        report.block("PowerShell Direct validation requires an elevated development token");
        return Ok(report);
    }
    eprint!("Guest username: ");
    std::io::stderr().flush().map_err(|e| e.to_string())?;
    let mut username = String::new();
    std::io::stdin()
        .read_line(&mut username)
        .map_err(|e| e.to_string())?;
    let password = rpassword::prompt_password("Guest password: ").map_err(|e| e.to_string())?;
    let credential =
        GuestCredential::new(username.trim_end_matches(['\r', '\n']).to_owned(), password)
            .map_err(|e| e.to_string())?;
    match validate_guest(&project, &credential, &inputs) {
        Ok(result) => Ok(result),
        Err(e) => {
            report.block(e);
            Ok(report)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn powershell(body: &str) -> Command {
        let mut command = Command::new(
            ProjectConfiguration::embedded()
                .unwrap()
                .guest
                .powershell_path,
        );
        command.args(["-NoProfile", "-NonInteractive", "-Command", body]);
        command
    }
    #[test]
    fn process_preserves_exit_and_bounds_timeout_and_output() {
        let output = bounded_process(
            powershell("[Console]::Out.Write('out');[Console]::Error.Write('err');exit 4"),
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(output.exit_code, Some(4));
        assert_eq!(output.stdout, "out");
        assert_eq!(output.stderr, "err");
        let start = Instant::now();
        assert!(
            bounded_process(
                powershell("Start-Sleep -Seconds 10"),
                Duration::from_millis(150)
            )
            .unwrap_err()
            .contains("deadline")
        );
        assert!(start.elapsed() < Duration::from_secs(5));
        assert!(
            bounded_process(
                powershell("[Console]::Out.Write(('x'*5000))"),
                Duration::from_secs(5)
            )
            .unwrap_err()
            .contains("output limit")
        );
    }
    #[test]
    fn native_transport_parses_without_privileged_effects() {
        let project = ProjectConfiguration::embedded().unwrap();
        let script =
            crate::windows_guest::expand_staging_script(include_str!("validation_transport.ps1"));
        assert!(!script.contains("__"));
        let output=crate::windows_guest::run_script(&project.guest.powershell_path,"$tokens=$null;$errors=$null;[System.Management.Automation.Language.Parser]::ParseInput([Console]::In.ReadToEnd(),[ref]$tokens,[ref]$errors)|Out-Null;if($errors.Count -ne 0){exit 3};[Console]::Out.Write('valid')",Zeroizing::new(script.into_bytes()),Duration::from_secs(10)).unwrap();
        assert_eq!(output, "valid");
    }
    #[test]
    fn worker_request_refuses_caller_selected_programs_and_wrong_configuration() {
        assert!(
            serde_json::from_str::<WorkerRequest>(r#"{"inputs":[],"command":"shutdown"}"#).is_err()
        );
        let mut project = ProjectConfiguration::embedded().unwrap();
        project.slot.vm_id = "wrong".into();
        let credential = GuestCredential::new("fixture".into(), "fixture".into()).unwrap();
        assert!(
            validate_guest(&project, &credential, &[])
                .unwrap_err()
                .contains("compiled authorization")
        );
    }

    #[test]
    fn deadline_child_entry() {
        if std::env::var("HYPER_GPU_VALIDATION_DEADLINE_FIXTURE").as_deref() == Ok("armed") {
            let _deadline = WorkerDeadline::start(Duration::from_millis(100));
            // Simulate an uncancellable native call after loss of its host controller.
            loop {
                thread::park();
            }
        }
    }
    #[test]
    fn hard_worker_deadline_survives_blocked_native_call_and_absent_controller() {
        let mut child = Command::new(std::env::current_exe().unwrap());
        child
            .args([
                "--exact",
                "windows_validation::tests::deadline_child_entry",
                "--nocapture",
            ])
            .env("HYPER_GPU_VALIDATION_DEADLINE_FIXTURE", "armed");
        let result = bounded_process(child, Duration::from_secs(5)).unwrap();
        assert_eq!(result.exit_code, Some(6));
        let (sender, receiver) = std::sync::mpsc::channel();
        let deadline = WorkerDeadline::with_expiry(Duration::from_secs(5), move || {
            let _ = sender.send(());
        });
        drop(deadline);
        assert!(receiver.recv_timeout(Duration::from_millis(50)).is_err());
    }
    #[test]
    #[allow(unsafe_code)]
    fn immediate_descendant_is_contained_before_parent_executes() {
        use windows::Win32::{
            Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, WAIT_OBJECT_0},
            System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
        };
        let result=bounded_process(powershell("$child=Start-Process (Join-Path $PSHOME 'powershell.exe') -WindowStyle Hidden -ArgumentList '-NoProfile -NonInteractive -Command Start-Sleep -Seconds 10' -PassThru;[Console]::Out.Write($child.Id)"),Duration::from_secs(5)).unwrap();
        assert_eq!(result.exit_code, Some(0));
        let id = result.stdout.trim().parse().unwrap();
        // SAFETY: open only the exact PID emitted by this test's contained parent.
        match unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, id) } {
            Ok(handle) => {
                // SAFETY: owned handle to the test's exact child; observe bounded exit.
                let status = unsafe { WaitForSingleObject(handle, 1000) };
                // SAFETY: close the handle owned by this test.
                unsafe { CloseHandle(handle) }.unwrap();
                assert_eq!(status, WAIT_OBJECT_0);
            }
            Err(e) => assert_eq!(
                e.code(),
                windows::core::HRESULT::from_win32(ERROR_INVALID_PARAMETER.0)
            ),
        }
    }
    #[test]
    fn permissive_existing_file_acl_is_refused_before_elevated_execution() {
        let script = format!(
            "{}\nfunction Get-Item {{ param($LiteralPath,[switch]$Force) [pscustomobject]@{{PSIsContainer=$false;Attributes=[IO.FileAttributes]0}} }}\nfunction Get-Acl {{ param($LiteralPath) $acl=[Security.AccessControl.FileSecurity]::new();$acl.SetOwner([Security.Principal.WindowsIdentity]::GetCurrent().User);$rule=[Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new('S-1-5-32-545'),[Security.AccessControl.FileSystemRights]::Write,[Security.AccessControl.AccessControlType]::Allow);$acl.AddAccessRule($rule);$acl }}\ntry {{ Assert-ProtectedFile 'fixture';exit 3 }} catch {{ if ($_.Exception.Message -cne 'unsafe path') {{throw}};[Console]::Out.Write('refused') }}",
            crate::windows_guest::ACL_VALIDATOR
        );
        let output = bounded_process(powershell(&script), Duration::from_secs(5)).unwrap();
        assert_eq!(output.exit_code, Some(0), "{}", output.stderr);
        assert_eq!(output.stdout, "refused");
    }
}
