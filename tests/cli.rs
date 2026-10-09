//! Hardware-free public process contracts.
use std::process::Command;
#[test]
fn help_does_not_load_laboratory_configuration() {
    let result = Command::new(env!("CARGO_BIN_EXE_hyper-gpu-support"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(result.status.success());
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.contains("--config FILE"));
    assert!(text.contains("--mock-gui"));
    assert!(!text.contains("declared but not implemented"));
    assert!(!text.contains("reset-slot"));
}
#[test]
fn runner_and_guest_refuse_arbitrary_modes_before_any_effect() {
    for executable in [
        env!("CARGO_BIN_EXE_hyper-gpu-runner"),
        env!("CARGO_BIN_EXE_hyper-gpu-guest"),
    ] {
        let result = Command::new(executable)
            .arg("arbitrary-command")
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(2));
    }
}
#[test]
fn unknown_commands_are_errors() {
    assert!(
        !Command::new(env!("CARGO_BIN_EXE_hyper-gpu-support"))
            .arg("reset-slot")
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn explicit_cli_dispatch_preserves_output_and_validation_without_gui() {
    for flag in ["--version", "-V"] {
        let result = Command::new(env!("CARGO_BIN_EXE_hyper-gpu-support"))
            .arg(flag)
            .env("SLINT_BACKEND", "invalid-backend-for-cli-test")
            .output()
            .unwrap();
        assert!(result.status.success());
        assert!(
            String::from_utf8(result.stdout)
                .unwrap()
                .starts_with("hyper-gpu-support ")
        );
    }
    let help = Command::new(env!("CARGO_BIN_EXE_hyper-gpu-support"))
        .arg("--help")
        .env("SLINT_BACKEND", "invalid-backend-for-cli-test")
        .output()
        .unwrap();
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    for command in [
        "inventory",
        "plan",
        "install",
        "apply",
        "enable",
        "disable",
        "status",
        "verify",
        "credentials",
        "forget",
    ] {
        assert!(help.contains(command));
    }
    for args in [
        vec!["status"],
        vec!["--config", "missing.toml"],
        vec!["--internal-worker", "arbitrary-command"],
        vec!["--mock-gui", "--config", "missing.toml"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_hyper-gpu-support"))
            .args(args)
            .env("SLINT_BACKEND", "invalid-backend-for-cli-test")
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(!result.stderr.is_empty());
    }
}
