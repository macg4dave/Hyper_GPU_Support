//! Candidate comparisons against the frozen OPT-001 table in ENGINEERING.md.
//! Updated expectations name each removed cycle/category check; isolated doubles are
//! not provider queries, native writes, worker final authentication, or timings.
use super::test_support::{Fake, extended_target, fake, recovery_record, trace};
use super::{Backend, apply, apply_approved, plan, plan_preliminary, plan_reinstall};
use crate::model::{Allocation, Configuration, Power, Resource, Target};
use crate::work_counts::{Counter, Counts};

struct Expected {
    reads: [u64; 3], // VM, GPU, journal
    journals: u64,
    payloads: u64,
    power: [u64; 2],    // start, graceful shutdown
    adapters: [u64; 2], // attach, detach
    settings: u64,
    allocations: [u64; 4], // VRAM, Compute, Encode, Decode
}

fn assert_counts(name: &str, measured: Counts, expected: Expected) {
    let mut counts = Counts::default();
    for (counter, value) in [
        (Counter::VmReads, expected.reads[0]),
        (Counter::GpuReads, expected.reads[1]),
        (Counter::JournalReads, expected.reads[2]),
        (Counter::JournalWrites, expected.journals),
        (Counter::PayloadPasses, expected.payloads),
        (Counter::PowerStarts, expected.power[0]),
        (Counter::PowerShutdowns, expected.power[1]),
        (Counter::AttachWrites, expected.adapters[0]),
        (Counter::DetachWrites, expected.adapters[1]),
        (Counter::SettingsWrites, expected.settings),
    ] {
        counts.add(counter, value);
    }
    for (resource, value) in Resource::ALL.into_iter().zip(expected.allocations) {
        counts.add(crate::work_counts::allocation(resource), value);
    }
    assert_eq!(
        measured, counts,
        "candidate compared with original isolated baseline: {name}"
    );
}

fn requested() -> Target {
    let mut target = extended_target();
    target.vram = Some(Allocation {
        minimum: 10,
        optimal: 50,
        maximum: 90,
    });
    target
}

fn established() -> (Fake, Target) {
    let mut backend = fake();
    let target = requested();
    apply(&mut backend, &target).unwrap();
    assert!(backend.journal.as_ref().unwrap().setup_eligible());
    backend.work = Counts::default();
    backend.events.clear();
    (backend, target)
}

#[test]
fn setup_removes_one_unattached_boot_cycle_against_original_counts() {
    for (power, reads, cycles) in [
        (Power::Off, [39, 36, 42], [2, 2]),
        (Power::Running, [37, 34, 40], [1, 2]),
    ] {
        let mut backend = fake();
        backend.state.power = power;
        let target = requested();
        let reviewed = plan(&mut backend, &target).unwrap();
        let result = apply_approved(&mut backend, &reviewed).unwrap();
        assert_eq!(backend.events, trace(&reviewed.preview));
        assert!(result.prepared && result.verified);
        assert_eq!(result.effective.power, Power::Off);
        assert!(!backend.journal.as_ref().unwrap().pending);
        assert_counts(
            "setup",
            backend.work,
            Expected {
                reads,
                journals: 7,
                payloads: 2,
                power: cycles,
                adapters: [1, 0],
                settings: 1,
                allocations: [1; 4],
            },
        );
    }
}

#[test]
fn retained_category_reduction_and_attached_reinstall_compared_with_original() {
    for full_plan in [false, true] {
        let (mut backend, mut target) = established();
        target.vram.as_mut().unwrap().maximum = 95;
        let reviewed = if full_plan {
            plan(&mut backend, &target)
        } else {
            plan_preliminary(&mut backend, &target)
        }
        .unwrap();
        assert_eq!(reviewed.authentication_deferred, !full_plan);
        let result = apply_approved(&mut backend, &reviewed).unwrap();
        assert!(!result.prepared && !result.verified);
        assert_eq!(backend.events, trace(&reviewed.preview));
        assert_eq!(result.effective.vram, target.vram);
        assert_counts(
            "ordinary Apply",
            backend.work,
            Expected {
                reads: [15, 13, 13],
                journals: 3,
                payloads: if full_plan { 2 } else { 1 },
                power: [0, 0],
                adapters: [0, 0],
                settings: 0,
                allocations: [1, 0, 0, 0], // Three matching categories each remove two fact checks.
            },
        );
    }
    let (mut backend, target) = established();
    let reviewed = plan_reinstall(&mut backend, &target).unwrap();
    let result = apply_approved(&mut backend, &reviewed).unwrap();
    assert!(result.prepared && result.verified);
    assert_eq!(backend.events, trace(&reviewed.preview));
    assert_counts(
        "reinstall",
        backend.work,
        Expected {
            reads: [40, 36, 42],
            journals: 7,
            payloads: 2,
            power: [2, 2],
            adapters: [1, 1],
            settings: 0,
            allocations: [1; 4],
        },
    );
}

#[test]
fn original_disable_and_unchanged_apply_remain_payload_free_or_effect_free() {
    for disable in [false, true] {
        let (mut backend, mut target) = established();
        target.enabled = !disable;
        let reviewed = plan_preliminary(&mut backend, &target).unwrap();
        let result = apply_approved(&mut backend, &reviewed).unwrap();
        assert!(!result.prepared && !result.verified);
        assert_eq!(result.effective.power, Power::Off);
        assert_eq!(backend.events, trace(&reviewed.preview));
        assert_counts(
            "disable/unchanged",
            backend.work,
            Expected {
                reads: if disable { [16, 13, 13] } else { [10, 8, 8] },
                journals: if disable { 2 } else { 1 },
                payloads: u64::from(!disable),
                power: [0, 0],
                adapters: [0, u64::from(disable)],
                settings: u64::from(disable),
                allocations: [0; 4],
            },
        );
    }
}

