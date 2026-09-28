//! Administrator-installed fixed runner for the enrolled disposable Hyper-V slot.

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use hyper_gpu_support::runner::{
    Enrollment, Operation, PIPE_NAME, Request, Response, authorize_request, consume_nonce,
    parse_enrollment, parse_inspect_result, parse_lifecycle_result, parse_request,
    parse_reset_result, policy_allows, policy_fingerprint, verify_policy,
};
use hyper_gpu_support::windows_runner::serve_one;

const INSTALL_ROOT: &str = r"C:\ProgramData\HyperGpuSupport\Runner";
const POLICY_PATH: &str = r"C:\ProgramData\HyperGpuSupport\Runner\policy-v1.json";
const ENROLLMENT_PATH: &str = r"C:\ProgramData\HyperGpuSupport\Runner\enrollment-v1.json";
const STARTUP_FAILURE_PATH: &str =
    r"C:\ProgramData\HyperGpuSupport\Runner\audit\runner-startup-failure-v1.txt";
const RECONCILIATION_MARKER: &str = "reconciliation-required-v1";
const RESET_TIMEOUT: Duration = Duration::from_secs(300);
const INSPECT_TIMEOUT: Duration = Duration::from_secs(300);
const START_TIMEOUT: Duration = Duration::from_secs(180);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(120);
const OUTPUT_LIMIT: usize = 64 * 1024;

fn main() -> ExitCode {
    let _ = fs::remove_file(STARTUP_FAILURE_PATH);
    let mut arguments = std::env::args_os().skip(1);
    let mode = arguments.next();
    if arguments.next().is_some() {
        eprintln!("runner error: exactly one fixed mode is required");
        return ExitCode::FAILURE;
    }
    let outcome = match mode.as_deref() {
        Some(value) if value == "serve-once" => run_server(),
        _ => Err("only fixed serve-once mode is accepted".into()),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = fs::write(
                STARTUP_FAILURE_PATH,
                startup_failure_message(error.as_ref()),
            );
            eprintln!("runner error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn startup_failure_message(error: &dyn std::fmt::Display) -> String {
    let mut message = String::with_capacity(513);
    for character in error.to_string().chars() {
        let character = if character.is_control() {
            ' '
        } else {
            character
        };
        if message.len() + character.len_utf8() > 512 {
            break;
        }
        message.push(character);
    }
    message.push('\n');
    message
}

fn prepare_install_state() -> Result<Enrollment, Box<dyn std::error::Error>> {
    let installed_policy = fs::read_to_string(POLICY_PATH)?;
    verify_policy(&installed_policy)?;
    let enrollment = parse_enrollment(&fs::read_to_string(ENROLLMENT_PATH)?)?;

    let state_directory = Path::new(INSTALL_ROOT).join("state");
    let result_directory = Path::new(INSTALL_ROOT).join("results");
    let audit_directory = Path::new(INSTALL_ROOT).join("audit");
    fs::create_dir_all(&state_directory)?;
    fs::create_dir_all(&result_directory)?;
    fs::create_dir_all(&audit_directory)?;
    fs::create_dir_all(state_directory.join("nonces"))?;
    Ok(enrollment)
}

fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let enrollment = prepare_install_state()?;
    serve_one(
        PIPE_NAME,
        enrollment.client_sid(),
        &enrollment.pipe_sddl(),
        |input| {
            let response = handle_request(input);
            Ok(response.encode().into_bytes())
        },
    )?;
    Ok(())
}

fn handle_request(input: &[u8]) -> Response {
    let parsed = std::str::from_utf8(input)
        .map_err(|_| "invalid-request")
        .and_then(|input| parse_request(input).map_err(|_| "invalid-request"));
    let request = match parsed {
        Ok(request) => request,
        Err(diagnostic) => {
            let response = failed_response(None, diagnostic);
            let _ = append_protocol_audit(None, "rejected", diagnostic);
            return response;
        }
    };
    if append_protocol_audit(Some(&request), "received", "none").is_err() {
        return failed_response(Some(&request), "audit-unavailable");
    }
    match execute_request(&request) {
        Ok(operation_id) => {
            if append_protocol_audit(Some(&request), "succeeded", "none").is_err() {
                return Response {
                    request_id: request.request_id.clone(),
                    status: "failed".into(),
                    operation_id: Some(operation_id),
                    diagnostic: "audit-finalization-failed".into(),
                };
            }
            Response {
                request_id: request.request_id.clone(),
                status: "succeeded".into(),
                operation_id: Some(operation_id),
                diagnostic: "none".into(),
            }
        }
        Err(failure) => {
            if append_protocol_audit(Some(&request), "failed", failure.diagnostic).is_err() {
                Response {
                    request_id: request.request_id.clone(),
                    status: "failed".into(),
                    operation_id: failure.operation_id,
                    diagnostic: "audit-finalization-failed".into(),
                }
            } else {
                Response {
                    request_id: request.request_id.clone(),
                    status: "failed".into(),
                    operation_id: failure.operation_id,
                    diagnostic: failure.diagnostic.into(),
                }
            }
        }
    }
}

fn append_protocol_audit(
    request: Option<&Request>,
    status: &str,
    diagnostic: &str,
) -> Result<(), std::io::Error> {
    let path = Path::new(INSTALL_ROOT).join("audit").join("events.jsonl");
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let request_id = request.map_or_else(|| "0".repeat(32), |value| value.request_id.clone());
    let operation = request.map_or("none", |value| value.operation.as_str());
    let nonce = request.map_or_else(|| "0".repeat(32), |value| value.nonce.clone());
    let plan = request.map_or_else(|| "0".repeat(64), |value| value.plan_fingerprint.clone());
    writeln!(
        file,
        concat!(
            "{{\"schema\":1,\"request_id\":\"{}\",\"operation\":\"{}\",",
            "\"slot\":\"gpu-pv-slot-01\",",
            "\"vm_id\":\"2627e735-5b33-4104-b739-622727dd3a40\",",
            "\"nonce\":\"{}\",\"plan_fingerprint\":\"{}\",",
            "\"status\":\"{}\",\"diagnostic\":\"{}\"}}"
        ),
        request_id,
        operation,
        nonce,
        plan,
        status,
        json_escape(diagnostic)
    )?;
    file.sync_all()
}

fn failed_response(request: Option<&Request>, diagnostic: &str) -> Response {
    Response {
        request_id: request.map_or_else(|| "0".repeat(32), |value| value.request_id.clone()),
        status: "failed".into(),
        operation_id: None,
        diagnostic: diagnostic.into(),
    }
}

struct ExecutionFailure {
    diagnostic: &'static str,
    operation_id: Option<String>,
}

fn execute_request(request: &Request) -> Result<String, ExecutionFailure> {
    let before_effect = |diagnostic| ExecutionFailure {
        diagnostic,
        operation_id: None,
    };
    if !policy_allows(request.operation) {
        return Err(before_effect("operation-denied"));
    }
    ensure_reconciled(&Path::new(INSTALL_ROOT).join("state"), request.operation)
        .map_err(before_effect)?;
    let mut process_nonces = BTreeSet::new();
    authorize_request(
        request,
        request.operation,
        &policy_fingerprint(),
        &mut process_nonces,
    )
    .map_err(|error| {
        before_effect(match error {
            hyper_gpu_support::runner::RunnerError::StalePlan => "stale-plan",
            _ => "authorization-failed",
        })
    })?;
    consume_nonce(
        &Path::new(INSTALL_ROOT).join("state").join("nonces"),
        &request.nonce,
    )
    .map_err(|error| {
        before_effect(match error {
            hyper_gpu_support::runner::PersistentStateError::Protocol(
                hyper_gpu_support::runner::RunnerError::Replay,
            ) => "replay",
            _ => "replay-state-failed",
        })
    })?;
    match request.operation {
        Operation::Inspect => run_inspect(request).map_err(|_| before_effect("operation-failed")),
        Operation::ResetSlot => run_reset(request).map_err(|_| before_effect("operation-failed")),
        Operation::StartSlot => {
            execute_lifecycle_request(request, Operation::StartSlot, START_SCRIPT, START_TIMEOUT)
        }
        Operation::ShutdownSlot => execute_lifecycle_request(
            request,
            Operation::ShutdownSlot,
            SHUTDOWN_SCRIPT,
            SHUTDOWN_TIMEOUT,
        ),
        _ => Err(before_effect("operation-denied")),
    }
}

fn ensure_reconciled(state_directory: &Path, operation: Operation) -> Result<(), &'static str> {
    if operation == Operation::Inspect {
        return Ok(());
    }
    match state_directory.join(RECONCILIATION_MARKER).try_exists() {
        Ok(false) => Ok(()),
        Ok(true) => Err("reconciliation-required"),
        Err(_) => Err("reconciliation-state-unavailable"),
    }
}

