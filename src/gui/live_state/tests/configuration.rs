use super::*;

#[test]
fn committed_conflict_preserves_draft_but_refreshes_other_intent_and_inventory() {
    let mut s = state();
    let vm_id = id(&s).to_owned();
    let original = s.target(&vm_id).unwrap();
    s.load_committed(committed(vec![original.clone()], "original"))
        .unwrap();
    let draft = s.draft(&vm_id, false, 0, &values("", "", "")).unwrap();
    s.view.draft = Some(draft.clone());
    s.load_committed(committed(vec![original.clone()], "original"))
        .unwrap();

    let mut updated = original.clone();
    updated.vram = Some(Allocation {
        minimum: 10,
        optimal: 50,
        maximum: 100,
    });
    let mut other = original;
    other.vm_id = "12345678-1234-1234-1234-000000000002".into();
    let latest = committed(vec![updated.clone(), other.clone()], "external edit");
    assert!(s.load_committed(latest.clone()).is_err());
    let mut fresh = inventory();
    fresh.discovery.vms[0].name = "Fresh VM name".into();
    s.refresh(fresh);
    assert_eq!(s.view.vm(&vm_id).unwrap().name, "Fresh VM name");
    assert!(!s.view.needs_readback);
    assert_eq!(s.view.draft, Some(draft));
    assert_eq!(s.target(&vm_id).unwrap(), updated);
    assert_eq!(s.target(&other.vm_id).unwrap(), other);
    assert!(s.saved_desired_text(&vm_id).contains("10 / 50 / 100"));
    assert!(s.draft(&vm_id, true, 0, &values("", "", "")).is_err());
    // A second Refresh must not silently accept the conflicting revision.
    assert!(s.load_committed(latest).is_err());
    s.discard();
    assert!(s.view.draft.is_none());
    assert!(s.editor_eligibility(&vm_id).is_ok());
}

#[test]
fn unreadable_committed_intent_is_unknown_and_draft_stays_blocked_after_repair() {
    let mut s = state();
    let vm_id = id(&s).to_owned();
    let target = s.target(&vm_id).unwrap();
    let original = committed(vec![target.clone()], "original");
    s.load_committed(original.clone()).unwrap();
    s.view.draft = Some(target.clone());
    let mut unreadable = hyper_gpu_support::configuration_store::StoreSnapshot::default();
    unreadable.insert(&format!("{vm_id}.toml"), Err("access denied".into()));
    assert!(s.load_committed(unreadable).is_err());
    assert!(s.saved_desired_text(&vm_id).contains("unknown"));
    assert!(!s.saved_desired_text(&vm_id).contains("No saved"));
    assert_eq!(s.view.draft, Some(target));
    assert!(s.load_committed(original).is_err());
    assert!(s.editor_eligibility(&vm_id).is_err());
    s.discard();
    assert!(s.editor_eligibility(&vm_id).is_ok());
}

#[test]
fn removed_committed_file_does_not_become_a_new_draft_baseline_until_discard() {
    let mut s = state();
    let vm_id = id(&s).to_owned();
    let target = s.target(&vm_id).unwrap();
    s.load_committed(committed(vec![target.clone()], "original"))
        .unwrap();
    s.view.draft = Some(target.clone());
    assert!(s.load_committed(Default::default()).is_err());
    assert!(s.saved_desired_text(&vm_id).contains("No saved"));
    assert_eq!(s.view.draft, Some(target));
    assert!(s.draft(&vm_id, false, 0, &values("", "", "")).is_err());
    s.discard();
    assert!(s.draft(&vm_id, false, 0, &values("", "", "")).is_ok());
}
