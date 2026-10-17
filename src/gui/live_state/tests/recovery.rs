use super::*;

#[test]
fn recovery_presentation_and_editor_admission_are_vm_scoped() {
    use crate::gui::{AppWindow, catalogue_tests::Headless, live::presentation};
    use hyper_gpu_support::configuration_store::SavePhase;
    slint::platform::set_platform(Box::new(Headless)).unwrap();
    let ui = AppWindow::new().unwrap();
    let mut s = state();
    let affected = id(&s).to_owned();
    let mut other = s.view.vm(&affected).unwrap().clone();
    other.vm_id = "87654321-4321-4321-4321-cba987654321".into();
    // Equal names must not cause another VM to inherit the recovery state.
    s.view
        .inventory
        .as_mut()
        .unwrap()
        .discovery
        .vms
        .push(other.clone());
    s.view.selected = Some(other.vm_id.clone());
    presentation::present(&ui, &s, true);
    assert!(!ui.get_selected_recovery());
    assert!(ui.get_vm_details().is_empty());
    let normal_summary = ui.get_support_summary();

    s.view.selected = Some(affected.clone());
    let mut record = save_record(&s);
    record.phase = SavePhase::EffectsStarted;
    s.recovery = Some(record);
    let before = serde_json::to_value(&s.recovery).unwrap();
    for selected in [&affected, &other.vm_id, &affected, &other.vm_id] {
        s.view.selected = Some(selected.clone());
        presentation::present(&ui, &s, true);
        assert_eq!(ui.get_recovery(), selected == &affected);
        assert_eq!(ui.get_action_recovery(), selected == &affected);
        assert!(ui.get_has_recovery());
        assert_eq!(
            s.editor_eligibility(selected).is_err(),
            selected == &affected
        );
        assert_eq!(ui.get_selected_recovery(), selected == &affected);
        assert_eq!(ui.get_vm_details().is_empty(), selected != &affected);
        if selected == &other.vm_id {
            assert_eq!(ui.get_support_summary(), normal_summary);
            assert!(ui.get_recovery_summary().contains(&affected));
        }
        let actions = s.selected_actions(-1, &[]);
        if selected == &affected {
            assert!(!actions.verify_allowed && !actions.repair_allowed);
            assert!(!actions.remove_allowed);
            assert!(actions.forget_allowed);
        }
    }
    s.view
        .inventory
        .as_mut()
        .unwrap()
        .discovery
        .vms
        .retain(|vm| vm.vm_id != affected);
    presentation::present(&ui, &s, true);
    assert!(!ui.get_selected_recovery());
    assert!(ui.get_vm_details().is_empty());
    assert!(ui.get_recovery_summary().contains(&affected));
    assert_eq!(serde_json::to_value(&s.recovery).unwrap(), before);
    s.recovery = None;
    presentation::present(&ui, &s, true);
    assert!(!ui.get_recovery());
    assert!(!ui.get_selected_recovery());
    assert!(ui.get_vm_details().is_empty());
}

