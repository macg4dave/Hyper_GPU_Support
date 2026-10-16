//! Recovery-record inspection and explicit reconciliation of recorded outcomes.
use super::SharedState;
#[cfg(windows)]
use super::{
    background::background_operation, credentials::credential_request_cancelled,
    presentation::present, progress::progress_to_ui, state_access::with_completion_state,
};
#[cfg(windows)]
use crate::gui::operation::{Kind, Operation};
use crate::gui::{AppWindow, dialog, live_state::State, set_details, wording};
#[cfg(windows)]
use slint::ComponentHandle;
use std::sync::{Arc, atomic::AtomicBool};

pub(super) fn review_records(
    ui: &AppWindow,
    state: &mut State,
    recovery: bool,
) -> Result<(), String> {
    #[cfg(windows)]
    if recovery && let Some(record) = &state.enrollment_recovery {
        let revision = hyper_gpu_support::runner::pair_revision()?;
        let known = revision == record.resulting || revision == record.review.expected;
        if !known {
            return Err("Authority does not match either recorded revision. Recovery remains held for administrator inspection.".into());
        }
        let outcome = if record.removal.is_some() {
            hyper_gpu_support::ui_text::operations::enrollment::recovery::PARTIAL_DELETION
        } else if revision == record.resulting {
            hyper_gpu_support::ui_text::operations::enrollment::recovery::POLICY_PUBLISHED
        } else {
            hyper_gpu_support::ui_text::operations::enrollment::recovery::POLICY_ORIGINAL
        };
        let body = format!(
            "Operation: {}\nVM ID: {}\nGPU: {}\n\n{outcome}",
            record.operation_id, record.review.vm_id, record.review.gpu_interface
        );
        let name = state.view.vm(&record.review.vm_id).map_or(
            hyper_gpu_support::ui_text::operations::enrollment::recovery::NAME_UNAVAILABLE,
            |vm| vm.name.as_str(),
        );
        let summary = format!(
            "Virtual machine: {name}\n\n{outcome}\n\nReviewing this record makes no changes. Confirmation is required for the recovery action shown."
        );
        state.enrollment_recovery_revision = Some(revision);
        ui.set_needs_shutdown(false);
        dialog(
            ui,
            "enrollment-recovery",
            if record.removal.is_some() {
                hyper_gpu_support::ui_text::operations::profile_deletion::recovery::TITLE
            } else {
                hyper_gpu_support::ui_text::operations::enrollment::recovery::TITLE
            },
            &summary,
        );
        set_details(ui, &body);
        return Ok(());
    }
    if recovery && let Some(record) = &state.import_recovery {
        dialog(
            ui,
            "info",
            hyper_gpu_support::ui_text::operations::configuration_import::recovery::TITLE,
            &hyper_gpu_support::ui_text::operations::configuration_import::recovery::body(
                &(record.operation_id),
                record.documents.len(),
                record
                    .documents
                    .values()
                    .filter(|item| item.published)
                    .count(),
            )
            .into_string(),
        );
        return Ok(());
    }
    if recovery && let Some(record) = &state.recovery {
        ui.set_needs_shutdown(record.requires_graphics() && !state.recovery_without_guest(record)?);
        ui.set_power_consent(wording::recovery_consent(&record.final_power()?).into());
        match record.phase {
            hyper_gpu_support::configuration_store::SavePhase::Verified(_)
            | hyper_gpu_support::configuration_store::SavePhase::Published(_) => {
                if record.publication_required {
                    dialog(
                        ui,
                        "save",
                        hyper_gpu_support::ui_text::operations::save_only::review::TITLE,
                        hyper_gpu_support::ui_text::operations::save_only::review::BODY,
                    );
                    set_details(
                        ui,
                        &format!(
                            "Operation: {}\nVM ID: {}\nGPU: {}\nReviewed configuration revision: {:?}\nDriver: {}\nExpected final power: {:?}",
                            record.operation_id,
                            record.target.vm_id,
                            record.target.gpu_interface,
                            record.expected,
                            record.driver_version,
                            record.final_power()?
                        ),
                    );
                } else {
                    dialog(
                        ui,
                        "recovery",
                        hyper_gpu_support::ui_text::operations::verify::recovery::TITLE,
                        hyper_gpu_support::ui_text::operations::verify::recovery::BODY,
                    );
                    ui.set_needs_shutdown(false);
                }
            }
            hyper_gpu_support::configuration_store::SavePhase::Admitted
            | hyper_gpu_support::configuration_store::SavePhase::EffectsStarted => {
                let name = state
                    .view
                    .vm(&record.target.vm_id)
                    .map_or(record.initial.name.as_str(), |vm| vm.name.as_str());
                if matches!(
                    record.phase,
                    hyper_gpu_support::configuration_store::SavePhase::Admitted
                ) {
                    dialog(
                        ui,
                        "recovery",
                        hyper_gpu_support::ui_text::operations::recovery::review::TITLE,
                        hyper_gpu_support::ui_text::operations::recovery::admitted::BODY,
                    );
                    ui.set_recovery_heading(
                        hyper_gpu_support::ui_text::operations::action::review_prompt(name)
                            .into_string()
                            .into(),
                    );
                } else if !record.publication_required {
                    let restored = state.recovery_without_guest(record)?;
                    let body = wording::verification_recovery(record, name, restored);
                    dialog(
                        ui,
                        "recovery",
                        hyper_gpu_support::ui_text::operations::verify::recovery::TITLE,
                        &body,
                    );
                    ui.set_needs_shutdown(!restored);
                } else if !record.requires_graphics() {
                    dialog(
                        ui,
                        "recovery",
                        hyper_gpu_support::ui_text::operations::apply::recovery::TITLE,
                        &hyper_gpu_support::ui_text::operations::apply::recovery::body(name)
                            .into_string(),
                    );
                } else {
                    dialog(
                        ui,
                        "recovery",
                        hyper_gpu_support::ui_text::operations::recovery::review::TITLE,
                        hyper_gpu_support::ui_text::operations::recovery::review::EXPLANATION,
                    );
                    ui.set_recovery_heading(
                        hyper_gpu_support::ui_text::operations::action::review_prompt(name)
                            .into_string()
                            .into(),
                    );
                }
                set_details(
                    ui,
                    &format!(
                        "Operation: {}\nVM ID: {}\nGPU: {}\nOriginal guest power: {:?}\nReviewed final guest power: {:?}",
                        record.operation_id,
                        record.target.vm_id,
                        record.target.gpu_interface,
                        record.initial.power,
                        record.final_power()?
                    ),
                );
                ui.set_recovery_vm_id(record.target.vm_id.clone().into());
            }
        }
        ui.set_recovery_vm_id(record.target.vm_id.clone().into());
        return Ok(());
    }
    state
        .view
        .inventory
        .as_ref()
        .ok_or("Refresh inventory first.")?;
    let body = crate::gui::diagnostics::selected_vm_details(state);
    let provenance = hyper_gpu_support::ui_text::operations::inventory::PROVENANCE;
    dialog(
        ui,
        "info",
        hyper_gpu_support::ui_text::operations::inventory::DETAILS_TITLE,
        &format!(
            "{provenance}\n\nTechnical Details contains the host observation, recorded history and authorization from that inventory. Missing or old records do not establish current guest health. Opening these details performs no new checks or changes."
        ),
    );
    set_details(ui, &body);
    Ok(())
}

