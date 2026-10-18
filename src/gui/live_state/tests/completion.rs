use super::*;
use hyper_gpu_support::configuration_store::Revision;

#[test]
fn invalid_completion_scope_or_revision_cannot_replace_loaded_state_or_release_recovery() {
    use hyper_gpu_support::observation::Mode;
    for fault in 0..6 {
        let mut s = state();
        let record = save_record(&s);
        s.sync_recovery(Some(record.clone()), None, None).unwrap();
        let before = s.view.vm(id(&s)).unwrap().clone();
        let mut observation = completion_observation(&s, s.committed.clone(), vec![]);
        observation.inventory.discovery.vms[0].settings.high_mmio += 1;
        match fault {
            0 => observation.provenance.selected = None,
            1 => observation.provenance.configuration = Revision::of(b"unrelated saved state"),
            2 => observation.provenance.schema += 1,
            3 => {
                observation.provenance.mode = Mode::Full;
                observation.provenance.selected = None;
            }
            4 => {
                let other = "12345678-1234-1234-1234-000000000002";
                observation.provenance.selected.as_mut().unwrap().vm_id = other.into();
                observation.inventory.discovery.vms[0].vm_id = other.into();
            }
            _ => observation.inventory.discovery.gpus[0].interface.push('x'),
        }
        assert!(s.finish_readback(Ok(observation)).is_err(), "fault {fault}");
        assert_eq!(s.view.vm(id(&s)).unwrap(), &before);
        assert_eq!(s.recovery_for_vm(id(&s)), Some(&record));
        assert!(s.view.needs_readback && s.editor_eligibility(id(&s)).is_err());
    }
}

#[test]
fn configure_completion_retains_newer_same_vm_edits_even_when_readback_fails() {
    for saved in [false, true] {
        let mut s = state();
        let applied = s.target(id(&s)).unwrap();
        let mut newer = applied.clone();
        newer.enabled = !newer.enabled;
        s.view.draft = Some(newer.clone());
        assert!(
            s.finish_apply(
                &applied,
                "operation",
                saved,
                Err("readback unavailable".into())
            )
            .is_err()
        );
        assert_eq!(s.view.draft, Some(newer));
        assert!(s.view.needs_readback);
        if saved {
            assert!(s.view.configuration.targets.contains(&applied));
        } else {
            assert_eq!(s.view.verified_unsaved, Some(applied));
        }
    }
}

#[test]
fn selected_completion_uses_fresh_rows_and_retains_draft_and_revision_conflicts() {
    let mut s = state();
    let original = s.target(id(&s)).unwrap();
    s.load_committed(committed(vec![original.clone()], "before"))
        .unwrap();
    let mut draft = original.clone();
    draft.enabled = !draft.enabled;
    s.view.draft = Some(draft.clone());
    let mut observation = completion_observation(&s, s.committed.clone(), vec![]);
    observation.inventory.discovery.vms[0].settings.high_mmio += 1;
    let wanted = observation.inventory.discovery.vms[0].clone();
    s.finish_readback(Ok(observation)).unwrap();
    assert_eq!(s.view.draft, Some(draft));
    assert_eq!(
        s.view.vm(&original.vm_id).unwrap().operational(),
        wanted.operational()
    );
    assert!(!s.view.needs_readback);
    assert!(s.observation.is_some());
    assert!(!s.observation_historical(&original.vm_id));
    assert!(s.observation_historical("12345678-1234-1234-1234-000000000002"));
    assert!(
        s.editor_eligibility("12345678-1234-1234-1234-000000000002")
            .is_err()
    );
    let mut changed = completion_observation(&s, s.committed.clone(), vec![]);
    changed.committed = committed(vec![original.clone()], "externally changed");
    changed.provenance.configuration =
        Revision::of(&serde_json::to_vec(&changed.committed).unwrap());
    assert!(s.finish_readback(Ok(changed)).is_err());
    assert!(s.view.needs_readback);
}

