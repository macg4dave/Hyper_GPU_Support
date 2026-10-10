//! Approved Slint presentation: live startup or explicitly selected mock rehearsal.
mod live;
mod live_state;
mod mock;
mod snapshot;
use slint::{ComponentHandle, Model, ModelRc, Timer, TimerMode, VecModel};
use std::{cell::RefCell, rc::Rc, time::Duration};
slint::include_modules!();

#[derive(Clone)]
struct Saved {
    enabled: bool,
    gpu: i32,
    memory_gb: i32,
    values: Vec<String>,
}
impl Default for Saved {
    fn default() -> Self {
        Self {
            enabled: true,
            gpu: 0,
            memory_gb: 4,
            values: ["100", "500", "1000"]
                .repeat(4)
                .into_iter()
                .map(str::to_owned)
                .collect(),
        }
    }
}
#[derive(Default)]
struct Session {
    inventory: Vec<VmRow>,
    saved: std::collections::HashMap<String, Saved>,
    pending: Option<String>,
    stage: usize,
}
fn model<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    Rc::new(VecModel::from(rows)).into()
}
fn draft(ui: &AppWindow) -> Saved {
    Saved {
        enabled: ui.get_gpu_enabled(),
        gpu: ui.get_gpu_index(),
        memory_gb: ui.get_memory_gb(),
        values: ui.get_values().iter().map(|v| v.to_string()).collect(),
    }
}
fn refresh_inventory(ui: &AppWindow, session: &Session) {
    ui.set_vms(model(session.inventory.clone()));
}
fn select(ui: &AppWindow, session: &Session, id: &str) {
    let Some(vm) = session.inventory.iter().find(|vm| vm.id == id) else {
        return;
    };
    let saved = session.saved.get(id).cloned().unwrap_or_else(|| Saved {
        enabled: vm.status == "Configured",
        gpu: if vm.gpu.contains("4070") { 1 } else { 0 },
        ..Saved::default()
    });
    ui.set_selected_id(vm.id.clone());
    ui.set_vm_name(vm.name.clone());
    ui.set_power(vm.power.clone());
    ui.set_status(vm.status.clone());
    ui.set_eligible(vm.status != "Unsupported" && vm.status != "Unknown");
    ui.set_observed(format!("{} (sample observation)", vm.gpu).into());
    ui.set_desired(
        if saved.enabled {
            "GPU enabled · sample saved configuration"
        } else {
            "GPU disabled · sample saved configuration"
        }
        .into(),
    );
    ui.set_verification(
        if vm.status == "Configured" {
            "Recorded sample verification; current graphics not verified"
        } else {
            "Not verified"
        }
        .into(),
    );
    ui.set_gpu_enabled(saved.enabled);
    ui.set_gpu_index(saved.gpu);
    ui.set_memory_gb(saved.memory_gb);
    ui.set_values(model(saved.values.into_iter().map(Into::into).collect()));
    ui.set_dirty(false);
    ui.set_validation("".into());
}
fn dialog(ui: &AppWindow, kind: &str, title: &str, body: &str) {
    ui.set_stages(model(Vec::new()));
    ui.set_dialog_kind(kind.into());
    ui.set_dialog_title(title.into());
    ui.set_dialog_body(if kind == "error" {
        hyper_gpu_support::reporting::operator_error(title, body).into()
    } else {
        body.into()
    });
}
fn mock_close_request(ui: &AppWindow) -> slint::CloseRequestResponse {
    if ui.get_running() {
        return slint::CloseRequestResponse::KeepWindowShown;
    }
    if ui.get_dirty() {
        dialog(
            ui,
            "exit",
            "Unapplied draft",
            "Keep editing or discard the in-memory draft and close the prototype. Nothing has been applied to a real VM.",
        );
        return slint::CloseRequestResponse::KeepWindowShown;
    }
    slint::CloseRequestResponse::HideWindow
}
fn review(ui: &AppWindow) {
    if !ui.get_eligible() || ui.get_recovery() {
        return;
    }
    let d = draft(ui);
    let error = mock::validate(&d.values, d.gpu, d.enabled);
    if !error.is_empty() {
        ui.set_validation(error.into());
        return;
    }
    ui.set_needs_shutdown(ui.get_power() == "Running");
    let gpu = ["RTX 5060", "RTX 4070", "Unavailable AMD adapter"][d.gpu as usize];
    let mut body = format!(
        "VM: {}\nPairing: {}\nDesired GPU support: {} → {}\nPhysical GPU: {gpu}\n\n",
        ui.get_vm_name(),
        ui.get_selected_id(),
        ui.get_desired(),
        if d.enabled { "Enabled" } else { "Disabled" }
    );
    body.push_str(&format!("GPU Memory slider: {} GB (visual mock preference only; advanced allocations unchanged)\n\n", d.memory_gb));
    for (name, triple) in ["VRAM", "Compute", "Encode", "Decode"]
        .iter()
        .zip(d.values.chunks(3))
    {
        body.push_str(&format!(
            "{name}: {} / {} / {} mock units\n",
            triple[0], triple[1], triple[2]
        ));
    }
    body.push_str("\nDemonstrated effects: enrollment if needed, guest preparation, allocation, readback and configuration save. Running guests illustrate graceful shutdown and restoration. No elevation or real changes will occur.");
    dialog(ui, "review", "Review your draft", &body);
}
fn save_sample(ui: &AppWindow, session: &mut Session) {
    session
        .saved
        .insert(ui.get_selected_id().to_string(), draft(ui));
    if let Some(row) = session
        .inventory
        .iter_mut()
        .find(|row| row.id == ui.get_selected_id())
    {
        row.status = "Configured".into();
    }
    ui.set_status("Configured".into());
    ui.set_dirty(false);
    let gpu = ["RTX 5060", "RTX 4070", "AMD sample"][ui.get_gpu_index() as usize];
    ui.set_desired(
        format!(
            "GPU {} · {gpu} · sample configuration",
            if ui.get_gpu_enabled() {
                "enabled"
            } else {
                "disabled"
            }
        )
        .into(),
    );
    refresh_inventory(ui, session);
}
pub(super) fn run(mock_mode: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ui = AppWindow::new()?;
    ui.set_mock_mode(mock_mode);
    if !mock_mode {
        return live::run(ui, live::Source::Protected);
    }
    let session = Rc::new(RefCell::new(Session {
        inventory: mock::inventory(false),
        ..Session::default()
    }));
    refresh_inventory(&ui, &session.borrow());
    select(&ui, &session.borrow(), &session.borrow().inventory[0].id);
    let timer = Rc::new(Timer::default());
    let weak = ui.as_weak();
    ui.on_edited(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_dirty(true);
            let d = draft(&ui);
            ui.set_validation(mock::validate(&d.values, d.gpu, d.enabled).into());
        }
    });
    let weak = ui.as_weak();
    let state = session.clone();
    ui.on_select_vm(move |id| { if let Some(ui) = weak.upgrade() {
        if id == ui.get_selected_id() { return; }
        if ui.get_dirty() { state.borrow_mut().pending = Some(id.to_string()); dialog(&ui, "switch", "Keep your changes?", "There is one unapplied VM draft. Stay here, review it, or discard it before switching. Nothing has been applied."); }
        else { select(&ui, &state.borrow(), &id); }
    } });
    let weak = ui.as_weak();
    let state = session.clone();
    ui.on_scenario_changed(move || {
        if let Some(ui) = weak.upgrade() {
            let scenario = ui.get_scenario();
            let mut s = state.borrow_mut();
            s.inventory = if scenario == 1 {
                Vec::new()
            } else {
                mock::inventory(scenario == 2)
            };
            ui.set_recovery(scenario == 3);
            refresh_inventory(&ui, &s);
            if scenario == 1 && !ui.get_dirty() {
                ui.set_selected_id("".into());
            } else if ui.get_selected_id().is_empty() && !s.inventory.is_empty() {
                select(&ui, &s, &s.inventory[0].id);
            }
        }
    });
    let weak = ui.as_weak();
    let state = session.clone();
    let operation_timer = timer.clone();
    ui.on_action(move |action| { let Some(ui) = weak.upgrade() else { return; };
        match action.as_str() {
            "close" if !ui.get_running() => ui.set_dialog_kind("".into()),
            "exit" if !ui.get_running() => { ui.set_dirty(false); let _ = slint::quit_event_loop(); }
            "review" | "reapply" => review(&ui),
            "discard" => { select(&ui, &state.borrow(), &ui.get_selected_id()); ui.set_notice("Draft discarded. Sample observed state was not changed.".into()); }
            "switch" => { let id = state.borrow_mut().pending.take(); if let Some(id) = id { select(&ui, &state.borrow(), &id); } ui.set_dialog_kind("".into()); }
            "enroll" => { ui.set_gpu_enabled(true); ui.set_dirty(true); ui.set_notice("Enrollment and guest preparation staged for demonstration review.".into()); }
            "refresh" => { refresh_inventory(&ui, &state.borrow()); ui.set_notice("Sample inventory refreshed. Your draft is preserved.".into()); }
            "more" => dialog(&ui, "more", "VM actions", "Choose an action to demonstrate. Verification and reapply use sample results. Forgetting only removes the in-memory sample pairing."),
            "forget" => dialog(&ui, "forget", "Forget sample pairing?", "Remove this prototype's saved pairing from memory? Observed state stays unchanged. No real records or VM configuration will be deleted."),
            "confirm-forget" => { let mut s = state.borrow_mut(); s.saved.remove(ui.get_selected_id().as_str()); if let Some(row) = s.inventory.iter_mut().find(|row| row.id == ui.get_selected_id()) { row.status = "Setup required".into(); } select(&ui, &s, &ui.get_selected_id()); refresh_inventory(&ui, &s); ui.set_dialog_kind("".into()); }
            "verify" => { ui.set_verification("Demonstration only · no actual guest graphics test".into()); dialog(&ui, "info", "Verification demonstration", "The sample guest reports PnP Code 0 and a hardware D3D11 frame. These are mock results and do not prove GPU support."); }
            "confirm" => {
                if ui.get_running() { return; }
                if ui.get_scenario() == 7 { dialog(&ui, "error", "Draft is stale · demonstration", "The sample VM configuration changed externally. Apply is blocked. Your draft is preserved; discard it and refresh before creating a new draft."); return; }
                state.borrow_mut().stage = 0; ui.set_running(true); dialog(&ui, "progress", "Applying · demonstration", "Watch the proposed stages. This timer only animates mock events; it does not operate on a VM.");
                let weak = ui.as_weak(); let s = state.clone(); let finish_timer = operation_timer.clone();
                operation_timer.start(TimerMode::Repeated, Duration::from_millis(650), move || {
                    let Some(ui) = weak.upgrade() else { finish_timer.stop(); return; };
                    let labels = ["Validate pairing and draft", "Authorize operation / guest downtime", "Prepare guest and stage allocation", "Read back observed state", "Save desired configuration", "Restore guest power"];
                    let mut s = s.borrow_mut(); s.stage += 1;
                    ui.set_stages(model(labels.iter().enumerate().map(|(i, label)| format!("{}  {label} · mock", if i < s.stage { "Complete" } else if i == s.stage { "Running" } else { "Pending" }).into()).collect()));
                    let scenario = ui.get_scenario();
                    if s.stage == 3 && (scenario == 4 || scenario == 5) {
                        finish_timer.stop(); ui.set_running(false); ui.set_recovery(true);
                        let message = if scenario == 4 { "Mock GPU allocation failed. Earlier stages completed in the demonstration. GPU modifications are blocked. Inspect mock reconciliation; no automatic rollback or retry." } else { "Mock worker disconnected. Allocation outcome is unknown. A timeout is not proof of failure. Inspect mock reconciliation before further changes." };
                        dialog(&ui, "recovery", "Recovery required · demonstration", message); return;
                    }
                    if s.stage >= labels.len() {
                        finish_timer.stop(); ui.set_running(false);
                        if scenario == 6 { dialog(&ui, "save", "Configuration save failed · demonstration", "Sample GPU readback matched the draft, but configuration saving failed. Retry only the in-memory save step; do not replay GPU changes."); }
                        else { save_sample(&ui, &mut s); dialog(&ui, "info", "Demonstration complete", "The sample flow finished. Only the prototype's in-memory desired configuration was updated. Observed hardware was not read or modified."); }
                    }
                });
            }
            "save-only" => { let mut s = state.borrow_mut(); save_sample(&ui, &mut s); dialog(&ui, "info", "Save-only demonstration complete", "Sample configuration was stored in memory. No disk or GPU changes occurred."); }
            "recovery" => dialog(&ui, "recovery", "Inspect recovery · demonstration", "Sample journal: enrollment complete; allocation outcome unknown; configuration not saved. Read-only mock reconciliation will inspect the sample result, retain the draft, and release only this demonstration hold."),
            "reconcile" => { ui.set_recovery(false); dialog(&ui, "info", "Mock reconciliation complete", "The demonstration hold is cleared. Your draft is preserved. No real operation was inspected or repaired."); }
            "reset" => { let mut s = state.borrow_mut(); *s = Session { inventory: mock::inventory(false), ..Session::default() }; ui.set_scenario(0); ui.set_recovery(false); refresh_inventory(&ui, &s); select(&ui, &s, &s.inventory[0].id); ui.set_notice("Prototype session reset.".into()); }
            "credentials" => dialog(&ui, "info", "Guest credentials", "The production application would use protected Windows credential facilities. This prototype never asks for or stores passwords."),
            "diagnostics" => dialog(&ui, "error", "Inventory unavailable · demonstration", "Sample category: Access denied. VM state and GPU eligibility are unknown. Refresh inventory or check access in the future application. No live query was made."),
            "shortcuts" => dialog(&ui, "info", "Explore the prototype", "Use Tab / Shift+Tab to move through standard controls; Enter or Space activates them. Drag the divider to resize panels. Each panel scrolls independently. At narrow widths, scroll the workspace horizontally. Settings contains failure, empty and long-inventory scenarios."),
            "help-toggle" => ui.set_notice("Contextual help preference changed for this demonstration session.".into()),
            _ => {}
        }
    });
    let weak = ui.as_weak();
    ui.window().on_close_requested(move || {
        weak.upgrade()
            .map_or(slint::CloseRequestResponse::HideWindow, |ui| {
                mock_close_request(&ui)
            })
    });
    ui.run()?;
    Ok(())
}