#[test]
fn multiple_vm_holds_preserve_independent_editing_and_save_only_selection() {
    use hyper_gpu_support::configuration_store::SavePhase;
    let mut s = action_state();
    let first = id(&s).to_owned();
    let second = "87654321-4321-4321-4321-cba987654321".to_owned();
    let third = "87654321-4321-4321-4321-cba987654322".to_owned();
    let mut next = s.view.inventory.clone().unwrap();
    for id in [&second, &third] {
        let mut vm = next.discovery.vms[0].clone();
        vm.vm_id = id.clone();
        let mut target = next.enrolled[0].clone();
        target.vm_id = id.clone();
        next.discovery.vms.push(vm);
        next.enrolled.push(target);
        next.managed.insert(id.clone(), None);
    }
    s.refresh(next);
    let mut failed = save_record(&s);
    failed.phase = SavePhase::EffectsStarted;
    let retained = serde_json::to_value(&failed).unwrap();
    s.sync_recovery(Some(failed.clone()), None, None).unwrap();
    assert!(s.editor_eligibility(&first).is_err());
    s.select_vm(Some(second.clone())).unwrap();
    let target = s.target(&second).unwrap();
    let values = allocation_values(&target);
    let actions = s.selected_actions(0, &values);
    assert!(!actions.recovery);
    assert!(
        actions.verify_allowed
            && actions.repair_allowed
            && actions.remove_allowed
            && actions.forget_allowed
    );
    assert!(s.draft(&second, true, 0, &values).is_ok());
    let mut unsaved = save_record(&s);
    unsaved.operation_id = "second-operation".into();
    // A separate save failure is retained alongside the first VM's uncertain effects.
    s.sync_recovery(vec![failed.clone(), unsaved.clone()], None, None)
        .unwrap();
    assert!(s.view.unsaved);
    assert_eq!(
        s.save_only_record().unwrap().operation_id,
        "second-operation"
    );
    s.select_vm(Some(third.clone())).unwrap();
    assert!(!s.view.unsaved);
    assert!(s.editor_eligibility(&third).is_ok());
    assert!(s.draft(&third, true, 0, &values).is_ok());
    s.select_vm(Some(first.clone())).unwrap();
    assert_eq!(
        s.recovery.as_ref().unwrap().operation_id,
        failed.operation_id
    );
    assert!(s.save_only_record().is_err());
    s.select_vm(Some(second.clone())).unwrap();
    assert!(s.view.unsaved);
    assert_eq!(s.save_only_record().unwrap().target.vm_id, second);
    // Finishing the second operation's readback must not attribute the first hold to it.
    let mut committed = hyper_gpu_support::configuration_store::StoreSnapshot::default();
    let source = hyper_gpu_support::model::Configuration {
        schema: 3,
        targets: vec![target.clone()],
    }
    .vm_documents()
    .unwrap();
    for (name, text) in source {
        committed.insert(&name, Ok(text));
    }
    s.finish_apply(
        &target,
        "second-operation",
        true,
        Ok(OperationReadback {
            observation: None,
            committed,
            recovery: vec![failed],
        }),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(s.recovery_for_vm(&first).unwrap()).unwrap(),
        retained
    );
    assert!(s.recovery_for_vm(&second).is_none());
    assert!(!s.view.unsaved);
}

#[test]
fn unfinished_operation_explains_recovery_without_discarding_edits_or_releasing_hold() {
    use hyper_gpu_support::configuration_store::SavePhase;
    let mut s = state();
    assert!(s.action_recovery_message().is_none());
    let draft = s.target(id(&s)).unwrap();
    s.view.draft = Some(draft.clone());
    let mut record = save_record(&s);
    record.publication_required = false;
    assert!(s.recovery_without_guest(&record).unwrap());
    record.phase = SavePhase::EffectsStarted;
    assert!(!s.recovery_without_guest(&record).unwrap());
    s.recovery = Some(record);
    let summary = crate::gui::live::presentation::recovery_summary(&s, id(&s)).unwrap();
    assert!(summary.contains("GPU verification"));
    assert!(!summary.contains("Setup interrupted"));
    let body = s.action_recovery_message().unwrap();
    assert!(body.contains("Renamable VM"));
    assert!(body.contains("Review recovery"));
    assert!(body.contains("separate confirmation"));
    assert_eq!(s.view.draft, Some(draft));
    assert!(matches!(
        s.recovery.as_ref().unwrap().phase,
        SavePhase::EffectsStarted
    ));
    assert!(s.editor_eligibility(id(&s)).is_err());
    // The retained hold remains visible without blocking another VM.
    s.view.selected = Some("another-vm".into());
    let summary = crate::gui::live::presentation::recovery_summary(&s, "another-vm").unwrap();
    assert!(summary.contains("Other VMs can still be changed") && summary.contains("Renamable VM"));
    assert!(s.action_recovery_message().is_none());
    s.recovery = None;
    assert!(s.action_recovery_message().is_none());
}

