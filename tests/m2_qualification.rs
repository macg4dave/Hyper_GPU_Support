//! Explicit affected-path qualification on the configured disposable VM only.
#![cfg(windows)]
use hyper_gpu_support::{
    configuration_store, credentials,
    model::{Power, Target},
    payload, process,
    runner::{self, Operation},
    windows_driver, windows_hyperv, worker, workflow,
};
use std::{fs, path::PathBuf, process::Command, time::Duration};

fn evidence(name: &str, value: &impl serde::Serialize) {
    let root = PathBuf::from(
        std::env::var_os("HYPER_GPU_SPRINT_OUTPUT")
            .expect("qualification evidence directory required"),
    );
    assert!(root.starts_with(std::env::current_dir().unwrap().join("local")));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn fixture(
    mode: &str,
    lab: &toml::Value,
    credential: Option<&credentials::Credential>,
    destination: Option<&str>,
    digest: Option<&str>,
) -> serde_json::Value {
    let request = zeroize::Zeroizing::new(serde_json::to_vec(&serde_json::json!({
        "mode":mode, "vm_id":lab["slot"]["vm_id"].as_str().unwrap(),
        "child":lab["slot"]["child_path"].as_str().unwrap(), "parent":lab["slot"]["parent_path"].as_str().unwrap(),
        "gpu":lab["slot"]["gpu_interface"].as_str().unwrap(), "credential":credential, "destination":destination, "digest":digest,
    })).unwrap());
    let mut command =
        Command::new(hyper_gpu_support::windows_paths::windows_powershell_executable().unwrap());
    command.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-File",
        "tests/fixtures/m2-environment.ps1",
    ]);
    let result = process::bounded_process_with_limit(
        command,
        Duration::from_secs(240),
        1024 * 1024,
        &request,
    )
    .unwrap_or_else(|_| panic!("disposable fixture supervision failed; inspect preimages"));
    assert_eq!(
        result.exit_code,
        Some(0),
        "designated fixture failed; inspect local preimages, do not replay GPU operations"
    );
    serde_json::from_str(result.stdout.trim()).unwrap()
}
fn approved(target: &Target) -> workflow::Plan {
    serde_json::from_value(
        runner::submit(runner::request(Operation::Plan, Some(target.clone()), None)).unwrap(),
    )
    .unwrap()
}
fn apply(target: &Target) -> worker::Outcome {
    let plan = approved(target);
    let credential = if plan.preview.credentials_required {
        Some(
            credentials::read(&target.vm_id)
                .unwrap()
                .expect("local vault credential required; use the product credential prompt"),
        )
    } else {
        None
    };
    let lab: toml::Value =
        toml::from_str(&fs::read_to_string("config/project.toml").unwrap()).unwrap();
    fixture("snapshot", &lab, None, None, None);
    let result = worker::apply(
        plan,
        configuration_store::read_committed()
            .revision(&target.vm_id)
            .unwrap(),
        true,
        credential,
        |stage| eprintln!("{:?}: {}", stage.status, stage.stage),
    )
    .unwrap();
    assert!(result.finished && result.saved);
    result
}

