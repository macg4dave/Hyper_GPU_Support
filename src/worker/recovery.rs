//! Explicit recovery, partial reset and save-only publication without replaying Apply.
use super::{
    operations,
    progress::{ObservedBackend, finish_progress},
    protocol::{Outcome, Progress, StageStatus},
    runner,
};
use crate::{
    configuration_store::{self, Revision, SavePhase},
    credentials::Credential,
    model::Power,
    runner::NativeBackend,
    workflow::{self, Backend},
};
use std::fs;

pub(super) fn reset_partial(
    context: &operations::Context<'_>,
    operation_id: String,
    record_revision: Revision,
    approved: workflow::ResetReview,
    shutdown_approved: bool,
    mut notify: impl FnMut(Progress) -> Result<(), String>,
) -> Result<Outcome, String> {
    let enrollment = context.enrollment;
    let data = context.data.to_path_buf();
    let recovery_path = Some(context.recovery_path.to_path_buf());
    let audit_directory = data.join("audit");
    let mut record = runner::read_recovery_at(
        recovery_path
            .as_deref()
            .ok_or("missing bound recovery path")?,
    )?
    .ok_or("No interrupted operation to remove")?;
    if record.operation_id != operation_id
        || Revision::of(&serde_json::to_vec(&record).map_err(|e| e.to_string())?) != record_revision
        || !enrollment.targets.iter().any(|t| {
            t.vm_id == record.target.vm_id && t.gpu_interface == record.target.gpu_interface
        })
        || runner::pending_pair()?.is_some()
        || runner::import_record()?.is_some()
    {
        return Err("Partial GPU removal review changed; Refresh and review again".into());
    }
    crate::windows_wmi::reconcile_pending_operation_for(&record.target.vm_id)?;
    let revision = configuration_store::protected::revision(&record.target.vm_id)?;
    approved.validate(&record, &revision)?;
    if approved.observed.power == Power::Running && !shutdown_approved {
        return Err("Separately approve graceful shutdown for this recovery disposition".into());
    }
    if record.reset.is_none() {
        let mut fresh = NativeBackend {
            reads: None,
            credential: None,
            manifest: None,
            payload_hold: Vec::new(),
            data: data.clone(),
        };
        if fresh.inspect(&record.target)?.operational() != approved.observed.operational()
            || fresh.gpu(&record.target)?.operational() != approved.gpu.operational()
            || serde_json::to_value(fresh.journal(&record.target)?).map_err(|e| e.to_string())?
                != serde_json::to_value(Some(&approved.journal)).map_err(|e| e.to_string())?
        {
            return Err(
                "Partial GPU removal facts changed before admission; Refresh and review again"
                    .into(),
            );
        }
    }
    if let Some(existing) = &record.reset
        && serde_json::to_value(existing).map_err(|e| e.to_string())?
            != serde_json::to_value(&approved).map_err(|e| e.to_string())?
    {
        return Err("Partial GPU removal is bound to its original review".into());
    }
    record.reset = Some(approved);
    runner::write_recovery_at(
        recovery_path
            .as_deref()
            .ok_or("missing bound recovery path")?,
        &record,
    )?;
    runner::atomic_json(
        &audit_directory.join(format!("partial-removal-{}.json", operation_id)),
        &record,
    )?;
    let bound = serde_json::to_value(&record).map_err(|e| e.to_string())?;
    let mut check_binding = || {
        if serde_json::to_value(runner::read_recovery_at(
            recovery_path
                .as_deref()
                .ok_or("missing bound recovery path")?,
        )?)
        .map_err(|e| e.to_string())?
            != serde_json::json!(bound)
            || configuration_store::protected::revision(&record.target.vm_id)? != revision
        {
            return Err("Partial GPU removal receipt changed; retain recovery".into());
        }
        crate::windows_wmi::require_no_pending_operation_for(&record.target.vm_id)
    };
    check_binding()?;
    let mut backend = NativeBackend {
        reads: None,
        credential: None,
        manifest: None,
        payload_hold: Vec::new(),
        data: data.clone(),
    };
    workflow::reset_partial(
        &mut ObservedBackend {
            backend: &mut backend,
            notify: &mut notify,
            before_effect: Some(&mut check_binding),
        },
        &record,
        &revision,
        shutdown_approved,
    )?;
    if configuration_store::protected::revision(&record.target.vm_id)? != revision
        || serde_json::to_value(runner::read_recovery_at(
            recovery_path
                .as_deref()
                .ok_or("missing bound recovery path")?,
        )?)
        .map_err(|e| e.to_string())?
            != serde_json::json!(bound)
    {
        return Err("Partial GPU removal final receipt changed; retain recovery".into());
    }
    crate::windows_wmi::require_no_pending_operation_for(&record.target.vm_id)?;
    Ok(Outcome {
        operation: None,
        saved: false,
        finished: true,
        operation_id,
        save_error: None,
    })
}

