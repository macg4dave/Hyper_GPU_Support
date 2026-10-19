//! Confirmed GPU changes and separate configuration-only publication retries.
use super::SharedState;
#[cfg(windows)]
use super::{
    background::background_operation, credentials::credential_request_cancelled, install_dialog,
    installation, presentation::draft_target, progress::progress_to_ui,
    state_access::complete_operation,
};
use crate::gui::AppWindow;
#[cfg(windows)]
use crate::gui::{
    dialog, failure_dialog, model,
    operation::{Kind, Operation},
};
#[cfg(windows)]
use slint::ComponentHandle;
use std::sync::{Arc, atomic::AtomicBool};

#[cfg(windows)]
pub(super) fn apply_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    reinstall: bool,
) -> Result<(), String> {
    if !ui.get_execution_ready() {
        return Err(hyper_gpu_support::ui_text::operations::apply::worker_unavailable().into());
    }
    let (selected, revision, gpu_name, vm_name, title) = {
        let state = state
            .lock()
            .map_err(|_| hyper_gpu_support::ui_text::common::state::presentation_unavailable())?;
        let target = draft_target(ui, &state)?;
        state.editor_eligibility(&target.vm_id)?;
        let revision = state.committed.revision(&target.vm_id)?;
        let inventory = state
            .view
            .inventory
            .as_ref()
            .ok_or(hyper_gpu_support::ui_text::common::errors::inventory_unavailable())?;
        let gpu_name = inventory
            .discovery
            .gpus
            .iter()
            .find(|gpu| gpu.interface == target.gpu_interface)
            .map_or_else(|| target.gpu_interface.clone(), |gpu| gpu.name.clone());
        let vm = state
            .view
            .vm(&target.vm_id)
            .ok_or(hyper_gpu_support::ui_text::dashboard::selection::vm_unavailable())?;
        let title = installation::apply_title(target.enabled, !vm.gpus.is_empty());
        (target, revision, gpu_name, vm.name.clone(), title)
    };
    let finished_state = state.clone();
    let result_vm_name = vm_name.clone();
    let prepared_plan = Arc::new(std::sync::Mutex::new(
        None::<hyper_gpu_support::workflow::Plan>,
    ));
    let result_plan = prepared_plan.clone();
    let approval_state = state.clone();
    let approval_ui = ui.as_weak();
    let requested = selected.clone();
    let expected = revision.clone();
    let preparation_gpu = gpu_name.clone();
    install_dialog::prepare(ui, &vm_name, &[]);
    install_dialog::context(ui, &gpu_name, &selected);
    let progress = progress_to_ui(ui.as_weak());
    let started = background_operation(
        ui,
        state,
        busy,
        Operation::new(Kind::Apply, title),
        Some(&vm_name),
        true,
        move || {
            hyper_gpu_support::worker::apply_prepared(
                requested,
                reinstall,
                expected,
                |review| {
                    let hyper_gpu_support::worker::PreparedReview::Apply { plan, expected } =
                        review
                    else {
                        return Err(
                            hyper_gpu_support::ui_text::common::errors::unexpected_preparation()
                                .into(),
                        );
                    };
                    let revision = expected.clone();
                    let accepted = plan.clone();
                    let state = approval_state.clone();
                    let gpu = preparation_gpu.clone();
                    let plan = super::background::accept_prepared(&approval_ui, move |ui| {
                        if !ui.get_review_preparing() {
                            return Err(
                                hyper_gpu_support::ui_text::common::errors::preparation_cancelled()
                                    .into(),
                            );
                        }
                        let state = state.lock().map_err(|_| {
                            hyper_gpu_support::ui_text::common::state::presentation_unavailable()
                        })?;
                        let current = draft_target(ui, &state)?;
                        state.validate_prepared(&accepted, &current, &revision)?;
                        ui.set_review_preparing(false);
                        ui.set_install_vm_name(accepted.observed.name.clone().into());
                        install_dialog::context(ui, &gpu, &accepted.desired);
                        ui.set_install_guest_work(accepted.preview.credentials_required);
                        ui.set_dialog_details(
                            crate::gui::live_state::review_text(&accepted).into(),
                        );
                        Ok(accepted)
                    })?;
                    *prepared_plan.lock().map_err(|_| {
                        hyper_gpu_support::ui_text::common::errors::prepared_result_unavailable()
                    })? = Some(plan.clone());
                    let credential =
                        installation::credentials(plan.preview.credentials_required, || {
                            hyper_gpu_support::credentials::prompt_named(
                                &plan.desired.vm_id,
                                &plan.observed.name,
                                windows::Win32::Foundation::HWND::default(),
                            )
                        })?;
                    Ok(hyper_gpu_support::worker::Authorization {
                        shutdown_approved: plan.preview.guest_downtime,
                        credential,
                    })
                },
                progress,
            )
        },
        move |ui, result, readback| {
            let plan = result_plan.lock().ok().and_then(|plan| plan.clone());
            let setup_required = plan.as_ref().is_some_and(|plan| {
                plan.preview
                    .actions
                    .contains(&hyper_gpu_support::workflow::PreviewAction::Verify)
            });
            let final_power = plan
                .as_ref()
                .map_or(hyper_gpu_support::model::Power::Off, |plan| {
                    plan.preview.restore_power.clone()
                });
            if ui.get_dialog_kind() == "cancelling" {
                ui.set_operation_dialog_active(false);
                ui.set_dialog_kind("".into());
                return;
            }
            if let Err(error) = &result
                && credential_request_cancelled(ui, error, false)
            {
                return;
            }
            let Some(published) =
                complete_operation(ui, &finished_state, Some(&selected), &result, readback)
            else {
                return;
            };
            match result {
                Ok(outcome) => {
                    if published {
                        match install_dialog::completion(
                            &selected,
                            outcome.operation.as_ref(),
                            outcome.finished,
                            outcome.saved && published,
                            &gpu_name,
                            &final_power,
                            setup_required,
                        ) {
                            Ok((body, facts)) => {
                                let log = ui.get_dialog_details();
                                dialog(
                                    ui,
                                    "install-complete",
                                    hyper_gpu_support::ui_text::dialogs::install::completed(),
                                    &body,
                                );
                                ui.set_install_facts(model(facts));
                                ui.set_dialog_details(log);
                            }
                            Err(error) => failure_dialog(
                                ui,
                                hyper_gpu_support::ui_text::operations::setup::incomplete::title(),
                                &error,
                                hyper_gpu_support::ui_text::operations::apply::may_have_completed(),
                            ),
                        }
                    } else {
                        dialog(
                            ui,
                            "save",
                            if setup_required {
                                hyper_gpu_support::ui_text::operations::setup::unsaved::title()
                            } else {
                                hyper_gpu_support::ui_text::operations::apply::unsaved::title()
                            },
                            &hyper_gpu_support::ui_text::operations::apply::unsaved::body(
                                &if setup_required {
                                    hyper_gpu_support::ui_text::operations::setup::unsaved::effect_summary(&(result_vm_name)).into_string()
                                } else {
                                    hyper_gpu_support::ui_text::operations::apply::unsaved::effect_summary(&(result_vm_name)).into_string()
                                },
                                &outcome.save_error.unwrap_or_default(),
                            ),
                        );
                    }
                }
                Err(error) => present_failure(ui, &error, setup_required),
            }
        },
    );
    if started.is_ok() {
        ui.set_review_preparing(true);
        ui.set_dialog_body(hyper_gpu_support::ui_text::operations::plan::progress_body().into());
    }
    started
}

