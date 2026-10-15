use super::*;
use std::collections::BTreeMap;

const VM: &str = "12345678-1234-1234-1234-123456789abc";
const OTHER: &str = "12345678-1234-1234-1234-123456789abd";

fn pending() -> PendingPair {
    let pair = PairReview {
        vm_id: VM.into(),
        gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#fixture\GPUPARAV".into(),
        expected: Revision::of(b"old policy"),
        operator_sid: "S-1-5-21-1".into(),
    };
    let approved = RemovalReview {
        pair: pair.clone(),
        configurations: BTreeMap::from([
            (VM.into(), Revision::of(b"disabled intent")),
            (OTHER.into(), Revision::of(b"other intent")),
        ]),
        journals: BTreeMap::from([
            (VM.into(), Revision::of(b"released history")),
            (OTHER.into(), Revision::of(b"other history")),
        ]),
    };
    PendingPair {
        operation_id: "0123456789abcdef0123456789abcdef".into(),
        review: pair,
        resulting: Revision::of(b"retained policy"),
        removal: Some(RemovalState {
            approved,
            configuration_removed: false,
            journal_removed: false,
        }),
    }
}

struct Fake {
    observed: RemovalSnapshot,
    saved: Option<PendingPair>,
    calls: usize,
    fail: Option<usize>,
    mutations: Vec<&'static str>,
}
impl Fake {
    fn new(pending: &PendingPair) -> Self {
        let state = pending.removal.as_ref().unwrap();
        Self {
            observed: RemovalSnapshot {
                policy: pending.review.expected.clone(),
                configurations: state.approved.configurations.clone(),
                journals: state.approved.journals.clone(),
            },
            saved: None,
            calls: 0,
            fail: None,
            mutations: vec![],
        }
    }
    fn step(&mut self) -> Result<(), String> {
        self.calls += 1;
        if self.fail == Some(self.calls) {
            return Err("injected denial/interruption/readback failure".into());
        }
        Ok(())
    }
}
impl RemovalBackend for Fake {
    fn snapshot(&mut self) -> Result<RemovalSnapshot, String> {
        self.step()?;
        Ok(RemovalSnapshot {
            policy: self.observed.policy.clone(),
            configurations: self.observed.configurations.clone(),
            journals: self.observed.journals.clone(),
        })
    }
    fn hold(&mut self, pending: &PendingPair) -> Result<(), String> {
        self.step()?;
        self.saved = Some(serde_json::from_str(&serde_json::to_string(pending).unwrap()).unwrap());
        Ok(())
    }
    fn remove_configuration(&mut self, expected: &Revision) -> Result<(), String> {
        self.step()?;
        assert!(self.saved.is_some());
        let actual = &self.observed.configurations[VM];
        assert!(actual == expected || *actual == Revision::Missing);
        self.observed
            .configurations
            .insert(VM.into(), Revision::Missing);
        self.mutations.push("selected configuration");
        Ok(())
    }
    fn remove_journal(&mut self, expected: &Revision) -> Result<(), String> {
        self.step()?;
        assert!(self.saved.is_some());
        let actual = &self.observed.journals[VM];
        assert!(actual == expected || *actual == Revision::Missing);
        self.observed.journals.insert(VM.into(), Revision::Missing);
        self.mutations.push("selected history");
        Ok(())
    }
    fn revoke(&mut self, review: &PairReview, resulting: &Revision) -> Result<(), String> {
        self.step()?;
        assert!(self.saved.is_some());
        if self.observed.policy == *resulting {
            return Ok(());
        }
        assert_eq!(self.observed.policy, review.expected);
        assert_eq!(self.observed.configurations[VM], Revision::Missing);
        assert_eq!(self.observed.journals[VM], Revision::Missing);
        self.observed.policy = resulting.clone();
        self.mutations.push("exact pairing");
        Ok(())
    }
}

#[test]
fn profile_admission_allows_active_support_but_preserves_power_identity_and_recovery_checks() {
    use crate::{
        model::{Power, Settings, VmState},
        workflow::Journal,
    };
    let target = pending().review.target().unwrap();
    let settings = Settings {
        low_mmio: 128,
        high_mmio: 512,
        cache_types: false,
        automatic_checkpoints: true,
    };
    let vm = VmState {
        vm_id: target.vm_id.clone(),
        name: "Selected VM".into(),
        power: Power::Off,
        generation: 2,
        gpus: vec![],
        settings: settings.clone(),
        vram: None,
        compute: None,
        encode: None,
        decode: None,
    };
    let journal = Journal {
        schema: 1,
        vm_id: target.vm_id.clone(),
        gpu_interface: target.gpu_interface.clone(),
        original: settings.clone(),
        applied: settings,
        prepared: None,
        pending: false,
        restore_power: Power::Off,
        verification_only: false,
        last_verified: None,
        latest_check: Some(crate::workflow::GuestCheck::PayloadMismatch),
        preparation_started: None,
        check_restored: false,
        settings_released: true,
    };
    assert!(validate_inactive_profile(&target, &vm, Some(&journal), Some(&target)).is_ok());
    assert!(validate_inactive_profile(&target, &vm, None, None).is_ok());
    let mut active_vm = vm.clone();
    active_vm.gpus.push(target.gpu_interface.clone());
    let mut active_history = journal.clone();
    active_history.settings_released = false;
    let mut active_saved = target.clone();
    active_saved.enabled = true;
    assert!(
        validate_inactive_profile(
            &target,
            &active_vm,
            Some(&active_history),
            Some(&active_saved)
        )
        .is_ok()
    );
    for change in 0..8 {
        let mut current = vm.clone();
        let mut history = journal.clone();
        let mut saved = target.clone();
        match change {
            0 => current.power = Power::Running,
            1 => current.power = Power::Other(32769),
            2 => history.pending = true,
            3 => history.gpu_interface.push_str("changed"),
            4 => saved.gpu_interface.push_str("changed"),
            5 => history.schema = 2,
            6 => history.vm_id = OTHER.into(),
            _ => current.vm_id = OTHER.into(),
        }
        assert!(
            validate_inactive_profile(&target, &current, Some(&history), Some(&saved)).is_err()
        );
    }
}

