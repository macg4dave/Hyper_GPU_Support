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
type InventoryRead = (Inventory, Option<String>, Option<Vec<u8>>);

#[derive(Clone)]
pub(super) enum Source {
    Protected,
    Snapshot(std::path::PathBuf, Option<std::path::PathBuf>),
}

impl Source {
    fn import_recovery(
        &self,
    ) -> Result<Option<hyper_gpu_support::configuration_store::ImportRecord>, String> {
        match self {
            #[cfg(windows)]
            Self::Protected => hyper_gpu_support::runner::import_record(),
            _ => Ok(None),
        }
    }
    fn recovery(
        &self,
    ) -> Result<Option<hyper_gpu_support::configuration_store::SaveRecord>, String> {
        match self {
            #[cfg(windows)]
            Self::Protected => hyper_gpu_support::runner::recovery_record(),
            _ => Ok(None),
        }
    }
    fn committed(&self) -> Option<hyper_gpu_support::configuration_store::StoreSnapshot> {
        match self {
            #[cfg(windows)]
            Self::Protected => Some(hyper_gpu_support::configuration_store::read_committed()),
            _ => None,
        }
    }
    fn historical(&self) -> bool {
        matches!(self, Self::Snapshot(..))
    }

    fn discover(&self) -> Result<InventoryRead, String> {
        match self {
            Self::Protected => discover().map(|(i, w)| (i, w, None)),
            Self::Snapshot(path, _) => super::snapshot::capture(path).map(|(inventory, bytes)| {
                (inventory, Some("Historical snapshot loaded. Drafts and shared plan rehearsal use recorded inputs only. No runner, credential access or persistent writes.".into()), Some(bytes))
            }),
        }
    }

    fn plan(&self, target: Target, expected_snapshot: Option<&[u8]>) -> Result<Plan, String> {
        match self {
            Self::Protected => plan(target),
            Self::Snapshot(path, _) => super::snapshot::plan(path, &target, expected_snapshot),
        }
    }

    fn credentials(&self, id: &str) -> Result<bool, String> {
        match self {
            Self::Protected => credential_dialog(id),
            Self::Snapshot(..) => Err(
                "Credential access and storage are blocked in no-write snapshot rehearsal.".into(),
            ),
        }
    }
}

impl Source {
    fn configuration(
        &self,
    ) -> Result<Option<(hyper_gpu_support::model::Configuration, String)>, String> {
        match self {
            Self::Snapshot(_, Some(path)) => {
                hyper_gpu_support::model::Configuration::read_vm_file(path).map(Some)
            }
            _ => Ok(None),
        }
    }
}

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

fn observed_row(vm: &VmState, inventory: &Inventory, historical: bool) -> VmRow {
    let mut row = vm_row(vm, &inventory.discovery);
    if vm.generation == 2 && inventory.enrolled.iter().any(|t| t.vm_id == vm.vm_id) {
        row.status = if inventory.recorded_state(&vm.vm_id).recovery_required {
            "Recovery required"
        } else {
            "Enrolled"
        }
        .into();
    }
    if historical {
        row.power = format!("{} (historical)", row.power).into();
        row.status = format!("{} (historical)", row.status).into();
        row.gpu = format!("{} (historical)", row.gpu).into();
    }
    row
}

