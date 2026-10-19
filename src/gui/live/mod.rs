//! Live GUI lifecycle over shared state and protected operations.
//!
//! Callback registration, reads and operation flows live in focused sibling modules.
//! Shared mutable state remains State; no module adds a second authority or effect layer.
mod actions;
mod apply;
mod background;
mod credentials;
mod editor;
#[cfg(windows)]
mod enrollment;
#[cfg(any(windows, test))]
mod install_dialog;
#[cfg(any(windows, test))]
mod installation;
mod language;
#[cfg(windows)]
mod partial_removal;
pub(super) mod polling;
pub(super) mod presentation;
#[cfg(windows)]
mod profile_removal;
mod progress;
mod progress_state;
mod recovery;
mod refresh;
mod review;
mod source;
mod state_access;
#[cfg(test)]
mod tests;
mod verification;

use crate::gui::{AppWindow, dialog, live_state::State, model};
pub(super) use refresh::finish_startup;
use refresh::refresh;
use slint::ComponentHandle;
pub(super) use source::Source;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

type SharedState = Arc<Mutex<State>>;

pub(super) fn run(ui: AppWindow, source: Source) -> Result<(), Box<dyn std::error::Error>> {
    ui.set_vms(model(Vec::new()));
    ui.set_selected_id("".into());
    ui.set_gpu_information(model(Vec::new()));
    ui.set_gpu_options(model(Vec::new()));
    ui.set_values(model(vec!["".into(); 12]));
    ui.set_eligible(false);
    let busy = Arc::new(AtomicBool::new(false));
    let state = Arc::new(Mutex::new(State::default()));
    if let Ok(mut state) = state.lock() {
        state.view.selected = crate::gui::selection::load();
    }
    language::register(&ui, &state);
    ui.set_system_summary(
        hyper_gpu_support::ui_text::operations::inventory::initial_notice().into(),
    );
    editor::register(&ui, &state);
    actions::register(&ui, &state, &busy, &source);
    let weak = ui.as_weak();
    ui.window().on_close_requested(move || {
        if let Some(ui) = weak.upgrade() {
            if ui.get_running() {
                return slint::CloseRequestResponse::KeepWindowShown;
            }
            if ui.get_dirty() {
                dialog(
                    &ui,
                    "exit",
                    hyper_gpu_support::ui_text::operations::draft::exit::title(),
                    hyper_gpu_support::ui_text::operations::draft::exit::body(),
                );
                return slint::CloseRequestResponse::KeepWindowShown;
            }
        }
        slint::CloseRequestResponse::HideWindow
    });
    // Show the splash before scheduling discovery; all provider/file work stays off-thread.
    ui.show()?;
    if let Err(error) = refresh(&ui, &state, &busy, &source, false) {
        finish_startup(&ui, Err(error));
    }
    // Only the installed read path polls. First-install elevation remains explicit.
    let status_timer = slint::Timer::default();
    {
        let weak = ui.as_weak();
        let state = state.clone();
        status_timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_secs(15),
            move || {
                if let Some(ui) = weak.upgrade()
                    && !ui.get_startup_loading()
                    && !busy.load(Ordering::Acquire)
                    && ui.get_dialog_kind().is_empty()
                    && let Err(error) = refresh(&ui, &state, &busy, &source, true)
                {
                    ui.set_notice(
                        hyper_gpu_support::ui_text::dashboard::refresh::poll_unavailable(
                            &error.to_string(),
                        )
                        .into(),
                    );
                }
            },
        );
    }
    ui.run()?;
    if let Ok(state) = state.lock() {
        crate::gui::selection::save(state.view.selected.as_deref())?;
    }
    Ok(())
}
