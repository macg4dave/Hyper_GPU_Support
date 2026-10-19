use super::*;

#[test]
fn initial_setup_controls_require_completed_receipts_for_the_exact_pair() {
    let mut s = state();
    let vm_id = id(&s).to_owned();
    assert!(s.initial_setup_required(&vm_id));
    let target = s.target(&vm_id).unwrap();
    // A provider attachment and enrollment cannot unlock first-time editing.
    s.view.inventory.as_mut().unwrap().discovery.vms[0].gpus = vec![target.gpu_interface.clone()];
    assert!(s.initial_setup_required(&vm_id));
    let mut journal =
        review_plan()
            .managed
            .unwrap_or_else(|| hyper_gpu_support::workflow::Journal {
                schema: 1,
                vm_id: vm_id.clone(),
                gpu_interface: target.gpu_interface.clone(),
                original: s.view.vm(&vm_id).unwrap().settings.clone(),
                applied: s.view.vm(&vm_id).unwrap().settings.for_gpu(),
                prepared: Some("checked-payload".into()),
                pending: false,
                restore_power: Power::Off,
                verification_only: false,
                last_verified: Some(1),
                latest_check: Some(hyper_gpu_support::workflow::GuestCheck::Passed),
                preparation_started: Some(true),
                check_restored: false,
                settings_released: false,
            });
    for complete in [false, true] {
        journal.last_verified = complete.then_some(1);
        s.view
            .inventory
            .as_mut()
            .unwrap()
            .managed
            .insert(vm_id.clone(), Some(journal.clone()));
        assert_eq!(s.initial_setup_required(&vm_id), !complete);
    }
    // Disabling a previously set-up VM leaves ordinary re-enablement available.
    s.view.inventory.as_mut().unwrap().discovery.vms[0]
        .gpus
        .clear();
    assert!(!s.initial_setup_required(&vm_id));
    journal.gpu_interface = "different-gpu".into();
    s.view
        .inventory
        .as_mut()
        .unwrap()
        .managed
        .insert(vm_id.clone(), Some(journal));
    assert!(s.initial_setup_required(&vm_id));
}

#[test]
fn permission_proposals_use_the_observed_revision_and_reject_duplicate_stale_or_held_targets() {
    use hyper_gpu_support::{
        configuration_store::Revision,
        observation::{Mode, Provenance},
    };
    let mut s = state();
    let target = s.target(id(&s)).unwrap();
    s.observation = Some(Provenance {
        schema: 1,
        authority: Revision::of(b"observed authority"),
        configuration: Revision::Missing,
        mode: Mode::Full,
        selected: None,
    });
    assert!(
        s.permission_review(&target, "operator".into()).is_err(),
        "already enrolled"
    );
    s.view.inventory.as_mut().unwrap().enrolled.clear();
    let review = s
        .permission_review(&target, "operator".into())
        .unwrap()
        .unwrap();
    assert_eq!(review.expected, Revision::of(b"observed authority"));
    assert_eq!(review.vm_id, target.vm_id);
    assert_eq!(review.gpu_interface, target.gpu_interface);
    s.view.needs_readback = true;
    assert!(s.permission_review(&target, "operator".into()).is_err());
    s.view.needs_readback = false;
    s.recoveries = vec![save_record(&s)];
    assert!(s.permission_review(&target, "operator".into()).is_err());
    s.recoveries.clear();
    s.observation = None;
    assert!(
        s.permission_review(&target, "operator".into())
            .unwrap()
            .is_none(),
        "first install retains artifact preparation"
    );
    s.view.needs_readback = true;
    assert!(s.permission_review(&target, "operator".into()).is_err());
    s.view.needs_readback = false;
    s.recoveries = vec![save_record(&s)];
    assert!(s.permission_review(&target, "operator".into()).is_err());
}

