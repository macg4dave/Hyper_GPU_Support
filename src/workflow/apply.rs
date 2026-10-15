//! Admission and execution of reviewed selected-VM changes.
use super::guest_check::{check_guest, record_restored_check_failure};
use super::journal::{persist_check, verified_at};
use super::planning::{Decision, decision, decision_for_operation, plan_from_decision};
use super::{Backend, GuestCheck, OperationResult, Plan};
use crate::model::{Power, Resource, Target};
mod facts;
use facts::ApplyFacts;

/// Apply one selected target after independently reading and validating current state.
/// Failures retain recovery intent; no disk recreation or forced shutdown is attempted.
pub fn apply(backend: &mut impl Backend, target: &Target) -> Result<OperationResult, String> {
    let d = decision(backend, target)?;
    execute_decision(backend, target, d)
}
/// Execute only when independently gathered state still matches the approved plan.
/// This check precedes journal admission and every GPU/guest effect.
pub fn apply_approved(
    backend: &mut impl Backend,
    approved: &Plan,
) -> Result<OperationResult, String> {
    let (_, admitted) = admit_approved(backend, approved)?;
    apply_admitted(backend, admitted)
}

/// In-process admission evidence. Private fields, no serialization or cloning:
/// callers cannot turn a frontend plan into authenticated execution inputs.
/// The restricted worker retains this only under its existing operation lock.
pub(crate) struct AdmittedApply {
    target: Target,
    decision: Decision,
}
impl AdmittedApply {
    pub(crate) fn driver_version(&self) -> &str {
        &self.decision.gpu.driver_version
    }
}

/// Independently authenticate current inputs and bind them to the approved plan.
/// This is the worker's single pre-effect payload pass, never a persistent cache.
pub(crate) fn admit_approved(
    backend: &mut impl Backend,
    approved: &Plan,
) -> Result<(Plan, super::AdmittedApply), String> {
    let target = &approved.desired;
    let d = decision_for_operation(backend, target, approved.reinstall)?;
    let actual = plan_from_decision(d.clone(), target);
    let mut reviewed = approved.clone();
    // Only this presentation marker differs. Every reviewed host, journal,
    // payload and effect fact must still match the freshly authenticated plan.
    reviewed.authentication_deferred = false;
    if serde_json::to_value(&actual).map_err(|e| e.to_string())?
        != serde_json::to_value(&reviewed).map_err(|e| e.to_string())?
    {
        return Err("approved plan is stale; refresh and review before effects".into());
    }
    if d.journal.pending {
        return Err("pending recovery requires manual reconciliation, not Apply".into());
    }
    Ok((
        actual,
        AdmittedApply {
            target: target.clone(),
            decision: d,
        },
    ))
}

/// Consume authenticated admission in the same locked worker/backend instance.
/// Recheck mutable VM/GPU/journal facts before any journal write or effect, without
/// rediscovering and reauthenticating the already retained payload. Guest transfer
/// still verifies the admitted file hashes; the worker independently validates the
/// current full payload again after effects before committing configuration.
pub(crate) fn apply_admitted(
    backend: &mut impl Backend,
    admitted: super::AdmittedApply,
) -> Result<OperationResult, String> {
    let AdmittedApply {
        target,
        decision: d,
    } = admitted;
    let current = backend.inspect(&target)?;
    let gpu = backend.gpu(&target)?;
    let journal = backend.journal(&target)?;
    if serde_json::to_value(&current).map_err(|e| e.to_string())?
        != serde_json::to_value(&d.initial).map_err(|e| e.to_string())?
        || serde_json::to_value(&gpu).map_err(|e| e.to_string())?
            != serde_json::to_value(&d.gpu).map_err(|e| e.to_string())?
        || serde_json::to_value(&journal).map_err(|e| e.to_string())?
            != serde_json::to_value(&d.managed).map_err(|e| e.to_string())?
    {
        return Err("admitted state changed; refresh and review before effects".into());
    }
    execute_decision(backend, &target, d)
}

