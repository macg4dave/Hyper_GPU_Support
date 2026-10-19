//! Action dispatch and dialog continuations; operation modules own their workflows.
use super::{
    SharedState,
    apply::save_only_live,
    credentials::review_credentials,
    presentation::present,
    recovery::{continue_recovery, review_records, start_recovery},
    refresh::refresh,
    review::review,
    source::Source,
    state_access::with_state,
    verification::review_verification,
};
#[cfg(windows)]
use super::{partial_removal::review_reset, profile_removal::review_forget_live};
use crate::gui::{AppWindow, dialog};
use slint::{ComponentHandle, Model};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub(super) fn register(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
) {
    let weak = ui.as_weak();
    let action_state = state.clone();
    let action_busy = busy.clone();
    let action_source = source.clone();
    ui.on_action(move |action| {
        let Some(ui) = weak.upgrade() else {
            return;
        };
        if ui.get_running() && !(action == "close" && ui.get_review_preparing()) {
            return;
        }
        if let Ok(state) = action_state.lock() {
            state.polling.invalidate();
        }
        if ui.get_dialog_kind().is_empty() {
            ui.set_focus_return_action(action.clone());
        }
        if action_busy.load(Ordering::Acquire)
            && matches!(
                action.as_str(),
                "refresh"
                    | "review"
                    | "reapply"
                    | "repair"
                    | "remove-gpu"
                    | "forget"
                    | "confirm-forget"
                    | "verify"
                    | "confirm"
                    | "confirm-enroll"
                    | "confirm-verify"
                    | "reconcile"
                    | "review-reset"
                    | "confirm-reset"
                    | "reconcile-enrollment"
                    | "save-only"
                    | "credentials"
                    | "discard-review-forget"
                    | "forget-recovery-profile"
            )
        {
            return;
        }
        if matches!(action.as_str(), "verify" | "repair" | "remove-gpu") {
            let message = action_state
                .lock()
                .map_err(|_| {
                    hyper_gpu_support::ui_text::common::state::live_presentation_unavailable()
                        .to_owned()
                })
                .map(|state| state.action_recovery_message());
            match message {
                Ok(Some(body)) => {
                    dialog(
                        &ui,
                        "recovery-needed",
                        hyper_gpu_support::ui_text::operations::recovery::review_first_title(),
                        &body,
                    );
                    return;
                }
                Err(error) => {
                    dialog(
                        &ui,
                        "error",
                        hyper_gpu_support::ui_text::operations::recovery::unreadable_title(),
                        &error,
                    );
                    return;
                }
                Ok(None) => {}
            }
            if !ui.get_execution_ready() {
                dialog(
                    &ui,
                    "info",
                    hyper_gpu_support::ui_text::operations::action::unavailable_title(),
                    hyper_gpu_support::ui_text::operations::action::setup_unavailable(),
                );
                return;
            }
            let availability = action_state
                .lock()
                .map_err(|_| {
                    hyper_gpu_support::ui_text::common::state::live_presentation_unavailable()
                        .to_owned()
                })
                .map(|state| {
                    let values = ui
                        .get_values()
                        .iter()
                        .map(|value| value.to_string())
                        .collect::<Vec<_>>();
                    let actions = state.selected_actions(ui.get_gpu_index(), &values);
                    (
                        actions.allowed(action.as_str()).unwrap_or(true),
                        actions
                            .reason(action.as_str())
                            .unwrap_or_default()
                            .to_owned(),
                    )
                });
            match availability {
                Ok((false, reason)) => {
                    dialog(
                        &ui,
                        "info",
                        hyper_gpu_support::ui_text::operations::action::unavailable_title(),
                        &reason,
                    );
                    return;
                }
                Err(error) => {
                    dialog(
                        &ui,
                        "error",
                        hyper_gpu_support::ui_text::operations::action::unreadable_title(),
                        &error,
                    );
                    return;
                }
                _ => {}
            }
        }
        let result = match action.as_str() {
            "refresh" => refresh(&ui, &action_state, &action_busy, &action_source, false),
            #[cfg(windows)]
            "discard-review-forget" if ui.get_dialog_kind() == "reset-complete" => {
                if !with_state(&ui, &action_state, |state| {
                    if state.action_recovery_message().is_some() {
                        return Err(
                            hyper_gpu_support::ui_text::dashboard::actions::recovery_first().into(),
                        );
                    }
                    state.discard();
                    present(&ui, state, true);
                    Ok(())
                }) {
                    return;
                }
                review_forget_live(&ui, &action_state, &action_busy)
            }
            #[cfg(windows)]
            "forget" => review_forget_live(&ui, &action_state, &action_busy),
            #[cfg(windows)]
            "forget-recovery-profile"
                if matches!(
                    ui.get_dialog_kind().as_str(),
                    "recovery" | "recovery-blocked"
                ) =>
            {
                let id = ui.get_recovery_vm_id().to_string();
                if id.is_empty() {
                    return;
                }
                if !with_state(&ui, &action_state, |state| {
                    state.select_vm(Some(id))?;
                    present(&ui, state, false);
                    Ok(())
                }) {
                    return;
                }
                review_forget_live(&ui, &action_state, &action_busy)
            }
            "review" => review(&ui, &action_state, &action_busy, false),
            "save-only" => save_only_live(&ui, &action_state, &action_busy),
            "verify" => review_verification(&ui, &action_state, &action_busy),
            #[cfg(windows)]
            "review-reset" => review_reset(&ui, &action_state, &action_busy),
            "reapply" => {
                // Preserve allocation edits while deliberately previewing enabled support.
                ui.set_gpu_enabled(true);
                ui.invoke_edited();
                review(&ui, &action_state, &action_busy, false)
            }
            "repair" => {
                // Reuse the enabled draft; reinstall intent belongs only to this review.
                ui.set_gpu_enabled(true);
                ui.invoke_edited();
                review(&ui, &action_state, &action_busy, true)
            }
            "remove-gpu" => {
                let removal = action_state
                    .lock()
                    .map_err(|_| {
                        hyper_gpu_support::ui_text::common::state::presentation_unavailable()
                            .to_owned()
                    })
                    .and_then(|state| {
                        state.removal_draft(
                            ui.get_selected_id().as_str(),
                            ui.get_gpu_index(),
                            &ui.get_values()
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<_>>(),
                        )
                    });
                removal.and_then(|_| {
                    ui.set_gpu_enabled(false);
                    ui.invoke_edited();
                    review(&ui, &action_state, &action_busy, false)
                })
            }
            "close" => {
                let cancelling = ui.get_review_preparing();
                if with_state(&ui, &action_state, |state| {
                    state.result_notice = None;
                    Ok(())
                }) {
                    if cancelling {
                        super::background::cancel_preparation(&ui, &action_state);
                    } else {
                        ui.set_operation_dialog_active(false);
                        ui.set_dialog_kind("".into());
                    }
                }
                Ok(())
            }
            "exit" => {
                ui.set_dirty(false);
                slint::quit_event_loop().map_err(|e| e.to_string())
            }
            "discard" | "switch" => {
                with_state(&ui, &action_state, |state| {
                    state.discard();
                    if action == "switch" {
                        let selected = state.pending_selection.take();
                        state.select_vm(selected)?;
                    }
                    present(&ui, state, true);
                    ui.set_dirty(false);
                    ui.set_dialog_kind("".into());
                    Ok(())
                });
                Ok(())
            }
            "more" => {
                if !with_state(&ui, &action_state, |_| Ok(())) {
                    return;
                }
                let body = hyper_gpu_support::ui_text::operations::action::review_prompt(
                    &(ui.get_vm_name()),
                )
                .into_string();
                dialog(
                    &ui,
                    "more",
                    hyper_gpu_support::ui_text::operations::action::menu_title(),
                    &body,
                );
                Ok(())
            }
            "credentials" => review_credentials(&ui, &action_state, &action_busy, &action_source),
            "recovery" => start_recovery(&ui, &action_state),
            "reconcile" if ui.get_dialog_kind() == "recovery" => {
                continue_recovery(&ui, &action_state, &action_busy, &action_source)
            }
            "diagnostics" => {
                with_state(&ui, &action_state, |state| {
                    review_records(&ui, state, false)
                });
                Ok(())
            }
            "shortcuts" => {
                let help = hyper_gpu_support::ui_text::operations::help::body();
                dialog(
                    &ui,
                    "info",
                    hyper_gpu_support::ui_text::operations::help::title(),
                    help,
                );
                Ok(())
            }
            _ => Err(hyper_gpu_support::ui_text::operations::action::not_connected().into()),
        };
        if let Err(error) = result {
            dialog(
                &ui,
                "error",
                hyper_gpu_support::ui_text::operations::action::unavailable_title(),
                &error,
            );
        }
    });
}
