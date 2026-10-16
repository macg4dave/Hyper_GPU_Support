//! Scenarios spanning operations, recovery retirement, enrollment and other VMs.
use crate::model::{Allocation, Gpu, Power, Resource, Settings, Target, VmState};
use crate::workflow::test_support::{Fake, fake, recovery_record, target, trace};
use crate::workflow::{
    Backend, GuestCheck, Journal, PreviewAction, apply, apply_approved, plan, plan_reinstall,
    reconcile_observed, reconcile_with_graphics, verify,
};

#[test]
fn changed_check_journals_block_power_restoration_and_completion() {
    use crate::workflow::test_support::Drift;
    for reconciliation in [false, true] {
        for stage in ["start", "verify", "shutdown"] {
            for failed_check in [false, true] {
                let mut backend = fake();
                apply(&mut backend, &target()).unwrap();
                let original = backend.journal.as_ref().unwrap().original.high_mmio;
                if reconciliation {
                    let journal = backend.journal.as_mut().unwrap();
                    journal.pending = true;
                    journal.verification_only = true;
                }
                backend.events.clear();
                backend.fail_verify = failed_check;
                backend.drift_at = Some((stage, Drift::Journal));
                let observed = backend.state.clone();
                let result = if reconciliation {
                    reconcile_with_graphics(
                        &mut backend,
                        &target(),
                        &observed,
                        Power::Off,
                        Some("current"),
                        true,
                    )
                } else {
                    verify(&mut backend, &target())
                };
                let error = result.unwrap_err();
                assert!(error.contains("journal"), "{error}");
                if failed_check && stage != "start" {
                    assert!(error.contains("graphics failed"), "{error}");
                }
                let journal = backend.journal.as_ref().unwrap();
                assert_eq!(journal.original.high_mmio, original + 1);
                assert!(journal.pending && !journal.check_restored);
                assert_eq!(
                    backend.events,
                    match stage {
                        "start" => vec!["start"],
                        "verify" => vec!["start", "graphics"],
                        _ => vec!["start", "graphics", "shutdown"],
                    }
                );
                assert_eq!(
                    backend.state.power,
                    if stage == "shutdown" {
                        Power::Off
                    } else {
                        Power::Running
                    }
                );
            }
        }
    }
}

#[test]
fn check_completion_requires_independent_journal_readback() {
    for operation in [
        "verify",
        "graphics-reconciliation",
        "ordinary-reconciliation",
    ] {
        for unavailable in [false, true] {
            let mut backend = fake();
            apply(&mut backend, &target()).unwrap();
            backend.journal_writes = 0;
            backend.events.clear();
            let final_write = if operation == "ordinary-reconciliation" {
                1
            } else {
                4
            };
            if unavailable {
                backend.fail_read_after_save_at = Some(final_write);
            } else {
                backend.mismatch_read_after_save_at = Some(final_write);
            }
            let observed = backend.state.clone();
            let result = if operation == "verify" {
                verify(&mut backend, &target())
            } else {
                backend.journal.as_mut().unwrap().pending = true;
                reconcile_with_graphics(
                    &mut backend,
                    &target(),
                    &observed,
                    Power::Off,
                    Some("current"),
                    operation == "graphics-reconciliation",
                )
            };
            assert!(result.unwrap_err().contains("journal"));
            assert_eq!(backend.journal_writes, final_write);
            assert_eq!(backend.state.power, Power::Off);
            assert!(!backend.events.contains(&"prepare"));
            assert!(!backend.events.contains(&"attach"));
            if operation == "ordinary-reconciliation" {
                assert!(backend.events.is_empty());
            }
        }
    }
}