fn begin_reconciliation(
    path: &Path,
    operation: Operation,
    operation_id: &str,
) -> std::io::Result<()> {
    let mut marker = OpenOptions::new().write(true).create_new(true).open(path)?;
    writeln!(marker, "{} {}", operation.as_str(), operation_id)?;
    marker.sync_all()
}

fn execute_lifecycle_request(
    request: &Request,
    operation: Operation,
    script: &str,
    timeout: Duration,
) -> Result<String, ExecutionFailure> {
    let operation_id = operation_id().map_err(|_| ExecutionFailure {
        diagnostic: "operation-id-failed",
        operation_id: None,
    })?;
    match run_lifecycle(request, operation, &operation_id, script, timeout) {
        Ok(()) => Ok(operation_id),
        Err(_) => Err(ExecutionFailure {
            diagnostic: "operation-failed-reconciliation-required",
            operation_id: Some(operation_id),
        }),
    }
}

fn run_reset(request: &Request) -> Result<String, Box<dyn std::error::Error>> {
    let _enrollment = prepare_install_state()?;
    let state_directory = Path::new(INSTALL_ROOT).join("state");
    let result_directory = Path::new(INSTALL_ROOT).join("results");
    let audit_directory = Path::new(INSTALL_ROOT).join("audit");
    let lock_path = state_directory.join("operation.lock");
    let _lock = LockFile::acquire(&lock_path)?;
    ensure_reconciled(&state_directory, Operation::ResetSlot).map_err(std::io::Error::other)?;

    let operation_id = operation_id()?;
    append_audit(
        &audit_directory,
        &operation_id,
        &request.request_id,
        Operation::ResetSlot,
        "started",
        "",
    )?;
    let output = match run_bounded(RESET_SCRIPT, RESET_TIMEOUT) {
        Ok(output) => output,
        Err(error) => {
            append_audit(
                &audit_directory,
                &operation_id,
                &request.request_id,
                Operation::ResetSlot,
                "failed",
                &error,
            )?;
            return Err(error.into());
        }
    };
    let result = parse_reset_result(&output)?;
    let result_json = format!(
        concat!(
            "{{\n  \"schema\": 1,\n  \"request_id\": \"{}\",\n  \"operation_id\": \"{}\",\n",
            "  \"operation\": \"reset-slot\",\n  \"status\": \"succeeded\",\n",
            "  \"vm_id\": \"{}\",\n  \"child\": \"{}\",\n",
            "  \"parent\": \"{}\",\n  \"parent_sha256\": \"{}\"\n}}\n"
        ),
        request.request_id,
        operation_id,
        result.vm_id,
        json_escape(&result.child),
        json_escape(&result.parent),
        result.parent_sha256
    );
    write_atomic(&result_directory, &operation_id, &result_json)?;
    append_audit(
        &audit_directory,
        &operation_id,
        &request.request_id,
        Operation::ResetSlot,
        "succeeded",
        "",
    )?;
    Ok(operation_id)
}