pub(super) fn reconcile(
    context: &operations::Context<'_>,
    operation_id: String,
    shutdown_approved: bool,
    credential: Option<Credential>,
    mut notify: impl FnMut(Progress) -> Result<(), String>,
) -> Result<Outcome, String> {
    let enrollment = context.enrollment;
    let session = context.session;
    let data = context.data.to_path_buf();
    let recovery_path = Some(context.recovery_path.to_path_buf());
    let audit_directory = data.join("audit");
    let mut record = runner::read_recovery_at(
        recovery_path
            .as_deref()
            .ok_or("missing bound recovery path")?,
    )?
    .ok_or("no durable operation requires reconciliation")?;
    if record.operation_id != operation_id {
        return Err("reconciliation operation identity mismatch".into());
    }
    if !enrollment
        .targets
        .iter()
        .any(|t| t.vm_id == record.target.vm_id && t.gpu_interface == record.target.gpu_interface)
    {
        return Err("recorded pair is no longer enrolled".into());
    }
    if record.publication_required
        && matches!(
            record.phase,
            SavePhase::Verified(_) | SavePhase::Published(_)
        )
    {
        return Err(
            "already verified: use save-only without repeating graphics or guest power".into(),
        );
    }
    let mut backend = NativeBackend {
        reads: None,
        credential,
        manifest: None,
        payload_hold: Vec::new(),
        data: data.clone(),
    };
    let mut backend = ObservedBackend {
        backend: &mut backend,
        notify: &mut notify,
        before_effect: None,
    };
    backend.stage_observed("Check recorded recovery outcome", |backend| {
    let before = backend.inspect(&record.target)?;
    let negative = record.negative_completion(
        &before,
        &backend.gpu(&record.target)?.driver_version,
        &configuration_store::protected::revision(&record.target.vm_id)?,
        backend.journal(&record.target)?.as_ref(),
    );
    let retirement = if record.retirement.is_some() {
        negative?
    } else {
        // A noncandidate follows ordinary reconciliation, including WMI recovery.
        negative.ok().flatten()
    };
    if let Some(retirement) = retirement {
        crate::windows_wmi::require_no_pending_operation_for(&record.target.vm_id)?;
        // Persist the exact transition before its journal write. A lost reply or
        // interruption can resume only the bound before/after states.
        record.retirement = Some(retirement.clone());
        runner::write_recovery_at(recovery_path.as_deref().ok_or("missing bound recovery path")?, &record)?;
        runner::atomic_json(
            &audit_directory.join(format!("ended-{}.json", record.operation_id)),
            &record,
        )?;
        record
            .negative_completion(
                &backend.inspect(&record.target)?,
                &backend.gpu(&record.target)?.driver_version,
                &configuration_store::protected::revision(&record.target.vm_id)?,
                backend.journal(&record.target)?.as_ref(),
            )?
            .ok_or("negative completion proof unavailable; retain recovery")?;
        backend.save(&record.target, &retirement.after)?;
        record
            .negative_completion(
                &backend.inspect(&record.target)?,
                &backend.gpu(&record.target)?.driver_version,
                &configuration_store::protected::revision(&record.target.vm_id)?,
                backend.journal(&record.target)?.as_ref(),
            )?
            .ok_or("negative completion readback unavailable; retain recovery")?;
        if serde_json::to_value(backend.journal(&record.target)?)
            .map_err(|e| e.to_string())?
            != serde_json::to_value(Some(&retirement.after)).map_err(|e| e.to_string())?
        {
            return Err(
                "negative completion journal publication not established; retain recovery"
                    .into(),
            );
        }
        crate::windows_wmi::require_no_pending_operation_for(&record.target.vm_id)?;
        return Ok(Outcome {
            operation: None,
            saved: false,
            finished: true,
            operation_id,
            save_error: None,
        });
    }
    if matches!(record.phase, SavePhase::EffectsStarted) {
        crate::windows_wmi::require_no_pending_operation_for(&record.target.vm_id)?;
        let journal = backend.journal(&record.target)?;
        if record
            .validate_restored_baseline(
                &before,
                &backend.gpu(&record.target)?.driver_version,
                &configuration_store::protected::revision(&record.target.vm_id)?,
                journal.as_ref(),
            )
            .is_ok()
        {
            // Archive the original uncertain attempt before retiring its journal.
            // Partial guest files remain untrusted: there is no preparation receipt.
            let archive =
                audit_directory.join(format!("ended-{}.json", record.operation_id));
            runner::atomic_json(
                &archive,
                &serde_json::json!({"schema":1,"disposition":"OriginalHostStateEstablished","record":record,"guest_files":"Unknown; retained without a preparation receipt"}),
            )?;
            record.validate_restored_baseline(
                &backend.inspect(&record.target)?,
                &backend.gpu(&record.target)?.driver_version,
                &configuration_store::protected::revision(&record.target.vm_id)?,
                backend.journal(&record.target)?.as_ref(),
            )?;
            if journal.is_some() {
                let path = data.join(format!("{}.json", record.target.vm_id));
                crate::security::verify(&path)?;
                fs::remove_file(path).map_err(|e| e.to_string())?;
            }
            record.validate_restored_baseline(
                &backend.inspect(&record.target)?,
                &backend.gpu(&record.target)?.driver_version,
                &configuration_store::protected::revision(&record.target.vm_id)?,
                backend.journal(&record.target)?.as_ref(),
            )?;
            return Ok(Outcome {
                operation: None,
                saved: false,
                finished: true,
                operation_id,
                save_error: None,
            });
        }
    }
    if matches!(record.phase, SavePhase::Admitted) {
        // The wrapper marks EffectsStarted before every guest/Hyper-V
        // adapter, so this phase proves no such effect was dispatched.
        crate::windows_wmi::require_no_pending_operation_for(&record.target.vm_id)?;
        if before.operational() != record.initial.operational()
            || configuration_store::protected::revision(&record.target.vm_id)?
                != record.expected
        {
            return Err("pre-effect rejection facts changed; retain recovery".into());
        }
        let current_journal = backend.journal(&record.target)?;
        if current_journal.as_ref().is_some_and(|j| {
            j.schema != 1
                || j.vm_id != record.target.vm_id
                || j.gpu_interface != record.target.gpu_interface
        }) {
            return Err("journal identity changed during pre-effect rejection".into());
        }
        if let Some(original) = &record.initial_journal {
            backend.save(&record.target, original)?;
        } else if current_journal.is_some() {
            let path = data.join(format!("{}.json", record.target.vm_id));
            crate::security::verify(&path)?;
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
        return Ok(Outcome {
            operation: None,
            saved: false,
            finished: true,
            operation_id,
            save_error: None,
        });
    }
    crate::windows_wmi::reconcile_pending_operation_for(&record.target.vm_id)?;
    if !record.publication_required
        && let SavePhase::Verified(verified) | SavePhase::Published(verified) =
            &record.phase
    {
        if before.operational() != verified.operational()
            || backend.gpu(&record.target)?.driver_version != record.driver_version
            || backend.journal(&record.target)?.is_some_and(|j| j.pending)
        {
            return Err("durably verified graphics receipt changed; retain recovery".into());
        }
        return Ok(Outcome {
            operation: None,
            saved: false,
            finished: true,
            operation_id,
            save_error: None,
        });
    }
    record
        .validate_reconciliation(&before, &backend.gpu(&record.target)?.driver_version)?;
    let final_power = record.final_power()?;
    if let Some(digest) = &record.payload_digest
        && backend.payload(&record.target)? != *digest
    {
        return Err("current payload differs from the recorded operation".into());
    }
    let check_graphics = record.requires_graphics();
    // A reviewed record-only completion supplies no credentials. If its
    // restoration proof drifted, refuse a fallback guest check before
    // starting the VM; return to a fresh recovery review instead.
    if check_graphics && backend.backend.credential.is_none() {
        return Err("Guest verification now requires credentials and a fresh recovery review; no guest lifecycle work was started.".into());
    }
    if check_graphics
        && (before.power == Power::Off || final_power == Power::Off)
        && !shutdown_approved
    {
        return Err("explicit graceful guest shutdown/restoration approval is required for reconciliation".into());
    }
    let mut checked_target = record.target.clone();
    if !record.publication_required {
        checked_target.enabled = true;
        checked_target.clear_allocations();
    }
    let result = workflow::reconcile_with_graphics(
        backend,
        &checked_target,
        &before,
        final_power,
        record.payload_digest.as_deref(),
        check_graphics,
    )?;
    record.validate_reconciliation(
        &backend.inspect(&record.target)?,
        &backend.gpu(&record.target)?.driver_version,
    )?;
    record.phase = SavePhase::Verified(result.effective.clone());
    runner::write_recovery_at(recovery_path.as_deref().ok_or("missing bound recovery path")?, &record)?;
    if !record.publication_required {
        return Ok(Outcome {
            operation: Some(result),
            saved: false,
            finished: true,
            operation_id,
            save_error: None,
        });
    }
    record.validate_save_only(
        &result.effective,
        &backend.gpu(&record.target)?.driver_version,
        &configuration_store::protected::revision(&record.target.vm_id)?,
    )?;
    operations::publish(&operations::Context { enrollment, session, data: &data, recovery_path: recovery_path.as_deref().ok_or("missing bound recovery path")? }, record, result, &mut *backend.notify)
    })
}

pub(super) fn save_only(
    context: &operations::Context<'_>,
    operation_id: String,
    mut notify: impl FnMut(Progress) -> Result<(), String>,
) -> Result<Outcome, String> {
    let enrollment = context.enrollment;
    let data = context.data.to_path_buf();
    let recovery_path = Some(context.recovery_path.to_path_buf());
    let mut record = runner::read_recovery_at(
        recovery_path
            .as_deref()
            .ok_or("missing bound recovery path")?,
    )?
    .ok_or("no verified-unsaved operation exists")?;
    crate::windows_wmi::require_no_pending_operation_for(&record.target.vm_id)?;
    if record.operation_id != operation_id {
        return Err("save-only operation identity mismatch".into());
    }
    if !enrollment.targets.iter().any(|target| {
        target.vm_id == record.target.vm_id && target.gpu_interface == record.target.gpu_interface
    }) {
        return Err("save-only pair no longer enrolled".into());
    }
    let mut backend = NativeBackend {
        reads: None,
        credential: None,
        manifest: None,
        payload_hold: Vec::new(),
        data: data.clone(),
    };
    if backend
        .journal(&record.target)?
        .is_some_and(|journal| journal.pending)
    {
        return Err(
            "unfinished managed operation requires manual reconciliation before saving".into(),
        );
    }
    if let Some(digest) = &record.payload_digest
        && backend.payload(&record.target)? != *digest
    {
        return Err("signed payload changed since verification; reconcile before saving".into());
    }
    crate::work_counts::add(crate::work_counts::Counter::CompletionReads, 1);
    record.validate_save_only(
        &backend.inspect(&record.target)?,
        &backend.gpu(&record.target)?.driver_version,
        &configuration_store::protected::revision(&record.target.vm_id)?,
    )?;
    notify(Progress {
        stage: "Save verified configuration only".into(),
        status: StageStatus::Running,
    })?;
    let publication = configuration_store::protected::publish(&record, &enrollment.client_sid);
    if publication.is_err() {
        let delivery = notify(Progress {
            stage: "Save verified configuration only".into(),
            status: StageStatus::Failed,
        });
        finish_progress(publication, delivery)?;
    }
    let verified = match &record.phase {
        SavePhase::Verified(state) | SavePhase::Published(state) => state.clone(),
        SavePhase::Admitted | SavePhase::EffectsStarted => {
            return Err("manual reconciliation required".into());
        }
    };
    record.phase = SavePhase::Published(verified);
    runner::write_recovery_at(
        recovery_path
            .as_deref()
            .ok_or("missing bound recovery path")?,
        &record,
    )?;
    notify(Progress {
        stage: "Save verified configuration only".into(),
        status: StageStatus::Done,
    })?;
    Ok(Outcome {
        operation: None,
        saved: true,
        finished: true,
        operation_id,
        save_error: None,
    })
}
