//! Confirmed GPU changes and separate configuration-only publication retries.
use super::SharedState;
#[cfg(windows)]
use super::{
    background::background_operation,
    credentials::credential_request_cancelled,
    install_dialog, installation,
    presentation::{draft_target, present},
    progress::{progress_prepared, progress_to_ui},
    state_access::{with_completion_state, with_state},
};
use crate::gui::AppWindow;
#[cfg(windows)]
use crate::gui::{
    dialog, failure_dialog, model,
    operation::{Kind, Operation},
};
#[cfg(windows)]
use slint::{ComponentHandle, Model};
use std::sync::{Arc, atomic::AtomicBool};

#[cfg(windows)]
pub(super) fn apply_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    reinstall: bool,
) -> Result<(), String> {
    if !ui.get_execution_ready() {
        return Err("The reviewed protected worker is not available.".into());
    }
    let (selected, revision, gpu_name, vm_name) = {
        let state = state.lock().map_err(|_| "Presentation state unavailable")?;
        let target = draft_target(ui, &state)?;
        state.editor_eligibility(&target.vm_id)?;
        let revision = state.committed.revision(&target.vm_id)?;
        let inventory = state
            .view
            .inventory
            .as_ref()
            .ok_or("Inventory unavailable")?;
        let gpu_name = inventory
            .discovery
            .gpus
            .iter()
            .find(|gpu| gpu.interface == target.gpu_interface)
            .map_or_else(|| target.gpu_interface.clone(), |gpu| gpu.name.clone());
        let vm_name = state
            .view
            .vm(&target.vm_id)
            .ok_or(hyper_gpu_support::ui_text::dashboard::selection::VM_UNAVAILABLE)?
            .name
            .clone();
        (target, revision, gpu_name, vm_name)
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
    let (progress, set_plan) = progress_prepared(ui.as_weak());
    let started = background_operation(
        ui,
        state,
        busy,
        Operation::new(
            Kind::Apply,
            hyper_gpu_support::ui_text::operations::apply::PROGRESS_TITLE,
        ),
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
                        return Err("Unexpected prepared operation".into());
                    };
                    let revision = expected.clone();
                    let accepted = plan.clone();
                    let state = approval_state.clone();
                    let gpu = preparation_gpu.clone();
                    let plan = super::background::accept_prepared(&approval_ui, move |ui| {
                        if !ui.get_review_preparing() {
                            return Err("Read-only preparation cancelled".into());
                        }
                        let mut state =
                            state.lock().map_err(|_| "Presentation state unavailable")?;
                        let current = draft_target(ui, &state)?;
                        state.accept_review(accepted.clone(), &current, revision)?;
                        let reviewed = state.reviewed_plan(&current)?;
                        ui.set_review_preparing(false);
                        ui.set_install_vm_name(accepted.observed.name.clone().into());
                        ui.set_install_guest_work(accepted.preview.credentials_required);
                        ui.set_dialog_body(
                            format!(
                                "{}\nPhysical GPU: {}\n{}\n{}",
                                crate::gui::live_state::review_summary(&accepted),
                                gpu,
                                crate::gui::live_state::memory_review(&accepted),
                                crate::gui::wording::apply_consent(&accepted)
                            )
                            .into(),
                        );
                        ui.set_dialog_details(
                            crate::gui::live_state::review_text(&accepted).into(),
                        );
                        Ok(reviewed.plan)
                    })?;
                    set_plan(&plan.preview.actions);
                    *prepared_plan
                        .lock()
                        .map_err(|_| "Prepared result unavailable")? = Some(plan.clone());
                    let credential =
                        installation::credentials(plan.preview.credentials_required, || {
                            hyper_gpu_support::credentials::read(&plan.desired.vm_id)?.map_or_else(
                                || {
                                    hyper_gpu_support::credentials::prompt_named(
                                        &plan.desired.vm_id,
                                        &plan.observed.name,
                                        windows::Win32::Foundation::HWND::default(),
                                    )
                                },
                                Ok,
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
            match result {
                Ok(outcome) => {
                    let mut published = outcome.saved;
                    if !with_state(ui, &finished_state, |state| {
                        let status = state.finish_apply(
                            &selected,
                            &outcome.operation_id,
                            outcome.saved,
                            readback,
                        );
                        published = state.publication_confirmed(&selected);
                        ui.set_dirty(state.view.draft.is_some());
                        present(ui, state, !ui.get_dirty() && published);
                        status
                    }) {
                        return;
                    }
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
                                    hyper_gpu_support::ui_text::dialogs::install::COMPLETED,
                                    &body,
                                );
                                ui.set_install_facts(model(facts));
                                ui.set_dialog_details(log);
                            }
                            Err(error) => failure_dialog(
                                ui,
                                hyper_gpu_support::ui_text::operations::setup::incomplete::TITLE,
                                &error,
                                "Changes may have completed. Inspect current status and recovery before another operation.",
                            ),
                        }
                    } else {
                        dialog(
                            ui,
                            "save",
                            if setup_required {
                                hyper_gpu_support::ui_text::operations::setup::unsaved::TITLE
                            } else {
                                hyper_gpu_support::ui_text::operations::apply::unsaved::TITLE
                            },
                            &format!(
                                "{}{}\nThe verified intent is retained separately from committed configuration. Retry saving only; do not repeat Apply.",
                                if setup_required {
                                    hyper_gpu_support::ui_text::operations::setup::unsaved::effect_summary(&(result_vm_name)).into_string()
                                } else {
                                    hyper_gpu_support::ui_text::operations::apply::unsaved::effect_summary(&(result_vm_name)).into_string()
                                },
                                outcome.save_error.unwrap_or_default(),
                            ),
                        );
                    }
                }
                Err(error) => {
                    if credential_request_cancelled(ui, &error, false) {
                        with_state(ui, &finished_state, |state| {
                            state.approved_plan = None;
                            Ok(())
                        });
                        return;
                    }
                    let stages: Vec<_> = ui.get_stages().iter().collect();
                    let reportable = ui.get_stage_reportable();
                    if !with_completion_state(
                        ui,
                        &finished_state,
                        &Err::<(), _>(error.clone()),
                        |state| {
                            state.view.needs_readback = true;
                            state.approved_plan = None;
                            let status = state.finish_readback(readback);
                            present(ui, state, false);
                            status
                        },
                    ) {
                        return;
                    }
                    ui.set_stages(model(stages));
                    ui.set_stage_reportable(reportable);
                    if hyper_gpu_support::reporting::installation_preflight_blocked(&error) {
                        failure_dialog(
                            ui,
                            "GPU support installation blocked",
                            &error,
                            "This attempt stopped before driver installation or VM power changes. Your draft is retained.",
                        );
                        return;
                    }
                    let last_stage = ui.get_stages().iter().last().map(|stage| stage.to_string());
                    dialog(
                        ui,
                        "error",
                        if setup_required {
                            hyper_gpu_support::ui_text::operations::setup::incomplete::TITLE
                        } else {
                            hyper_gpu_support::ui_text::operations::apply::incomplete::TITLE
                        },
                        &error,
                    );
                    let reason = hyper_gpu_support::reporting::operator_error(
                        hyper_gpu_support::ui_text::operations::apply::FAILURE_STAGE,
                        &error,
                    );
                    ui.set_dialog_body(format!(
                    "{reason}\n{}\nYour draft is retained. Changes may have started; the final VM state has not been confirmed. Use Refresh and inspect recovery before trying again. Do not repeat Apply while recovery is required.",
                    last_stage.map_or_else(|| hyper_gpu_support::ui_text::operations::apply::STAGE_ABSENT.into(), |stage| format!("Last reported step: {stage}")),
                ).into());
                }
            }
        },
    );
    if started.is_ok() {
        ui.set_review_preparing(true);
        ui.set_dialog_body(hyper_gpu_support::ui_text::operations::plan::PROGRESS_BODY.into());
    }
    started
}
#[cfg(windows)]
pub(super) fn save_only_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let record = state
        .lock()
        .map_err(|_| "Presentation state unavailable")?
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
            hyper_gpu_support::ui_text::operations::save_only::PROGRESS_TITLE,
        ),
        None,
        false,
        move || hyper_gpu_support::worker::save_only(record.operation_id, progress),
        move |ui, result, readback| {
            let mut readback = Some(readback);
            if result.is_err()
                && !with_completion_state(ui, &finished_state, &result, |state| {
                    let status = state.finish_readback(
                        readback
                            .take()
                            .ok_or("Completion readback was already consumed")?,
                    );
                    present(ui, state, false);
                    status
                })
            {
                return;
            }
            match result {
                Ok(outcome) if outcome.saved => {
                    if !with_state(ui, &finished_state, |state| {
                        let status = state.finish_apply(
                            &target,
                            &outcome.operation_id,
                            true,
                            readback
                                .take()
                                .ok_or("Completion readback was already consumed")?,
                        );
                        ui.set_dirty(state.view.draft.is_some());
                        present(ui, state, !ui.get_dirty());
                        status
                    }) {
                        return;
                    }
                    dialog(
                        ui,
                        "info",
                        hyper_gpu_support::ui_text::operations::save_only::completed::TITLE,
                        hyper_gpu_support::ui_text::operations::save_only::completed::BODY,
                    );
                }
                Ok(_) => dialog(
                    ui,
                    "error",
                    hyper_gpu_support::ui_text::operations::save_only::unsaved::TITLE,
                    hyper_gpu_support::ui_text::operations::save_only::unsaved::BODY,
                ),
                Err(error) => failure_dialog(
                    ui,
                    hyper_gpu_support::ui_text::operations::save_only::blocked::TITLE,
                    &error,
                    "The protected recovery record remains; no GPU operation was repeated.",
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
    Err("GPU operations require Windows".into())
}

#[cfg(not(windows))]
pub(super) fn save_only_live(
    _: &AppWindow,
    _: &SharedState,
    _: &Arc<AtomicBool>,
) -> Result<(), String> {
    Err("Protected publication requires Windows".into())
}