#[test]
fn failed_verification_only_apply_can_end_without_publication_then_repair_or_remove() {
    use crate::configuration_store::Revision;
    for initial_power in [Power::Off, Power::Running] {
        for missing_success in [false, true] {
            let mut backend = fake();
            apply(&mut backend, &target()).unwrap();
            let journal = backend.journal.as_mut().unwrap();
            if missing_success {
                journal.last_verified = None;
            } else {
                journal.latest_check = Some(GuestCheck::Unavailable);
            }
            backend.state.power = initial_power.clone();
            let mut record = recovery_record(&backend, true);
            let history = backend.journal.as_ref().unwrap().last_verified;
            backend.payload_check_failure = Some(crate::reporting::FailureCause::FileMissing);
            backend.events.clear();
            let approved = plan(&mut backend, &target()).unwrap();
            assert!(!approved.preview.actions.contains(&PreviewAction::Prepare));
            assert!(apply_approved(&mut backend, &approved).is_err());
            assert_eq!(backend.state.power, Power::Off);
            assert!(!backend.events.contains(&"prepare"));
            let pending = backend.journal.as_ref().unwrap().clone();
            assert!(pending.pending && pending.verification_only && pending.check_restored);
            assert!(pending.prepared.is_none());
            assert_eq!(pending.last_verified, history);
            assert!(
                plan_reinstall(&mut backend, &target())
                    .unwrap()
                    .preview
                    .pending_recovery
            );
            let effects = backend.events.clone();
            let retirement = record
                .negative_completion(&backend.state, "new", &Revision::Missing, Some(&pending))
                .unwrap()
                .unwrap();
            assert!(!retirement.after.pending);
            assert_eq!(
                retirement.after.latest_check,
                Some(GuestCheck::PayloadMismatch)
            );
            assert_eq!(retirement.after.last_verified, history);
            assert!(retirement.after.prepared.is_none());
            record.retirement = Some(retirement.clone());
            for journal in [&retirement.before, &retirement.after] {
                assert!(
                    record
                        .negative_completion(
                            &backend.state,
                            "new",
                            &Revision::Missing,
                            Some(journal)
                        )
                        .unwrap()
                        .is_some()
                );
            }
            let mut changed = retirement.after.clone();
            changed.last_verified = Some(999);
            assert!(
                record
                    .negative_completion(&backend.state, "new", &Revision::Missing, Some(&changed))
                    .is_err()
            );
            backend.save(&target(), &retirement.after).unwrap();
            assert_eq!(backend.events, effects); // Retirement has no guest/GPU effects.
            backend.payload_check_failure = None;
            if missing_success {
                let repair = plan_reinstall(&mut backend, &target()).unwrap();
                assert!(apply_approved(&mut backend, &repair).unwrap().verified);
            } else {
                let mut disabled = target();
                disabled.enabled = false;
                assert!(
                    !plan(&mut backend, &disabled)
                        .unwrap()
                        .preview
                        .credentials_required
                );
                apply(&mut backend, &disabled).unwrap();
                assert!(backend.state.gpus.is_empty());
            }
        }
    }
}

#[test]
fn failed_check_apply_retirement_refuses_changed_missing_or_uncertain_proof() {
    use crate::configuration_store::Revision;
    for scenario in 0..12 {
        let mut backend = fake();
        apply(&mut backend, &target()).unwrap();
        backend.journal.as_mut().unwrap().last_verified = None;
        let mut record = recovery_record(&backend, true);
        backend.payload_check_failure = Some(crate::reporting::FailureCause::FileChanged);
        if scenario == 0 {
            backend.fail_shutdown = true;
        }
        if scenario == 1 {
            backend.fail_save_at = Some(backend.journal_writes + 4);
        }
        if scenario == 9 {
            backend.fail_read_after_save_at = Some(backend.journal_writes + 4);
        }
        if scenario == 10 {
            backend.mismatch_read_after_save_at = Some(backend.journal_writes + 4);
        }
        if scenario == 11 {
            backend.fail_start_after_effect = true;
        }
        assert!(apply(&mut backend, &target()).is_err());
        let mut revision = Revision::Missing;
        let mut driver = "new";
        match scenario {
            2 => backend.state.settings.high_mmio += 1,
            3 => backend.state.gpus.clear(),
            4 => revision = Revision::of(b"external change"),
            5 => driver = "changed",
            6 => record.graphics_required = None,
            7 => record.initial_journal = None,
            8 => backend.journal.as_mut().unwrap().preparation_started = None,
            _ => {}
        }
        let effects = backend.events.clone();
        let journal_read = backend.journal(&target());
        assert!(
            !journal_read
                .and_then(|journal| record.negative_completion(
                    &backend.state,
                    driver,
                    &revision,
                    journal.as_ref()
                ))
                .is_ok_and(|candidate| candidate.is_some()),
            "scenario {scenario}"
        );
        assert_eq!(backend.events, effects);
        assert!(backend.journal.as_ref().unwrap().pending);
    }
}

