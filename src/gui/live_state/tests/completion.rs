use super::*;

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
        Ok(OperationReadback {
            committed: store,
            recovery: vec![record],
        }),
    )
    .unwrap();
    assert_eq!(s.target(&original.vm_id).unwrap(), original);
    assert_eq!(
        serde_json::to_value(s.view.vm(&applied.vm_id).unwrap()).unwrap(),
        serde_json::to_value(observed).unwrap()
    );
    assert_eq!(s.view.verified_unsaved, Some(applied.clone()));
    assert!(s.view.draft.is_none());
    assert!(s.view.needs_readback);
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
    assert!(s.approved_plan.is_none());
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
                Ok(OperationReadback {
                    committed: Default::default(),
                    recovery: vec![record],
                })
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
        Ok(OperationReadback {
            committed: store.clone(),
            recovery: vec![],
        }),
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
