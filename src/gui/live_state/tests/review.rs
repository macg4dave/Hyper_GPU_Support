use super::*;

#[test]
fn permission_continuation_requires_fresh_matching_enrollment_without_approving_apply() {
    let s = state();
    let target = s.target(id(&s)).unwrap();
    assert!(s.enrollment_continuation(&target, &target).is_ok());

    for scenario in 0..6 {
        let mut s = state();
        let mut current = target.clone();
        match scenario {
            0 => s.view.needs_readback = true,
            1 => s.view.inventory.as_mut().unwrap().enrolled.clear(),
            2 => s.view.selected = Some("another-vm".into()),
            3 => current.enabled = !current.enabled,
            4 => current.gpu_interface = "another-gpu".into(),
            _ => s.recoveries = vec![save_record(&s)],
        }
        assert!(
            s.enrollment_continuation(&target, &current).is_err(),
            "scenario {scenario}"
        );
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
    s.validate_prepared(&plan, &removed, &revision).unwrap();
    assert!(s.enrolled(&removed));
}

#[test]
fn removal_does_not_bypass_recovery_authority_or_selection() {
    for scenario in 0..3 {
        let mut s = state();
        let vm_id = id(&s).to_owned();
        match scenario {
            0 => s.view.inventory.as_mut().unwrap().enrolled.clear(),
            1 => s.recoveries = vec![save_record(&s)],
            _ => s.view.selected = Some("another-vm".into()),
        }
        assert!(
            s.removal_draft(&vm_id, 0, &values("10", "50", "100"))
                .is_err()
        );
    }
}

#[test]
fn prepared_acknowledgement_rejects_changed_target_selection_revision_and_recovery() {
    use hyper_gpu_support::configuration_store::{Revision, SavedDocument};
    for scenario in 0..9 {
        let mut s = state();
        let mut plan = review_plan();
        let target = plan.desired.clone();
        let mut current = target.clone();
        let revision = s.committed.revision(&target.vm_id).unwrap();
        match scenario {
            0 => {}
            1 => current.enabled = !current.enabled,
            2 => current.vram.as_mut().unwrap().optimal += 1,
            3 => s.view.selected = Some("another-vm".into()),
            4 => {
                s.committed.documents.insert(
                    target.vm_id.clone(),
                    SavedDocument {
                        target: target.clone(),
                        source: "changed".into(),
                        revision: Revision::of(b"changed"),
                    },
                );
            }
            5 => {
                plan.observed.vm_id = "another-vm".into();
            }
            6 => s.view.needs_readback = true,
            7 => s.recoveries = vec![save_record(&s)],
            _ => {
                s.committed
                    .errors
                    .insert(format!("{}.toml", target.vm_id), "unreadable".into());
            }
        }
        assert_eq!(
            s.validate_prepared(&plan, &current, &revision).is_ok(),
            scenario == 0,
            "scenario {scenario}"
        );
        assert!(
            s.view.draft.is_none(),
            "acknowledgement never creates intent"
        );
    }
}

#[test]
fn permission_preparation_rejects_changed_selection_draft_pair_or_eligibility_without_mutation() {
    for scenario in 0..11 {
        let mut s = state();
        let expected = s.target(id(&s)).unwrap();
        let mut current = expected.clone();
        let mut pair = hyper_gpu_support::enrollment::PairReview {
            vm_id: expected.vm_id.clone(),
            gpu_interface: expected.gpu_interface.clone(),
            expected: hyper_gpu_support::configuration_store::Revision::of(b"authority"),
            operator_sid: "S-1-5-21-1".into(),
        };
        match scenario {
            0 => {}
            1 => s.view.selected = Some("another-vm".into()),
            2 => current.enabled = !current.enabled,
            3 => {
                current.vram = Some(Allocation {
                    minimum: 10,
                    optimal: 50,
                    maximum: 100,
                })
            }
            4 => current.gpu_interface = "another-gpu".into(),
            5 => pair.vm_id = "another-vm".into(),
            6 => pair.gpu_interface = "another-gpu".into(),
            7 => s.view.needs_readback = true,
            8 => s.recoveries = vec![save_record(&s)],
            9 => s.view.inventory.as_mut().unwrap().discovery.vms[0].power = Power::Other(32768),
            _ => s.view.inventory.as_mut().unwrap().discovery.gpus.clear(),
        }
        let before = s.view.configuration.targets.clone();
        assert_eq!(
            s.validate_permission_preparation(&expected, &current, &pair)
                .is_ok(),
            scenario == 0,
            "scenario {scenario}"
        );
        assert_eq!(s.view.configuration.targets, before);
        assert!(s.view.draft.is_none());
    }
}
