//! Display language: follows Windows at startup and switches from Settings without a restart.
//! Like the other Settings preferences, the choice is not saved and resets on launch.
use super::{SharedState, presentation::present};
use crate::gui::{AppWindow, UiText, apply_ui_text, model};
use hyper_gpu_support::ui_text;
use slint::ComponentHandle;

/// Picker entry 0 follows Windows; entry `i + 1` is `ui_text::LOCALES[i]`.
const FOLLOW_WINDOWS: i32 = 0;

/// Compiled language index for a picker entry.
fn locale_for(entry: i32) -> usize {
    match usize::try_from(entry) {
        Ok(entry) if entry > 0 && entry <= ui_text::LOCALES.len() => entry - 1,
        _ => ui_text::best_match(&ui_text::system_languages()),
    }
}

/// Make a language active and refresh every catalogue-backed value in the window.
fn activate(ui: &AppWindow, entry: i32) {
    ui_text::set_current(locale_for(entry));
    apply_ui_text(&ui.global::<UiText>());
    // The first option is itself translated; native language names are not.
    let mut options: Vec<slint::SharedString> =
        vec![ui_text::settings::language::follow_windows().into()];
    options.extend(ui_text::LOCALES.iter().map(|l| l.name.into()));
    ui.set_language_options(model(options));
    ui.set_language_index(entry);
}

/// Select the Windows display language and register the Settings picker.
pub(super) fn register(ui: &AppWindow, state: &SharedState) {
    activate(ui, FOLLOW_WINDOWS);
    let weak = ui.as_weak();
    let state = state.clone();
    ui.on_language_selected(move |entry| {
        let Some(ui) = weak.upgrade() else { return };
        activate(&ui, entry);
        // Rust-composed text (rows, summaries, notices) is rebuilt from current state.
        if let Ok(state) = state.lock() {
            present(&ui, &state, false);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_entries_map_to_compiled_languages() {
        assert_eq!(locale_for(1), 0);
        // Out-of-range entries follow Windows, which always resolves to a compiled language.
        assert!(locale_for(FOLLOW_WINDOWS) < ui_text::LOCALES.len());
        assert!(locale_for(-1) < ui_text::LOCALES.len());
        let past_end = i32::try_from(ui_text::LOCALES.len() + 1).unwrap();
        assert!(locale_for(past_end) < ui_text::LOCALES.len());
    }
}
