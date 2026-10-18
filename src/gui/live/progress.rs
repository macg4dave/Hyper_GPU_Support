//! Ordered worker progress and in-place transfer counter updates on the UI thread.
#[cfg(windows)]
use super::progress_state::{Event, Identity, State, Status};
#[cfg(any(windows, test))]
use crate::gui::AppWindow;
#[cfg(any(windows, test))]
use crate::gui::model;
#[cfg(any(windows, test))]
use slint::Model;

/// Local preparation feedback precedes worker launch/UAC, not a worker receipt.
#[cfg(any(windows, test))]
pub(super) fn permission_preparation(ui: &AppWindow) {
    let stage = hyper_gpu_support::ui_text::progress::stage::PERMISSION_PREPARATION;
    ui.set_current_stage(stage.into());
    ui.set_dialog_details(stage.into());
    ui.set_stages(model(vec![stage.into()]));
    ui.set_progress_steps(model(vec![stage.into()]));
    ui.set_stage_reportable(model(vec![false]));
    ui.set_transfer_fraction(-1.0);
}

/// Publish before spawning the credential/worker task, including its first repaint.
#[cfg(any(windows, test))]
pub(super) fn recovery_preparation(ui: &AppWindow, guest_access: bool) {
    let stage = if guest_access {
        hyper_gpu_support::ui_text::progress::stage::RECOVERY_CREDENTIALS
    } else {
        hyper_gpu_support::ui_text::progress::stage::RECOVERY_PREPARATION
    };
    ui.set_current_stage(stage.into());
    ui.set_progress_steps(model(vec![stage.into()]));
    ui.set_stages(model(vec![stage.into()]));
    // Local waiting feedback is not an authenticated worker event.
    ui.set_stage_reportable(model(vec![false]));
    ui.set_dialog_details(stage.into());
    ui.set_transfer_fraction(-1.0);
}

#[cfg(windows)]
pub(super) fn progress_to_ui(
    weak: slint::Weak<AppWindow>,
) -> impl FnMut(hyper_gpu_support::worker::Progress) {
    progress_with_install(weak, Some(super::install_dialog::Progress::default()))
}

#[cfg(windows)]
pub(super) fn progress_with_install(
    weak: slint::Weak<AppWindow>,
    installation: Option<super::install_dialog::Progress>,
) -> impl FnMut(hyper_gpu_support::worker::Progress) {
    let state = std::sync::Arc::new(std::sync::Mutex::new((State::default(), installation)));
    progress_callback(weak, state)
}

#[cfg(windows)]
type ProgressState =
    std::sync::Arc<std::sync::Mutex<(State, Option<super::install_dialog::Progress>)>>;

#[cfg(windows)]
fn progress_callback(
    weak: slint::Weak<AppWindow>,
    state: ProgressState,
) -> impl FnMut(hyper_gpu_support::worker::Progress) {
    move |progress| {
        let event = Event::from_worker(
            &progress.stage,
            match progress.status {
                hyper_gpu_support::worker::StageStatus::Running => Status::Running,
                hyper_gpu_support::worker::StageStatus::Done => Status::Done,
                hyper_gpu_support::worker::StageStatus::Failed => Status::Failed,
            },
        );
        let state = state.clone();
        let _ = weak.upgrade_in_event_loop(move |ui| {
            let mut state = state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let (state, installation) = &mut *state;
            let stage = match progress.stage.as_str() {
                "Check recorded recovery outcome" => {
                    hyper_gpu_support::ui_text::progress::stage::RECOVERY_OUTCOME
                }
                "Prepare protected worker connection" => {
                    hyper_gpu_support::ui_text::progress::stage::WORKER_CONNECTION
                }
                "Check protected worker admission" => {
                    hyper_gpu_support::ui_text::progress::stage::WORKER_ADMISSION
                }
                "Read current VM state" => hyper_gpu_support::ui_text::progress::stage::READ_VM,
                "Read selected GPU and driver version" => {
                    hyper_gpu_support::ui_text::progress::stage::READ_GPU
                }
                "Authenticate current signed driver files" => {
                    hyper_gpu_support::ui_text::progress::stage::AUTHENTICATE
                }
                "Persist operation recovery" => {
                    hyper_gpu_support::ui_text::progress::stage::PERSIST
                }
                "Prepare current signed payload" => {
                    hyper_gpu_support::ui_text::progress::stage::PREPARE
                }
                "Compatibility settings" => {
                    hyper_gpu_support::ui_text::progress::stage::COMPATIBILITY
                }
                "Provider allocation" => hyper_gpu_support::ui_text::progress::stage::ALLOCATION,
                "Graceful guest shutdown" => hyper_gpu_support::ui_text::progress::stage::SHUTDOWN,
                "Start guest" => hyper_gpu_support::ui_text::progress::stage::START,
                "Publish checked VM configuration" => {
                    hyper_gpu_support::ui_text::progress::stage::PUBLISH
                }
                "Save verified configuration only" => {
                    hyper_gpu_support::ui_text::progress::stage::SAVE_ONLY
                }
                _ => progress.stage.as_str(),
            };
            let status = match progress.status {
                hyper_gpu_support::worker::StageStatus::Running => "In progress",
                hyper_gpu_support::worker::StageStatus::Done => {
                    hyper_gpu_support::ui_text::progress::status::COMPLETE
                }
                hyper_gpu_support::worker::StageStatus::Failed => {
                    hyper_gpu_support::ui_text::progress::status::FAILED
                }
            };
            let row: slint::SharedString = format!("{stage} · {status}").into();
            let slot = state.record(&event);
            let stages = write_row(ui.get_stages(), slot, row.clone());
            ui.set_stage_reportable(model(state.reportable().collect()));
            ui.set_dialog_details(
                format!(
                    "Reported events (not proof of final state):\n{}",
                    stages
                        .iter()
                        .map(|row| row.to_string())
                        .collect::<Vec<_>>()
                        .join("\n")
                )
                .into(),
            );
            ui.set_stages(stages);
            if !event.persistence {
                let mut steps = ui.get_progress_steps();
                if event.copy_complete {
                    steps = write_row(
                        steps,
                        state.step(&Identity::Transfer),
                        hyper_gpu_support::ui_text::progress::copy::COMPLETE.into(),
                    );
                }
                let steps = write_row(steps, state.step(&event.identity), row);
                let current = state.active_step().and_then(|slot| steps.row_data(slot));
                ui.set_current_stage(current.unwrap_or_else(|| {
                    hyper_gpu_support::ui_text::progress::status::WAITING_RESULT.into()
                }));
                ui.set_transfer_fraction(if event.status == Status::Running {
                    event.fraction.unwrap_or(-1.0)
                } else {
                    -1.0
                });
                ui.set_progress_steps(steps);
            }
            if ui.get_dialog_kind() == "install-progress"
                && let Some(installation) = installation.as_mut()
            {
                installation.record(&event, &progress.stage, stage);
                installation.present(&ui);
                if event.identity == Identity::Transfer {
                    ui.set_current_stage(
                        hyper_gpu_support::ui_text::dialogs::install::PREPARING_FILES.into(),
                    );
                    ui.set_install_transfer(progress.stage.replacen("; ", "\n", 1).into());
                } else {
                    if event.status == Status::Running {
                        ui.set_install_transfer("".into());
                    }
                    let current = ui.get_current_stage();
                    ui.set_current_stage(
                        current
                            .rsplit_once(" · ")
                            .map_or(current.as_str(), |(label, _)| label)
                            .into(),
                    );
                }
            }
        });
    }
}

