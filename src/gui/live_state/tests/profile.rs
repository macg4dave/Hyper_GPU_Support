use super::*;

#[test]
fn deletion_preparation_is_bound_to_selection_pairing_and_saved_profiles() {
    use hyper_gpu_support::{
        configuration_store::Revision,
        enrollment::{PairReview, RemovalReview},
    };
    let mut s = state();
    let vm_id = id(&s).to_owned();
    s.view
        .inventory
        .as_mut()
        .unwrap()
        .managed
        .insert(vm_id.clone(), None);
    let target = s.forget_target().unwrap();
    s.view.inventory.as_mut().unwrap().discovery.vms[0].gpus = vec![target.gpu_interface.clone()];
    let review = RemovalReview {
        recovery: Default::default(),
        pair: PairReview {
            vm_id: vm_id.clone(),
            gpu_interface: target.gpu_interface.clone(),
            expected: Revision::of(b"policy"),
            operator_sid: "S-1-5-21-1".into(),
        },
        configurations: std::collections::BTreeMap::from([(vm_id.clone(), Revision::Missing)]),
        journals: std::collections::BTreeMap::from([(vm_id.clone(), Revision::Missing)]),
    };
    s.validate_forget_review(&review).unwrap();
    assert!(s.enrolled(&target));
    assert!(s.committed.documents.is_empty());
    let mut changed_pair = review.clone();
    changed_pair.pair.gpu_interface = "different-gpu".into();
    assert!(s.validate_forget_review(&changed_pair).is_err());
    s.view.selected = Some("another-vm".into());
    assert!(s.validate_forget_review(&review).is_err());
    s.view.selected = Some(vm_id.clone());
    let mut changed = target.clone();
    changed.enabled = false;
    s.committed.documents.insert(
        vm_id.clone(),
        hyper_gpu_support::configuration_store::SavedDocument {
            target: changed,
            source: "new disabled intent".into(),
            revision: Revision::of(b"new disabled intent"),
        },
    );
    assert!(s.validate_forget_review(&review).is_err());
}

fn profiles_for_deletion() -> (State, Target, Target) {
    let mut s = state();
    let mut selected = s.target(id(&s)).unwrap();
    selected.enabled = false;
    let mut other = selected.clone();
    other.vm_id = "87654321-4321-4321-4321-cba987654321".into();
    s.load_committed(committed(vec![selected.clone(), other.clone()], "profiles"))
        .unwrap();
    let inventory = s.view.inventory.as_mut().unwrap();
    let mut vm = inventory.discovery.vms[0].clone();
    vm.vm_id = other.vm_id.clone();
    vm.name = "Other VM".into();
    inventory.discovery.vms.push(vm);
    inventory.enrolled.push(other.clone());
    inventory.managed.insert(selected.vm_id.clone(), None);
    inventory.managed.insert(other.vm_id.clone(), None);
    (s, selected, other)
}

#[test]
fn completed_profile_deletion_keeps_vm_and_other_profiles_without_host_changes() {
    let (mut s, selected, other) = profiles_for_deletion();
    s.view.inventory.as_mut().unwrap().discovery.vms[0].power = Power::Running;
    let vm_before = s.view.vm(&selected.vm_id).unwrap().clone();
    let other_document = s.committed.documents[&other.vm_id].clone();
    let other_vm = s.view.vm(&other.vm_id).unwrap().clone();
    let removed = save_record(&s);
    let mut retained = removed.clone();
    retained.target = other.clone();
    retained.initial = other_vm.clone();
    let retained_bytes = serde_json::to_value(&retained).unwrap();
    s.recoveries = vec![removed, retained];
    assert!(s.forget_target().is_ok());
    s.finish_profile_deletion(&selected.vm_id, Ok(())).unwrap();
    assert!(s.recovery_for_vm(&selected.vm_id).is_none());
    assert_eq!(s.recoveries.len(), 1);
    assert_eq!(
        serde_json::to_value(&s.recoveries[0]).unwrap(),
        retained_bytes
    );
    assert_eq!(s.view.selected.as_deref(), Some(selected.vm_id.as_str()));
    assert_eq!(
        serde_json::to_value(s.view.vm(&selected.vm_id).unwrap()).unwrap(),
        serde_json::to_value(&vm_before).unwrap()
    );
    assert_eq!(
        serde_json::to_value(s.view.vm(&other.vm_id).unwrap()).unwrap(),
        serde_json::to_value(&other_vm).unwrap()
    );
    assert!(!s.committed.documents.contains_key(&selected.vm_id));
    assert_eq!(
        s.committed.documents[&other.vm_id].target,
        other_document.target
    );
    assert_eq!(
        s.committed.documents[&other.vm_id].revision,
        other_document.revision
    );
    assert_eq!(s.view.configuration.targets, vec![other.clone()]);
    let inventory = s.view.inventory.as_ref().unwrap();
    assert_eq!(inventory.enrolled, vec![other.clone()]);
    assert!(!inventory.managed.contains_key(&selected.vm_id));
    assert!(inventory.managed.contains_key(&other.vm_id));
    assert!(s.view.needs_readback);
    let candidate = s.target(&selected.vm_id).unwrap();
    assert!(!candidate.enabled);
    assert!(!s.enrolled(&candidate));
    assert!(s.view.eligibility(&candidate).is_err());
    assert!(s.forget_target().is_err());
    // A fresh discovery keeps the VM available for new setup, with no old history.
    let fresh = s.view.inventory.clone().unwrap();
    s.refresh(fresh);
    assert!(s.editor_eligibility(&selected.vm_id).is_ok());
    assert!(!s.enrolled(&s.target(&selected.vm_id).unwrap()));
}