fn run_inspect(request: &Request) -> Result<String, Box<dyn std::error::Error>> {
    let _enrollment = prepare_install_state()?;
    let state_directory = Path::new(INSTALL_ROOT).join("state");
    let result_directory = Path::new(INSTALL_ROOT).join("results");
    let audit_directory = Path::new(INSTALL_ROOT).join("audit");
    let lock_path = state_directory.join("operation.lock");
    let _lock = LockFile::acquire(&lock_path)?;
    let operation_id = operation_id()?;
    append_audit(
        &audit_directory,
        &operation_id,
        &request.request_id,
        Operation::Inspect,
        "started",
        "",
    )?;
    let output = match run_bounded(INSPECT_SCRIPT, INSPECT_TIMEOUT) {
        Ok(output) => output,
        Err(error) => {
            append_audit(
                &audit_directory,
                &operation_id,
                &request.request_id,
                Operation::Inspect,
                "failed",
                &error,
            )?;
            return Err(error.into());
        }
    };
    let result = parse_inspect_result(&output)?;
    let result_json = format!(
        concat!(
            "{{\n  \"schema\": 1,\n  \"request_id\": \"{}\",\n  \"operation_id\": \"{}\",\n",
            "  \"operation\": \"inspect\",\n  \"status\": \"succeeded\",\n",
            "  \"vm_id\": \"{}\",\n  \"state\": \"{}\",\n",
            "  \"gpu_adapters\": {},\n  \"child\": \"{}\",\n",
            "  \"parent\": \"{}\",\n  \"parent_sha256\": \"{}\",\n",
            "  \"gpu_interface\": \"{}\"\n}}\n"
        ),
        request.request_id,
        operation_id,
        result.vm_id,
        result.state,
        result.gpu_adapters,
        json_escape(&result.child),
        json_escape(&result.parent),
        result.parent_sha256,
        json_escape(&result.gpu_interface)
    );
    write_atomic(&result_directory, &operation_id, &result_json)?;
    append_audit(
        &audit_directory,
        &operation_id,
        &request.request_id,
        Operation::Inspect,
        "succeeded",
        "",
    )?;
    Ok(operation_id)
}

fn run_lifecycle(
    request: &Request,
    operation: Operation,
    operation_id: &str,
    script: &str,
    timeout: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let _enrollment = prepare_install_state()?;
    let state_directory = Path::new(INSTALL_ROOT).join("state");
    let result_directory = Path::new(INSTALL_ROOT).join("results");
    let audit_directory = Path::new(INSTALL_ROOT).join("audit");
    let _lock = LockFile::acquire(&state_directory.join("operation.lock"))?;
    append_audit(
        &audit_directory,
        operation_id,
        &request.request_id,
        operation,
        "started",
        "",
    )?;
    let reconciliation_path = state_directory.join(RECONCILIATION_MARKER);
    begin_reconciliation(&reconciliation_path, operation, operation_id)?;

    let operation_result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let inspection_output =
            run_bounded(INSPECT_SCRIPT, INSPECT_TIMEOUT).map_err(std::io::Error::other)?;
        let inspection = parse_inspect_result(&inspection_output)?;
        let expected_state = match operation {
            Operation::StartSlot => "Off",
            Operation::ShutdownSlot => "Running",
            _ => return Err("invalid lifecycle operation".into()),
        };
        if inspection.state != expected_state {
            return Err("lifecycle preflight state mismatch".into());
        }
        let output = run_bounded(script, timeout).map_err(std::io::Error::other)?;
        let result = parse_lifecycle_result(&output, operation)?;
        let result_json = format!(
            concat!(
                "{{\n  \"schema\": 1,\n  \"request_id\": \"{}\",\n  \"operation_id\": \"{}\",\n",
                "  \"operation\": \"{}\",\n  \"status\": \"succeeded\",\n",
                "  \"vm_id\": \"{}\",\n  \"previous_state\": \"{}\",\n",
                "  \"state\": \"{}\",\n  \"gpu_adapters\": {},\n",
                "  \"child\": \"{}\",\n  \"parent\": \"{}\",\n",
                "  \"parent_sha256\": \"{}\"\n}}\n"
            ),
            request.request_id,
            operation_id,
            operation.as_str(),
            result.vm_id,
            result.previous_state,
            result.state,
            result.gpu_adapters,
            json_escape(&result.child),
            json_escape(&result.parent),
            result.parent_sha256
        );
        write_atomic(&result_directory, operation_id, &result_json)?;
        Ok(())
    })();
    if let Err(error) = operation_result {
        let _ = append_audit(
            &audit_directory,
            operation_id,
            &request.request_id,
            operation,
            "failed-reconciliation-required",
            &error.to_string(),
        );
        let failure_json = format!(
            concat!(
                "{{\n  \"schema\": 1,\n  \"request_id\": \"{}\",\n",
                "  \"operation_id\": \"{}\",\n  \"operation\": \"{}\",\n",
                "  \"status\": \"failed\",\n  \"reconciliation_required\": true,\n",
                "  \"diagnostic\": \"{}\"\n}}\n"
            ),
            request.request_id,
            operation_id,
            operation.as_str(),
            json_escape(&error.to_string())
        );
        let _ = write_atomic(&result_directory, operation_id, &failure_json);
        return Err(error);
    }
    append_audit(
        &audit_directory,
        operation_id,
        &request.request_id,
        operation,
        "succeeded",
        "",
    )?;
    fs::remove_file(reconciliation_path)?;
    Ok(())
}

struct LockFile {
    _file: File,
}

impl LockFile {
    fn acquire(path: &Path) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .share_mode(0)
            .open(path)?;
        Ok(Self { _file: file })
    }
}

fn operation_id() -> Result<String, std::time::SystemTimeError> {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(format!(
        "{}-{:09}",
        duration.as_secs(),
        duration.subsec_nanos()
    ))
}

fn append_audit(
    directory: &Path,
    operation_id: &str,
    request_id: &str,
    operation: Operation,
    status: &str,
    diagnostic: &str,
) -> Result<(), std::io::Error> {
    let path = directory.join("events.jsonl");
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(
        file,
        "{{\"request_id\":\"{}\",\"operation_id\":\"{}\",\"operation\":\"{}\",\"status\":\"{}\",\"diagnostic\":\"{}\"}}",
        request_id,
        operation_id,
        operation.as_str(),
        status,
        json_escape(diagnostic)
    )?;
    file.sync_all()
}

