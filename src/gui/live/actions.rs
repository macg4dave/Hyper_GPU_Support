//! Action dispatch and dialog continuations; operation modules own their workflows.
use super::{
    SharedState,
    apply::{apply_live, save_only_live},
    credentials::review_credentials,
    presentation::present,
    recovery::{reconcile_live, review_records},
    refresh::refresh,
    review::{review, review_operation},
    source::Source,
    state_access::with_state,
    verification::{review_verification, verify_live},
};
#[cfg(windows)]
use super::{
    enrollment::{enroll_live, reconcile_enrollment_live},
    partial_removal::{reset_live, review_reset},
    profile_removal::{forget_live, review_forget_live},
};
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
        let Some(ui) = weak.upgrade() else { return; };
        if ui.get_running() && !(action == "close" && ui.get_review_preparing()) { return; }
        if let Ok(state) = action_state.lock() {
            state.polling.invalidate();
        }
        if ui.get_dialog_kind().is_empty() { ui.set_focus_return_action(action.clone()); }
        if action_busy.load(Ordering::Acquire)
            && matches!(action.as_str(), "refresh" | "review" | "reapply" | "repair" | "remove-gpu" | "forget" | "confirm-forget" | "verify" | "confirm" | "confirm-enroll" | "confirm-verify" | "reconcile" | "review-reset" | "confirm-reset" | "reconcile-enrollment" | "save-only" | "credentials" | "discard-review-forget" | "forget-recovery-profile")
        {

            return;
        }
        if matches!(action.as_str(), "verify" | "repair" | "remove-gpu") {
            let message = action_state.lock()
                .map_err(|_| "Live presentation state unavailable".to_owned())
                .map(|state| state.action_recovery_message());
            match message {
                Ok(Some(body)) => {
                    dialog(&ui, "recovery-needed", hyper_gpu_support::ui_text::operations::recovery::REVIEW_FIRST_TITLE, &body);
                    return;
                }
                Err(error) => {
                    dialog(&ui, "error", hyper_gpu_support::ui_text::operations::recovery::UNREADABLE_TITLE, &error);
                    return;
                }
                Ok(None) => {}
            }
            if !ui.get_execution_ready() {
                dialog(&ui, "info", hyper_gpu_support::ui_text::operations::action::UNAVAILABLE_TITLE, hyper_gpu_support::ui_text::operations::action::SETUP_UNAVAILABLE);
                return;
            }
            let availability = action_state.lock()
                .map_err(|_| "Live presentation state unavailable".to_owned())
                .map(|state| {
                    let values = ui.get_values().iter().map(|value| value.to_string()).collect::<Vec<_>>();
                    let actions = state.selected_actions(ui.get_gpu_index(), &values);
                    (actions.allowed(action.as_str()).unwrap_or(true), actions.reason(action.as_str()).unwrap_or_default().to_owned())
                });
            match availability {
                Ok((false, reason)) => {
                    dialog(&ui, "info", hyper_gpu_support::ui_text::operations::action::UNAVAILABLE_TITLE, &reason);
                    return;
                }
                Err(error) => {
                    dialog(&ui, "error", hyper_gpu_support::ui_text::operations::action::UNREADABLE_TITLE, &error);
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
                    if state.action_recovery_message().is_some() { return Err("Review unfinished recovery first.".into()); }
                    state.discard();
                    present(&ui, state, true);
                    Ok(())
                }) { return; }
                review_forget_live(&ui, &action_state, &action_busy)
            }
            #[cfg(windows)]
            "forget" => review_forget_live(&ui, &action_state, &action_busy),
            #[cfg(windows)]
            "forget-recovery-profile" if matches!(ui.get_dialog_kind().as_str(), "recovery" | "recovery-blocked") => {
                let id = ui.get_recovery_vm_id().to_string();
                if id.is_empty() { return; }
                if !with_state(&ui, &action_state, |state| {
                    state.select_vm(Some(id))?;
                    present(&ui, state, false);
                    Ok(())
                }) { return; }
                review_forget_live(&ui, &action_state, &action_busy)
            }
            #[cfg(windows)]
            "confirm-forget" if ui.get_dialog_kind() == "forget-review" => forget_live(&ui, &action_state, &action_busy),
            "review" => review(&ui, &action_state, &action_busy, &action_source),
            #[cfg(windows)]
            "confirm-enroll" => enroll_live(&ui, &action_state, &action_busy, &action_source),
            #[cfg(windows)]
            "reconcile-enrollment" => reconcile_enrollment_live(&ui, &action_state, &action_busy, &action_source),
            "confirm" if ui.get_dialog_kind() == "review" => apply_live(&ui, &action_state, &action_busy),
            "confirm" => Err("Open a fresh review before Apply.".into()),
            "save-only" => save_only_live(&ui, &action_state, &action_busy),
            "verify" => review_verification(&ui, &action_state, &action_busy),
            "confirm-verify" => verify_live(&ui, &action_state, &action_busy),
            "reconcile" => reconcile_live(&ui, &action_state, &action_busy),
            #[cfg(windows)]
            "review-reset" => review_reset(&ui, &action_state),
            #[cfg(windows)]
            "confirm-reset" if ui.get_dialog_kind() == "reset-review" => reset_live(&ui, &action_state, &action_busy),
            "reapply" => {
                // Preserve allocation edits while deliberately previewing enabled support.
                ui.set_gpu_enabled(true);
                ui.invoke_edited();
                review(&ui, &action_state, &action_busy, &action_source)
            }
            "repair" => {
                // Reuse the enabled draft; reinstall intent belongs only to this review.
                ui.set_gpu_enabled(true);
                ui.invoke_edited();
                review_operation(&ui, &action_state, &action_busy, &action_source, true)
            }
            "remove-gpu" => {
                let removal = action_state.lock().map_err(|_| "Presentation state unavailable".to_owned())
                    .and_then(|state| state.removal_draft(
                        ui.get_selected_id().as_str(), ui.get_gpu_index(),
                        &ui.get_values().iter().map(|v| v.to_string()).collect::<Vec<_>>(),
                    ));
                removal.and_then(|_| {
                    ui.set_gpu_enabled(false);
                    ui.invoke_edited();
                    review(&ui, &action_state, &action_busy, &action_source)
                })
            }
            "close" => {
                if ui.get_review_preparing() {
                    ui.set_review_preparing(false);
                    ui.set_running(false);
                    // The background admission remains held until the abandoned read finishes.
                }
                with_state(&ui, &action_state, |state| {
                    state.cancel_review();
                    ui.set_operation_dialog_active(false);
                    ui.set_dialog_kind("".into());
                    Ok(())
                });
                Ok(())
            },
            "exit" => { ui.set_dirty(false); slint::quit_event_loop().map_err(|e| e.to_string()) },
            "discard" | "switch" => {
                with_state(&ui, &action_state, |state| {
                    state.discard();
                    if action == "switch" { let selected = state.pending_selection.take(); state.select_vm(selected)?; }
                    present(&ui, state, true); ui.set_dirty(false); ui.set_dialog_kind("".into());
                     Ok(())
                }); Ok(())
            }
            "more" => {
                if !with_state(&ui, &action_state, |_| Ok(())) { return; }
                let body = hyper_gpu_support::ui_text::operations::action::review_prompt(&(ui.get_vm_name())).into_string();
                dialog(&ui, "more", hyper_gpu_support::ui_text::operations::action::MENU_TITLE, &body); Ok(())
            },
            "credentials" => review_credentials(&ui, &action_state, &action_busy, &action_source),
            "diagnostics" | "recovery" => {
                with_state(&ui, &action_state, |state| {
                    review_records(&ui, state, action == "recovery")
                }); Ok(())
            }
            "shortcuts" => {
                let help = hyper_gpu_support::ui_text::operations::help::BODY;
                dialog(&ui, "info", hyper_gpu_support::ui_text::operations::help::TITLE, help); Ok(())
            },
            _ => Err("This action is not connected yet. Use the supported CLI; no operation was performed.".into()),
        };
        if let Err(error) = result { dialog(&ui, "error", hyper_gpu_support::ui_text::operations::action::UNAVAILABLE_TITLE, &error); }
    });
}