/// Called only after the operation's independent completion readback succeeded.
#[cfg(any(windows, test))]
fn present_failure(ui: &AppWindow, error: &str, setup_required: bool) {
    use slint::Model;
    let title = if setup_required {
        hyper_gpu_support::ui_text::operations::setup::incomplete::title()
    } else {
        hyper_gpu_support::ui_text::operations::apply::incomplete::title()
    };
    let last_stage = ui.get_stages().iter().last().map(|stage| stage.to_string());
    crate::gui::failure_dialog(
        ui,
        title,
        error,
        &hyper_gpu_support::ui_text::operations::apply::failure_context(
            &last_stage.map_or_else(
                || hyper_gpu_support::ui_text::operations::apply::stage_absent().into(),
                |stage| {
                    hyper_gpu_support::ui_text::operations::apply::last_stage(&stage).into_string()
                },
            ),
            ui.get_power().as_str(),
            if ui.get_action_recovery() {
                hyper_gpu_support::ui_text::operations::apply::next_recovery()
            } else {
                hyper_gpu_support::ui_text::operations::apply::next_inspect()
            },
        ),
    );
}

#[cfg(windows)]
pub(super) fn save_only_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let record = state
        .lock()
        .map_err(|_| hyper_gpu_support::ui_text::common::state::presentation_unavailable())?
        .save_only_record()?;
    let finished_state = state.clone();
    let target = record.target.clone();
    let progress = progress_to_ui(ui.as_weak());
    background_operation(
        ui,
        state,
        busy,
        Operation::new(
            Kind::SaveOnly,
            hyper_gpu_support::ui_text::operations::save_only::progress_title(),
        ),
        None,
        false,
        move || hyper_gpu_support::worker::save_only(record.operation_id, progress),
        move |ui, result, readback| {
            if complete_operation(ui, &finished_state, Some(&target), &result, readback).is_none() {
                return;
            }
            match result {
                Ok(outcome) if outcome.saved => {
                    dialog(
                        ui,
                        "info",
                        hyper_gpu_support::ui_text::operations::save_only::completed::title(),
                        hyper_gpu_support::ui_text::operations::save_only::completed::body(),
                    );
                }
                Ok(_) => dialog(
                    ui,
                    "error",
                    hyper_gpu_support::ui_text::operations::save_only::unsaved::title(),
                    hyper_gpu_support::ui_text::operations::save_only::unsaved::body(),
                ),
                Err(error) => failure_dialog(
                    ui,
                    hyper_gpu_support::ui_text::operations::save_only::blocked::title(),
                    &error,
                    hyper_gpu_support::ui_text::operations::save_only::blocked::context(),
                ),
            }
        },
    )
}

