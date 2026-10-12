//! Ordered worker progress and in-place transfer counter updates on the UI thread.
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
    move |progress| {
        let _ = weak.upgrade_in_event_loop(move |ui| {
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
            let stages = append_progress(ui.get_stages(), row.clone());
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
            if progress.stage != "Persist operation recovery" {
                ui.set_current_stage(row.clone());
                ui.set_transfer_fraction(transfer_fraction(&progress.stage).unwrap_or(-1.0));
                let mut steps = ui.get_progress_steps();
                if progress.stage == "Checking and installing copied driver files" {
                    steps = update_steps(steps, "Copying files ? Complete".into());
                }
                ui.set_progress_steps(update_steps(steps, row));
            }
        });
    }
}

/// Only the worker's fixed acknowledged-copy format supplies a determinate value.
#[cfg(any(windows, test))]
fn transfer_fraction(stage: &str) -> Option<f32> {
    if !stage.starts_with(hyper_gpu_support::reporting::TRANSFER_PROGRESS_PREFIX) {
        return None;
    }
    let percent = stage
        .split_once('(')?
        .1
        .split_once("%)")?
        .0
        .parse::<u8>()
        .ok()?;
    (percent <= 100).then_some(f32::from(percent) / 100.0)
}

#[cfg(any(windows, test))]
fn update_steps(
    steps: slint::ModelRc<slint::SharedString>,
    row: slint::SharedString,
) -> slint::ModelRc<slint::SharedString> {
    let key = |text: &str| {
        if text.starts_with(hyper_gpu_support::reporting::TRANSFER_PROGRESS_PREFIX) {
            "Copying files".to_owned()
        } else {
            text.rsplit_once(" · ")
                .map_or(text, |(stage, _)| stage)
                .to_owned()
        }
    };
    let mut rows: Vec<_> = steps.iter().collect();
    if let Some(existing) = rows.iter_mut().find(|previous| key(previous) == key(&row)) {
        *existing = row;
    } else {
        rows.push(row);
    }
    super::presentation::update_model(steps, rows)
}

#[cfg(any(windows, test))]
fn append_progress(
    stages: slint::ModelRc<slint::SharedString>,
    row: slint::SharedString,
) -> slint::ModelRc<slint::SharedString> {
    if let Some(vector) = stages
        .as_any()
        .downcast_ref::<slint::VecModel<slint::SharedString>>()
    {
        if let Some(index) = vector.row_count().checked_sub(1)
            && vector
                .row_data(index)
                .is_some_and(|previous| transfer_update(&previous, &row))
        {
            vector.set_row_data(index, row);
        } else {
            vector.push(row);
        }
        stages
    } else {
        let mut rows: Vec<_> = stages.iter().collect();
        if let Some(previous) = rows.last_mut()
            && transfer_update(previous, &row)
        {
            *previous = row;
        } else {
            rows.push(row);
        }
        model(rows)
    }
}

#[cfg(any(windows, test))]
fn transfer_update(previous: &str, current: &str) -> bool {
    // The worker formats numeric acknowledgements using this fixed prefix.
    // Never replace a failure, completion, or a different lifecycle stage.
    [previous, current].iter().all(|row| {
        row.starts_with(hyper_gpu_support::reporting::TRANSFER_PROGRESS_PREFIX)
            && row.ends_with("In progress")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copying_percentage_does_not_certify_preparation_and_phase_changes_clear_it() {
        assert_eq!(
            transfer_fraction("Copying files: 8 MB of 10 MB (80%); verification pending"),
            Some(0.8)
        );
        assert_eq!(
            transfer_fraction("Copying files: progress unavailable; verification pending"),
            None
        );
        assert_eq!(
            transfer_fraction("Checking and installing copied driver files"),
            None
        );
        assert_eq!(transfer_fraction("Copying files: (101%)"), None);
        let steps = update_steps(model(vec![]), "Starting the VM · In progress".into());
        let steps = update_steps(steps, "Starting the VM · Complete".into());
        assert_eq!(steps.row_count(), 1);
        assert_eq!(steps.row_data(0).unwrap(), "Starting the VM · Complete");
        let steps = update_steps(steps, "Graphics check · Failed".into());
        assert_eq!(steps.row_count(), 2);
    }

    #[test]
    fn progress_retains_each_event_in_order_without_rebuilding_the_existing_model() {
        let rows = model::<slint::SharedString>(vec![]);
        let original = rows.clone();
        let rows = append_progress(rows, "Repeated stage: in progress".into());
        let rows = append_progress(rows, "Repeated stage: in progress".into());
        let rows = append_progress(rows, "Stage failed".into());
        assert_eq!(original.row_count(), 3);
        assert_eq!(
            rows.iter().map(|row| row.to_string()).collect::<Vec<_>>(),
            [
                "Repeated stage: in progress",
                "Repeated stage: in progress",
                "Stage failed",
            ]
        );
    }

    #[test]
    fn transfer_counters_update_in_place_and_preserve_stage_boundaries() {
        let rows = model::<slint::SharedString>(vec![]);
        let original = rows.clone();
        let rows = append_progress(rows, "Preparing files - In progress".into());
        let rows = append_progress(
            rows,
            "Copying files: 1.0 MB of 10.0 MB (10%); 0 of 2 files; verification pending - In progress".into(),
        );
        let rows = append_progress(
            rows,
            "Copying files: 8.0 MB of 10.0 MB (80%); 1 of 2 files; verification pending - In progress".into(),
        );
        assert_eq!(original.row_count(), 2);
        assert_eq!(
            original.row_data(1).unwrap(),
            "Copying files: 8.0 MB of 10.0 MB (80%); 1 of 2 files; verification pending - In progress"
        );
        let rows = append_progress(rows, "Copying files: interrupted - Failed".into());
        let rows = append_progress(
            rows,
            "Copying files: 9.0 MB of 10.0 MB (90%); 1 of 2 files; verification pending - In progress".into(),
        );
        assert_eq!(rows.row_count(), 4);
        assert_eq!(
            rows.row_data(2).unwrap(),
            "Copying files: interrupted - Failed"
        );
    }
}
