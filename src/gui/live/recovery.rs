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

/// Only an explicit foreground action enters recovery. Refresh and result
/// publication never call this; reconciliation never replays Apply.
pub(super) fn start_recovery(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &super::source::Source,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        let (permission, import, save, operation) = {
            let state = state.lock().map_err(|_| "Presentation state unavailable")?;
            (
                state.enrollment_recovery.is_some(),
                state.import_recovery.is_some(),
                state.recovery.as_ref().is_some_and(|record| {
                    record.publication_required
                        && matches!(
                            record.phase,
                            hyper_gpu_support::configuration_store::SavePhase::Verified(_)
                                | hyper_gpu_support::configuration_store::SavePhase::Published(_)
                        )
                }),
                state.recovery.is_some(),
            )
        };
        if permission {
            return super::enrollment::reconcile_enrollment_live(ui, state, busy, source);
        }
        if import {
            return review_records(
                ui,
                &mut *state.lock().map_err(|_| "Presentation state unavailable")?,
                true,
            );
        }
        if save {
            return super::apply::save_only_live(ui, state, busy);
        }
        if operation {
            return reconcile_live(ui, state, busy);
        }
        review_records(
            ui,
            &mut *state.lock().map_err(|_| "Presentation state unavailable")?,
            false,
        )
    }
    #[cfg(not(windows))]
    {
        let _ = (ui, state, busy, source);
        Err("Recovery requires Windows".into())
    }
}

pub(super) fn review_records(
    ui: &AppWindow,
    state: &mut State,
    recovery: bool,
) -> Result<(), String> {
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
    let selected = crate::gui::diagnostics::selected_vm_details(state);
    let failure = ui.get_inventory_error();
    // Initial discovery failure must not make its own diagnostics unavailable.
    let body = [ui.get_inventory_details().as_str(), selected.as_str()]
        .into_iter()
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    let provenance = hyper_gpu_support::ui_text::operations::inventory::PROVENANCE;
    dialog(
        ui,
        "info",
        hyper_gpu_support::ui_text::operations::inventory::DETAILS_TITLE,
        &if !failure.is_empty() {
            failure.to_string()
        } else if state.view.inventory.is_none() {
            hyper_gpu_support::ui_text::operations::inventory::INITIAL_NOTICE.to_owned()
        } else {
            format!(
                "{provenance}\n\nTechnical Details contains the host observation, recorded history and authorization from that inventory. Missing or old records do not establish current guest health. Opening these details performs no new checks or changes."
            )
        },
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
    let progress_body = wording::recovery_progress(&record, &vm_name, restored_candidate);
    let started = background_operation(
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
    );
    if started.is_ok() {
        ui.set_dialog_body(progress_body.into());
    }
    started
}

#[cfg(not(windows))]
pub(super) fn reconcile_live(
    _: &AppWindow,
    _: &SharedState,
    _: &Arc<AtomicBool>,
) -> Result<(), String> {
    Err("Reconciliation requires Windows".into())
}
