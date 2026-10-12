//! Approved Slint presentation over the protected live backend.
mod actions;
mod host_configuration;
mod live;
mod live_state;
mod memory;
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
        let mut details = hyper_gpu_support::reporting::technical_error(title, body);
        if !previous_stages.is_empty() {
            details.push_str("\n\nLast reported stages:\n");
            details.push_str(&previous_stages.join("\n"));
        }
        details.into()
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
