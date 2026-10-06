//! Public executable contracts, requiring no hardware or administrator rights.

use std::process::Command;

#[test]
fn inventory_worker_rejects_arguments_before_native_queries() {
    let output = Command::new(env!("CARGO_BIN_EXE_hyper-gpu-inventory-worker"))
        .arg("SELECT * FROM arbitrary")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("accepts no arguments"));
}

#[test]
fn driver_environment_inspector_rejects_arguments_before_native_discovery() {
    let output = Command::new(env!("CARGO_BIN_EXE_hyper-gpu-driver-environment"))
        .arg("--target")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("accepts no target"));
}

#[test]
fn help_and_default_invocation_succeed() {
    for arguments in [vec![], vec!["--help"], vec!["-h"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_hyper-gpu-support"))
            .args(arguments)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Usage: hyper-gpu-support"));
        assert!(stdout.contains("Inventory and validate are implemented"));
    }
}

#[test]
fn inventory_emits_a_versioned_report_without_elevation() {
    let output = Command::new(env!("CARGO_BIN_EXE_hyper-gpu-support"))
        .arg("inventory")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("inventory.schema=1\n"));
    assert!(stdout.contains("host.build=known:"));
    assert!(stdout.contains("gpu.model="));
    assert!(stdout.contains("gpup.interface="));
    assert!(stdout.contains("vm.selection="));
}

#[test]
fn version_matches_package_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_hyper-gpu-support"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        concat!("hyper-gpu-support ", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn invalid_usage_has_consistent_exit_code_and_no_stdout() {
    for arguments in [vec!["shell"], vec!["--version", "extra"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_hyper-gpu-support"))
            .args(arguments)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8(output.stderr).unwrap().trim(),
            "error: invalid arguments; use --help for usage"
        );
    }
}

#[test]
fn declared_operations_have_stable_not_implemented_exit() {
    for operation in [
        "plan", "apply", "status", "remove", "recover", "start", "shutdown", "restart",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_hyper-gpu-support"))
            .arg(operation)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(70));
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8(output.stderr).unwrap().trim(),
            format!("error: {operation} is declared but not implemented")
        );
    }
}

#[test]
fn public_validate_reports_missing_prerequisites_without_credentials_or_effects() {
    // This directory deliberately lacks the configured relative CUDA artifacts.
    let output = Command::new(env!("CARGO_BIN_EXE_hyper-gpu-support"))
        .arg("validate")
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/config"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let report: hyper_gpu_support::validation::ValidationReport =
        serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report.checks[0].status,
        hyper_gpu_support::validation::CheckStatus::Blocked
    );
    assert!(
        report.checks[1..]
            .iter()
            .all(|c| c.status == hyper_gpu_support::validation::CheckStatus::Untested)
    );
}

#[test]
fn staging_inspector_rejects_arguments_before_reading_machine_state() {
    let output = Command::new(env!("CARGO_BIN_EXE_hyper-gpu-stage"))
        .arg("unexpected")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap().trim(),
        concat!(
            "staging manifest error: usage: hyper-gpu-stage [inspect]\n",
            "       hyper-gpu-stage apply USER\n",
            "       hyper-gpu-stage apply --interactive"
        )
    );
}

#[test]
fn guest_copy_rejects_argument_count_before_reading_credentials_or_machine_state() {
    let output = Command::new(env!("CARGO_BIN_EXE_hyper-gpu-guest-copy"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap().trim(),
        concat!(
            "guest copy error: usage: hyper-gpu-guest-copy USER SOURCE DESTINATION_FILENAME SHA256\n",
            "       hyper-gpu-guest-copy --interactive SOURCE DESTINATION_FILENAME SHA256"
        )
    );
}