#[test]
fn removal_preserves_other_vm_sharing_gpu_and_all_retained_intent() {
    let mut pending = pending();
    let mut selected = pending.review.target().unwrap();
    let mut other = selected.clone();
    other.vm_id = OTHER.into();
    other.enabled = true;
    other.compute = Some(crate::model::Allocation {
        minimum: 10,
        optimal: 50,
        maximum: 90,
    });
    selected.enabled = true; // Enrollment intent is authority, not observed attachment.
    let retained = pending
        .review
        .remove(
            &[selected, other.clone()],
            &pending.review.expected,
            &pending.review.operator_sid,
        )
        .unwrap();
    assert_eq!(retained, [other]);
    let mut backend = Fake::new(&pending);
    publish_removal(&mut backend, &mut pending).unwrap();
    assert_eq!(
        backend.mutations,
        [
            "selected configuration",
            "selected history",
            "exact pairing"
        ]
    );
    assert_eq!(
        backend.observed.configurations[OTHER],
        Revision::of(b"other intent")
    );
    assert_eq!(
        backend.observed.journals[OTHER],
        Revision::of(b"other history")
    );
    assert_eq!(backend.observed.policy, pending.resulting);
}

#[test]
fn denial_or_interruption_at_every_boundary_retains_exact_resumable_scope() {
    for failure in 1..=13 {
        let mut pending = pending();
        let mut backend = Fake::new(&pending);
        backend.fail = Some(failure);
        assert!(
            publish_removal(&mut backend, &mut pending).is_err(),
            "boundary {failure}"
        );
        if let Some(saved) = &backend.saved {
            pending = saved.clone();
        } else {
            assert!(backend.mutations.is_empty());
        }
        backend.fail = None;
        publish_removal(&mut backend, &mut pending).unwrap();
        assert_eq!(backend.observed.policy, pending.resulting);
        assert_eq!(backend.observed.configurations[VM], Revision::Missing);
        assert_eq!(backend.observed.journals[VM], Revision::Missing);
        assert_eq!(
            backend.observed.configurations[OTHER],
            Revision::of(b"other intent")
        );
        assert_eq!(
            backend.observed.journals[OTHER],
            Revision::of(b"other history")
        );
    }
}

#[test]
fn changed_records_authority_and_recreated_completed_deletions_stay_blocked() {
    for scenario in 0..5 {
        let mut pending = pending();
        let mut backend = Fake::new(&pending);
        match scenario {
            0 => {
                backend.observed.policy = Revision::of(b"unrelated policy");
            }
            1 => {
                backend
                    .observed
                    .configurations
                    .insert(OTHER.into(), Revision::of(b"externally edited other VM"));
            }
            2 => {
                backend
                    .observed
                    .journals
                    .insert(VM.into(), Revision::of(b"new history"));
            }
            3 => {
                backend
                    .observed
                    .configurations
                    .insert("another-record".into(), Revision::Missing);
            }
            _ => {
                pending.removal.as_mut().unwrap().configuration_removed = true;
            }
        }
        assert!(publish_removal(&mut backend, &mut pending).is_err());
        assert!(backend.mutations.is_empty());
    }
}

#[test]
fn absent_records_last_pair_and_stale_or_already_missing_authority_are_explicit() {
    let mut pending = pending();
    assert!(
        pending
            .review
            .remove(&[], &pending.review.expected, &pending.review.operator_sid)
            .is_err()
    );
    let selected = pending.review.target().unwrap();
    assert!(
        pending
            .review
            .remove(
                std::slice::from_ref(&selected),
                &Revision::of(b"changed"),
                &pending.review.operator_sid
            )
            .is_err()
    );
    assert!(
        pending
            .review
            .remove(
                std::slice::from_ref(&selected),
                &pending.review.expected,
                "wrong operator"
            )
            .is_err()
    );
    assert!(
        pending
            .review
            .remove(
                &[selected],
                &pending.review.expected,
                &pending.review.operator_sid
            )
            .unwrap()
            .is_empty()
    );
    let state = pending.removal.as_mut().unwrap();
    state
        .approved
        .configurations
        .insert(VM.into(), Revision::Missing);
    state.approved.journals.insert(VM.into(), Revision::Missing);
    let mut backend = Fake::new(&pending);
    publish_removal(&mut backend, &mut pending).unwrap();
    let mut new = pending.review.clone();
    new.expected = pending.resulting.clone();
    let authority = new.append(&[], &new.expected, &new.operator_sid).unwrap();
    assert_eq!(authority, [new.target().unwrap()]);
    assert_eq!(backend.observed.journals[VM], Revision::Missing);
}