#[test]
#[ignore = "explicit one-shot Reapply; requires durable tracing, elevated token and configured disposable target"]
fn reapply_with_durable_diagnostics() {
    assert!(process::is_elevated().unwrap());
    let install = runner::install_directory().unwrap();
    assert!(install.join("diagnostics.enabled").is_file());
    let lab: toml::Value =
        toml::from_str(&fs::read_to_string("config/project.toml").unwrap()).unwrap();
    let target = Target {
        vm_id: lab["slot"]["vm_id"].as_str().unwrap().into(),
        gpu_interface: lab["slot"]["gpu_interface"].as_str().unwrap().into(),
        enabled: true,
        vram: None,
    };
    target.validate().unwrap();
    evidence("before.json", &fixture("snapshot", &lab, None, None, None));
    let initial = windows_hyperv::inspect(&target.vm_id).unwrap();
    assert_eq!(
        initial.power,
        Power::Off,
        "bounded Reapply requires initial Off"
    );
    assert!(runner::recovery_record().unwrap().is_none());
    assert!(runner::import_record().unwrap().is_none());
    let journal: workflow::Journal = serde_json::from_slice(
        &fs::read(
            runner::data_directory()
                .unwrap()
                .join(format!("{}.json", target.vm_id)),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        !journal.pending && journal.prepared.is_some(),
        "Reapply requires existing prepared receipt"
    );
    let result = apply(&target);
    evidence("reapply.json", &result);
    let operation = result.operation.as_ref().unwrap();
    assert!(!operation.prepared, "existing payload must remain current");
    assert!(operation.verified);
    assert_eq!(operation.effective.power, initial.power);
    // Stop here. No additional workload, disable, investigation or retry.
}

#[test]
#[ignore = "requires elevated token, installed reviewed current artifacts, disposable identity checks and local vault credential"]
fn fresh_preparation_and_installed_worker_contract() {
    assert!(
        process::is_elevated().unwrap(),
        "launch this selected test through bounded elevation"
    );
    let lab: toml::Value =
        toml::from_str(&fs::read_to_string("config/project.toml").unwrap()).unwrap();
    let target = Target {
        vm_id: lab["slot"]["vm_id"].as_str().unwrap().into(),
        gpu_interface: lab["slot"]["gpu_interface"].as_str().unwrap().into(),
        enabled: true,
        vram: None,
    };
    target.validate().unwrap();
    let enrollment: hyper_gpu_support::gui_model::Inventory = serde_json::from_value(
        runner::submit(runner::request(Operation::Discover, None, None)).unwrap(),
    )
    .unwrap();
    assert!(
        enrollment
            .enrolled
            .iter()
            .any(|t| t.vm_id == target.vm_id && t.gpu_interface == target.gpu_interface)
    );
    worker::available().unwrap();
    let credential = credentials::read(&target.vm_id)
        .unwrap()
        .expect("local vault credential required; use the product credential prompt");
    let before = fixture("snapshot", &lab, None, None, None);
    evidence("before.json", &before);
    let initial = windows_hyperv::inspect(&target.vm_id).unwrap();
    assert_eq!(
        initial.power,
        Power::Off,
        "this bounded fixture requires the disposable VM Off"
    );
    assert!(
        initial.gpus.is_empty(),
        "fixture requires no GPU assignment"
    );
    assert!(runner::recovery_record().unwrap().is_none());
    assert!(runner::import_record().unwrap().is_none());
    // Exercise a real installed worker exchange without an admitted effect.
    let no_record = worker::save_only("00000000000000000000000000000000".into(), |_| {})
        .expect_err("no save receipt exists");
    assert!(
        no_record.contains("no durable")
            || no_record.contains("no recorded")
            || no_record.contains("no verified"),
        "unexpected worker exchange: {no_record}"
    );
    evidence(
        "worker-no-effect.json",
        &serde_json::json!({"installed_exchange":true,"rejected_without_receipt":true}),
    );
    let discovery =
        windows_driver::discover_driver_environment(&target, Duration::from_secs(300)).unwrap();
    let manifest = payload::discover(
        &hyper_gpu_support::windows_paths::windows_directory().unwrap(),
        discovery,
    )
    .unwrap();
    hyper_gpu_support::guest::validate_trust(&manifest).unwrap();
    let digest = manifest.digest().unwrap();
    let file = manifest
        .files
        .iter()
        .filter(|file| {
            file.destination
                .to_ascii_lowercase()
                .starts_with("system32\\hostdriverstore\\")
                && file.destination.to_ascii_lowercase().ends_with(".dll")
        })
        .max_by_key(|file| file.bytes)
        .expect("dynamic signed HostDriverStore DLL");
    let journal_path = runner::data_directory()
        .unwrap()
        .join(format!("{}.json", target.vm_id));
    let mut journal: workflow::Journal =
        serde_json::from_slice(&fs::read(&journal_path).unwrap()).unwrap();
    assert_eq!(journal.vm_id, target.vm_id);
    assert_eq!(journal.gpu_interface, target.gpu_interface);
    assert!(!journal.pending);
    evidence("journal-preimage.json", &journal);
    // Resume is exclusively for a separately reconciled contributor-wrapper
    // interruption. It cannot clear product recovery or silently restage a file.
    let staged = if std::env::var("HYPER_GPU_SPRINT_RESUME").as_deref() == Ok("1") {
        let root = PathBuf::from(std::env::var_os("HYPER_GPU_SPRINT_OUTPUT").unwrap());
        let staged: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("fresh-write-preimage.json")).unwrap())
                .unwrap();
        assert_eq!(staged["destination"], file.destination);
        assert_eq!(staged["sha256"], file.sha256);
        staged
    } else {
        // Only this verified disposable record is changed. No production force
        // option or manufactured host-driver version is introduced.
        fixture("snapshot", &lab, None, None, None);
        windows_hyperv::power(&target, Power::Running).unwrap();
        let staged = fixture(
            "stage",
            &lab,
            Some(&credential),
            Some(&file.destination),
            Some(&digest),
        );
        windows_hyperv::power(&target, Power::Off).unwrap();
        staged
    };
    evidence("fresh-write-preimage.json", &staged);
    journal.prepared = None;
    fs::write(&journal_path, serde_json::to_vec_pretty(&journal).unwrap()).unwrap();
    let fresh = apply(&target);
    assert!(fresh.operation.as_ref().unwrap().prepared);
    assert!(fresh.operation.as_ref().unwrap().verified);
    assert_eq!(
        fresh.operation.as_ref().unwrap().effective.power,
        Power::Off
    );
    evidence("fresh-preparation.json", &fresh);
    let reapply = apply(&target);
    assert!(!reapply.operation.as_ref().unwrap().prepared);
    evidence("reapply.json", &reapply);
    // Independent transfer/write hash and guest OS readback after actual preparation.
    windows_hyperv::power(&target, Power::Running).unwrap();
    evidence(
        "guest-readback.json",
        &fixture(
            "readback",
            &lab,
            Some(&credential),
            Some(&file.destination),
            Some(&digest),
        ),
    );
    windows_hyperv::power(&target, Power::Off).unwrap();
    let mut disabled = target.clone();
    disabled.enabled = false;
    let result = apply(&disabled);
    evidence("disable.json", &result);
    let after = fixture("snapshot", &lab, None, None, None);
    evidence("after.json", &after);
    for field in [
        "vm_id",
        "generation",
        "state",
        "disk",
        "parent",
        "processors",
        "memory",
        "dynamic_memory",
        "secure_boot",
        "tpm",
        "gpu",
    ] {
        assert_eq!(
            before[field], after[field],
            "preservation mismatch: {field}"
        );
    }
    let status = runner::submit(runner::request(Operation::Status, Some(disabled), None)).unwrap();
    assert_eq!(status["managed"]["pending"], false);
    assert!(runner::recovery_record().unwrap().is_none());
    evidence("final-status.json", &status);
}