fn execute_decision(
    backend: &mut impl Backend,
    target: &Target,
    d: Decision,
) -> Result<OperationResult, String> {
    let Decision {
        initial,
        gpu,
        mut journal,
        restore_power,
        settings_preserved,
        disabled_settings,
        desired,
        digest,
        stale,
        assignment,
        needs_changes,
        setup_required,
        managed,
        ..
    } = d;
    let mut facts = ApplyFacts::new(initial.clone(), gpu.clone(), managed);
    facts.check(backend, target)?;
    // All preparation (first install, host-driver refresh and explicit repair)
    // converges here. Refusal precedes journaling, power and GPU/guest effects.
    // Ordinary management and verification-only operations do not inspect OS.
    if stale {
        backend.check_installation_os(target)?;
        facts.check(backend, target)?;
    }
    // Record power-only ordinary operations too, before graceful shutdown.
    if needs_changes || setup_required || initial.power != restore_power {
        journal.pending = true;
        journal.verification_only = setup_required && !needs_changes;
        if setup_required {
            journal.latest_check = Some(GuestCheck::Interrupted);
        }
        journal.preparation_started = setup_required.then_some(false);
        journal.check_restored = false;
        if stale {
            // A failed refresh may leave a mixture of the old and new payload.
            // Invalidate the old receipt before effects, including if the host
            // later rolls back to that old driver's digest.
            journal.prepared = None;
            // Historical rendering is not eligibility; latest_check records this
            // incomplete operation for both ordinary updates and explicit repair.
        }
        if setup_required {
            persist_check(backend, target, &journal)?;
        } else {
            backend.save(target, &journal)?;
        }
        facts.journal = Some(journal.clone());
        facts.check(backend, target)?;
    }
    let operation: Result<OperationResult, String> = (|| {
        if needs_changes {
            if initial.power == Power::Running {
                facts.power(backend, target, Power::Off)?;
            }
            if stale {
                facts.check(backend, target)?;
                journal.preparation_started = Some(true);
                persist_check(backend, target, &journal)?;
                facts.journal = Some(journal.clone());
                facts.check(backend, target)?;
                if assignment {
                    facts.assign(backend, target, false)?;
                }
                facts.power(backend, target, Power::Running)?;
                let prepared = backend.prepare(target)?;
                if Some(&prepared) != digest.as_ref() {
                    return Err(
                        "driver changed during preparation; reconcile and rediscover".into(),
                    );
                }
                facts.power(backend, target, Power::Off)?;
                journal.prepared = Some(prepared);
                facts.save(backend, target, &journal)?;
            }
            if target.enabled {
                if initial.settings != desired {
                    facts
                        .settings(backend, target, &desired)
                        .map_err(|e| format!("apply compatibility settings: {e}"))?;
                }
                journal.applied = desired.clone();
                journal.settings_released = false;
                facts.save(backend, target, &journal)?;
                if facts.state.gpus.is_empty() {
                    facts
                        .assign(backend, target, true)
                        .map_err(|e| format!("attach selected GPU: {e}"))?;
                }
                for resource in Resource::ALL {
                    if let Some(value) = target.allocation(resource) {
                        facts.allocation(backend, target, resource, value)?;
                    }
                }
            } else {
                if assignment {
                    facts.assign(backend, target, false)?;
                }
                if initial.settings != disabled_settings {
                    facts.settings(backend, target, &disabled_settings)?;
                }
            }
        }
        let mut verified = false;
        if setup_required {
            facts.check(backend, target)?;
            if facts.state.power != Power::Running {
                facts.power(backend, target, Power::Running)?;
            }
            let checked = check_guest(backend, target, &gpu, &mut journal);
            facts.journal = Some(journal.clone());
            checked?;
            verified = true;
        }
        facts.restore(backend, target, &restore_power)?;
        let effective = backend.inspect(target)?;
        let expected_gpus = if target.enabled {
            vec![target.gpu_interface.clone()]
        } else {
            vec![]
        };
        if effective.gpus != expected_gpus
            || effective.power != restore_power
            || (target.enabled && effective.settings != desired)
            || (!target.enabled && effective.settings != disabled_settings)
            || (target.enabled && !target.allocations_match(&effective))
        {
            return Err("independent final state verification failed".into());
        }
        journal.pending = false;
        if !target.enabled {
            journal.settings_released = true;
        }
        if verified {
            journal.last_verified = Some(verified_at());
        }
        facts.save(backend, target, &journal)?;
        Ok(OperationResult {
            effective,
            prepared: stale,
            verified,
            settings_preserved,
        })
    })();
    // Only restore power after success. A failed preparation must not trigger an
    // attached boot using partial guest files. Pending state explains the next apply.
    operation.map_err(|error| {
        if journal.verification_only && restore_power == Power::Off {
            let restoration = facts.restore(backend, target, &Power::Off);
            let checked = Err(error.clone());
            if let Err(proof) =
                record_restored_check_failure(backend, target, &mut journal, &checked, &restoration)
            {
                return proof;
            }
            return match restoration {
                Ok(()) => format!("GPU-PV verification failed: {error}"),
                Err(restoration) => format!(
                    "power restoration failed: {restoration}; GPU-PV verification failed: {error}"
                ),
            };
        }
        format!("GPU-PV operation failed: {error}")
    })
}

#[cfg(test)]
mod tests {
    use crate::model::{Allocation, Power, Resource, Target};
    use crate::workflow::test_support::{extended_target, fake, target, trace};
    use crate::workflow::{
        GuestCheck, PreviewAction, admit_approved, apply, apply_admitted, apply_approved, plan,
        plan_reinstall,
    };

    #[test]
    fn installation_os_refusal_blocks_first_install_refresh_and_repair_before_effects() {
        for path in 0..3 {
            for failure in [
                crate::guest_compatibility::unknown(),
                crate::reporting::ComponentFailure {
                    stage: None,
                    component: crate::reporting::Component::GuestCompatibility,
                    cause: crate::reporting::FailureCause::UnsupportedGuest,
                    code: None,
                }
                .into_error(),
            ] {
                let mut backend = fake();
                let target = target();
                if path != 0 {
                    apply(&mut backend, &target).unwrap();
                }
                if path == 1 {
                    backend.driver = "changed";
                }
                backend.os_checks = 0;
                backend.events.clear();
                backend.os_failure = Some(failure.clone());
                let before = serde_json::to_value(&backend.journal).unwrap();
                let writes = backend.journal_writes;
                let reviewed = if path == 2 {
                    plan_reinstall(&mut backend, &target).unwrap()
                } else {
                    plan(&mut backend, &target).unwrap()
                };
                assert_eq!(backend.os_checks, 0, "review must not query the OS");
                let error = apply_approved(&mut backend, &reviewed).unwrap_err();
                assert_eq!(error, failure);
                assert_eq!(backend.os_checks, 1);
                assert!(
                    backend.events.is_empty(),
                    "no power, preparation or GPU effects"
                );
                assert_eq!(backend.journal_writes, writes);
                assert_eq!(serde_json::to_value(&backend.journal).unwrap(), before);
            }
        }
    }

    #[test]
    fn ordinary_management_and_verification_do_not_require_an_os_query() {
        let mut backend = fake();
        let mut requested = extended_target();
        apply(&mut backend, &requested).unwrap();
        assert_eq!(backend.os_checks, 1);
        backend.os_checks = 0;
        backend.os_failure = Some(crate::guest_compatibility::unknown());
        let reviewed = plan(&mut backend, &requested).unwrap();
        apply_approved(&mut backend, &reviewed).unwrap();
        requested.compute.as_mut().unwrap().optimal -= 1;
        apply(&mut backend, &requested).unwrap();
        crate::workflow::verify(&mut backend, &requested).unwrap();
        requested.enabled = false;
        apply(&mut backend, &requested).unwrap();
        assert_eq!(backend.os_checks, 0);
    }

    #[test]
    fn preparation_drift_blocks_later_shutdown_settings_and_attachment() {
        use crate::workflow::test_support::Drift;
        for drift in [
            Drift::Settings,
            Drift::Gpu,
            Drift::Assignment,
            Drift::Resource,
            Drift::Power,
            Drift::Journal,
            Drift::Unavailable,
        ] {
            let mut backend = fake();
            let original = backend.state.settings.clone();
            backend.drift_at = Some(("prepare", drift));
            let error = apply(&mut backend, &target()).unwrap_err();
            assert!(
                error.contains("changed") || error.contains("unavailable"),
                "{error}"
            );
            assert_eq!(backend.events, ["shutdown", "start", "prepare"]);
            assert_eq!(backend.settings_writes, 0);
            assert_eq!(backend.allocation_writes, 0);
            assert!(backend.journal.as_ref().unwrap().pending);
            if !matches!(drift, Drift::Journal) {
                assert_eq!(backend.journal.as_ref().unwrap().original, original);
            }
        }
    }

