//! Exact-pair enrollment reviews, setup and authority recovery.
use super::{
    SharedState,
    background::background,
    presentation::{draft_target, present},
    progress::progress_to_ui,
    refresh::{refresh, refresh_with_result},
    source::{Source, protected_worker_available},
    state_access::with_state,
};
use crate::gui::{
    AppWindow, dialog, failure_dialog,
    operation::{Kind, Operation},
};
use hyper_gpu_support::model::Target;
use slint::ComponentHandle;
use std::sync::{Arc, atomic::AtomicBool};

#[cfg(windows)]
pub(super) fn review_enrollment(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    target: Target,
) -> Result<(), String> {
    let reviewed_state = state.clone();
    let expected = target.clone();
    let enroll_busy = busy.clone();
    background(
        ui,
        busy,
        Operation::new(
            Kind::PermissionPlan,
            hyper_gpu_support::ui_text::operations::enrollment::review::TITLE,
        ),
        move || {
            let pending = hyper_gpu_support::runner::pending_setup()?;
            if pending.is_none() && hyper_gpu_support::runner::installed()? {
                hyper_gpu_support::runner::review_pair(&target)
                    .map(|review| (review, None, protected_worker_available()))
            } else {
                let approved = hyper_gpu_support::enrollment::PairReview {
                    vm_id: target.vm_id,
                    gpu_interface: target.gpu_interface,
                    expected: hyper_gpu_support::configuration_store::Revision::Missing,
                    operator_sid: hyper_gpu_support::setup::operator_sid()?,
                };
                approved.target()?;
                let artifacts = hyper_gpu_support::setup::source_artifacts()?;
                if let Some(record) = pending
                    && (record.review != approved || record.artifacts != artifacts)
                {
                    return Err(
                        "Interrupted first-install setup belongs to another original pair/package. Select its original VM/GPU and reviewed package; setup recovery cannot replace authority."
                            .into(),
                    );
                }
                Ok((approved, Some(artifacts), true))
            }
        },
        move |ui, result| match result {
            Ok((approved, artifacts, worker_available)) => {
                if !with_state(ui, &reviewed_state, |state| {
                    if draft_target(ui, state)? != expected
                        || state.view.selected.as_deref() != Some(expected.vm_id.as_str())
                    {
                        return Err("Selection or draft changed during permission preparation. Refresh before trying again.".into());
                    }
                    state.enrollment_review = Some(approved);
                    state.setup_artifacts = artifacts;
                    Ok(())
                }) {
                    return;
                }
                ui.set_execution_ready(worker_available);
                super::background::continue_prepared(ui, Ok(()), || {
                    enroll_live(ui, &reviewed_state, &enroll_busy, &Source)
                });
            }
            Err(error) => dialog(ui, "error", "GPU permission preparation failed", &error),
        },
    )
}

#[cfg(windows)]
pub(super) fn enroll_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
) -> Result<(), String> {
    if !ui.get_execution_ready() {
        return Err("The protected worker is unavailable. Refresh before trying again.".into());
    }

    let (approved, artifacts, expected) = {
        let mut state = state.lock().map_err(|_| "Presentation state unavailable")?;
        let approved = state
            .enrollment_review
            .take()
            .ok_or("Review exact enrollment first")?;
        let draft = draft_target(ui, &state)?;
        if draft.vm_id != approved.vm_id || draft.gpu_interface != approved.gpu_interface {
            return Err("Pair changed after enrollment review; review again".into());
        }
        let artifacts = state.setup_artifacts.take();
        (approved, artifacts, draft)
    };
    let source = source.clone();
    let finished_state = state.clone();
    let refresh_busy = busy.clone();
    let progress = progress_to_ui(ui.as_weak());
    background(
        ui,
        busy,
        Operation::new(
            if artifacts.is_some() {
                Kind::Install
            } else {
                Kind::Setup
            },
            if artifacts.is_some() {
                hyper_gpu_support::ui_text::dialogs::workflow::INSTALL_PROGRESS_TITLE
            } else {
                hyper_gpu_support::ui_text::operations::enrollment::PROGRESS_TITLE
            },
        ),
        move || {
            if let Some(artifacts) = artifacts {
                hyper_gpu_support::setup::install(approved, artifacts)
            } else {
                hyper_gpu_support::worker::enroll_pair(approved, progress).map(|_| ())
            }
        },
        move |ui, result| match result {
            Ok(()) => {
                if !with_state(ui, &finished_state, |state| {
                    state.view.needs_readback = true;
                    state.result_notice = None;
                    present(ui, state, false);
                    Ok(())
                }) {
                    return;
                }
                if let Err(error) = refresh_with_result(
                    ui,
                    &finished_state,
                    &refresh_busy,
                    &source,
                    false,
                    None,
                    Some(expected),
                ) {
                    dialog(
                        ui,
                        "error",
                        hyper_gpu_support::ui_text::operations::plan::UNAVAILABLE_TITLE,
                        &error,
                    );
                }
            }
            Err(error) => {
                if !with_state(ui, &finished_state, |state| {
                    state.view.needs_readback = true;
                    present(ui, state, false);
                    Ok(())
                }) {
                    return;
                }
                ui.set_eligible(false);
                ui.set_execution_ready(false);
                failure_dialog(
                    ui,
                    hyper_gpu_support::ui_text::operations::enrollment::incomplete::TITLE,
                    &error,
                    "Your edits are kept. Use Refresh and Review recovery before retrying this permission step. GPU setup has not been confirmed.",
                );
            }
        },
    )
}

#[cfg(windows)]
pub(super) fn reconcile_enrollment_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
) -> Result<(), String> {
    let (operation_id, expected) = {
        let state = state.lock().map_err(|_| "Presentation state unavailable")?;
        let id = state
            .enrollment_recovery
            .as_ref()
            .ok_or("Refresh enrollment recovery first")?
            .operation_id
            .clone();
        let record = state
            .enrollment_recovery
            .as_ref()
            .ok_or("Refresh enrollment recovery first")?;
        let expected = hyper_gpu_support::runner::pair_revision()?;
        if expected != record.resulting && expected != record.review.expected {
            return Err("Authority does not match either recorded revision. Recovery remains held for administrator inspection.".into());
        }
        (id, expected)
    };
    let finished_state = state.clone();
    let refresh_busy = busy.clone();
    let source = source.clone();
    let progress = progress_to_ui(ui.as_weak());
    background(
        ui,
        busy,
        Operation::new(
            Kind::PermissionRecovery,
            hyper_gpu_support::ui_text::operations::enrollment::recovery::PROGRESS_TITLE,
        ),
        move || hyper_gpu_support::worker::reconcile_pair(operation_id, expected, progress),
        move |ui, result| match result {
            Ok(_) => {
                if let Err(error) = refresh(ui, &finished_state, &refresh_busy, &source, false) {
                    dialog(ui, "error", hyper_gpu_support::ui_text::operations::enrollment::recovery::STATUS_UNAVAILABLE_TITLE, &error);
                }
            }
            Err(error) => {
                ui.set_execution_ready(false);
                dialog(
                    ui,
                    "error",
                    hyper_gpu_support::ui_text::operations::enrollment::recovery::RETAINED_TITLE,
                    &error,
                );
            }
        },
    )
}
