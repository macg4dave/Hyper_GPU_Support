//! Administrator-installed fixed runner for the enrolled disposable Hyper-V slot.

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use hyper_gpu_support::config::{ConfigError, ProjectConfiguration};
use hyper_gpu_support::runner::{
    Enrollment, Operation, PIPE_NAME, Request, Response, authorize_request, consume_nonce,
    embedded_policy, parse_enrollment, parse_gpu_assignment_result, parse_inspect_result,
    parse_lifecycle_result, parse_request, parse_reset_result, policy_allows, policy_fingerprint,
    verify_policy,
};
use hyper_gpu_support::windows_runner::serve_one;

#[path = "hyper_gpu_runner/supervision.rs"]
mod supervision;

const RECONCILIATION_MARKER: &str = "reconciliation-required-v1";
const OUTPUT_LIMIT: usize = 64 * 1024;

fn main() -> ExitCode {
    let started = Instant::now();
    let startup_failure_path =
        data_directory().map(|path| path.join("audit").join("runner-startup-failure-v1.txt"));
    if let Ok(path) = &startup_failure_path {
        let _ = fs::remove_file(path);
    }
    let mut arguments = std::env::args_os().skip(1);
    let mode = arguments.next();
    if arguments.next().is_some() {
        eprintln!("runner error: exactly one fixed mode is required");
        return ExitCode::FAILURE;
    }
    let outcome = match mode.as_deref() {
        Some(value) if value == "serve-once" => run_server(started),
        _ => Err("only fixed serve-once mode is accepted".into()),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if let Ok(path) = startup_failure_path {
                let _ = fs::write(path, startup_failure_message(error.as_ref()));
            }
            eprintln!("runner error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn project_configuration() -> Result<ProjectConfiguration, ConfigError> {
    ProjectConfiguration::embedded()
}

fn data_directory() -> Result<PathBuf, ConfigError> {
    Ok(project_configuration()?.runner.data_directory)
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
    let data_directory = data_directory()?;
    let installed_policy = fs::read_to_string(data_directory.join("policy-v1.json"))?;
    verify_policy(&installed_policy)?;
    let enrollment = parse_enrollment(&fs::read_to_string(
        data_directory.join("enrollment-v1.json"),
    )?)?;

    let state_directory = data_directory.join("state");
    let result_directory = data_directory.join("results");
    let audit_directory = data_directory.join("audit");
    fs::create_dir_all(&state_directory)?;
    fs::create_dir_all(&result_directory)?;
    fs::create_dir_all(&audit_directory)?;
    fs::create_dir_all(state_directory.join("nonces"))?;
    Ok(enrollment)
}

fn run_server(started: Instant) -> Result<(), Box<dyn std::error::Error>> {
    let enrollment = prepare_install_state()?;
    serve_one(
        PIPE_NAME,
        enrollment.client_sid(),
        &enrollment.pipe_sddl(),
        |input| {
            let response = handle_request(input, started);
            Ok(response.encode().into_bytes())
        },
    )?;
    Ok(())
}

fn handle_request(input: &[u8], started: Instant) -> Response {
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
    match execute_request(&request, started) {
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
    let policy = embedded_policy().map_err(std::io::Error::other)?;
    let path = data_directory()
        .map_err(std::io::Error::other)?
        .join("audit")
        .join("events.jsonl");
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let request_id = request.map_or_else(|| "0".repeat(32), |value| value.request_id.clone());
    let operation = request.map_or("none", |value| value.operation.as_str());
    let nonce = request.map_or_else(|| "0".repeat(32), |value| value.nonce.clone());
    let plan = request.map_or_else(|| "0".repeat(64), |value| value.plan_fingerprint.clone());
    writeln!(
        file,
        concat!(
            "{{\"schema\":1,\"request_id\":\"{}\",\"operation\":\"{}\",",
            "\"slot\":\"{}\",\"vm_id\":\"{}\",",
            "\"nonce\":\"{}\",\"plan_fingerprint\":\"{}\",",
            "\"status\":\"{}\",\"diagnostic\":\"{}\"}}"
        ),
        request_id,
        operation,
        json_escape(policy.slot()),
        json_escape(policy.vm_id()),
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

fn execute_request(request: &Request, started: Instant) -> Result<String, ExecutionFailure> {
    let before_effect = |diagnostic| ExecutionFailure {
        diagnostic,
        operation_id: None,
    };
    if !policy_allows(request.operation) {
        return Err(before_effect("operation-denied"));
    }
    let project = project_configuration().map_err(|_| before_effect("configuration-invalid"))?;
    ensure_reconciled(
        &project.runner.data_directory.join("state"),
        request.operation,
    )
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
        &project.runner.data_directory.join("state").join("nonces"),
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
        Operation::Inspect => {
            run_inspect(request, started).map_err(|_| before_effect("operation-failed"))
        }
        Operation::ResetSlot => execute_reset_request(request, started),
        Operation::StartSlot => execute_lifecycle_request(
            request,
            Operation::StartSlot,
            START_SCRIPT,
            project.runner.start_timeout,
            started,
        ),
        Operation::ShutdownSlot => execute_lifecycle_request(
            request,
            Operation::ShutdownSlot,
            SHUTDOWN_SCRIPT,
            project.runner.shutdown_timeout,
            started,
        ),
        Operation::AssignGpu => execute_gpu_assignment_request(
            request,
            Operation::AssignGpu,
            ASSIGN_GPU_SCRIPT,
            started,
        ),
        Operation::RemoveGpu => execute_gpu_assignment_request(
            request,
            Operation::RemoveGpu,
            REMOVE_GPU_SCRIPT,
            started,
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
    started: Instant,
) -> Result<String, ExecutionFailure> {
    let operation_id = operation_id().map_err(|_| ExecutionFailure {
        diagnostic: "operation-id-failed",
        operation_id: None,
    })?;
    match run_lifecycle(request, operation, &operation_id, script, timeout, started) {
        Ok(()) => Ok(operation_id),
        Err(_) => Err(ExecutionFailure {
            diagnostic: "operation-failed-reconciliation-required",
            operation_id: Some(operation_id),
        }),
    }
}

fn execute_reset_request(request: &Request, started: Instant) -> Result<String, ExecutionFailure> {
    let operation_id = operation_id().map_err(|_| ExecutionFailure {
        diagnostic: "operation-id-failed",
        operation_id: None,
    })?;
    match run_reset(request, &operation_id, started) {
        Ok(()) => Ok(operation_id),
        Err(_) => Err(ExecutionFailure {
            diagnostic: "operation-failed-reconciliation-required",
            operation_id: Some(operation_id),
        }),
    }
}

fn execute_gpu_assignment_request(
    request: &Request,
    operation: Operation,
    script: &str,
    started: Instant,
) -> Result<String, ExecutionFailure> {
    let operation_id = operation_id().map_err(|_| ExecutionFailure {
        diagnostic: "operation-id-failed",
        operation_id: None,
    })?;
    match run_gpu_assignment(request, operation, &operation_id, script, started) {
        Ok(()) => Ok(operation_id),
        Err(_) => Err(ExecutionFailure {
            diagnostic: "operation-failed-reconciliation-required",
            operation_id: Some(operation_id),
        }),
    }
}

fn run_reset(
    request: &Request,
    operation_id: &str,
    started: Instant,
) -> Result<(), Box<dyn std::error::Error>> {
    let _enrollment = prepare_install_state()?;
    let project = project_configuration()?;
    let state_directory = project.runner.data_directory.join("state");
    let result_directory = project.runner.data_directory.join("results");
    let audit_directory = project.runner.data_directory.join("audit");
    let lock_path = state_directory.join("operation.lock");
    let _lock = LockFile::acquire(&lock_path)?;
    ensure_reconciled(&state_directory, Operation::ResetSlot).map_err(std::io::Error::other)?;
    run_reconciled_operation(
        &state_directory,
        &result_directory,
        &audit_directory,
        request,
        Operation::ResetSlot,
        operation_id,
        || {
            let script = fixed_script(RESET_SCRIPT)?;
            let output = run_supervised(
                &script,
                project.runner.reset_timeout,
                adapter_budget(
                    project.runner.task_execution_timeout,
                    Duration::ZERO,
                    started.elapsed(),
                ),
                &project.guest.powershell_path,
                "reset-slot adapter",
            )
            .map_err(std::io::Error::other)?;
            let result = parse_reset_result(&output)?;
            Ok(format!(
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
            ))
        },
    )
}

fn run_reconciled_operation(
    state_directory: &Path,
    result_directory: &Path,
    audit_directory: &Path,
    request: &Request,
    operation: Operation,
    operation_id: &str,
    action: impl FnOnce() -> Result<String, Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    append_audit(
        audit_directory,
        operation_id,
        &request.request_id,
        operation,
        "started",
        "",
    )?;
    let reconciliation_path = state_directory.join(RECONCILIATION_MARKER);
    begin_reconciliation(&reconciliation_path, operation, operation_id)?;

    let operation_result = action().and_then(|result_json| {
        write_atomic(result_directory, operation_id, &result_json)
            .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
    });
    if let Err(error) = operation_result {
        let _ = append_audit(
            audit_directory,
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
        let _ = write_atomic(result_directory, operation_id, &failure_json);
        return Err(error);
    }
    append_audit(
        audit_directory,
        operation_id,
        &request.request_id,
        operation,
        "succeeded",
        "",
    )?;
    fs::remove_file(reconciliation_path)?;
    Ok(())
}

fn run_inspect(request: &Request, started: Instant) -> Result<String, Box<dyn std::error::Error>> {
    let _enrollment = prepare_install_state()?;
    let project = project_configuration()?;
    let state_directory = project.runner.data_directory.join("state");
    let result_directory = project.runner.data_directory.join("results");
    let audit_directory = project.runner.data_directory.join("audit");
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
    let script = fixed_script(INSPECT_SCRIPT)?;
    let output = match run_supervised(
        &script,
        project.runner.inspect_timeout,
        adapter_budget(
            project.runner.task_execution_timeout,
            Duration::ZERO,
            started.elapsed(),
        ),
        &project.guest.powershell_path,
        "inspect adapter",
    ) {
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
    started: Instant,
) -> Result<(), Box<dyn std::error::Error>> {
    let _enrollment = prepare_install_state()?;
    let project = project_configuration()?;
    let state_directory = project.runner.data_directory.join("state");
    let result_directory = project.runner.data_directory.join("results");
    let audit_directory = project.runner.data_directory.join("audit");
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
        let inspection_script = fixed_script(INSPECT_SCRIPT)?;
        let inspection_output = run_supervised(
            &inspection_script,
            project.runner.inspect_timeout,
            adapter_budget(
                project.runner.task_execution_timeout,
                timeout,
                started.elapsed(),
            ),
            &project.guest.powershell_path,
            "lifecycle preflight adapter",
        )
        .map_err(std::io::Error::other)?;
        let inspection = parse_inspect_result(&inspection_output)?;
        let expected_state = match operation {
            Operation::StartSlot => "Off",
            Operation::ShutdownSlot => "Running",
            _ => return Err("invalid lifecycle operation".into()),
        };
        if inspection.state != expected_state {
            return Err("lifecycle preflight state mismatch".into());
        }
        let operation_script = fixed_script(script)?;
        require_transition_budget(
            project.runner.task_execution_timeout,
            timeout,
            started.elapsed(),
        )?;
        let output =
            run_bounded_with_powershell(&operation_script, timeout, &project.guest.powershell_path)
                .map_err(std::io::Error::other)?;
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

fn run_gpu_assignment(
    request: &Request,
    operation: Operation,
    operation_id: &str,
    script: &str,
    started: Instant,
) -> Result<(), Box<dyn std::error::Error>> {
    let _enrollment = prepare_install_state()?;
    let project = project_configuration()?;
    let state_directory = project.runner.data_directory.join("state");
    let result_directory = project.runner.data_directory.join("results");
    let audit_directory = project.runner.data_directory.join("audit");
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
        let inspection_script = fixed_script(INSPECT_SCRIPT)?;
        let inspection_output = run_supervised(
            &inspection_script,
            project.runner.inspect_timeout,
            adapter_budget(
                project.runner.task_execution_timeout,
                project.runner.gpu_assignment_timeout,
                started.elapsed(),
            ),
            &project.guest.powershell_path,
            "GPU assignment preflight adapter",
        )
        .map_err(std::io::Error::other)?;
        let inspection = parse_inspect_result(&inspection_output)?;
        let expected_count = match operation {
            Operation::AssignGpu => 0,
            Operation::RemoveGpu => 1,
            _ => return Err("invalid GPU assignment operation".into()),
        };
        if inspection.state != "Off" || inspection.gpu_adapters != expected_count {
            return Err("GPU assignment preflight state mismatch".into());
        }
        let operation_script = fixed_script(script)?;
        require_transition_budget(
            project.runner.task_execution_timeout,
            project.runner.gpu_assignment_timeout,
            started.elapsed(),
        )?;
        let output = run_bounded_with_powershell(
            &operation_script,
            project.runner.gpu_assignment_timeout,
            &project.guest.powershell_path,
        )
        .map_err(std::io::Error::other)?;
        let result = parse_gpu_assignment_result(&output, operation)?;
        let result_json = format!(
            concat!(
                "{{\n  \"schema\": 1,\n  \"request_id\": \"{}\",\n",
                "  \"operation_id\": \"{}\",\n  \"operation\": \"{}\",\n",
                "  \"status\": \"succeeded\",\n  \"vm_id\": \"{}\",\n",
                "  \"state\": \"{}\",\n  \"previous_gpu_adapters\": {},\n",
                "  \"gpu_adapters\": {},\n  \"child\": \"{}\",\n",
                "  \"parent\": \"{}\",\n  \"parent_sha256\": \"{}\",\n",
                "  \"gpu_interface\": \"{}\"\n}}\n"
            ),
            request.request_id,
            operation_id,
            operation.as_str(),
            result.vm_id,
            result.state,
            result.previous_gpu_adapters,
            result.gpu_adapters,
            json_escape(&result.child),
            json_escape(&result.parent),
            result.parent_sha256,
            json_escape(&result.gpu_interface)
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

fn powershell_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn fixed_script(body: &str) -> Result<String, hyper_gpu_support::runner::RunnerError> {
    let policy = embedded_policy()?;
    Ok(format!(
        concat!(
            "$ErrorActionPreference = 'Stop'\n",
            "$vmId = [guid]{}\n$vmName = {}\n",
            "$parentPath = {}\n$parentHash = {}\n",
            "$childPath = {}\n$gpuPath = {}\n{}"
        ),
        powershell_literal(policy.vm_id()),
        powershell_literal(policy.vm_name()),
        powershell_literal(policy.parent()),
        powershell_literal(policy.parent_sha256()),
        powershell_literal(policy.child()),
        powershell_literal(policy.gpu_interface()),
        body
    ))
}

fn run_bounded_with_powershell(
    script: &str,
    timeout: Duration,
    powershell_path: &Path,
) -> Result<String, String> {
    run_supervised(script, timeout, timeout, powershell_path, "fixed adapter")
}

// Reserve time for result publication before Task Scheduler's outer limit.
fn adapter_budget(
    task_limit: Duration,
    following_operation: Duration,
    elapsed: Duration,
) -> Duration {
    task_limit.saturating_sub(elapsed + following_operation + Duration::from_secs(30))
}

fn require_transition_budget(
    task_limit: Duration,
    transition: Duration,
    elapsed: Duration,
) -> Result<(), std::io::Error> {
    if adapter_budget(task_limit, Duration::ZERO, elapsed) < transition {
        return Err(std::io::Error::other(
            "insufficient runner budget to begin native transition",
        ));
    }
    Ok(())
}

fn run_supervised(
    script: &str,
    idle_limit: Duration,
    total_limit: Duration,
    powershell_path: &Path,
    phase: &str,
) -> Result<String, String> {
    if total_limit.is_zero() || idle_limit.is_zero() {
        return Err(format!("{phase}: no execution budget available"));
    }
    let mut child = Command::new(powershell_path)
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
    let started = Instant::now();
    let mut watchdog = supervision::Watchdog::new(idle_limit, total_limit);
    let outcome = loop {
        if overflow.load(Ordering::Acquire) {
            break Err("fixed adapter output exceeded limit".to_owned());
        }
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) => {
                let progress = supervision::read_bytes(&child)
                    .and_then(|bytes| watchdog.observe(started.elapsed(), bytes));
                if let Err(error) = progress {
                    break Err(format!("{phase}: {error}"));
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(error) => break Err(format!("cannot wait for fixed adapter: {error}")),
        }
    };
    if outcome.is_err() {
        // Always reap the adapter and drain both bounded readers on failure.
        let _ = child.kill();
        let _ = child.wait();
    }
    let stdout = stdout_reader.join();
    let stderr = stderr_reader.join();
    let status = outcome?;
    let stdout = stdout
        .map_err(|_| "stdout reader failed")?
        .map_err(|error| error.to_string())?;
    let stderr = stderr
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

#[cfg(test)]
fn run_bounded(script: &str, timeout: Duration) -> Result<String, String> {
    let project = project_configuration().map_err(|error| error.to_string())?;
    run_bounded_with_powershell(script, timeout, &project.guest.powershell_path)
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
if ($gpuAdapters.Count -gt 1 -or ($gpuAdapters.Count -eq 1 -and $gpuAdapters[0].InstancePath -ine $gpuPath)) { throw 'GPU adapter identity rejected' }
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

const ASSIGN_GPU_SCRIPT: &str = r#"
$vm = Get-VM -Id $vmId -ErrorAction Stop
if ($vm.Name -ne $vmName -or $vm.State -ne 'Off' -or $vm.Generation -ne 2 -or [string]$vm.Version -ne '12.0') { throw 'enrolled VM identity or GPU assignment state mismatch' }
if ($vm.AutomaticCheckpointsEnabled -or @(Get-VMSnapshot -VM $vm -ErrorAction Stop).Count -ne 0) { throw 'checkpoint state rejected' }
$drives = @(Get-VMHardDiskDrive -VM $vm -ErrorAction Stop)
if ($drives.Count -ne 1 -or $drives[0].Path -ine $childPath) { throw 'enrolled child attachment mismatch' }
$childItem = Get-Item -LiteralPath $childPath -ErrorAction Stop
if ($childItem.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'child reparse point rejected' }
$child = Get-VHD -Path $childPath -ErrorAction Stop
if ($child.ParentPath -ine $parentPath -or $child.VhdType -ne 'Differencing') { throw 'child chain mismatch' }
$parent = Get-Item -LiteralPath $parentPath -ErrorAction Stop
if (-not $parent.IsReadOnly -or ($parent.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'parent protection mismatch' }
$hostGpu = Get-VMHostPartitionableGpu -Name $gpuPath -ErrorAction Stop
if ($hostGpu.Name -ine $gpuPath) { throw 'partitionable GPU identity mismatch' }
$gpuAdapters = @(Get-VMGpuPartitionAdapter -VM $vm -ErrorAction Stop)
if ($gpuAdapters.Count -ne 0) { throw 'GPU adapter already assigned' }
$otherGpuAssignments = @(Get-VM -ErrorAction Stop | Where-Object { $_.Id -ne $vmId } | ForEach-Object { Get-VMGpuPartitionAdapter -VM $_ -ErrorAction Stop })
if ($otherGpuAssignments.Count -ne 0) { throw 'another VM GPU assignment rejected' }
Add-VMGpuPartitionAdapter -VM $vm -InstancePath $gpuPath -ErrorAction Stop | Out-Null
$verifiedVm = Get-VM -Id $vmId -ErrorAction Stop
$verifiedAdapters = @(Get-VMGpuPartitionAdapter -VM $verifiedVm -ErrorAction Stop)
if ($verifiedVm.State -ne 'Off' -or $verifiedAdapters.Count -ne 1 -or $verifiedAdapters[0].InstancePath -ine $gpuPath) { throw 'GPU assignment verification failed' }
[Console]::Out.WriteLine('status' + "`t" + 'ok')
[Console]::Out.WriteLine('vm_id' + "`t" + $vmId.ToString().ToLowerInvariant())
[Console]::Out.WriteLine('state' + "`t" + [string]$verifiedVm.State)
[Console]::Out.WriteLine('previous_gpu_adapters' + "`t" + '0')
[Console]::Out.WriteLine('gpu_adapters' + "`t" + $verifiedAdapters.Count)
[Console]::Out.WriteLine('child' + "`t" + $child.Path)
[Console]::Out.WriteLine('parent' + "`t" + $child.ParentPath)
[Console]::Out.WriteLine('parent_sha256' + "`t" + $parentHash)
[Console]::Out.WriteLine('gpu_interface' + "`t" + $verifiedAdapters[0].InstancePath)
"#;

const REMOVE_GPU_SCRIPT: &str = r#"
$vm = Get-VM -Id $vmId -ErrorAction Stop
if ($vm.Name -ne $vmName -or $vm.State -ne 'Off' -or $vm.Generation -ne 2 -or [string]$vm.Version -ne '12.0') { throw 'enrolled VM identity or GPU removal state mismatch' }
if ($vm.AutomaticCheckpointsEnabled -or @(Get-VMSnapshot -VM $vm -ErrorAction Stop).Count -ne 0) { throw 'checkpoint state rejected' }
$drives = @(Get-VMHardDiskDrive -VM $vm -ErrorAction Stop)
if ($drives.Count -ne 1 -or $drives[0].Path -ine $childPath) { throw 'enrolled child attachment mismatch' }
$childItem = Get-Item -LiteralPath $childPath -ErrorAction Stop
if ($childItem.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'child reparse point rejected' }
$child = Get-VHD -Path $childPath -ErrorAction Stop
if ($child.ParentPath -ine $parentPath -or $child.VhdType -ne 'Differencing') { throw 'child chain mismatch' }
$parent = Get-Item -LiteralPath $parentPath -ErrorAction Stop
if (-not $parent.IsReadOnly -or ($parent.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'parent protection mismatch' }
$gpuAdapters = @(Get-VMGpuPartitionAdapter -VM $vm -ErrorAction Stop)
if ($gpuAdapters.Count -ne 1 -or $gpuAdapters[0].InstancePath -ine $gpuPath) { throw 'GPU adapter identity rejected' }
Remove-VMGpuPartitionAdapter -VMGpuPartitionAdapter $gpuAdapters[0] -ErrorAction Stop
$verifiedVm = Get-VM -Id $vmId -ErrorAction Stop
$verifiedAdapters = @(Get-VMGpuPartitionAdapter -VM $verifiedVm -ErrorAction Stop)
if ($verifiedVm.State -ne 'Off' -or $verifiedAdapters.Count -ne 0) { throw 'GPU removal verification failed' }
[Console]::Out.WriteLine('status' + "`t" + 'ok')
[Console]::Out.WriteLine('vm_id' + "`t" + $vmId.ToString().ToLowerInvariant())
[Console]::Out.WriteLine('state' + "`t" + [string]$verifiedVm.State)
[Console]::Out.WriteLine('previous_gpu_adapters' + "`t" + '1')
[Console]::Out.WriteLine('gpu_adapters' + "`t" + $verifiedAdapters.Count)
[Console]::Out.WriteLine('child' + "`t" + $child.Path)
[Console]::Out.WriteLine('parent' + "`t" + $child.ParentPath)
[Console]::Out.WriteLine('parent_sha256' + "`t" + $parentHash)
[Console]::Out.WriteLine('gpu_interface' + "`t" + $gpuPath)
"#;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{Duration, Instant};

    use super::{
        ASSIGN_GPU_SCRIPT, INSPECT_SCRIPT, LockFile, RECONCILIATION_MARKER, REMOVE_GPU_SCRIPT,
        RESET_SCRIPT, SHUTDOWN_SCRIPT, START_SCRIPT, append_audit, begin_reconciliation,
        ensure_reconciled, fixed_script, handle_request, json_escape, project_configuration,
        run_bounded, run_reconciled_operation, startup_failure_message,
    };
    use hyper_gpu_support::runner::{Operation, POLICY_V1, Request};

    #[test]
    fn fixed_script_has_no_external_input_or_broad_vm_deletion() {
        let policy = hyper_gpu_support::runner::embedded_policy().unwrap();
        let start_script = fixed_script(START_SCRIPT).unwrap();
        let shutdown_script = fixed_script(SHUTDOWN_SCRIPT).unwrap();
        let assign_script = fixed_script(ASSIGN_GPU_SCRIPT).unwrap();
        let remove_script = fixed_script(REMOVE_GPU_SCRIPT).unwrap();
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
        for script in [&start_script, &shutdown_script] {
            assert!(script.contains(policy.vm_id()));
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
        assert!(
            ASSIGN_GPU_SCRIPT.contains("Add-VMGpuPartitionAdapter -VM $vm -InstancePath $gpuPath")
        );
        assert!(
            REMOVE_GPU_SCRIPT
                .contains("Remove-VMGpuPartitionAdapter -VMGpuPartitionAdapter $gpuAdapters[0]")
        );
        for script in [&assign_script, &remove_script] {
            assert!(script.contains(policy.vm_id()));
            assert!(script.contains("$vm.State -ne 'Off'"));
            assert!(script.contains("Get-VMSnapshot"));
            assert!(script.contains("Get-VMGpuPartitionAdapter"));
            assert!(!script.contains("Start-VM"));
            assert!(!script.contains("Stop-VM"));
            assert!(!script.contains("Invoke-Expression"));
            assert!(!script.contains("param("));
            assert!(!script.contains("SilentlyContinue"));
            assert!(!script.contains("Get-FileHash"));
        }
    }

    #[test]
    fn lifecycle_deadlines_include_parent_inspection_and_transition_allowance() {
        let project = project_configuration().unwrap();
        let runner = project.runner;
        assert!(!runner.reset_timeout.is_zero());
        assert!(!runner.inspect_timeout.is_zero());
        assert!(!runner.start_timeout.is_zero());
        assert!(!runner.shutdown_timeout.is_zero());
        assert!(!runner.gpu_assignment_timeout.is_zero());
        let installer = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scripts/setup/install-runner-v1.ps1"
        ));
        assert!(installer.contains("runner.task_execution_timeout_seconds"));
        assert!(runner.inspect_timeout + runner.start_timeout < runner.task_execution_timeout);
        assert!(runner.inspect_timeout + runner.shutdown_timeout < runner.task_execution_timeout);
        assert!(
            runner.inspect_timeout + runner.gpu_assignment_timeout < runner.task_execution_timeout
        );
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
    fn reset_failure_paths_retain_reconciliation_marker() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("local")
            .join("test-work")
            .join(format!(
                "hyper-gpu-runner-reset-reconcile-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        let request = Request {
            request_id: "0123456789abcdef0123456789abcdef".into(),
            operation: Operation::ResetSlot,
            nonce: "fedcba9876543210fedcba9876543210".into(),
            plan_fingerprint: "0".repeat(64),
        };

        for scenario in ["timeout", "publication", "audit"] {
            let directory = root.join(scenario);
            let state = directory.join("state");
            let results = directory.join("results");
            let audit = directory.join("audit");
            fs::create_dir_all(&state).unwrap();
            fs::create_dir_all(&results).unwrap();
            fs::create_dir_all(&audit).unwrap();
            let operation_id = format!("1790391920-00000000{}", scenario.len());

            if scenario == "publication" {
                fs::write(results.join(format!("{operation_id}.tmp")), "occupied").unwrap();
            }
            let result = run_reconciled_operation(
                &state,
                &results,
                &audit,
                &request,
                Operation::ResetSlot,
                &operation_id,
                || {
                    if scenario == "timeout" {
                        return Err(std::io::Error::other("fixed Hyper-V adapter timed out").into());
                    }
                    if scenario == "audit" {
                        fs::remove_file(audit.join("events.jsonl"))?;
                        fs::remove_dir(&audit)?;
                        fs::write(&audit, "not a directory")?;
                    }
                    Ok("{\"status\":\"succeeded\"}\n".into())
                },
            );

            assert!(result.is_err(), "{scenario} unexpectedly succeeded");
            assert_eq!(
                fs::read_to_string(state.join(RECONCILIATION_MARKER)).unwrap(),
                format!("reset-slot {operation_id}\n")
            );
        }

        let directory = root.join("success");
        let state = directory.join("state");
        let results = directory.join("results");
        let audit = directory.join("audit");
        fs::create_dir_all(&state).unwrap();
        fs::create_dir_all(&results).unwrap();
        fs::create_dir_all(&audit).unwrap();
        let operation_id = "1790391920-000000099";
        run_reconciled_operation(
            &state,
            &results,
            &audit,
            &request,
            Operation::ResetSlot,
            operation_id,
            || Ok("{\"status\":\"succeeded\"}\n".into()),
        )
        .unwrap();
        assert!(!state.join(RECONCILIATION_MARKER).exists());
        assert!(results.join(format!("{operation_id}.json")).exists());
        assert!(
            fs::read_to_string(audit.join("events.jsonl"))
                .unwrap()
                .contains("\"status\":\"succeeded\"")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn malformed_pipe_request_returns_bounded_failure() {
        let response = handle_request(b"not-a-request", Instant::now());
        assert_eq!(response.status, "failed");
        assert_eq!(response.diagnostic, "invalid-request");
        assert!(response.operation_id.is_none());
    }

    #[test]
    fn fixed_adapter_timeout_and_output_limit_cancel_process() {
        let started = Instant::now();
        assert!(
            run_bounded("Start-Sleep -Seconds 5", Duration::from_millis(100))
                .unwrap_err()
                .contains("overall execution budget exhausted")
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
$script:adapterPath = '__GPU_INTERFACE__'
$script:hostGpuPath = '__GPU_INTERFACE__'
$script:adapterCount = 0
$script:otherAssignment = $false
$script:denyGpuQuery = $false
$script:freeMemory = [uint64]12582912
function Get-VM { [CmdletBinding()] param([guid]$Id) $target = [pscustomobject]@{ Id = [guid]'__VM_ID__'; Name = '__VM_NAME__'; State = $script:state; Generation = 2; Version = '12.0'; AutomaticCheckpointsEnabled = $false }; if ($PSBoundParameters.ContainsKey('Id')) { $target } else { $target; if ($script:otherAssignment) { [pscustomobject]@{ Id = [guid]'00000000-0000-0000-0000-000000000001'; Name = 'Other' } } } }
function Get-VMSnapshot { [CmdletBinding()] param($VM) }
function Get-VMHardDiskDrive { [CmdletBinding()] param($VM) [pscustomobject]@{ Path = '__CHILD__' } }
function Get-Item { [CmdletBinding()] param([string]$LiteralPath) [pscustomobject]@{ IsReadOnly = $true; Attributes = [IO.FileAttributes]::Normal } }
function Get-VHD { [CmdletBinding()] param([string]$Path) [pscustomobject]@{ Path = '__CHILD__'; ParentPath = '__PARENT__'; VhdType = 'Differencing' } }
function Get-FileHash { [CmdletBinding()] param([string]$LiteralPath, [string]$Algorithm) [pscustomobject]@{ Hash = '__PARENT_HASH__' } }
function Get-VMHostPartitionableGpu { [CmdletBinding()] param([string]$Name) [pscustomobject]@{ Name = $script:hostGpuPath } }
function Get-VMGpuPartitionAdapter { [CmdletBinding()] param($VM) if ($script:denyGpuQuery) { throw 'gpu query denied' }; if (($VM.Id -eq [guid]'__VM_ID__' -and $script:adapterCount -eq 1) -or ($VM.Id -ne [guid]'__VM_ID__' -and $script:otherAssignment)) { [pscustomobject]@{ InstancePath = $script:adapterPath } } }
function Get-CimInstance { [CmdletBinding()] param([string]$ClassName) [pscustomobject]@{ FreePhysicalMemory = $script:freeMemory } }
function Start-VM { [CmdletBinding()] param($VM) $script:state = 'Running' }
function Stop-VM { [CmdletBinding(SupportsShouldProcess)] param($VM) $script:state = 'Off' }
function Add-VMGpuPartitionAdapter { [CmdletBinding()] param($VM,[string]$InstancePath) $script:adapterCount = 1 }
function Remove-VMGpuPartitionAdapter { [CmdletBinding()] param($VMGpuPartitionAdapter) $script:adapterCount = 0 }
"#;
        let policy = hyper_gpu_support::runner::embedded_policy().unwrap();
        let fakes = FAKES
            .replace("__GPU_INTERFACE__", policy.gpu_interface())
            .replace("__VM_ID__", policy.vm_id())
            .replace("__VM_NAME__", policy.vm_name())
            .replace("__CHILD__", policy.child())
            .replace("__PARENT__", policy.parent())
            .replace("__PARENT_HASH__", policy.parent_sha256());
        let start_script = fixed_script(START_SCRIPT).unwrap();
        let shutdown_script = fixed_script(SHUTDOWN_SCRIPT).unwrap();
        let assign_script = fixed_script(ASSIGN_GPU_SCRIPT).unwrap();
        let remove_script = fixed_script(REMOVE_GPU_SCRIPT).unwrap();
        let (_, inspect_after_token_check) = INSPECT_SCRIPT.split_once("$vm =").unwrap();
        let inspect_script = fixed_script(&format!("$vm ={inspect_after_token_check}")).unwrap();
        let start_output =
            run_bounded(&format!("{fakes}\n{start_script}"), Duration::from_secs(5)).unwrap();
        hyper_gpu_support::runner::parse_lifecycle_result(&start_output, Operation::StartSlot)
            .unwrap();

        let shutdown_output = run_bounded(
            &format!("{fakes}\n$script:state = 'Running'\n{shutdown_script}"),
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
                &format!("{fakes}\n$script:adapterCount = 1\n$script:adapterPath = 'wrong'\n{start_script}"),
                Duration::from_secs(5)
            )
            .unwrap_err()
            .contains("GPU adapter identity rejected")
        );
        let inspect_error = run_bounded(
            &format!(
                "{fakes}\n$script:adapterCount = 1\n$script:adapterPath = 'wrong'\n{inspect_script}"
            ),
            Duration::from_secs(5),
        )
        .unwrap_err();
        assert!(
            inspect_error.contains("GPU adapter identity rejected"),
            "unexpected inspect error: {inspect_error}"
        );
        assert!(
            run_bounded(
                &format!("{fakes}\n$script:otherAssignment = $true\n{start_script}"),
                Duration::from_secs(5)
            )
            .unwrap_err()
            .contains("another VM GPU assignment rejected")
        );
        assert!(
            run_bounded(
                &format!("{fakes}\n$script:freeMemory = 12582911\n{start_script}"),
                Duration::from_secs(5)
            )
            .unwrap_err()
            .contains("host available memory below 12 GiB")
        );
        assert!(
            run_bounded(
                &format!("{fakes}\n$script:denyGpuQuery = $true\n{start_script}"),
                Duration::from_secs(5)
            )
            .unwrap_err()
            .contains("gpu query denied")
        );

        let assign_output =
            run_bounded(&format!("{fakes}\n{assign_script}"), Duration::from_secs(5)).unwrap();
        hyper_gpu_support::runner::parse_gpu_assignment_result(
            &assign_output,
            Operation::AssignGpu,
        )
        .unwrap();
        let remove_output = run_bounded(
            &format!("{fakes}\n$script:adapterCount = 1\n{remove_script}"),
            Duration::from_secs(5),
        )
        .unwrap();
        hyper_gpu_support::runner::parse_gpu_assignment_result(
            &remove_output,
            Operation::RemoveGpu,
        )
        .unwrap();
        assert!(
            run_bounded(
                &format!("{fakes}\n$script:adapterCount = 1\n{assign_script}"),
                Duration::from_secs(5),
            )
            .unwrap_err()
            .contains("GPU adapter already assigned")
        );
        assert!(
            run_bounded(
                &format!("{fakes}\n$script:otherAssignment = $true\n{assign_script}"),
                Duration::from_secs(5),
            )
            .unwrap_err()
            .contains("another VM GPU assignment rejected")
        );
        assert!(
            run_bounded(
                &format!("{fakes}\n$script:denyGpuQuery = $true\n{assign_script}"),
                Duration::from_secs(5),
            )
            .unwrap_err()
            .contains("gpu query denied")
        );
        assert!(
            run_bounded(&format!("{fakes}\n{remove_script}"), Duration::from_secs(5),)
                .unwrap_err()
                .contains("GPU adapter identity rejected")
        );
    }
}
