//! Approved Slint presentation over the protected live backend.
#[cfg(test)]
mod accessibility_tests;
mod actions;
#[cfg(test)]
mod catalogue_tests;
mod diagnostics;
mod dialog_presentation;
mod host_configuration;
#[cfg(test)]
mod layout_tests;
mod live;
mod live_state;
mod memory;
mod operation;
mod os_metadata;
mod result_notice;
mod selection;
mod wording;
use slint::{Model, ModelRc, VecModel};
use std::rc::Rc;
slint::include_modules!();

fn model<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    Rc::new(VecModel::from(rows)).into()
}
fn dialog(ui: &AppWindow, kind: &str, title: &str, body: &str) {
    let operation = ui.get_operation_dialog_active();
    let retained_log = ui.get_dialog_details();
    let kind = if operation && kind == "info" {
        "operation-complete"
    } else if operation && kind == "error" {
        "operation-error"
    } else {
        kind
    };
    if kind != "install-progress" {
        ui.set_operation_dialog_active(false);
    }
    ui.set_recovery_heading("".into());
    ui.set_recovery_vm_id("".into());
    let reportable = ui.get_stage_reportable();
    let previous_stages: Vec<_> = ui
        .get_stages()
        .iter()
        .enumerate()
        .filter(|(index, _)| reportable.row_data(*index).unwrap_or(false))
        .map(|(_, stage)| stage.to_string())
        .collect();
    ui.set_stages(model(Vec::new()));
    ui.set_stage_reportable(model(Vec::new()));
    ui.set_progress_steps(model(Vec::new()));
    ui.set_current_stage("".into());
    ui.set_transfer_fraction(-1.0);
    ui.set_dialog_details(if kind == "error" || kind == "operation-error" {
        diagnostics::failure_details(title, body, &previous_stages).into()
    } else if operation && kind != "install-progress" {
        retained_log
    } else {
        "".into()
    });
    ui.set_dialog_kind(kind.into());
    dialog_presentation::opened(ui, kind, operation);
    ui.set_dialog_title(title.into());
    ui.set_dialog_body(if kind == "error" || kind == "operation-error" {
        hyper_gpu_support::reporting::operator_error(title, body).into()
    } else {
        body.into()
    });
}

// Keep the typed backend envelope intact until both diagnostic views are built.
fn failure_dialog(ui: &AppWindow, title: &str, error: &str, context: &str) {
    if hyper_gpu_support::reporting::installation_preflight_blocked(error) {
        let title = "GPU support installation blocked";
        dialog(ui, "error", title, error);
        ui.set_dialog_body(diagnostics::failure_summary(title, error,
            "This attempt stopped before driver installation or VM power changes. Your draft is retained.").into());
        return;
    }
    dialog(ui, "error", title, error);
    ui.set_dialog_body(diagnostics::failure_summary(title, error, context).into());
}
pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
    live::run(AppWindow::new()?, live::Source)
}

fn set_details(ui: &AppWindow, details: &str) {
    let details = diagnostics::additional_details(ui.get_dialog_body().as_str(), details);
    ui.set_dialog_details(if details.is_empty() { "".into() } else {
        format!("Source: facts captured for this review/record. Fresh protected checks are required before any effects.\n\n{details}").into()
    });
}