#[test]
fn failed_selected_completion_and_disappearing_save_receipt_never_reuse_stale_success() {
    let mut s = state();
    let mut observation = completion_observation(&s, s.committed.clone(), vec![]);
    observation.inventory.discovery.vms.clear();
    assert!(s.finish_readback(Ok(observation)).is_err());
    assert!(s.view.needs_readback);
    let mut s = state();
    let applied = s.target(id(&s)).unwrap();
    let observation = completion_observation(&s, s.committed.clone(), vec![]);
    assert!(
        s.finish_apply(&applied, "missing-receipt", false, Ok(observation))
            .is_err()
    );
    assert!(s.view.needs_readback && s.view.unsaved);
}

#[test]
fn completed_unsaved_apply_keeps_saved_observed_and_applied_state_distinct() {
    let mut s = state();
    let original = s.target(id(&s)).unwrap();
    let store = committed(vec![original.clone()], "saved before operation");
    s.load_committed(store.clone()).unwrap();
    let mut applied = original.clone();
    applied.enabled = false;
    s.view.draft = Some(applied.clone());
    let observed = s.view.vm(id(&s)).unwrap().clone();
    let mut record = save_record(&s);
    record.target = applied.clone();
    s.finish_apply(
        &applied,
        "test-operation",
        false,
        Ok(completion_observation(&s, store, vec![record])),
    )
    .unwrap();
    assert_eq!(s.target(&original.vm_id).unwrap(), original);
    assert_eq!(
        serde_json::to_value(s.view.vm(&applied.vm_id).unwrap()).unwrap(),
        serde_json::to_value(observed).unwrap()
    );
    assert_eq!(s.view.verified_unsaved, Some(applied.clone()));
    assert!(s.view.draft.is_none());
    assert!(!s.view.needs_readback);
    assert!(
        s.applied_text(&applied.vm_id)
            .contains("publication is pending")
    );
    assert_eq!(s.save_only_record().unwrap().target, applied);
    assert!(s.editor_eligibility(id(&s)).is_err());
}

#[test]
fn completed_apply_read_failure_retains_known_unsaved_intent_without_retry_authority() {
    let mut s = state();
    let target = s.target(id(&s)).unwrap();
    s.view.draft = Some(target.clone());
    assert!(
        s.finish_apply(
            &target,
            "test-operation",
            false,
            Err("protected read failed".into())
        )
        .is_err()
    );
    assert_eq!(s.view.verified_unsaved, Some(target));
    assert!(s.view.unsaved && s.view.needs_readback);
    assert!(s.save_only_record().is_err());
}

#[test]
fn completed_apply_rejects_a_replaced_operation_receipt() {
    for changed_target in [false, true] {
        let mut s = state();
        let target = s.target(id(&s)).unwrap();
        let mut record = save_record(&s);
        if changed_target {
            record.target.enabled = !target.enabled;
        } else {
            record.operation_id = "another-operation".into();
        }
        assert!(
            s.finish_apply(
                &target,
                "test-operation",
                false,
                Ok(completion_observation(&s, Default::default(), vec![record]))
            )
            .unwrap_err()
            .contains("receipt changed")
        );
        assert!(s.view.needs_readback);
        assert!(s.editor_eligibility(id(&s)).is_err());
    }
}

#[test]
fn save_only_completion_updates_selected_intent_and_preserves_another_vm_draft() {
    let mut s = state();
    let mut applied = s.target(id(&s)).unwrap();
    applied.enabled = false;
    let mut other = applied.clone();
    other.vm_id = "12345678-1234-1234-1234-000000000002".into();
    let mut other_draft = other.clone();
    other_draft.enabled = true;
    let store = committed(vec![applied.clone(), other.clone()], "published");
    s.load_committed(store.clone()).unwrap();
    s.view.draft = Some(other_draft.clone());
    s.finish_apply(
        &applied,
        "test-operation",
        true,
        Ok(completion_observation(&s, store.clone(), vec![])),
    )
    .unwrap();
    assert_eq!(s.view.draft, Some(other_draft));
    assert_eq!(s.target(&applied.vm_id).unwrap(), applied);
    assert_eq!(s.target(&other.vm_id).unwrap(), other);
    assert!(!s.view.unsaved);
    assert!(s.view.verified_unsaved.is_none());
    let mut reopened = State::default();
    reopened.load_committed(store).unwrap();
    reopened.refresh(inventory());
    assert_eq!(
        reopened.target(id(&reopened)).unwrap(),
        s.target(id(&s)).unwrap()
    );
    assert!(reopened.view.draft.is_none());
}
