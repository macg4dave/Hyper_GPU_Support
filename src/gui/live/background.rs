//! Foreground admission, background execution and event-loop completion delivery.
use super::{SharedState, polling};
#[cfg(windows)]
use super::{progress, refresh::finish_result_observation, source::Source};
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

/// A foreground request continues directly after fresh preparation. Failed
/// preparation or an unavailable worker never reaches the execution callback.
#[cfg(any(windows, test))]
pub(super) fn continue_prepared(
    ui: &AppWindow,
    prepared: Result<(), String>,
    execute: impl FnOnce() -> Result<(), String>,
) {
    let result = prepared.and_then(|()| {
        if !ui.get_execution_ready() {
            return Err("The protected worker is unavailable. Refresh before trying again.".into());
        }
        execute()
    });
    if let Err(error) = result {
        dialog(
            ui,
            "error",
            hyper_gpu_support::ui_text::operations::action::UNAVAILABLE_TITLE,
            &error,
        );
    }
}

/// Validate the still-current selection/draft on the event loop before sending
/// the exact worker preparation acknowledgement. Expired delivery does nothing.
#[cfg(windows)]
pub(super) fn accept_prepared<T: Send + 'static>(
    weak: &slint::Weak<AppWindow>,
    action: impl FnOnce(&AppWindow) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    let (sent, result) = std::sync::mpsc::sync_channel(1);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    weak.upgrade_in_event_loop(move |ui| {
        if std::time::Instant::now() >= deadline {
            return;
        }
        let _ = sent.send(action(&ui));
    })
    .map_err(|e| e.to_string())?;
    result
        .recv_timeout(std::time::Duration::from_secs(30))
        .map_err(|_| "Prepared operation acknowledgement expired".to_owned())?
}

/// Abandon preparation without pretending the underlying read has stopped.
pub(super) fn cancel_preparation(ui: &AppWindow, state: &SharedState) {
    if let Ok(state) = state.lock() {
        state.polling.cancel_preparation();
    }
    ui.set_review_preparing(false);
    ui.set_operation_dialog_active(false);
    ui.set_running(true);
    dialog(
        ui,
        "cancelling",
        hyper_gpu_support::ui_text::dialogs::cancelling::TITLE,
        hyper_gpu_support::ui_text::dialogs::cancelling::BODY,
    );
}

/// Called after background admission is released. Abandoned reads never publish
/// a plan, error or continuation; their waiting dialog ends with the block.
pub(super) fn finish_cancelled_preparation(ui: &AppWindow, preparing: bool) -> bool {
    if !preparing || ui.get_review_preparing() {
        return false;
    }
    if ui.get_dialog_kind() == "cancelling" {
        ui.set_running(false);
        ui.set_dialog_kind("".into());
    }
    true
}

fn finish_cancelled_worker_preparation(ui: &AppWindow, kind: Kind, released: bool) -> bool {
    if !released
        || ui.get_dialog_kind() != "cancelling"
        || !matches!(kind, Kind::Apply | Kind::Verify)
    {
        return false;
    }
    ui.set_review_preparing(false);
    ui.set_running(false);
    ui.set_operation_dialog_active(false);
    ui.set_dialog_kind("".into());
    true
}

pub(super) fn background<T: Send + 'static>(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    operation: Operation<'_>,
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
    complete: impl FnOnce(&AppWindow, Result<T, String>) + Send + 'static,
) -> Result<(), String> {
    background_for_vm(ui, state, busy, operation, None, false, task, complete)
}

