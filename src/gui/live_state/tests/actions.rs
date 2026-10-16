use super::*;

#[test]
fn selected_actions_allow_profile_deletion_with_attached_support_at_either_power() {
    let mut s = action_state();
    let values = allocation_values(&s.target(id(&s)).unwrap());
    for power in [Power::Off, Power::Running] {
        s.view.inventory.as_mut().unwrap().discovery.vms[0].power = power.clone();
        let actions = s.selected_actions(0, &values);
        assert!(actions.verify_allowed);
        assert!(actions.repair_allowed);
        assert!(actions.remove_allowed);
        assert!(actions.forget_allowed);
        assert!(!actions.recovery);
    }
    s.view.inventory.as_mut().unwrap().discovery.vms[0].power = Power::Off;
    s.view.inventory.as_mut().unwrap().discovery.vms[0]
        .gpus
        .clear();
    let actions = s.selected_actions(0, &values);
    assert!(actions.verify.contains("not attached"));
    assert!(actions.repair_allowed);
    assert!(actions.remove_allowed);
    assert!(actions.forget_allowed);
    // Inactive profile cleanup does not require the physical GPU to be available.
    s.view.inventory.as_mut().unwrap().discovery.gpus.clear();
    assert!(s.selected_actions(-1, &values).forget_allowed);
}

#[test]
fn action_explanations_keep_invalid_drafts_and_other_profiles() {
    let mut s = action_state();
    let target = s.target(id(&s)).unwrap();
    s.view.draft = Some(target.clone());
    let mut other = target.clone();
    other.vm_id = "87654321-4321-4321-4321-cba987654321".into();
    s.view.configuration.targets.push(other.clone());
    let mut values = allocation_values(&target);
    values[0] = "invalid".into();
    let actions = s.selected_actions(0, &values);
    assert!(actions.verify.contains("pending edits"));
    assert!(actions.forget_allowed);
    assert!(actions.forget.is_empty());
    assert!(!actions.repair_allowed);
    assert!(!actions.remove_allowed);
    assert_eq!(s.view.draft, Some(target));
    assert_eq!(s.view.configuration.targets, vec![other]);
    assert!(s.approved_plan.is_none());
    assert!(s.removal_review.is_none());
    assert!(s.verification_plan.is_none());
    assert_eq!(values[0], "invalid");
    s.discard();
    let values = allocation_values(&s.target(id(&s)).unwrap());
    assert!(s.selected_actions(0, &values).verify_allowed);
}

#[test]
fn action_reasons_change_with_selection_power_and_failed_observation() {
    let mut s = action_state();
    let vm_id = id(&s).to_owned();
    let values = allocation_values(&s.target(&vm_id).unwrap());
    s.view.selected = None;
    assert!(
        s.selected_actions(0, &values)
            .verify
            .contains("Select a VM")
    );
    s.view.selected = Some("missing-vm".into());
    assert!(
        s.selected_actions(0, &values)
            .remove
            .contains("unavailable")
    );
    s.view.selected = Some(vm_id.clone());
    s.view.inventory.as_mut().unwrap().discovery.vms[0].generation = 1;
    assert!(
        s.selected_actions(0, &values)
            .repair
            .contains("Generation 2")
    );
    s.view.inventory.as_mut().unwrap().discovery.vms[0].generation = 2;
    s.view.inventory.as_mut().unwrap().discovery.vms[0].power = Power::Other(4);
    assert!(
        s.selected_actions(0, &values)
            .repair
            .contains("stable state")
    );
    s.view.inventory.as_mut().unwrap().discovery.vms[0].power = Power::Off;
    s.view.needs_readback = true;
    assert!(
        s.selected_actions(0, &values)
            .verify
            .contains("Use Refresh")
    );
    let fresh = s.view.inventory.clone().unwrap();
    s.refresh(fresh);
    assert!(s.selected_actions(0, &values).verify_allowed);
    s.committed
        .errors
        .insert(format!("{vm_id}.toml"), "fixture read denied".into());
    let actions = s.selected_actions(0, &values);
    assert!(!actions.verify_allowed);
    assert!(!actions.repair_allowed);
    assert!(!actions.remove_allowed);
    assert!(
        actions.forget_allowed,
        "app-record review does not require host observation or an editable profile"
    );
}

#[test]
fn unavailable_managed_history_requests_refresh_without_inventing_recovery() {
    let mut s = action_state();
    let vm_id = id(&s).to_owned();
    let target = s.target(&vm_id).unwrap();
    s.view.draft = Some(target.clone());
    s.view.inventory.as_mut().unwrap().managed.remove(&vm_id);
    let actions = s.selected_actions(0, &allocation_values(&target));
    assert!(!actions.recovery);
    assert!(
        actions.forget_allowed,
        "profile deletion remains separately reviewable"
    );
    for action in ["verify", "repair", "remove-gpu"] {
        assert!(actions.reason(action).unwrap().contains("Use Refresh"));
    }
    assert!(s.recovery.is_none());
    assert_eq!(s.view.draft, Some(target));
    assert!(
        !s.view
            .inventory
            .as_ref()
            .unwrap()
            .managed
            .contains_key(&vm_id)
    );
}

