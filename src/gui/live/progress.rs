//! Ordered worker progress and in-place transfer counter updates on the UI thread.
#[cfg(windows)]
use super::progress_state::{Event, Identity, State, Status};
#[cfg(windows)]
use crate::gui::AppWindow;
#[cfg(any(windows, test))]
use crate::gui::model;
#[cfg(any(windows, test))]
use slint::Model;

#[cfg(windows)]
pub(super) fn progress_to_ui(
    weak: slint::Weak<AppWindow>,
) -> impl FnMut(hyper_gpu_support::worker::Progress) {
    let state = std::sync::Arc::new(std::sync::Mutex::new(State::default()));
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
            let stage = match progress.stage.as_str() {
                "Persist operation recovery" => "Saving progress",
                "Prepare current signed payload" => "Preparing GPU driver files",
                "Compatibility settings" => "Updating GPU compatibility settings",
                "Provider allocation" => "Applying your GPU resource settings",
                "Graceful guest shutdown" => "Shutting down the VM safely",
                "Start guest" => "Starting the VM",
                "Publish checked VM configuration" => "Saving the checked VM settings",
                "Save verified configuration only" => "Retrying the bound configuration save",
                _ => progress.stage.as_str(),
            };
            let status = match progress.status {
                hyper_gpu_support::worker::StageStatus::Running => "In progress",
                hyper_gpu_support::worker::StageStatus::Done => "Complete",
                hyper_gpu_support::worker::StageStatus::Failed => "Failed",
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
                ui.set_current_stage(row.clone());
                ui.set_transfer_fraction(event.fraction.unwrap_or(-1.0));
                let mut steps = ui.get_progress_steps();
                if event.copy_complete {
                    steps = write_row(
                        steps,
                        state.step(&Identity::Transfer),
                        "Copying files · Complete".into(),
                    );
                }
                ui.set_progress_steps(write_row(steps, state.step(&event.identity), row));
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