#[test]
fn restored_failure_retains_primary_when_proof_save_or_readback_fails() {
    use crate::reporting::{ComponentFailure, FailureCause};
    for reconciliation in [false, true] {
        for payload_failure in [false, true] {
            for persistence in ["save", "read", "mismatch"] {
                let mut backend = fake();
                apply(&mut backend, &target()).unwrap();
                backend.events.clear();
                let history = backend.journal.as_ref().unwrap().last_verified;
                if payload_failure {
                    backend.payload_check_failure = Some(FailureCause::FileMissing);
                } else {
                    backend.fail_verify = true;
                }
                // Admission, check-start, and negative result precede the proof.
                let proof_write = backend.journal_writes + 4;
                match persistence {
                    "save" => backend.fail_save_at = Some(proof_write),
                    "read" => backend.fail_read_after_save_at = Some(proof_write),
                    _ => backend.mismatch_read_after_save_at = Some(proof_write),
                }
                let error = if reconciliation {
                    let journal = backend.journal.as_mut().unwrap();
                    journal.pending = true;
                    journal.verification_only = true;
                    let observed = backend.state.clone();
                    reconcile_with_graphics(
                        &mut backend,
                        &target(),
                        &observed,
                        Power::Off,
                        Some("current"),
                        true,
                    )
                    .unwrap_err()
                } else {
                    verify(&mut backend, &target()).unwrap_err()
                };
                assert!(
                    error.contains("restoration-proof persistence failed"),
                    "{error}"
                );
                let public = crate::reporting::operator_error("Verify", &error);
                assert!(public.contains("restoration proof"));
                assert!(public.contains(if payload_failure {
                    "driver files are missing"
                } else {
                    "GuestGraphics"
                }));
                let details = crate::reporting::technical_error("Verify", &error);
                assert!(details.contains("Restoration-proof persistence failed"));
                assert!(
                    error.contains(match persistence {
                        "save" => "publication denied",
                        "read" => "readback unavailable",
                        _ => "readback mismatch",
                    }),
                    "{error}"
                );
                let primary =
                    ComponentFailure::decode(&error).expect("typed primary remains reportable");
                assert!(matches!(
                    primary.cause,
                    FailureCause::FileMissing | FailureCause::RenderingFailed
                ));
                assert_eq!(backend.state.power, Power::Off);
                assert_eq!(
                    backend.events,
                    [
                        "start",
                        if payload_failure {
                            "payload-check"
                        } else {
                            "graphics"
                        },
                        "shutdown"
                    ]
                );
                let journal = backend.journal.as_ref().unwrap();
                assert!(journal.pending);
                assert_eq!(journal.last_verified, history);
                assert_eq!(
                    journal.latest_check,
                    Some(if payload_failure {
                        GuestCheck::PayloadMismatch
                    } else {
                        GuestCheck::Failed
                    })
                );
                if persistence == "save" {
                    assert!(!journal.check_restored);
                }
            }
        }
    }
}

#[test]
fn verify_without_management_history_never_acquires_settings_ownership() {
    let mut backend = fake();
    backend.state.gpus = vec![target().gpu_interface];
    backend.state.power = Power::Off;
    verify(&mut backend, &target()).unwrap();
    assert!(backend.journal.as_ref().unwrap().settings_released);
    assert!(backend.journal.as_ref().unwrap().prepared.is_none());
    let original = backend.state.settings.clone();
    let disabled = Target {
        enabled: false,
        ..target()
    };
    apply(&mut backend, &disabled).unwrap();
    assert_eq!(backend.state.settings, original);
    crate::enrollment::validate_inactive_profile(
        &disabled,
        &backend.state,
        backend.journal.as_ref(),
        Some(&disabled),
    )
    .unwrap();
}

