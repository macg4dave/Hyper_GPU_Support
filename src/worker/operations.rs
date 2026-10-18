//! Locked Apply/Verify execution and their shared publication boundary.
use super::{
    progress::ObservedBackend,
    protocol::{Outcome, Progress, StageStatus},
    runner,
};
use crate::{
    configuration_store::{self, Revision, SavePhase, SaveRecord},
    model::{Configuration, Gpu, VmState},
    runner::NativeBackend,
    workflow::{self, Backend, OperationResult},
};

pub(super) struct Context<'a> {
    pub enrollment: &'a runner::Enrollment,
    pub session: &'a str,
    pub data: &'a std::path::Path,
    pub recovery_path: &'a std::path::Path,
}

/// Private consumed tokens can only originate in this worker's preparation.
pub(super) enum Admitted {
    Apply(Box<workflow::AdmittedApply>),
    Verify(Box<workflow::AdmittedVerification>),
}

pub(super) fn execute(
    context: &Context<'_>,
    review: super::preparation::PreparedReview,
    mut backend: NativeBackend,
    admitted: Admitted,
    mut notify: impl FnMut(Progress) -> Result<(), String>,
) -> Result<Outcome, String> {
    let mut record = recovery_record(&review, context.session)?;
    runner::require_no_vm_recovery(&record.target.vm_id)?;
    crate::windows_wmi::require_no_pending_operation_for(&record.target.vm_id)?;
    record.target.validate()?;
    if !context.enrollment.targets.iter().any(|target| {
        target.vm_id == record.target.vm_id && target.gpu_interface == record.target.gpu_interface
    }) {
        return Err("Prepared target is outside enrollment".into());
    }
    if configuration_store::protected::revision(&record.target.vm_id)? != record.expected {
        return Err("Committed configuration changed during preparation; Refresh".into());
    }
    if record.publication_required && record.expected == Revision::Missing {
        configuration_store::protected::check_capacity(std::slice::from_ref(&record.target.vm_id))?;
    }
    let gpu = match &review {
        super::preparation::PreparedReview::Apply { plan, .. } => &plan.gpu,
        super::preparation::PreparedReview::Verify { plan, .. } => &plan.gpu,
    };
    let result = run_admitted(
        &mut backend,
        &mut record,
        gpu,
        admitted,
        &mut notify,
        |record| runner::write_recovery_at(context.recovery_path, record),
    )?;
    if record.publication_required {
        publish(context, record, result, notify)
    } else {
        Ok(Outcome {
            operation: Some(result),
            saved: false,
            finished: true,
            operation_id: record.operation_id,
            save_error: None,
        })
    }
}

/// Build one receipt format while retaining each operation's publication/power scope.
fn recovery_record(
    review: &super::preparation::PreparedReview,
    session: &str,
) -> Result<SaveRecord, String> {
    use super::preparation::PreparedReview;
    let (target, initial, gpu, managed, expected) = match review {
        PreparedReview::Apply { plan, expected } => (
            &plan.desired,
            &plan.observed,
            &plan.gpu,
            &plan.managed,
            expected,
        ),
        PreparedReview::Verify { plan, expected } => (
            &plan.target,
            &plan.observed,
            &plan.gpu,
            &plan.managed,
            expected,
        ),
    };
    let plan_bytes = match review {
        PreparedReview::Apply { plan, .. } => serde_json::to_vec(plan),
        PreparedReview::Verify { plan, .. } => serde_json::to_vec(plan),
    }
    .map_err(|e| e.to_string())?;
    let mut record = SaveRecord {
        schema: 1,
        retirement: None,
        reset: None,
        operation_id: session.into(),
        target: target.clone(),
        expected: expected.clone(),
        driver_version: gpu.driver_version.clone(),
        payload_digest: None,
        plan_revision: Revision::of(&plan_bytes),
        intended_source: Configuration {
            schema: 3,
            targets: vec![target.clone()],
        }
        .vm_documents()?
        .into_values()
        .next()
        .ok_or("missing receipt intent")?,
        publication_required: false,
        initial: initial.clone(),
        expected_power: None,
        graphics_required: Some(true),
        initial_journal: managed.clone(),
        expected_settings: initial.settings.clone(),
        phase: SavePhase::Admitted,
    };
    match review {
        PreparedReview::Apply { plan, .. } => {
            record.publication_required = true;
            record.payload_digest = plan.preparation.as_ref().map(|p| p.digest.clone());
            record.expected_power = Some(plan.preview.restore_power.clone());
            record.graphics_required = Some(
                plan.preview
                    .actions
                    .contains(&workflow::PreviewAction::Verify),
            );
            if let Some(settings) = &plan.preview.settings {
                record.expected_settings = settings.after.clone();
            }
        }
        PreparedReview::Verify { plan, .. } => {
            record.payload_digest = Some(plan.payload_digest.clone());
            record.expected_power = Some(workflow::verification_power(
                target,
                initial,
                managed.as_ref(),
            )?);
        }
    }
    Ok(record)
}

