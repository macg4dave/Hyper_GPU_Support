//! Locked Apply/Verify execution and their shared publication boundary.
use super::{
    progress::ObservedBackend,
    protocol::{Outcome, Progress, StageStatus},
    runner,
};
use crate::{
    configuration_store::{self, Revision, SavePhase, SaveRecord},
    model::{Configuration, Gpu, Power, VmState},
    runner::NativeBackend,
    workflow::{self, Backend, OperationResult, Plan},
};

pub(super) struct Context<'a> {
    pub enrollment: &'a runner::Enrollment,
    pub session: &'a str,
    pub data: &'a std::path::Path,
    pub recovery_path: &'a std::path::Path,
}

pub(super) fn apply(
    context: &Context<'_>,
    approved: Plan,
    expected: Revision,
    shutdown_approved: bool,
    (mut backend, admitted): (NativeBackend, workflow::AdmittedApply),
    mut notify: impl FnMut(Progress) -> Result<(), String>,
) -> Result<Outcome, String> {
    let enrollment = context.enrollment;
    let session = context.session;
    let recovery_path = Some(context.recovery_path.to_path_buf());
    runner::require_no_vm_recovery(&approved.desired.vm_id)?;
    crate::windows_wmi::require_no_pending_operation_for(&approved.desired.vm_id)?;
    approved.desired.validate()?;
    if !enrollment.targets.iter().any(|target| {
        target.vm_id == approved.desired.vm_id
            && target.gpu_interface == approved.desired.gpu_interface
    }) {
        return Err("selected pair requires administrator enrollment before Apply".into());
    }
    if approved.preview.guest_downtime && !shutdown_approved {
        return Err("graceful guest shutdown was not separately approved".into());
    }
    if configuration_store::protected::revision(&approved.desired.vm_id)? != expected {
        return Err("committed configuration changed before admission; Refresh".into());
    }
    if expected == Revision::Missing {
        configuration_store::protected::check_capacity(std::slice::from_ref(
            &approved.desired.vm_id,
        ))?;
    }
    let mut record = SaveRecord {
        schema: 1,
        retirement: None,
        reset: None,
        operation_id: session.into(),
        target: approved.desired.clone(),
        expected,
        driver_version: admitted.driver_version().to_owned(),
        payload_digest: approved.preparation.as_ref().map(|p| p.digest.clone()),
        plan_revision: Revision::of(&serde_json::to_vec(&approved).map_err(|e| e.to_string())?),
        intended_source: Configuration {
            schema: 3,
            targets: vec![approved.desired.clone()],
        }
        .vm_documents()?
        .into_values()
        .next()
        .ok_or("missing output document")?,
        publication_required: true,
        initial: approved.observed.clone(),
        expected_power: Some(approved.preview.restore_power.clone()),
        graphics_required: Some(
            approved
                .preview
                .actions
                .contains(&workflow::PreviewAction::Verify),
        ),
        initial_journal: approved.managed.clone(),
        expected_settings: approved
            .preview
            .settings
            .as_ref()
            .map_or_else(|| approved.observed.settings.clone(), |s| s.after.clone()),
        phase: SavePhase::Admitted,
    };
    runner::write_recovery_at(
        recovery_path
            .as_deref()
            .ok_or("missing bound recovery path")?,
        &record,
    )?;
    let mut mark_effects = || {
        if matches!(record.phase, SavePhase::Admitted) {
            record.phase = SavePhase::EffectsStarted;
            runner::write_recovery_at(
                recovery_path
                    .as_deref()
                    .ok_or("missing bound recovery path")?,
                &record,
            )?;
        }
        Ok(())
    };
    let result = workflow::apply_admitted(
        &mut ObservedBackend {
            backend: &mut backend,
            notify: &mut notify,
            before_effect: Some(&mut mark_effects),
        },
        admitted,
    )?;
    ObservedBackend {
        backend: &mut backend,
        notify: &mut notify,
        before_effect: None,
    }
    .stage_observed(
        "Confirm final VM state and signed driver files",
        |backend| {
            confirm(
                backend,
                &record.target,
                &result.effective,
                &approved.gpu,
                record.payload_digest.as_deref(),
            )
        },
    )?;
    record.phase = SavePhase::Verified(result.effective.clone());
    runner::write_recovery_at(
        recovery_path
            .as_deref()
            .ok_or("missing bound recovery path")?,
        &record,
    )?;
    publish(context, record, result, notify)
}