#[test]
fn restored_reconciliation_failure_permits_reviewed_repair_then_removal_and_forget() {
    let mut backend = fake();
    apply(&mut backend, &target()).unwrap();
    let record = recovery_record(&backend, false);
    backend.payload_check_failure = Some(crate::reporting::FailureCause::FileMissing);
    backend.fail_shutdown = true;
    assert!(verify(&mut backend, &target()).is_err());
    assert!(!backend.journal.as_ref().unwrap().check_restored);
    assert_eq!(backend.state.power, Power::Running);
    backend.fail_shutdown = false;
    backend.events.clear();
    let observed = backend.state.clone();
    assert!(
        reconcile_with_graphics(
            &mut backend,
            &target(),
            &observed,
            Power::Off,
            Some("current"),
            true
        )
        .is_err()
    );
    let journal = backend.journal.as_ref().unwrap();
    assert!(journal.pending && journal.check_restored);
    assert_eq!(journal.latest_check, Some(GuestCheck::PayloadMismatch));
    assert!(journal.prepared.is_none());
    assert_eq!(backend.events, ["payload-check", "shutdown"]);
    let retirement = record
        .negative_completion(
            &backend.state,
            "new",
            &crate::configuration_store::Revision::Missing,
            Some(journal),
        )
        .unwrap()
        .unwrap();
    backend.journal = Some(retirement.after);
    backend.payload_check_failure = None;
    let repair = plan_reinstall(&mut backend, &target()).unwrap();
    let result = apply_approved(&mut backend, &repair).unwrap();
    assert!(result.prepared && result.verified);
    let disabled = Target {
        enabled: false,
        ..target()
    };
    let removal = plan(&mut backend, &disabled).unwrap();
    assert!(!removal.preview.credentials_required);
    apply_approved(&mut backend, &removal).unwrap();
    crate::enrollment::validate_inactive_profile(
        &disabled,
        &backend.state,
        backend.journal.as_ref(),
        Some(&disabled),
    )
    .unwrap();
}

#[test]
fn failed_verify_can_end_recovery_without_certifying_setup_then_review_repair() {
    let mut backend = fake();
    apply(&mut backend, &target()).unwrap();
    let mut record = recovery_record(&backend, false);
    let history = backend.journal.as_ref().unwrap().last_verified;
    backend.payload_check_failure = Some(crate::reporting::FailureCause::FileMissing);
    assert!(verify(&mut backend, &target()).is_err());
    let pending = backend.journal.as_ref().unwrap();
    assert!(pending.pending && pending.check_restored);
    let retirement = record
        .negative_completion(
            &backend.state,
            "new",
            &crate::configuration_store::Revision::Missing,
            Some(pending),
        )
        .unwrap()
        .unwrap();
    assert!(retirement.after.prepared.is_none());
    assert_eq!(
        retirement.after.latest_check,
        Some(GuestCheck::PayloadMismatch)
    );
    assert_eq!(retirement.after.last_verified, history);
    record.retirement = Some(retirement.clone());
    // Interrupted retirement accepts exactly its saved before/after journals.
    for journal in [&retirement.before, &retirement.after] {
        assert!(
            record
                .negative_completion(
                    &backend.state,
                    "new",
                    &crate::configuration_store::Revision::Missing,
                    Some(journal)
                )
                .unwrap()
                .is_some()
        );
    }
    backend.save(&target(), &retirement.after).unwrap();
    backend.payload_check_failure = None;
    let reviewed = plan_reinstall(&mut backend, &target()).unwrap();
    assert!(reviewed.preview.actions.contains(&PreviewAction::Prepare));
    assert!(apply_approved(&mut backend, &reviewed).unwrap().verified);
}

#[test]
fn refused_reinstall_shutdown_can_end_without_losing_original_history_or_replaying() {
    let mut backend = fake();
    apply(&mut backend, &target()).unwrap();
    backend.state.power = Power::Running;
    let mut record = recovery_record(&backend, true);
    let reviewed = plan_reinstall(&mut backend, &target()).unwrap();
    backend.events.clear();
    backend.fail_shutdown = true;
    assert!(apply_approved(&mut backend, &reviewed).is_err());
    assert_eq!(backend.events, ["shutdown"]);
    let retirement = record
        .negative_completion(
            &backend.state,
            "new",
            &crate::configuration_store::Revision::Missing,
            backend.journal.as_ref(),
        )
        .unwrap()
        .unwrap();
    assert!(retirement.after.setup_eligible());
    assert!(!retirement.after.pending);
    assert_eq!(
        serde_json::to_value(&retirement.after).unwrap(),
        serde_json::to_value(record.initial_journal.as_ref().unwrap()).unwrap()
    );
    record.retirement = Some(retirement.clone());
    let mut changed = retirement.before.clone();
    changed.preparation_started = Some(true);
    assert!(
        record
            .negative_completion(
                &backend.state,
                "new",
                &crate::configuration_store::Revision::Missing,
                Some(&changed)
            )
            .is_err()
    );
    assert!(
        record
            .negative_completion(
                &backend.state,
                "changed",
                &crate::configuration_store::Revision::Missing,
                Some(&retirement.before)
            )
            .is_err()
    );
    backend.save(&target(), &retirement.after).unwrap();
    assert_eq!(backend.events, ["shutdown"]);
    assert!(
        !plan(&mut backend, &target())
            .unwrap()
            .preview
            .credentials_required
    );
}

