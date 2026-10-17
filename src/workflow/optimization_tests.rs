//! R2 regressions for preparation lifetime, lifecycle and category scope.
use super::test_support::{extended_target, fake, target, trace};
use super::*;
use crate::work_counts::{Counter, Counts};

#[test]
fn prepared_apply_uses_one_admission_pass_and_independent_final_pass() {
    for reinstall in [false, true] {
        for attached in [false, true] {
            for power in [Power::Off, Power::Running] {
                let mut backend = fake();
                let target = extended_target();
                backend.state.power = power.clone();
                if attached {
                    backend.state.gpus = vec![target.gpu_interface.clone()];
                }
                let (plan, admitted) = prepare_apply(&mut backend, &target, reinstall).unwrap();
                assert_eq!(backend.payload_reads, 1);
                assert_eq!(backend.journal_writes, 0);
                assert!(backend.events.is_empty());
                assert_eq!(backend.os_checks, 0, "no guest access during preparation");
                let result = apply_admitted(&mut backend, admitted).unwrap();
                assert_eq!(backend.events, trace(&plan.preview));
                assert!(result.prepared && result.verified);
                assert_eq!(backend.state.power, Power::Off);
                assert_eq!(backend.payload_reads, 1, "consumed token retains admission");
                assert_eq!(
                    backend.payload(&target).unwrap(),
                    plan.preparation.unwrap().digest
                );
                assert_eq!(
                    backend.payload_reads, 2,
                    "final full pass remains independent"
                );
                assert_eq!(
                    backend.work.get(Counter::PowerStarts),
                    if attached {
                        if power == Power::Off { 3 } else { 2 }
                    } else if power == Power::Off {
                        2
                    } else {
                        1
                    }
                );
                assert_eq!(
                    backend.os_check_traces[0],
                    if power == Power::Off {
                        vec!["start"]
                    } else {
                        vec![]
                    }
                );
            }
        }
    }
}

#[test]
fn prepared_verify_keeps_one_payload_until_consumption_and_final_authentication() {
    let mut backend = fake();
    let target = target();
    apply(&mut backend, &target).unwrap();
    backend.payload_reads = 0;
    backend.events.clear();
    let (plan, token) = prepare_verification(&mut backend, &target).unwrap();
    assert!(backend.events.is_empty());
    assert_eq!(backend.payload_reads, 1);
    assert!(verify_admitted(&mut backend, token).unwrap().verified);
    assert_eq!(backend.payload_reads, 1);
    assert_eq!(backend.payload(&target).unwrap(), plan.payload_digest);
    assert_eq!(backend.payload_reads, 2);
}

#[test]
fn prepared_mutable_drift_rejects_before_journal_or_effects() {
    for drift in 0..4 {
        let mut backend = fake();
        let (plan, token) = prepare_apply(&mut backend, &target(), false).unwrap();
        match drift {
            0 => backend.state.settings.high_mmio += 1,
            1 => backend.gpu_driver = "changed",
            2 => backend.state.gpus.push("other".into()),
            _ => {
                let mut other = fake();
                apply(&mut other, &target()).unwrap();
                backend.journal = other.journal;
            }
        }
        assert!(apply_admitted(&mut backend, token).is_err());
        assert!(backend.events.is_empty());
        assert_eq!(backend.journal_writes, 0);
        assert_eq!(backend.state.vm_id, plan.desired.vm_id);
    }
}

#[test]
fn ordinary_cli_prediction_never_hashes_missing_or_adverse_history() {
    for pending in [false, true] {
        let mut backend = fake();
        let target = target();
        if pending {
            apply(&mut backend, &target).unwrap();
            backend.journal.as_mut().unwrap().pending = true;
        }
        backend.payload_reads = 0;
        backend.events.clear();
        assert!(plan_if_ordinary(&mut backend, &target).unwrap().is_none());
        assert_eq!(backend.payload_reads, 0);
        assert!(backend.events.is_empty());
    }
    let mut backend = fake();
    let target = target();
    apply(&mut backend, &target).unwrap();
    backend.work = Counts::default();
    assert!(
        plan_if_ordinary(&mut backend, &target)
            .unwrap()
            .unwrap()
            .authentication_deferred
    );
    assert_eq!(backend.work.get(Counter::PayloadPasses), 0);
}

#[test]
fn retained_vram_change_skips_matching_categories_but_recreated_adapter_writes_all() {
    let mut backend = fake();
    let mut target = extended_target();
    target.vram = Some(Allocation {
        minimum: 0,
        optimal: 50,
        maximum: 90,
    });
    apply(&mut backend, &target).unwrap();
    backend.allocation_calls.clear();
    target.vram.as_mut().unwrap().maximum = 95;
    let plan = plan(&mut backend, &target).unwrap();
    assert_eq!(plan.preview.allocation, target.vram);
    assert!(
        plan.preview.compute.is_none()
            && plan.preview.encode.is_none()
            && plan.preview.decode.is_none()
    );
    apply_approved(&mut backend, &plan).unwrap();
    assert_eq!(backend.allocation_calls, [Resource::Vram]);
    backend.allocation_calls.clear();
    let plan = plan_reinstall(&mut backend, &target).unwrap();
    apply_approved(&mut backend, &plan).unwrap();
    assert_eq!(backend.allocation_calls, Resource::ALL);
}
