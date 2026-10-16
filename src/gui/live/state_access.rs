//! State locking and failure presentation shared by event-loop completions.
use super::{SharedState, presentation::present_actions};
use crate::gui::{AppWindow, dialog, live_state::State};

pub(super) fn with_state(
    ui: &AppWindow,
    state: &SharedState,
    operation: impl FnOnce(&mut State) -> Result<(), String>,
) -> bool {
    let result = state
        .lock()
        .map_err(|_| "Live presentation state is unavailable; restart the application.".to_owned())
        .and_then(|mut state| {
            let result = operation(&mut state);
            present_actions(ui, &state);
            result
        });
    if let Err(error) = result {
        ui.set_execution_ready(false);
        ui.set_eligible(false);
        dialog(
            ui,
            "error",
            hyper_gpu_support::ui_text::operations::action::UNAVAILABLE_TITLE,
            &error,
        );
        return false;
    }
    true
}

/// A failed completion read must not replace the operation's original failure.
#[cfg(windows)]
pub(super) fn with_completion_state<T>(
    ui: &AppWindow,
    state: &SharedState,
    outcome: &Result<T, String>,
    operation: impl FnOnce(&mut State) -> Result<(), String>,
) -> bool {
    let completed = with_state(ui, state, operation);
    if !completed && let Err(primary) = outcome {
        let (body, details) = crate::gui::diagnostics::completion_failure(
            ui.get_dialog_body().as_str(),
            ui.get_dialog_details().as_str(),
            primary,
        );
        ui.set_dialog_body(body.into());
        ui.set_dialog_details(details.into());
    }
    completed
}
