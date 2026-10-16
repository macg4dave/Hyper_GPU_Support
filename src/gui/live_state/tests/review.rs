use super::*;

#[test]
fn permission_continuation_requires_fresh_matching_enrollment_without_approving_apply() {
    let s = state();
    let target = s.target(id(&s)).unwrap();
    assert!(s.enrollment_continuation(&target, &target).is_ok());
    assert!(s.approved_plan.is_none());
    assert!(s.reviewed_plan(&target).is_err());

    for scenario in 0..6 {
        let mut s = state();
        let mut current = target.clone();
        match scenario {
            0 => s.view.needs_readback = true,
            1 => s.view.inventory.as_mut().unwrap().enrolled.clear(),
            2 => s.view.selected = Some("another-vm".into()),
            3 => current.enabled = !current.enabled,
            4 => current.gpu_interface = "another-gpu".into(),
            _ => s.recovery = Some(save_record(&s)),
        }
        assert!(
            s.enrollment_continuation(&target, &current).is_err(),
            "scenario {scenario}"
        );
        assert!(s.approved_plan.is_none());
    }
}

#[test]
fn removal_is_reviewable_without_preparation_or_attachment_and_preserves_requests() {
    let mut s = state();
    let vm_id = id(&s).to_owned();
    let mut requested = s.target(&vm_id).unwrap();
    requested.compute = Some(Allocation {
        minimum: 10,
        optimal: 50,
        maximum: 90,
    });
    requested.vram = Some(Allocation {
        minimum: 10,
        optimal: 50,
        maximum: 100,
    });
    let values = allocation_values(&requested);
    s.view
        .inventory
        .as_mut()
        .unwrap()
        .managed
        .insert(vm_id.clone(), None);
    s.view.inventory.as_mut().unwrap().discovery.vms[0]
        .gpus
        .clear();
    let removed = s.removal_draft(&vm_id, 0, &values).unwrap();
    assert!(!removed.enabled);
    assert_eq!(removed.compute, requested.compute);
    assert_eq!(removed.vram, requested.vram);
    let mut plan = review_plan();
    plan.desired = removed.clone();
    plan.preview.credentials_required = false;
    let revision = s.committed.revision(&vm_id).unwrap();
    assert!(review_summary(&plan).contains("Remove GPU support"));
    assert!(review_summary(&plan).contains("Keep guest driver files, authorization"));
    s.accept_review(plan, &removed, revision).unwrap();
    assert!(!s.reviewed_plan(&removed).unwrap().plan.desired.enabled);
    s.cancel_review();
    assert!(s.reviewed_plan(&removed).is_err());
    assert!(s.enrolled(&removed));
}

#[test]
fn removal_does_not_bypass_recovery_authority_or_selection() {
    for scenario in 0..3 {
        let mut s = state();
        let vm_id = id(&s).to_owned();
        match scenario {
            0 => s.view.inventory.as_mut().unwrap().enrolled.clear(),
            1 => s.recovery = Some(save_record(&s)),
            _ => s.view.selected = Some("another-vm".into()),
        }
        assert!(
            s.removal_draft(&vm_id, 0, &values("10", "50", "100"))
                .is_err()
        );
        assert!(s.approved_plan.is_none());
    }
}

#[test]
fn reinstall_review_is_explicit_cancelable_and_bound_to_selection_and_revision() {
    let mut s = state();
    let mut plan = review_plan();
    plan.reinstall = true;
    plan.preview.credentials_required = true;
    plan.preview.actions = vec![
        hyper_gpu_support::workflow::PreviewAction::Prepare,
        hyper_gpu_support::workflow::PreviewAction::Verify,
        hyper_gpu_support::workflow::PreviewAction::Shutdown,
    ];
    let target = plan.desired.clone();
    let revision = s.committed.revision(&target.vm_id).unwrap();
    let original = target.clone();
    assert!(review_summary(&plan).contains("Repair / Reinstall"));
    assert!(review_summary(&plan).contains("matching history"));
    s.accept_review(plan.clone(), &target, revision.clone())
        .unwrap();
    assert!(s.reviewed_plan(&target).unwrap().plan.reinstall);
    assert_eq!(s.reviewed_plan(&target).unwrap().plan.desired, original);
    s.view.selected = Some("another-vm".into());
    assert!(s.reviewed_plan(&target).is_err());
    s.view.selected = Some(target.vm_id.clone());
    s.cancel_review();
    assert!(s.reviewed_plan(&target).is_err());
    s.accept_review(plan, &target, revision).unwrap();
    s.committed.documents.remove(&target.vm_id);
    s.committed
        .errors
        .insert(format!("{}.toml", target.vm_id), "unreadable".into());
    assert!(s.reviewed_plan(&target).is_err());
}