    #[test]
    fn verification_only_failure_restores_power_only_with_unchanged_host_facts() {
        use crate::workflow::test_support::Drift;
        for drift in [
            Drift::Settings,
            Drift::Gpu,
            Drift::Assignment,
            Drift::Resource,
            Drift::Power,
            Drift::Journal,
            Drift::Unavailable,
        ] {
            let mut backend = fake();
            apply(&mut backend, &target()).unwrap();
            backend.journal.as_mut().unwrap().latest_check = Some(GuestCheck::Failed);
            backend.events.clear();
            backend.fail_verify = true;
            backend.drift_at = Some(("verify", drift));
            let approved = plan(&mut backend, &target()).unwrap();
            assert!(!approved.preview.actions.contains(&PreviewAction::Prepare));
            let error = apply_approved(&mut backend, &approved).unwrap_err();
            assert!(error.contains("graphics failed"), "{error}");
            assert_eq!(backend.events, ["start", "graphics"]);
            assert!(backend.journal.as_ref().unwrap().pending);
            assert_eq!(
                backend.state.power,
                if matches!(drift, Drift::Power) {
                    Power::Off
                } else {
                    Power::Running
                }
            );
        }
    }

    #[test]
    fn changed_facts_between_own_writes_block_following_effects() {
        use crate::workflow::test_support::Drift;
        let mut backend = fake();
        backend.drift_at = Some(("settings", Drift::Assignment));
        assert!(
            apply(&mut backend, &target())
                .unwrap_err()
                .contains("facts changed")
        );
        assert!(!backend.events.contains(&"attach"));
        assert!(backend.journal.as_ref().unwrap().pending);

        let mut backend = fake();
        let requested = extended_target();
        backend.drift_at = Some(("allocation", Drift::Resource));
        assert!(
            apply(&mut backend, &requested)
                .unwrap_err()
                .contains("facts changed")
        );
        assert_eq!(backend.allocation_calls, [Resource::Compute]);
        assert!(!backend.events.contains(&"graphics"));
        assert!(backend.journal.as_ref().unwrap().pending);
    }

    #[test]
    fn vram_change_and_reinstall_skip_unchanged_compatibility_writes() {
        for reinstall in [false, true] {
            let mut backend = fake();
            apply(&mut backend, &target()).unwrap();
            assert_eq!(backend.settings_writes, 1);
            let original = backend.journal.as_ref().unwrap().original.clone();
            let matching = backend.state.settings.clone();
            backend.events.clear();
            backend.fail_settings = true;
            let mut requested = target();
            let approved = if reinstall {
                plan_reinstall(&mut backend, &requested).unwrap()
            } else {
                requested.vram = Some(Allocation {
                    minimum: 10,
                    optimal: 50,
                    maximum: 90,
                });
                plan(&mut backend, &requested).unwrap()
            };
            assert!(!approved.preview.actions.contains(&PreviewAction::Settings));
            let result = apply_approved(&mut backend, &approved).unwrap();
            assert_eq!(backend.settings_writes, 1);
            assert_eq!(result.effective.settings, matching);
            assert_eq!(result.effective.power, Power::Off);
            assert!(requested.allocations_match(&result.effective));
            assert_eq!(result.prepared, reinstall);
            assert_eq!(result.verified, reinstall);
            assert_eq!(backend.events, trace(&approved.preview));
            let journal = backend.journal.as_ref().unwrap();
            assert_eq!(journal.original, original);
            assert_eq!(journal.applied, matching);
            assert!(!journal.settings_released && !journal.pending);
        }
    }

    #[test]
    fn reenable_with_matching_settings_acquires_ownership_without_a_settings_write() {
        let mut backend = fake();
        apply(&mut backend, &target()).unwrap();
        let disabled = Target {
            enabled: false,
            ..target()
        };
        apply(&mut backend, &disabled).unwrap();
        assert!(backend.journal.as_ref().unwrap().settings_released);
        backend.state.settings = backend.state.settings.for_gpu();
        let baseline = backend.state.settings.clone();
        let writes = backend.settings_writes;
        backend.fail_settings = true;
        let approved = plan(&mut backend, &target()).unwrap();
        assert!(!approved.preview.actions.contains(&PreviewAction::Settings));
        let result = apply_approved(&mut backend, &approved).unwrap();
        assert_eq!(backend.settings_writes, writes);
        assert_eq!(result.effective.gpus, [target().gpu_interface]);
        assert_eq!(result.effective.settings, baseline);
        let journal = backend.journal.as_ref().unwrap();
        assert_eq!(journal.original, baseline);
        assert_eq!(journal.applied, baseline);
        assert!(!journal.settings_released && !journal.pending);
    }

    #[test]
    fn required_compatibility_write_failure_still_blocks_attachment_and_retains_recovery() {
        let mut backend = fake();
        let original = backend.state.settings.clone();
        backend.fail_settings = true;
        let approved = plan(&mut backend, &target()).unwrap();
        assert!(approved.preview.actions.contains(&PreviewAction::Settings));
        assert!(
            apply_approved(&mut backend, &approved)
                .unwrap_err()
                .contains("apply compatibility settings")
        );
        assert_eq!(backend.settings_writes, 1);
        assert_eq!(backend.state.settings, original);
        assert!(backend.state.gpus.is_empty());
        assert!(!backend.events.contains(&"attach"));
        assert!(!backend.events.contains(&"graphics"));
        let journal = backend.journal.as_ref().unwrap();
        assert!(journal.pending);
        assert!(journal.prepared.is_some());
        assert_eq!(journal.original, original);
    }

    #[test]
    fn approved_plan_changes_block_before_any_journal_or_effect() {
        for changed in 0..6 {
            let mut backend = fake();
            let approved = plan(&mut backend, &target()).unwrap();
            match changed {
                0 => backend.state.power = Power::Off,
                1 => backend.driver = "changed-payload",
                2 => backend.state.settings.high_mmio += 1,
                3 => backend.state.vm_id = "22222222-2222-2222-2222-222222222222".into(),
                4 => backend.state.gpus.push(target().gpu_interface),
                _ => {
                    backend.state.vram = Some(Allocation {
                        minimum: 10,
                        optimal: 50,
                        maximum: 100,
                    });
                }
            }
            let error = apply_approved(&mut backend, &approved).unwrap_err();
            assert!(error.contains(if changed == 3 { "selected VM" } else { "stale" }));
            assert_eq!(backend.journal_writes, 0);
            assert!(backend.events.is_empty());
        }
        let mut backend = fake();
        let approved = plan(&mut backend, &target()).unwrap();
        assert!(apply_approved(&mut backend, &approved).unwrap().verified);
    }