/// Automatic reads leave foreground admission and visible UI state untouched.
pub(super) fn background_refresh<T: Send + 'static>(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    poll: Option<polling::Ticket>,
    operation: Operation<'_>,
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
    complete: impl FnOnce(&AppWindow, Result<T, String>) + Send + 'static,
) -> Result<(), String> {
    let Some(ticket) = poll else {
        return background(ui, state, busy, operation, task, complete);
    };
    let weak = ui.as_weak();
    let foreground_busy = busy.clone();
    std::thread::Builder::new()
        .name("gui-status-poll".into())
        .spawn(move || {
            let result = run_background_task(|| ticket.read(task));
            ticket.backend_finished();
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
    let target = state
        .lock()
        .map_err(|_| "Presentation state unavailable")?
        .target(ui.get_selected_id().as_str())?;
    let readback_ui = ui.as_weak();
    background_for_vm(
        ui,
        state,
        busy,
        operation,
        vm_name,
        guest_access,
        move || {
            let outcome = run_background_task(task);
            let _ = readback_ui.upgrade_in_event_loop(|ui| {
                crate::gui::dialog_presentation::readback(&ui);
                ui.set_current_stage(hyper_gpu_support::ui_text::dialogs::install::READBACK.into());
                ui.set_transfer_fraction(-1.0);
            });
            Ok((outcome, Source.operation_readback(target)))
        },
        move |ui, result| {
            match result {
                Ok((outcome, readback)) => complete(ui, outcome, readback),
                Err(error) => complete(ui, Err(error.clone()), Err(error)),
            }
            finish_result_observation(ui, &finished_state);
        },
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "Keep read admission, operation identity and foreground completion explicit"
)]
pub(super) fn background_for_vm<T: Send + 'static>(
    ui: &AppWindow,
    state: &SharedState,
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
    let admission = match state
        .lock()
        .map_err(|_| "Read admission unavailable".to_owned())
        .and_then(|state| state.polling.foreground())
    {
        Ok(admission) => admission,
        Err(error) => {
            busy.store(false, Ordering::Release);
            return Err(error);
        }
    };
    ui.set_updating(true);
    let silent =
        operation.kind.silent() || (ui.get_startup_loading() && operation.kind == Kind::Read);
    let preparing = operation.kind.review_preparation();
    let readonly = preparing || matches!(operation.kind, Kind::Read | Kind::StatusRefresh);
    ui.set_review_preparing(preparing);
    if !silent {
        ui.set_running(true);
    }
    if !silent {
        ui.set_operation_dialog_active(operation.kind.operation_dialog());
        if operation.kind.operation_dialog() && operation.kind != Kind::Apply {
            ui.set_install_vm_name(vm_name.map_or_else(|| ui.get_vm_name(), Into::into));
            ui.set_install_gpu_name(hyper_gpu_support::ui_text::common::state::UNAVAILABLE.into());
            ui.set_install_vram(hyper_gpu_support::ui_text::common::state::UNAVAILABLE.into());
            if let Ok(state) = state.lock()
                && let Ok(target) = super::presentation::draft_target(ui, &state)
            {
                let gpu_name = state
                    .view
                    .inventory
                    .as_ref()
                    .and_then(|inventory| {
                        inventory
                            .discovery
                            .gpus
                            .iter()
                            .find(|gpu| gpu.interface == target.gpu_interface)
                    })
                    .map_or(target.gpu_interface.as_str(), |gpu| gpu.name.as_str());
                super::install_dialog::context(ui, gpu_name, &target);
            }
            ui.set_install_steps(crate::gui::model(Vec::new()));
            ui.set_install_facts(crate::gui::model(Vec::new()));
            ui.set_install_transfer("".into());
            ui.set_install_guest_work(false);
        }
        crate::gui::dialog_presentation::begin(ui, operation.kind);
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
            } else if operation.kind == Kind::SaveOnly {
                hyper_gpu_support::ui_text::operations::save_only::PROGRESS_BODY
            } else {
                hyper_gpu_support::ui_text::operations::background::READING
            },
        );
        if operation.kind.operation_dialog() {
            ui.set_current_stage(crate::gui::dialog_presentation::task(operation.kind).into());
        }
    }
    #[cfg(windows)]
    if operation.kind == Kind::Reconcile {
        progress::recovery_preparation(ui, guest_access);
    }
    #[cfg(windows)]
    if matches!(
        operation.kind,
        Kind::PermissionPlan | Kind::Setup | Kind::Install
    ) {
        progress::permission_preparation(ui);
    }
    let weak = ui.as_weak();
    let worker_busy = busy.clone();
    let operation_kind = operation.kind;
    if let Err(error) = std::thread::Builder::new()
        .name("gui-backend-read".into())
        .spawn(move || {
            let result = run_background_task(|| admission.run(readonly, task));
            let released = admission.backend_released();
            drop(admission);
            let completion_busy = worker_busy.clone();
            if weak
                .upgrade_in_event_loop(move |ui| {
                    if released {
                        completion_busy.store(false, Ordering::Release);
                    }
                    ui.set_updating(false);
                    if preparing && !released {
                        ui.set_review_preparing(false);
                        ui.set_running(false);
                        ui.set_execution_ready(false);
                        ui.set_operation_dialog_active(false);
                        dialog(&ui, "error", hyper_gpu_support::ui_text::operations::action::UNAVAILABLE_TITLE,
                            "The protected reader has not confirmed release. New actions remain blocked; no operation was retried.");
                        return;
                    }
                    if finish_cancelled_worker_preparation(&ui, operation_kind, released) {
                        return;
                    }
                    // Cancel abandons this read's result, including failures. It
                    // never terminates a protected effect or publishes a late plan.
                    if finish_cancelled_preparation(&ui, preparing) {
                        return;
                    }
                    ui.set_review_preparing(false);
                    if !silent {
                        ui.set_running(false);
                        ui.set_dialog_kind("".into());
                    }
                    complete(&ui, result);
                    if !released { ui.set_execution_ready(false); }
                })
                .is_err()
                && released
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
    fn cancelled_worker_preparation_waits_for_release_and_never_hides_effect_results() {
        slint::platform::set_platform(Box::new(crate::gui::catalogue_tests::Headless)).unwrap();
        let ui = AppWindow::new().unwrap();
        ui.set_running(true);
        ui.set_dialog_kind("cancelling".into());
        assert!(!finish_cancelled_worker_preparation(
            &ui,
            Kind::Apply,
            false
        ));
        assert!(ui.get_running());
        assert!(finish_cancelled_worker_preparation(&ui, Kind::Apply, true));
        assert!(!ui.get_running());
        ui.set_dialog_kind("install-complete".into());
        assert!(!finish_cancelled_worker_preparation(&ui, Kind::Apply, true));
        assert_eq!(ui.get_dialog_kind(), "install-complete");
    }

    #[test]
    fn direct_continuation_requires_successful_preparation_and_an_available_worker() {
        slint::platform::set_platform(Box::new(crate::gui::catalogue_tests::Headless)).unwrap();
        let ui = AppWindow::new().unwrap();
        let started = std::cell::Cell::new(0);
        for (prepared, ready) in [
            (Err("stale plan".into()), true),
            (Err("cancelled preparation".into()), true),
            (Ok(()), false),
        ] {
            ui.set_execution_ready(ready);
            continue_prepared(&ui, prepared, || {
                started.set(started.get() + 1);
                Ok(())
            });
            assert_eq!(started.get(), 0);
            assert_eq!(ui.get_dialog_kind(), "error");
        }
        ui.set_execution_ready(true);
        continue_prepared(&ui, Ok(()), || {
            started.set(started.get() + 1);
            dialog(
                &ui,
                "install-progress",
                "Applying selected action",
                "Progress",
            );
            Ok(())
        });
        assert_eq!(started.get(), 1);
        assert_eq!(ui.get_dialog_kind(), "install-progress");
        // Failure to start is shown, without retrying the callback.
        continue_prepared(&ui, Ok(()), || {
            started.set(started.get() + 1);
            Err("start failed".into())
        });
        assert_eq!(started.get(), 2);
        assert_eq!(ui.get_dialog_kind(), "error");
    }

    #[test]
    fn operation_dialog_covers_apply_setup_verification_recovery_save_and_removal() {
        slint::platform::set_platform(Box::new(crate::gui::catalogue_tests::Headless)).unwrap();
        for (kind, guest_access) in [
            (Kind::PermissionPlan, false),
            (Kind::PermissionRecovery, false),
            (Kind::Apply, false),
            (Kind::Apply, true),
            (Kind::Verify, true),
            (Kind::Reconcile, false),
            (Kind::SaveOnly, false),
            (Kind::Setup, false),
            (Kind::Install, false),
            (Kind::Remove, false),
            (Kind::Forget, false),
        ] {
            let ui = AppWindow::new().unwrap();
            let busy = Arc::new(AtomicBool::new(false));
            let state = Arc::new(std::sync::Mutex::new(
                crate::gui::live_state::State::default(),
            ));
            // Only an isolated no-op background task; no worker or inventory calls.
            background_for_vm(
                &ui,
                &state,
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
                } else if matches!(kind, Kind::PermissionPlan | Kind::Setup | Kind::Install) {
                    hyper_gpu_support::ui_text::progress::stage::PERMISSION_PREPARATION
                } else {
                    crate::gui::dialog_presentation::task(kind)
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
        for kind in [Kind::Read, Kind::StatusRefresh, Kind::Credentials] {
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
