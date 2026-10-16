//! Foreground admission, background execution and event-loop completion delivery.
use super::polling;
#[cfg(windows)]
use super::{SharedState, progress, refresh::refresh_after_result, source::Source};
#[cfg(windows)]
use crate::gui::live_state::OperationReadback;
use crate::gui::{
    AppWindow, dialog,
    operation::{Kind, Operation},
    wording,
};
use slint::ComponentHandle;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

// Only the event loop touches Slint models.

pub(super) fn background<T: Send + 'static>(
    ui: &AppWindow,
    busy: &Arc<AtomicBool>,
    operation: Operation<'_>,
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
    complete: impl FnOnce(&AppWindow, Result<T, String>) + Send + 'static,
) -> Result<(), String> {
    background_for_vm(ui, busy, operation, None, false, task, complete)
}

/// Automatic reads leave foreground admission and visible UI state untouched.
pub(super) fn background_refresh<T: Send + 'static>(
    ui: &AppWindow,
    busy: &Arc<AtomicBool>,
    poll: Option<polling::Ticket>,
    operation: Operation<'_>,
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
    complete: impl FnOnce(&AppWindow, Result<T, String>) + Send + 'static,
) -> Result<(), String> {
    let Some(ticket) = poll else {
        return background(ui, busy, operation, task, complete);
    };
    let weak = ui.as_weak();
    let foreground_busy = busy.clone();
    std::thread::Builder::new()
        .name("gui-status-poll".into())
        .spawn(move || {
            let result = run_background_task(task);
            let _ = weak.upgrade_in_event_loop(move |ui| {
                if ticket.can_publish(
                    foreground_busy.load(Ordering::Acquire),
                    !ui.get_dialog_kind().is_empty() || ui.get_running(),
                ) {
                    complete(&ui, result);
                }
                // Ticket drop releases polling admission even for discarded results.
            });
        })
        .map_err(|error| format!("Cannot start status poll: {error}"))?;
    Ok(())
}

fn run_background_task<T>(task: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(task)).unwrap_or_else(|_| {
        Err("The backend task ended without a terminal outcome. Refresh and inspect protected recovery before retrying.".into())
    })
}

/// Gather protected files while still busy, without blocking Slint completion.
#[cfg(windows)]
#[allow(
    clippy::too_many_arguments,
    reason = "Keep operation state, progress context, task and completion explicit at this orchestration boundary"
)]
pub(super) fn background_operation<T: Send + 'static>(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    operation: Operation<'_>,
    vm_name: Option<&str>,
    guest_access: bool,
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
    complete: impl FnOnce(&AppWindow, Result<T, String>, Result<OperationReadback, String>)
    + Send
    + 'static,
) -> Result<(), String> {
    let finished_state = state.clone();
    let refresh_busy = busy.clone();
    let readback_ui = ui.as_weak();
    background_for_vm(
        ui,
        busy,
        operation,
        vm_name,
        guest_access,
        move || {
            let outcome = run_background_task(task);
            let _ = readback_ui.upgrade_in_event_loop(|ui| {
                ui.set_current_stage(hyper_gpu_support::ui_text::dialogs::install::READBACK.into());
                ui.set_transfer_fraction(-1.0);
            });
            Ok((outcome, Source.operation_readback()))
        },
        move |ui, result| {
            match result {
                Ok((outcome, readback)) => complete(ui, outcome, readback),
                Err(error) => complete(ui, Err(error.clone()), Err(error)),
            }
            refresh_after_result(ui, &finished_state, &refresh_busy);
        },
    )
}

