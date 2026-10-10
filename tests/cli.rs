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

#[cfg(windows)]
#[test]
fn gui_packaging_and_restricted_worker_rejection() {
    let exe = env!("CARGO_BIN_EXE_hyper-gpu-support");
    let bytes = std::fs::read(exe).unwrap();
    let pe = u32::from_le_bytes(bytes[0x3c..0x40].try_into().unwrap()) as usize;
    let subsystem = u16::from_le_bytes(bytes[pe + 24 + 68..pe + 24 + 70].try_into().unwrap());
    assert_eq!(subsystem, 2, "GUI launch must not allocate a console");
    for args in [
        vec!["--internal-worker", "not-a-session", "1"],
        vec!["--internal-worker", "0123456789abcdef0123456789abcdef", "0"],
    ] {
        let result = Command::new(exe).args(args).output().unwrap();
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
    }
}

#[test]
fn parser_errors_do_not_echo_secret_input() {
    let path = std::env::current_dir()
        .unwrap()
        .join("local/cli-secret-redaction.toml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "password = 'private-parser-secret'\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_hyper-gpu-support"))
        .args(["status", "--config"])
        .arg(&path)
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("private-parser-secret"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("private-parser-secret"));
}
