//! Audited live reads; effects remain behind unfinished worker/save contracts.
use super::{
    AppWindow, GpuRow, VmRow, dialog,
    live_state::{OperationReadback, State, allocation_values, review_summary, review_text},
    model,
};
use hyper_gpu_support::{
    gui_model::Inventory,
    model::{Discovery, Power, Resource, Target, VmState},
    workflow::Plan,
};
use slint::{ComponentHandle, Model};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
type SharedState = Arc<Mutex<State>>;
type InventoryRead = (Inventory, Option<String>);

#[cfg(windows)]
mod profile_removal;
#[cfg(windows)]
use profile_removal::{forget_live, review_forget_live};

// Artifact authentication hashes installed files; call only inside background jobs.
// This availability observation never substitutes for the worker's fresh checks.
fn protected_worker_available() -> bool {
    #[cfg(windows)]
    {
        hyper_gpu_support::worker::available().is_ok()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

// Preserve repeated UI elements and editor focus when an inventory read only
// changes row contents. Rebuild only when the number of rows changes.
fn update_model<T: Clone + PartialEq + 'static>(
    current: slint::ModelRc<T>,
    rows: Vec<T>,
) -> slint::ModelRc<T> {
    if current.row_count() == rows.len()
        && let Some(vector) = current.as_any().downcast_ref::<slint::VecModel<T>>()
    {
        for (index, row) in rows.into_iter().enumerate() {
            if vector.row_data(index).as_ref() != Some(&row) {
                vector.set_row_data(index, row);
            }
        }
        current
    } else {
        model(rows)
    }
}

#[derive(Clone)]
pub(super) struct Source;

impl Source {
    fn enrollment_recovery(
        &self,
    ) -> Result<Option<hyper_gpu_support::enrollment::PendingPair>, String> {
        #[cfg(windows)]
        {
            hyper_gpu_support::runner::pending_pair()
        }
        #[cfg(not(windows))]
        {
            Ok(None)
        }
    }
    fn import_recovery(
        &self,
    ) -> Result<Option<hyper_gpu_support::configuration_store::ImportRecord>, String> {
        #[cfg(windows)]
        {
            hyper_gpu_support::runner::import_record()
        }
        #[cfg(not(windows))]
        {
            Ok(None)
        }
    }
    fn recovery(
        &self,
    ) -> Result<Option<hyper_gpu_support::configuration_store::SaveRecord>, String> {
        #[cfg(windows)]
        {
            hyper_gpu_support::runner::recovery_record()
        }
        #[cfg(not(windows))]
        {
            Ok(None)
        }
    }
    fn committed(&self) -> Option<hyper_gpu_support::configuration_store::StoreSnapshot> {
        #[cfg(windows)]
        {
            Some(hyper_gpu_support::configuration_store::read_committed())
        }
        #[cfg(not(windows))]
        {
            None
        }
    }
    fn discover(&self) -> Result<InventoryRead, String> {
        discover()
    }
    fn plan(&self, target: Target, reinstall: bool) -> Result<Plan, String> {
        plan(target, reinstall)
    }
    fn credentials(&self, id: &str, name: &str) -> Result<bool, String> {
        credential_dialog(id, name)
    }

    fn operation_readback(&self) -> Result<OperationReadback, String> {
        Ok(OperationReadback {
            committed: self
                .committed()
                .ok_or("Saved configuration requires Windows")?,
            recovery: self.recovery()?,
        })
    }
}

#[cfg(windows)]
fn discover() -> Result<(Inventory, Option<String>), String> {
    use hyper_gpu_support::{process, runner, windows_hyperv};
    if runner::pending_setup()?.is_some() || !runner::installed()? {
        return hyper_gpu_support::setup::discover().map(|inventory| (inventory,
            Some("First-install inventory read through Windows elevation. Select a VM/GPU and Apply to review package setup and exact enrollment. GPU changes require a separate subsequent review.".into())));
    }
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
                    "{}. Native inventory only; editing is blocked.",
                    hyper_gpu_support::reporting::operator_error("Protected enrollment", &error)
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

/// Poll only installed inventory; never initiate first-install elevation.
#[cfg(windows)]
fn discover_status() -> Result<InventoryRead, String> {
    use hyper_gpu_support::runner;
    if runner::pending_setup()?.is_some() || !runner::installed()? {
        return Err("Setup requires attention. Use Refresh to continue.".into());
    }
    let value = runner::submit(runner::request(runner::Operation::Discover, None, None))?;
    serde_json::from_value(value)
        .map(|inventory| (inventory, None))
        .map_err(|e| format!("Invalid protected inventory reply: {e}"))
}

#[cfg(not(windows))]
fn discover_status() -> Result<InventoryRead, String> {
    Err("Hyper-V discovery requires Windows x64".into())
}

#[cfg(windows)]
fn plan(target: Target, reinstall: bool) -> Result<Plan, String> {
    use hyper_gpu_support::runner::{self, Operation};
    let operation = if reinstall {
        Operation::ReinstallPlan
    } else {
        Operation::Plan
    };
    let value = runner::submit(runner::request(operation, Some(target), None))?;
    serde_json::from_value(value).map_err(|e| format!("Invalid plan reply: {e}"))
}
#[cfg(not(windows))]
fn plan(_: Target, _: bool) -> Result<Plan, String> {
    Err("Planning requires Windows x64".into())
}

#[cfg(windows)]
fn credential_dialog(id: &str, name: &str) -> Result<bool, String> {
    hyper_gpu_support::credentials::prompt_named(
        id,
        name,
        windows::Win32::Foundation::HWND::default(),
    )?;
    // This settings action has no guest operation to consume ephemeral input.
    // Apply/Verify prompt separately; confirm actual opt-in vault storage here.
    Ok(hyper_gpu_support::credentials::read(id)?.is_some())
}
#[cfg(not(windows))]
fn credential_dialog(_: &str, _: &str) -> Result<bool, String> {
    Err("Guest credentials require Windows".into())
}

fn credential_request_cancelled(ui: &AppWindow, error: &str, recovery: bool) -> bool {
    #[cfg(windows)]
    if hyper_gpu_support::credentials::is_prompt_cancelled(error) {
        dialog(
            ui,
            "info",
            "Guest credential request cancelled",
            if recovery {
                "No reconciliation was started. The existing recovery record and hold remain. Your pending edits are retained."
            } else {
                "No guest operation was started. Your pending edits are retained. Review the action again when you are ready to continue."
            },
        );
        return true;
    }
    #[cfg(not(windows))]
    let _ = (ui, error, recovery);
    false
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

fn observed_row(vm: &VmState, inventory: &Inventory, historical: bool) -> VmRow {
    let mut row = vm_row(vm, &inventory.discovery);
    if vm.generation == 2 && inventory.enrolled.iter().any(|t| t.vm_id == vm.vm_id) {
        row.status = if inventory.recorded_state(&vm.vm_id).recovery_required {
            "Recovery required"
        } else {
            "Authorized"
        }
        .into();
    }
    if historical {
        row.power = "Status unavailable".into();
        row.status = "Refresh needed".into();
        row.gpu = "Status unavailable".into();
    }
    row
}

fn present(ui: &AppWindow, state: &State, reset_draft: bool) {
    ui.set_presenting(true);
    present_inner(ui, state, reset_draft);
    present_actions(ui, state);
    ui.set_presenting(false);
}

fn present_actions(ui: &AppWindow, state: &State) {
    let values = ui
        .get_values()
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    let actions = state.selected_actions(ui.get_gpu_index(), &values);
    ui.set_verify_unavailable(actions.verify.into());
    ui.set_repair_unavailable(actions.repair.into());
    ui.set_remove_unavailable(actions.remove.into());
    ui.set_forget_unavailable(actions.forget.into());
    ui.set_action_recovery(actions.recovery);
    ui.set_gpu_selection_fixed(state.fixed_gpu_index().is_some());
}

fn present_inner(ui: &AppWindow, state: &State, reset_draft: bool) {
    let Some(id) = state.view.selected.as_ref() else {
        ui.set_selected_id("".into());
        ui.set_vm_name("No VM selected".into());
        ui.set_power("Unknown".into());
        ui.set_status("Unknown".into());
        ui.set_observed("No VM selected".into());
        ui.set_desired("No VM selected".into());
        ui.set_applied("No VM selected".into());
        ui.set_pending("No unapplied edits".into());
        ui.set_verification("Unknown; no VM selected".into());
        ui.set_support_summary("Select a VM to read its state.".into());
        ui.set_eligible(false);
        ui.set_recovery(
            state.recovery.is_some()
                || state.import_recovery.is_some()
                || state.enrollment_recovery.is_some(),
        );
        ui.set_dirty(false);
        ui.set_validation("".into());
        return;
    };
    ui.set_selected_id(id.clone().into());
    let Some(vm) = state.view.vm(id) else {
        ui.set_eligible(false);
        ui.set_vm_name("Selected VM unavailable".into());
        ui.set_power("Unknown".into());
        ui.set_status("Unavailable".into());
        ui.set_support_summary(state.host_configuration_text(id).into());
        ui.set_observed("Selected VM was not returned by the latest inventory read".into());
        ui.set_verification("Unknown; the selected VM is unavailable".into());
        ui.set_desired(state.saved_desired_text(id).into());
        ui.set_applied(state.applied_text(id).into());
        ui.set_pending(
            if ui.get_dirty() {
                "Unapplied edits retained for the unavailable VM."
            } else {
                "No unapplied edits"
            }
            .into(),
        );
        ui.set_recovery(
            state.recovery.is_some()
                || state.import_recovery.is_some()
                || state.enrollment_recovery.is_some(),
        );
        ui.set_validation(
            "The draft VM is no longer present. Discard the draft and Refresh.".into(),
        );
        return;
    };
    let Some(inventory) = state.view.inventory.as_ref() else {
        return;
    };
    let row = observed_row(vm, inventory, state.view.needs_readback);
    ui.set_vm_name(row.name);
    ui.set_power(row.power);
    let enrolled = state.target(id).ok();
    ui.set_status(row.status);
    let recorded = inventory.recorded_state(id);
    ui.set_recovery(
        recorded.recovery_required
            || state.recovery.is_some()
            || state.import_recovery.is_some()
            || state.enrollment_recovery.is_some(),
    );
    ui.set_observed(
        format!(
            "{}{}; {}; {}",
            if state.view.needs_readback {
                "Historical: "
            } else {
                ""
            },
            row.gpu,
            Resource::ALL
                .into_iter()
                .map(|resource| {
                    vm.allocation(resource).map_or_else(
                        || format!("{}: unobserved or provider default", resource.name()),
                        |value| {
                            format!(
                                "{} readback: {} / {} / {} (raw)",
                                resource.name(),
                                value.minimum,
                                value.optimal,
                                value.maximum
                            )
                        },
                    )
                })
                .collect::<Vec<_>>()
                .join("; "),
            recorded.preparation,
        )
        .into(),
    );
    ui.set_desired(state.saved_desired_text(id).into());
    ui.set_applied(state.applied_text(id).into());
    ui.set_pending(
        if !reset_draft && ui.get_dirty() {
            "Unapplied edits for this VM; validate and review before effects."
        } else {
            "No unapplied edits"
        }
        .into(),
    );
    ui.set_verification(recorded.graphics.into());
    ui.set_support_summary(
        format!(
            "Management: {}. GPU support: {}. Saved setting: {}. Graphics: {}. {}",
            if state.view.needs_readback {
                "status unavailable"
            } else if inventory.enrolled.iter().any(|target| target.vm_id == *id) {
                "authorized; guest drivers are not verified by authorization"
            } else {
                "not authorized"
            },
            if state.view.needs_readback {
                "status unavailable"
            } else if vm.gpus.is_empty() {
                "not attached"
            } else {
                "attached"
            },
            if state.committed.revision(id).is_err() {
                "unavailable"
            } else if let Some(target) = state
                .view
                .configuration
                .targets
                .iter()
                .find(|t| &t.vm_id == id)
            {
                if target.enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            } else {
                "not saved"
            },
            if recorded.recovery_required {
                "recovery required"
            } else if inventory
                .managed
                .get(id)
                .and_then(|j| j.as_ref())
                .is_some_and(|j| j.last_verified.is_some())
            {
                "previously verified; current health unknown"
            } else {
                "not verified"
            },
            state.host_configuration_text(id),
        )
        .into(),
    );
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
        ui.set_values(update_model(
            ui.get_values(),
            enrolled
                .as_ref()
                .map_or_else(|| vec![String::new(); 12], allocation_values)
                .into_iter()
                .map(Into::into)
                .collect(),
        ));
        ui.set_dirty(false);
    }
    resource_controls(ui, state);
    if let Err(error) = eligibility {
        ui.set_validation(
            hyper_gpu_support::reporting::operator_error("Configuration validation", &error).into(),
        );
    } else if !ui.get_dirty() {
        ui.set_validation(draft_target(ui, state).err().unwrap_or_default().into());
    }
}

fn resource_controls(ui: &AppWindow, state: &State) {
    let gpu = state.view.inventory.as_ref().and_then(|inventory| {
        usize::try_from(ui.get_gpu_index())
            .ok()
            .and_then(|index| inventory.discovery.gpus.get(index))
    });
    ui.set_resource_available(model(
        Resource::ALL
            .into_iter()
            .map(|resource| gpu.is_some_and(|gpu| gpu.limits(resource).is_some()))
            .collect(),
    ));
    ui.set_resource_hints(model(
        Resource::ALL
            .into_iter()
            .map(|resource| {
                gpu.map_or_else(
                    || "Select an available physical GPU; capability unknown".into(),
                    |gpu| {
                        let mut text = gpu.resource_text(resource);
                        if resource == Resource::Vram
                            && let Some(bytes) = gpu.physical_vram_bytes
                        {
                            text.push_str(&format!(
                                "; physical dedicated memory: {}",
                                super::memory::format_gib(bytes)
                            ));
                        }
                        text
                    },
                )
                .into()
            })
            .collect(),
    ));
    ui.set_resource_defaults(model(
        Resource::ALL
            .into_iter()
            .flat_map(|resource| {
                gpu.and_then(|gpu| gpu.limits(resource)).map_or_else(
                    || vec![String::new(); 3],
                    |value| {
                        vec![
                            value.minimum.to_string(),
                            value.optimal.to_string(),
                            value.maximum.to_string(),
                        ]
                    },
                )
            })
            .map(Into::into)
            .collect(),
    ));
    let values = ui
        .get_values()
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>();
    match state.memory_slider(ui.get_gpu_index(), &values) {
        Ok(slider) => {
            ui.set_memory_available(true);
            ui.set_memory_minimum(slider.minimum);
            ui.set_memory_maximum(slider.maximum);
            ui.set_memory_position(slider.position);
            ui.set_memory_label(slider.label.into());
            ui.set_memory_range_start(slider.range_start.into());
            ui.set_memory_range_end(slider.range_end.into());
            let note = slider
                .range_note
                .map_or_else(String::new, |note| format!("\n{note}"));
            ui.set_memory_hint(format!("Choose a request from 1 GB to the card's detected capacity. Hyper-V validates the requested bytes; actual allocation behavior awaits live qualification.{note}").into());
        }
        Err(reason) => {
            ui.set_memory_available(false);
            ui.set_memory_minimum(1);
            ui.set_memory_maximum(2);
            ui.set_memory_position(1);
            ui.set_memory_label("Unavailable".into());
            ui.set_memory_range_start("—".into());
            ui.set_memory_range_end("—".into());
            ui.set_memory_hint(reason.into());
        }
    }
}

fn with_state(
    ui: &AppWindow,
    state: &SharedState,
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
        dialog(ui, "error", "Action unavailable", &error);
        return false;
    }
    true
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
    background_for_vm(ui, busy, title, None, false, task, complete)
}

fn run_background_task<T>(task: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(task))
        .unwrap_or_else(|_| Err("The backend task ended without a terminal outcome. Refresh and inspect protected recovery before retrying.".into()))
}

