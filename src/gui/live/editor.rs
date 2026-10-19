//! Selection, allocation editing and draft validation callbacks.
use super::{
    SharedState,
    presentation::{draft_target, pending_summary, present, resource_controls},
    state_access::with_state,
};
use crate::gui::{AppWindow, dialog, model};
use slint::{ComponentHandle, Model};

pub(super) fn register(ui: &AppWindow, state: &SharedState) {
    let weak = ui.as_weak();
    let selection_state = state.clone();
    ui.on_select_vm(move |id| {
        if let Some(ui) = weak.upgrade() {
            if ui.get_presenting() {
                return;
            }
            with_state(&ui, &selection_state, |state| {
                if ui.get_running() || id == ui.get_selected_id() {
                    return Ok(());
                }
                if ui.get_dirty() {
                    state.pending_selection = Some(id.to_string());
                    dialog(
                        &ui,
                        "switch",
                        hyper_gpu_support::ui_text::operations::draft::switch::title(),
                        hyper_gpu_support::ui_text::operations::draft::switch::body(),
                    );
                } else {
                    state.select_vm(Some(id.to_string()))?;
                    present(&ui, state, true);
                }
                Ok(())
            });
        }
    });
    let weak = ui.as_weak();
    let edit_state = state.clone();
    ui.on_edited(move || {
        if let Some(ui) = weak.upgrade() {
            if ui.get_presenting() {
                return;
            }
            with_state(&ui, &edit_state, |state| {
                if let Some(index) = state.fixed_gpu_index()
                    && ui.get_gpu_index() != index
                {
                    ui.set_gpu_index(index);

                    return Ok(());
                }
                state.draft_gpu = state
                    .view
                    .inventory
                    .as_ref()
                    .and_then(|i| {
                        usize::try_from(ui.get_gpu_index())
                            .ok()
                            .and_then(|index| i.discovery.gpus.get(index))
                    })
                    .map(|g| g.interface.clone());
                resource_controls(&ui, state);
                let baseline = state.target(ui.get_selected_id().as_str())?;
                let raw_values = ui
                    .get_values()
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>();
                let changed = state.draft_changed(
                    &baseline,
                    ui.get_gpu_enabled(),
                    state.draft_gpu.as_deref(),
                    &raw_values,
                );
                ui.set_dirty(changed);
                ui.set_pending(
                    if changed {
                        pending_summary(ui.get_gpu_enabled(), Some(baseline.enabled), &raw_values)
                    } else {
                        hyper_gpu_support::ui_text::dashboard::draft::absent().to_owned()
                    }
                    .into(),
                );
                if !changed {
                    state.view.draft = None;
                    state.draft_gpu = None;
                }
                match draft_target(&ui, state) {
                    Ok(target) => {
                        if changed {
                            state.view.draft = Some(target);
                        }
                        ui.set_draft_valid(true);
                        ui.set_validation("".into());
                    }
                    Err(error) => {
                        // A marker retains the VM even for invalid text; raw edits stay in the UI.
                        if changed {
                            state.view.draft = Some(baseline);
                        }
                        ui.set_draft_valid(false);
                        ui.set_validation(
                            hyper_gpu_support::reporting::operator_error(
                                hyper_gpu_support::ui_text::operations::validation::stage(),
                                &error,
                            )
                            .into(),
                        );
                    }
                }
                Ok(())
            });
        }
    });
    let weak = ui.as_weak();
    let memory_state = state.clone();
    ui.on_memory_edited(move |position| {
        let Some(ui) = weak.upgrade() else {
            return;
        };
        if ui.get_presenting() || ui.get_running() || !ui.get_dialog_kind().is_empty() {
            return;
        }
        let values = ui
            .get_values()
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>();
        let selection = memory_state
            .lock()
            .map_err(|_| {
                hyper_gpu_support::ui_text::common::state::presentation_unavailable().to_owned()
            })
            .and_then(|state| {
                state.memory_values(
                    ui.get_selected_id().as_str(),
                    ui.get_gpu_index(),
                    &values,
                    position,
                )
            });
        match selection {
            Ok(values) => {
                // The shared presentation helper changes only VRAM. The normal
                // draft callback revalidates all retained compatibility intent.
                ui.set_values(model(values.into_iter().map(Into::into).collect()));
                // The state guard above is released before invoking this callback.
                ui.invoke_edited();
            }
            Err(error) => {
                // This is application-owned numeric validation text, with no
                // adapter output. Keep the selected draft/position unchanged.
                ui.set_memory_hint(error.into());
            }
        }
    });
    ui.on_memory_text_acceptable(crate::gui::memory::acceptable_input);
    let validation_weak = ui.as_weak();
    ui.on_memory_text_valid(move |text| {
        validation_weak
            .upgrade()
            .is_some_and(|ui| crate::gui::memory::valid_input(&text, &ui.get_memory_capacity()))
    });
    let numeric_weak = ui.as_weak();
    let numeric_state = state.clone();
    ui.on_memory_text_edited(move |text| {
        let Some(ui) = numeric_weak.upgrade() else {
            return;
        };
        if ui.get_presenting()
            || ui.get_updating()
            || ui.get_running()
            || !ui.get_dialog_kind().is_empty()
        {
            return;
        }
        let values = ui
            .get_values()
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>();
        let selection = numeric_state
            .lock()
            .map_err(|_| {
                hyper_gpu_support::ui_text::common::state::presentation_unavailable().to_owned()
            })
            .and_then(|state| {
                state.memory_text_values(
                    ui.get_selected_id().as_str(),
                    ui.get_gpu_index(),
                    &values,
                    &text,
                )
            });
        match selection {
            Ok(values) => {
                ui.set_values(model(values.into_iter().map(Into::into).collect()));
                ui.invoke_edited();
            }
            Err(error) => ui.set_memory_hint(error.into()),
        }
    });
}
