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