#[test]
fn recovery_preserves_another_vms_draft_without_claiming_its_actions_need_recovery() {
    let mut s = action_state();
    let target = s.target(id(&s)).unwrap();
    s.recovery = Some(save_record(&s));
    s.view.draft = Some(target.clone());
    s.view.selected = Some("another-vm".into());
    let actions = s.selected_actions(0, &allocation_values(&target));
    assert!(!actions.recovery);
    assert!(!actions.remove.contains("Review recovery"));
    assert!(s.recovery.is_some());
    assert_eq!(s.view.draft, Some(target));
}

#[test]
fn unconfigured_actions_offer_setup_without_authorizing_the_pair() {
    let mut s = action_state();
    s.view.inventory.as_mut().unwrap().enrolled.clear();
    s.view.inventory.as_mut().unwrap().discovery.vms[0]
        .gpus
        .clear();
    let target = s.target(id(&s)).unwrap();
    let actions = s.selected_actions(0, &allocation_values(&target));
    for reason in [&actions.verify, &actions.repair, &actions.remove] {
        assert!(reason.contains("first-time setup"));
    }
    assert!(!actions.forget_allowed);
    assert!(!actions.recovery);
    assert_eq!(s.fixed_gpu_index(), None);
    assert!(s.view.inventory.as_ref().unwrap().enrolled.is_empty());
    assert!(s.view.configuration.targets.is_empty());
}

#[test]
fn fixed_gpu_identity_survives_reordering_and_missing_device() {
    let mut s = action_state();
    let target = s.target(id(&s)).unwrap();
    let mut other = s.view.inventory.as_ref().unwrap().discovery.gpus[0].clone();
    other.interface = r"\\?\PCI#VEN_10DE&DEV_2D05#other\GPUPARAV".into();
    s.view
        .inventory
        .as_mut()
        .unwrap()
        .discovery
        .gpus
        .insert(0, other);
    assert_eq!(s.fixed_gpu_index(), Some(1));
    let values = allocation_values(&target);
    assert!(s.draft(id(&s), true, 0, &values).is_err());
    assert!(s.selected_actions(1, &values).repair_allowed);
    s.view.inventory.as_mut().unwrap().discovery.gpus.remove(1);
    assert_eq!(s.fixed_gpu_index(), Some(-1));
    assert!(!s.selected_actions(0, &values).repair_allowed);
    assert_eq!(s.target(id(&s)).unwrap(), target);
    assert!(s.view.draft.is_none());
}

#[test]
fn unsupported_preparation_does_not_hide_host_removal_or_inactive_cleanup() {
    let mut s = action_state();
    let values = allocation_values(&s.target(id(&s)).unwrap());
    s.view.inventory.as_mut().unwrap().discovery.gpus[0].preparation_supported = false;
    let actions = s.selected_actions(0, &values);
    assert!(actions.repair.contains("not supported"));
    assert!(actions.remove_allowed);
    s.view.inventory.as_mut().unwrap().discovery.vms[0]
        .gpus
        .clear();
    assert!(s.selected_actions(0, &values).forget_allowed);
    let vm_id = id(&s).to_owned();
    s.committed_conflict = Some(vm_id);
    let actions = s.selected_actions(0, &values);
    assert!(actions.remove.contains("Draft preserved"));
    assert!(actions.forget_allowed);
    assert!(s.committed_conflict.is_some());
}

#[test]
fn partial_inventory_blocks_only_actions_for_the_affected_vm() {
    let mut s = action_state();
    let vm_id = id(&s).to_owned();
    let values = allocation_values(&s.target(&vm_id).unwrap());
    let issue = hyper_gpu_support::reporting::Diagnostic::observation;
    s.view
        .inventory
        .as_mut()
        .unwrap()
        .discovery
        .issues
        .push(issue(
            "VM settings",
            Some("other-vm".into()),
            "access denied",
        ));
    assert!(s.selected_actions(0, &values).verify_allowed);
    s.view
        .inventory
        .as_mut()
        .unwrap()
        .discovery
        .issues
        .push(issue("VM settings", Some(vm_id), "access denied"));
    let actions = s.selected_actions(0, &values);
    assert!(
        actions.forget_allowed,
        "profile deletion remains separately reviewable"
    );
    for action in ["verify", "repair", "remove-gpu"] {
        assert!(actions.reason(action).unwrap().contains("Correct access"));
    }
    assert!(!actions.recovery);
    assert_eq!(s.view.inventory.as_ref().unwrap().discovery.issues.len(), 2);
}
