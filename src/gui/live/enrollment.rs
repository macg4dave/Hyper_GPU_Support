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
    set_details,
};
use hyper_gpu_support::model::Target;
use slint::{ComponentHandle, Model};
use std::sync::{Arc, atomic::AtomicBool};

#[cfg(windows)]
pub(super) fn review_enrollment(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    target: Target,
) -> Result<(), String> {
    let reviewed_state = state.clone();
    let values = ui
        .get_values()
        .iter()
        .take(3)
        .map(|v| v.to_string())
        .collect::<Vec<_>>();
    let allocation = crate::gui::memory::review_allocation(&values)?;
    background(
        ui,
        busy,
        Operation::new(
            Kind::Plan,
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
                let heading = hyper_gpu_support::ui_text::operations::enrollment::review::body(
                    &(ui.get_vm_name()),
                    &(ui.get_gpu_options()
                        .row_data(usize::try_from(ui.get_gpu_index()).unwrap_or(usize::MAX))
                        .unwrap_or_default()),
                )
                .into_string();
                let summary = hyper_gpu_support::ui_text::operations::enrollment::ELEVATION_NOTICE;
                let mut body = format!(
                    "VM ID: {}\nGPU interface: {}\nOperator: {}\nPolicy revision: {:?}\n\nAuthorize this exact VM/GPU pair through Windows elevation, preserving all other enrolled VMs. Enrollment does not enable GPU support, change guest power, prepare drivers or save desired configuration. Your draft is retained. After fresh authority readback, Apply obtains a separate GPU operation plan.",
                    approved.vm_id,
                    approved.gpu_interface,
                    approved.operator_sid,
                    approved.expected
                );
                if let Some(pins) = &artifacts {
                    body.push_str(
                        hyper_gpu_support::ui_text::operations::enrollment::INSTALL_DETAILS,
                    );
                    for (name, hash) in pins {
                        body.push_str(
                            &hyper_gpu_support::ui_text::operations::enrollment::artifact_pin(
                                name, hash,
                            )
                            .into_string(),
                        );
                    }
                }
                if !with_state(ui, &reviewed_state, |state| {
                    state.enrollment_review = Some(approved);
                    state.setup_artifacts = artifacts;
                    Ok(())
                }) {
                    return;
                }
                ui.set_execution_ready(worker_available);
                ui.set_enrollment_heading(heading.into());
                ui.set_enrollment_allocation(allocation.into());
                ui.set_needs_shutdown(false);
                dialog(
                    ui,
                    "enroll-review",
                    hyper_gpu_support::ui_text::operations::enrollment::review::TITLE,
                    summary,
                );
                set_details(ui, &body);
            }
            Err(error) => dialog(ui, "error", "Permission review unavailable", &error),
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
            Kind::Setup,
            hyper_gpu_support::ui_text::operations::enrollment::PROGRESS_TITLE,
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
        let mut state = state.lock().map_err(|_| "Presentation state unavailable")?;
        let id = state
            .enrollment_recovery
            .as_ref()
            .ok_or("Refresh enrollment recovery first")?
            .operation_id
            .clone();
        let expected = state
            .enrollment_recovery_revision
            .take()
            .ok_or("Review authority readback first")?;
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
            Kind::Setup,
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
