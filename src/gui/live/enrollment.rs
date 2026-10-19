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
    let proposed = {
        let state = state
            .lock()
            .map_err(|_| hyper_gpu_support::ui_text::common::state::presentation_unavailable())?;
        state.permission_review(&target, hyper_gpu_support::setup::operator_sid()?)?
    };
    if let Some(approved) = proposed {
        return enroll_live(ui, state, busy, approved, None, target);
    }
    let reviewed_state = state.clone();
    let expected = target.clone();
    let enroll_busy = busy.clone();
    background(
        ui,
        state,
        busy,
        Operation::new(
            Kind::PermissionPlan,
            hyper_gpu_support::ui_text::operations::enrollment::review::title(),
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
                        hyper_gpu_support::ui_text::operations::enrollment::other_pair().into(),
                    );
                }
                Ok((approved, Some(artifacts), true))
            }
        },
        move |ui, result| match result {
            Ok((approved, artifacts, worker_available)) => {
                ui.set_execution_ready(worker_available);
                super::background::continue_prepared(ui, Ok(()), || {
                    enroll_live(
                        ui,
                        &reviewed_state,
                        &enroll_busy,
                        approved,
                        artifacts,
                        expected,
                    )
                });
            }
            Err(error) => dialog(
                ui,
                "error",
                hyper_gpu_support::ui_text::operations::enrollment::preparation_failed(),
                &error,
            ),
        },
    )
}

#[cfg(windows)]
fn enroll_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    approved: hyper_gpu_support::enrollment::PairReview,
    artifacts: Option<std::collections::BTreeMap<String, String>>,
    expected: Target,
) -> Result<(), String> {
    if !ui.get_execution_ready() {
        return Err(hyper_gpu_support::ui_text::operations::worker::unavailable_refresh().into());
    }
    if !with_state(ui, state, |state| {
        state.validate_permission_preparation(&expected, &draft_target(ui, state)?, &approved)
    }) {
        return Ok(());
    }
    let source = Source;
    let finished_state = state.clone();
    let refresh_busy = busy.clone();
    let progress = progress_to_ui(ui.as_weak());
    background(
        ui,
        state,
        busy,
        Operation::new(
            if artifacts.is_some() {
                Kind::Install
            } else {
                Kind::Setup
            },
            if artifacts.is_some() {
                hyper_gpu_support::ui_text::dialogs::workflow::install_progress_title()
            } else {
                hyper_gpu_support::ui_text::operations::enrollment::progress_title()
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
                        hyper_gpu_support::ui_text::operations::plan::unavailable_title(),
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
                    hyper_gpu_support::ui_text::operations::enrollment::incomplete::title(),
                    &error,
                    hyper_gpu_support::ui_text::operations::enrollment::incomplete::context(),
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
        let state = state
            .lock()
            .map_err(|_| hyper_gpu_support::ui_text::common::state::presentation_unavailable())?;
        let id = state
            .enrollment_recovery
            .as_ref()
            .ok_or(hyper_gpu_support::ui_text::operations::enrollment::refresh_recovery())?
            .operation_id
            .clone();
        let record = state
            .enrollment_recovery
            .as_ref()
            .ok_or(hyper_gpu_support::ui_text::operations::enrollment::refresh_recovery())?;
        let expected = hyper_gpu_support::runner::pair_revision()?;
        if expected != record.resulting && expected != record.review.expected {
            return Err(
                hyper_gpu_support::ui_text::operations::enrollment::authority_mismatch().into(),
            );
        }
        (id, expected)
    };
    let finished_state = state.clone();
    let refresh_busy = busy.clone();
    let source = source.clone();
    let progress = progress_to_ui(ui.as_weak());
    background(
        ui,
        state,
        busy,
        Operation::new(
            Kind::PermissionRecovery,
            hyper_gpu_support::ui_text::operations::enrollment::recovery::progress_title(),
        ),
        move || hyper_gpu_support::worker::reconcile_pair(operation_id, expected, progress),
        move |ui, result| match result {
            Ok(_) => {
                if let Err(error) = refresh(ui, &finished_state, &refresh_busy, &source, false) {
                    dialog(ui, "error", hyper_gpu_support::ui_text::operations::enrollment::recovery::status_unavailable_title(), &error);
                }
            }
            Err(error) => {
                ui.set_execution_ready(false);
                dialog(
                    ui,
                    "error",
                    hyper_gpu_support::ui_text::operations::enrollment::recovery::retained_title(),
                    &error,
                );
            }
        },
    )
}