#[test]
fn original_verify_from_off_and_running_counts_include_restoration() {
    for power in [Power::Off, Power::Running] {
        let (mut backend, target) = established();
        backend.state.power = power.clone();
        let reviewed = super::plan_verification(&mut backend, &target).unwrap();
        let result = super::verify_approved(&mut backend, &reviewed).unwrap();
        assert!(result.verified && !result.prepared);
        assert_eq!(result.effective.power, power);
        assert!(!backend.journal.as_ref().unwrap().pending);
        assert_counts(
            "Verify",
            backend.work,
            Expected {
                reads: [8, 5, 12],
                journals: 4,
                payloads: 2,
                power: if power == Power::Off { [1, 1] } else { [0, 0] },
                adapters: [0, 0],
                settings: 0,
                allocations: [0; 4],
            },
        );
    }
}

#[test]
fn original_reenable_keeps_completed_preparation_but_rewrites_requested_categories() {
    let (mut backend, mut target) = established();
    target.enabled = false;
    apply(&mut backend, &target).unwrap();
    backend.work = Counts::default();
    backend.events.clear();
    target.enabled = true;
    let reviewed = plan_preliminary(&mut backend, &target).unwrap();
    assert!(reviewed.authentication_deferred);
    let result = apply_approved(&mut backend, &reviewed).unwrap();
    assert!(!result.prepared && !result.verified);
    assert_eq!(backend.events, trace(&reviewed.preview));
    assert_counts(
        "reenable",
        backend.work,
        Expected {
            reads: [26, 23, 23],
            journals: 3,
            payloads: 1,
            power: [0, 0],
            adapters: [1, 0],
            settings: 1,
            allocations: [1; 4],
        },
    );
}

#[test]
fn failed_preparation_retains_recovery_without_a_version_identification_cycle() {
    let mut backend = fake();
    backend.state.power = Power::Off;
    backend.fail_prepare = true;
    let reviewed = plan(&mut backend, &requested()).unwrap();
    assert!(apply_approved(&mut backend, &reviewed).is_err());
    assert_eq!(backend.events, ["start", "prepare"]);
    assert_eq!(backend.work.get(Counter::PowerStarts), 1);
    assert_eq!(backend.work.get(Counter::PowerShutdowns), 0);
    assert_eq!(backend.work.get(Counter::AttachWrites), 0);
    assert_eq!(backend.work.get(Counter::SettingsWrites), 0);
    assert_eq!(backend.state.power, Power::Running);
    assert!(backend.journal.as_ref().unwrap().pending);
}

#[test]
fn baseline_save_only_admission_reads_without_replaying_workflow_effects() {
    use crate::configuration_store::{Revision, SavePhase};
    let (mut backend, target) = established();
    let mut record = recovery_record(&backend, true);
    record.target = target.clone();
    record.intended_source = Configuration {
        schema: 3,
        targets: vec![target.clone()],
    }
    .vm_documents()
    .unwrap()
    .into_values()
    .next()
    .unwrap();
    record.phase = SavePhase::Verified(backend.state.clone());
    // Only the existing save-only read/validation contract, without protected
    // storage, publication, runner checks, or a synthesized successful save.
    assert!(!backend.journal(&target).unwrap().unwrap().pending);
    assert_eq!(
        backend.payload(&target).unwrap(),
        record.payload_digest.clone().unwrap()
    );
    let current = backend.inspect(&target).unwrap();
    let driver = backend.gpu(&target).unwrap().driver_version;
    record
        .validate_save_only(&current, &driver, &Revision::Missing)
        .unwrap();
    assert!(backend.events.is_empty());
    assert_counts(
        "save-only admission",
        backend.work,
        Expected {
            reads: [1; 3],
            journals: 0,
            payloads: 1,
            power: [0, 0],
            adapters: [0, 0],
            settings: 0,
            allocations: [0; 4],
        },
    );
    let mut changed = current;
    changed.settings.high_mmio += 1;
    assert!(
        record
            .validate_save_only(&changed, &driver, &Revision::Missing)
            .is_err()
    );
}

#[test]
fn baseline_read_only_plan_has_no_mutations_and_counts_failed_payload_attempts() {
    for fail in [false, true] {
        let mut backend = fake();
        backend.fail_payload = fail;
        let result = plan(&mut backend, &requested());
        assert_eq!(result.is_err(), fail);
        assert!(backend.events.is_empty());
        assert!(backend.journal.is_none());
        assert_counts(
            "read-only plan",
            backend.work,
            Expected {
                reads: [1; 3],
                journals: 0,
                payloads: 1,
                power: [0, 0],
                adapters: [0, 0],
                settings: 0,
                allocations: [0; 4],
            },
        );
    }
}

#[test]
fn failed_category_counts_attempt_without_claiming_success_or_later_writes() {
    let (mut backend, mut target) = established();
    target.vram.as_mut().unwrap().maximum = 95;
    target.encode.as_mut().unwrap().maximum = 95;
    let reviewed = plan_preliminary(&mut backend, &target).unwrap();
    backend.fail_allocation = Some(Resource::Encode);
    let writes_before = backend.allocation_writes;
    assert!(apply_approved(&mut backend, &reviewed).is_err());
    assert_eq!(backend.work.get(Counter::EncodeWrites), 1);
    assert_eq!(backend.work.get(Counter::DecodeWrites), 0);
    assert_eq!(backend.allocation_writes - writes_before, 1);
    assert!(backend.journal.as_ref().unwrap().pending);
    assert!(!backend.events.contains(&"graphics"));
}
