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
    let stage = hyper_gpu_support::ui_text::progress::stage::permission_preparation();
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
        hyper_gpu_support::ui_text::progress::stage::recovery_credentials()
    } else {
        hyper_gpu_support::ui_text::progress::stage::recovery_preparation()
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
            present_progress(&ui, progress, event, state, installation);
        });
    }
}

#[cfg(windows)]
fn present_progress(
    ui: &AppWindow,
    progress: hyper_gpu_support::worker::Progress,
    event: Event,
    state: &mut State,
    installation: &mut Option<super::install_dialog::Progress>,
) {
    let stage = match progress.stage.as_str() {
        "Check recorded recovery outcome" => {
            hyper_gpu_support::ui_text::progress::stage::recovery_outcome()
        }
        "Prepare protected worker connection" => {
            hyper_gpu_support::ui_text::progress::stage::worker_connection()
        }
        "Check protected worker admission" => {
            hyper_gpu_support::ui_text::progress::stage::worker_admission()
        }
        "Read current VM state" => hyper_gpu_support::ui_text::progress::stage::read_vm(),
        "Read selected GPU and driver version" => {
            hyper_gpu_support::ui_text::progress::stage::read_gpu()
        }
        "Authenticate current signed driver files" => {
            hyper_gpu_support::ui_text::progress::stage::authenticate()
        }
        "Persist operation recovery" => hyper_gpu_support::ui_text::progress::stage::persist(),
        "Prepare current signed payload" => hyper_gpu_support::ui_text::progress::stage::prepare(),
        "Compatibility settings" => hyper_gpu_support::ui_text::progress::stage::compatibility(),
        "Provider allocation" => hyper_gpu_support::ui_text::progress::stage::allocation(),
        "Graceful guest shutdown" => hyper_gpu_support::ui_text::progress::stage::shutdown(),
        "Start guest" => hyper_gpu_support::ui_text::progress::stage::start(),
        "Publish checked VM configuration" => {
            hyper_gpu_support::ui_text::progress::stage::publish()
        }
        "Save verified configuration only" => {
            hyper_gpu_support::ui_text::progress::stage::save_only()
        }
        _ => progress.stage.as_str(),
    };
    let status = match progress.status {
        hyper_gpu_support::worker::StageStatus::Running => {
            hyper_gpu_support::ui_text::progress::status::in_progress()
        }
        hyper_gpu_support::worker::StageStatus::Done => {
            hyper_gpu_support::ui_text::progress::status::complete()
        }
        hyper_gpu_support::worker::StageStatus::Failed => {
            hyper_gpu_support::ui_text::progress::status::failed()
        }
    };
    let row: slint::SharedString =
        hyper_gpu_support::ui_text::progress::status::line(stage, status).into();
    let slot = state.record(&event);
    let stages = write_row(ui.get_stages(), slot, row.clone());
    ui.set_stage_reportable(model(state.reportable().collect()));
    ui.set_dialog_details(
        hyper_gpu_support::ui_text::progress::log::heading(
            &stages
                .iter()
                .map(|row| row.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
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
                hyper_gpu_support::ui_text::progress::copy::complete().into(),
            );
        }
        let steps = write_row(steps, state.step(&event.identity), row);
        let current = state.active_step().and_then(|slot| steps.row_data(slot));
        ui.set_current_stage(current.unwrap_or_else(|| {
            hyper_gpu_support::ui_text::progress::status::waiting_result().into()
        }));
        ui.set_transfer_fraction(state.fraction().unwrap_or(-1.0));
        ui.set_progress_steps(steps);
    }
    if ui.get_dialog_kind() == "install-progress"
        && let Some(installation) = installation.as_mut()
    {
        installation.record(&event, &progress.stage, stage);
        installation.present(ui);
        if event.identity == Identity::Transfer {
            ui.set_install_transfer(progress.stage.replacen("; ", "\n", 1).into());
        }
        if state.copying() {
            ui.set_current_stage(
                hyper_gpu_support::ui_text::dialogs::install::preparing_files().into(),
            );
        } else {
            ui.set_install_transfer("".into());
            let current = ui.get_current_stage();
            ui.set_current_stage(
                current
                    .rsplit_once(" · ")
                    .map_or(current.as_str(), |(label, _)| label)
                    .into(),
            );
        }
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

    #[cfg(windows)]
    #[test]
    fn setup_copy_updates_and_guest_phases_replace_stale_counters_together() {
        use hyper_gpu_support::worker::{Progress, StageStatus};
        slint::platform::set_platform(Box::new(crate::gui::catalogue_tests::Headless)).unwrap();
        let ui = AppWindow::new().unwrap();
        ui.set_dialog_kind("install-progress".into());
        let mut state = State::default();
        let mut installation = Some(super::super::install_dialog::Progress::default());
        let mut deliver = |stage: &str, status: Status| {
            let event = Event::from_worker(stage, status);
            let progress = Progress {
                stage: stage.into(),
                status: match status {
                    Status::Running => StageStatus::Running,
                    Status::Done => StageStatus::Done,
                    Status::Failed => StageStatus::Failed,
                },
            };
            present_progress(&ui, progress, event, &mut state, &mut installation);
        };
        deliver("Prepare current signed payload", Status::Running);
        for (bytes, files, fraction) in [(68, 194, 0.68), (69, 195, 0.69), (100, 273, 1.0)] {
            let stage = hyper_gpu_support::reporting::transfer_progress(bytes, 100, files, 273);
            deliver(&stage, Status::Running);
            assert_eq!(ui.get_transfer_fraction(), fraction);
            assert_eq!(ui.get_install_transfer(), stage.replacen("; ", "\n", 1));
            deliver("Read current VM state", Status::Done);
            assert_eq!(ui.get_transfer_fraction(), fraction);
            assert!(
                ui.get_install_transfer()
                    .contains(&format!("{files} of 273"))
            );
            assert_eq!(
                ui.get_current_stage(),
                hyper_gpu_support::ui_text::dialogs::install::preparing_files()
            );
        }
        for stage in [
            "File copying complete; waiting for guest checks",
            "Checking copied driver files",
            "Installing GPU driver files",
            "Checking installed driver files",
        ] {
            deliver(stage, Status::Running);
            assert!(ui.get_install_transfer().is_empty());
            assert_eq!(ui.get_transfer_fraction(), -1.0);
            assert_eq!(ui.get_current_stage(), stage);
            deliver(stage, Status::Done);
        }
        deliver("Prepare current signed payload", Status::Done);
        assert!(ui.get_install_transfer().is_empty());
    }

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
                hyper_gpu_support::ui_text::progress::stage::recovery_credentials()
            } else {
                hyper_gpu_support::ui_text::progress::stage::recovery_preparation()
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
