//! Explicit installed-runner qualification; never enabled by the normal test suite.
#![cfg(windows)]

use hyper_gpu_support::{
    model::{Configuration, Discovery},
    process,
    runner::{self, Operation},
};

#[test]
#[ignore = "requires reviewed installed product runner and explicit designated-target configuration"]
fn installed_existing_vm_enrollment_contract() {
    assert!(
        !process::is_elevated().unwrap(),
        "run with the ordinary operator token"
    );
    let path = std::env::var_os("HYPER_GPU_M1_CONFIG")
        .expect("set HYPER_GPU_M1_CONFIG to the freshly qualified designated-target TOML");
    let configuration = Configuration::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        configuration.targets.len(),
        1,
        "qualification selects one designated VM"
    );
    let target = configuration.targets[0].clone();

    let inventory = runner::submit(runner::request(Operation::Discover, None, None)).unwrap();
    let discovery: Discovery = serde_json::from_value(inventory).unwrap();
    assert!(
        discovery
            .vms
            .iter()
            .any(|vm| vm.vm_id == target.vm_id && vm.generation == 2)
    );
    assert!(
        discovery
            .gpus
            .iter()
            .any(|gpu| gpu.interface == target.gpu_interface)
    );
    let preview =
        runner::submit(runner::request(Operation::Plan, Some(target.clone()), None)).unwrap();
    assert_eq!(preview["desired"]["vm_id"], target.vm_id);
    assert_eq!(preview["observed"]["generation"], 2);
    if target.enabled {
        let gpu = discovery
            .gpus
            .iter()
            .find(|gpu| gpu.interface == target.gpu_interface)
            .unwrap();
        assert_eq!(preview["preparation"]["current_driver"], gpu.driver_version);
        assert!(
            !preview["preparation"]["digest"]
                .as_str()
                .unwrap()
                .is_empty()
        );
    }

    let request = runner::request(Operation::Status, Some(target.clone()), None);
    let nonce = request.nonce.clone();
    let status = runner::submit(request).unwrap();
    assert_eq!(status["observed"]["vm_id"], target.vm_id);
    let audit = runner::data_directory().unwrap().join("audit");
    let record: serde_json::Value =
        serde_json::from_slice(&std::fs::read(audit.join(format!("{nonce}.json"))).unwrap())
            .unwrap();
    assert_eq!(record["nonce"], nonce);
    assert_eq!(record["operation"], "Status");
    assert_eq!(record["outcome"], "Succeeded");
    assert!(record.get("credential").is_none());
    let mut replay = runner::request(Operation::Status, Some(target.clone()), None);
    replay.nonce = nonce;
    assert!(
        runner::submit(replay)
            .unwrap_err()
            .contains("replayed product request")
    );

    let mut wrong_schema = runner::request(Operation::Status, Some(target.clone()), None);
    wrong_schema.schema = 1;
    assert!(
        runner::submit(wrong_schema)
            .unwrap_err()
            .contains("invalid protocol identity")
    );
    let mut bad_nonce = runner::request(Operation::Status, Some(target.clone()), None);
    bad_nonce.nonce = "../arbitrary-path".into();
    assert!(
        runner::submit(bad_nonce)
            .unwrap_err()
            .contains("invalid protocol identity")
    );
    assert!(
        runner::submit(runner::request(Operation::Status, None, None))
            .unwrap_err()
            .contains("operation requires an enrolled target")
    );

    let mut outside = target.clone();
    outside.vm_id = if target.vm_id == "11111111-1111-1111-1111-111111111111" {
        "22222222-2222-2222-2222-222222222222"
    } else {
        "11111111-1111-1111-1111-111111111111"
    }
    .into();
    let refusal = runner::request(Operation::Status, Some(outside), None);
    let refusal_nonce = refusal.nonce.clone();
    assert!(
        runner::submit(refusal)
            .unwrap_err()
            .contains("outside administrator enrollment")
    );
    let record: serde_json::Value = serde_json::from_slice(
        &std::fs::read(audit.join(format!("{refusal_nonce}.json"))).unwrap(),
    )
    .unwrap();
    assert_eq!(record["outcome"], "Failed");
    let mut outside = target;
    outside.gpu_interface = r"\\?\PCI#VEN_0000&DEV_0000#not-enrolled\GPUPARAV".into();
    assert!(
        runner::submit(runner::request(Operation::Status, Some(outside), None))
            .unwrap_err()
            .contains("outside administrator enrollment")
    );

    // Intentional refusal test after the privilege requirement was established.
    assert!(
        runner::install(&configuration)
            .unwrap_err()
            .contains("elevated administrator")
    );
    for path in [
        runner::data_directory().unwrap().join("enrollment.json"),
        runner::install_directory()
            .unwrap()
            .join("hyper-gpu-runner.exe"),
    ] {
        let error = std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .expect_err(
                "ordinary operator must not have write access to protected enrollment/artifacts",
            );
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    }
}
