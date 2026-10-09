//! Read-only live presentation using the existing core discovery boundary.
//! Unconnected mutation actions never dispatch mock callbacks or privileged effects.
use super::{AppWindow, GpuRow, VmRow, dialog, model};
use hyper_gpu_support::model::{Discovery, Power, VmState};
use slint::{ComponentHandle, Model};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[cfg(windows)]
fn discover() -> Result<Discovery, String> {
    use hyper_gpu_support::{process, runner, windows_hyperv};
    if process::is_elevated()? {
        windows_hyperv::discover()
    } else {
        // The existing fixed runner authorizes discovery and retains its required
        // audit records. Do not install/elevate or weaken audit for GUI reads.
        let value = runner::submit(runner::request(runner::Operation::Discover, None, None))?;
        serde_json::from_value(value).map_err(|error| format!("Invalid discovery reply: {error}"))
    }
}

#[cfg(not(windows))]
fn discover() -> Result<Discovery, String> {
    Err("Hyper-V discovery requires Windows x64".into())
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
        // Discovery proves neither enrollment nor successful guest preparation.
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

fn select(ui: &AppWindow, id: &str) {
    let Some(vm) = ui.get_vms().iter().find(|vm| vm.id == id) else {
        return;
    };
    ui.set_selected_id(vm.id.clone());
    ui.set_vm_name(vm.name.clone());
    ui.set_power(vm.power);
    ui.set_status(vm.status);
    ui.set_gpu_index(
        ui.get_gpu_options()
            .iter()
            .position(|name| name == vm.gpu)
            .map_or(-1, |index| index as i32),
    );
    ui.set_observed(vm.gpu);
    ui.set_desired("Not loaded; configuration binding is pending".into());
    ui.set_verification("Not queried; attachment does not prove guest graphics health".into());
    ui.set_validation(
        "Live configuration actions are not connected yet. CLI commands remain available.".into(),
    );
    ui.set_eligible(false);
    ui.set_gpu_enabled(false);
    ui.set_dirty(false);
}

fn refresh(ui: &AppWindow, busy: &Arc<AtomicBool>) -> Result<(), String> {
    if busy.swap(true, Ordering::AcqRel) {
        return Ok(());
    }
    ui.set_notice("Reading actual VM and GPU inventory... Existing observations remain historical until refresh succeeds.".into());
    let weak = ui.as_weak();
    let worker_busy = busy.clone();
    if let Err(error) = std::thread::Builder::new().name("gui-discovery".into()).spawn(move || {
        let result = discover();
        // If the window/event loop has closed, there is no recipient. This read
        // performs no VM effects and owns no recoverable mutation to reconcile.
        let completion_busy = worker_busy.clone();
        if weak.upgrade_in_event_loop(move |ui| {
            completion_busy.store(false, Ordering::Release);
            match result {
            Ok(discovery) => {
                let selected = ui.get_selected_id();
                let rows: Vec<_> = discovery.vms.iter().map(|vm| vm_row(vm, &discovery)).collect();
                let next = rows.iter().find(|vm| vm.id == selected).or_else(|| rows.first()).map(|vm| vm.id.clone());
                ui.set_vms(model(rows));
                ui.set_gpu_information(model(discovery.gpus.iter().map(|gpu| GpuRow {
                    name: gpu.name.clone().into(),
                    description: format!("Driver: {}\nVRAM provider values: {} / {} / {}\nPreparation adapter: {}\nAttachment does not prove guest graphics health; sharing is not qualified.", gpu.driver_version, gpu.vram.minimum, gpu.vram.optimal, gpu.vram.maximum, if gpu.preparation_supported { "available" } else { "not implemented" }).into(),
                }).collect()));
                ui.set_gpu_options(model(discovery.gpus.iter().map(|gpu| gpu.name.clone().into()).collect()));
                ui.set_system_summary("Live Hyper-V inventory read successfully. Enrollment, guest health and configuration are not inferred from discovery.".into());
                if let Some(id) = next { select(&ui, &id); } else { ui.set_selected_id("".into()); }
                ui.set_notice("Live inventory loaded. Configuration actions await backend binding; no sample outcomes are used.".into());
            }
            Err(error) => {
                // Preserve existing rows only as explicitly historical observations.
                ui.set_eligible(false);
                ui.set_notice(format!("Live inventory unavailable: {error}. Existing rows, if any, are historical; Refresh retries discovery.").into());
                ui.set_system_summary(format!("Live discovery unavailable: {error}").into());
            }
            }
        }).is_err() {
            worker_busy.store(false, Ordering::Release);
        }
    }) {
        busy.store(false, Ordering::Release);
        return Err(format!("Cannot start live discovery: {error}"));
    }
    Ok(())
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
    let weak = ui.as_weak();
    ui.on_select_vm(move |id| {
        if let Some(ui) = weak.upgrade() {
            select(&ui, &id);
        }
    });
    let weak = ui.as_weak();
    let callback_busy = busy.clone();
    ui.on_action(move |action| {
        let Some(ui) = weak.upgrade() else { return; };
        match action.as_str() {
            "refresh" => if let Err(error) = refresh(&ui, &callback_busy) { dialog(&ui, "info", "Refresh unavailable", &format!("{error}. Existing observations are historical until refresh succeeds.")); },
            "close" => ui.set_dialog_kind("".into()),
            "shortcuts" => dialog(&ui, "info", "Application help", "Select a real VM, inspect System, or Refresh actual inventory. Real configuration actions are not connected yet. Run with --mock-gui for an explicitly simulated rehearsal."),
            _ => dialog(&ui, "info", "Action unavailable", "This live action is not connected yet; no operation was performed. Use the supported CLI. Any inventory freshness warning remains in the workspace banner."),
        }
    });
    refresh(&ui, &busy)?;
    ui.run()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper_gpu_support::model::{Allocation, Gpu, Settings};

    fn vm() -> VmState {
        VmState {
            vm_id: "12345678-1234-1234-1234-123456789abc".into(),
            name: "Actual VM name".into(),
            power: Power::Off,
            generation: 2,
            gpus: vec!["selected-interface".into()],
            vram: None,
            settings: Settings {
                low_mmio: 0,
                high_mmio: 0,
                cache_types: false,
                automatic_checkpoints: true,
            },
        }
    }

    #[test]
    fn live_rows_preserve_identity_and_attachment_without_inventing_health_or_enrollment() {
        let vm = vm();
        let discovery = Discovery {
            vms: vec![],
            gpus: vec![Gpu {
                interface: "SELECTED-INTERFACE".into(),
                name: "Discovered adapter".into(),
                vendor: 0x10de,
                device: 1,
                driver_version: "discovered-version".into(),
                preparation_supported: true,
                vram: Allocation {
                    minimum: 1,
                    optimal: 2,
                    maximum: 3,
                },
            }],
        };
        let row = vm_row(&vm, &discovery);
        assert_eq!(row.id, vm.vm_id);
        assert_eq!(row.name, vm.name);
        assert_eq!(row.gpu, "Discovered adapter");
        assert_eq!(row.status, "Unknown");
        assert_eq!(row.os, "unknown");
        assert_eq!(row.power, "Stopped");
    }

    #[test]
    fn unresolved_attachment_and_power_are_not_replaced_by_samples() {
        let mut vm = vm();
        vm.power = Power::Other(42);
        vm.generation = 1;
        let discovery = Discovery {
            vms: vec![],
            gpus: vec![],
        };
        let row = vm_row(&vm, &discovery);
        assert_eq!(row.gpu, "selected-interface");
        assert_eq!(row.power, "Unknown");
        assert_eq!(row.status, "Unsupported");
        vm.gpus.clear();
        assert_eq!(vm_row(&vm, &discovery).gpu, "No GPU attached");
    }
}
