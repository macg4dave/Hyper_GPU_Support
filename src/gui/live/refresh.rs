//! Inventory publication, startup progress and VM-bound result readback.
use super::{
    SharedState,
    background::background_refresh,
    presentation::{self, draft_target, observed_row, present, update_model},
    review::review,
    source::{Source, protected_worker_available},
    state_access::with_state,
};
use crate::gui::{
    AppWindow, GpuRow, dialog, model,
    operation::{Kind, Operation},
    result_notice::ResultNotice,
};
use hyper_gpu_support::model::{Resource, Target};
use slint::ComponentHandle;
use std::sync::{Arc, atomic::AtomicBool};

/// Finish presentation only; failure leaves the splash available for continuation.
pub(in crate::gui) fn finish_startup(ui: &AppWindow, result: Result<(), String>) {
    match result {
        Ok(()) => ui.set_startup_loading(false),
        Err(error) => {
            let (summary, details) = crate::gui::diagnostics::inventory_failure(&error);
            ui.set_startup_error(summary.clone().into());
            ui.set_inventory_error(summary.clone().into());
            ui.set_notice(summary.into());
            ui.set_inventory_details(details.into());
            ui.set_dialog_kind("".into());
        }
    }
}

pub(super) fn refresh(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
    automatic: bool,
) -> Result<(), String> {
    let notice = if !automatic
        && matches!(
            ui.get_dialog_kind().as_str(),
            "status-error" | "recovery-blocked"
        ) {
        let state = state.lock().map_err(|_| "Presentation state unavailable")?;
        if let Some(notice) = &state.result_notice {
            if !notice.matches_selection(ui.get_selected_id().as_str()) {
                return Err(
                    "The result belongs to another VM. Close it before refreshing the selected VM."
                        .into(),
                );
            }
            Some(notice.clone())
        } else {
            Some(capture_result(ui))
        }
    } else {
        None
    };
    refresh_with_result(ui, state, busy, source, automatic, notice, None)
}

// Keep the result visible while independently updating inventory. Continuation
// buttons open new reviews; this never submits a subsequent effect.
fn capture_result(ui: &AppWindow) -> ResultNotice {
    ResultNotice {
        vm_id: ui.get_selected_id().to_string(),
        kind: ui.get_dialog_kind().to_string(),
        title: ui.get_dialog_title().to_string(),
        body: ui.get_dialog_body().to_string(),
        details: ui.get_dialog_details().to_string(),
    }
}

pub(super) fn refresh_after_result(ui: &AppWindow, state: &SharedState, busy: &Arc<AtomicBool>) {
    let notice = capture_result(ui);
    if !with_state(ui, state, |state| {
        state.view.needs_readback = true;
        state.result_notice = Some(notice.clone());
        present(ui, state, false);
        Ok(())
    }) {
        return;
    }
    if let Err(error) =
        refresh_with_result(ui, state, busy, &Source, false, Some(notice.clone()), None)
    {
        dialog(
            ui,
            "status-error",
            &notice.title,
            &format!(
                "{}\n\nVM status update could not start: {error}\nRetry the status update before another review.",
                notice.body
            ),
        );
        ui.set_dialog_details(notice.details.into());
    }
}

/// The selected completion bundle was gathered before foreground release. Publish
/// it without immediately repeating all reads; the next poll handles other VMs.
pub(super) fn finish_result_observation(ui: &AppWindow, state: &SharedState) {
    let notice = capture_result(ui);
    with_state(ui, state, |state| {
        if let Some(inventory) = &state.view.inventory {
            publish_inventory_models(ui, inventory, state.view.needs_readback);
        }
        ui.set_execution_ready(!state.view.needs_readback && state.worker_available);
        let gpu = state
            .view
            .draft
            .as_ref()
            .map(|target| target.gpu_interface.clone())
            .or_else(|| state.draft_gpu.clone())
            .or_else(|| {
                state
                    .target(ui.get_selected_id().as_str())
                    .ok()
                    .map(|target| target.gpu_interface)
            });
        ui.set_presenting(true);
        ui.set_gpu_index(
            state
                .view
                .inventory
                .as_ref()
                .and_then(|inventory| {
                    inventory
                        .discovery
                        .gpus
                        .iter()
                        .position(|current| Some(&current.interface) == gpu.as_ref())
                })
                .map_or(-1, |index| index as i32),
        );
        ui.set_presenting(false);
        present(ui, state, false);
        if state.view.needs_readback {
            state.result_notice = Some(notice.clone());
            dialog(
                ui,
                "status-error",
                &notice.title,
                &notice.unavailable_body(
                    "Completion readback is incomplete. Refresh before another operation.",
                ),
            );
            ui.set_dialog_details(notice.details.clone().into());
        } else {
            state.result_notice = None;
        }
        Ok(())
    });
}