#[cfg(any(windows, test))]
fn write_row(
    rows: slint::ModelRc<slint::SharedString>,
    slot: usize,
    row: slint::SharedString,
) -> slint::ModelRc<slint::SharedString> {
    if let Some(vector) = rows
        .as_any()
        .downcast_ref::<slint::VecModel<slint::SharedString>>()
    {
        if slot < vector.row_count() {
            vector.set_row_data(slot, row);
        } else {
            vector.push(row);
        }
        rows
    } else {
        let mut values: Vec<_> = rows.iter().collect();
        if slot < values.len() {
            values[slot] = row;
        } else {
            values.push(row);
        }
        model(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn permission_feedback_is_available_before_any_uac_or_worker_event() {
        slint::platform::set_platform(Box::new(crate::gui::catalogue_tests::Headless)).unwrap();
        let ui = AppWindow::new().unwrap();
        permission_preparation(&ui);
        assert!(!ui.get_dialog_details().is_empty());
        assert!(ui.get_dialog_details().contains("UAC"));
        assert_eq!(ui.get_stage_reportable().row_data(0), Some(false));
        assert_eq!(ui.get_stages().row_count(), 1);
    }
    #[test]
    fn recovery_has_named_feedback_and_details_before_any_worker_event() {
        slint::platform::set_platform(Box::new(crate::gui::catalogue_tests::Headless)).unwrap();
        let ui = AppWindow::new().unwrap();
        for guest_access in [true, false] {
            crate::gui::dialog(&ui, "progress", "Recovery", "Progress appears below");
            recovery_preparation(&ui, guest_access);
            let expected = if guest_access {
                hyper_gpu_support::ui_text::progress::stage::RECOVERY_CREDENTIALS
            } else {
                hyper_gpu_support::ui_text::progress::stage::RECOVERY_PREPARATION
            };
            assert_eq!(ui.get_current_stage(), expected);
            assert_eq!(ui.get_progress_steps().row_data(0).unwrap(), expected);
            assert_eq!(ui.get_dialog_details(), expected);
            assert_eq!(ui.get_stage_reportable().row_data(0), Some(false));
            assert_eq!(ui.get_transfer_fraction(), -1.0);
            // A cancelled/failed credential request replaces waiting feedback.
            crate::gui::dialog(
                &ui,
                "error",
                "Recovery blocked",
                "Credential request failed",
            );
            assert!(ui.get_current_stage().is_empty());
            assert_eq!(ui.get_progress_steps().row_count(), 0);
            assert!(!ui.get_dialog_details().contains(expected));
        }
    }

    #[test]
    fn edited_rows_use_identity_slots_and_retain_the_model() {
        let first: slint::SharedString = ['λ', '\n', '{'].iter().collect::<String>().into();
        let second: slint::SharedString = ['}', '\n', 'λ'].iter().collect::<String>().into();
        let rows = model(vec![]);
        let original = rows.clone();
        let rows = write_row(rows, 0, first);
        let rows = write_row(rows, 0, second.clone());
        assert_eq!(original.row_count(), 1);
        assert_eq!(original.row_data(0), Some(second.clone()));
        let rows = write_row(rows, 1, second);
        assert_eq!(rows.row_count(), 2);
        assert_eq!(original.row_count(), 2);
    }
}
