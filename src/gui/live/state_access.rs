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
        hyper_gpu_support::ui_text::operations::action::unavailable_title(),
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
        .map_err(|_| hyper_gpu_support::ui_text::common::state::restart_required().to_owned())
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
        hyper_gpu_support::ui_text::dialogs::install::failure_title(),
        operation,
    );
    if !completed && let Err(primary) = outcome {
        let (body, details) = crate::gui::diagnostics::completion_failure(
            ui.get_dialog_body().as_str(),
            ui.get_dialog_details().as_str(),
            primary,
        );
        ui.set_dialog_title(hyper_gpu_support::ui_text::dialogs::install::failure_title().into());
        ui.set_dialog_body(body.into());
        ui.set_dialog_details(details.into());
    }
    completed
}

/// Merge one bound terminal result and its independent readback before presenting it.
#[cfg(windows)]
pub(super) fn complete_operation(
    ui: &AppWindow,
    state: &SharedState,
    target: Option<&hyper_gpu_support::model::Target>,
    outcome: &Result<hyper_gpu_support::worker::Outcome, String>,
    readback: Result<hyper_gpu_support::observation::Bundle, String>,
) -> Option<bool> {
    use slint::Model;
    let stages: Vec<_> = ui.get_stages().iter().collect();
    let reportable = ui.get_stage_reportable();
    let mut published = false;
    if !with_completion_state(ui, state, outcome, |state| {
        let status = if let (Some(target), Ok(outcome)) = (target, outcome)
            && (outcome.operation.is_some() || outcome.saved)
        {
            let status = state.finish_apply(target, &outcome.operation_id, outcome.saved, readback);
            published = state.publication_confirmed(target);
            status
        } else {
            state.finish_readback(readback)
        };
        ui.set_dirty(state.view.draft.is_some());
        super::presentation::present(ui, state, published && !ui.get_dirty());
        status
    }) {
        return None;
    }
    ui.set_stages(crate::gui::model(stages));
    ui.set_stage_reportable(reportable);
    Some(published)
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
            hyper_gpu_support::ui_text::dialogs::install::failure_title()
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
