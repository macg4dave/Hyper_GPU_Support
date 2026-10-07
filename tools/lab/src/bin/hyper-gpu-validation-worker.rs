//! Fixed Rust guest readiness/workload worker. No arbitrary execution arguments.

use hyper_gpu_support::{
    config::ProjectConfiguration,
    validation::{CheckStatus, ValidationReport, validate_workloads, verify_inputs},
    windows_validation::{LocalValidationAdapter, WorkerRequest},
};
use std::{
    io::{Read, Write},
    process::ExitCode,
};

fn main() -> ExitCode {
    match run() {
        Ok((report, _deadline)) => {
            let code = report.exit_code();
            match serde_json::to_string(&report) {
                Ok(json) => match std::io::stdout().lock().write_all(json.as_bytes()) {
                    Ok(()) => ExitCode::from(code),
                    Err(_) => ExitCode::FAILURE,
                },
                Err(_) => ExitCode::FAILURE,
            }
        }
        Err(error) => {
            eprintln!("validation worker: {error}");
            ExitCode::from(2)
        }
    }
}
fn run() -> Result<
    (
        ValidationReport,
        hyper_gpu_support::windows_validation::WorkerDeadline,
    ),
    String,
> {
    if std::env::args_os().len() != 1 {
        return Err("validation worker accepts no arguments".into());
    }
    let project = ProjectConfiguration::embedded().map_err(|e| e.to_string())?;
    let deadline = hyper_gpu_support::windows_validation::WorkerDeadline::start(
        std::time::Duration::from_secs(project.validation.worker_timeout_seconds),
    );
    let mut payload = Vec::new();
    std::io::stdin()
        .take(16385)
        .read_to_end(&mut payload)
        .map_err(|e| e.to_string())?;
    if payload.len() > 16384 {
        return Err("worker request exceeds bound".into());
    }
    let request: WorkerRequest =
        serde_json::from_slice(&payload).map_err(|_| "invalid worker request")?;
    let root = project.guest.staging_root.join("Validation");
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    if executable != root.join("hyper-gpu-validation-worker.exe") {
        return Err("worker must run from its configured protected directory".into());
    }
    use std::os::windows::fs::OpenOptionsExt;
    let _lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .share_mode(0)
        .open(root.join("validation.lock"))
        .map_err(|e| format!("validation worker lock: {e}"))?;
    let mut report = ValidationReport::new(&project);
    report.architecture = if cfg!(target_arch = "x86_64") {
        "x64"
    } else {
        "unsupported"
    }
    .into();
    report.guest_build = hyper_gpu_support::windows_validation::os_build()?;
    report.inputs = request.inputs;
    if let Err(error) = verify_inputs(&root, &report.inputs) {
        report.block(error);
        return Ok((report, deadline));
    }
    report.checks[0].status = CheckStatus::Pass;
    let mut adapter = LocalValidationAdapter::new(root);
    validate_workloads(&project, &mut report, &mut adapter);
    // Execution must not silently change pinned inputs during a successful run.
    if let Err(error) = verify_inputs(
        &project.guest.staging_root.join("Validation"),
        &report.inputs,
    ) {
        report.block(error);
    }
    Ok((report, deadline))
}
