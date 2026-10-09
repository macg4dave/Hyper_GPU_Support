//! Audited live reads; effects remain behind unfinished worker/save contracts.
use super::{
    AppWindow, GpuRow, VmRow, dialog,
    live_state::{State, allocation_values, review_text},
    model,
};
use hyper_gpu_support::{
    gui_model::Inventory,
    model::{Discovery, Power, Target, VmState},
    workflow::Plan,
};
use slint::{ComponentHandle, Model};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
type SharedState = Arc<Mutex<State>>;

#[cfg(windows)]
fn discover() -> Result<(Inventory, Option<String>), String> {
    use hyper_gpu_support::{process, runner, windows_hyperv};
    match runner::submit(runner::request(runner::Operation::Discover, None, None)) {
        Ok(value) => serde_json::from_value(value)
            .map(|i| (i, None))
            .map_err(|e| format!("Invalid protected inventory reply: {e}")),
        Err(error) if process::is_elevated()? => {
            // Preserve standalone elevated discovery; attachment never grants enrollment.
            Ok((
                Inventory {
                    discovery: windows_hyperv::discover()?,
                    managed: Default::default(),
                    enrolled: vec![],
                },
                Some(format!(
                    "Protected enrollment unavailable: {error}. Native inventory only; editing is blocked."
                )),
            ))
        }
        Err(error) => Err(error),
    }
}
#[cfg(not(windows))]
fn discover() -> Result<(Inventory, Option<String>), String> {
    Err("Hyper-V discovery requires Windows x64".into())
}

#[cfg(windows)]
fn plan(target: Target) -> Result<Plan, String> {
    use hyper_gpu_support::runner::{self, Operation};
    let value = runner::submit(runner::request(Operation::Plan, Some(target), None))?;
    serde_json::from_value(value).map_err(|e| format!("Invalid plan reply: {e}"))
}
#[cfg(not(windows))]
fn plan(_: Target) -> Result<Plan, String> {
    Err("Planning requires Windows x64".into())
}

#[cfg(windows)]
fn credential_dialog(id: &str) -> Result<bool, String> {
    hyper_gpu_support::credentials::prompt(id, windows::Win32::Foundation::HWND::default())?;
    // This slice has no guest operation to consume ephemeral credentials. Confirm
    // actual vault storage rather than claiming discarded input is session state.
    Ok(hyper_gpu_support::credentials::read(id)?.is_some())
}
#[cfg(not(windows))]
fn credential_dialog(_: &str) -> Result<bool, String> {
    Err("Guest credentials require Windows".into())
}

