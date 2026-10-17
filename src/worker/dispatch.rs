//! Common locked admission, audit binding, command routing and terminal receipt retirement.
use super::{
    configuration_import, operations, pairing,
    preparation::{self, Authorization, PreparedReview},
    progress::preparation_stage,
    protocol::{Command, Outcome, Progress, Request},
    recovery, runner,
};
use crate::{configuration_store::Revision, windows_pipe};
use std::fs;

pub(super) fn execute(
    enrollment: &runner::Enrollment,
    session: &str,
    request: Request,
    notify: impl FnMut(Progress) -> Result<(), String>,
    exchange: impl FnMut(&PreparedReview) -> Result<Authorization, String>,
) -> Result<Outcome, String> {
    crate::diagnostics::run("worker-command", session, || {
        execute_inner(enrollment, session, request, notify, exchange)
    })
}

fn execute_inner(
    enrollment: &runner::Enrollment,
    session: &str,
    request: Request,
    mut notify: impl FnMut(Progress) -> Result<(), String>,
    mut exchange: impl FnMut(&PreparedReview) -> Result<Authorization, String>,
) -> Result<Outcome, String> {
    let (data, _lock) = preparation_stage("Check protected worker admission", &mut notify, || {
        let data = runner::data_directory()?;
        crate::security::verify(&data)?;
        let _lock = runner::operation_lock()?;
        let current = runner::enrollment()?;
        if serde_json::to_value(enrollment).map_err(|e| e.to_string())?
            != serde_json::to_value(&current).map_err(|e| e.to_string())?
        {
            return Err("enrollment changed before admission; Refresh".into());
        }
        runner::verify_artifacts(&current)?;
        if windows_pipe::current_user_sid_string().map_err(|e| e.to_string())? != current.client_sid
        {
            return Err("operator enrollment changed before admission".into());
        }
        Ok((data, _lock))
    })?;
    let audit_directory = data.join("audit");
    crate::security::verify(&audit_directory)?;
    let audit = audit_directory.join(format!("worker-{session}.json"));
    if fs::symlink_metadata(&audit).is_ok() {
        return Err("replayed worker session".into());
    }
    let operation_name = match &request.command {
        Command::Prepare { intent } => intent.operation_name(),
        Command::ResetPartial { .. } => "ResetPartial",
        Command::ForgetPair { .. } => "ForgetPair",
        Command::EnrollPair { .. } => "EnrollPair",
        Command::ReconcilePair { .. } => "ReconcilePair",
        Command::Apply { .. } => "ApplyApproved",
        Command::Verify { .. } => "VerifyApproved",
        Command::Reconcile { .. } => "ReconcileObserved",
        Command::SaveOnly { .. } => "SaveOnly",
        Command::Import { .. } => "ImportConfiguration",
        Command::ImportResume { .. } => "ReconcileImport",
    };
    let mut binding = match &request.command {
        Command::Prepare { intent } => serde_json::to_value(intent).map_err(|e| e.to_string())?,
        Command::ResetPartial {
            operation_id,
            record_revision,
            approved,
            shutdown_approved,
        } => {
            serde_json::json!({"original_operation_id":operation_id,"record_revision":record_revision,"review":approved,"shutdown_approved":shutdown_approved})
        }
        Command::ForgetPair { approved } => {
            serde_json::to_value(approved).map_err(|e| e.to_string())?
        }
        Command::EnrollPair { approved } => {
            serde_json::to_value(approved).map_err(|e| e.to_string())?
        }
        Command::ReconcilePair {
            operation_id,
            expected,
        } => serde_json::json!({"original_operation_id":operation_id,"expected":expected}),
        Command::Apply {
            approved,
            expected,
            shutdown_approved,
        } => {
            serde_json::json!({"vm_id":approved.desired.vm_id,"gpu_interface":approved.desired.gpu_interface,"plan_revision":Revision::of(&serde_json::to_vec(approved).map_err(|e| e.to_string())?),"expected":expected,"shutdown_approved":shutdown_approved})
        }
        Command::SaveOnly { operation_id } => {
            serde_json::json!({"original_operation_id":operation_id})
        }
        Command::Reconcile {
            operation_id,
            shutdown_approved,
        } => {
            serde_json::json!({"original_operation_id":operation_id,"shutdown_approved":shutdown_approved})
        }
        Command::Verify {
            approved,
            shutdown_approved,
        } => {
            serde_json::json!({"vm_id":approved.target.vm_id,"gpu_interface":approved.target.gpu_interface,"plan_revision":Revision::of(&serde_json::to_vec(approved).map_err(|e| e.to_string())?),"shutdown_approved":shutdown_approved})
        }
        Command::Import { source, expected } => {
            serde_json::json!({"source_revision":Revision::of(source.as_bytes()),"expected_destinations":expected})
        }
        Command::ImportResume { operation_id } => {
            serde_json::json!({"original_operation_id":operation_id,"receipt":runner::import_record()?})
        }
    };
    runner::atomic_json(
        &audit,
        &serde_json::json!({"schema":1,"session":session,"operation":operation_name,"binding":binding,"outcome":"Started"}),
    )?;
    let mut recovery_path = None;
    let result = (|| {
        if matches!(
            &request.command,
            Command::Prepare { .. }
                | Command::Apply { .. }
                | Command::Verify { .. }
                | Command::Reconcile { .. }
                | Command::SaveOnly { .. }
                | Command::ResetPartial { .. }
        ) {
            runner::prepare_vm_recovery()?;
        }
        // Bind every write and final retirement to this command's own receipt.
        recovery_path = match &request.command {
            Command::Prepare { intent } => Some(runner::new_recovery_path(&intent.target().vm_id)?),
            Command::Apply { approved, .. } => {
                Some(runner::new_recovery_path(&approved.desired.vm_id)?)
            }
            Command::Verify { approved, .. } => {
                Some(runner::new_recovery_path(&approved.target.vm_id)?)
            }
            Command::ResetPartial { operation_id, .. }
            | Command::Reconcile { operation_id, .. }
            | Command::SaveOnly { operation_id } => Some(runner::recovery_record_path(
                &runner::recovery_for_operation(operation_id)?
                    .ok_or("recorded recovery operation is unavailable")?,
            )?),
            _ => None,
        };
        match request.command {
            Command::Prepare { intent } => {
                if request.credential.is_some() {
                    return Err("Preparation does not accept credentials".into());
                }
                preparation::execute(
                    &operations::Context {
                        enrollment,
                        session,
                        data: &data,
                        recovery_path: recovery_path
                            .as_deref()
                            .ok_or("missing bound recovery path")?,
                    },
                    intent,
                    &mut notify,
                    |review| {
                        binding["prepared_revision"] =
                            serde_json::to_value(review.revision()?).map_err(|e| e.to_string())?;
                        runner::atomic_json(
                            &audit,
                            &serde_json::json!({"schema":1,"session":session,"operation":operation_name,"binding":binding,"outcome":"Started"}),
                        )?;
                        exchange(review)
                    },
                )
            }
            Command::ResetPartial {
                operation_id,
                record_revision,
                approved,
                shutdown_approved,
            } => {
                if request.credential.is_some() {
                    return Err("Partial GPU removal does not accept credentials".into());
                }
                recovery::reset_partial(
                    &operations::Context {
                        enrollment,
                        session,
                        data: &data,
                        recovery_path: recovery_path
                            .as_deref()
                            .ok_or("missing bound recovery path")?,
                    },
                    operation_id,
                    record_revision,
                    approved,
                    shutdown_approved,
                    &mut notify,
                )
            }
            Command::ForgetPair { approved } => {
                pairing::forget(session, approved, request.credential, &mut notify)
            }
            Command::EnrollPair { approved } => {
                pairing::enroll(session, approved, request.credential, &mut notify)
            }
            Command::ReconcilePair {
                operation_id,
                expected,
            } => pairing::reconcile(session, operation_id, expected, request.credential),
            Command::Import { source, expected } => {
                configuration_import::import(enrollment, session, source, expected, &mut notify)
            }
            Command::ImportResume { operation_id } => {
                configuration_import::resume(enrollment, operation_id, &mut notify)
            }
            Command::Apply {
                approved,
                expected,
                shutdown_approved,
            } => operations::apply(
                &operations::Context {
                    enrollment,
                    session,
                    data: &data,
                    recovery_path: recovery_path
                        .as_deref()
                        .ok_or("missing bound recovery path")?,
                },
                approved,
                expected,
                shutdown_approved,
                request.credential,
                None,
                &mut notify,
            ),
            Command::Verify {
                approved,
                shutdown_approved,
            } => operations::verify(
                &operations::Context {
                    enrollment,
                    session,
                    data: &data,
                    recovery_path: recovery_path
                        .as_deref()
                        .ok_or("missing bound recovery path")?,
                },
                approved,
                shutdown_approved,
                request.credential,
                None,
                &mut notify,
            ),
            Command::Reconcile {
                operation_id,
                shutdown_approved,
            } => recovery::reconcile(
                &operations::Context {
                    enrollment,
                    session,
                    data: &data,
                    recovery_path: recovery_path
                        .as_deref()
                        .ok_or("missing bound recovery path")?,
                },
                operation_id,
                shutdown_approved,
                request.credential,
                &mut notify,
            ),
            Command::SaveOnly { operation_id } => recovery::save_only(
                &operations::Context {
                    enrollment,
                    session,
                    data: &data,
                    recovery_path: recovery_path
                        .as_deref()
                        .ok_or("missing bound recovery path")?,
                },
                operation_id,
                &mut notify,
            ),
        }
    })();
    runner::atomic_json(
        &audit,
        &serde_json::json!({"schema":1,"session":session,"operation":operation_name,"binding":binding,"outcome":if result.as_ref().is_ok_and(|o| o.finished) { "Succeeded" } else { "Failed" }}),
    )?;
    // Release only this command's hold, after publication and terminal audit.
    if let Ok(outcome) = &result
        && outcome.finished
    {
        if let Some(path) = recovery_path {
            runner::retire_recovery_at(&path, &outcome.operation_id)?;
        } else {
            let path = if matches!(
                operation_name,
                "EnrollPair" | "ReconcilePair" | "ForgetPair"
            ) {
                runner::pair_pending_path()?
            } else {
                runner::import_path()?
            };
            crate::security::verify(&path)?;
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }
    result
}