#[test]
fn recovery_identity_follows_the_record_when_an_unconfigured_vm_is_selected() {
    use hyper_gpu_support::configuration_store::SavePhase;
    let mut s = state();
    let mut record = save_record(&s);
    record.publication_required = false;
    record.phase = SavePhase::EffectsStarted;
    record.initial.name = "Recorded VM name".into();
    let affected_id = record.target.vm_id.clone();
    let mut unrelated = s.view.vm(&affected_id).unwrap().clone();
    unrelated.vm_id = "87654321-4321-4321-4321-cba987654321".into();
    unrelated.name = "Untouched VM".into();
    unrelated.gpus.clear();
    s.view
        .inventory
        .as_mut()
        .unwrap()
        .discovery
        .vms
        .push(unrelated.clone());
    s.view.selected = Some(unrelated.vm_id.clone());
    s.recovery = Some(record);

    let summary = crate::gui::live::presentation::recovery_summary(&s, &unrelated.vm_id).unwrap();
    assert!(summary.contains("Renamable VM") && summary.contains("GPU verification"));
    assert!(summary.contains(&affected_id));
    assert!(!summary.contains("Untouched VM"));
    assert!(!summary.contains("setup"));
    assert!(s.editor_eligibility(&unrelated.vm_id).is_ok());
    assert_eq!(
        serde_json::to_value(s.view.vm(&unrelated.vm_id).unwrap()).unwrap(),
        serde_json::to_value(&unrelated).unwrap()
    );

    // Discovery can lose the affected VM; never borrow the selected VM's name.
    s.view
        .inventory
        .as_mut()
        .unwrap()
        .discovery
        .vms
        .retain(|vm| vm.vm_id != affected_id);
    let summary = crate::gui::live::presentation::recovery_summary(&s, &unrelated.vm_id).unwrap();
    assert!(summary.contains("Recorded VM name") && summary.contains("GPU verification"));
    assert!(s.action_recovery_message().is_none());
    assert!(s.editor_eligibility(&unrelated.vm_id).is_ok());

    s.recovery.as_mut().unwrap().publication_required = true;
    let summary = crate::gui::live::presentation::recovery_summary(&s, &unrelated.vm_id).unwrap();
    assert!(summary.contains("Recorded VM name") && summary.contains("GPU change"));
    assert!(!summary.contains("verification"));
    assert!(matches!(
        s.recovery.as_ref().unwrap().phase,
        SavePhase::EffectsStarted
    ));
    assert_eq!(
        serde_json::to_value(s.view.vm(&unrelated.vm_id).unwrap()).unwrap(),
        serde_json::to_value(&unrelated).unwrap()
    );
}

#[test]
fn admitted_recovery_needs_no_guest_sign_in_even_when_presentation_is_stale() {
    use hyper_gpu_support::configuration_store::SavePhase;
    let mut s = state();
    let mut record = save_record(&s);
    record.phase = SavePhase::Admitted;
    record.graphics_required = Some(true);
    record.initial_journal = None;
    s.recovery = Some(record.clone());
    let retained = serde_json::to_value(&s.recovery).unwrap();
    assert!(s.recovery_without_guest(&record).unwrap());
    s.view.needs_readback = true;
    s.view.inventory = None;
    assert!(s.recovery_without_guest(&record).unwrap());
    assert_eq!(serde_json::to_value(&s.recovery).unwrap(), retained);
    // Once effects started, stale facts cannot promise a record-only exit.
    record.phase = SavePhase::EffectsStarted;
    assert!(!s.recovery_without_guest(&record).unwrap());
    assert_eq!(serde_json::to_value(&s.recovery).unwrap(), retained);
}

#[test]
fn verification_recovery_requires_restoration_proof_and_rejects_changed_facts() {
    use hyper_gpu_support::{
        configuration_store::SavePhase,
        workflow::{GuestCheck, Journal},
    };
    let mut s = state();
    let mut record = save_record(&s);
    record.publication_required = false;
    record.phase = SavePhase::EffectsStarted;
    record.driver_version = "fixture".into();
    record.expected = s.committed.revision(&record.target.vm_id).unwrap();
    let vm_id = record.target.vm_id.clone();
    let journal = Journal {
        schema: 1,
        vm_id: vm_id.clone(),
        gpu_interface: record.target.gpu_interface.clone(),
        original: record.initial.settings.clone(),
        applied: record.initial.settings.clone(),
        prepared: Some("historical-payload".into()),
        pending: true,
        restore_power: record.initial.power.clone(),
        verification_only: true,
        last_verified: Some(1),
        latest_check: Some(GuestCheck::Failed),
        preparation_started: None,
        check_restored: false,
        settings_released: false,
    };
    s.view
        .inventory
        .as_mut()
        .unwrap()
        .managed
        .insert(vm_id.clone(), Some(journal));
    assert!(!s.recovery_without_guest(&record).unwrap());
    s.view
        .inventory
        .as_mut()
        .unwrap()
        .managed
        .get_mut(&vm_id)
        .unwrap()
        .as_mut()
        .unwrap()
        .check_restored = true;
    assert!(s.recovery_without_guest(&record).unwrap());
    let body = crate::gui::wording::recovery_progress(&record, "Renamable VM", true);
    assert!(body.contains("does not retry the guest check"));
    s.view.inventory.as_mut().unwrap().discovery.vms[0]
        .settings
        .high_mmio += 1;
    assert!(!s.recovery_without_guest(&record).unwrap());
    s.view.inventory.as_mut().unwrap().discovery.vms[0] = record.initial.clone();
    s.view.needs_readback = true;
    assert!(!s.recovery_without_guest(&record).unwrap());
}