#[test]
fn negative_completion_rejects_uncertain_or_changed_facts_and_legacy_proofs() {
    let mut backend = fake();
    apply(&mut backend, &target()).unwrap();
    let record = recovery_record(&backend, false);
    backend.payload_check_failure = Some(crate::reporting::FailureCause::FileChanged);
    assert!(verify(&mut backend, &target()).is_err());
    let journal = backend.journal.as_ref().unwrap();
    for scenario in 0..4 {
        let mut uncertain = journal.clone();
        match scenario {
            0 => uncertain.check_restored = false,
            1 => uncertain.latest_check = Some(GuestCheck::Interrupted),
            2 => uncertain.verification_only = false,
            _ => uncertain.pending = false,
        }
        assert!(
            record
                .negative_completion(
                    &backend.state,
                    "new",
                    &crate::configuration_store::Revision::Missing,
                    Some(&uncertain)
                )
                .unwrap()
                .is_none()
        );
    }
    let mut changed = backend.state.clone();
    changed.settings.high_mmio += 1;
    assert!(
        record
            .negative_completion(
                &changed,
                "new",
                &crate::configuration_store::Revision::Missing,
                Some(journal)
            )
            .is_err()
    );
    assert!(
        record
            .negative_completion(
                &backend.state,
                "new",
                &crate::configuration_store::Revision::of(b"external settings"),
                Some(journal)
            )
            .is_err()
    );
    // Unknown transport failure never establishes a terminal guest observation.
    backend.payload_check_failure = None;
    backend.journal.as_mut().unwrap().pending = false;
    backend.fail_start_after_effect = true;
    assert!(verify(&mut backend, &target()).is_err());
    assert!(!backend.journal.as_ref().unwrap().check_restored);
}

#[test]
fn successful_verify_reconciliation_does_not_resurrect_invalidated_preparation() {
    let mut backend = fake();
    apply(&mut backend, &target()).unwrap();
    backend.payload_check_failure = Some(crate::reporting::FailureCause::FileChanged);
    assert!(verify(&mut backend, &target()).is_err());
    backend.payload_check_failure = None;
    backend.events.clear();
    let observed = backend.state.clone();
    reconcile_with_graphics(
        &mut backend,
        &target(),
        &observed,
        Power::Off,
        Some("current"),
        true,
    )
    .unwrap();
    let journal = backend.journal.as_ref().unwrap();
    assert!(!journal.pending);
    assert_eq!(journal.latest_check, Some(GuestCheck::Passed));
    assert!(journal.prepared.is_none() && !journal.setup_eligible());
    assert!(!backend.events.contains(&"prepare"));
    let reviewed = plan(&mut backend, &target()).unwrap();
    assert!(reviewed.preview.actions.contains(&PreviewAction::Prepare));
    assert!(reviewed.preview.credentials_required);
}

#[test]
fn check_outcomes_survive_interruption_lifecycle_and_publication_failures() {
    for scenario in 0..6 {
        let mut backend = fake();
        apply(&mut backend, &target()).unwrap();
        let history = backend.journal.as_ref().unwrap().last_verified;
        backend.events.clear();
        match scenario {
            0 => backend.fail_start_after_effect = true,
            1 => backend.fail_verify = true,
            2 => backend.fail_final_shutdown = true,
            // Refuse terminal check publication, retaining admitted unknown state.
            3 => backend.fail_save_at = Some(backend.journal_writes + 3),
            // Refuse lifecycle completion publication, retaining the hold.
            4 => backend.fail_save_at = Some(backend.journal_writes + 4),
            _ => backend.fail_read_after_save_at = Some(backend.journal_writes + 3),
        }
        assert!(verify(&mut backend, &target()).is_err());
        let journal = backend.journal.as_ref().unwrap();
        assert!(journal.pending && journal.verification_only);
        assert_eq!(journal.last_verified, history);
        assert_eq!(
            journal.latest_check,
            Some(match scenario {
                0 | 3 => GuestCheck::Interrupted,
                1 => GuestCheck::Failed,
                _ => GuestCheck::Passed,
            })
        );
        let events = backend.events.clone();
        backend.fail_read_after_save_at = None;
        let approved = plan(&mut backend, &target()).unwrap();
        assert!(apply_approved(&mut backend, &approved).is_err());
        assert_eq!(backend.events, events);
        assert!(backend.journal.as_ref().unwrap().pending);
    }
}