/// Persist before dispatch, confirm independently, and retain every uncertain failure.
fn run_admitted(
    backend: &mut impl Backend,
    record: &mut SaveRecord,
    gpu: &Gpu,
    admitted: Admitted,
    notify: &mut impl FnMut(Progress) -> Result<(), String>,
    mut persist: impl FnMut(&SaveRecord) -> Result<(), String>,
) -> Result<OperationResult, String> {
    persist(record)?;
    let result = {
        let mut mark_effects = || {
            if matches!(record.phase, SavePhase::Admitted) {
                let mut starting = record.clone();
                starting.phase = SavePhase::EffectsStarted;
                persist(&starting)?;
                record.phase = starting.phase;
            }
            Ok(())
        };
        let mut observed = ObservedBackend {
            backend,
            notify,
            before_effect: Some(&mut mark_effects),
        };
        match admitted {
            Admitted::Apply(token) => workflow::apply_admitted(&mut observed, *token),
            Admitted::Verify(token) => workflow::verify_admitted(&mut observed, *token),
        }
    }?;
    ObservedBackend {
        backend,
        notify,
        before_effect: None,
    }
    .stage_observed(
        "Confirm final VM state and signed driver files",
        |backend| {
            confirm(
                backend,
                &record.target,
                &result.effective,
                gpu,
                record.payload_digest.as_deref(),
            )
        },
    )?;
    record.phase = SavePhase::Verified(result.effective.clone());
    persist(record)?;
    Ok(result)
}

/// Publish only an independently verified result; callers keep their own
/// operation/recovery checks and the common lock through terminal audit.
pub(super) fn publish(
    context: &Context<'_>,
    mut record: SaveRecord,
    result: OperationResult,
    mut notify: impl FnMut(Progress) -> Result<(), String>,
) -> Result<Outcome, String> {
    let enrollment = context.enrollment;
    notify(Progress {
        stage: "Publish checked VM configuration".into(),
        status: StageStatus::Running,
    })?;
    let saved = configuration_store::protected::publish(&record, &enrollment.client_sid);
    if saved.is_ok() {
        record.phase = SavePhase::Published(result.effective.clone());
        runner::write_recovery_at(context.recovery_path, &record)?;
    }
    let delivery = notify(Progress {
        stage: "Publish checked VM configuration".into(),
        status: if saved.is_ok() {
            StageStatus::Done
        } else {
            StageStatus::Failed
        },
    });
    if let Err(delivery) = delivery {
        return Err(saved.err().map_or_else(
    || format!("Publication completed but progress delivery failed; outcome uncertain: {delivery}"),
    |primary| format!("Progress delivery failed: {delivery}; {primary}"),
));
    }
    Ok(Outcome {
        operation: Some(result),
        saved: saved.is_ok(),
        finished: saved.is_ok(),
        operation_id: record.operation_id,
        save_error: saved.err(),
    })
}