fn present(ui: &AppWindow, state: &State, reset_draft: bool) {
    let Some(id) = state.view.selected.as_ref() else {
        ui.set_selected_id("".into());
        ui.set_eligible(false);
        ui.set_recovery(state.recovery.is_some() || state.import_recovery.is_some());
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
        ui.set_observed("Selected VM was not returned by the latest inventory read".into());
        ui.set_verification("Unknown; the selected VM is unavailable".into());
        ui.set_recovery(state.recovery.is_some() || state.import_recovery.is_some());
        ui.set_validation(
            "The draft VM is no longer present. Discard the draft and Refresh.".into(),
        );
        return;
    };
    let Some(inventory) = state.view.inventory.as_ref() else {
        return;
    };
    let row = observed_row(vm, inventory, state.view.needs_readback || state.historical);
    ui.set_vm_name(row.name);
    ui.set_power(row.power);
    let enrolled = state.target(id).ok();
    ui.set_status(row.status);
    let recorded = inventory.recorded_state(id);
    ui.set_recovery(
        recorded.recovery_required || state.recovery.is_some() || state.import_recovery.is_some(),
    );
    ui.set_observed(
        format!(
            "{}{}; {}{}; {}",
            if state.view.needs_readback || state.historical {
                "Historical: "
            } else {
                ""
            },
            row.gpu,
            hyper_gpu_support::gui_model::allocation_text(vm),
            vm.vram.as_ref().map_or(String::new(), |v| format!(
                ": {} / {} / {}",
                v.minimum, v.optimal, v.maximum
            )),
            recorded.preparation,
        )
        .into(),
    );
    ui.set_desired(
        if state.historical && state.configuration_source.is_some() {
            format!(
                "Candidate file (read-only rehearsal): {}",
                state.view.saved_desired_text(id)
            )
        } else {
            state.view.saved_desired_text(id)
        }
        .into(),
    );
    ui.set_verification(recorded.graphics.into());
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
        if title == "Apply approved changes"
            || title == "Verify approved graphics"
            || title == "Reconcile observed outcome"
        {
            "Requesting Windows elevation before the approved operation. Actual stages will appear as the shared backend executes."
        } else if title == "Retry configuration save only" {
            "Requesting Windows elevation to save the verified configuration. No GPU operation will be repeated."
        } else {
            "Waiting for the shared backend. No VM or guest changes are requested by this action."
        },
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

fn refresh(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
) -> Result<(), String> {
    let historical = source.historical();
    ui.set_notice(if historical {
        "Reading a historical inventory snapshot; no backend operations will be requested."
    } else {
        "Reading protected inventory. Existing observations remain historical until refresh succeeds."
    }.into());
    let state = state.clone();
    let source = source.clone();
    background(
        ui,
        busy,
        "Refreshing inventory",
        move || {
            let (inventory, warning, snapshot) = source.discover()?;
            Ok((
                inventory,
                warning,
                source.configuration()?,
                snapshot,
                source.committed(),
                source.recovery()?,
                source.import_recovery()?,
            ))
        },
        move |ui, result| {
            with_state(ui, &state, |state| {
                match result {
                    Ok((
                        inventory,
                        warning,
                        configuration,
                        snapshot,
                        committed,
                        recovery,
                        import_recovery,
                    )) => {
                        #[cfg(windows)]
                        ui.set_execution_ready(
                            !historical && hyper_gpu_support::worker::available().is_ok(),
                        );
                        state.recovery = recovery;
                        state.import_recovery = import_recovery;
                        if let Some(record) = &state.recovery {
                            state.view.unsaved = record.publication_required
                                && matches!(
                                    record.phase,
                                    hyper_gpu_support::configuration_store::SavePhase::Verified(_)
                                );
                            state.view.verified_unsaved =
                                state.view.unsaved.then(|| record.target.clone());
                        }
                        state.rehearsal_plan = None;
                        state.verification_plan = None;
                        if state.view.draft.is_some() && snapshot != state.snapshot_source {
                            state.view.needs_readback = true;
                            present(ui, state, false);
                            ui.set_eligible(false);
                            ui.set_validation("Snapshot changed externally. Draft preserved; discard and Refresh.".into());
                            return Err("Snapshot changed externally. Draft preserved; discard and Refresh.".into());
                        }
                        if let Some((configuration, text)) = configuration
                            && let Err(error) = state.load_configuration(configuration, text)
                        {
                            state.view.needs_readback = true;
                            present(ui, state, false);
                            ui.set_eligible(false);
                            ui.set_validation(error.clone().into());
                            return Err(error);
                        }
                        if let Some(committed) = committed
                            && let Err(error) = state.load_committed(committed)
                        {
                            state.view.needs_readback = true;
                            present(ui, state, false);
                            ui.set_eligible(false);
                            ui.set_validation(error.clone().into());
                            return Err(error);
                        }
                        let rows = inventory
                            .discovery
                            .vms
                            .iter()
                            .map(|vm| observed_row(vm, &inventory, historical))
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
                        state.snapshot_source = snapshot;
                        // Imported enrollment is display data, never execution authorization.
                        state.historical = historical;
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
                        ui.set_system_summary(if historical {
                            "Historical inventory snapshot. Provider bounds, enrollment and records describe its capture, not current machine state. No runner, credentials, journal or audit writes are requested."
                        } else {
                            "Actual Hyper-V inventory, protected enrollment and GUID-keyed committed configuration read. Graphics results remain historical."
                        }.into());
                        ui.set_notice(warning.unwrap_or_else(|| "Protected inventory loaded. Enrolled pairs support drafts and fresh plan previews. Apply and guest verification await worker integration.".into()).into());
                        Ok(())
                    }
                    Err(error) => {
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
                        ui.set_eligible(false);
                        ui.set_validation(
                            "Inventory is historical; Refresh before further actions.".into(),
                        );
                        ui.set_notice(format!("{} inventory unavailable: {error}. Existing rows are historical; Refresh retries the read.", if historical { "Snapshot" } else { "Live" }).into());
                        ui.set_system_summary(
                            format!(
                                "{} unavailable: {error}",
                                if historical {
                                    "Snapshot read"
                                } else {
                                    "Live discovery"
                                }
                            )
                            .into(),
                        );
                        Err(error)
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
    let (target, configuration_source, snapshot_source, committed_revision) = state
        .lock()
        .map_err(|_| "Live presentation state unavailable".to_owned())
        .and_then(|mut state| {
            state.rehearsal_plan = None;
            draft_target(ui, &state).and_then(|target| {
                let revision = state.committed.revision(&target.vm_id)?;
                Ok((
                    target,
                    state.configuration_source.clone(),
                    state.snapshot_source.clone(),
                    revision,
                ))
            })
        })?;
    let expected = target.clone();
    let historical = source.historical();
    let source = source.clone();
    let review_state = state.clone();
    background(
        ui,
        busy,
        if historical {
            "Building historical rehearsal plan"
        } else {
            "Building fresh plan"
        },
        move || {
            if source.configuration()?.map(|(_, text)| text) != configuration_source {
                return Err("Candidate configuration changed externally. Draft preserved; discard and Refresh.".into());
            }
            if let Some(committed) = source.committed()
                && committed.revision(&target.vm_id)? != committed_revision
            {
                return Err("Committed configuration changed externally. Draft preserved; discard and Refresh.".into());
            }
            source.plan(target, snapshot_source.as_deref())
        },
        move |ui, result| match result {
            Ok(plan) if plan.desired == expected && plan.observed.vm_id == expected.vm_id => {
                #[cfg(windows)]
                ui.set_execution_ready(
                    !historical && hyper_gpu_support::worker::available().is_ok(),
                );
                with_state(ui, &review_state, |state| {
                    state.rehearsal_plan = Some(plan.clone());
                    Ok(())
                });
                ui.set_needs_shutdown(plan.preview.guest_downtime);
                dialog(
                    ui,
                    "review",
                    if historical {
                        "Review historical draft · rehearsal only"
                    } else {
                        "Review your draft"
                    },
                    &if historical {
                        format!(
                            "HISTORICAL REHEARSAL · NO WRITES\nInputs and payload digest are recorded, not currently authenticated. No credentials, guest probe or VM effect is requested.\n\n{}",
                            review_text(&plan)
                        )
                    } else {
                        format!(
                            "{}\n\nAfter confirmation Windows elevation is requested before any guest effect. Configuration is published only after verified readback.",
                            review_text(&plan)
                        )
                    },
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

#[cfg(windows)]
fn progress_to_ui(weak: slint::Weak<AppWindow>) -> impl FnMut(hyper_gpu_support::worker::Progress) {
    move |progress| {
        let _ = weak.upgrade_in_event_loop(move |ui| {
            let mut rows: Vec<_> = ui.get_stages().iter().collect();
            if rows.len() < 256 {
                rows.push(format!("{} · {:?}", progress.stage, progress.status).into());
            }
            ui.set_stages(model(rows));
        });
    }
}
#[cfg(windows)]
fn apply_live(ui: &AppWindow, state: &SharedState, busy: &Arc<AtomicBool>) -> Result<(), String> {
    if !ui.get_execution_ready() {
        return Err("The reviewed protected worker is not available.".into());
    }
    let (approved, revision) = {
        let state = state.lock().map_err(|_| "Presentation state unavailable")?;
        let approved = state
            .rehearsal_plan
            .clone()
            .ok_or("Review the current draft before Apply")?;
        if draft_target(ui, &state)? != approved.desired {
            return Err("The draft changed after review; review it again.".into());
        }
        let revision = state.committed.revision(&approved.desired.vm_id)?;
        (approved, revision)
    };
    let selected = approved.desired.clone();
    let finished_state = state.clone();
    let progress = progress_to_ui(ui.as_weak());
    background(
        ui,
        busy,
        "Apply approved changes",
        move || {
            let credential = if approved.preview.credentials_required {
                Some(
                    hyper_gpu_support::credentials::read(&approved.desired.vm_id)?.map_or_else(
                        || {
                            hyper_gpu_support::credentials::prompt(
                                &approved.desired.vm_id,
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
        move |ui, result| match result {
            Ok(outcome) => {
                with_state(ui, &finished_state, |state| {
                    state.view.applied(&selected)?;
                    if outcome.saved {
                        state.view.published(&selected)?;
                    }
                    state.committed = hyper_gpu_support::configuration_store::read_committed();
                    state.recovery = hyper_gpu_support::runner::recovery_record()?;
                    state.rehearsal_plan = None;
                    ui.set_dirty(false);
                    present(ui, state, false);
                    Ok(())
                });
                if outcome.saved {
                    dialog(
                        ui,
                        "info",
                        "Operation verified and configuration saved",
                        "Independent GPU readback and protected configuration publication completed. Refresh shows current observations and committed intent.",
                    );
                } else {
                    dialog(
                        ui,
                        "save",
                        "Operation verified; configuration not saved",
                        &format!(
                            "{}\nThe verified intent is retained separately from committed configuration. Retry saving only; do not repeat Apply.",
                            outcome.save_error.unwrap_or_default()
                        ),
                    );
                }
            }
            Err(error) => {
                with_state(ui, &finished_state, |state| {
                    state.recovery = hyper_gpu_support::runner::recovery_record()?;
                    state.view.needs_readback = true;
                    state.rehearsal_plan = None;
                    present(ui, state, false);
                    Ok(())
                });
                dialog(
                    ui,
                    "error",
                    "Operation did not complete",
                    &format!(
                        "{error}\nYour draft is retained. Inspect recovery and refresh before retrying an uncertain operation."
                    ),
                );
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
        .recovery
        .clone()
        .ok_or("No protected save-only record is loaded")?;
    let finished_state = state.clone();
    let target = record.target.clone();
    let progress = progress_to_ui(ui.as_weak());
    background(
        ui,
        busy,
        "Retry configuration save only",
        move || hyper_gpu_support::worker::save_only(record.operation_id, progress),
        move |ui, result| match result {
            Ok(outcome) if outcome.saved => {
                with_state(ui, &finished_state, |state| {
                    state.view.published(&target)?;
                    state.committed = hyper_gpu_support::configuration_store::read_committed();
                    state.recovery = hyper_gpu_support::runner::recovery_record()?;
                    present(ui, state, false);
                    Ok(())
                });
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
        },
        move |ui, result| match result {
            Ok(plan) => {
                ui.set_needs_shutdown(plan.observed.power == hyper_gpu_support::model::Power::Off);
                let body = format!(
                    "VM: {}\nVM ID: {}\nGPU: {}\nDriver: {}\n\nCheck guest device health and hardware rendering using guest administrator credentials. {}\nGPU assignment, resources, preparation and committed configuration are not changed by this check. The worker independently rechecks the reviewed state before effects.",
                    plan.observed.name,
                    plan.target.vm_id,
                    plan.target.gpu_interface,
                    plan.gpu.driver_version,
                    if plan.observed.power == hyper_gpu_support::model::Power::Off {
                        "Temporarily start the stopped guest, then gracefully restore it to stopped."
                    } else {
                        "The guest remains running."
                    }
                );
                with_state(ui, &review_state, |state| {
                    state.verification_plan = Some(plan);
                    Ok(())
                });
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
    background(
        ui,
        busy,
        "Verify approved graphics",
        move || {
            let credential = hyper_gpu_support::credentials::read(&approved.target.vm_id)?
                .map_or_else(
                    || {
                        hyper_gpu_support::credentials::prompt(
                            &approved.target.vm_id,
                            windows::Win32::Foundation::HWND::default(),
                        )
                    },
                    Ok,
                )?;
            hyper_gpu_support::worker::verify(approved, shutdown, credential, progress)
        },
        move |ui, result| {
            with_state(ui, &finished_state, |state| {
                state.verification_plan = None;
                state.view.needs_readback = true;
                state.recovery = hyper_gpu_support::runner::recovery_record()?;
                present(ui, state, false);
                Ok(())
            });
            match result {
                Ok(outcome) if outcome.finished => dialog(
                    ui,
                    "info",
                    "Graphics verification passed",
                    "Fresh guest device health and hardware rendering passed. Disclosed guest power was restored. Refresh to read the new protected verification record.",
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
    background(
        ui,
        busy,
        "Reconcile observed outcome",
        move || {
            let graphics = record.target.enabled || !record.publication_required;
            let credential = if graphics {
                Some(
                    hyper_gpu_support::credentials::read(&record.target.vm_id)?.map_or_else(
                        || {
                            hyper_gpu_support::credentials::prompt(
                                &record.target.vm_id,
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
        move |ui, result| {
            with_state(ui, &finished_state, |state| {
                if let Ok(outcome) = &result
                    && publication
                    && outcome.operation.is_some()
                {
                    state.view.applied(&target)?;
                    if outcome.saved {
                        state.view.published(&target)?;
                    }
                }
                state.view.needs_readback = true;
                state.recovery = hyper_gpu_support::runner::recovery_record()?;
                state.committed = hyper_gpu_support::configuration_store::read_committed();
                ui.set_dirty(state.view.draft.is_some());
                present(ui, state, false);
                Ok(())
            });
            match result {
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
                Err(error) => dialog(
                    ui,
                    "error",
                    "Reconciliation remains blocked",
                    &format!(
                        "{error}\nThe durable hold remains. Partial GPU state is not automatically repaired or replayed."
                    ),
                ),
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
                state.rehearsal_plan = None;
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
    let action_source = source.clone();
    ui.on_action(move |action| {
        let Some(ui) = weak.upgrade() else { return; }; if ui.get_running() { return; }
        let result = match action.as_str() {
            "refresh" => refresh(&ui, &action_state, &action_busy, &action_source),
            "review" => review(&ui, &action_state, &action_busy, &action_source),
            "confirm" => {
                if !action_source.historical() {
                    apply_live(&ui, &action_state, &action_busy)
                } else {
                    let inputs = action_state.lock().map_err(|_| "Presentation state unavailable".to_owned())
                        .and_then(|state| {
                            let plan = state.rehearsal_plan.clone().ok_or("Review the historical draft first.")?;
                            Ok((plan, state.configuration_source.clone(), state.snapshot_source.clone()))
                        });
                    inputs.and_then(|(recorded, configuration, snapshot)| {
                        let source = action_source.clone();
                        background(&ui, &action_busy, "Checking rehearsal inputs", move || {
                            if source.configuration()?.map(|(_, text)| text) != configuration {
                                return Err("Candidate configuration changed externally. Draft preserved; discard and Refresh.".into());
                            }
                            // Recompute the plan from identical captured bytes, then display
                            // its actions. No workflow apply or mutation adapter is called.
                            source.plan(recorded.desired, snapshot.as_deref())
                        }, |ui, result| match result {
                            Ok(plan) => {
                                dialog(ui, "info", "Historical stage rehearsal complete", "These stages come from the shared plan. No VM operation, guest verification, configuration save, credential access or recovery clearing occurred. The draft remains unapplied; simulated stages do not prove GPU support.");
                                ui.set_stages(model(plan.preview.actions.iter().enumerate()
                                    .map(|(i, action)| format!("{}. {:?} · simulated only, not executed", i + 1, action).into()).collect()));
                            }
                            Err(error) => dialog(ui, "error", "Rehearsal blocked", &error),
                        })
                    })
                }
            }
            "save-only" if !action_source.historical() => save_only_live(&ui, &action_state, &action_busy),
            "verify" if !action_source.historical() => review_verification(&ui, &action_state, &action_busy),
            "confirm-verify" if !action_source.historical() => verify_live(&ui, &action_state, &action_busy),
            "reconcile" if !action_source.historical() => reconcile_live(&ui, &action_state, &action_busy),
            "reapply" => {
                // Preserve allocation edits while deliberately previewing enabled support.
                ui.set_gpu_enabled(true);
                ui.invoke_edited();
                review(&ui, &action_state, &action_busy, &action_source)
            }
            "close" => { ui.set_dialog_kind("".into()); Ok(()) },
            "exit" => { ui.set_dirty(false); slint::quit_event_loop().map_err(|e| e.to_string()) },
            "discard" | "switch" => {
                with_state(&ui, &action_state, |state| {
                    state.view.discard();
                    state.rehearsal_plan = None;
                    state.draft_gpu = None;
                    if action == "switch" { state.view.selected = state.pending_selection.take(); }
                    present(&ui, state, true); ui.set_dirty(false); ui.set_dialog_kind("".into());
                    ui.set_notice("Draft discarded. No configuration was saved or VM state changed.".into()); Ok(())
                }); Ok(())
            }
            "more" => {
                let body = if action_source.historical() {
                    "Reapply rehearses an enabled draft against historical inputs. Verification and forgetting remain blocked. No backend effect or persistent write will occur."
                } else {
                    "Reapply / Update reads a fresh shared plan for the current draft. Guest verification and forgetting pairing remain unavailable until worker and protected configuration bindings are ready."
                };
                dialog(&ui, "more", "VM actions", body); Ok(())
            },
            "credentials" => {
                let selected = ui.get_selected_id().to_string();
                let credential_source = action_source.clone();
                action_state.lock().map_err(|_| "Live presentation state unavailable".to_owned()).and_then(|s| s.target(&selected))
                    .and_then(|target| background(&ui, &action_busy, "Guest credentials", move || credential_source.credentials(&target.vm_id), |ui, result| match result {
                        Ok(true) => dialog(ui, "info", "Stored guest credentials available", "A stored credential was confirmed in this user's Windows Credential Manager for the selected VM. No guest connection or graphics check was performed."),
                        Ok(false) => dialog(ui, "info", "Guest credentials not retained", "Remember was not selected. The entered credential was discarded because no guest operation is connected yet. Nothing was stored or checked in the guest."),
                        Err(error) => dialog(ui, "error", "Guest credentials unavailable", &error),
                    }))
            }
            "diagnostics" | "recovery" => {
                with_state(&ui, &action_state, |state| {
                    if action == "recovery" && let Some(record) = &state.import_recovery {
                        dialog(&ui, "info", "Configuration import requires reconciliation", &format!("Import operation: {}\n{} destinations, {} recorded as published. Resolve this explicit CLI import with import-reconcile --operation {}. Changed files block continuation; import never changes enrollment or GPU state.", record.operation_id, record.documents.len(), record.documents.values().filter(|item| item.published).count(), record.operation_id));
                        return Ok(());
                    }
                    if action == "recovery" && !action_source.historical()
                        && let Some(record) = &state.recovery {
                        ui.set_needs_shutdown(record.target.enabled || !record.publication_required);
                        match record.phase {
                            hyper_gpu_support::configuration_store::SavePhase::Verified(_) | hyper_gpu_support::configuration_store::SavePhase::Published(_) => {
                                if record.publication_required {
                                    dialog(&ui, "save", "Verified operation awaits configuration publication", "GPU effects already passed independent readback. Retry saves only the bound configuration; it will not repeat GPU modification. Changed configuration, driver, payload or effective state blocks this retry.");
                                } else {
                                    dialog(&ui, "recovery", "Reconcile interrupted graphics check", "Check the recorded pair and fresh graphics outcome; restore only the originally approved guest power if safe. No GPU assignment, allocation, preparation or configuration publication is performed.");
                                }
                            }
                            hyper_gpu_support::configuration_store::SavePhase::Admitted | hyper_gpu_support::configuration_store::SavePhase::EffectsStarted => {
                                dialog(&ui, "recovery", "Review explicit reconciliation", &format!("Operation: {}\nVM ID: {}\nGPU: {}\nOriginal guest power: {:?}\n\nIndependently check whether the reviewed outcome was achieved. Partial assignment/settings or changed driver/payload remain blocked. An enabled pair requires fresh guest health and hardware rendering; the stopped guest may be temporarily started, then the originally approved power restored only if safe. GPU assignment, allocation, settings and preparation are never replayed. Verified intent can then be saved; conflicts retain a save-only hold.", record.operation_id, record.target.vm_id, record.target.gpu_interface, record.initial.power));
                            }
                        }
                        return Ok(());
                    }
                    let id = ui.get_selected_id(); let inventory = state.view.inventory.as_ref().ok_or("Refresh inventory first.")?;
                    let body = serde_json::to_string_pretty(&serde_json::json!({"observed": state.view.vm(&id), "managed": inventory.managed.get(id.as_str()).and_then(Option::as_ref), "historical": state.view.needs_readback, "enrollment": state.target(&id).ok()})).map_err(|e| e.to_string())?;
                    let provenance = if action_source.historical() {
                        "Historical snapshot only; no fresh guest verification. Refresh rereads the input file, not the host. No journal, enrollment or audit is modified."
                    } else {
                        "Last protected inventory read; no fresh guest verification. Refresh obtains current host state. Pending journals are retained and are not cleared by inspection."
                    };
                    dialog(&ui, "info", "Observed state and recovery records", &format!("{provenance}\n\n{body}")); Ok(())
                }); Ok(())
            }
            "shortcuts" => {
                let help = if action_source.historical() {
                    "Select a recorded enrolled VM, edit raw VRAM or enable/disable intent and Review. The shared planner uses recorded inputs only; enabled previews require snapshot plans with a matching historical payload digest. Optional --config reads one GUID-keyed candidate file. Refresh rereads inputs; credentials, verification and execution remain blocked."
                } else {
                    "Select a real VM, inspect System, or Refresh protected inventory. Enrolled pairs support in-memory toggles and raw VRAM drafts with fresh CLI planner previews. Settings opens the native credential dialog for the selected enrolled VM. Apply, Verify and enrollment await worker integration. --mock-gui rehearses fixtures without writes."
                };
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
    refresh(&ui, &state, &busy, &source)?;
    ui.run()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper_gpu_support::model::Settings;
    #[test]
    fn snapshot_source_requires_captured_input_and_refuses_credentials() {
        let source = Source::Snapshot("unused-snapshot.json".into(), None);
        assert!(source.historical());
        let target = Target {
            vm_id: "12345678-1234-1234-1234-123456789abc".into(),
            gpu_interface: "untrusted-snapshot-interface".into(),
            enabled: true,
            vram: None,
        };
        assert!(
            source
                .plan(target, None)
                .unwrap_err()
                .contains("Cannot open inventory snapshot")
        );
        assert!(source.credentials("vm").unwrap_err().contains("blocked"));
    }
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

    #[test]
    fn historical_rows_keep_identity_and_do_not_claim_current_power() {
        let vm = VmState {
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
            vm_id: vm.vm_id.clone(),
            gpu_interface: "actual-interface".into(),
            enabled: true,
            vram: None,
        };
        let mut inventory = Inventory {
            discovery: Discovery {
                vms: vec![vm.clone()],
                gpus: vec![],
            },
            managed: Default::default(),
            enrolled: vec![target.clone()],
        };
        let current = observed_row(&vm, &inventory, false);
        assert_eq!(current.power, "Running");
        assert_eq!(current.status, "Enrolled");
        let historical = observed_row(&vm, &inventory, true);
        assert_eq!(historical.id, current.id);
        assert_eq!(historical.name, current.name);
        assert_eq!(historical.power, "Running (historical)");
        assert_eq!(historical.status, "Enrolled (historical)");
        assert!(historical.gpu.ends_with("(historical)"));
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
            }),
        );
        assert_eq!(
            observed_row(&vm, &inventory, false).status,
            "Recovery required"
        );
        assert!(inventory.managed[&vm.vm_id].as_ref().unwrap().pending);
    }
}
