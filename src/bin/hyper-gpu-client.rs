//! Unelevated client for the administrator-installed one-shot GPU-PV runner.

use std::fs;
use std::process::{Command, ExitCode, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hyper_gpu_support::config::{ProjectConfiguration, RunnerConfiguration};
use hyper_gpu_support::runner::{
    FRAME_LIMIT, GpuAssignmentResult, InspectResult, Operation, PIPE_NAME, Request, Response,
    embedded_policy, parse_enrollment, parse_response, policy_fingerprint,
};
use hyper_gpu_support::windows_paths::task_scheduler_executable;
use hyper_gpu_support::windows_runner::{current_user_sid_string, transact, wait_for_task_ready};
use serde::Deserialize;
use sha2::{Digest, Sha256};

static TOKEN_COUNTER: AtomicU64 = AtomicU64::new(0);

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("runner client error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let project = ProjectConfiguration::embedded()?;
    let mut arguments = std::env::args().skip(1);
    let command = arguments
        .next()
        .ok_or("one fixed runner operation or ensure-gpu is required")?;
    if arguments.next().is_some() {
        return Err("runner operations accept no target, path or command arguments".into());
    }
    let enrollment_path = project.runner.data_directory.join("enrollment-v1.json");
    let enrollment = parse_enrollment(&fs::read_to_string(enrollment_path)?)?;
    if current_user_sid_string()? != enrollment.client_sid() {
        return Err("current user is not the enrolled runner client".into());
    }
    if command == "ensure-gpu" {
        return ensure_gpu(&project.runner, enrollment.runner_sid());
    }
    let operation = Operation::parse(&command).ok_or("unknown fixed runner operation")?;
    let (_, response) = submit_operation(operation, &project.runner, enrollment.runner_sid())?;
    println!("{}", response.encode().trim_end());
    require_succeeded(&response)?;
    Ok(())
}

fn submit_operation(
    operation: Operation,
    runner: &RunnerConfiguration,
    runner_sid: &str,
) -> Result<(Request, hyper_gpu_support::runner::Response), Box<dyn std::error::Error>> {
    let request = build_request(operation)?;
    trigger_runner(runner)?;
    let encoded_request = request.encode()?;
    let bytes = transact(
        PIPE_NAME,
        runner_sid,
        encoded_request.as_bytes(),
        response_timeout(operation, runner),
    )?;
    let text = std::str::from_utf8(&bytes).map_err(|_| "runner response was not UTF-8")?;
    let response = parse_response(text)?;
    if response.request_id != request.request_id {
        return Err("runner response request ID mismatch".into());
    }
    Ok((request, response))
}

fn require_succeeded(response: &Response) -> Result<(), Box<dyn std::error::Error>> {
    if response.status == "succeeded" {
        Ok(())
    } else {
        Err(format!("runner rejected request: {}", response.diagnostic).into())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PublishedInspectResult {
    schema: u32,
    request_id: String,
    operation_id: String,
    operation: String,
    status: String,
    vm_id: String,
    state: String,
    gpu_adapters: u32,
    child: String,
    parent: String,
    parent_sha256: String,
    gpu_interface: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PublishedGpuResult {
    schema: u32,
    request_id: String,
    operation_id: String,
    operation: String,
    status: String,
    vm_id: String,
    state: String,
    previous_gpu_adapters: u32,
    gpu_adapters: u32,
    child: String,
    parent: String,
    parent_sha256: String,
    gpu_interface: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssignmentDecision {
    Assign,
    AlreadyAssigned,
}

fn assignment_decision(result: &InspectResult) -> Result<AssignmentDecision, &'static str> {
    if result.state != "Off" {
        return Err("the enrolled VM must be off before ensuring GPU assignment");
    }
    match result.gpu_adapters {
        0 => Ok(AssignmentDecision::Assign),
        1 => Ok(AssignmentDecision::AlreadyAssigned),
        _ => Err("the inspected GPU adapter count is invalid"),
    }
}

fn ensure_gpu(
    runner: &RunnerConfiguration,
    runner_sid: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (inspect_request, inspect_response) =
        submit_operation(Operation::Inspect, runner, runner_sid)?;
    require_succeeded(&inspect_response)?;
    let inspect_operation_id = inspect_response
        .operation_id
        .as_deref()
        .ok_or("successful inspection omitted its operation ID")?;
    let inspect = parse_published_inspect(
        &read_result(runner, inspect_operation_id)?,
        &inspect_request.request_id,
        inspect_operation_id,
    )?;

    let (status, result) = match assignment_decision(&inspect)? {
        AssignmentDecision::AlreadyAssigned => ("already-assigned", inspect),
        AssignmentDecision::Assign => {
            let (request, response) = submit_operation(Operation::AssignGpu, runner, runner_sid)?;
            require_succeeded(&response)?;
            let operation_id = response
                .operation_id
                .as_deref()
                .ok_or("successful GPU assignment omitted its operation ID")?;
            let assigned = parse_published_gpu(
                &read_result(runner, operation_id)?,
                &request.request_id,
                operation_id,
            )?;
            (
                "assigned",
                InspectResult {
                    vm_id: assigned.vm_id,
                    state: assigned.state,
                    gpu_adapters: assigned.gpu_adapters,
                    child: assigned.child,
                    parent: assigned.parent,
                    parent_sha256: assigned.parent_sha256,
                    gpu_interface: assigned.gpu_interface,
                },
            )
        }
    };
    println!("attachment_status={status}");
    println!("vm_id={}", result.vm_id);
    println!("state={}", result.state);
    println!("gpu_adapters={}", result.gpu_adapters);
    println!("gpu_interface={}", result.gpu_interface);
    println!("staging_readiness=not-checked");
    Ok(())
}

fn read_result(
    runner: &RunnerConfiguration,
    operation_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let path = runner
        .data_directory
        .join("results")
        .join(format!("{operation_id}.json"));
    if fs::metadata(&path)?.len() > FRAME_LIMIT as u64 {
        return Err("published runner result exceeded the protocol limit".into());
    }
    Ok(fs::read_to_string(path)?)
}

fn parse_published_inspect(
    input: &str,
    request_id: &str,
    operation_id: &str,
) -> Result<InspectResult, &'static str> {
    let record: PublishedInspectResult =
        serde_json::from_str(input).map_err(|_| "invalid published inspection result")?;
    let policy = embedded_policy().map_err(|_| "invalid embedded runner policy")?;
    if record.schema != 1
        || record.request_id != request_id
        || record.operation_id != operation_id
        || record.operation != Operation::Inspect.as_str()
        || record.status != "succeeded"
        || record.vm_id != policy.vm_id()
        || !matches!(record.state.as_str(), "Off" | "Running")
        || record.gpu_adapters > 1
        || record.child != policy.child()
        || record.parent != policy.parent()
        || record.parent_sha256 != policy.parent_sha256()
        || record.gpu_interface != policy.gpu_interface()
    {
        return Err("published inspection result did not match the enrolled target");
    }
    Ok(InspectResult {
        vm_id: record.vm_id,
        state: record.state,
        gpu_adapters: record.gpu_adapters,
        child: record.child,
        parent: record.parent,
        parent_sha256: record.parent_sha256,
        gpu_interface: record.gpu_interface,
    })
}

fn parse_published_gpu(
    input: &str,
    request_id: &str,
    operation_id: &str,
) -> Result<GpuAssignmentResult, &'static str> {
    let record: PublishedGpuResult =
        serde_json::from_str(input).map_err(|_| "invalid published GPU assignment result")?;
    let policy = embedded_policy().map_err(|_| "invalid embedded runner policy")?;
    if record.schema != 1
        || record.request_id != request_id
        || record.operation_id != operation_id
        || record.operation != Operation::AssignGpu.as_str()
        || record.status != "succeeded"
        || record.vm_id != policy.vm_id()
        || record.state != "Off"
        || record.previous_gpu_adapters != 0
        || record.gpu_adapters != 1
        || record.child != policy.child()
        || record.parent != policy.parent()
        || record.parent_sha256 != policy.parent_sha256()
        || record.gpu_interface != policy.gpu_interface()
    {
        return Err("published GPU assignment result did not match the requested transition");
    }
    Ok(GpuAssignmentResult {
        vm_id: record.vm_id,
        state: record.state,
        previous_gpu_adapters: record.previous_gpu_adapters,
        gpu_adapters: record.gpu_adapters,
        child: record.child,
        parent: record.parent,
        parent_sha256: record.parent_sha256,
        gpu_interface: record.gpu_interface,
    })
}

fn response_timeout(operation: Operation, runner: &RunnerConfiguration) -> Duration {
    let transport_allowance = Duration::from_secs(30);
    match operation {
        Operation::Inspect
        | Operation::ResetSlot
        | Operation::StartSlot
        | Operation::ShutdownSlot
        | Operation::AssignGpu
        | Operation::RemoveGpu => runner.task_execution_timeout + transport_allowance,
        _ => Duration::from_secs(15),
    }
}

fn build_request(operation: Operation) -> Result<Request, std::time::SystemTimeError> {
    Ok(Request {
        request_id: fresh_token("request")?,
        operation,
        nonce: fresh_token("nonce")?,
        plan_fingerprint: policy_fingerprint(),
    })
}

fn fresh_token(domain: &str) -> Result<String, std::time::SystemTimeError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let counter = TOKEN_COUNTER.fetch_add(1, Ordering::Relaxed);
    let digest =
        Sha256::digest(format!("{domain}:{}:{now}:{counter}", std::process::id()).as_bytes());
    Ok(digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn trigger_runner(runner: &RunnerConfiguration) -> Result<(), Box<dyn std::error::Error>> {
    wait_for_task_ready(
        &runner.task_path,
        &runner.task_name,
        runner.task_execution_timeout,
    )?;
    let task_name = format!("{}{}", runner.task_path, runner.task_name);
    let status = Command::new(task_scheduler_executable()?)
        .args(["/Run", "/TN", &task_name])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if !status.success() {
        return Err(format!("fixed runner task trigger failed with {status}").into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use hyper_gpu_support::config::ProjectConfiguration;
    use hyper_gpu_support::runner::{
        InspectResult, Operation, embedded_policy, parse_request, policy_fingerprint,
    };

    use super::{
        AssignmentDecision, assignment_decision, build_request, fresh_token, parse_published_gpu,
        parse_published_inspect, response_timeout,
    };

    #[test]
    fn generated_request_is_canonical_and_policy_bound() {
        let request = build_request(Operation::ResetSlot).unwrap();
        assert_eq!(request.plan_fingerprint, policy_fingerprint());
        assert_eq!(parse_request(&request.encode().unwrap()), Ok(request));
    }

    #[test]
    fn generated_tokens_are_canonical_and_distinct() {
        let first = fresh_token("nonce").unwrap();
        let second = fresh_token("nonce").unwrap();
        assert_eq!(first.len(), 32);
        assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_ne!(first, second);
    }

    #[test]
    fn response_deadlines_cover_fixed_adapter_bounds() {
        let project = ProjectConfiguration::embedded().unwrap();
        let runner = &project.runner;
        assert_eq!(
            response_timeout(Operation::Inspect, runner),
            runner.task_execution_timeout + Duration::from_secs(30)
        );
        assert_eq!(
            response_timeout(Operation::StartSlot, runner),
            runner.task_execution_timeout + Duration::from_secs(30)
        );
        assert_eq!(
            response_timeout(Operation::ShutdownSlot, runner),
            runner.task_execution_timeout + Duration::from_secs(30)
        );
        assert_eq!(
            response_timeout(Operation::AssignGpu, runner),
            runner.task_execution_timeout + Duration::from_secs(30)
        );
        assert_eq!(
            response_timeout(Operation::RemoveGpu, runner),
            runner.task_execution_timeout + Duration::from_secs(30)
        );
        let installer = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scripts/setup/install-runner-v1.ps1"
        ));
        assert!(installer.contains("runner.task_execution_timeout_seconds"));
        assert!(response_timeout(Operation::StartSlot, runner) > runner.task_execution_timeout);
        assert!(response_timeout(Operation::ShutdownSlot, runner) > runner.task_execution_timeout);
    }

    fn inspected(state: &str, gpu_adapters: u32) -> InspectResult {
        let policy = embedded_policy().unwrap();
        InspectResult {
            vm_id: policy.vm_id().into(),
            state: state.into(),
            gpu_adapters,
            child: policy.child().into(),
            parent: policy.parent().into(),
            parent_sha256: policy.parent_sha256().into(),
            gpu_interface: policy.gpu_interface().into(),
        }
    }

    fn published_inspect(state: &str, gpu_adapters: u32) -> String {
        let result = inspected(state, gpu_adapters);
        serde_json::json!({
            "schema": 1,
            "request_id": "0123456789abcdef0123456789abcdef",
            "operation_id": "1790860476-127613900",
            "operation": "inspect",
            "status": "succeeded",
            "vm_id": result.vm_id,
            "state": result.state,
            "gpu_adapters": result.gpu_adapters,
            "child": result.child,
            "parent": result.parent,
            "parent_sha256": result.parent_sha256,
            "gpu_interface": result.gpu_interface,
        })
        .to_string()
    }

    #[test]
    fn assignment_decision_is_idempotent_and_rejects_state_drift() {
        assert_eq!(
            assignment_decision(&inspected("Off", 0)),
            Ok(AssignmentDecision::Assign)
        );
        assert_eq!(
            assignment_decision(&inspected("Off", 1)),
            Ok(AssignmentDecision::AlreadyAssigned)
        );
        assert_eq!(
            assignment_decision(&inspected("Running", 0)),
            Err("the enrolled VM must be off before ensuring GPU assignment")
        );
    }

    #[test]
    fn published_inspection_is_correlated_and_identity_checked() {
        let input = published_inspect("Off", 0);
        assert!(
            parse_published_inspect(
                &input,
                "0123456789abcdef0123456789abcdef",
                "1790860476-127613900"
            )
            .is_ok()
        );
        for invalid in [
            input.replace("GPUPARAV", "wrong"),
            input.replace("\"operation\":\"inspect\"", "\"operation\":\"assign-gpu\""),
            input.replace(
                "0123456789abcdef0123456789abcdef",
                "ffffffffffffffffffffffffffffffff",
            ),
            input.replace("\"status\":\"succeeded\"", "\"status\":\"failed\""),
        ] {
            assert!(
                parse_published_inspect(
                    &invalid,
                    "0123456789abcdef0123456789abcdef",
                    "1790860476-127613900"
                )
                .is_err()
            );
        }
    }

    #[test]
    fn published_assignment_requires_exact_zero_to_one_transition() {
        let policy = embedded_policy().unwrap();
        let input = serde_json::json!({
            "schema": 1,
            "request_id": "0123456789abcdef0123456789abcdef",
            "operation_id": "1790860476-127613900",
            "operation": "assign-gpu",
            "status": "succeeded",
            "vm_id": policy.vm_id(),
            "state": "Off",
            "previous_gpu_adapters": 0,
            "gpu_adapters": 1,
            "child": policy.child(),
            "parent": policy.parent(),
            "parent_sha256": policy.parent_sha256(),
            "gpu_interface": policy.gpu_interface(),
        })
        .to_string();
        assert!(
            parse_published_gpu(
                &input,
                "0123456789abcdef0123456789abcdef",
                "1790860476-127613900"
            )
            .is_ok()
        );
        for invalid in [
            input.replace("\"gpu_adapters\":1", "\"gpu_adapters\":0"),
            input.replace("\"previous_gpu_adapters\":0", "\"previous_gpu_adapters\":1"),
            input.replace("GPUPARAV", "foreign"),
            format!("{}{}", input.trim_end_matches('}'), ",\"unexpected\":true}"),
        ] {
            assert!(
                parse_published_gpu(
                    &invalid,
                    "0123456789abcdef0123456789abcdef",
                    "1790860476-127613900"
                )
                .is_err()
            );
        }
    }
}