#[test]
fn review_binds_target_draft_and_exact_saved_revision() {
    use hyper_gpu_support::configuration_store::{Revision, SavedDocument};
    let mut s = state();
    let plan = review_plan();
    let target = plan.desired.clone();
    let revision = s.committed.revision(&target.vm_id).unwrap();
    s.accept_review(plan.clone(), &target, revision.clone())
        .unwrap();
    assert_eq!(s.reviewed_plan(&target).unwrap().revision, revision);

    let mut edited = target.clone();
    edited.enabled = false;
    assert!(s.reviewed_plan(&edited).is_err());
    edited = target.clone();
    edited.vram.as_mut().unwrap().optimal += 1;
    assert!(s.reviewed_plan(&edited).is_err());
    s.view.selected = Some("another-vm".into());
    assert!(s.reviewed_plan(&target).is_err());
    assert!(
        s.accept_review(plan.clone(), &target, revision.clone())
            .is_err()
    );
    s.view.selected = Some(target.vm_id.clone());
    s.accept_review(plan.clone(), &target, revision.clone())
        .unwrap();
    s.committed.documents.insert(
        target.vm_id.clone(),
        SavedDocument {
            target: target.clone(),
            source: "externally changed bytes".into(),
            revision: Revision::of(b"externally changed bytes"),
        },
    );
    assert!(s.reviewed_plan(&target).is_err());
    assert!(s.accept_review(plan, &target, revision).is_err());
    assert!(s.approved_plan.is_none());
}

#[test]
fn late_plan_reply_cannot_approve_an_edited_draft_or_different_observed_vm() {
    let mut s = state();
    let mut plan = review_plan();
    let target = plan.desired.clone();
    let revision = s.committed.revision(&target.vm_id).unwrap();
    let mut edited = target.clone();
    edited.enabled = false;
    assert!(
        s.accept_review(plan.clone(), &edited, revision.clone())
            .is_err()
    );
    assert!(s.approved_plan.is_none());
    plan.observed.vm_id = "another-vm".into();
    assert!(s.accept_review(plan, &target, revision).is_err());
    assert!(s.approved_plan.is_none());
}

#[test]
fn cancel_returns_to_draft_and_refresh_requires_new_review() {
    let mut s = state();
    let plan = review_plan();
    let target = plan.desired.clone();
    s.view.draft = Some(target.clone());
    let revision = s.committed.revision(&target.vm_id).unwrap();
    s.accept_review(plan.clone(), &target, revision.clone())
        .unwrap();
    s.cancel_review();
    assert_eq!(s.view.draft.as_ref(), Some(&target));
    assert!(s.reviewed_plan(&target).is_err());
    s.accept_review(plan, &target, revision).unwrap();
    s.sync_recovery(None, None, None).unwrap();
    assert!(s.reviewed_plan(&target).is_err());
    assert_eq!(s.view.draft.as_ref(), Some(&target));
}

#[test]
fn cancel_and_discard_clear_verification_and_authority_approvals() {
    use hyper_gpu_support::{configuration_store::Revision, workflow::VerificationPlan};
    for discard in [false, true] {
        let mut s = state();
        let plan = review_plan();
        let target = plan.desired.clone();
        let saved = s.view.configuration.targets.clone();
        s.view.draft = Some(target.clone());
        let revision = s.committed.revision(&target.vm_id).unwrap();
        s.accept_review(plan.clone(), &target, revision).unwrap();
        s.verification_plan = Some(VerificationPlan {
            target: target.clone(),
            observed: plan.observed,
            gpu: s.view.inventory.as_ref().unwrap().discovery.gpus[0].clone(),
            managed: None,
            payload_digest: "current".into(),
        });
        s.enrollment_recovery_revision = Some(Revision::of(b"reviewed authority"));
        if discard {
            s.discard();
            assert!(s.view.draft.is_none());
        } else {
            s.cancel_review();
            assert_eq!(s.view.draft.as_ref(), Some(&target));
        }
        assert!(s.approved_plan.is_none());
        assert!(s.verification_plan.is_none());
        assert!(s.enrollment_recovery_revision.is_none());
        assert_eq!(s.view.configuration.targets, saved);
        assert!(s.view.vm(&target.vm_id).unwrap().gpus.is_empty());
    }
}