pub(super) fn background_for_vm<T: Send + 'static>(
    ui: &AppWindow,
    busy: &Arc<AtomicBool>,
    operation: Operation<'_>,
    vm_name: Option<&str>,
    guest_access: bool,
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
    complete: impl FnOnce(&AppWindow, Result<T, String>) + Send + 'static,
) -> Result<(), String> {
    if busy.swap(true, Ordering::AcqRel) {
        return Err("Another GUI operation is still running.".into());
    }
    ui.set_updating(true);
    let silent =
        operation.kind.silent() || (ui.get_startup_loading() && operation.kind == Kind::Read);
    let preparing = operation.kind.review_preparation();
    ui.set_review_preparing(preparing);
    if !silent {
        ui.set_running(true);
    }
    if !silent {
        ui.set_operation_dialog_active(operation.kind.operation_dialog());
        if operation.kind.operation_dialog() {
            if operation.kind != Kind::Apply {
                ui.set_install_vm_name(vm_name.map_or_else(|| ui.get_vm_name(), Into::into));
                ui.set_install_steps(crate::gui::model(Vec::new()));
                ui.set_install_facts(crate::gui::model(Vec::new()));
                ui.set_install_transfer("".into());
                ui.set_install_guest_work(false);
            }
            ui.set_install_is_apply(operation.kind == Kind::Apply);
            ui.set_install_action_title(operation.title.into());
        }
        let guest_message = wording::guest_progress(
            operation.kind,
            vm_name.unwrap_or(hyper_gpu_support::ui_text::operations::selection::FALLBACK),
        );
        dialog(
            ui,
            if operation.kind.operation_dialog() {
                "install-progress"
            } else {
                "progress"
            },
            operation.title,
            if guest_access {
                &guest_message
            } else if operation.kind == Kind::Apply {
                hyper_gpu_support::ui_text::operations::apply::HOST_PROGRESS
            } else if operation.kind == Kind::Reconcile {
                hyper_gpu_support::ui_text::operations::recovery::HOST_PROGRESS
            } else if preparing {
                hyper_gpu_support::ui_text::operations::plan::PROGRESS_BODY
            } else if operation.kind == Kind::SaveOnly {
                hyper_gpu_support::ui_text::operations::save_only::PROGRESS_BODY
            } else {
                hyper_gpu_support::ui_text::operations::background::READING
            },
        );
        if operation.kind.operation_dialog() {
            ui.set_current_stage(hyper_gpu_support::ui_text::dialogs::install::STARTING.into());
        }
    }
    #[cfg(windows)]
    if operation.kind == Kind::Reconcile {
        progress::recovery_preparation(ui, guest_access);
    }
    let weak = ui.as_weak();
    let worker_busy = busy.clone();
    if let Err(error) = std::thread::Builder::new()
        .name("gui-backend-read".into())
        .spawn(move || {
            let result = run_background_task(task);
            let completion_busy = worker_busy.clone();
            if weak
                .upgrade_in_event_loop(move |ui| {
                    completion_busy.store(false, Ordering::Release);
                    ui.set_updating(false);
                    // Cancel abandons this read's result, including failures. It
                    // never terminates a protected effect or publishes a late plan.
                    if preparing && !ui.get_review_preparing() {
                        return;
                    }
                    ui.set_review_preparing(false);
                    if !silent {
                        ui.set_running(false);
                        ui.set_dialog_kind("".into());
                    }
                    complete(&ui, result);
                })
                .is_err()
            {
                worker_busy.store(false, Ordering::Release);
            }
        })
    {
        busy.store(false, Ordering::Release);
        ui.set_review_preparing(false);
        ui.set_updating(false);
        if !silent {
            ui.set_running(false);
            ui.set_dialog_kind("".into());
        }
        return Err(format!("Cannot start backend task: {error}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_dialog_covers_apply_setup_verification_recovery_save_and_removal() {
        slint::platform::set_platform(Box::new(crate::gui::catalogue_tests::Headless)).unwrap();
        for (kind, guest_access) in [
            (Kind::Apply, false),
            (Kind::Apply, true),
            (Kind::Verify, true),
            (Kind::Reconcile, false),
            (Kind::SaveOnly, false),
            (Kind::Setup, false),
            (Kind::Remove, false),
            (Kind::Forget, false),
        ] {
            let ui = AppWindow::new().unwrap();
            let busy = Arc::new(AtomicBool::new(false));
            // Only an isolated no-op background task; no worker or inventory calls.
            background_for_vm(
                &ui,
                &busy,
                Operation::new(kind, "Operation"),
                Some("VM2"),
                guest_access,
                || Ok(()),
                |_, _| {},
            )
            .unwrap();
            assert_eq!(ui.get_dialog_kind(), "install-progress");
            assert_eq!(
                ui.get_current_stage(),
                if kind == Kind::Reconcile {
                    hyper_gpu_support::ui_text::progress::stage::RECOVERY_PREPARATION
                } else {
                    hyper_gpu_support::ui_text::dialogs::install::STARTING
                }
            );
            ui.set_dialog_details("00:01   Reported step".into());
            crate::gui::dialog(&ui, "info", "Confirmed outcome", "Confirmed result");
            assert_eq!(ui.get_dialog_kind(), "operation-complete");
            assert_eq!(ui.get_dialog_title(), "Confirmed outcome");
            assert_eq!(ui.get_dialog_details(), "00:01   Reported step");
            ui.set_operation_dialog_active(true);
            crate::gui::dialog(&ui, "error", "Unconfirmed outcome", "Failure");
            assert_eq!(ui.get_dialog_kind(), "operation-error");
        }
        for kind in [
            Kind::Plan,
            Kind::VerifyPlan,
            Kind::Read,
            Kind::StatusRefresh,
            Kind::Credentials,
        ] {
            assert!(!kind.operation_dialog());
        }
    }

    #[test]
    fn background_failure_or_panic_never_becomes_a_completed_operation() {
        assert_eq!(run_background_task(|| Ok(7)).unwrap(), 7);
        assert_eq!(
            run_background_task::<()>(|| Err("read failed".into())).unwrap_err(),
            "read failed"
        );
        let error = run_background_task::<()>(|| panic!("fixture panic payload")).unwrap_err();
        assert!(error.contains("without a terminal outcome"));
        assert!(!error.contains("fixture panic payload"));
    }
}