pub(super) fn refresh_with_result(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
    automatic: bool,
    notice: Option<ResultNotice>,
    review_after_enrollment: Option<Target>,
) -> Result<(), String> {
    let poll = {
        let state = state.lock().map_err(|_| "Presentation state unavailable")?;
        if automatic {
            // First-install observations remain usable until an explicit refresh.
            if !state.can_poll() {
                return Ok(());
            }
            let Some(ticket) = state.polling.begin() else {
                return Ok(());
            };
            Some(ticket)
        } else {
            state.polling.invalidate();
            None
        }
    };
    let previous = state
        .lock()
        .map_err(|_| "Presentation state unavailable")?
        .view
        .inventory
        .clone();
    let state = state.clone();
    let source = source.clone();
    let review_source = source.clone();
    let review_busy = busy.clone();
    let startup = ui.get_startup_loading();
    let startup_ui = ui.as_weak();
    background_refresh(
        ui,
        &state.clone(),
        busy,
        poll,
        if automatic || notice.is_some() {
            Operation::new(
                Kind::StatusRefresh,
                hyper_gpu_support::ui_text::dashboard::inventory::UPDATING,
            )
        } else {
            Operation::new(
                Kind::Read,
                hyper_gpu_support::ui_text::operations::inventory::REFRESH_TITLE,
            )
        },
        move || {
            // Publish the current read before it blocks, including initial discovery.
            // Progress counts completed reads; it never implies recovery effects.
            let report = |completed: u8, stage: &'static str| {
                if startup {
                    let progress = f32::from(completed) / 6.0;
                    let _ = startup_ui.upgrade_in_event_loop(move |ui| {
                        ui.set_startup_progress(progress);
                        ui.set_startup_stage(stage.into());
                    });
                }
            };
            use hyper_gpu_support::ui_text::dashboard::startup as text;
            report(0, text::INVENTORY);
            let mode = if automatic {
                hyper_gpu_support::observation::Mode::Poll
            } else {
                hyper_gpu_support::observation::Mode::Full
            };
            if let Some(bundle) = source.observe(mode, None)? {
                let inventory = bundle.inventory_with_display(previous.as_ref())?;
                report(6, text::WORKER);
                return Ok((
                    inventory,
                    None,
                    Some(bundle.committed),
                    bundle.recovery,
                    bundle.import,
                    bundle.enrollment,
                    bundle.worker_available,
                    Some(bundle.provenance),
                ));
            }
            // First-install discovery remains explicit and retains its existing
            // permission/continuation path; automatic polls never enter it.
            if automatic {
                return Err("Installed polling requires a protected observation".into());
            }
            let (inventory, warning) = source.discover()?;
            report(1, text::CONFIGURATION);
            let committed = source.committed();
            report(2, text::RECOVERY);
            let recovery = source.recovery()?;
            report(3, text::IMPORT_RECOVERY);
            let import_recovery = source.import_recovery()?;
            report(4, text::ENROLLMENT_RECOVERY);
            let enrollment_recovery = source.enrollment_recovery()?;
            report(5, text::WORKER);
            let worker_available = protected_worker_available();
            report(6, text::WORKER);
            Ok((
                inventory,
                warning,
                committed,
                recovery,
                import_recovery,
                enrollment_recovery,
                worker_available,
                None,
            ))
        },
        move |ui, result| {
            // Preserve the backend failure before with_state/dialog formatting.
            let startup_failure = result.as_ref().err().cloned();
            let observed = result.is_ok();
            let result_notice = notice;
            let published = with_state(ui, &state, |state| {
                match result {
                    Ok((
                        inventory,
                        warning,
                        committed,
                        recovery,
                        import_recovery,
                        enrollment_recovery,
                        worker_available,
                        provenance,
                    )) => {
                        state.observation = provenance;
                        state.worker_available = worker_available;
                        ui.set_execution_ready(false);
                        // A conflicting draft blocks that editor, not fresh VM/GPU
                        // observations or readable intent for other VMs.
                        let configuration_warning =
                            committed.and_then(|committed| state.load_committed(committed).err());
                        publish_inventory_models(ui, &inventory, false);
                        state.refresh(inventory);
                        let recovery_warning = state
                            .sync_recovery(recovery, import_recovery, enrollment_recovery)
                            .err();
                        ui.set_execution_ready(recovery_warning.is_none() && worker_available);
                        let reset = !ui.get_dirty();
                        // Bind the draft GPU by identity across provider enumeration changes.
                        if !reset {
                            ui.set_gpu_index(
                                state
                                    .view
                                    .inventory
                                    .as_ref()
                                    .and_then(|i| {
                                        i.discovery.gpus.iter().position(|g| {
                                            Some(&g.interface) == state.draft_gpu.as_ref()
                                        })
                                    })
                                    .map_or(-1, |i| i as i32),
                            );
                        }
                        present(ui, state, reset);
                        if ui.get_dirty() {
                            let validation = draft_target(ui, state);
                            ui.set_draft_valid(validation.is_ok());
                            ui.set_validation(validation.err().unwrap_or_default().into());
                        }
                        ui.set_system_summary(
                            hyper_gpu_support::ui_text::operations::inventory::SUMMARY.into(),
                        );
                        let issues = &state
                            .view
                            .inventory
                            .as_ref()
                            .ok_or("inventory unavailable")?
                            .discovery
                            .issues;
                        let mut notice = if issues.is_empty() {
                            warning.unwrap_or_default()
                        } else {
                            format!(
                                "Partial inventory: {}. Other successful observations remain available.",
                                issues
                                    .iter()
                                    .map(|issue| issue.text())
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            )
                        };
                        if let Some(warning) = &configuration_warning {
                            notice.push_str(&format!(" {warning}"));
                        }
                        if let Some(warning) = &recovery_warning {
                            notice.push_str(&format!(" {warning}"));
                        }
                        if !state.committed.errors.is_empty() {
                            notice.push_str(
                                hyper_gpu_support::ui_text::operations::inventory::SAVED_UNREADABLE,
                            );
                        }
                        if let Some(summary) =
                            presentation::recovery_summary(state, ui.get_selected_id().as_str())
                        {
                            notice.push_str(&format!(" {summary}"));
                        }
                        ui.set_inventory_details(notice.clone().into());
                        if !issues.is_empty()
                            && configuration_warning.is_none()
                            && recovery_warning.is_none()
                            && state.committed.errors.is_empty()
                        {
                            notice =
                                hyper_gpu_support::ui_text::operations::inventory::DETAILS_PROMPT
                                    .into();
                        }
                        ui.set_notice(notice.into());
                        if let Some(notice) = &result_notice {
                            if notice.matches_selection(ui.get_selected_id().as_str()) {
                                dialog(ui, &notice.kind, &notice.title, &notice.body);
                                ui.set_dialog_details(notice.details.clone().into());
                            }
                            state.result_notice = None;
                        }
                        Ok(())
                    }
                    Err(error) => {
                        ui.set_execution_ready(false);
                        let (summary, details) = crate::gui::diagnostics::inventory_failure(&error);
                        state.view.needs_readback = true;
                        if let Some(inventory) = state.view.inventory.as_ref() {
                            ui.set_vms(model(
                                inventory
                                    .discovery
                                    .vms
                                    .iter()
                                    .map(|vm| observed_row(vm, inventory, true))
                                    .collect(),
                            ));
                        }
                        present(ui, state, false);
                        ui.set_draft_valid(false);
                        ui.set_validation(
                            hyper_gpu_support::ui_text::operations::inventory::DRAFT_RETAINED
                                .into(),
                        );
                        ui.set_inventory_details(details.into());
                        ui.set_inventory_error(summary.clone().into());
                        ui.set_notice(summary.clone().into());
                        ui.set_system_summary(summary.clone().into());
                        if let Some(notice) = &result_notice {
                            state.result_notice = Some(notice.clone());
                            dialog(
                                ui,
                                "status-error",
                                &notice.title,
                                &notice.unavailable_body(&summary),
                            );
                            ui.set_dialog_details(notice.details.clone().into());
                            Ok(())
                        } else {
                            // The inline error and diagnostics own this failure.
                            // Do not create a second generic action error.
                            Ok(())
                        }
                    }
                }
            });
            if startup {
                finish_startup(
                    ui,
                    if let Some(error) = startup_failure {
                        Err(error)
                    } else if published {
                        Ok(())
                    } else {
                        Err(ui.get_dialog_body().to_string())
                    },
                );
            }
            if published
                && observed
                && let Some(expected) = review_after_enrollment
            {
                let continuation = state
                    .lock()
                    .map_err(|_| "Presentation state unavailable".to_owned())
                    .and_then(|state| {
                        let current = draft_target(ui, &state)?;
                        state.enrollment_continuation(&expected, &current)?;
                        if !ui.get_execution_ready() {
                            return Err(
                                "GPU management is unavailable. Refresh before reviewing setup."
                                    .into(),
                            );
                        }
                        Ok(())
                    })
                    .and_then(|()| review(ui, &state, &review_busy, &review_source));
                if let Err(error) = continuation {
                    dialog(
                        ui,
                        "error",
                        hyper_gpu_support::ui_text::operations::plan::UNAVAILABLE_TITLE,
                        &error,
                    );
                }
            }
        },
    )
}