#[test]
fn invalid_preparation_requires_explicit_setup_update_review() {
    for invalid in 0..3 {
        let mut f = fake();
        apply(&mut f, &target()).unwrap();
        match invalid {
            0 => f.driver = "updated",
            1 => f.journal.as_mut().unwrap().prepared = None,
            _ => f.journal.as_mut().unwrap().last_verified = None,
        }
        f.events.clear();
        let approved = plan(&mut f, &target()).unwrap();
        assert!(approved.preview.credentials_required);
        assert!(approved.preview.actions.contains(&PreviewAction::Verify));
        assert!(
            approved
                .preview
                .summary
                .iter()
                .any(|text| text.contains("setup"))
        );
        assert!(f.events.is_empty());
        let result = apply_approved(&mut f, &approved).unwrap();
        assert!(result.verified);
        assert_eq!(result.effective.power, Power::Off);
        assert_eq!(f.events, trace(&approved.preview));
    }
}

#[test]
fn initial_setup_of_second_vm_preserves_first_on_the_same_gpu() {
    struct Fleet(std::collections::BTreeMap<String, Fake>);
    impl Fleet {
        fn selected(&mut self, target: &Target) -> &mut Fake {
            self.0
                .get_mut(&target.vm_id)
                .expect("effect targeted a known VM")
        }
    }
    impl Backend for Fleet {
        fn preflight_installation_os(&mut self, t: &Target) -> Result<(), String> {
            self.selected(t).preflight_installation_os(t)
        }
        fn check_installation_os(&mut self, t: &Target) -> Result<(), String> {
            self.selected(t).check_installation_os(t)
        }
        fn inspect(&mut self, t: &Target) -> Result<VmState, String> {
            self.selected(t).inspect(t)
        }
        fn gpu(&mut self, t: &Target) -> Result<Gpu, String> {
            self.selected(t).gpu(t)
        }
        fn payload(&mut self, t: &Target) -> Result<String, String> {
            self.selected(t).payload(t)
        }
        fn journal(&mut self, t: &Target) -> Result<Option<Journal>, String> {
            self.selected(t).journal(t)
        }
        fn save(&mut self, t: &Target, j: &Journal) -> Result<(), String> {
            self.selected(t).save(t, j)
        }
        fn power(&mut self, t: &Target, p: Power) -> Result<(), String> {
            self.selected(t).power(t, p)
        }
        fn assign(&mut self, t: &Target, enabled: bool) -> Result<(), String> {
            self.selected(t).assign(t, enabled)
        }
        fn settings(&mut self, t: &Target, s: &Settings) -> Result<(), String> {
            self.selected(t).settings(t, s)
        }
        fn allocation(&mut self, t: &Target, r: Resource, a: &Allocation) -> Result<(), String> {
            self.selected(t).allocation(t, r, a)
        }
        fn prepare(&mut self, t: &Target) -> Result<String, String> {
            self.selected(t).prepare(t)
        }
        fn verify(&mut self, t: &Target, g: &Gpu) -> Result<(), String> {
            self.selected(t).verify(t, g)
        }
    }
    let first = target();
    let mut second = first.clone();
    second.vm_id = "22222222-2222-2222-2222-222222222222".into();
    let mut second_backend = fake();
    second_backend.state.vm_id = second.vm_id.clone();
    let mut fleet = Fleet(std::collections::BTreeMap::from([
        (first.vm_id.clone(), fake()),
        (second.vm_id.clone(), second_backend),
    ]));
    apply(&mut fleet, &first).unwrap();
    // The first guest is running when setup is requested for the second.
    fleet.0.get_mut(&first.vm_id).unwrap().state.power = Power::Running;
    let first_state = serde_json::to_value(&fleet.0[&first.vm_id].state).unwrap();
    let first_journal = serde_json::to_value(&fleet.0[&first.vm_id].journal).unwrap();
    let first_events = fleet.0[&first.vm_id].events.clone();
    let approved = plan(&mut fleet, &second).unwrap();
    let result = apply_approved(&mut fleet, &approved).unwrap();
    assert!(result.verified);
    assert_eq!(result.effective.power, Power::Off);
    assert_eq!(
        serde_json::to_value(&fleet.0[&first.vm_id].state).unwrap(),
        first_state
    );
    assert_eq!(
        serde_json::to_value(&fleet.0[&first.vm_id].journal).unwrap(),
        first_journal
    );
    assert_eq!(fleet.0[&first.vm_id].events, first_events);
    let reinstall = plan_reinstall(&mut fleet, &second).unwrap();
    let repaired = apply_approved(&mut fleet, &reinstall).unwrap();
    assert!(repaired.prepared && repaired.verified);
    assert_eq!(
        serde_json::to_value(&fleet.0[&first.vm_id].state).unwrap(),
        first_state
    );
    assert_eq!(
        serde_json::to_value(&fleet.0[&first.vm_id].journal).unwrap(),
        first_journal
    );
    assert_eq!(fleet.0[&first.vm_id].events, first_events);
    let mut removal = second.clone();
    removal.enabled = false;
    let reviewed = plan(&mut fleet, &removal).unwrap();
    let removed = apply_approved(&mut fleet, &reviewed).unwrap();
    assert!(removed.effective.gpus.is_empty() && !removed.verified);
    assert_eq!(
        serde_json::to_value(&fleet.0[&first.vm_id].state).unwrap(),
        first_state
    );
    assert_eq!(
        serde_json::to_value(&fleet.0[&first.vm_id].journal).unwrap(),
        first_journal
    );
    assert_eq!(fleet.0[&first.vm_id].events, first_events);
    let second_state = serde_json::to_value(&fleet.0[&second.vm_id].state).unwrap();
    let second_journal = serde_json::to_value(&fleet.0[&second.vm_id].journal).unwrap();
    let second_events = fleet.0[&second.vm_id].events.clone();
    let mut ordinary = first.clone();
    ordinary.vram = Some(Allocation {
        minimum: 10,
        optimal: 50,
        maximum: 90,
    });
    let approved = plan(&mut fleet, &ordinary).unwrap();
    let result = apply_approved(&mut fleet, &approved).unwrap();
    assert!(!result.verified && !result.prepared);
    assert_eq!(result.effective.power, Power::Off);
    assert_eq!(
        serde_json::to_value(&fleet.0[&second.vm_id].state).unwrap(),
        second_state
    );
    assert_eq!(
        serde_json::to_value(&fleet.0[&second.vm_id].journal).unwrap(),
        second_journal
    );
    assert_eq!(fleet.0[&second.vm_id].events, second_events);
}

