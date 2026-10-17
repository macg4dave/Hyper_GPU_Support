use super::*;
use crate::{
    model::ResourceCapability,
    workflow::{
        Backend,
        test_support::{fake, target},
    },
};

fn bundle(mode: Mode) -> Bundle {
    let mut backend = fake();
    let target = target();
    let vm = backend.inspect(&target).unwrap();
    let gpu = backend.gpu(&target).unwrap();
    let committed = StoreSnapshot::default();
    Bundle {
        inventory: Inventory {
            discovery: crate::model::Discovery {
                vms: vec![vm],
                gpus: vec![gpu],
                guest_os: Default::default(),
                issues: vec![],
            },
            managed: Default::default(),
            enrolled: vec![target.clone()],
        },
        committed: committed.clone(),
        recovery: vec![],
        import: None,
        enrollment: None,
        worker_available: true,
        provenance: Provenance {
            schema: 1,
            authority: Revision::of(b"policy"),
            configuration: Revision::of(&serde_json::to_vec(&committed).unwrap()),
            mode,
            selected: (mode == Mode::Completion).then_some(target),
        },
    }
}

#[test]
fn poll_merges_display_by_identity_and_invalidates_removed_or_changed_metadata() {
    let mut prior = bundle(Mode::Full).inventory;
    prior.discovery.gpus[0].name = "Physical GPU".into();
    prior.discovery.gpus[0].physical_vram_bytes = Some(8_000);
    let id = prior.discovery.vms[0].vm_id.clone();
    prior
        .discovery
        .guest_os
        .insert(id.clone(), "Windows".into());
    let mut poll = bundle(Mode::Poll);
    poll.inventory.discovery.gpus[0].vram = ResourceCapability::Unknown;
    let merged = poll.inventory_with_display(Some(&prior)).unwrap();
    assert_eq!(merged.discovery.gpus[0].name, "Physical GPU");
    assert_eq!(merged.discovery.gpus[0].physical_vram_bytes, Some(8_000));
    assert_eq!(merged.discovery.gpus[0].vram, ResourceCapability::Unknown);
    assert_eq!(merged.discovery.guest_os.get(&id).unwrap(), "Windows");
    poll.inventory.discovery.gpus[0].driver_version = "changed".into();
    let merged = poll.inventory_with_display(Some(&prior)).unwrap();
    assert_eq!(merged.discovery.gpus[0].physical_vram_bytes, None);
    poll.inventory.discovery.vms.clear();
    poll.inventory.discovery.gpus.clear();
    let removed = poll.inventory_with_display(Some(&prior)).unwrap();
    assert!(
        removed.discovery.vms.is_empty()
            && removed.discovery.gpus.is_empty()
            && removed.discovery.guest_os.is_empty()
    );
    let full = bundle(Mode::Full)
        .inventory_with_display(Some(&prior))
        .unwrap();
    assert!(full.discovery.guest_os.is_empty());
    assert_eq!(full.discovery.gpus[0].physical_vram_bytes, None);
}

#[test]
fn additions_and_reordering_do_not_transfer_display_to_another_identity() {
    let mut previous = bundle(Mode::Full).inventory;
    previous.discovery.gpus[0].name = "First GPU".into();
    let mut second = previous.discovery.gpus[0].clone();
    second.interface.push_str("-second");
    second.name = "Second GPU".into();
    previous.discovery.gpus.push(second.clone());
    let mut poll = bundle(Mode::Poll);
    second.name = "unobserved label".into();
    poll.inventory.discovery.gpus.insert(0, second);
    let mut added = poll.inventory.discovery.vms[0].clone();
    added.vm_id = "22222222-2222-2222-2222-222222222222".into();
    poll.inventory.discovery.vms.insert(0, added);
    let merged = poll.inventory_with_display(Some(&previous)).unwrap();
    assert_eq!(merged.discovery.gpus[0].name, "Second GPU");
    assert_eq!(merged.discovery.gpus[1].name, "First GPU");
    assert_eq!(
        merged.discovery.vms[1].vm_id,
        previous.discovery.vms[0].vm_id
    );
    assert_eq!(merged.discovery.vms.len(), 2);
}

#[test]
fn completion_replaces_selected_facts_preserves_unrelated_rows_and_uses_fresh_authority() {
    let mut previous = bundle(Mode::Full).inventory;
    let mut other = previous.discovery.vms[0].clone();
    other.vm_id = "22222222-2222-2222-2222-222222222222".into();
    previous.discovery.vms.push(other.clone());
    let mut completion = bundle(Mode::Completion);
    completion.inventory.discovery.vms[0].settings.high_mmio += 10;
    completion.inventory.enrolled.clear();
    let merged = completion.inventory_with_display(Some(&previous)).unwrap();
    assert_eq!(merged.discovery.vms.len(), 2);
    assert_eq!(merged.discovery.vms[1].operational(), other.operational());
    assert_ne!(
        merged.discovery.vms[0].operational(),
        previous.discovery.vms[0].operational()
    );
    assert!(merged.enrolled.is_empty());
    completion.inventory.discovery.vms.clear();
    let missing = completion.inventory_with_display(Some(&previous)).unwrap();
    assert_eq!(
        missing.discovery.vms.len(),
        1,
        "missing selected read cannot restore old selected facts"
    );
    assert_eq!(missing.discovery.vms[0].vm_id, other.vm_id);
}

#[test]
fn revisions_duplicates_and_mismatched_completion_scope_fail_before_publication() {
    let mut snapshot = bundle(Mode::Poll);
    snapshot
        .committed
        .errors
        .insert("store".into(), "unavailable".into());
    assert!(snapshot.validate().is_err());
    let mut snapshot = bundle(Mode::Poll);
    snapshot
        .inventory
        .discovery
        .vms
        .push(snapshot.inventory.discovery.vms[0].clone());
    assert!(snapshot.validate().is_err());
    let mut snapshot = bundle(Mode::Completion);
    snapshot.provenance.selected.as_mut().unwrap().vm_id =
        "22222222-2222-2222-2222-222222222222".into();
    assert!(snapshot.validate().is_err());
}