    #[test]
    fn admitted_apply_authenticates_once_for_fresh_reapply_and_disable() {
        for scenario in ["fresh", "reapply", "disable"] {
            let mut backend = fake();
            let mut target = target();
            if scenario != "fresh" {
                apply(&mut backend, &target).unwrap();
            }
            if scenario == "disable" {
                target.enabled = false;
            }
            backend.payload_reads = 0;
            backend.events.clear();
            let approved = plan(&mut backend, &target).unwrap();
            let (_, admitted) = admit_approved(&mut backend, &approved).unwrap();
            assert_eq!(backend.payload_reads, if target.enabled { 2 } else { 0 });
            let before = backend.payload_reads;
            let result = apply_admitted(&mut backend, admitted).unwrap();
            assert_eq!(
                backend.payload_reads, before,
                "duplicate execution payload pass: {scenario}"
            );
            assert_eq!(result.prepared, scenario == "fresh");
            assert_eq!(result.verified, scenario == "fresh");
            assert_eq!(result.effective.power, approved.preview.restore_power);
            assert_eq!(result.effective.gpus.len(), usize::from(target.enabled));
        }
    }

    #[test]
    fn admitted_apply_rejects_vm_driver_and_journal_drift_before_writes() {
        for changed in 0..4 {
            let mut backend = fake();
            apply(&mut backend, &target()).unwrap();
            backend.state.power = Power::Running;
            let approved = plan(&mut backend, &target()).unwrap();
            let (_, admitted) = admit_approved(&mut backend, &approved).unwrap();
            backend.events.clear();
            backend.journal_writes = 0;
            match changed {
                0 => backend.state.power = Power::Off,
                1 => backend.gpu_driver = "updated-driver",
                2 => backend.journal.as_mut().unwrap().prepared = None,
                _ => backend.journal.as_mut().unwrap().pending = true,
            }
            assert!(
                apply_admitted(&mut backend, admitted)
                    .unwrap_err()
                    .contains("changed")
            );
            assert!(backend.events.is_empty());
            assert_eq!(backend.journal_writes, 0);
        }
    }

    #[test]
    fn admission_rejects_payload_drift_and_pending_recovery_without_effects() {
        for changed in 0..3 {
            let mut backend = fake();
            apply(&mut backend, &target()).unwrap();
            let mut approved = plan(&mut backend, &target()).unwrap();
            backend.events.clear();
            backend.journal_writes = 0;
            match changed {
                0 => backend.driver = "changed-payload",
                1 => backend.fail_payload = true,
                _ => {
                    backend.journal.as_mut().unwrap().pending = true;
                    // Even a matching reviewed pending plan must not admit Apply.
                    approved = plan(&mut backend, &target()).unwrap();
                }
            }
            assert!(admit_approved(&mut backend, &approved).is_err());
            assert!(backend.events.is_empty());
            assert_eq!(backend.journal_writes, 0);
        }
    }

    #[test]
    fn failed_ordinary_payload_update_retains_history_without_certifying_preparation() {
        let mut backend = fake();
        apply(&mut backend, &target()).unwrap();
        let history = backend.journal.as_ref().unwrap().last_verified;
        backend.driver = "updated-payload";
        backend.fail_prepare = true;
        let approved = plan(&mut backend, &target()).unwrap();
        assert!(!approved.reinstall);
        assert!(apply_approved(&mut backend, &approved).is_err());
        let journal = backend.journal.as_ref().unwrap();
        assert!(journal.pending && journal.prepared.is_none());
        assert_eq!(journal.latest_check, Some(GuestCheck::Interrupted));
        assert_eq!(journal.last_verified, history);
        assert!(!journal.setup_eligible());
    }

    #[test]
    fn same_digest_reinstall_runs_complete_sequence_and_preserves_resource_intent() {
        for power in [Power::Off, Power::Running] {
            let mut backend = fake();
            let target = extended_target();
            apply(&mut backend, &target).unwrap();
            backend.state.power = power.clone();
            backend.events.clear();
            let writes = backend.journal_writes;
            let reviewed = plan_reinstall(&mut backend, &target).unwrap();
            assert!(reviewed.reinstall && reviewed.preview.credentials_required);
            assert_eq!(reviewed.desired, target);
            assert_eq!(reviewed.preview.restore_power, Power::Off);
            assert_eq!(reviewed.preview.guest_downtime, power == Power::Running);
            assert!(backend.events.is_empty());
            assert_eq!(backend.journal_writes, writes);
            let result = apply_approved(&mut backend, &reviewed).unwrap();
            assert!(result.prepared && result.verified);
            assert_eq!(result.effective.power, Power::Off);
            assert!(target.allocations_match(&result.effective));
            let effects: Vec<_> = backend
                .events
                .iter()
                .copied()
                .filter(|e| !matches!(*e, "allocation" | "settings"))
                .collect();
            let mut expected = vec![
                "detach", "start", "prepare", "shutdown", "attach", "start", "graphics", "shutdown",
            ];
            if power == Power::Running {
                expected.insert(0, "shutdown");
            }
            assert_eq!(effects, expected);
            assert!(backend.journal.as_ref().unwrap().setup_eligible());
            assert!(!backend.journal.as_ref().unwrap().pending);
            let ordinary = plan(&mut backend, &target).unwrap();
            assert!(!ordinary.reinstall && !ordinary.preview.credentials_required);
            assert!(!ordinary.preview.actions.contains(&PreviewAction::Prepare));
        }
    }

    #[test]
    fn reinstall_rejects_stale_review_pending_recovery_and_disabled_intent_before_effects() {
        for scenario in 0..4 {
            let mut backend = fake();
            apply(&mut backend, &target()).unwrap();
            let mut reviewed = plan_reinstall(&mut backend, &target()).unwrap();
            match scenario {
                0 => reviewed.reinstall = false,
                1 => backend.driver = "changed-after-review",
                2 => {
                    backend.journal.as_mut().unwrap().pending = true;
                    reviewed = plan_reinstall(&mut backend, &target()).unwrap();
                }
                _ => reviewed.desired.enabled = false,
            }
            backend.events.clear();
            let writes = backend.journal_writes;
            assert!(apply_approved(&mut backend, &reviewed).is_err());
            assert!(backend.events.is_empty());
            assert_eq!(backend.journal_writes, writes);
        }
    }

