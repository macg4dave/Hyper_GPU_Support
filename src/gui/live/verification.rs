//! Independent graphics-check review, execution and completion readback.
use super::SharedState;
#[cfg(windows)]
use super::{
    background::background_operation, credentials::credential_request_cancelled,
    progress::progress_to_ui, state_access::complete_operation,
};
use crate::gui::AppWindow;
#[cfg(windows)]
use crate::gui::{
    dialog, failure_dialog,
    operation::{Kind, Operation},
    wording,
};
#[cfg(windows)]
use slint::ComponentHandle;
use std::sync::{Arc, atomic::AtomicBool};

#[cfg(windows)]
pub(super) fn review_verification(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let target = {
        let state = state.lock().map_err(|_| "Presentation state unavailable")?;
        if state.view.draft.is_some() {
            return Err(
                "Discard or Apply the draft before reviewing a separate graphics check.".into(),
            );
        }
        state.editor_eligibility(ui.get_selected_id().as_str())?;
        state.target(ui.get_selected_id().as_str())?
    };
    let expected = state
        .lock()
        .map_err(|_| "Presentation state unavailable")?
        .committed
        .revision(&target.vm_id)?;
    let vm_name = state
        .lock()
        .map_err(|_| "Presentation state unavailable")?
        .view
        .vm(&target.vm_id)
        .ok_or(hyper_gpu_support::ui_text::dashboard::selection::VM_UNAVAILABLE)?
        .name
        .clone();
    let approval_state = state.clone();
    let approval_ui = ui.as_weak();
    let requested = target.clone();
    let expected_revision = expected.clone();
    let progress = progress_to_ui(ui.as_weak());
    let finished_state = state.clone();
    let result_vm_name = vm_name.clone();
    let started = background_operation(
        ui,
        state,
        busy,
        Operation::new(
            Kind::Verify,
            hyper_gpu_support::ui_text::operations::verify::title().as_str(),
        ),
        Some(&vm_name),
        true,
        move || {
            hyper_gpu_support::worker::verify_prepared(
                target,
                expected,
                |review| {
                    let hyper_gpu_support::worker::PreparedReview::Verify { plan, expected } =
                        review
                    else {
                        return Err("Unexpected prepared operation".into());
                    };
                    let accepted = plan.clone();
                    let revision = expected.clone();
                    let requested = requested.clone();
                    let original_revision = expected_revision.clone();
                    let state = approval_state.clone();
                    super::background::accept_prepared(&approval_ui, move |ui| {
                        if !ui.get_review_preparing() {
                            return Err("Read-only preparation cancelled".into());
                        }
                        let state = state.lock().map_err(|_| "Presentation state unavailable")?;
                        state.editor_eligibility(&requested.vm_id)?;
                        if state.view.selected.as_deref() != Some(requested.vm_id.as_str())
                            || state.view.draft.is_some()
                            || state.target(&requested.vm_id)? != requested
                            || state.committed.revision(&requested.vm_id)? != revision
                            || revision != original_revision
                        {
                            return Err("Selection or saved settings changed during verification preparation".into());
                        }
                        ui.set_review_preparing(false);
                        ui.set_dialog_details(wording::verify_details(&accepted).into());
                        Ok(())
                    })?;
                    let credential = hyper_gpu_support::credentials::read(&plan.target.vm_id)?
                        .map_or_else(
                            || {
                                hyper_gpu_support::credentials::prompt_named(
                                    &plan.target.vm_id,
                                    &plan.observed.name,
                                    windows::Win32::Foundation::HWND::default(),
                                )
                            },
                            Ok,
                        )?;
                    Ok(hyper_gpu_support::worker::Authorization {
                        shutdown_approved: plan.observed.power
                            == hyper_gpu_support::model::Power::Off,
                        credential: Some(credential),
                    })
                },
                progress,
            )
        },
        move |ui, result, readback| {
            if let Err(error) = &result
                && credential_request_cancelled(ui, error, false)
            {
                return;
            }
            if complete_operation(ui, &finished_state, None, &result, readback).is_none() {
                return;
            }
            match result {
                Ok(outcome) if outcome.finished => dialog(
                    ui,
                    "info",
                    hyper_gpu_support::ui_text::operations::verify::completed::TITLE,
                    &hyper_gpu_support::ui_text::operations::verify::completed::body(
                        &(result_vm_name),
                    )
                    .into_string(),
                ),
                Ok(_) => dialog(
                    ui,
                    "error",
                    hyper_gpu_support::ui_text::operations::verify::unknown::TITLE,
                    &hyper_gpu_support::ui_text::operations::verify::unknown::body(
                        &(result_vm_name),
                    )
                    .into_string(),
                ),
                Err(error) => failure_dialog(
                    ui,
                    hyper_gpu_support::ui_text::operations::verify::incomplete::TITLE,
                    &error,
                    &format!(
                        "{result_vm_name}: The check did not complete. Use Refresh and Review recovery before another operation; prior successful checks do not establish current health."
                    ),
                ),
            }
        },
    );
    if started.is_ok() {
        ui.set_review_preparing(true);
    }
    started
}

#[cfg(not(windows))]
pub(super) fn review_verification(
    _: &AppWindow,
    _: &SharedState,
    _: &Arc<AtomicBool>,
) -> Result<(), String> {
    Err("Verify GPU setup requires Windows".into())
}
