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
                    if power == Power::Off || attached {
                        2
                    } else {
                        1
                    }
                );
                assert_eq!(
                    backend.os_checks, 0,
                    "no Windows-version probe during execution"
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
fn ordinary_prepared_changes_share_the_exact_sequence_without_guest_work() {
    for enabled in [true, false] {
        for changed in [true, false] {
            for power in [Power::Off, Power::Running] {
                let mut backend = fake();
                let mut wanted = extended_target();
                wanted.vram = Some(Allocation {
                    minimum: 10,
                    optimal: 50,
                    maximum: 90,
                });
                apply(&mut backend, &wanted).unwrap();
                wanted.enabled = enabled;
                if changed {
                    wanted.vram.as_mut().unwrap().maximum -= 1;
                }
                // Omitted categories must not acquire a write on a retained adapter.
                wanted.compute = None;
                backend.state.power = power;
                backend.events.clear();
                backend.allocation_calls.clear();
                backend.work = Counts::default();
                let (plan, admitted) = prepare_apply(&mut backend, &wanted, false).unwrap();
                assert!(!plan.preview.credentials_required);
                assert!(!plan.preview.actions.contains(&PreviewAction::Verify));
                let result = apply_admitted(&mut backend, admitted).unwrap();
                assert_eq!(backend.events, trace(&plan.preview));
                assert!(!result.prepared && !result.verified);
                assert_eq!(result.effective.power, Power::Off);
                assert_eq!(backend.work.get(Counter::PayloadPasses), u64::from(enabled));
                assert!(!backend.allocation_calls.contains(&Resource::Compute));
                assert_eq!(
                    backend.allocation_calls.len(),
                    usize::from(enabled && changed)
                );
            }
        }
    }
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
    let plan = prepare_apply(&mut backend, &target, true)
        .map(|(plan, _)| plan)
        .unwrap();
    apply_approved(&mut backend, &plan).unwrap();
    assert_eq!(backend.allocation_calls, Resource::ALL);
}