    #[test]
    fn reinstall_failure_retains_recovery_and_invalidates_old_success_before_payload_work() {
        for scenario in 0..4 {
            let mut backend = fake();
            apply(&mut backend, &target()).unwrap();
            let history = backend.journal.as_ref().unwrap().last_verified;
            backend.state.power = Power::Running;
            let reviewed = plan_reinstall(&mut backend, &target()).unwrap();
            backend.events.clear();
            match scenario {
                0 => backend.fail_shutdown = true,
                1 => backend.fail_prepare = true,
                2 => backend.fail_verify = true,
                _ => backend.fail_final_shutdown = true,
            }
            assert!(apply_approved(&mut backend, &reviewed).is_err());
            let journal = backend.journal.as_ref().unwrap();
            assert!(journal.pending && !journal.verification_only);
            assert_eq!(journal.last_verified, history);
            if scenario < 2 {
                assert!(journal.prepared.is_none());
            }
            if scenario == 0 {
                assert_eq!(backend.events, ["shutdown"]);
            }
            if scenario == 1 {
                assert!(!backend.events.contains(&"attach"));
            }
            let events = backend.events.clone();
            let retry = plan_reinstall(&mut backend, &target()).unwrap();
            assert!(apply_approved(&mut backend, &retry).is_err());
            assert_eq!(backend.events, events);
        }
    }

    #[test]
    fn vram_request_above_reported_metadata_is_applied_and_read_back_exactly() {
        let mut f = fake();
        let mut t = target();
        t.vram = Some(Allocation {
            minimum: 1_000_000_000,
            optimal: 8_000_000_000,
            maximum: 16_000_000_000,
        });
        let result = apply(&mut f, &t).unwrap();
        assert_eq!(result.effective.vram, t.vram);
        assert_eq!(f.allocation_calls, [Resource::Vram]);
        assert_eq!(result.effective.power, Power::Off);
        assert!(!f.journal.as_ref().unwrap().pending);

        for rejected in [true, false] {
            let mut failed = fake();
            if rejected {
                failed.fail_allocation = Some(Resource::Vram);
            } else {
                failed.corrupt_resource = Some(Resource::Vram);
            }
            assert!(apply(&mut failed, &t).is_err());
            assert!(failed.journal.as_ref().unwrap().pending);
            assert_eq!(failed.allocation_calls, [Resource::Vram]);
        }
    }

    #[test]
    fn extended_resources_plan_write_in_order_and_independently_read_back() {
        let mut f = fake();
        let target = extended_target();
        let approved = plan(&mut f, &target).unwrap();
        assert_eq!(approved.preview.compute, target.compute);
        assert_eq!(approved.preview.encode, target.encode);
        assert_eq!(approved.preview.decode, target.decode);
        assert!(approved.preview.allocation.is_none());
        let result = apply(&mut f, &target).unwrap();
        assert_eq!(
            f.allocation_calls,
            [Resource::Compute, Resource::Encode, Resource::Decode]
        );
        assert!(target.allocations_match(&result.effective));
        assert!(!f.journal.as_ref().unwrap().pending);
    }

    #[test]
    fn omitted_categories_are_not_written_or_reset_on_existing_attachment() {
        let mut f = fake();
        let mut target = extended_target();
        apply(&mut f, &target).unwrap();
        let retained = f.state.encode.clone();
        target.encode = None;
        target.decode = None;
        target.compute.as_mut().unwrap().optimal = 60;
        f.allocation_calls.clear();
        apply(&mut f, &target).unwrap();
        assert_eq!(f.allocation_calls, [Resource::Compute]);
        assert_eq!(f.state.encode, retained);
        assert!(f.state.decode.is_some());
    }

    #[test]
    fn partial_category_failure_retains_pending_recovery_and_never_skips_forward() {
        let mut f = fake();
        f.fail_allocation = Some(Resource::Encode);
        let target = extended_target();
        assert!(apply(&mut f, &target).is_err());
        assert_eq!(f.allocation_calls, [Resource::Compute, Resource::Encode]);
        assert_eq!(f.state.compute, target.compute);
        assert!(f.state.decode.is_none());
        assert!(f.journal.as_ref().unwrap().pending);
    }

    #[test]
    fn capability_drift_after_admission_refuses_effects_even_within_new_range() {
        let mut f = fake();
        let approved = plan(&mut f, &extended_target()).unwrap();
        let (_, admitted) = admit_approved(&mut f, &approved).unwrap();
        f.resource_capability = crate::model::ResourceCapability::Supported(Allocation {
            minimum: 0,
            optimal: 60,
            maximum: 100,
        });
        assert!(apply_admitted(&mut f, admitted).is_err());
        assert!(f.events.is_empty());
        assert_eq!(f.journal_writes, 0);
        assert_eq!(f.allocation_writes, 0);
    }

    #[test]
    fn independent_extended_readback_failure_retains_pending_recovery() {
        for resource in [Resource::Compute, Resource::Encode, Resource::Decode] {
            let mut f = fake();
            f.corrupt_resource = Some(resource);
            assert!(apply(&mut f, &extended_target()).is_err());
            assert!(f.journal.as_ref().unwrap().pending);
        }
    }

    #[test]
    fn ordinary_noop_from_running_only_shuts_down_without_guest_access() {
        let mut f = fake();
        apply(&mut f, &target()).unwrap();
        f.state.power = Power::Running;
        f.fail_verify = true;
        f.fail_prepare = true;
        f.events.clear();
        let p = plan(&mut f, &target()).unwrap();
        assert!(!p.preview.credentials_required);
        assert!(!p.automatic_guest_restart);
        assert!(p.preview.guest_downtime);
        assert_eq!(p.preview.actions, [PreviewAction::Shutdown]);
        let result = apply_approved(&mut f, &p).unwrap();
        assert!(!result.verified && !result.prepared);
        assert_eq!(f.events, ["shutdown"]);
        assert_eq!(result.effective.power, Power::Off);
    }