pub(super) fn run_snapshot(
    path: std::path::PathBuf,
    configuration: Option<std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ui = AppWindow::new()?;
    // Use real-data presentation, without fixture labels or illustrative allocations.
    ui.set_mock_mode(false);
    ui.set_snapshot_mode(true);
    ui.set_workspace_label("SNAPSHOT REHEARSAL · NO WRITES".into());
    live::run(ui, live::Source::Snapshot(path, configuration))
}

#[cfg(test)]
mod layout_qualification {
    use super::*;

    // Opt-in: ordinary unit tests must not launch a renderer.
    #[test]
    #[ignore = "explicit software mock layout qualification; use SLINT_BACKEND=headless with slint/mcp"]
    fn approved_ui_layout_qualification() {
        let output = std::env::current_dir()
            .unwrap()
            .join("local/evidence/gui-003-layout");
        std::fs::create_dir_all(&output).unwrap();
        let ui = AppWindow::new().unwrap();
        ui.set_mock_mode(true);
        let session = Session {
            inventory: mock::inventory(true),
            ..Session::default()
        };
        refresh_inventory(&ui, &session);
        select(&ui, &session, &session.inventory[0].id);
        ui.show().unwrap();
        for (name, width, height, page, review, scale_factor) in [
            ("narrow-dashboard", 700, 520, 0, false, 1.0),
            ("wide-dashboard", 1600, 1000, 0, false, 1.0),
            ("system", 1240, 860, 1, false, 1.0),
            ("settings", 1240, 860, 2, false, 1.0),
            ("about", 1240, 860, 3, false, 1.0),
            ("narrow-review", 700, 520, 0, true, 1.0),
            ("narrow-review-150", 1050, 780, 0, true, 1.5),
            ("narrow-review-200", 1400, 1040, 0, true, 2.0),
        ] {
            ui.set_page(page);
            ui.window()
                .dispatch_event(slint::platform::WindowEvent::ScaleFactorChanged { scale_factor });
            ui.window()
                .set_size(slint::PhysicalSize::new(width, height));
            // The headless adapter's set_size uses scale 1. Supply the matching
            // logical resize event explicitly for simulated DPI qualification.
            ui.window()
                .dispatch_event(slint::platform::WindowEvent::Resized {
                    size: slint::LogicalSize::new(
                        width as f32 / scale_factor,
                        height as f32 / scale_factor,
                    ),
                });
            if review {
                dialog(
                    &ui,
                    "review",
                    "Review historical draft · rehearsal",
                    "Historical preview; no backend effects.\nDesired GPU support: Enabled\nVM ID: a5801e91-1083-4e79-a803-000000000001\nSelected GPU: \\\\?\\PCI#VEN_10DE&DEV_2D05#SAMPLE_GPU_A#{064092b3-625e-43bf-9eb5-dc845897dd59}\\GPUPARAV\nPreparation and guest verification required. Guest lifecycle consent required.",
                );
                ui.set_needs_shutdown(true);
            }
            let pixels = ui.window().take_snapshot().unwrap();
            assert_eq!((pixels.width(), pixels.height()), (width, height));
            // Uncompressed top-down BMP avoids a test-only image dependency.
            // This buffer contains synthetic UI state, never live inventory.
            let bytes = pixels.as_bytes();
            let mut bitmap = vec![0u8; 54];
            bitmap[..2].copy_from_slice(b"BM");
            bitmap[2..6].copy_from_slice(&(54 + bytes.len() as u32).to_le_bytes());
            bitmap[10..14].copy_from_slice(&54u32.to_le_bytes());
            bitmap[14..18].copy_from_slice(&40u32.to_le_bytes());
            bitmap[18..22].copy_from_slice(&(width as i32).to_le_bytes());
            bitmap[22..26].copy_from_slice(&(-(height as i32)).to_le_bytes());
            bitmap[26..28].copy_from_slice(&1u16.to_le_bytes());
            bitmap[28..30].copy_from_slice(&32u16.to_le_bytes());
            for pixel in bytes.chunks_exact(4) {
                bitmap.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
            }
            std::fs::write(output.join(format!("{name}.bmp")), bitmap).unwrap();
        }
        let weak = ui.as_weak();
        ui.window()
            .on_close_requested(move || mock_close_request(&weak.upgrade().unwrap()));
        ui.set_running(true);
        ui.window()
            .dispatch_event(slint::platform::WindowEvent::CloseRequested);
        assert!(ui.window().is_visible(), "active work must defer close");
        ui.set_running(false);
        ui.set_dirty(true);
        ui.window()
            .dispatch_event(slint::platform::WindowEvent::CloseRequested);
        assert!(
            ui.window().is_visible(),
            "an unapplied draft must defer close"
        );
        assert_eq!(ui.get_dialog_kind(), "exit");
        ui.set_dirty(false);
        ui.set_dialog_kind("".into());
        ui.window()
            .dispatch_event(slint::platform::WindowEvent::CloseRequested);
        assert!(!ui.window().is_visible(), "idle clean windows may close");
    }
}