#[cfg(windows)]
pub(super) fn reconcile_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let record = state
        .lock()
        .map_err(|_| "Presentation state unavailable")?
        .recovery
        .clone()
        .ok_or("No reviewed operation recovery is loaded")?;
    let progress = progress_to_ui(ui.as_weak());
    let finished_state = state.clone();
    let target = record.target.clone();
    let publication = record.publication_required;
    let recovery_vm_id = target.vm_id.clone();
    let (vm_name, restored_candidate) = {
        let s = state.lock().map_err(|_| "Presentation state unavailable")?;
        let vm = s
            .view
            .vm(&target.vm_id)
            .ok_or("Recovery VM unavailable; Refresh first")?;
        (vm.name.clone(), s.recovery_without_guest(&record)?)
    };
    let prompt_name = vm_name.clone();
    let result_vm_name = vm_name.clone();
    background_operation(
        ui,
        state,
        busy,
        Operation::new(
            Kind::Reconcile,
            hyper_gpu_support::ui_text::operations::recovery::PROGRESS_TITLE,
        ),
        Some(&vm_name),
        record.requires_graphics() && !restored_candidate,
        move || {
            let graphics = record.requires_graphics() && !restored_candidate;
            let credential = if graphics && !restored_candidate {
                Some(
                    hyper_gpu_support::credentials::read(&record.target.vm_id)?.map_or_else(
                        || {
                            hyper_gpu_support::credentials::prompt_named(
                                &record.target.vm_id,
                                &prompt_name,
                                windows::Win32::Foundation::HWND::default(),
                            )
                        },
                        Ok,
                    )?,
                )
            } else {
                None
            };
            hyper_gpu_support::worker::reconcile(
                record.operation_id,
                graphics,
                credential,
                progress,
            )
        },
        move |ui, result, readback| {
            if let Err(error) = &result
                && credential_request_cancelled(ui, error, true)
            {
                return;
            }
            if !with_completion_state(ui, &finished_state, &result, |state| {
                let status = if let Ok(outcome) = &result
                    && publication
                    && outcome.operation.is_some()
                {
                    state.finish_apply(&target, &outcome.operation_id, outcome.saved, readback)
                } else {
                    state.finish_readback(readback)
                };
                ui.set_dirty(state.view.draft.is_some());
                present(ui, state, !ui.get_dirty());
                status
            }) {
                return;
            }
            match result {
                Ok(outcome) if outcome.finished && outcome.operation.is_none() => dialog(
                    ui,
                    "recovery-ended",
                    hyper_gpu_support::ui_text::operations::recovery::ended::TITLE,
                    hyper_gpu_support::ui_text::operations::recovery::ended::BODY,
                ),
                Ok(outcome) if outcome.finished => dialog(
                    ui,
                    "info",
                    hyper_gpu_support::ui_text::operations::recovery::confirmed::TITLE,
                    hyper_gpu_support::ui_text::operations::recovery::confirmed::BODY,
                ),
                Ok(outcome) => dialog(
                    ui,
                    "save",
                    hyper_gpu_support::ui_text::operations::recovery::unsaved::TITLE,
                    &format!(
                        "{}\nRetry configuration saving only.",
                        outcome.save_error.unwrap_or_default()
                    ),
                ),
                Err(error) => {
                    dialog(
                        ui,
                        "error",
                        hyper_gpu_support::ui_text::operations::recovery::blocked::TITLE,
                        &error,
                    );
                    ui.set_dialog_kind("recovery-blocked".into());
                    ui.set_dialog_body(format!(
                        "{}\nUse Retry status update to read the current facts. Delete profile instead reviews deletion of this VM's app settings, pairing, history and interrupted-operation records. It leaves the VM, GPU support and guest files unchanged.",
                        hyper_gpu_support::reporting::operator_error(hyper_gpu_support::ui_text::operations::recovery::FAILURE_STAGE, &error),
                    ).into());
                    ui.set_recovery_vm_id(recovery_vm_id.clone().into());
                    ui.set_recovery_heading(
                        hyper_gpu_support::ui_text::operations::action::review_prompt(
                            &result_vm_name,
                        )
                        .into_string()
                        .into(),
                    );
                }
            }
        },
    )
}

#[cfg(not(windows))]
pub(super) fn reconcile_live(
    _: &AppWindow,
    _: &SharedState,
    _: &Arc<AtomicBool>,
) -> Result<(), String> {
    Err("Reconciliation requires Windows".into())
}
