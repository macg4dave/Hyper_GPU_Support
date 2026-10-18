//! State locking and failure presentation shared by event-loop completions.
use super::{SharedState, presentation::present_actions};
use crate::gui::{AppWindow, dialog, live_state::State};

pub(super) fn with_state(
    ui: &AppWindow,
    state: &SharedState,
    operation: impl FnOnce(&mut State) -> Result<(), String>,
) -> bool {
    with_state_titled(
        ui,
        state,
        hyper_gpu_support::ui_text::operations::action::UNAVAILABLE_TITLE,
        operation,
    )
}

fn with_state_titled(
    ui: &AppWindow,
    state: &SharedState,
    title: &str,
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
        dialog(ui, "error", title, &error);
        return false;
    }
    true
}

/// A failed completion read must not replace the operation's original failure.
#[cfg(any(windows, test))]
pub(super) fn with_completion_state<T>(
    ui: &AppWindow,
    state: &SharedState,
    outcome: &Result<T, String>,
    operation: impl FnOnce(&mut State) -> Result<(), String>,
) -> bool {
    let completed = with_state_titled(
        ui,
        state,
        hyper_gpu_support::ui_text::dialogs::install::FAILURE_TITLE,
        operation,
    );
    if !completed && let Err(primary) = outcome {
        let (body, details) = crate::gui::diagnostics::completion_failure(
            ui.get_dialog_body().as_str(),
            ui.get_dialog_details().as_str(),
            primary,
        );
        ui.set_dialog_title(hyper_gpu_support::ui_text::dialogs::install::FAILURE_TITLE.into());
        ui.set_dialog_body(body.into());
        ui.set_dialog_details(details.into());
    }
    completed
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper_gpu_support::reporting::{Component, ComponentFailure, FailureCause, FailureStage};

    #[test]
    fn operation_failure_survives_failed_readback_with_useful_title_and_technical_details() {
        slint::platform::set_platform(Box::new(crate::gui::catalogue_tests::Headless)).unwrap();
        let ui = AppWindow::new().unwrap();
        let state = std::sync::Arc::new(std::sync::Mutex::new(State::default()));
        let original = ComponentFailure {
            stage: Some(FailureStage::Preparation),
            component: Component::GuestTransport,
            cause: FailureCause::InvalidReply,
            code: None,
        }
        .into_error();
        ui.set_operation_dialog_active(true);
        assert!(!with_completion_state(
            &ui,
            &state,
            &Err::<(), _>(original),
            |_| { Err("Completion read unavailable".into()) }
        ));
        assert_eq!(
            ui.get_dialog_title(),
            hyper_gpu_support::ui_text::dialogs::install::FAILURE_TITLE
        );
        let body = ui.get_dialog_body();
        assert!(body.contains("incomplete or invalid reply"), "{body}");
        assert!(body.contains("Refresh") && body.contains("recovery"));
        assert!(!body.contains("Action unavailable"), "{body}");
        let details = ui.get_dialog_details();
        for fact in [
            "Preparation",
            "GuestTransport",
            "InvalidReply",
            "no terminal numeric code",
        ] {
            assert!(details.contains(fact), "Missing {fact}: {details}");
        }
        assert!(!ui.get_execution_ready());
    }
}
