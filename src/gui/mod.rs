//! Approved Slint presentation over the protected live backend.
mod actions;
mod diagnostics;
mod host_configuration;
mod live;
mod live_state;
mod memory;
mod wording;
use slint::{Model, ModelRc, VecModel};
use std::rc::Rc;
slint::include_modules!();

fn model<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    Rc::new(VecModel::from(rows)).into()
}
fn dialog(ui: &AppWindow, kind: &str, title: &str, body: &str) {
    let previous_stages: Vec<_> = ui
        .get_stages()
        .iter()
        .map(|stage| stage.to_string())
        .collect();
    ui.set_stages(model(Vec::new()));
    ui.set_dialog_details(if kind == "error" {
        diagnostics::failure_details(title, body, &previous_stages).into()
    } else {
        "".into()
    });
    ui.set_dialog_kind(kind.into());
    ui.set_dialog_title(title.into());
    ui.set_dialog_body(if kind == "error" {
        hyper_gpu_support::reporting::operator_error(title, body).into()
    } else {
        body.into()
    });
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
