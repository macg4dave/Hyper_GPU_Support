use super::*;

fn comparison_state(enabled: bool) -> (State, String, Target) {
    let mut s = state();
    let vm_id = id(&s).to_owned();
    let mut saved = s.target(&vm_id).unwrap();
    saved.enabled = enabled;
    saved.vram = Some(Allocation {
        minimum: 10,
        optimal: 50,
        maximum: 100,
    });
    s.load_committed(committed(vec![saved.clone()], "comparison"))
        .unwrap();
    (s, vm_id, saved)
}

#[test]
fn dashboard_problem_requires_fresh_difference_not_missing_or_old_guest_results() {
    let (mut s, vm_id, saved) = comparison_state(true);
    let problem = |s: &State| crate::gui::host_configuration::has_current_problem(s, &vm_id);
    assert!(
        problem(&s),
        "fresh missing expected attachment is a problem"
    );
    s.view.needs_readback = true;
    assert!(!problem(&s), "retained inventory is not a current failure");
    s.view.needs_readback = false;
    let inventory = s.view.inventory.as_mut().unwrap();
    inventory.discovery.vms[0].gpus = vec![saved.gpu_interface.clone()];
    inventory.discovery.vms[0].vram = saved.vram.clone();
    assert!(
        !problem(&s),
        "absent guest history is not a detected problem"
    );
    let vm = &s.view.inventory.as_ref().unwrap().discovery.vms[0];
    let record = hyper_gpu_support::workflow::Journal {
        schema: 1,
        vm_id: vm_id.clone(),
        gpu_interface: saved.gpu_interface,
        original: vm.settings.clone(),
        applied: vm.settings.clone(),
        prepared: Some("old".into()),
        pending: false,
        restore_power: Power::Off,
        verification_only: false,
        last_verified: Some(1),
        latest_check: Some(hyper_gpu_support::workflow::GuestCheck::Failed),
        preparation_started: None,
        check_restored: false,
        settings_released: false,
    };
    s.view
        .inventory
        .as_mut()
        .unwrap()
        .managed
        .insert(vm_id.clone(), Some(record));
    assert!(!problem(&s), "old failed checks remain history");
    s.view.inventory.as_mut().unwrap().discovery.vms[0]
        .vram
        .as_mut()
        .unwrap()
        .optimal += 1;
    assert!(problem(&s), "fresh invalid allocation is a problem");
}

#[test]
fn host_comparison_uses_saved_intent_and_preserves_missing_attachment_repair() {
    let (mut s, vm_id, saved) = comparison_state(true);
    let before = s.committed.documents[&vm_id].source.clone();
    s.view.draft = Some(Target {
        enabled: false,
        ..saved.clone()
    });
    let text = s.host_configuration_text(&vm_id);
    assert!(text.contains(&format!("expected {}; observed none", saved.gpu_interface)));
    assert!(text.contains("observed Unknown"));
    assert!(s.editor_eligibility(&vm_id).is_ok());
    assert!(s.view.eligibility(&saved).is_ok());
    assert_eq!(s.committed.documents[&vm_id].source, before);
    assert!(!s.view.draft.as_ref().unwrap().enabled);
    assert!(s.view.vm(&vm_id).unwrap().gpus.is_empty());
    let inventory = s.view.inventory.as_mut().unwrap();
    inventory.discovery.vms[0].gpus = vec!["unrelated-gpu".into()];
    assert!(
        s.host_configuration_text(&vm_id)
            .contains("observed unrelated-gpu")
    );
    assert!(s.view.eligibility(&saved).is_err());
}