fn write_atomic(directory: &Path, operation_id: &str, contents: &str) -> std::io::Result<()> {
    let temporary = directory.join(format!("{operation_id}.tmp"));
    let final_path = directory.join(format!("{operation_id}.json"));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()?;
    drop(file);
    fs::rename(temporary, final_path)
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

fn run_bounded(script: &str, timeout: Duration) -> Result<String, String> {
    let mut child = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot launch fixed Hyper-V adapter: {error}"))?;
    let stdout = child.stdout.take().ok_or("stdout unavailable")?;
    let stderr = child.stderr.take().ok_or("stderr unavailable")?;
    let overflow = Arc::new(AtomicBool::new(false));
    let stdout_overflow = Arc::clone(&overflow);
    let stderr_overflow = Arc::clone(&overflow);
    let stdout_reader = thread::spawn(move || read_bounded(stdout, &stdout_overflow));
    let stderr_reader = thread::spawn(move || read_bounded(stderr, &stderr_overflow));
    let deadline = Instant::now() + timeout;
    let status = loop {
        if overflow.load(Ordering::Acquire) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err("fixed adapter output exceeded limit".into());
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(50)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("fixed Hyper-V adapter timed out".into());
            }
            Err(error) => return Err(format!("cannot wait for fixed adapter: {error}")),
        }
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| "stdout reader failed")?
        .map_err(|error| error.to_string())?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| "stderr reader failed")?
        .map_err(|error| error.to_string())?;
    if !status.success() {
        return Err(format!(
            "fixed adapter failed with {:?}: {}",
            status.code(),
            String::from_utf8_lossy(&stderr)
                .chars()
                .filter(|character| !character.is_control() || *character == ' ')
                .take(512)
                .collect::<String>()
        ));
    }
    String::from_utf8(stdout).map_err(|_| "fixed adapter output was not UTF-8".into())
}

fn read_bounded(mut reader: impl Read, overflow: &AtomicBool) -> std::io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            return Ok(output);
        }
        if output.len() + count > OUTPUT_LIMIT {
            overflow.store(true, Ordering::Release);
            return Err(std::io::Error::other("adapter output exceeded limit"));
        }
        output.extend_from_slice(&buffer[..count]);
    }
}

const RESET_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$vmId = [guid]'2627E735-5B33-4104-B739-622727DD3A40'
$vmName = 'HyperGpuSupport-Disposable-01'
$parentPath = 'Z:\HyperGpuSupport\images\golden\win11-pro-25h2-26200.9457-x64-v1\parent.vhdx'
$parentHash = '0fb4dfe6dd51eed64d36e482e4f58d67c19922802e34aa6ae2d1f4ccd5daeb07'
$childPath = 'Z:\HyperGpuSupport\images\disposable\gpu-pv-slot-01\child.vhdx'
$principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
$hyperVAdministrators = [Security.Principal.SecurityIdentifier]::new('S-1-5-32-578')
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator) -and
    -not $principal.IsInRole($hyperVAdministrators)) { throw 'Hyper-V management token required' }
$vm = Get-VM -Id $vmId
if ($vm.Name -ne $vmName -or $vm.State -ne 'Off' -or $vm.Generation -ne 2 -or [string]$vm.Version -ne '12.0') { throw 'enrolled VM identity or state mismatch' }
if ($vm.AutomaticCheckpointsEnabled -or @(Get-VMSnapshot -VM $vm -ErrorAction Stop).Count -ne 0) { throw 'checkpoint state rejected' }
if (@(Get-VMGpuPartitionAdapter -VM $vm -ErrorAction Stop).Count -ne 0) { throw 'GPU adapter must be removed before reset' }
$parent = Get-Item -LiteralPath $parentPath
if (-not $parent.IsReadOnly -or ($parent.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'parent protection mismatch' }
$hash = (Get-FileHash -LiteralPath $parentPath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($hash -ne $parentHash) { throw 'parent hash mismatch' }
$parentVhd = Get-VHD -Path $parentPath
if ($parentVhd.ParentPath -or $parentVhd.VhdType -ne 'Dynamic' -or $parentVhd.Attached) { throw 'parent VHD state mismatch' }
$drives = @(Get-VMHardDiskDrive -VM $vm)
if ($drives.Count -gt 1 -or ($drives.Count -eq 1 -and $drives[0].Path -ine $childPath)) { throw 'enrolled child attachment mismatch' }
if (Test-Path -LiteralPath $childPath) {
    $childItem = Get-Item -LiteralPath $childPath
    if ($childItem.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'child reparse point rejected' }
    $child = Get-VHD -Path $childPath
    if ($child.ParentPath -ine $parentPath -or $child.VhdType -ne 'Differencing') { throw 'child chain mismatch' }
}
if ($drives.Count -eq 1) { Remove-VMHardDiskDrive -VMHardDiskDrive $drives[0] }
if (Test-Path -LiteralPath $childPath) { Remove-Item -LiteralPath $childPath }
New-VHD -Path $childPath -ParentPath $parentPath -Differencing | Out-Null
Add-VMHardDiskDrive -VM $vm -ControllerType SCSI -ControllerNumber 0 -ControllerLocation 0 -Path $childPath
$verifiedDrive = Get-VMHardDiskDrive -VM $vm
$verifiedChild = Get-VHD -Path $verifiedDrive.Path
if ($verifiedDrive.Path -ine $childPath -or $verifiedChild.ParentPath -ine $parentPath -or -not (Get-Item -LiteralPath $parentPath).IsReadOnly) { throw 'reset verification failed' }
[Console]::Out.WriteLine('status' + "`t" + 'ok')
[Console]::Out.WriteLine('vm_id' + "`t" + $vmId.ToString().ToLowerInvariant())
[Console]::Out.WriteLine('child' + "`t" + $verifiedChild.Path)
[Console]::Out.WriteLine('parent' + "`t" + $verifiedChild.ParentPath)
[Console]::Out.WriteLine('parent_sha256' + "`t" + $hash)
"#;

const INSPECT_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$vmId = [guid]'2627E735-5B33-4104-B739-622727DD3A40'
$vmName = 'HyperGpuSupport-Disposable-01'
$parentPath = 'Z:\HyperGpuSupport\images\golden\win11-pro-25h2-26200.9457-x64-v1\parent.vhdx'
$parentHash = '0fb4dfe6dd51eed64d36e482e4f58d67c19922802e34aa6ae2d1f4ccd5daeb07'
$childPath = 'Z:\HyperGpuSupport\images\disposable\gpu-pv-slot-01\child.vhdx'
$gpuPath = '\\?\PCI#VEN_10DE&DEV_2D05&SUBSYS_8A151043&REV_A1#95B0EB63032DB04800#{064092b3-625e-43bf-9eb5-dc845897dd59}\GPUPARAV'
$principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
$hyperVAdministrators = [Security.Principal.SecurityIdentifier]::new('S-1-5-32-578')
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator) -and
    -not $principal.IsInRole($hyperVAdministrators)) { throw 'Hyper-V management token required' }