fn vm_row(vm: &VmState, discovery: &Discovery) -> VmRow {
    let gpu = if vm.gpus.is_empty() {
        "No GPU attached".to_owned()
    } else {
        vm.gpus
            .iter()
            .map(|interface| {
                discovery
                    .gpus
                    .iter()
                    .find(|gpu| gpu.interface.eq_ignore_ascii_case(interface))
                    .map_or_else(|| interface.clone(), |gpu| gpu.name.clone())
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    VmRow {
        id: vm.vm_id.clone().into(),
        name: vm.name.clone().into(),
        power: match vm.power {
            Power::Running => "Running",
            Power::Off => "Stopped",
            Power::Other(_) => "Unknown",
        }
        .into(),
        status: if vm.generation == 2 {
            "Unknown"
        } else {
            "Unsupported"
        }
        .into(),
        gpu: gpu.into(),
        os: "unknown".into(),
    }
}

fn present(ui: &AppWindow, state: &State, reset_draft: bool) {
    let Some(id) = state.view.selected.as_ref() else {
        ui.set_selected_id("".into());
        ui.set_eligible(false);
        ui.set_recovery(false);
        return;
    };
    ui.set_selected_id(id.clone().into());
    let Some(vm) = state.view.vm(id) else {
        ui.set_eligible(false);
        ui.set_validation(
            "The draft VM is no longer present. Discard the draft and Refresh.".into(),
        );
        return;
    };
    let Some(inventory) = state.view.inventory.as_ref() else {
        return;
    };
    let row = vm_row(vm, &inventory.discovery);
    ui.set_vm_name(row.name);
    ui.set_power(row.power);
    let enrolled = state.target(id).ok();
    ui.set_status(
        if vm.generation != 2 {
            "Unsupported"
        } else if enrolled.is_some() {
            "Enrolled"
        } else {
            "Unknown"
        }
        .into(),
    );
    let journal = inventory.managed.get(id).and_then(Option::as_ref);
    ui.set_recovery(journal.is_some_and(|j| j.pending));
    ui.set_observed(
        format!(
            "{}; {}{}",
            row.gpu,
            hyper_gpu_support::gui_model::allocation_text(vm),
            vm.vram.as_ref().map_or(String::new(), |v| format!(
                ": {} / {} / {}",
                v.minimum, v.optimal, v.maximum
            ))
        )
        .into(),
    );
    ui.set_desired(
        "Committed configuration not loaded. Draft starts from protected enrollment intent.".into(),
    );
    ui.set_verification(journal.and_then(|j| j.last_verified).map_or_else(
        || "Not verified; attachment does not prove guest graphics health".to_owned(),
        |t| format!("Last successful checked rendering: Unix UTC {t} (historical; not freshly verified)"),
    ).into());
    let eligibility = state.editor_eligibility(id);
    ui.set_eligible(eligibility.is_ok());
    if reset_draft {
        ui.set_gpu_enabled(enrolled.as_ref().is_some_and(|t| t.enabled));
        ui.set_gpu_index(
            enrolled
                .as_ref()
                .and_then(|t| {
                    inventory
                        .discovery
                        .gpus
                        .iter()
                        .position(|g| g.interface == t.gpu_interface)
                })
                .map_or(-1, |i| i as i32),
        );
        ui.set_values(model(
            enrolled
                .as_ref()
                .map_or_else(|| vec![String::new(); 12], allocation_values)
                .into_iter()
                .map(Into::into)
                .collect(),
        ));
        ui.set_dirty(false);
    }
    if let Err(error) = eligibility {
        ui.set_validation(error.into());
    } else if !ui.get_dirty() {
        ui.set_validation(draft_target(ui, state).err().unwrap_or_default().into());
    }
}

fn with_state(
    ui: &AppWindow,
    state: &SharedState,
    operation: impl FnOnce(&mut State) -> Result<(), String>,
) {
    let result = state
        .lock()
        .map_err(|_| "Live presentation state is unavailable; restart the application.".to_owned())
        .and_then(|mut state| operation(&mut state));
    if let Err(error) = result {
        dialog(ui, "error", "Action unavailable", &error);
    }
}
fn draft_target(ui: &AppWindow, state: &State) -> Result<Target, String> {
    state.draft(
        ui.get_selected_id().as_str(),
        ui.get_gpu_enabled(),
        ui.get_gpu_index(),
        &ui.get_values()
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>(),
    )
}

// Backend work runs off-thread; one in-flight job prevents older completions from
// overwriting newer session state. Only the event loop touches Slint models.
fn background<T: Send + 'static>(
    ui: &AppWindow,
    busy: &Arc<AtomicBool>,
    title: &str,
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
    complete: impl FnOnce(&AppWindow, Result<T, String>) + Send + 'static,
) -> Result<(), String> {
    if busy.swap(true, Ordering::AcqRel) {
        return Err("Another GUI operation is still running.".into());
    }
    ui.set_running(true);
    dialog(
        ui,
        "progress",
        title,
        "Waiting for the shared backend. No VM or guest changes are requested by this action.",
    );
    let weak = ui.as_weak();
    let worker_busy = busy.clone();
    if let Err(error) = std::thread::Builder::new()
        .name("gui-backend-read".into())
        .spawn(move || {
            let result = task();
            let completion_busy = worker_busy.clone();
            if weak
                .upgrade_in_event_loop(move |ui| {
                    completion_busy.store(false, Ordering::Release);
                    ui.set_running(false);
                    ui.set_dialog_kind("".into());
                    complete(&ui, result);
                })
                .is_err()
            {
                worker_busy.store(false, Ordering::Release);
            }
        })
    {
        busy.store(false, Ordering::Release);
        ui.set_running(false);
        ui.set_dialog_kind("".into());
        return Err(format!("Cannot start backend task: {error}"));
    }
    Ok(())
}

fn refresh(ui: &AppWindow, state: &SharedState, busy: &Arc<AtomicBool>) -> Result<(), String> {
    ui.set_notice("Reading protected inventory. Existing observations remain historical until refresh succeeds.".into());
    let state = state.clone();
    background(
        ui,
        busy,
        "Refreshing inventory",
        discover,
        move |ui, result| {
            with_state(ui, &state, |state| {
                match result {
                    Ok((inventory, warning)) => {
                        let rows = inventory
                            .discovery
                            .vms
                            .iter()
                            .map(|vm| {
                                let mut row = vm_row(vm, &inventory.discovery);
                                if vm.generation == 2
                                    && inventory.enrolled.iter().any(|t| t.vm_id == vm.vm_id)
                                {
                                    row.status = "Enrolled".into();
                                }
                                row
                            })
                            .collect();
                        ui.set_vms(model(rows));
                        ui.set_gpu_information(model(inventory.discovery.gpus.iter().map(|gpu| GpuRow {
                        name: gpu.name.clone().into(),
                        description: format!("Interface: {}\nDriver: {}\nVRAM provider range: {} .. {} (optimal {})\nPreparation adapter: {}\nCompute / encode / decode: not exposed by the current backend.\nAttachment does not prove guest graphics health; sharing is not qualified.", gpu.interface, gpu.driver_version, gpu.vram.minimum, gpu.vram.maximum, gpu.vram.optimal, if gpu.preparation_supported { "available" } else { "not implemented" }).into(),
                    }).collect()));
                        ui.set_gpu_options(model(
                            inventory
                                .discovery
                                .gpus
                                .iter()
                                .map(|gpu| gpu.name.clone().into())
                                .collect(),
                        ));
                        state.refresh(inventory);
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
                            ui.set_validation(
                                draft_target(ui, state).err().unwrap_or_default().into(),
                            );
                        }
                        ui.set_system_summary("Actual Hyper-V inventory and protected enrollment read. Graphics results are historical; committed configuration is not loaded.".into());
                        ui.set_notice(warning.unwrap_or_else(|| "Protected inventory loaded. Enrolled pairs support drafts and fresh plan previews. Apply and guest verification await worker integration.".into()).into());
                        Ok(())
                    }
                    Err(error) => {
                        state.view.needs_readback = true;
                        ui.set_eligible(false);
                        ui.set_validation(
                            "Inventory is historical; Refresh before further actions.".into(),
                        );
                        ui.set_notice(format!("Live inventory unavailable: {error}. Existing rows are historical; Refresh retries discovery.").into());
                        ui.set_system_summary(
                            format!("Live discovery unavailable: {error}").into(),
                        );
                        Err(error)
                    }
                }
            });
        },
    )
}

fn review(ui: &AppWindow, state: &SharedState, busy: &Arc<AtomicBool>) -> Result<(), String> {
    let target = state
        .lock()
        .map_err(|_| "Live presentation state unavailable".to_owned())
        .and_then(|state| draft_target(ui, &state))?;
    let expected = target.clone();
    background(
        ui,
        busy,
        "Building fresh plan",
        move || plan(target),
        move |ui, result| match result {
            Ok(plan) if plan.desired == expected && plan.observed.vm_id == expected.vm_id => {
                ui.set_needs_shutdown(plan.preview.guest_downtime);
                dialog(
                    ui,
                    "review",
                    "Review your draft · execution unavailable",
                    &review_text(&plan),
                );
            }
            Ok(_) => dialog(
                ui,
                "error",
                "Plan unavailable",
                "The backend plan does not match the selected draft. Refresh before retrying.",
            ),
            Err(error) => dialog(
                ui,
                "error",
                "Plan unavailable",
                &format!(
                    "{error}\nYour draft is preserved. No VM or guest changes were performed."
                ),
            ),
        },
    )
}

pub(super) fn run(ui: AppWindow) -> Result<(), Box<dyn std::error::Error>> {
    ui.set_vms(model(Vec::new()));
    ui.set_selected_id("".into());
    ui.set_gpu_information(model(Vec::new()));
    ui.set_gpu_options(model(Vec::new()));
    ui.set_values(model(vec!["".into(); 12]));
    ui.set_eligible(false);
    ui.set_system_summary("Live inventory has not been read yet.".into());
    let busy = Arc::new(AtomicBool::new(false));
    let state = Arc::new(Mutex::new(State::default()));
    let weak = ui.as_weak();
    let selection_state = state.clone();
    ui.on_select_vm(move |id| {
        if let Some(ui) = weak.upgrade() { with_state(&ui, &selection_state, |state| {
            if ui.get_running() || id == ui.get_selected_id() { return Ok(()); }
            if ui.get_dirty() {
                state.pending_selection = Some(id.to_string());
                dialog(&ui, "switch", "Keep your changes?", "There is one unapplied VM draft. Stay here, review it, or discard it before switching. Nothing has been applied.");
            } else { state.view.selected = Some(id.to_string()); present(&ui, state, true); }
            Ok(())
        }); }
    });
    let weak = ui.as_weak();
    let edit_state = state.clone();
    ui.on_edited(move || {
        if let Some(ui) = weak.upgrade() {
            with_state(&ui, &edit_state, |state| {
                ui.set_dirty(true);
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
                match draft_target(&ui, state) {
                    Ok(target) => {
                        state.view.draft = Some(target);
                        ui.set_validation("".into());
                    }
                    Err(error) => {
                        // A marker retains the VM even for invalid text; raw edits stay in the UI.
                        state.view.draft = Some(state.target(ui.get_selected_id().as_str())?);
                        ui.set_validation(error.into());
                    }
                }
                Ok(())
            });
        }
    });
    let weak = ui.as_weak();
    let action_state = state.clone();
    let action_busy = busy.clone();
    ui.on_action(move |action| {
        let Some(ui) = weak.upgrade() else { return; }; if ui.get_running() { return; }
        let result = match action.as_str() {
            "refresh" => refresh(&ui, &action_state, &action_busy),
            "review" => review(&ui, &action_state, &action_busy),
            "reapply" => {
                // Preserve allocation edits while deliberately previewing enabled support.
                ui.set_gpu_enabled(true);
                ui.invoke_edited();
                review(&ui, &action_state, &action_busy)
            }
            "close" => { ui.set_dialog_kind("".into()); Ok(()) },
            "exit" => { ui.set_dirty(false); slint::quit_event_loop().map_err(|e| e.to_string()) },
            "discard" | "switch" => {
                with_state(&ui, &action_state, |state| {
                    state.view.discard();
                    state.draft_gpu = None;
                    if action == "switch" { state.view.selected = state.pending_selection.take(); }
                    present(&ui, state, true); ui.set_dirty(false); ui.set_dialog_kind("".into());
                    ui.set_notice("Draft discarded. No configuration was saved or VM state changed.".into()); Ok(())
                }); Ok(())
            }
            "more" => { dialog(&ui, "more", "VM actions", "Reapply / Update reads a fresh shared plan for the current draft. Guest verification and forgetting pairing remain unavailable until worker and protected configuration bindings are ready."); Ok(()) },
            "credentials" => {
                let selected = ui.get_selected_id().to_string();
                action_state.lock().map_err(|_| "Live presentation state unavailable".to_owned()).and_then(|s| s.target(&selected))
                    .and_then(|target| background(&ui, &action_busy, "Guest credentials", move || credential_dialog(&target.vm_id), |ui, result| match result {
                        Ok(true) => dialog(ui, "info", "Stored guest credentials available", "A stored credential was confirmed in this user's Windows Credential Manager for the selected VM. No guest connection or graphics check was performed."),
                        Ok(false) => dialog(ui, "info", "Guest credentials not retained", "Remember was not selected. The entered credential was discarded because no guest operation is connected yet. Nothing was stored or checked in the guest."),
                        Err(error) => dialog(ui, "error", "Guest credentials unavailable", &error),
                    }))
            }
            "diagnostics" | "recovery" => {
                with_state(&ui, &action_state, |state| {
                    let id = ui.get_selected_id(); let inventory = state.view.inventory.as_ref().ok_or("Refresh inventory first.")?;
                    let body = serde_json::to_string_pretty(&serde_json::json!({"observed": state.view.vm(&id), "managed": inventory.managed.get(id.as_str()).and_then(Option::as_ref), "historical": state.view.needs_readback, "enrollment": state.target(&id).ok()})).map_err(|e| e.to_string())?;
                    dialog(&ui, "info", "Observed state and recovery records", &format!("Last protected inventory read; no fresh guest verification. Refresh obtains current host state. Pending journals are retained and are not cleared by inspection.\n\n{body}")); Ok(())
                }); Ok(())
            }
            "shortcuts" => { dialog(&ui, "info", "Application help", "Select a real VM, inspect System, or Refresh protected inventory. Enrolled pairs support in-memory toggles and raw VRAM drafts with fresh CLI planner previews. Settings opens the native credential dialog for the selected enrolled VM. Apply, Verify and enrollment await worker integration. --mock-gui rehearses fixtures without writes."); Ok(()) },
            _ => Err("This action is not connected yet. Use the supported CLI; no operation was performed.".into()),
        };
        if let Err(error) = result { dialog(&ui, "error", "Action unavailable", &error); }
    });
    let weak = ui.as_weak();
    ui.window().on_close_requested(move || {
        if let Some(ui) = weak.upgrade() {
            if ui.get_running() { return slint::CloseRequestResponse::KeepWindowShown; }
            if ui.get_dirty() { dialog(&ui, "exit", "Unapplied draft", "Keep editing or discard the in-memory draft and close. Nothing has been applied."); return slint::CloseRequestResponse::KeepWindowShown; }
        }
        slint::CloseRequestResponse::HideWindow
    });
    refresh(&ui, &state, &busy)?;
    ui.run()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper_gpu_support::model::Settings;
    #[test]
    fn unknown_health_enrollment_and_unresolved_attachment_are_not_samples() {
        let vm = VmState {
            vm_id: "12345678-1234-1234-1234-123456789abc".into(),
            name: "Actual name".into(),
            power: Power::Other(42),
            generation: 1,
            gpus: vec!["selected-interface".into()],
            vram: None,
            settings: Settings {
                low_mmio: 0,
                high_mmio: 0,
                cache_types: false,
                automatic_checkpoints: true,
            },
        };
        let row = vm_row(
            &vm,
            &Discovery {
                vms: vec![],
                gpus: vec![],
            },
        );
        assert_eq!(row.id, vm.vm_id);
        assert_eq!(row.name, vm.name);
        assert_eq!(row.gpu, "selected-interface");
        assert_eq!(row.status, "Unsupported");
        assert_eq!(row.power, "Unknown");
        assert_eq!(row.os, "unknown");
    }
}