/// Always an independent full final pass, including after prepared admission.
fn confirm(
    backend: &mut impl Backend,
    target: &crate::model::Target,
    effective: &VmState,
    gpu: &Gpu,
    digest: Option<&str>,
) -> Result<(), String> {
    crate::work_counts::add(crate::work_counts::Counter::CompletionReads, 1);
    if backend.inspect(target)?.operational() != effective.operational()
        || backend.gpu(target)?.operational() != gpu.operational()
    {
        return Err(
            "Final independent VM/GPU readback changed; manual reconciliation required".into(),
        );
    }
    if let Some(digest) = digest
        && backend.payload(target)? != digest
    {
        return Err("Signed payload changed during effects; manual reconciliation required".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::test_support::{fake, target};

    use super::super::preparation::PreparedReview;
    use crate::model::Power;

    #[test]
    fn shared_receipts_keep_exact_configure_and_verify_recovery_scope() {
        for scenario in 0..6 {
            let mut backend = fake();
            let mut target = target();
            if scenario > 0 {
                workflow::apply(&mut backend, &target).unwrap();
            }
            if scenario == 1 {
                target.vram = Some(crate::model::Allocation {
                    minimum: 10,
                    optimal: 50,
                    maximum: 90,
                });
            }
            if scenario == 2 {
                target.enabled = false;
            }
            if scenario == 5 {
                backend.state.power = Power::Running;
            }
            let expected = Revision::of(b"captured saved settings");
            let review = if scenario >= 4 {
                let (plan, _) = workflow::prepare_verification(&mut backend, &target).unwrap();
                PreparedReview::Verify {
                    plan,
                    expected: expected.clone(),
                }
            } else {
                let (plan, _) =
                    workflow::prepare_apply(&mut backend, &target, scenario == 3).unwrap();
                PreparedReview::Apply {
                    plan,
                    expected: expected.clone(),
                }
            };
            let record = recovery_record(&review, "original-session").unwrap();
            assert_eq!(record.operation_id, "original-session");
            assert_eq!(record.target, target);
            assert_eq!(record.expected, expected);
            assert!(matches!(record.phase, SavePhase::Admitted));
            assert!(record.reset.is_none() && record.retirement.is_none());
            assert_eq!(
                Configuration::parse(&record.intended_source)
                    .unwrap()
                    .targets,
                vec![target]
            );
            assert_eq!(
                serde_json::to_value(&record.initial).unwrap(),
                serde_json::to_value(&backend.state).unwrap()
            );
            match review {
                PreparedReview::Apply { plan, .. } => {
                    assert!(record.publication_required);
                    assert_eq!(
                        record.expected_power.as_ref(),
                        Some(&plan.preview.restore_power)
                    );
                    assert_eq!(
                        record.graphics_required,
                        Some(scenario == 0 || scenario == 3)
                    );
                    assert_eq!(
                        record.payload_digest.as_deref(),
                        plan.preparation.as_ref().map(|p| p.digest.as_str())
                    );
                    assert_eq!(
                        record.expected_settings,
                        if scenario == 2 {
                            plan.managed.as_ref().unwrap().original.clone()
                        } else {
                            backend.state.settings.for_gpu()
                        }
                    );
                    assert_eq!(
                        record.plan_revision,
                        Revision::of(&serde_json::to_vec(&plan).unwrap())
                    );
                }
                PreparedReview::Verify { plan, .. } => {
                    assert!(!record.publication_required);
                    assert_eq!(record.expected_power.as_ref(), Some(&backend.state.power));
                    assert_eq!(record.graphics_required, Some(true));
                    assert_eq!(
                        record.payload_digest.as_deref(),
                        Some(plan.payload_digest.as_str())
                    );
                    assert_eq!(record.expected_settings, backend.state.settings);
                    assert_eq!(
                        record.plan_revision,
                        Revision::of(&serde_json::to_vec(&plan).unwrap())
                    );
                }
            }
        }
    }

    #[test]
    fn shared_lifecycle_persists_before_effects_and_never_certifies_interrupted_completion() {
        for operation in 0..3 {
            for fault in 0..9 {
                let mut backend = fake();
                let target = target();
                let (review, admitted) = if operation == 0 {
                    let (plan, token) =
                        workflow::prepare_apply(&mut backend, &target, false).unwrap();
                    (
                        PreparedReview::Apply {
                            plan,
                            expected: Revision::Missing,
                        },
                        Admitted::Apply(Box::new(token)),
                    )
                } else {
                    workflow::apply(&mut backend, &target).unwrap();
                    backend.state.power = if operation == 1 {
                        Power::Off
                    } else {
                        Power::Running
                    };
                    let (plan, token) =
                        workflow::prepare_verification(&mut backend, &target).unwrap();
                    (
                        PreparedReview::Verify {
                            plan,
                            expected: Revision::Missing,
                        },
                        Admitted::Verify(Box::new(token)),
                    )
                };
                let mut record = recovery_record(&review, "session").unwrap();
                let gpu = backend.gpu(&target).unwrap();
                backend.events.clear();
                let journal_writes = backend.journal_writes;
                backend.fail_verify = fault == 3;
                backend.fail_payload = fault == 4;
                if fault == 8 {
                    backend.state.settings.high_mmio += 1;
                }
                let mut durable = Vec::new();
                let mut updates = Vec::new();
                let result = run_admitted(
                    &mut backend,
                    &mut record,
                    &gpu,
                    admitted,
                    &mut |update| {
                        let fail = (fault == 6
                            && update.stage == "Confirm final VM state and signed driver files"
                            && update.status == StageStatus::Done)
                            || (fault == 7
                                && update.stage == "Check guest health and hardware rendering"
                                && update.status == StageStatus::Done);
                        updates.push(update);
                        if fail {
                            Err("terminal progress pipe lost".into())
                        } else {
                            Ok(())
                        }
                    },
                    |record| {
                        if (fault == 1 && matches!(record.phase, SavePhase::Admitted))
                            || (fault == 2 && matches!(record.phase, SavePhase::EffectsStarted))
                            || (fault == 5 && matches!(record.phase, SavePhase::Verified(_)))
                        {
                            return Err("protected receipt write failed".into());
                        }
                        durable.push(record.clone());
                        Ok(())
                    },
                );
                assert_eq!(
                    result.is_ok(),
                    fault == 0,
                    "operation {operation} fault {fault}: {result:?}"
                );
                assert!(
                    !durable
                        .iter()
                        .any(|r| matches!(r.phase, SavePhase::Published(_)))
                );
                if fault == 0 {
                    assert_eq!(durable.len(), 3);
                    assert!(matches!(durable[0].phase, SavePhase::Admitted));
                    assert!(matches!(durable[1].phase, SavePhase::EffectsStarted));
                    assert!(matches!(durable[2].phase, SavePhase::Verified(_)));
                    assert_eq!(
                        Some(&result.unwrap().effective.power),
                        record.expected_power.as_ref()
                    );
                    assert!(updates.iter().any(|p| p.stage
                        == "Confirm final VM state and signed driver files"
                        && p.status == StageStatus::Done));
                } else if fault == 1 {
                    assert!(durable.is_empty());
                    assert!(backend.events.is_empty());
                    assert_eq!(backend.journal_writes, journal_writes);
                } else {
                    assert!(
                        !durable
                            .iter()
                            .any(|r| matches!(r.phase, SavePhase::Verified(_)))
                    );
                    assert!(
                        matches!(durable.last().unwrap().phase, SavePhase::Admitted)
                            == (fault == 2 || fault == 8)
                    );
                    if fault == 2 || fault == 8 {
                        assert!(backend.events.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    fn failed_effect_marker_cannot_be_bypassed_by_setup_check_power_restoration() {
        for persisted_before_error in [false, true] {
            let mut backend = fake();
            let target = target();
            workflow::apply(&mut backend, &target).unwrap();
            // Current prepared files without a successful check need only graphics
            // and final shutdown. A failed first marker must also block that restoration.
            backend.state.power = Power::Running;
            let mut journal = backend.journal(&target).unwrap().unwrap();
            journal.last_verified = None;
            backend.save(&target, &journal).unwrap();
            let (plan, token) = workflow::prepare_apply(&mut backend, &target, false).unwrap();
            assert!(
                plan.preview
                    .actions
                    .contains(&workflow::PreviewAction::Verify)
            );
            assert!(
                !plan
                    .preview
                    .actions
                    .contains(&workflow::PreviewAction::Prepare)
            );
            let gpu = plan.gpu.clone();
            let review = PreparedReview::Apply {
                plan,
                expected: Revision::Missing,
            };
            let mut record = recovery_record(&review, "session").unwrap();
            let mut durable = Vec::new();
            backend.events.clear();
            let error = run_admitted(
                &mut backend,
                &mut record,
                &gpu,
                Admitted::Apply(Box::new(token)),
                &mut |_| Ok(()),
                |record| {
                    if matches!(record.phase, SavePhase::EffectsStarted) {
                        if persisted_before_error {
                            durable.push(record.phase.clone());
                        }
                        return Err("effect receipt write refused".into());
                    }
                    durable.push(record.phase.clone());
                    Ok(())
                },
            )
            .unwrap_err();
            assert!(error.contains("effect receipt write refused"), "{error}");
            assert!(
                backend.events.is_empty(),
                "restoration must durably mark effects too: {:?}",
                backend.events
            );
            assert_eq!(backend.state.power, Power::Running);
            assert!(matches!(durable[0], SavePhase::Admitted));
            assert_eq!(
                matches!(durable.last().unwrap(), SavePhase::EffectsStarted),
                persisted_before_error
            );
            assert!(
                !durable
                    .iter()
                    .any(|phase| matches!(phase, SavePhase::Verified(_) | SavePhase::Published(_)))
            );
            assert!(matches!(record.phase, SavePhase::Admitted));
        }
    }

    #[test]
    fn final_confirmation_authenticates_independently_and_rejects_failure_or_same_version_byte_drift()
     {
        for fault in 0..4 {
            let mut backend = fake();
            let target = target();
            let (plan, token) = workflow::prepare_apply(&mut backend, &target, false).unwrap();
            let result = workflow::apply_admitted(&mut backend, token).unwrap();
            backend.events.clear();
            let writes = backend.journal_writes;
            let passes = backend.payload_reads;
            match fault {
                1 => backend.fail_payload = true,
                2 => backend.driver = "same-version-changed-bytes-or-membership",
                3 => backend.state.settings.high_mmio += 1,
                _ => {}
            }
            let confirmed = confirm(
                &mut backend,
                &target,
                &result.effective,
                &plan.gpu,
                Some(&plan.preparation.unwrap().digest),
            );
            assert_eq!(confirmed.is_err(), fault != 0);
            assert_eq!(
                backend.gpu(&target).unwrap().driver_version,
                plan.gpu.driver_version
            );
            assert_eq!(backend.payload_reads - passes, usize::from(fault != 3));
            assert_eq!(
                backend.journal_writes, writes,
                "confirmation cannot publish or retire recovery"
            );
            assert!(backend.events.is_empty());
        }
    }
}
