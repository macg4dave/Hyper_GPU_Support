use super::*;

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
        reopened.recovery = held.then_some(record);
        let recovery_before = serde_json::to_value(&reopened.recovery).unwrap();
        let inventory_before = serde_json::to_value(&next).unwrap();
        reopened.refresh(next.clone());
        assert_eq!(reopened.view.selected.as_deref(), Some(selected.as_str()));
        assert_eq!(
            reopened.view.vm(&selected).unwrap().name,
            "Renamed selected VM"
        );
        assert_eq!(
            serde_json::to_value(&reopened.recovery).unwrap(),
            recovery_before
        );
        assert_eq!(
            serde_json::to_value(reopened.view.inventory.as_ref().unwrap()).unwrap(),
            inventory_before
        );
        assert!(reopened.view.draft.is_none());
        assert!(reopened.approved_plan.is_none());
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
            serde_json::to_value(&reopened.recovery).unwrap(),
            recovery_before
        );
        next.discovery.vms.clear();
        reopened.refresh(next);
        assert!(reopened.view.selected.is_none());
        assert_eq!(
            serde_json::to_value(&reopened.recovery).unwrap(),
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