    #[test]
    fn ordinary_toggle_and_vram_changes_finish_off_and_preserve_setup_receipts() {
        for starting_power in [Power::Off, Power::Running] {
            for change in ["disable", "enable", "vram"] {
                let mut f = fake();
                let mut t = target();
                apply(&mut f, &t).unwrap();
                if change == "enable" {
                    t.enabled = false;
                    apply(&mut f, &t).unwrap();
                    t.enabled = true;
                } else if change == "disable" {
                    t.enabled = false;
                } else {
                    t.vram = Some(Allocation {
                        minimum: 10,
                        optimal: 50,
                        maximum: 90,
                    });
                }
                let previous = f.journal.as_ref().unwrap().last_verified;
                f.state.power = starting_power.clone();
                f.events.clear();
                f.fail_prepare = true;
                f.fail_verify = true;
                let approved = plan(&mut f, &t).unwrap();
                assert!(!approved.initial_setup);
                assert!(!approved.preview.credentials_required);
                assert_eq!(approved.preview.restore_power, Power::Off);
                assert!(!approved.preview.actions.contains(&PreviewAction::Start));
                let result = apply_approved(&mut f, &approved).unwrap();
                assert_eq!(f.events, trace(&approved.preview));
                assert!(!result.prepared && !result.verified);
                assert_eq!(result.effective.power, Power::Off);
                assert_eq!(f.journal.as_ref().unwrap().last_verified, previous);
                assert!(!f.journal.as_ref().unwrap().pending);
            }
        }
    }

    #[test]
    fn ordinary_shutdown_or_allocation_failure_retains_recovery_without_guest_work() {
        for fail_shutdown in [true, false] {
            let mut f = fake();
            let mut t = extended_target();
            apply(&mut f, &t).unwrap();
            f.state.power = Power::Running;
            t.compute.as_mut().unwrap().optimal = 60;
            f.events.clear();
            f.fail_shutdown = fail_shutdown;
            if !fail_shutdown {
                f.fail_allocation = Some(Resource::Compute);
            }
            let approved = plan(&mut f, &t).unwrap();
            assert!(apply_approved(&mut f, &approved).is_err());
            assert_eq!(f.events, ["shutdown"]);
            let journal = f.journal.as_ref().unwrap();
            assert!(journal.pending && !journal.verification_only);
            assert_eq!(journal.restore_power, Power::Off);
            let pending = plan(&mut f, &t).unwrap();
            assert!(apply_approved(&mut f, &pending).is_err());
        }
    }

    #[test]
    fn ordinary_power_only_apply_requires_independent_final_off_readback() {
        let mut f = fake();
        apply(&mut f, &target()).unwrap();
        f.state.power = Power::Running;
        f.events.clear();
        f.ignore_shutdown = true;
        let approved = plan(&mut f, &target()).unwrap();
        assert!(
            apply_approved(&mut f, &approved)
                .unwrap_err()
                .contains("independent final state")
        );
        assert_eq!(f.events, ["shutdown"]);
        assert_eq!(f.state.power, Power::Running);
        let journal = f.journal.as_ref().unwrap();
        assert!(journal.pending && !journal.verification_only);
        assert_eq!(journal.restore_power, Power::Off);
    }

    #[test]
    fn ordinary_allocation_readback_mismatch_retains_recovery_without_graphics() {
        let mut f = fake();
        apply(&mut f, &target()).unwrap();
        let mut t = target();
        t.vram = Some(Allocation {
            minimum: 10,
            optimal: 50,
            maximum: 90,
        });
        f.corrupt_resource = Some(Resource::Vram);
        f.events.clear();
        let approved = plan(&mut f, &t).unwrap();
        assert!(
            apply_approved(&mut f, &approved)
                .unwrap_err()
                .contains("independent final state")
        );
        assert!(f.events.is_empty());
        assert!(f.journal.as_ref().unwrap().pending);
        assert_eq!(f.state.power, Power::Off);
    }

    #[test]
    fn refresh_reapplies_matching_raw_allocation_after_recreating_adapter() {
        let mut f = fake();
        let mut t = target();
        t.vram = Some(Allocation {
            minimum: 10,
            maximum: 90,
            optimal: 50,
        });
        apply(&mut f, &t).unwrap();
        assert_eq!(f.state.vram, t.vram);
        f.driver = "updated";
        f.events.clear();
        f.allocation_writes = 0;
        let p = plan(&mut f, &t).unwrap();
        assert_eq!(p.preview.allocation, t.vram);
        assert!(p.preview.actions.contains(&PreviewAction::Detach));
        assert!(p.preview.actions.contains(&PreviewAction::Attach));
        assert!(p.preview.actions.contains(&PreviewAction::Allocation));
        apply(&mut f, &t).unwrap();
        assert_eq!(f.allocation_writes, 1);
        assert_eq!(trace(&p.preview), f.events);
        assert_eq!(f.state.vram, t.vram);
    }

    #[test]
    fn prepare_precedes_attachment_noop_has_no_restart_disable_preserves_files() {
        let mut f = fake();
        let mut t = target();
        assert!(apply(&mut f, &t).unwrap().verified);
        assert_eq!(
            f.events,
            [
                "shutdown", "start", "prepare", "shutdown", "attach", "start", "graphics",
                "shutdown"
            ]
        );
        // The user starts the completed setup through Hyper-V before ordinary work.
        f.state.power = Power::Running;
        f.events.clear();
        assert!(!apply(&mut f, &t).unwrap().prepared);
        assert_eq!(f.events, ["shutdown"]);
        t.enabled = false;
        f.events.clear();
        apply(&mut f, &t).unwrap();
        assert_eq!(f.events, ["detach"]);
        assert_eq!(
            f.journal.as_ref().unwrap().prepared.as_deref(),
            Some("current")
        );
        assert!(!f.state.settings.cache_types);
    }

    #[test]
    fn initial_setup_from_off_or_running_verifies_and_finishes_off() {
        for starting_power in [Power::Off, Power::Running] {
            let mut backend = fake();
            backend.state.power = starting_power.clone();
            let approved = plan(&mut backend, &target()).unwrap();
            assert!(approved.initial_setup);
            assert_eq!(approved.observed.power, starting_power);
            assert_eq!(approved.preview.restore_power, Power::Off);
            assert_eq!(
                approved.preview.actions.last(),
                Some(&PreviewAction::Shutdown)
            );
            assert_eq!(backend.journal_writes, 0);
            assert!(backend.events.is_empty());
            let result = apply_approved(&mut backend, &approved).unwrap();
            assert!(result.prepared && result.verified);
            assert_eq!(result.effective.power, Power::Off);
            assert_eq!(backend.events, trace(&approved.preview));
            let journal = backend.journal.as_ref().unwrap();
            assert!(!journal.pending);
            assert!(journal.prepared.is_some() && journal.last_verified.is_some());
            assert!(!plan(&mut backend, &target()).unwrap().initial_setup);
            backend.state.power = Power::Running;
            let later = plan(&mut backend, &target()).unwrap();
            assert!(!later.initial_setup);
            assert_eq!(later.preview.restore_power, Power::Off);
        }
    }