$vm = Get-VM -Id $vmId
if ($vm.Name -ne $vmName -or $vm.Generation -ne 2 -or [string]$vm.Version -ne '12.0') { throw 'enrolled VM identity mismatch' }
if ($vm.State -notin @('Off','Running') -or $vm.AutomaticCheckpointsEnabled -or @(Get-VMSnapshot -VM $vm -ErrorAction Stop).Count -ne 0) { throw 'enrolled VM state rejected' }
$drives = @(Get-VMHardDiskDrive -VM $vm)
if ($drives.Count -ne 1 -or $drives[0].Path -ine $childPath) { throw 'enrolled child attachment mismatch' }
$child = Get-VHD -Path $childPath
if ($child.ParentPath -ine $parentPath -or $child.VhdType -ne 'Differencing') { throw 'child chain mismatch' }
$parent = Get-Item -LiteralPath $parentPath
if (-not $parent.IsReadOnly -or ($parent.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'parent protection mismatch' }
$hash = (Get-FileHash -LiteralPath $parentPath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($hash -ne $parentHash) { throw 'parent hash mismatch' }
$hostGpu = Get-VMHostPartitionableGpu -Name $gpuPath
if ($hostGpu.Name -ine $gpuPath) { throw 'partitionable GPU identity mismatch' }
$gpuAdapters = @(Get-VMGpuPartitionAdapter -VM $vm -ErrorAction Stop)
[Console]::Out.WriteLine('status' + "`t" + 'ok')
[Console]::Out.WriteLine('vm_id' + "`t" + $vmId.ToString().ToLowerInvariant())
[Console]::Out.WriteLine('state' + "`t" + [string]$vm.State)
[Console]::Out.WriteLine('gpu_adapters' + "`t" + $gpuAdapters.Count)
[Console]::Out.WriteLine('child' + "`t" + $child.Path)
[Console]::Out.WriteLine('parent' + "`t" + $child.ParentPath)
[Console]::Out.WriteLine('parent_sha256' + "`t" + $hash)
[Console]::Out.WriteLine('gpu_interface' + "`t" + $hostGpu.Name)
"#;

const START_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$vmId = [guid]'2627E735-5B33-4104-B739-622727DD3A40'
$vmName = 'HyperGpuSupport-Disposable-01'
$parentPath = 'Z:\HyperGpuSupport\images\golden\win11-pro-25h2-26200.9457-x64-v1\parent.vhdx'
$parentHash = '0fb4dfe6dd51eed64d36e482e4f58d67c19922802e34aa6ae2d1f4ccd5daeb07'
$childPath = 'Z:\HyperGpuSupport\images\disposable\gpu-pv-slot-01\child.vhdx'
$gpuPath = '\\?\PCI#VEN_10DE&DEV_2D05&SUBSYS_8A151043&REV_A1#95B0EB63032DB04800#{064092b3-625e-43bf-9eb5-dc845897dd59}\GPUPARAV'
$vm = Get-VM -Id $vmId -ErrorAction Stop
if ($vm.Name -ne $vmName -or $vm.State -ne 'Off' -or $vm.Generation -ne 2 -or [string]$vm.Version -ne '12.0') { throw 'enrolled VM identity or start state mismatch' }
if ($vm.AutomaticCheckpointsEnabled -or @(Get-VMSnapshot -VM $vm -ErrorAction Stop).Count -ne 0) { throw 'checkpoint state rejected' }
$drives = @(Get-VMHardDiskDrive -VM $vm -ErrorAction Stop)
if ($drives.Count -ne 1 -or $drives[0].Path -ine $childPath) { throw 'enrolled child attachment mismatch' }
$childItem = Get-Item -LiteralPath $childPath -ErrorAction Stop
if ($childItem.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'child reparse point rejected' }
$child = Get-VHD -Path $childPath -ErrorAction Stop
if ($child.ParentPath -ine $parentPath -or $child.VhdType -ne 'Differencing') { throw 'child chain mismatch' }
$parent = Get-Item -LiteralPath $parentPath -ErrorAction Stop
if (-not $parent.IsReadOnly -or ($parent.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'parent protection mismatch' }
$hash = $parentHash
$gpuAdapters = @(Get-VMGpuPartitionAdapter -VM $vm -ErrorAction Stop)
if ($gpuAdapters.Count -gt 1 -or ($gpuAdapters.Count -eq 1 -and $gpuAdapters[0].InstancePath -ine $gpuPath)) { throw 'GPU adapter identity rejected' }
$otherGpuAssignments = @(Get-VM -ErrorAction Stop | Where-Object { $_.Id -ne $vmId } | ForEach-Object { Get-VMGpuPartitionAdapter -VM $_ -ErrorAction Stop })
if ($otherGpuAssignments.Count -ne 0) { throw 'another VM GPU assignment rejected' }
$availableMemoryKiB = [uint64](Get-CimInstance -ClassName Win32_OperatingSystem -ErrorAction Stop).FreePhysicalMemory
if ($availableMemoryKiB -lt 12582912) { throw 'host available memory below 12 GiB' }
Start-VM -VM $vm -ErrorAction Stop | Out-Null
$verifiedVm = Get-VM -Id $vmId -ErrorAction Stop
$verifiedDrives = @(Get-VMHardDiskDrive -VM $verifiedVm -ErrorAction Stop)
$verifiedAdapters = @(Get-VMGpuPartitionAdapter -VM $verifiedVm -ErrorAction Stop)
if ($verifiedVm.Name -ne $vmName -or $verifiedVm.State -ne 'Running' -or $verifiedDrives.Count -ne 1 -or $verifiedDrives[0].Path -ine $childPath -or $verifiedAdapters.Count -ne $gpuAdapters.Count -or ($verifiedAdapters.Count -eq 1 -and $verifiedAdapters[0].InstancePath -ine $gpuPath)) { throw 'start verification failed' }
[Console]::Out.WriteLine('status' + "`t" + 'ok')
[Console]::Out.WriteLine('vm_id' + "`t" + $vmId.ToString().ToLowerInvariant())
[Console]::Out.WriteLine('previous_state' + "`t" + 'Off')
[Console]::Out.WriteLine('state' + "`t" + [string]$verifiedVm.State)
[Console]::Out.WriteLine('gpu_adapters' + "`t" + $verifiedAdapters.Count)
[Console]::Out.WriteLine('child' + "`t" + $child.Path)
[Console]::Out.WriteLine('parent' + "`t" + $child.ParentPath)
[Console]::Out.WriteLine('parent_sha256' + "`t" + $hash)
"#;

const SHUTDOWN_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$vmId = [guid]'2627E735-5B33-4104-B739-622727DD3A40'
$vmName = 'HyperGpuSupport-Disposable-01'
$parentPath = 'Z:\HyperGpuSupport\images\golden\win11-pro-25h2-26200.9457-x64-v1\parent.vhdx'
$parentHash = '0fb4dfe6dd51eed64d36e482e4f58d67c19922802e34aa6ae2d1f4ccd5daeb07'
$childPath = 'Z:\HyperGpuSupport\images\disposable\gpu-pv-slot-01\child.vhdx'
$gpuPath = '\\?\PCI#VEN_10DE&DEV_2D05&SUBSYS_8A151043&REV_A1#95B0EB63032DB04800#{064092b3-625e-43bf-9eb5-dc845897dd59}\GPUPARAV'
$vm = Get-VM -Id $vmId -ErrorAction Stop
if ($vm.Name -ne $vmName -or $vm.State -ne 'Running' -or $vm.Generation -ne 2 -or [string]$vm.Version -ne '12.0') { throw 'enrolled VM identity or shutdown state mismatch' }
if ($vm.AutomaticCheckpointsEnabled -or @(Get-VMSnapshot -VM $vm -ErrorAction Stop).Count -ne 0) { throw 'checkpoint state rejected' }
$drives = @(Get-VMHardDiskDrive -VM $vm -ErrorAction Stop)
if ($drives.Count -ne 1 -or $drives[0].Path -ine $childPath) { throw 'enrolled child attachment mismatch' }
$childItem = Get-Item -LiteralPath $childPath -ErrorAction Stop
if ($childItem.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'child reparse point rejected' }
$child = Get-VHD -Path $childPath -ErrorAction Stop
if ($child.ParentPath -ine $parentPath -or $child.VhdType -ne 'Differencing') { throw 'child chain mismatch' }
$parent = Get-Item -LiteralPath $parentPath -ErrorAction Stop
if (-not $parent.IsReadOnly -or ($parent.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'parent protection mismatch' }
$hash = $parentHash
$gpuAdapters = @(Get-VMGpuPartitionAdapter -VM $vm -ErrorAction Stop)
if ($gpuAdapters.Count -gt 1 -or ($gpuAdapters.Count -eq 1 -and $gpuAdapters[0].InstancePath -ine $gpuPath)) { throw 'GPU adapter identity rejected' }
Stop-VM -VM $vm -Confirm:$false -ErrorAction Stop
$verifiedVm = Get-VM -Id $vmId -ErrorAction Stop
$verifiedDrives = @(Get-VMHardDiskDrive -VM $verifiedVm -ErrorAction Stop)
$verifiedAdapters = @(Get-VMGpuPartitionAdapter -VM $verifiedVm -ErrorAction Stop)
if ($verifiedVm.Name -ne $vmName -or $verifiedVm.State -ne 'Off' -or $verifiedDrives.Count -ne 1 -or $verifiedDrives[0].Path -ine $childPath -or $verifiedAdapters.Count -ne $gpuAdapters.Count -or ($verifiedAdapters.Count -eq 1 -and $verifiedAdapters[0].InstancePath -ine $gpuPath)) { throw 'shutdown verification failed' }
[Console]::Out.WriteLine('status' + "`t" + 'ok')
[Console]::Out.WriteLine('vm_id' + "`t" + $vmId.ToString().ToLowerInvariant())
[Console]::Out.WriteLine('previous_state' + "`t" + 'Running')
[Console]::Out.WriteLine('state' + "`t" + [string]$verifiedVm.State)
[Console]::Out.WriteLine('gpu_adapters' + "`t" + $verifiedAdapters.Count)
[Console]::Out.WriteLine('child' + "`t" + $child.Path)
[Console]::Out.WriteLine('parent' + "`t" + $child.ParentPath)
[Console]::Out.WriteLine('parent_sha256' + "`t" + $hash)
"#;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{Duration, Instant};

    use super::{
        INSPECT_SCRIPT, INSPECT_TIMEOUT, LockFile, RECONCILIATION_MARKER, RESET_SCRIPT,
        RESET_TIMEOUT, SHUTDOWN_SCRIPT, SHUTDOWN_TIMEOUT, START_SCRIPT, START_TIMEOUT,
        append_audit, begin_reconciliation, ensure_reconciled, handle_request, json_escape,
        run_bounded, startup_failure_message,
    };
    use hyper_gpu_support::runner::{Operation, POLICY_V1};

    #[test]
    fn fixed_script_has_no_external_input_or_broad_vm_deletion() {
        assert!(POLICY_V1.contains("reset-slot"));
        assert!(!RESET_SCRIPT.contains("Remove-VM "));
        assert!(!RESET_SCRIPT.contains("Invoke-Expression"));
        assert!(!RESET_SCRIPT.contains("param("));
        assert!(RESET_SCRIPT.contains("Remove-VMHardDiskDrive"));
        assert!(RESET_SCRIPT.contains("Get-FileHash"));
        assert!(RESET_SCRIPT.contains("S-1-5-32-578"));
        assert!(!RESET_SCRIPT.contains("SilentlyContinue"));
        assert!(!INSPECT_SCRIPT.contains("Add-VM"));
        assert!(!INSPECT_SCRIPT.contains("Set-VM"));
        assert!(!INSPECT_SCRIPT.contains("Remove-VM"));
        assert!(!INSPECT_SCRIPT.contains("Start-VM"));
        assert!(!INSPECT_SCRIPT.contains("Stop-VM"));
        assert!(!INSPECT_SCRIPT.contains("SilentlyContinue"));
        assert!(INSPECT_SCRIPT.contains("Get-FileHash"));
        assert!(START_SCRIPT.contains("Start-VM -VM $vm"));
        assert!(START_SCRIPT.contains("FreePhysicalMemory"));
        assert!(START_SCRIPT.contains("-lt 12582912"));
        assert!(START_SCRIPT.contains("another VM GPU assignment rejected"));
        assert!(SHUTDOWN_SCRIPT.contains("Stop-VM -VM $vm"));
        for script in [START_SCRIPT, SHUTDOWN_SCRIPT] {
            assert!(script.contains("2627E735-5B33-4104-B739-622727DD3A40"));
            assert!(script.contains("Get-VMSnapshot"));
            assert!(script.contains("Get-VMGpuPartitionAdapter"));
            assert!(script.contains("$gpuAdapters[0].InstancePath -ine $gpuPath"));
            assert!(script.contains("$verifiedAdapters[0].InstancePath -ine $gpuPath"));
            assert!(!script.contains("Remove-VM"));
            assert!(!script.contains("Invoke-Expression"));
            assert!(!script.contains("param("));
            assert!(!script.contains("SilentlyContinue"));
            assert!(!script.contains("Get-FileHash"));
        }
        assert!(!SHUTDOWN_SCRIPT.contains("-Force"));
        assert!(!SHUTDOWN_SCRIPT.contains("-TurnOff"));
        assert!(!SHUTDOWN_SCRIPT.contains("-Save"));
    }

    #[test]
    fn lifecycle_deadlines_include_parent_inspection_and_transition_allowance() {
        assert_eq!(RESET_TIMEOUT, Duration::from_secs(300));
        assert_eq!(INSPECT_TIMEOUT, Duration::from_secs(300));
        assert_eq!(START_TIMEOUT, Duration::from_secs(180));
        assert_eq!(SHUTDOWN_TIMEOUT, Duration::from_secs(120));
        let installer = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scripts/setup/install-runner-v1.ps1"
        ));
        assert!(installer.contains("ExecutionTimeLimit = 'PT10M'"));
        assert!(INSPECT_TIMEOUT + START_TIMEOUT < Duration::from_secs(600));
        assert!(INSPECT_TIMEOUT + SHUTDOWN_TIMEOUT < Duration::from_secs(600));
    }

    #[test]
    fn audit_json_escapes_control_characters() {
        assert_eq!(
            json_escape("one\\two\r\n\"three"),
            "one\\\\two\\r\\n\\\"three"
        );
    }

    #[test]
    fn startup_failure_record_is_single_line_and_bounded() {
        let diagnostic = format!("first\r\n{}", "é".repeat(300));
        let message = startup_failure_message(&diagnostic);
        assert!(message.ends_with('\n'));
        assert_eq!(message.matches('\n').count(), 1);
        assert!(!message.contains('\r'));
        assert!(message.len() <= 513);
    }

    #[test]
    fn operation_audit_records_the_actual_fixed_operation() {
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("local")
            .join("test-work")
            .join(format!(
                "hyper-gpu-runner-audit-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        fs::create_dir_all(&directory).unwrap();
        append_audit(
            &directory,
            "1234567890-123456789",
            "0123456789abcdef0123456789abcdef",
            Operation::Inspect,
            "succeeded",
            "",
        )
        .unwrap();
        let audit = fs::read_to_string(directory.join("events.jsonl")).unwrap();
        assert!(audit.contains("\"request_id\":\"0123456789abcdef0123456789abcdef\""));
        assert!(audit.contains("\"operation\":\"inspect\""));
        assert!(!audit.contains("\"operation\":\"reset-slot\""));
        fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn operation_lock_recovers_after_owner_exit_without_deleting_marker() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("local")
            .join("test-work")
            .join(format!(
                "hyper-gpu-runner-lock-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let first = LockFile::acquire(&path).unwrap();
        assert!(LockFile::acquire(&path).is_err());
        drop(first);
        let second = LockFile::acquire(&path).unwrap();
        drop(second);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn uncertain_lifecycle_marker_blocks_mutation_but_allows_inspection() {
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("local")
            .join("test-work")
            .join(format!(
                "hyper-gpu-runner-reconcile-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        fs::create_dir_all(&directory).unwrap();
        let marker = directory.join(RECONCILIATION_MARKER);
        begin_reconciliation(&marker, Operation::StartSlot, "1790391920-577596700").unwrap();
        assert!(
            begin_reconciliation(&marker, Operation::ShutdownSlot, "1790391921-577596700").is_err()
        );
        assert_eq!(
            ensure_reconciled(&directory, Operation::ResetSlot),
            Err("reconciliation-required")
        );
        assert_eq!(ensure_reconciled(&directory, Operation::Inspect), Ok(()));
        assert_eq!(
            fs::read_to_string(&marker).unwrap(),
            "start-slot 1790391920-577596700\n"
        );
        fs::remove_dir_all(&directory).unwrap();

        assert_eq!(
            ensure_reconciled(std::path::Path::new("invalid\0state"), Operation::ResetSlot),
            Err("reconciliation-state-unavailable")
        );
    }

    #[test]
    fn malformed_pipe_request_returns_bounded_failure() {
        let response = handle_request(b"not-a-request");
        assert_eq!(response.status, "failed");
        assert_eq!(response.diagnostic, "invalid-request");
        assert!(response.operation_id.is_none());
    }

    #[test]
    fn fixed_adapter_timeout_and_output_limit_cancel_process() {
        let started = Instant::now();
        assert_eq!(
            run_bounded("Start-Sleep -Seconds 5", Duration::from_millis(100)).unwrap_err(),
            "fixed Hyper-V adapter timed out"
        );
        assert!(started.elapsed() < Duration::from_secs(3));

        let started = Instant::now();
        assert_eq!(
            run_bounded(
                "[Console]::Out.Write('A' * 70000); Start-Sleep -Seconds 5",
                Duration::from_secs(5)
            )
            .unwrap_err(),
            "fixed adapter output exceeded limit"
        );
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn fixed_lifecycle_scripts_execute_against_typed_fakes() {
        const FAKES: &str = r#"
$script:state = 'Off'
$script:adapterPath = '\\?\PCI#VEN_10DE&DEV_2D05&SUBSYS_8A151043&REV_A1#95B0EB63032DB04800#{064092b3-625e-43bf-9eb5-dc845897dd59}\GPUPARAV'
$script:otherAssignment = $false
$script:denyGpuQuery = $false
$script:freeMemory = [uint64]12582912
function Get-VM { [CmdletBinding()] param([guid]$Id) $target = [pscustomobject]@{ Id = [guid]'2627E735-5B33-4104-B739-622727DD3A40'; Name = 'HyperGpuSupport-Disposable-01'; State = $script:state; Generation = 2; Version = '12.0'; AutomaticCheckpointsEnabled = $false }; if ($PSBoundParameters.ContainsKey('Id')) { $target } else { $target; if ($script:otherAssignment) { [pscustomobject]@{ Id = [guid]'00000000-0000-0000-0000-000000000001'; Name = 'Other' } } } }
function Get-VMSnapshot { [CmdletBinding()] param($VM) }
function Get-VMHardDiskDrive { [CmdletBinding()] param($VM) [pscustomobject]@{ Path = 'Z:\HyperGpuSupport\images\disposable\gpu-pv-slot-01\child.vhdx' } }
function Get-Item { [CmdletBinding()] param([string]$LiteralPath) [pscustomobject]@{ IsReadOnly = $true; Attributes = [IO.FileAttributes]::Normal } }
function Get-VHD { [CmdletBinding()] param([string]$Path) [pscustomobject]@{ Path = 'Z:\HyperGpuSupport\images\disposable\gpu-pv-slot-01\child.vhdx'; ParentPath = 'Z:\HyperGpuSupport\images\golden\win11-pro-25h2-26200.9457-x64-v1\parent.vhdx'; VhdType = 'Differencing' } }
function Get-FileHash { [CmdletBinding()] param([string]$LiteralPath, [string]$Algorithm) [pscustomobject]@{ Hash = '0fb4dfe6dd51eed64d36e482e4f58d67c19922802e34aa6ae2d1f4ccd5daeb07' } }
function Get-VMGpuPartitionAdapter { [CmdletBinding()] param($VM) if ($script:denyGpuQuery) { throw 'gpu query denied' }; if ($VM.Id -eq [guid]'2627E735-5B33-4104-B739-622727DD3A40' -or $script:otherAssignment) { [pscustomobject]@{ InstancePath = $script:adapterPath } } }
function Get-CimInstance { [CmdletBinding()] param([string]$ClassName) [pscustomobject]@{ FreePhysicalMemory = $script:freeMemory } }
function Start-VM { [CmdletBinding()] param($VM) $script:state = 'Running' }
function Stop-VM { [CmdletBinding(SupportsShouldProcess)] param($VM) $script:state = 'Off' }
"#;
        let start_output =
            run_bounded(&format!("{FAKES}\n{START_SCRIPT}"), Duration::from_secs(5)).unwrap();
        hyper_gpu_support::runner::parse_lifecycle_result(&start_output, Operation::StartSlot)
            .unwrap();

        let shutdown_output = run_bounded(
            &format!("{FAKES}\n$script:state = 'Running'\n{SHUTDOWN_SCRIPT}"),
            Duration::from_secs(5),
        )
        .unwrap();
        hyper_gpu_support::runner::parse_lifecycle_result(
            &shutdown_output,
            Operation::ShutdownSlot,
        )
        .unwrap();

        assert!(
            run_bounded(
                &format!("{FAKES}\n$script:adapterPath = 'wrong'\n{START_SCRIPT}"),
                Duration::from_secs(5)
            )
            .unwrap_err()
            .contains("GPU adapter identity rejected")
        );
        assert!(
            run_bounded(
                &format!("{FAKES}\n$script:otherAssignment = $true\n{START_SCRIPT}"),
                Duration::from_secs(5)
            )
            .unwrap_err()
            .contains("another VM GPU assignment rejected")
        );
        assert!(
            run_bounded(
                &format!("{FAKES}\n$script:freeMemory = 12582911\n{START_SCRIPT}"),
                Duration::from_secs(5)
            )
            .unwrap_err()
            .contains("host available memory below 12 GiB")
        );
        assert!(
            run_bounded(
                &format!("{FAKES}\n$script:denyGpuQuery = $true\n{START_SCRIPT}"),
                Duration::from_secs(5)
            )
            .unwrap_err()
            .contains("gpu query denied")
        );
    }
}