#[test]
fn external_save_clears_save_only_state_and_preserves_conflicting_draft() {
    let mut s = state();
    let target = s.target(id(&s)).unwrap();
    s.view.applied(&target).unwrap();
    let mut draft = target.clone();
    draft.enabled = !draft.enabled;
    s.view.draft = Some(draft.clone());
    assert!(
        s.load_committed(committed(vec![target], "external save"))
            .is_err()
    );
    s.refresh(inventory());
    s.sync_recovery(None, None, None).unwrap();
    assert!(!s.view.unsaved);
    assert!(s.view.verified_unsaved.is_none());
    assert_eq!(s.view.draft, Some(draft));
    assert!(s.editor_eligibility(id(&s)).is_err());
}

#[test]
fn absent_recovery_requires_matching_readable_committed_intent() {
    for unreadable in [false, true] {
        let mut s = state();
        let target = s.target(id(&s)).unwrap();
        s.view.applied(&target).unwrap();
        if unreadable {
            s.committed
                .errors
                .insert("store".into(), "access denied".into());
        }
        s.refresh(inventory());
        assert!(s.sync_recovery(None, None, None).is_err());
        assert!(s.view.unsaved);
        assert_eq!(s.view.verified_unsaved, Some(target));
        assert!(s.view.needs_readback);
    }
}

#[test]
fn standalone_verify_does_not_offer_publication_and_external_reconcile_releases_hold() {
    let mut s = state();
    let mut record = save_record(&s);
    record.publication_required = false;
    s.sync_recovery(Some(record), None, None).unwrap();
    assert!(!s.view.unsaved);
    assert!(s.view.verified_unsaved.is_none());
    assert!(s.editor_eligibility(id(&s)).is_err());
    s.refresh(inventory());
    s.sync_recovery(None, None, None).unwrap();
    assert!(s.editor_eligibility(id(&s)).is_ok());
    assert!(s.view.configuration.targets.is_empty());
}

#[test]
fn verified_record_derives_save_only_state_and_import_holds_edits() {
    let mut s = state();
    let record = save_record(&s);
    let target = record.target.clone();
    s.sync_recovery(Some(record), None, None).unwrap();
    assert!(s.view.unsaved);
    assert_eq!(s.view.verified_unsaved, Some(target.clone()));
    s.load_committed(committed(vec![target], "external save"))
        .unwrap();
    let import = hyper_gpu_support::configuration_store::ImportRecord {
        schema: 1,
        operation_id: "test-import".into(),
        source_revision: hyper_gpu_support::configuration_store::Revision::Missing,
        documents: Default::default(),
    };
    s.refresh(inventory());
    s.sync_recovery(None, Some(import), None).unwrap();
    assert!(!s.view.unsaved);
    assert!(s.editor_eligibility(id(&s)).is_err());
    s.refresh(inventory());
    s.sync_recovery(None, None, None).unwrap();
    assert!(s.editor_eligibility(id(&s)).is_ok());
}

#[test]
fn lost_terminal_reply_uses_durable_phase_without_inventing_completion() {
    use hyper_gpu_support::configuration_store::SavePhase;
    let mut s = state();
    let target = s.target(id(&s)).unwrap();
    s.view.draft = Some(target.clone());
    let mut record = save_record(&s);
    record.phase = SavePhase::EffectsStarted;
    s.finish_readback(Ok(OperationReadback {
        observation: None,
        committed: Default::default(),
        recovery: vec![record],
    }))
    .unwrap();
    assert!(s.view.needs_readback);
    assert_eq!(s.view.draft, Some(target));
    assert!(s.applied_text(id(&s)).contains("uncertain"));
    assert!(s.save_only_record().is_err());
    assert!(s.editor_eligibility(id(&s)).is_err());
}

#[test]
fn save_retry_is_available_only_for_publication_receipts() {
    use hyper_gpu_support::configuration_store::SavePhase;
    let mut s = state();
    let mut record = save_record(&s);
    let effective = record.initial.clone();
    for phase in [SavePhase::Admitted, SavePhase::EffectsStarted] {
        record.phase = phase;
        s.sync_recovery(Some(record.clone()), None, None).unwrap();
        assert!(s.save_only_record().is_err());
    }
    record.phase = SavePhase::Published(effective);
    s.sync_recovery(Some(record.clone()), None, None).unwrap();
    assert!(s.save_only_record().is_ok());
    assert!(!s.view.unsaved);
    assert!(s.applied_text(id(&s)).contains("published"));
    record.publication_required = false;
    s.sync_recovery(Some(record), None, None).unwrap();
    assert!(s.save_only_record().is_err());
}
