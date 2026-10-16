//! Confirmed GPU changes and separate configuration-only publication retries.
use super::SharedState;
#[cfg(windows)]
use super::{
    background::background_operation,
    credentials::credential_request_cancelled,
    installation,
    presentation::{draft_target, present},
    progress::progress_to_ui,
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
) -> Result<(), String> {
    if !ui.get_execution_ready() {
        return Err("The reviewed protected worker is not available.".into());
    }
    let (approved, revision) = {
        let state = state.lock().map_err(|_| "Presentation state unavailable")?;
        let reviewed = state.reviewed_plan(&draft_target(ui, &state)?)?;
        (reviewed.plan, reviewed.revision)
    };
    let selected = approved.desired.clone();
    let finished_state = state.clone();
    let progress = progress_to_ui(ui.as_weak());
    let vm_name = approved.observed.name.clone();
    let result_vm_name = vm_name.clone();
    let setup_required = approved
        .preview
        .actions
        .contains(&hyper_gpu_support::workflow::PreviewAction::Verify);
    background_operation(
        ui,
        state,
        busy,
        Operation::new(
            Kind::Apply,
            hyper_gpu_support::ui_text::operations::apply::PROGRESS_TITLE,
        ),
        Some(&vm_name),
        setup_required,
        move || {
            let credential =
                installation::credentials(approved.preview.credentials_required, || {
                    hyper_gpu_support::credentials::read(&approved.desired.vm_id)?.map_or_else(
                        || {
                            hyper_gpu_support::credentials::prompt_named(
                                &approved.desired.vm_id,
                                &approved.observed.name,
                                windows::Win32::Foundation::HWND::default(),
                            )
                        },
                        Ok,
                    )
                })?;
            let shutdown = approved.preview.guest_downtime;
            hyper_gpu_support::worker::apply(approved, revision, shutdown, credential, progress)
        },
        move |ui, result, readback| match result {
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
                    dialog(
                        ui,
                        "info",
                        if setup_required {
                            hyper_gpu_support::ui_text::operations::setup::completed::TITLE
                        } else {
                            hyper_gpu_support::ui_text::operations::apply::completed::TITLE
                        },
                        &if setup_required {
                            format!(
                                "{result_vm_name}: GPU support {}. VRAM request: {}. Hardware rendering passed, graceful final shutdown was confirmed, and your settings were saved. The VM is off; start it through Hyper-V when ready.",
                                if selected.enabled {
                                    hyper_gpu_support::ui_text::common::support::ENABLED_SENTENCE
                                } else {
                                    hyper_gpu_support::ui_text::common::support::DISABLED_SENTENCE
                                },
                                selected.vram.as_ref().map_or_else(
                                    || hyper_gpu_support::ui_text::common::allocation::PROVIDER_DEFAULT.to_owned(),
                                    |v| crate::gui::memory::format_gb(v.optimal)
                                )
                            )
                        } else {
                            hyper_gpu_support::ui_text::operations::apply::completed::body(
                                &(result_vm_name),
                            )
                            .into_string()
                        },
                    );
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
                        state.require_authenticated_review = true;
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
        },
    )
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