#[test]
fn failed_or_partial_deletion_keeps_cache_unknown_and_retains_recovery() {
    let (mut s, selected, other) = profiles_for_deletion();
    let vm_before = s.view.vm(&selected.vm_id).unwrap().clone();
    s.recoveries = vec![save_record(&s)];
    assert!(
        s.finish_profile_deletion(
            &selected.vm_id,
            Err("history deletion failed after settings removal".into())
        )
        .unwrap_err()
        .contains("history deletion failed")
    );
    // Cached records are not claimed to still exist in the protected store.
    assert!(s.committed.documents.contains_key(&selected.vm_id));
    assert!(s.committed.documents.contains_key(&other.vm_id));
    assert!(s.view.needs_readback);
    assert!(!s.recoveries.is_empty());
    assert_eq!(
        serde_json::to_value(s.view.vm(&selected.vm_id).unwrap()).unwrap(),
        serde_json::to_value(&vm_before).unwrap()
    );
    assert!(
        s.forget_target().is_ok(),
        "a new read-only review remains reachable; the worker binds any interrupted deletion"
    );
    assert!(
        s.selected_actions(0, &allocation_values(&selected))
            .recovery
    );
}

#[test]
fn delete_profile_never_stages_host_removal_or_power_changes_for_active_support() {
    let (mut s, selected, _) = profiles_for_deletion();
    s.view.inventory.as_mut().unwrap().discovery.vms[0].gpus = vec![selected.gpu_interface.clone()];
    s.view.inventory.as_mut().unwrap().discovery.vms[0].power = Power::Off;
    let vm_before = s.view.vm(&selected.vm_id).unwrap().clone();
    let target = s.forget_target().unwrap();
    assert_eq!(target.vm_id, selected.vm_id);
    assert_eq!(target.gpu_interface, selected.gpu_interface);
    assert!(!target.enabled);
    assert_eq!(
        serde_json::to_value(s.view.vm(&selected.vm_id).unwrap()).unwrap(),
        serde_json::to_value(&vm_before).unwrap()
    );
    assert!(s.view.draft.is_none());
    assert!(s.committed.documents.contains_key(&selected.vm_id));
}

#[test]
fn profile_deletion_review_remains_available_during_power_changes_and_recovery() {
    for scenario in 0..6 {
        let mut s = state();
        let vm_id = id(&s).to_owned();
        s.view
            .inventory
            .as_mut()
            .unwrap()
            .managed
            .insert(vm_id.clone(), None);
        match scenario {
            0 => s.view.inventory.as_mut().unwrap().discovery.vms[0].power = Power::Other(32769),
            1 => s.view.inventory.as_mut().unwrap().discovery.vms[0].power = Power::Running,
            2 => s.recoveries = vec![save_record(&s)],
            3 => {
                s.view.inventory.as_mut().unwrap().managed.remove(&vm_id);
            }
            4 => s.view.inventory.as_mut().unwrap().enrolled.clear(),
            _ => s.view.needs_readback = true,
        }
        assert_eq!(s.forget_target().is_ok(), scenario != 4);
        assert_eq!(
            s.selected_actions(0, &allocation_values(&s.target(&vm_id).unwrap()))
                .forget_allowed,
            scenario != 4
        );
    }
}