#[cfg(not(windows))]
pub(super) fn apply_live(
    _: &AppWindow,
    _: &SharedState,
    _: &Arc<AtomicBool>,
    _: bool,
) -> Result<(), String> {
    Err(hyper_gpu_support::ui_text::common::errors::windows_required_gpu().into())
}

#[cfg(not(windows))]
pub(super) fn save_only_live(
    _: &AppWindow,
    _: &SharedState,
    _: &Arc<AtomicBool>,
) -> Result<(), String> {
    Err(hyper_gpu_support::ui_text::common::errors::windows_required_publication().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper_gpu_support::reporting::{Component, ComponentFailure, FailureCause, FailureStage};

    #[test]
    fn failed_apply_after_current_readback_offers_loaded_recovery_without_manual_refresh() {
        slint::platform::set_platform(Box::new(crate::gui::catalogue_tests::Headless)).unwrap();
        let ui = AppWindow::new().unwrap();
        let error = ComponentFailure {
            stage: Some(FailureStage::Preparation),
            component: Component::GuestTransport,
            cause: FailureCause::InvalidReply,
            code: None,
        }
        .into_error();
        for (power, recovery) in [("Off", true), ("Running", false)] {
            ui.set_power(power.into());
            ui.set_action_recovery(recovery);
            ui.set_operation_dialog_active(true);
            ui.set_stages(crate::gui::model(vec![
                "Preparing signed driver files".into(),
            ]));
            ui.set_stage_reportable(crate::gui::model(vec![true]));
            present_failure(&ui, &error, true);
            assert_eq!(ui.get_dialog_kind(), "operation-error");
            let body = ui.get_dialog_body();
            assert!(body.contains("incomplete or invalid reply"), "{body}");
            assert!(body.contains("refreshed automatically"), "{body}");
            assert!(body.contains(&format!("Power: {power}.")), "{body}");
            assert!(body.contains("Your draft is retained"), "{body}");
            assert_eq!(body.contains("Choose Review recovery"), recovery, "{body}");
            assert!(
                !body.contains("Use Refresh") && !body.contains("use Refresh"),
                "{body}"
            );
            assert!(
                !body.contains("final VM state has not been confirmed"),
                "{body}"
            );
            let details = ui.get_dialog_details();
            for fact in [
                "GuestTransport",
                "InvalidReply",
                "Preparation",
                "Preparing signed driver files",
            ] {
                assert!(details.contains(fact), "Missing {fact}: {details}");
            }
        }
        present_failure(&ui, "private adapter text 0x80010106", true);
        let body = ui.get_dialog_body();
        assert!(body.contains("refreshed automatically"), "{body}");
        assert!(!body.contains("Refresh"), "{body}");
        assert!(!body.contains("private adapter text"), "{body}");
        assert!(ui.get_dialog_details().contains("0x80010106"));
    }
}