    #[test]
    fn initial_setup_failures_retain_recovery_and_cannot_be_replayed_by_approval() {
        for failure in 0..5 {
            let mut backend = fake();
            match failure {
                0 => backend.fail_shutdown = true,
                1 => backend.fail_prepare = true,
                2 => backend.fail_verify = true,
                3 => backend.fail_final_shutdown = true,
                _ => backend.ignore_final_shutdown = true,
            }
            let approved = plan(&mut backend, &target()).unwrap();
            assert!(apply_approved(&mut backend, &approved).is_err());
            let journal = backend.journal.as_ref().unwrap();
            assert!(journal.pending);
            assert_eq!(journal.restore_power, Power::Off);
            assert!(journal.last_verified.is_none());
            if failure < 2 {
                assert!(!backend.events.contains(&"attach"));
                assert!(!backend.events.contains(&"graphics"));
            }
            let writes = backend.journal_writes;
            let events = backend.events.clone();
            let pending = plan(&mut backend, &target()).unwrap();
            assert!(pending.preview.pending_recovery);
            assert!(apply_approved(&mut backend, &pending).is_err());
            assert_eq!(backend.journal_writes, writes);
            assert_eq!(backend.events, events);
        }
    }

    #[test]
    fn partial_preparation_is_durable_and_never_attached() {
        let mut f = fake();
        f.fail_prepare = true;
        assert!(
            apply(&mut f, &target())
                .unwrap_err()
                .contains("interrupted")
        );
        assert!(f.journal.as_ref().unwrap().pending);
        assert!(!f.events.contains(&"attach"));
        f.fail_prepare = false;
        assert!(apply(&mut f, &target()).is_ok());
    }

    #[test]
    fn unrelated_gpu_is_rejected_before_mutation() {
        let mut f = fake();
        f.state.gpus.push("another".into());
        assert!(apply(&mut f, &target()).is_err());
        assert!(f.events.is_empty());
        assert!(f.journal.is_none());
    }

    #[test]
    fn interrupted_disable_restores_settings_and_finishes_off_on_retry() {
        let mut f = fake();
        let mut t = target();
        apply(&mut f, &t).unwrap();
        f.state.power = Power::Running;
        t.enabled = false;
        f.fail_settings = true;
        assert!(apply(&mut f, &t).is_err());
        assert!(f.state.gpus.is_empty());
        assert_eq!(f.state.power, Power::Off);
        assert!(f.journal.as_ref().unwrap().pending);
        f.fail_settings = false;
        apply(&mut f, &t).unwrap();
        assert_eq!(f.state.power, Power::Off);
        assert!(!f.state.settings.cache_types);
    }

    #[test]
    fn removal_after_missing_preparation_or_adapter_is_host_only_and_finishes_off() {
        for power in [Power::Off, Power::Running] {
            for missing in 0..3 {
                let mut backend = fake();
                let mut target = extended_target();
                apply(&mut backend, &target).unwrap();
                backend.state.power = power.clone();
                let initial_settings = backend.state.settings.clone();
                let original = backend.journal.as_ref().unwrap().original.clone();
                if missing == 0 {
                    let journal = backend.journal.as_mut().unwrap();
                    journal.prepared = None;
                    journal.last_verified = None;
                    journal.latest_check = Some(GuestCheck::PayloadMismatch);
                } else if missing == 1 {
                    backend.state.gpus.clear();
                } else {
                    backend.journal = None;
                }
                target.enabled = false;
                backend.events.clear();
                backend.payload_reads = 0;
                backend.fail_payload = true;
                backend.fail_prepare = true;
                backend.fail_verify = true;
                let allocation_writes = backend.allocation_writes;
                let reviewed = plan(&mut backend, &target).unwrap();
                assert!(!reviewed.preview.credentials_required && !reviewed.reinstall);
                assert!(!reviewed.preview.actions.iter().any(|action| matches!(
                    action,
                    PreviewAction::Start
                        | PreviewAction::Prepare
                        | PreviewAction::Verify
                        | PreviewAction::Allocation
                )));
                let result = apply_approved(&mut backend, &reviewed).unwrap();
                assert!(!result.prepared && !result.verified);
                assert_eq!(result.effective.power, Power::Off);
                assert!(result.effective.gpus.is_empty());
                assert_eq!(backend.payload_reads, 0);
                assert_eq!(backend.allocation_writes, allocation_writes);
                assert_eq!(
                    result.effective.settings,
                    if missing == 2 {
                        initial_settings
                    } else {
                        original
                    }
                );
                assert!(!backend.journal.as_ref().unwrap().pending);
                assert!(
                    !backend
                        .events
                        .iter()
                        .any(|event| matches!(*event, "start" | "graphics" | "prepare" | "attach"))
                );
            }
        }
    }

    #[test]
    fn reenabling_an_externally_detached_adapter_retains_original_settings_for_removal() {
        let mut backend = fake();
        let original = backend.state.settings.clone();
        apply(&mut backend, &target()).unwrap();
        backend.state.gpus.clear();
        let reviewed = plan(&mut backend, &target()).unwrap();
        assert!(!reviewed.preview.credentials_required);
        apply_approved(&mut backend, &reviewed).unwrap();
        assert_eq!(backend.journal.as_ref().unwrap().original, original);
        let disabled = Target {
            enabled: false,
            ..target()
        };
        apply(&mut backend, &disabled).unwrap();
        assert_eq!(backend.state.settings, original);
        assert!(backend.journal.as_ref().unwrap().settings_released);
    }