/// Gather protected files while still busy, without blocking Slint completion.
#[cfg(windows)]
fn background_operation<T: Send + 'static>(
    ui: &AppWindow,
    busy: &Arc<AtomicBool>,
    title: &str,
    vm_name: Option<&str>,
    guest_access: bool,
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
    complete: impl FnOnce(&AppWindow, Result<T, String>, Result<OperationReadback, String>)
    + Send
    + 'static,
) -> Result<(), String> {
    background_for_vm(
        ui,
        busy,
        title,
        vm_name,
        guest_access,
        move || Ok((run_background_task(task), Source.operation_readback())),
        move |ui, result| match result {
            Ok((outcome, readback)) => complete(ui, outcome, readback),
            Err(error) => complete(ui, Err(error.clone()), Err(error)),
        },
    )
}

fn background_for_vm<T: Send + 'static>(
    ui: &AppWindow,
    busy: &Arc<AtomicBool>,
    title: &str,
    vm_name: Option<&str>,
    guest_access: bool,
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
    complete: impl FnOnce(&AppWindow, Result<T, String>) + Send + 'static,
) -> Result<(), String> {
    if busy.swap(true, Ordering::AcqRel) {
        return Err("Another GUI operation is still running.".into());
    }
    ui.set_updating(true);
    let silent = title == "Updating VM status";
    if !silent {
        ui.set_running(true);
    }
    if !silent {
        let guest_message = format!(
            "Virtual machine: {}\n\nSign in with the Windows administrator account you use inside this virtual machine. The app will set up GPU support and check that graphics work. Progress will appear below.",
            vm_name.unwrap_or("the selected VM")
        );
        dialog(
            ui,
            "progress",
            title,
            if guest_access
                && matches!(
                    title,
                    "Apply approved changes"
                        | "Verify approved graphics"
                        | "Reconcile observed outcome"
                        | "Guest credentials"
                )
            {
                &guest_message
            } else if title == "Apply approved changes" {
                "Applying the reviewed GPU changes and checking host readback. The VM will be left off after successful completion. No guest credentials, preparation or graphics check are required."
            } else if title == "Reconcile observed outcome" {
                "Checking the already-achieved recorded outcome without repeating GPU work or accessing the guest. Unresolved state retains recovery."
            } else if title == "Building fresh plan" {
                "Preparing your changes may take some time. Checking the selected VM, GPU and installed driver files before showing the changes for approval. Nothing is being applied yet."
            } else if title == "Retry configuration save only" {
                "Requesting Windows elevation to save the verified configuration. No GPU operation will be repeated."
            } else if title == "Remove inactive pairing and profile" {
                "Requesting Windows approval to remove the reviewed inactive pairing, saved settings and managed history. Other profiles, the VM, guest files and shared package artifacts remain. Interrupted work retains profile recovery."
            } else {
                "Reading the information needed for this action…"
            },
        );
    }
    let weak = ui.as_weak();
    let worker_busy = busy.clone();
    if let Err(error) = std::thread::Builder::new()
        .name("gui-backend-read".into())
        .spawn(move || {
            let result = run_background_task(task);
            let completion_busy = worker_busy.clone();
            if weak
                .upgrade_in_event_loop(move |ui| {
                    completion_busy.store(false, Ordering::Release);
                    ui.set_updating(false);
                    if !silent {
                        ui.set_running(false);
                        ui.set_dialog_kind("".into());
                    }
                    complete(&ui, result);
                })
                .is_err()
            {
                worker_busy.store(false, Ordering::Release);
            }
        })
    {
        busy.store(false, Ordering::Release);
        ui.set_updating(false);
        if !silent {
            ui.set_running(false);
            ui.set_dialog_kind("".into());
        }
        return Err(format!("Cannot start backend task: {error}"));
    }
    Ok(())
}