pub(super) fn verify(
    context: &Context<'_>,
    approved: workflow::VerificationPlan,
    shutdown_approved: bool,
    (mut backend, admitted): (NativeBackend, workflow::AdmittedVerification),
    mut notify: impl FnMut(Progress) -> Result<(), String>,
) -> Result<Outcome, String> {
    let enrollment = context.enrollment;
    let session = context.session;
    let recovery_path = Some(context.recovery_path.to_path_buf());
    runner::require_no_vm_recovery(&approved.target.vm_id)?;
    crate::windows_wmi::require_no_pending_operation_for(&approved.target.vm_id)?;
    approved.target.validate()?;
    if approved.observed.power == Power::Off && !shutdown_approved {
        return Err("temporary guest start and graceful shutdown were not approved".into());
    }
    if !enrollment.targets.iter().any(|target| {
        target.vm_id == approved.target.vm_id
            && target.gpu_interface == approved.target.gpu_interface
    }) {
        return Err("verification target is outside enrollment".into());
    }
    let mut record = SaveRecord {
        schema: 1,
        retirement: None,
        reset: None,
        operation_id: session.into(),
        target: approved.target.clone(),
        expected: configuration_store::protected::revision(&approved.target.vm_id)?,
        driver_version: approved.gpu.driver_version.clone(),
        payload_digest: Some(approved.payload_digest.clone()),
        plan_revision: Revision::of(&serde_json::to_vec(&approved).map_err(|e| e.to_string())?),
        intended_source: Configuration {
            schema: 3,
            targets: vec![approved.target.clone()],
        }
        .vm_documents()?
        .into_values()
        .next()
        .ok_or("missing receipt intent")?,
        publication_required: false,
        initial: approved.observed.clone(),
        expected_power: Some(workflow::verification_power(
            &approved.target,
            &approved.observed,
            approved.managed.as_ref(),
        )?),
        graphics_required: Some(true),
        initial_journal: approved.managed.clone(),
        expected_settings: approved.observed.settings.clone(),
        phase: SavePhase::Admitted,
    };
    runner::write_recovery_at(
        recovery_path
            .as_deref()
            .ok_or("missing bound recovery path")?,
        &record,
    )?;
    let mut mark_effects = || {
        if matches!(record.phase, SavePhase::Admitted) {
            record.phase = SavePhase::EffectsStarted;
            runner::write_recovery_at(
                recovery_path
                    .as_deref()
                    .ok_or("missing bound recovery path")?,
                &record,
            )?;
        }
        Ok(())
    };
    let result = workflow::verify_admitted(
        &mut ObservedBackend {
            backend: &mut backend,
            notify: &mut notify,
            before_effect: Some(&mut mark_effects),
        },
        admitted,
    )?;
    confirm(
        &mut backend,
        &record.target,
        &result.effective,
        &approved.gpu,
        Some(&approved.payload_digest),
    )?;
    record.phase = SavePhase::Verified(result.effective.clone());
    runner::write_recovery_at(
        recovery_path
            .as_deref()
            .ok_or("missing bound recovery path")?,
        &record,
    )?;
    Ok(Outcome {
        operation: Some(result),
        saved: false,
        finished: true,
        operation_id: record.operation_id,
        save_error: None,
    })
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
    let recovery_path = Some(context.recovery_path.to_path_buf());
    notify(Progress {
        stage: "Publish checked VM configuration".into(),
        status: StageStatus::Running,
    })?;
    let saved = configuration_store::protected::publish(&record, &enrollment.client_sid);
    if saved.is_ok() {
        record.phase = SavePhase::Published(result.effective.clone());
        runner::write_recovery_at(
            recovery_path
                .as_deref()
                .ok_or("missing bound recovery path")?,
            &record,
        )?;
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