    #[test]
    fn reenabling_released_ownership_captures_current_baseline_even_with_external_attachment() {
        let mut backend = fake();
        apply(&mut backend, &target()).unwrap();
        let disabled = Target {
            enabled: false,
            ..target()
        };
        apply(&mut backend, &disabled).unwrap();
        backend.state.settings.high_mmio = 1024;
        let baseline = backend.state.settings.clone();
        backend.state.gpus = vec![target().gpu_interface];
        apply(&mut backend, &target()).unwrap();
        assert_eq!(backend.journal.as_ref().unwrap().original, baseline);
        apply(&mut backend, &disabled).unwrap();
        assert_eq!(backend.state.settings, baseline);
    }

    #[test]
    fn completed_removal_releases_ownership_without_erasing_history() {
        let mut backend = fake();
        let mut disabled = target();
        apply(&mut backend, &disabled).unwrap();
        let history = backend.journal.as_ref().unwrap().last_verified;
        let prepared = backend.journal.as_ref().unwrap().prepared.clone();
        disabled.enabled = false;
        let reviewed = plan(&mut backend, &disabled).unwrap();
        apply_approved(&mut backend, &reviewed).unwrap();
        assert!(backend.journal.as_ref().unwrap().settings_released);
        assert_eq!(backend.journal.as_ref().unwrap().last_verified, history);
        assert_eq!(backend.journal.as_ref().unwrap().prepared, prepared);
        // Another manager uses values identical to old GPU settings after removal.
        backend.state.settings = backend.state.settings.for_gpu();
        let external = backend.state.settings.clone();
        let reviewed = plan(&mut backend, &disabled).unwrap();
        assert!(!reviewed.preview.actions.contains(&PreviewAction::Settings));
        apply_approved(&mut backend, &reviewed).unwrap();
        assert_eq!(backend.state.settings, external);
        disabled.enabled = true;
        let reviewed = plan(&mut backend, &disabled).unwrap();
        apply_approved(&mut backend, &reviewed).unwrap();
        assert!(!backend.journal.as_ref().unwrap().settings_released);
        assert_eq!(backend.journal.as_ref().unwrap().original, external);
    }

    #[test]
    fn removal_rejects_unrelated_hardware_recovery_and_stale_review_without_effects() {
        for scenario in 0..3 {
            let mut backend = fake();
            apply(&mut backend, &target()).unwrap();
            let mut disabled = target();
            disabled.enabled = false;
            let mut approved = plan(&mut backend, &disabled).unwrap();
            match scenario {
                0 => backend.state.gpus = vec!["another-gpu".into()],
                1 => {
                    backend.journal.as_mut().unwrap().pending = true;
                    approved = plan(&mut backend, &disabled).unwrap();
                }
                _ => backend.state.settings.high_mmio += 1,
            }
            backend.events.clear();
            let writes = backend.journal_writes;
            assert!(apply_approved(&mut backend, &approved).is_err());
            assert!(backend.events.is_empty());
            assert_eq!(backend.journal_writes, writes);
        }
    }

    #[test]
    fn partial_removal_and_settings_changes_during_detach_retain_recovery() {
        for external_change in [false, true] {
            let mut backend = fake();
            apply(&mut backend, &target()).unwrap();
            let before = backend.state.settings.clone();
            let mut disabled = target();
            disabled.enabled = false;
            let approved = plan(&mut backend, &disabled).unwrap();
            backend.events.clear();
            backend.change_settings_on_detach = external_change;
            backend.fail_settings = !external_change;
            assert!(apply_approved(&mut backend, &approved).is_err());
            assert!(backend.state.gpus.is_empty());
            let mut expected = before;
            if external_change {
                expected.high_mmio += 1;
            }
            assert_eq!(backend.state.settings, expected);
            assert!(backend.journal.as_ref().unwrap().pending);
            let events = backend.events.clone();
            let retry = plan(&mut backend, &disabled).unwrap();
            assert!(apply_approved(&mut backend, &retry).is_err());
            assert_eq!(backend.events, events);
        }
    }

    #[test]
    fn disable_preserves_external_settings_and_completes_power_restoration() {
        let mut f = fake();
        let mut t = target();
        apply(&mut f, &t).unwrap();
        f.state.power = Power::Running;
        f.state.settings.high_mmio += 1024;
        let external_high_mmio = f.state.settings.high_mmio;
        let original = f.journal.as_ref().unwrap().original.clone();
        t.enabled = false;
        assert!(apply(&mut f, &t).unwrap().settings_preserved);
        assert_eq!(f.state.power, Power::Off);
        let mut expected = original;
        expected.high_mmio = external_high_mmio;
        assert_eq!(f.state.settings, expected);
        assert!(!f.journal.as_ref().unwrap().pending);
    }

    #[test]
    fn shutdown_failure_never_prepares_or_detaches() {
        let mut f = fake();
        f.fail_shutdown = true;
        assert!(
            apply(&mut f, &target())
                .unwrap_err()
                .contains("shutdown refused")
        );
        assert_eq!(f.events, ["shutdown"]);
        assert_eq!(f.state.power, Power::Running);
        assert!(f.journal.as_ref().unwrap().pending);
    }

    #[test]
    fn completed_preparation_survives_settings_failure_without_retransfer() {
        let mut f = fake();
        f.fail_settings = true;
        assert!(
            apply(&mut f, &target())
                .unwrap_err()
                .contains("apply compatibility settings")
        );
        assert_eq!(
            f.journal.as_ref().unwrap().prepared.as_deref(),
            Some("current")
        );
        assert!(f.journal.as_ref().unwrap().pending);
        f.fail_settings = false;
        f.events.clear();
        assert!(!apply(&mut f, &target()).unwrap().prepared);
        assert_eq!(f.events, ["attach", "start", "graphics", "shutdown"]);
        assert!(!f.journal.as_ref().unwrap().pending);
    }

    #[test]
    fn interrupted_refresh_requires_preparation_after_host_driver_rollback() {
        let mut f = fake();
        apply(&mut f, &target()).unwrap();
        f.driver = "new-driver";
        f.fail_prepare = true;
        assert!(
            apply(&mut f, &target())
                .unwrap_err()
                .contains("interrupted")
        );
        assert!(f.state.gpus.is_empty());
        f.driver = "current";
        f.fail_prepare = false;
        f.events.clear();
        assert!(apply(&mut f, &target()).unwrap().prepared);
        let preparation = f.events.iter().position(|e| *e == "prepare").unwrap();
        let attachment = f.events.iter().position(|e| *e == "attach").unwrap();
        assert!(preparation < attachment);
        assert!(!f.journal.as_ref().unwrap().pending);
    }
}