fn refresh(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
    automatic: bool,
) -> Result<(), String> {
    if automatic
        && !state
            .lock()
            .map_err(|_| "Presentation state unavailable")?
            .can_poll()
    {
        // First-install observations remain usable until an explicit refresh.
        // The installed-only poll must not turn them historical every 15 seconds.
        return Ok(());
    }
    if !automatic {
        ui.set_notice("Refreshing VM information...".into());
    }
    let state = state.clone();
    let source = source.clone();
    background(
        ui,
        busy,
        if automatic {
            "Updating VM status"
        } else {
            "Refreshing inventory"
        },
        move || {
            let (inventory, warning) = if automatic {
                discover_status()?
            } else {
                source.discover()?
            };
            Ok((
                inventory,
                warning,
                source.committed(),
                source.recovery()?,
                source.import_recovery()?,
                source.enrollment_recovery()?,
                protected_worker_available(),
            ))
        },
        move |ui, result| {
            with_state(ui, &state, |state| {
                match result {
                    Ok((
                        inventory,
                        warning,
                        committed,
                        recovery,
                        import_recovery,
                        enrollment_recovery,
                        worker_available,
                    )) => {
                        ui.set_execution_ready(false);
                        state.enrollment_review = None;
                        state.setup_artifacts = None;
                        state.enrollment_recovery_revision = None;
                        state.approved_plan = None;
                        state.verification_plan = None;
                        // A conflicting draft blocks that editor, not fresh VM/GPU
                        // observations or readable intent for other VMs.
                        let configuration_warning =
                            committed.and_then(|committed| state.load_committed(committed).err());
                        ui.set_presenting(true);
                        let rows = inventory
                            .discovery
                            .vms
                            .iter()
                            .map(|vm| observed_row(vm, &inventory, false))
                            .collect();
                        ui.set_vms(update_model(ui.get_vms(), rows));
                        ui.set_gpu_information(update_model(ui.get_gpu_information(), inventory.discovery.gpus.iter().map(|gpu| GpuRow {
                        name: gpu.name.clone().into(),
                        description: format!("Interface: {}\nDriver: {}\n{}\nPreparation adapter: {}\nAttachment does not prove guest graphics health; sharing is not qualified.", gpu.interface, gpu.driver_version, Resource::ALL.into_iter().map(|resource| gpu.resource_text(resource)).collect::<Vec<_>>().join("\n"), if gpu.preparation_supported { "available" } else { "not implemented" }).into(),
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
                            ui.set_validation(
                                draft_target(ui, state).err().unwrap_or_default().into(),
                            );
                        }
                        ui.set_system_summary("Hyper-V virtual machines and available GPUs. Graphics results reflect previous checks.".into());
                        let issues = &state
                            .view
                            .inventory
                            .as_ref()
                            .ok_or("inventory unavailable")?
                            .discovery
                            .issues;
                        let mut notice = if issues.is_empty() {
                            warning.unwrap_or_else(|| {
                                "VM status updated. Graphics results reflect previous checks."
                                    .into()
                            })
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
                            notice.push_str(" Some committed configuration could not be read or validated. Affected intent remains unknown; other readable VM files are available.");
                        }
                        ui.set_inventory_details(notice.clone().into());
                        if !issues.is_empty()
                            && configuration_warning.is_none()
                            && recovery_warning.is_none()
                            && state.committed.errors.is_empty()
                        {
                            notice = "Some VM or GPU information could not be read. See Technical Details.".into();
                        }
                        ui.set_notice(notice.into());
                        Ok(())
                    }
                    Err(error) => {
                        state.approved_plan = None;
                        state.verification_plan = None;
                        state.enrollment_review = None;
                        state.setup_artifacts = None;
                        state.enrollment_recovery_revision = None;
                        ui.set_execution_ready(false);
                        let error =
                            hyper_gpu_support::reporting::operator_error("Inventory read", &error);
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
                        ui.set_validation(
                            "Your edits are kept. Refresh VM information before Apply.".into(),
                        );
                        ui.set_inventory_details(error.clone().into());
                        ui.set_notice("Could not refresh VM status. You can keep editing; use Refresh before Apply.".into());
                        ui.set_system_summary(
                            format!("Live discovery unavailable: {error}").into(),
                        );
                        if automatic { Ok(()) } else { Err(error) }
                    }
                }
            });
        },
    )
}

fn review(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
) -> Result<(), String> {
    review_operation(ui, state, busy, source, false)
}

fn review_operation(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
    reinstall: bool,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        let target = {
            let mut state = state
                .lock()
                .map_err(|_| "Live presentation state unavailable")?;
            state.enrollment_review = None;
            state.approved_plan = None;
            let target = draft_target(ui, &state)?;
            state.view.proposal_eligibility(&target)?;
            (!state.enrolled(&target)).then_some(target)
        };
        if let Some(target) = target {
            if reinstall {
                return Err("Repair / Reinstall requires an authorized VM/GPU pair. Review enrollment first.".into());
            }
            return review_enrollment(ui, state, busy, target);
        }
    }
    let (target, committed_revision) = state
        .lock()
        .map_err(|_| "Live presentation state unavailable".to_owned())
        .and_then(|mut state| {
            state.approved_plan = None;
            draft_target(ui, &state).and_then(|target| {
                let revision = state.committed.revision(&target.vm_id)?;
                Ok((target, revision))
            })
        })?;
    let expected = target.clone();
    let planning_revision = committed_revision.clone();
    let source = source.clone();
    let review_state = state.clone();
    background(
        ui,
        busy,
        "Building fresh plan",
        move || {
            if let Some(committed) = source.committed()
                && committed.revision(&target.vm_id)? != planning_revision
            {
                return Err("Committed configuration changed externally. Draft preserved; discard and Refresh.".into());
            }
            source
                .plan(target, reinstall)
                .map(|plan| (plan, protected_worker_available()))
        },
        move |ui, result| match result {
            Ok((plan, worker_available))
                if plan.desired == expected
                    && plan.observed.vm_id == expected.vm_id
                    && plan.reinstall == reinstall =>
            {
                let gpu_name = review_state
                    .lock()
                    .ok()
                    .and_then(|state| {
                        state
                            .view
                            .inventory
                            .as_ref()?
                            .discovery
                            .gpus
                            .iter()
                            .find(|gpu| gpu.interface == plan.desired.gpu_interface)
                            .map(|gpu| gpu.name.clone())
                    })
                    .unwrap_or_else(|| plan.desired.gpu_interface.clone());
                let memory_summary = super::live_state::memory_review(&plan);
                ui.set_execution_ready(worker_available);
                let accepted = review_state
                    .lock()
                    .map_err(|_| "Presentation state unavailable".to_owned())
                    .and_then(|mut state| {
                        let current = draft_target(ui, &state)?;
                        state.accept_review(plan.clone(), &current, committed_revision.clone())
                    });
                if let Err(error) = accepted {
                    dialog(ui, "error", "Review again", &error);
                    return;
                }
                ui.set_needs_shutdown(plan.preview.guest_downtime);
                dialog(
                    ui,
                    "review",
                    if reinstall {
                        "Review Repair / Reinstall"
                    } else {
                        "Review changes before Apply"
                    },
                    &format!(
                        "Physical GPU: {gpu_name}\n{memory_summary}\n{}\n\nThese changes are still pending. Apply confirms this review; Cancel returns to editing. Windows will request approval before changes. Configuration is saved only after verification.",
                        review_summary(&plan)
                    ),
                );
                ui.set_dialog_details(review_text(&plan).into());
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

#[cfg(windows)]
fn review_enrollment(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    target: Target,
) -> Result<(), String> {
    let reviewed_state = state.clone();
    background(
        ui,
        busy,
        "Review VM/GPU enrollment",
        move || {
            let pending = hyper_gpu_support::runner::pending_setup()?;
            if pending.is_none() && hyper_gpu_support::runner::installed()? {
                hyper_gpu_support::runner::review_pair(&target)
                    .map(|review| (review, None, protected_worker_available()))
            } else {
                let approved = hyper_gpu_support::enrollment::PairReview {
                    vm_id: target.vm_id,
                    gpu_interface: target.gpu_interface,
                    expected: hyper_gpu_support::configuration_store::Revision::Missing,
                    operator_sid: hyper_gpu_support::setup::operator_sid()?,
                };
                approved.target()?;
                let artifacts = hyper_gpu_support::setup::source_artifacts()?;
                if let Some(record) = pending
                    && (record.review != approved || record.artifacts != artifacts)
                {
                    return Err("Interrupted first-install setup belongs to another original pair/package. Select its original VM/GPU and reviewed package; setup recovery cannot replace authority.".into());
                }
                Ok((approved, Some(artifacts), true))
            }
        },
        move |ui, result| match result {
            Ok((approved, artifacts, worker_available)) => {
                let setup = artifacts.is_some();
                let mut summary = format!(
                    "Allow this application to manage the selected GPU for this virtual machine:\n\nVirtual machine: {}\nGPU: {}\n\nThis saves permission only. GPU support and the VM's power state will not change. Your draft stays available; you will review GPU changes separately afterward. Permissions for other VMs are preserved.",
                    ui.get_vm_name(),
                    ui.get_gpu_options()
                        .row_data(usize::try_from(ui.get_gpu_index()).unwrap_or(usize::MAX))
                        .unwrap_or_default(),
                );
                if setup {
                    summary.push_str("\n\nThe application also needs to install or finish setting up its management components on this Windows host. Windows will ask for administrator approval. If setup was interrupted, the original VM, GPU and application package must still match.");
                } else {
                    summary.push_str("\n\nWindows approval is required to save this permission.");
                }
                let mut body = format!(
                    "VM ID: {}\nGPU interface: {}\nOperator: {}\nPolicy revision: {:?}\n\nAuthorize this exact VM/GPU pair through Windows elevation, preserving all other enrolled VMs. Enrollment does not enable GPU support, change guest power, prepare drivers or save desired configuration. Your draft is retained. After fresh authority readback, Apply obtains a separate GPU operation plan.",
                    approved.vm_id,
                    approved.gpu_interface,
                    approved.operator_sid,
                    approved.expected
                );
                if let Some(pins) = &artifacts {
                    body.push_str("\n\nFIRST INSTALL / INTERRUPTED SETUP: install the fixed product package into Program Files, create protected ProgramData state and register the fixed SYSTEM runner task for this operator. Interrupted setup retains its original pair and package pins. If authority is already published, recovery only validates it and completes task/audit registration; authority and artifact files are never overwritten. No VM disk, CPU/RAM or Secure Boot change. Reviewed source artifact SHA-256 pins:\n");
                    for (name, hash) in pins {
                        body.push_str(&format!("{name}: {hash}\n"));
                    }
                }
                if !with_state(ui, &reviewed_state, |state| {
                    state.enrollment_review = Some(approved);
                    state.setup_artifacts = artifacts;
                    Ok(())
                }) {
                    return;
                }
                ui.set_execution_ready(worker_available);
                ui.set_needs_shutdown(false);
                dialog(ui, "enroll-review", "Review VM/GPU enrollment", &summary);
                ui.set_dialog_details(body.into());
            }
            Err(error) => dialog(ui, "error", "Enrollment unavailable", &error),
        },
    )
}

#[cfg(windows)]
fn enroll_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
) -> Result<(), String> {
    let (approved, artifacts) = {
        let mut state = state.lock().map_err(|_| "Presentation state unavailable")?;
        let approved = state
            .enrollment_review
            .take()
            .ok_or("Review exact enrollment first")?;
        let draft = draft_target(ui, &state)?;
        if draft.vm_id != approved.vm_id || draft.gpu_interface != approved.gpu_interface {
            return Err("Pair changed after enrollment review; review again".into());
        }
        let artifacts = state.setup_artifacts.take();
        (approved, artifacts)
    };
    let finished_state = state.clone();
    let refresh_busy = busy.clone();
    let source = source.clone();
    let progress = progress_to_ui(ui.as_weak());
    background(
        ui,
        busy,
        "Enroll reviewed VM/GPU pair",
        move || {
            if let Some(artifacts) = artifacts {
                hyper_gpu_support::setup::install(approved, artifacts)
            } else {
                hyper_gpu_support::worker::enroll_pair(approved, progress).map(|_| ())
            }
        },
        move |ui, result| match result {
            Ok(()) => {
                if let Err(error) = refresh(ui, &finished_state, &refresh_busy, &source, false) {
                    dialog(ui, "error", "Authority refresh unavailable", &error);
                }
            }
            Err(error) => {
                if !with_state(ui, &finished_state, |state| {
                    state.view.needs_readback = true;
                    present(ui, state, false);
                    Ok(())
                }) {
                    return;
                }
                ui.set_eligible(false);
                ui.set_execution_ready(false);
                dialog(
                    ui,
                    "error",
                    "Enrollment did not finish",
                    &format!(
                        "{error}\nDraft retained. Refresh authority and inspect any enrollment recovery hold before retrying."
                    ),
                );
            }
        },
    )
}

#[cfg(windows)]
fn reconcile_enrollment_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
) -> Result<(), String> {
    let (operation_id, expected) = {
        let mut state = state.lock().map_err(|_| "Presentation state unavailable")?;
        let id = state
            .enrollment_recovery
            .as_ref()
            .ok_or("Refresh enrollment recovery first")?
            .operation_id
            .clone();
        let expected = state
            .enrollment_recovery_revision
            .take()
            .ok_or("Review authority readback first")?;
        (id, expected)
    };
    let finished_state = state.clone();
    let refresh_busy = busy.clone();
    let source = source.clone();
    let progress = progress_to_ui(ui.as_weak());
    background(
        ui,
        busy,
        "Reconcile enrollment authority",
        move || hyper_gpu_support::worker::reconcile_pair(operation_id, expected, progress),
        move |ui, result| match result {
            Ok(_) => {
                if let Err(error) = refresh(ui, &finished_state, &refresh_busy, &source, false) {
                    dialog(ui, "error", "Authority refresh unavailable", &error);
                }
            }
            Err(error) => {
                ui.set_execution_ready(false);
                dialog(
                    ui,
                    "error",
                    "Enrollment recovery retained",
                    &format!("{error}\nRefresh before reviewing authority again."),
                );
            }
        },
    )
}

#[cfg(windows)]
fn progress_to_ui(weak: slint::Weak<AppWindow>) -> impl FnMut(hyper_gpu_support::worker::Progress) {
    move |progress| {
        let _ = weak.upgrade_in_event_loop(move |ui| {
            let stage = match progress.stage.as_str() {
                "Persist operation recovery" => "Saving progress",
                "Prepare current signed payload" => "Installing the GPU driver files in the VM",
                "Compatibility settings" => "Updating GPU compatibility settings",
                "Provider allocation" => "Applying your GPU resource settings",
                "Graceful guest shutdown" => "Shutting down the VM safely",
                "Start guest" => "Starting the VM",
                "Publish checked VM configuration" => "Saving the checked VM settings",
                "Save verified configuration only" => "Retrying the bound configuration save",
                _ => progress.stage.as_str(),
            };
            let status = match progress.status {
                hyper_gpu_support::worker::StageStatus::Running => "In progress",
                hyper_gpu_support::worker::StageStatus::Done => "Complete",
                hyper_gpu_support::worker::StageStatus::Failed => "Failed",
            };
            // The information header is stable. Transfer counters replace the
            // current transfer row; lifecycle and terminal events stay distinct.
            ui.set_stages(append_progress(
                ui.get_stages(),
                format!("{stage} · {status}").into(),
            ));
        });
    }
}

#[cfg(any(windows, test))]
fn append_progress(
    stages: slint::ModelRc<slint::SharedString>,
    row: slint::SharedString,
) -> slint::ModelRc<slint::SharedString> {
    if let Some(vector) = stages
        .as_any()
        .downcast_ref::<slint::VecModel<slint::SharedString>>()
    {
        if let Some(index) = vector.row_count().checked_sub(1)
            && vector
                .row_data(index)
                .is_some_and(|previous| transfer_update(&previous, &row))
        {
            vector.set_row_data(index, row);
        } else {
            vector.push(row);
        }
        stages
    } else {
        let mut rows: Vec<_> = stages.iter().collect();
        if let Some(previous) = rows.last_mut()
            && transfer_update(previous, &row)
        {
            *previous = row;
        } else {
            rows.push(row);
        }
        model(rows)
    }
}

#[cfg(any(windows, test))]
fn transfer_update(previous: &str, current: &str) -> bool {
    // The worker formats numeric acknowledgements using this fixed prefix.
    // Never replace a failure, completion, or a different lifecycle stage.
    [previous, current].iter().all(|row| {
        row.starts_with(hyper_gpu_support::reporting::TRANSFER_PROGRESS_PREFIX)
            && row.ends_with("In progress")
    })
}

#[cfg(windows)]
fn apply_live(ui: &AppWindow, state: &SharedState, busy: &Arc<AtomicBool>) -> Result<(), String> {
    if !ui.get_execution_ready() {
        return Err("The reviewed protected worker is not available.".into());
    }
    let (approved, revision) = {
        let state = state.lock().map_err(|_| "Presentation state unavailable")?;
        let reviewed = state.reviewed_plan(&draft_target(ui, &state)?)?;
        (reviewed.plan, reviewed.revision)
    };
    let selected = approved.desired.clone();
    let finished_state = state.clone();
    let progress = progress_to_ui(ui.as_weak());
    let vm_name = approved.observed.name.clone();
    let result_vm_name = vm_name.clone();
    let setup_required = approved
        .preview
        .actions
        .contains(&hyper_gpu_support::workflow::PreviewAction::Verify);
    background_operation(
        ui,
        busy,
        "Apply approved changes",
        Some(&vm_name),
        setup_required,
        move || {
            let credential = if approved.preview.credentials_required {
                Some(
                    hyper_gpu_support::credentials::read(&approved.desired.vm_id)?.map_or_else(
                        || {
                            hyper_gpu_support::credentials::prompt_named(
                                &approved.desired.vm_id,
                                &approved.observed.name,
                                windows::Win32::Foundation::HWND::default(),
                            )
                        },
                        Ok,
                    )?,
                )
            } else {
                None
            };
            let shutdown = approved.preview.guest_downtime;
            hyper_gpu_support::worker::apply(approved, revision, shutdown, credential, progress)
        },
        move |ui, result, readback| match result {
            Ok(outcome) => {
                let mut published = outcome.saved;
                if !with_state(ui, &finished_state, |state| {
                    let status = state.finish_apply(
                        &selected,
                        &outcome.operation_id,
                        outcome.saved,
                        readback,
                    );
                    published = state.publication_confirmed(&selected);
                    ui.set_dirty(state.view.draft.is_some());
                    present(ui, state, !ui.get_dirty() && published);
                    status
                }) {
                    return;
                }
                if published {
                    dialog(
                        ui,
                        "info",
                        if setup_required {
                            "GPU setup/update passed"
                        } else {
                            "GPU change confirmed and configuration saved"
                        },
                        &if setup_required {
                            format!(
                                "{result_vm_name}: hardware rendering passed, graceful final shutdown was confirmed, and your settings were saved. The VM is off; start it through Hyper-V when ready."
                            )
                        } else {
                            format!(
                                "{result_vm_name}: GPU assignment, requested settings and final-off state were confirmed; your settings were saved. No guest graphics check was run. Start the VM through Hyper-V when ready."
                            )
                        },
                    );
                } else {
                    dialog(
                        ui,
                        "save",
                        if setup_required {
                            "GPU setup/update passed; settings not saved"
                        } else {
                            "GPU change confirmed; configuration not saved"
                        },
                        &format!(
                            "{}{}\nThe verified intent is retained separately from committed configuration. Retry saving only; do not repeat Apply.",
                            if setup_required {
                                format!(
                                    "{result_vm_name}: hardware rendering passed and final shutdown was confirmed. The VM is off.\n"
                                )
                            } else {
                                format!(
                                    "{result_vm_name}: GPU changes and final-off state were confirmed. No guest graphics check was run. The VM is off.\n"
                                )
                            },
                            outcome.save_error.unwrap_or_default(),
                        ),
                    );
                }
            }
            Err(error) => {
                if credential_request_cancelled(ui, &error, false) {
                    with_state(ui, &finished_state, |state| {
                        state.approved_plan = None;
                        Ok(())
                    });
                    return;
                }
                let stages: Vec<_> = ui.get_stages().iter().collect();
                if !with_state(ui, &finished_state, |state| {
                    state.view.needs_readback = true;
                    state.approved_plan = None;
                    let status = state.finish_readback(readback);
                    present(ui, state, false);
                    status
                }) {
                    return;
                }
                ui.set_stages(model(stages));
                let last_stage = ui.get_stages().iter().last().map(|stage| stage.to_string());
                dialog(
                    ui,
                    "error",
                    if setup_required {
                        "GPU setup/update did not complete"
                    } else {
                        "Operation did not complete"
                    },
                    &error,
                );
                let reason =
                    hyper_gpu_support::reporting::operator_error("Applying changes", &error);
                ui.set_dialog_body(format!(
                    "{reason}\n{}\nYour draft is retained. Changes may have started; the final VM state has not been confirmed. Use Refresh and inspect recovery before trying again. Do not repeat Apply while recovery is required.",
                    last_stage.map_or_else(|| "No execution stage was reported.".into(), |stage| format!("Last reported step: {stage}")),
                ).into());
            }
        },
    )
}
#[cfg(windows)]
fn save_only_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let record = state
        .lock()
        .map_err(|_| "Presentation state unavailable")?
        .save_only_record()?;
    let finished_state = state.clone();
    let target = record.target.clone();
    let progress = progress_to_ui(ui.as_weak());
    background_operation(
        ui,
        busy,
        "Retry configuration save only",
        None,
        false,
        move || hyper_gpu_support::worker::save_only(record.operation_id, progress),
        move |ui, result, readback| {
            let mut readback = Some(readback);
            if result.is_err()
                && !with_state(ui, &finished_state, |state| {
                    let status = state.finish_readback(
                        readback
                            .take()
                            .ok_or("Completion readback was already consumed")?,
                    );
                    present(ui, state, false);
                    status
                })
            {
                return;
            }
            match result {
                Ok(outcome) if outcome.saved => {
                    if !with_state(ui, &finished_state, |state| {
                        let status = state.finish_apply(
                            &target,
                            &outcome.operation_id,
                            true,
                            readback
                                .take()
                                .ok_or("Completion readback was already consumed")?,
                        );
                        ui.set_dirty(state.view.draft.is_some());
                        present(ui, state, !ui.get_dirty());
                        status
                    }) {
                        return;
                    }
                    dialog(
                        ui,
                        "info",
                        "Configuration saved",
                        "Protected publication passed readback. No GPU operation was repeated.",
                    );
                }
                Ok(_) => dialog(
                    ui,
                    "error",
                    "Configuration still unsaved",
                    "The protected receipt remains available for inspection. No GPU operation was repeated.",
                ),
                Err(error) => dialog(
                    ui,
                    "error",
                    "Save-only retry blocked",
                    &format!(
                        "{error}\nThe protected recovery record remains; no GPU operation was repeated."
                    ),
                ),
            }
        },
    )
}
#[cfg(windows)]
fn review_verification(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let target = {
        let state = state.lock().map_err(|_| "Presentation state unavailable")?;
        if state.view.draft.is_some() {
            return Err(
                "Discard or Apply the draft before reviewing a separate graphics check.".into(),
            );
        }
        state.editor_eligibility(ui.get_selected_id().as_str())?;
        state.target(ui.get_selected_id().as_str())?
    };
    let review_state = state.clone();
    background(
        ui,
        busy,
        "Reviewing graphics verification",
        move || {
            let value = hyper_gpu_support::runner::submit(hyper_gpu_support::runner::request(
                hyper_gpu_support::runner::Operation::VerifyPlan,
                Some(target),
                None,
            ))?;
            serde_json::from_value::<hyper_gpu_support::workflow::VerificationPlan>(value)
                .map_err(|e| e.to_string())
                .map(|plan| (plan, protected_worker_available()))
        },
        move |ui, result| match result {
            Ok((plan, worker_available)) => {
                ui.set_execution_ready(worker_available);
                ui.set_needs_shutdown(plan.observed.power == hyper_gpu_support::model::Power::Off);
                let body = format!(
                    "VM: {}\nVM ID: {}\nGPU: {}\nDriver: {}\nPayload digest: {}\n\nCompare current guest driver files and the preparation receipt with the authenticated host manifest, then check device health and hardware rendering using this VM's guest administrator credentials. {}\nVerify refreshes its protected guest helper executables, manifest metadata and optional diagnostic files. It does not reinstall driver payloads, rewrite the preparation receipt, or change GPU assignment, resources or saved settings. The worker independently rechecks the reviewed state before effects.",
                    plan.observed.name,
                    plan.target.vm_id,
                    plan.target.gpu_interface,
                    plan.gpu.driver_version,
                    plan.payload_digest,
                    if plan.observed.power == hyper_gpu_support::model::Power::Off {
                        "Temporarily start the stopped guest, then gracefully restore it to stopped."
                    } else {
                        "The guest remains running."
                    }
                );
                if !with_state(ui, &review_state, |state| {
                    state.verification_plan = Some(plan);
                    Ok(())
                }) {
                    return;
                }
                dialog(ui, "verify-review", "Review graphics verification", &body);
            }
            Err(error) => dialog(ui, "error", "Graphics verification blocked", &error),
        },
    )
}
#[cfg(windows)]
fn verify_live(ui: &AppWindow, state: &SharedState, busy: &Arc<AtomicBool>) -> Result<(), String> {
    let approved = state
        .lock()
        .map_err(|_| "Presentation state unavailable")?
        .verification_plan
        .clone()
        .ok_or("Review verification first")?;
    let shutdown = approved.observed.power == hyper_gpu_support::model::Power::Off;
    let progress = progress_to_ui(ui.as_weak());
    let finished_state = state.clone();
    let vm_name = approved.observed.name.clone();
    background_operation(
        ui,
        busy,
        "Verify approved graphics",
        Some(&vm_name),
        true,
        move || {
            let credential = hyper_gpu_support::credentials::read(&approved.target.vm_id)?
                .map_or_else(
                    || {
                        hyper_gpu_support::credentials::prompt_named(
                            &approved.target.vm_id,
                            &approved.observed.name,
                            windows::Win32::Foundation::HWND::default(),
                        )
                    },
                    Ok,
                )?;
            hyper_gpu_support::worker::verify(approved, shutdown, credential, progress)
        },
        move |ui, result, readback| {
            if let Err(error) = &result
                && credential_request_cancelled(ui, error, false)
            {
                with_state(ui, &finished_state, |state| {
                    state.verification_plan = None;
                    Ok(())
                });
                return;
            }
            if !with_state(ui, &finished_state, |state| {
                state.verification_plan = None;
                state.view.needs_readback = true;
                let status = state.finish_readback(readback);
                present(ui, state, false);
                status
            }) {
                return;
            }
            match result {
                Ok(outcome) if outcome.finished => dialog(
                    ui,
                    "info",
                    "Guest verification passed",
                    "Current guest driver files and preparation receipt matched the authenticated host manifest; device health and hardware rendering passed. Disclosed guest power was restored. Refresh to read the new protected verification record.",
                ),
                Ok(_) => dialog(
                    ui,
                    "error",
                    "Graphics outcome unresolved",
                    "Inspect the retained recovery record and Refresh.",
                ),
                Err(error) => dialog(
                    ui,
                    "error",
                    "Graphics verification did not complete",
                    &format!("{error}\nRefresh and inspect recovery before another operation."),
                ),
            }
        },
    )
}
#[cfg(windows)]
fn reconcile_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let record = state
        .lock()
        .map_err(|_| "Presentation state unavailable")?
        .recovery
        .clone()
        .ok_or("No reviewed operation recovery is loaded")?;
    let progress = progress_to_ui(ui.as_weak());
    let finished_state = state.clone();
    let target = record.target.clone();
    let publication = record.publication_required;
    let (vm_name, restored_candidate) = {
        let s = state.lock().map_err(|_| "Presentation state unavailable")?;
        let vm = s
            .view
            .vm(&target.vm_id)
            .ok_or("Recovery VM unavailable; Refresh first")?;
        let journal = s
            .view
            .inventory
            .as_ref()
            .and_then(|i| i.managed.get(&target.vm_id))
            .and_then(Option::as_ref);
        let driver = s
            .view
            .inventory
            .as_ref()
            .and_then(|i| {
                i.discovery
                    .gpus
                    .iter()
                    .find(|g| g.interface == target.gpu_interface)
            })
            .map(|g| g.driver_version.as_str())
            .unwrap_or_default();
        (
            vm.name.clone(),
            record
                .validate_restored_baseline(
                    vm,
                    driver,
                    &s.committed.revision(&target.vm_id)?,
                    journal,
                )
                .is_ok()
                || record
                    .negative_completion(vm, driver, &s.committed.revision(&target.vm_id)?, journal)
                    .is_ok_and(|candidate| candidate.is_some()),
        )
    };
    let prompt_name = vm_name.clone();
    background_operation(
        ui,
        busy,
        "Reconcile observed outcome",
        Some(&vm_name),
        record.requires_graphics() && !restored_candidate,
        move || {
            let graphics = record.requires_graphics();
            let credential = if graphics && !restored_candidate {
                Some(
                    hyper_gpu_support::credentials::read(&record.target.vm_id)?.map_or_else(
                        || {
                            hyper_gpu_support::credentials::prompt_named(
                                &record.target.vm_id,
                                &prompt_name,
                                windows::Win32::Foundation::HWND::default(),
                            )
                        },
                        Ok,
                    )?,
                )
            } else {
                None
            };
            hyper_gpu_support::worker::reconcile(
                record.operation_id,
                graphics,
                credential,
                progress,
            )
        },
        move |ui, result, readback| {
            if let Err(error) = &result
                && credential_request_cancelled(ui, error, true)
            {
                return;
            }
            if !with_state(ui, &finished_state, |state| {
                let status = if let Ok(outcome) = &result
                    && publication
                    && outcome.operation.is_some()
                {
                    state.finish_apply(&target, &outcome.operation_id, outcome.saved, readback)
                } else {
                    state.finish_readback(readback)
                };
                ui.set_dirty(state.view.draft.is_some());
                present(ui, state, !ui.get_dirty());
                status
            }) {
                return;
            }
            match result {
                Ok(outcome) if outcome.finished && outcome.operation.is_none() => dialog(
                    ui,
                    "info",
                    "Interrupted attempt ended",
                    "Fresh checks established the recorded host state and safely ended the attempt without repeating GPU work or saving requested settings. Any failed or unknown guest check remains recorded; ending recovery does not certify guest setup or rendering. Refresh, then review repair or another supported operation.",
                ),
                Ok(outcome) if outcome.finished => dialog(
                    ui,
                    "info",
                    "Recorded outcome reconciled",
                    "Fresh checks established the reviewed outcome and any required configuration publication. No GPU assignment, allocation or preparation was repeated. Refresh before another operation.",
                ),
                Ok(outcome) => dialog(
                    ui,
                    "save",
                    "Outcome verified; configuration unsaved",
                    &format!(
                        "{}\nRetry configuration saving only.",
                        outcome.save_error.unwrap_or_default()
                    ),
                ),
                Err(error) => {
                    dialog(ui, "error", "Reconciliation remains blocked", &error);
                    ui.set_dialog_body(format!(
                        "{}\nThe recovery hold remains. Refresh and have an administrator reconcile the VM state before applying new changes.",
                        hyper_gpu_support::reporting::operator_error("Reconciliation", &error),
                    ).into());
                }
            }
        },
    )
}
#[cfg(not(windows))]
fn review_verification(_: &AppWindow, _: &SharedState, _: &Arc<AtomicBool>) -> Result<(), String> {
    Err("Graphics verification requires Windows".into())
}
#[cfg(not(windows))]
fn verify_live(_: &AppWindow, _: &SharedState, _: &Arc<AtomicBool>) -> Result<(), String> {
    Err("Graphics verification requires Windows".into())
}
#[cfg(not(windows))]
fn reconcile_live(_: &AppWindow, _: &SharedState, _: &Arc<AtomicBool>) -> Result<(), String> {
    Err("Reconciliation requires Windows".into())
}
#[cfg(not(windows))]
fn apply_live(_: &AppWindow, _: &SharedState, _: &Arc<AtomicBool>) -> Result<(), String> {
    Err("GPU operations require Windows".into())
}
#[cfg(not(windows))]
fn save_only_live(_: &AppWindow, _: &SharedState, _: &Arc<AtomicBool>) -> Result<(), String> {
    Err("Protected publication requires Windows".into())
}

pub(super) fn run(ui: AppWindow, source: Source) -> Result<(), Box<dyn std::error::Error>> {
    ui.set_vms(model(Vec::new()));
    ui.set_selected_id("".into());
    ui.set_gpu_information(model(Vec::new()));
    ui.set_gpu_options(model(Vec::new()));
    ui.set_values(model(vec!["".into(); 12]));
    ui.set_eligible(false);
    ui.set_system_summary("Virtual machine information has not been read yet.".into());
    let busy = Arc::new(AtomicBool::new(false));
    let state = Arc::new(Mutex::new(State::default()));
    let weak = ui.as_weak();
    let selection_state = state.clone();
    ui.on_select_vm(move |id| {
        if let Some(ui) = weak.upgrade() { if ui.get_presenting() { return; } with_state(&ui, &selection_state, |state| {
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
            if ui.get_presenting() {
                return;
            }
            with_state(&ui, &edit_state, |state| {
                if let Some(index) = state.fixed_gpu_index()
                    && ui.get_gpu_index() != index
                {
                    ui.set_gpu_index(index);
                    ui.set_notice("The recorded GPU pairing is fixed. GPU replacement is unavailable; your edits are kept.".into());
                    return Ok(());
                }
                state.approved_plan = None;
                state.enrollment_review = None;
                state.setup_artifacts = None;
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
                        "Unapplied edits for this VM; validate and review before effects."
                    } else {
                        "No unapplied edits"
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
                        ui.set_validation("".into());
                    }
                    Err(error) => {
                        // A marker retains the VM even for invalid text; raw edits stay in the UI.
                        if changed {
                            state.view.draft = Some(baseline);
                        }
                        ui.set_validation(
                            hyper_gpu_support::reporting::operator_error(
                                "Configuration validation",
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
            .map_err(|_| "Presentation state unavailable".to_owned())
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
    let weak = ui.as_weak();
    let action_state = state.clone();
    let action_busy = busy.clone();
    let action_source = source.clone();
    ui.on_action(move |action| {
        let Some(ui) = weak.upgrade() else { return; }; if ui.get_running() { return; }
        if action_busy.load(Ordering::Acquire)
            && matches!(action.as_str(), "refresh" | "review" | "reapply" | "repair" | "remove-gpu" | "forget" | "confirm-forget" | "verify" | "confirm" | "confirm-enroll" | "confirm-verify" | "reconcile" | "reconcile-enrollment" | "save-only" | "credentials")
        {
            ui.set_notice("VM information is updating. You can keep editing; try this action when the update finishes.".into());
            return;
        }
        if matches!(action.as_str(), "verify" | "repair" | "remove-gpu" | "forget") {
            let message = action_state.lock()
                .map_err(|_| "Live presentation state unavailable".to_owned())
                .map(|state| state.action_recovery_message());
            match message {
                Ok(Some(body)) => {
                    dialog(&ui, "recovery-needed", "Review the unfinished operation first", &body);
                    return;
                }
                Err(error) => {
                    dialog(&ui, "error", "Cannot read operation state", &error);
                    return;
                }
                Ok(None) => {}
            }
            if !ui.get_execution_ready() {
                dialog(&ui, "info", "Action unavailable", "Use Refresh to check application setup and worker availability. Your edits are kept.");
                return;
            }
            let availability = action_state.lock()
                .map_err(|_| "Live presentation state unavailable".to_owned())
                .map(|state| {
                    let values = ui.get_values().iter().map(|value| value.to_string()).collect::<Vec<_>>();
                    state.selected_actions(ui.get_gpu_index(), &values)
                        .reason(action.as_str()).unwrap_or_default().to_owned()
                });
            match availability {
                Ok(reason) if !reason.is_empty() => {
                    dialog(&ui, "info", "Action unavailable", &reason);
                    return;
                }
                Err(error) => {
                    dialog(&ui, "error", "Cannot read action availability", &error);
                    return;
                }
                _ => {}
            }
        }
        let result = match action.as_str() {
            "refresh" => refresh(&ui, &action_state, &action_busy, &action_source, false),
            #[cfg(windows)]
            "forget" => review_forget_live(&ui, &action_state, &action_busy),
            #[cfg(windows)]
            "confirm-forget" if ui.get_dialog_kind() == "forget-review" => forget_live(&ui, &action_state, &action_busy),
            "review" => review(&ui, &action_state, &action_busy, &action_source),
            #[cfg(windows)]
            "confirm-enroll" => enroll_live(&ui, &action_state, &action_busy, &action_source),
            #[cfg(windows)]
            "reconcile-enrollment" => reconcile_enrollment_live(&ui, &action_state, &action_busy, &action_source),
            "confirm" if ui.get_dialog_kind() == "review" => apply_live(&ui, &action_state, &action_busy),
            "confirm" => Err("Open a fresh review before Apply.".into()),
            "save-only" => save_only_live(&ui, &action_state, &action_busy),
            "verify" => review_verification(&ui, &action_state, &action_busy),
            "confirm-verify" => verify_live(&ui, &action_state, &action_busy),
            "reconcile" => reconcile_live(&ui, &action_state, &action_busy),
            "reapply" => {
                // Preserve allocation edits while deliberately previewing enabled support.
                ui.set_gpu_enabled(true);
                ui.invoke_edited();
                review(&ui, &action_state, &action_busy, &action_source)
            }
            "repair" => {
                // Reuse the enabled draft; reinstall intent belongs only to this review.
                ui.set_gpu_enabled(true);
                ui.invoke_edited();
                review_operation(&ui, &action_state, &action_busy, &action_source, true)
            }
            "remove-gpu" => {
                let removal = action_state.lock().map_err(|_| "Presentation state unavailable".to_owned())
                    .and_then(|state| state.removal_draft(
                        ui.get_selected_id().as_str(), ui.get_gpu_index(),
                        &ui.get_values().iter().map(|v| v.to_string()).collect::<Vec<_>>(),
                    ));
                removal.and_then(|_| {
                    ui.set_gpu_enabled(false);
                    ui.invoke_edited();
                    review(&ui, &action_state, &action_busy, &action_source)
                })
            }
            "close" => {
                with_state(&ui, &action_state, |state| {
                    state.cancel_review();
                    ui.set_dialog_kind("".into());
                    Ok(())
                });
                Ok(())
            },
            "exit" => { ui.set_dirty(false); slint::quit_event_loop().map_err(|e| e.to_string()) },
            "discard" | "switch" => {
                with_state(&ui, &action_state, |state| {
                    state.discard();
                    if action == "switch" { state.view.selected = state.pending_selection.take(); }
                    present(&ui, state, true); ui.set_dirty(false); ui.set_dialog_kind("".into());
                    ui.set_notice("Draft discarded. No configuration was saved or VM state changed.".into()); Ok(())
                }); Ok(())
            }
            "more" => {
                if !with_state(&ui, &action_state, |_| Ok(())) { return; }
                let body = format!("Virtual machine: {}\n\nChoose an action to open its review. Unavailable actions explain what to do first. Your edits are kept until you explicitly discard or apply them.", ui.get_vm_name());
                dialog(&ui, "more", "VM actions", &body); Ok(())
            },
            "credentials" => {
                let selected = ui.get_selected_id().to_string();
                let vm_name = ui.get_vm_name().to_string();
                let prompt_name = vm_name.clone();
                let credential_source = action_source.clone();
                action_state.lock().map_err(|_| "Live presentation state unavailable".to_owned()).and_then(|s| s.target(&selected))
                    .and_then(|target| background_for_vm(&ui, &action_busy, "Guest credentials", Some(&vm_name), true, move || credential_source.credentials(&target.vm_id, &prompt_name), |ui, result| match result {
                        Ok(true) => dialog(ui, "info", "Stored guest credentials available", "A stored credential was confirmed in this user's Windows Credential Manager for the selected VM. No guest connection or graphics check was performed."),
                        Ok(false) => dialog(ui, "info", "Guest credentials not retained", "No stored credential is available. Apply or Verify will prompt locally when credentials are needed. Select Remember to store them in Windows Credential Manager."),
                        Err(error) => {
                            if !credential_request_cancelled(ui, &error, false) {
                                dialog(ui, "error", "Guest credentials unavailable", &error);
                            }
                        },
                    }))
            }
            "diagnostics" | "recovery" => {
                with_state(&ui, &action_state, |state| {
                    #[cfg(windows)]
                    if action == "recovery"
                        && let Some(record) = &state.enrollment_recovery {
                        let revision = hyper_gpu_support::runner::pair_revision()?;
                        let known = revision == record.resulting || revision == record.review.expected;
                        if !known { return Err("Authority does not match either recorded revision. Recovery remains held for administrator inspection.".into()); }
                        let outcome = if record.removal.is_some() {
                            "Profile removal was interrupted. Confirm finishes only remaining reviewed record deletions and authority removal after fresh inactive-VM/revision checks. Completed deletions must remain absent; changed or unrelated records stay blocked. Other VMs, guest files and shared artifacts are retained. No guest checks, credentials or VM/GPU effects run."
                        } else if revision == record.resulting {
                            "The complete new enrollment policy is present. Confirm independently checks this reviewed authority and ends the hold; no enrollment write or GPU work is replayed."
                        } else {
                            "The original policy is present; the pair was not added. Confirm independently checks this reviewed authority and ends the hold; no enrollment write or GPU work is replayed."
                        };
                        let body = format!("Operation: {}\nVM ID: {}\nGPU: {}\n\n{outcome}", record.operation_id, record.review.vm_id, record.review.gpu_interface);
                        state.enrollment_recovery_revision = Some(revision);
                        ui.set_needs_shutdown(false);
                        dialog(&ui, "enrollment-recovery", if record.removal.is_some() { "Review interrupted profile removal" } else { "Review interrupted enrollment" }, &body);
                        return Ok(());
                    }
                    if action == "recovery" && let Some(record) = &state.import_recovery {
                        dialog(&ui, "info", "Configuration import requires reconciliation", &format!("Import operation: {}\n{} destinations, {} recorded as published. Resolve this explicit CLI import with import-reconcile --operation {}. Changed files block continuation; import never changes enrollment or GPU state.", record.operation_id, record.documents.len(), record.documents.values().filter(|item| item.published).count(), record.operation_id));
                        return Ok(());
                    }
                    if action == "recovery"
                        && let Some(record) = &state.recovery {
                        ui.set_needs_shutdown(record.requires_graphics());
                        match record.phase {
                            hyper_gpu_support::configuration_store::SavePhase::Verified(_) | hyper_gpu_support::configuration_store::SavePhase::Published(_) => {
                                if record.publication_required {
                                    dialog(&ui, "save", "Verified operation awaits configuration publication", "GPU effects already passed independent readback. Retry saves only the bound configuration; it will not repeat GPU modification. Changed configuration, driver, payload or effective state blocks this retry.");
                                } else {
                                    dialog(&ui, "recovery", "Reconcile interrupted graphics check", "Check the recorded pair and fresh graphics outcome; restore only the originally approved guest power if safe. No GPU assignment, allocation, preparation or configuration publication is performed.");
                                }
                            }
                            hyper_gpu_support::configuration_store::SavePhase::Admitted | hyper_gpu_support::configuration_store::SavePhase::EffectsStarted => {
                                let name = state.view.vm(&record.target.vm_id).map_or(record.initial.name.as_str(), |vm| vm.name.as_str());
                                if !record.requires_graphics() {
                                    dialog(&ui, "recovery", "Review interrupted GPU change", &format!("Virtual machine: {name}\n\nCheck the already-achieved GPU assignment, settings and final-off state without repeating changes or accessing the guest. Partial or changed state retains recovery. No guest boot, credential prompt or graphics check is required."));
                                } else {
                                    dialog(&ui, "recovery", "Review interrupted operation", &format!("Virtual machine: {name}\n\nCheck the interrupted GPU operation without repeating it. A terminal failed guest check can end recovery after fresh reads confirm its original host lifecycle was restored; its failure remains recorded. A setup/reinstall attempt that never entered preparation can end after fresh reads confirm unchanged starting host state. These exits require no new guest check or credentials and do not certify setup. If fresh checks prove that this first setup attempt is back at its original stopped, unattached host configuration, end that attempt without saving its requested changes. Guest files may remain and will not count as verified preparation.\n\nOtherwise, check the already-achieved approved outcome. This may temporarily start the VM and require its guest administrator username and password. Stop it safely afterward only with your approval. Partial or changed state stays blocked; no GPU setup work is replayed."));
                                }
                                ui.set_dialog_details(format!("Operation: {}\nVM ID: {}\nGPU: {}\nOriginal guest power: {:?}\nReviewed final guest power: {:?}", record.operation_id, record.target.vm_id, record.target.gpu_interface, record.initial.power, record.final_power()?).into());
                            }
                        }
                        return Ok(());
                    }
                    let id = ui.get_selected_id(); let inventory = state.view.inventory.as_ref().ok_or("Refresh inventory first.")?;
                    let body = serde_json::to_string_pretty(&serde_json::json!({"observed": state.view.vm(&id), "managed": inventory.managed.get(id.as_str()).and_then(Option::as_ref), "historical": state.view.needs_readback, "enrollment": inventory.enrolled.iter().find(|t| t.vm_id == id.as_str())})).map_err(|e| e.to_string())?;
                    let provenance = "Last protected inventory read; no fresh guest verification. Refresh obtains current host state. Pending journals are retained and are not cleared by inspection.";
                    dialog(&ui, "info", "Observed state and recovery records", &format!("{provenance}\n\n{body}")); Ok(())
                }); Ok(())
            }
            "shortcuts" => {
                let help = "Select a real VM and physical GPU, then enable GPU support in the draft and choose a VRAM request from 1 GB to the card's detected capacity. VRAM requests use bytes without a metadata-derived 1 GB ceiling; hardware behavior awaits live validation. Use the toggle or More actions > Remove GPU support to disable support. Advanced Allocation retains exact VRAM triples; saved Compute/Encode/Decode requests are preserved but are not edited here. Apply first reviews exact enrollment and initial package setup when needed. After enrollment, Apply separately reviews GPU changes; confirmation requests elevation and any guest downtime consent. New publication uses schema 3; schema-2 input remains readable. Failed saving permits save-only retry; uncertain operations require explicit reconciliation. Settings opens the native credential dialog. GPU replacement remains unavailable. Delete profile removes only app records, requires an inactive profile and separate confirmation, and never changes VM power or GPU support.";
                dialog(&ui, "info", "Application help", help); Ok(())
            },
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
    refresh(&ui, &state, &busy, &source, false)?;
    // Only the installed read path polls. First-install elevation remains explicit.
    let status_timer = slint::Timer::default();
    {
        let weak = ui.as_weak();
        status_timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_secs(15),
            move || {
                if let Some(ui) = weak.upgrade()
                    && !busy.load(Ordering::Acquire)
                    && ui.get_dialog_kind().is_empty()
                    && let Err(error) = refresh(&ui, &state, &busy, &source, true)
                {
                    ui.set_notice(
                        format!("VM status refresh unavailable: {error}. Use Refresh to retry.")
                            .into(),
                    );
                }
            },
        );
    }
    ui.run()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper_gpu_support::model::Settings;

    #[test]
    fn background_failure_or_panic_never_becomes_a_completed_operation() {
        assert_eq!(run_background_task(|| Ok(7)).unwrap(), 7);
        assert_eq!(
            run_background_task::<()>(|| Err("read failed".into())).unwrap_err(),
            "read failed"
        );
        let error = run_background_task::<()>(|| panic!("fixture panic payload")).unwrap_err();
        assert!(error.contains("without a terminal outcome"));
        assert!(!error.contains("fixture panic payload"));
    }

    #[test]
    fn progress_retains_each_event_in_order_without_rebuilding_the_existing_model() {
        let rows = model::<slint::SharedString>(vec![]);
        let original = rows.clone();
        let rows = append_progress(rows, "Repeated stage: in progress".into());
        let rows = append_progress(rows, "Repeated stage: in progress".into());
        let rows = append_progress(rows, "Stage failed".into());
        assert_eq!(original.row_count(), 3);
        assert_eq!(
            rows.iter().map(|row| row.to_string()).collect::<Vec<_>>(),
            [
                "Repeated stage: in progress",
                "Repeated stage: in progress",
                "Stage failed",
            ]
        );
    }

    #[test]
    fn transfer_counters_update_in_place_and_preserve_stage_boundaries() {
        let rows = model::<slint::SharedString>(vec![]);
        let original = rows.clone();
        let rows = append_progress(rows, "Preparing files - In progress".into());
        let rows = append_progress(
            rows,
            "Copying files: 1.0 MB of 10.0 MB (10%); 0 of 2 files; verification pending - In progress".into(),
        );
        let rows = append_progress(
            rows,
            "Copying files: 8.0 MB of 10.0 MB (80%); 1 of 2 files; verification pending - In progress".into(),
        );
        assert_eq!(original.row_count(), 2);
        assert_eq!(
            original.row_data(1).unwrap(),
            "Copying files: 8.0 MB of 10.0 MB (80%); 1 of 2 files; verification pending - In progress"
        );
        let rows = append_progress(rows, "Copying files: interrupted - Failed".into());
        let rows = append_progress(
            rows,
            "Copying files: 9.0 MB of 10.0 MB (90%); 1 of 2 files; verification pending - In progress".into(),
        );
        assert_eq!(rows.row_count(), 4);
        assert_eq!(
            rows.row_data(2).unwrap(),
            "Copying files: interrupted - Failed"
        );
    }

    #[test]
    fn unknown_health_enrollment_and_unresolved_attachment_are_not_samples() {
        let vm = VmState {
            compute: None,
            encode: None,
            decode: None,
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
                issues: vec![],
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

    #[test]
    fn historical_rows_keep_identity_and_do_not_claim_current_power() {
        let vm = VmState {
            compute: None,
            encode: None,
            decode: None,
            vm_id: "12345678-1234-1234-1234-123456789abc".into(),
            name: "Actual VM".into(),
            power: Power::Running,
            generation: 2,
            gpus: vec![],
            vram: None,
            settings: Settings {
                low_mmio: 0,
                high_mmio: 0,
                cache_types: false,
                automatic_checkpoints: true,
            },
        };
        let target = Target {
            compute: None,
            encode: None,
            decode: None,
            vm_id: vm.vm_id.clone(),
            gpu_interface: "actual-interface".into(),
            enabled: true,
            vram: None,
        };
        let mut inventory = Inventory {
            discovery: Discovery {
                issues: vec![],
                vms: vec![vm.clone()],
                gpus: vec![],
            },
            managed: Default::default(),
            enrolled: vec![target.clone()],
        };
        let current = observed_row(&vm, &inventory, false);
        assert_eq!(current.power, "Running");
        assert_eq!(current.status, "Authorized");
        let historical = observed_row(&vm, &inventory, true);
        assert_eq!(historical.id, current.id);
        assert_eq!(historical.name, current.name);
        assert_eq!(historical.power, "Status unavailable");
        assert_eq!(historical.status, "Refresh needed");
        assert_eq!(historical.gpu, "Status unavailable");
        inventory.managed.insert(
            vm.vm_id.clone(),
            Some(hyper_gpu_support::workflow::Journal {
                schema: 1,
                vm_id: vm.vm_id.clone(),
                gpu_interface: target.gpu_interface,
                original: vm.settings.clone(),
                applied: vm.settings.clone(),
                prepared: None,
                pending: true,
                restore_power: Power::Off,
                verification_only: false,
                last_verified: None,
                latest_check: None,
                preparation_started: None,
                check_restored: false,
                settings_released: false,
            }),
        );
        assert_eq!(
            observed_row(&vm, &inventory, false).status,
            "Recovery required"
        );
        assert!(inventory.managed[&vm.vm_id].as_ref().unwrap().pending);
    }
}
