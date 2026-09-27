//! Unelevated client for the administrator-installed one-shot GPU-PV runner.

use std::fs;
use std::process::{Command, ExitCode, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hyper_gpu_support::runner::{
    Operation, PIPE_NAME, Request, parse_enrollment, parse_response, policy_fingerprint,
};
use hyper_gpu_support::windows_runner::{current_user_sid_string, transact};
use sha2::{Digest, Sha256};

const TASK_NAME: &str = r"\HyperGpuSupport\Runner-v1";
const ENROLLMENT_PATH: &str = r"C:\ProgramData\HyperGpuSupport\Runner\enrollment-v1.json";
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
    let mut arguments = std::env::args().skip(1);
    let operation = arguments
        .next()
        .and_then(|value| Operation::parse(&value))
        .ok_or("one fixed runner operation is required")?;
    if arguments.next().is_some() {
        return Err("runner operations accept no target, path or command arguments".into());
    }
    let request = build_request(operation)?;
    let enrollment = parse_enrollment(&fs::read_to_string(ENROLLMENT_PATH)?)?;
    if current_user_sid_string()? != enrollment.client_sid() {
        return Err("current user is not the enrolled runner client".into());
    }
    trigger_runner()?;
    let bytes = transact(
        PIPE_NAME,
        enrollment.runner_sid(),
        request.encode().as_bytes(),
        Duration::from_secs(15),
    )?;
    let text = std::str::from_utf8(&bytes).map_err(|_| "runner response was not UTF-8")?;
    let response = parse_response(text)?;
    if response.request_id != request.request_id {
        return Err("runner response request ID mismatch".into());
    }
    println!("{}", response.encode().trim_end());
    if response.status != "succeeded" {
        return Err(format!("runner rejected request: {}", response.diagnostic).into());
    }
    Ok(())
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

fn trigger_runner() -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("schtasks.exe")
        .args(["/Run", "/TN", TASK_NAME])
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
    use hyper_gpu_support::runner::{Operation, parse_request, policy_fingerprint};

    use super::{build_request, fresh_token};

    #[test]
    fn generated_request_is_canonical_and_policy_bound() {
        let request = build_request(Operation::ResetSlot).unwrap();
        assert_eq!(request.plan_fingerprint, policy_fingerprint());
        assert_eq!(parse_request(&request.encode()), Ok(request));
    }

    #[test]
    fn generated_tokens_are_canonical_and_distinct() {
        let first = fresh_token("nonce").unwrap();
        let second = fresh_token("nonce").unwrap();
        assert_eq!(first.len(), 32);
        assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_ne!(first, second);
    }
}