#[test]
fn setup_affordance_follows_live_attachment_even_with_saved_enabled_intent_and_driver_history() {
    slint::platform::set_platform(Box::new(crate::gui::catalogue_tests::Headless)).unwrap();
    let ui = crate::gui::AppWindow::new().unwrap();
    let mut s = state();
    let selected = id(&s).to_owned();
    let target = s.target(&selected).unwrap();
    s.load_committed(committed(vec![target.clone()], "retained enabled intent"))
        .unwrap();
    let settings = s.view.vm(&selected).unwrap().settings.clone();
    let history = hyper_gpu_support::workflow::Journal {
        schema: 1,
        vm_id: selected.clone(),
        gpu_interface: target.gpu_interface.clone(),
        original: settings.clone(),
        applied: settings.for_gpu(),
        prepared: Some("historical-payload".into()),
        pending: false,
        restore_power: Power::Off,
        verification_only: false,
        last_verified: Some(1),
        latest_check: Some(hyper_gpu_support::workflow::GuestCheck::Passed),
        preparation_started: Some(true),
        check_restored: false,
        settings_released: true,
    };
    assert!(history.setup_eligible());
    s.view
        .inventory
        .as_mut()
        .unwrap()
        .managed
        .insert(selected.clone(), Some(history));
    crate::gui::live::presentation::present(&ui, &s, true);
    assert!(
        ui.get_setup_required(),
        "an enrolled but unattached VM still needs GPU support"
    );
    assert!(ui.get_support_summary().contains("isn't enabled"));
    s.view.inventory.as_mut().unwrap().discovery.vms[0].gpus = vec![target.gpu_interface];
    crate::gui::live::presentation::present(&ui, &s, true);
    assert!(!ui.get_setup_required());
    s.view.inventory.as_mut().unwrap().discovery.vms[0]
        .gpus
        .clear();
    crate::gui::live::presentation::present(&ui, &s, true);
    assert!(
        ui.get_setup_required(),
        "removal must restore the setup affordance"
    );
    s.view.needs_readback = true;
    crate::gui::live::presentation::present(&ui, &s, false);
    assert!(
        !ui.get_setup_required(),
        "historical rows cannot establish removal"
    );
}

#[test]
fn reopened_selection_survives_discovery_order_and_rename_with_or_without_recovery() {
    for held in [false, true] {
        let original = state();
        let selected = original.view.selected.clone().unwrap();
        let record = save_record(&original);
        let mut next = inventory();
        next.discovery.vms[0].name = "Renamed selected VM".into();
        let mut first = next.discovery.vms[0].clone();
        first.vm_id = "87654321-4321-4321-4321-cba987654321".into();
        first.name = "First listed VM".into();
        next.discovery.vms.insert(0, first);

        // Startup restores the preference before accepting first discovery.
        let mut reopened = State::default();
        reopened.view.selected = Some(selected.clone());
        reopened.recoveries = held.then_some(record).into_iter().collect();
        let recovery_before = serde_json::to_value(&reopened.recoveries).unwrap();
        let inventory_before = serde_json::to_value(&next).unwrap();
        reopened.refresh(next.clone());
        assert_eq!(reopened.view.selected.as_deref(), Some(selected.as_str()));
        assert_eq!(
            reopened.view.vm(&selected).unwrap().name,
            "Renamed selected VM"
        );
        assert_eq!(
            serde_json::to_value(&reopened.recoveries).unwrap(),
            recovery_before
        );
        assert_eq!(
            serde_json::to_value(reopened.view.inventory.as_ref().unwrap()).unwrap(),
            inventory_before
        );
        assert!(reopened.view.draft.is_none());
        if held {
            assert!(reopened.editor_eligibility(&selected).is_err());
            assert!(
                crate::gui::live::presentation::recovery_summary(&reopened, &selected).is_some()
            );
        }

        next.discovery.vms.reverse();
        reopened.refresh(next.clone());
        assert_eq!(reopened.view.selected.as_deref(), Some(selected.as_str()));
        next.discovery.vms.retain(|vm| vm.vm_id != selected);
        reopened.refresh(next.clone());
        assert_eq!(
            reopened.view.selected.as_deref(),
            Some(next.discovery.vms[0].vm_id.as_str())
        );
        assert_eq!(
            serde_json::to_value(&reopened.recoveries).unwrap(),
            recovery_before
        );
        next.discovery.vms.clear();
        reopened.refresh(next);
        assert!(reopened.view.selected.is_none());
        assert_eq!(
            serde_json::to_value(&reopened.recoveries).unwrap(),
            recovery_before
        );
    }
}

#[test]
fn first_install_inventory_stays_current_until_explicit_refresh() {
    let mut s = state();
    assert!(s.can_poll());
    let mut first_install = inventory();
    first_install.enrolled.clear();
    s.refresh(first_install);
    assert!(!s.can_poll());
    assert!(!s.view.needs_readback);
    assert!(s.editor_eligibility(id(&s)).is_ok());
    s.refresh(inventory());
    assert!(s.can_poll());
}