#[test]
fn interrupted_final_shutdown_reconciles_to_off_without_repeating_setup() {
    let mut backend = fake();
    backend.fail_final_shutdown = true;
    let approved = plan(&mut backend, &target()).unwrap();
    assert!(apply_approved(&mut backend, &approved).is_err());
    assert_eq!(backend.state.power, Power::Running);
    backend.fail_final_shutdown = false;
    backend.events.clear();
    backend.allocation_writes = 0;
    let observed = backend.state.clone();
    let result = reconcile_observed(
        &mut backend,
        &target(),
        &observed,
        Power::Off,
        Some("current"),
    )
    .unwrap();
    assert!(result.verified);
    assert_eq!(result.effective.power, Power::Off);
    assert_eq!(backend.events, ["graphics", "shutdown"]);
    assert_eq!(backend.allocation_writes, 0);
    assert!(!backend.journal.as_ref().unwrap().pending);
}

#[test]
fn forgotten_history_requires_complete_setup_after_fresh_enrollment() {
    let mut backend = fake();
    apply(&mut backend, &target()).unwrap();
    let mut disabled = target();
    disabled.enabled = false;
    let reviewed = plan(&mut backend, &disabled).unwrap();
    apply_approved(&mut backend, &reviewed).unwrap();
    // Profile removal deletes the host journal; guest files may still exist.
    backend.journal = None;
    backend.events.clear();
    let fresh = plan(&mut backend, &target()).unwrap();
    assert!(fresh.initial_setup && fresh.preview.credentials_required);
    assert!(fresh.preview.actions.contains(&PreviewAction::Prepare));
    assert!(fresh.preview.actions.contains(&PreviewAction::Verify));
    assert!(backend.events.is_empty());
}
