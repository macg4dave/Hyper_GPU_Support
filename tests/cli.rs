//! Public executable contracts, requiring no hardware or administrator rights.

use std::process::Command;

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
        assert!(stdout.contains("Only inventory is implemented"));
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
        "plan", "apply", "status", "validate", "remove", "recover", "start", "shutdown", "restart",
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