#[test]
fn host_comparison_reports_exact_resources_and_managed_settings() {
    let (mut s, vm_id, saved) = comparison_state(true);
    let inventory = s.view.inventory.as_mut().unwrap();
    let vm = &mut inventory.discovery.vms[0];
    vm.gpus = vec![saved.gpu_interface.clone()];
    vm.vram = saved.vram.clone();
    let original = vm.settings.clone();
    vm.settings = original.for_gpu();
    inventory.managed.insert(
        vm_id.clone(),
        Some(hyper_gpu_support::workflow::Journal {
            schema: 1,
            vm_id: vm_id.clone(),
            gpu_interface: saved.gpu_interface.clone(),
            original,
            applied: vm.settings.clone(),
            prepared: Some("digest".into()),
            pending: false,
            restore_power: Power::Off,
            verification_only: false,
            last_verified: None,
            latest_check: None,
            preparation_started: None,
            check_restored: false,
            settings_released: false,
        }),
    );
    let text = s.host_configuration_text(&vm_id);
    assert!(text.contains("compared values match"));
    assert!(text.contains("Compute has no explicit saved request"));
    let vm = &mut s.view.inventory.as_mut().unwrap().discovery.vms[0];
    vm.vram.as_mut().unwrap().optimal = 60;
    vm.settings.low_mmio = 1;
    vm.settings.high_mmio = 2;
    vm.settings.cache_types = false;
    vm.settings.automatic_checkpoints = true;
    let text = s.host_configuration_text(&vm_id);
    assert!(text.contains("VRAM: expected 10 / 50 / 100; observed 10 / 60 / 100"));
    for name in [
        "Low MMIO",
        "High MMIO",
        "Guest cache types",
        "Automatic checkpoints",
    ] {
        assert!(text.contains(name), "{text}");
    }
    // A confirmed guest rollback mismatch invalidates payload eligibility,
    // not the independently recorded host settings baseline.
    let journal = s
        .view
        .inventory
        .as_mut()
        .unwrap()
        .managed
        .get_mut(&vm_id)
        .unwrap()
        .as_mut()
        .unwrap();
    journal.prepared = None;
    journal.latest_check = Some(hyper_gpu_support::workflow::GuestCheck::PayloadMismatch);
    assert!(s.host_configuration_text(&vm_id).contains("Low MMIO"));
    s.view
        .inventory
        .as_mut()
        .unwrap()
        .managed
        .get_mut(&vm_id)
        .unwrap()
        .as_mut()
        .unwrap()
        .pending = true;
    let text = s.host_configuration_text(&vm_id);
    assert!(text.contains("baseline is unavailable"));
    assert!(!text.contains("Low MMIO"));
}

#[test]
fn host_comparison_handles_disabled_missing_malformed_and_stale_reads() {
    let (mut s, vm_id, _) = comparison_state(false);
    assert_eq!(
        s.host_configuration_text(&vm_id),
        "Host configuration: compared values match the saved intent."
    );
    s.view.needs_readback = true;
    assert!(s.host_configuration_text(&vm_id).contains("historical"));
    s.refresh(inventory());
    assert!(
        s.host_configuration_text(&vm_id)
            .contains("compared values match")
    );
    s.view.inventory.as_mut().unwrap().discovery.vms.clear();
    assert!(s.host_configuration_text(&vm_id).contains("Unknown"));
    s.view.inventory = None;
    assert!(
        s.host_configuration_text(&vm_id)
            .contains("inventory is unavailable")
    );
    s.load_committed(Default::default()).unwrap();
    assert!(
        s.host_configuration_text(&vm_id)
            .contains("no saved intent")
    );
    let mut store = hyper_gpu_support::configuration_store::StoreSnapshot::default();
    store.insert(&format!("{vm_id}.toml"), Ok("malformed".into()));
    s.load_committed(store).unwrap();
    assert!(
        s.host_configuration_text(&vm_id)
            .contains("saved intent is unreadable")
    );
}

#[test]
fn host_comparison_isolates_incomplete_observations_and_requires_fresh_review() {
    let (mut s, vm_id, saved) = comparison_state(true);
    let diagnostic = hyper_gpu_support::reporting::Diagnostic::observation;
    let mut fresh = inventory();
    fresh.discovery.issues.push(diagnostic(
        "VM settings",
        Some("other-vm".into()),
        "access denied",
    ));
    s.refresh(fresh.clone());
    assert!(
        s.host_configuration_text(&vm_id)
            .contains("GPU attachment: expected")
    );
    fresh.discovery.issues.push(diagnostic(
        "VM settings",
        Some(vm_id.clone()),
        "access denied",
    ));
    s.refresh(fresh);
    assert!(
        s.host_configuration_text(&vm_id)
            .contains("observations are incomplete")
    );
    let text = s.host_configuration_text(&vm_id);
    assert!(!text.contains("configuration differences"));
    s.refresh(inventory());
    assert!(
        s.host_configuration_text(&vm_id)
            .contains("configuration differences")
    );
    s.view.needs_readback = true;
    assert!(s.view.eligibility(&saved).is_err());
    assert!(s.reviewed_plan(&saved).is_err());
    assert_eq!(s.committed.documents[&vm_id].target, saved);
}