fn publish_inventory_models(
    ui: &AppWindow,
    inventory: &hyper_gpu_support::gui_model::Inventory,
    stale: bool,
) {
    if !stale {
        ui.set_inventory_error("".into());
        ui.set_startup_error("".into());
    }
    ui.set_presenting(true);
    let rows = inventory
        .discovery
        .vms
        .iter()
        .map(|vm| observed_row(vm, inventory, stale))
        .collect();
    ui.set_vms(update_model(ui.get_vms(), rows));
    ui.set_gpu_information(update_model(ui.get_gpu_information(), inventory.discovery.gpus.iter().map(|gpu| GpuRow {
    name: gpu.name.clone().into(),
    description: format!("Interface: {}\nDriver: {}\n{}\nPreparation adapter: {}\nAttachment does not prove guest graphics health; sharing is not qualified.", gpu.interface, gpu.driver_version, Resource::ALL.into_iter().map(|resource| gpu.resource_text(resource)).collect::<Vec<_>>().join("\n"), if gpu.preparation_supported { "available" } else { hyper_gpu_support::ui_text::system::PREPARATION_UNSUPPORTED }).into(),
                    }).collect()));
    ui.set_gpu_options(update_model(
        ui.get_gpu_options(),
        inventory
            .discovery
            .gpus
            .iter()
            .map(|gpu| gpu.name.clone().into())
            .collect(),
    ));
    ui.set_presenting(false);
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::Model;

    #[test]
    fn refreshed_models_replace_vm_status_and_gpu_information_and_clear_failed_startup() {
        slint::platform::set_platform(Box::new(crate::gui::catalogue_tests::Headless)).unwrap();
        let ui = AppWindow::new().unwrap();
        let mut inventory: hyper_gpu_support::gui_model::Inventory = serde_json::from_value(
            serde_json::json!({
                "managed": {}, "enrolled": [],
                "vms": [{"vm_id": "12345678-1234-1234-1234-123456789abc", "name": "VM",
                    "power": "Off", "generation": 2, "gpus": [], "vram": null,
                    "settings": {"low_mmio": 0, "high_mmio": 0, "cache_types": false, "automatic_checkpoints": true}}],
                "gpus": [{"interface": "gpu", "name": "GPU", "vendor": 4318, "device": 1,
                    "driver_version": "1", "preparation_supported": true}]
            }),
        ).unwrap();
        publish_inventory_models(&ui, &inventory, false);
        assert!(ui.get_vms().row_data(0).unwrap().power_stopped);
        let first = ui.get_gpu_information().row_data(0).unwrap().description;
        finish_startup(&ui, Err("failed 0x80070005".into()));
        assert!(!ui.get_inventory_error().is_empty());
        inventory.discovery.vms[0].power = hyper_gpu_support::model::Power::Running;
        inventory.discovery.vms[0].name = "Renamed VM".into();
        inventory.discovery.gpus[0].driver_version = "2".into();
        publish_inventory_models(&ui, &inventory, false);
        finish_startup(&ui, Ok(()));
        assert!(ui.get_inventory_error().is_empty());
        assert!(ui.get_startup_error().is_empty());
        assert!(!ui.get_startup_loading());
        let vm = ui.get_vms().row_data(0).unwrap();
        assert!(vm.power_running);
        assert_eq!(vm.name, "Renamed VM");
        assert_ne!(
            ui.get_gpu_information().row_data(0).unwrap().description,
            first
        );
        inventory.discovery.vms.clear();
        inventory.discovery.gpus.clear();
        publish_inventory_models(&ui, &inventory, false);
        assert_eq!(ui.get_vms().row_count(), 0);
        assert_eq!(ui.get_gpu_information().row_count(), 0);
        assert_eq!(ui.get_gpu_options().row_count(), 0);
    }
}
