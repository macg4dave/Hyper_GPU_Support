//! Independent graphics-check review, execution and completion readback.
use super::SharedState;
#[cfg(windows)]
use super::{
    background::{background, background_operation},
    credentials::credential_request_cancelled,
    presentation::present,
    progress::progress_to_ui,
    source::protected_worker_available,
    state_access::{with_completion_state, with_state},
};
use crate::gui::AppWindow;
#[cfg(windows)]
use crate::gui::{
    dialog, failure_dialog,
    operation::{Kind, Operation},
    set_details, wording,
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
    let review_state = state.clone();
    background(
        ui,
        busy,
        Operation::new(
            Kind::VerifyPlan,
            hyper_gpu_support::ui_text::operations::verify::review::PROGRESS_TITLE,
        ),
        move || {
            let value = hyper_gpu_support::runner::submit(hyper_gpu_support::runner::request(
                hyper_gpu_support::runner::Operation::VerifyPlan,
                Some(target),
                None,
            ))?;
            serde_json::from_value::<hyper_gpu_support::workflow::VerificationPlan>(value)
                .map_err(|e| e.to_string())
                .map(|plan| (plan, protected_worker_available()))
        },
        move |ui, result| match result {
            Ok((plan, worker_available)) => {
                ui.set_execution_ready(worker_available);
                ui.set_needs_shutdown(plan.observed.power == hyper_gpu_support::model::Power::Off);
                ui.set_power_consent(wording::verify_consent(&plan).into());
                let body = wording::verify_review(&plan);
                let details = wording::verify_details(&plan);
                if !with_state(ui, &review_state, |state| {
                    state.verification_plan = Some(plan);
                    Ok(())
                }) {
                    return;
                }
                dialog(
                    ui,
                    "verify-review",
                    hyper_gpu_support::ui_text::operations::verify::review::TITLE,
                    &body,
                );
                set_details(ui, &details);
            }
            Err(error) => dialog(ui, "error", "Verify GPU setup unavailable", &error),
        },
    )
}
#[cfg(windows)]
pub(super) fn verify_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let approved = state
        .lock()
        .map_err(|_| "Presentation state unavailable")?
        .verification_plan
        .clone()
        .ok_or("Review verification first")?;
    let shutdown = approved.observed.power == hyper_gpu_support::model::Power::Off;
    let progress = progress_to_ui(ui.as_weak());
    let finished_state = state.clone();
    let vm_name = approved.observed.name.clone();
    let result_vm_name = vm_name.clone();
    background_operation(
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
            let credential = hyper_gpu_support::credentials::read(&approved.target.vm_id)?
                .map_or_else(
                    || {
                        hyper_gpu_support::credentials::prompt_named(
                            &approved.target.vm_id,
                            &approved.observed.name,
                            windows::Win32::Foundation::HWND::default(),
                        )
                    },
                    Ok,
                )?;
            hyper_gpu_support::worker::verify(approved, shutdown, credential, progress)
        },
        move |ui, result, readback| {
            if let Err(error) = &result
                && credential_request_cancelled(ui, error, false)
            {
                with_state(ui, &finished_state, |state| {
                    state.verification_plan = None;
                    Ok(())
                });
                return;
            }
            if !with_completion_state(ui, &finished_state, &result, |state| {
                state.verification_plan = None;
                state.view.needs_readback = true;
                let status = state.finish_readback(readback);
                present(ui, state, false);
                status
            }) {
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
    )
}

#[cfg(not(windows))]
pub(super) fn review_verification(
    _: &AppWindow,
    _: &SharedState,
    _: &Arc<AtomicBool>,
) -> Result<(), String> {
    Err("Verify GPU setup requires Windows".into())
}

#[cfg(not(windows))]
pub(super) fn verify_live(
    _: &AppWindow,
    _: &SharedState,
    _: &Arc<AtomicBool>,
) -> Result<(), String> {
    Err("Verify GPU setup requires Windows".into())
}
