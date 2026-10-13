//! Slint model and editor adapters; backend decisions remain in State and the shared core.
use crate::gui::{
    AppWindow, VmRow, diagnostics,
    live_state::{State, allocation_values},
    memory, model,
};
use hyper_gpu_support::{
    gui_model::Inventory,
    model::{Discovery, Power, Resource, Target, VmState},
};
use slint::Model;

// Preserve repeated UI elements and editor focus when an inventory read only
// changes row contents. Rebuild only when the number of rows changes.
pub(super) fn update_model<T: Clone + PartialEq + 'static>(
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

fn vm_row(vm: &VmState, discovery: &Discovery) -> VmRow {
    let gpu = if vm.gpus.is_empty() {
        hyper_gpu_support::ui_text::dashboard::attachment::ABSENT.to_owned()
    } else {
        vm.gpus
            .iter()
            .map(|interface| {
                discovery
                    .gpus
                    .iter()
                    .find(|gpu| gpu.interface.eq_ignore_ascii_case(interface))
                    .map_or_else(
                        || {
                            hyper_gpu_support::ui_text::dashboard::attachment::IDENTITY_UNKNOWN
                                .to_owned()
                        },
                        |gpu| gpu.name.clone(),
                    )
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    VmRow {
        power_running: vm.power == Power::Running,
        power_stopped: vm.power == Power::Off,
        unsupported: vm.generation != 2,
        id: vm.vm_id.clone().into(),
        name: vm.name.clone().into(),
        power: match vm.power {
            Power::Running => hyper_gpu_support::ui_text::common::power::RUNNING,
            Power::Off => hyper_gpu_support::ui_text::common::power::STOPPED,
            Power::Other(_) => hyper_gpu_support::ui_text::common::state::UNKNOWN,
        }
        .into(),
        status: if vm.generation == 2 {
            hyper_gpu_support::ui_text::common::state::UNKNOWN
        } else {
            hyper_gpu_support::ui_text::common::eligibility::UNSUPPORTED
        }
        .into(),
        gpu: gpu.into(),
        os: "unknown".into(),
    }
}

pub(super) fn observed_row(vm: &VmState, inventory: &Inventory, historical: bool) -> VmRow {
    let mut row = vm_row(vm, &inventory.discovery);
    if vm.generation == 2 && inventory.enrolled.iter().any(|t| t.vm_id == vm.vm_id) {
        row.status = if inventory.recorded_state(&vm.vm_id).recovery_required {
            hyper_gpu_support::ui_text::dashboard::state::RECOVERY
        } else {
            hyper_gpu_support::ui_text::dashboard::state::AUTHORIZED
        }
        .into();
    }
    if historical {
        row.power_running = false;
        row.power_stopped = false;
        row.unsupported = false;
        row.power = hyper_gpu_support::ui_text::dashboard::state::STATUS_UNAVAILABLE.into();
        row.status = hyper_gpu_support::ui_text::dashboard::state::REFRESH_REQUIRED.into();
        row.gpu = hyper_gpu_support::ui_text::dashboard::state::STATUS_UNAVAILABLE.into();
    }
    row
}

pub(super) fn present(ui: &AppWindow, state: &State, reset_draft: bool) {
    ui.set_presenting(true);
    if let Some(inventory) = &state.view.inventory {
        ui.set_vms(update_model(
            ui.get_vms(),
            inventory
                .discovery
                .vms
                .iter()
                .map(|vm| observed_row(vm, inventory, state.view.needs_readback))
                .collect(),
        ));
    }
    present_inner(ui, state, reset_draft);
    present_actions(ui, state);
    ui.set_presenting(false);
}

pub(super) fn present_actions(ui: &AppWindow, state: &State) {
    ui.set_vm_details(diagnostics::selected_vm_details(state).into());
    let values = ui
        .get_values()
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    let actions = state.selected_actions(ui.get_gpu_index(), &values);
    ui.set_verify_allowed(actions.verify_allowed);
    ui.set_repair_allowed(actions.repair_allowed);
    ui.set_remove_allowed(actions.remove_allowed);
    ui.set_forget_allowed(actions.forget_allowed);
    ui.set_verify_unavailable(actions.verify.into());
    ui.set_repair_unavailable(actions.repair.into());
    ui.set_remove_unavailable(actions.remove.into());
    ui.set_forget_unavailable(actions.forget.into());
    ui.set_action_recovery(actions.recovery);
    #[cfg(windows)]
    {
        let result = super::partial_removal_candidate(state);
        ui.set_reset_allowed(result.is_ok());
        ui.set_reset_unavailable(result.err().unwrap_or_default().into());
    }
    #[cfg(not(windows))]
    {
        ui.set_reset_allowed(false);
        ui.set_reset_unavailable(
            hyper_gpu_support::ui_text::dashboard::actions::partial_removal::WINDOWS_REQUIRED
                .into(),
        );
    }
    ui.set_gpu_selection_fixed(state.fixed_gpu_index().is_some());
}

pub(super) fn pending_summary(enabled: bool, values: &[String]) -> String {
    let request = values
        .get(..3)
        .ok_or_else(|| "VRAM fields unavailable".to_owned())
        .and_then(memory::request);
    match request {
        Ok(request) => format!(
            "Pending: {} GPU support. VRAM request: {}. Review before applying.",
            if enabled {
                hyper_gpu_support::ui_text::common::support::ENABLE_ACTION
            } else {
                hyper_gpu_support::ui_text::common::support::DISABLE_ACTION
            },
            request.map_or_else(
                || hyper_gpu_support::ui_text::common::allocation::PROVIDER_DEFAULT.to_owned(),
                |v| memory::format_gb(v.optimal)
            )
        ),
        Err(_) => "Pending edits retained. Correct the VRAM fields before reviewing.".into(),
    }
}

fn present_inner(ui: &AppWindow, state: &State, reset_draft: bool) {
    let Some(id) = state.view.selected.as_ref() else {
        ui.set_selected_id("".into());
        ui.set_vm_name(hyper_gpu_support::ui_text::dashboard::selection::ABSENT.into());
        ui.set_power(hyper_gpu_support::ui_text::common::state::UNKNOWN.into());
        ui.set_status(hyper_gpu_support::ui_text::common::state::UNKNOWN.into());
        ui.set_vm_unsupported(false);
        ui.set_observed(hyper_gpu_support::ui_text::dashboard::selection::ABSENT.into());
        ui.set_desired(hyper_gpu_support::ui_text::dashboard::selection::ABSENT.into());
        ui.set_applied(hyper_gpu_support::ui_text::dashboard::selection::ABSENT.into());
        ui.set_pending(hyper_gpu_support::ui_text::dashboard::draft::ABSENT.into());
        ui.set_verification(
            hyper_gpu_support::ui_text::dashboard::verification::SELECTION_ABSENT.into(),
        );
        ui.set_support_summary(hyper_gpu_support::ui_text::dashboard::selection::PROMPT.into());
        ui.set_eligible(false);
        ui.set_recovery(
            state.recovery.is_some()
                || state.import_recovery.is_some()
                || state.enrollment_recovery.is_some(),
        );
        ui.set_dirty(false);
        ui.set_draft_valid(false);
        ui.set_validation("".into());
        return;
    };
    ui.set_selected_id(id.clone().into());
    let Some(vm) = state.view.vm(id) else {
        ui.set_eligible(false);
        ui.set_vm_name(hyper_gpu_support::ui_text::dashboard::selection::VM_UNAVAILABLE.into());
        ui.set_power(hyper_gpu_support::ui_text::common::state::UNKNOWN.into());
        ui.set_status(hyper_gpu_support::ui_text::common::state::UNAVAILABLE.into());
        ui.set_vm_unsupported(false);
        ui.set_support_summary(state.host_configuration_text(id).into());
        ui.set_observed(hyper_gpu_support::ui_text::dashboard::observation::VM_MISSING.into());
        ui.set_verification(
            hyper_gpu_support::ui_text::dashboard::verification::VM_UNAVAILABLE.into(),
        );
        ui.set_desired(state.saved_desired_text(id).into());
        ui.set_applied(state.applied_text(id).into());
        ui.set_pending(
            if ui.get_dirty() {
                hyper_gpu_support::ui_text::dashboard::draft::VM_UNAVAILABLE
            } else {
                hyper_gpu_support::ui_text::dashboard::draft::ABSENT
            }
            .into(),
        );
        ui.set_recovery(
            state.recovery.is_some()
                || state.import_recovery.is_some()
                || state.enrollment_recovery.is_some(),
        );
        ui.set_draft_valid(false);
        ui.set_validation(hyper_gpu_support::ui_text::dashboard::draft::VM_REMOVED.into());
        return;
    };
    let Some(inventory) = state.view.inventory.as_ref() else {
        return;
    };
    let row = observed_row(vm, inventory, state.view.needs_readback);
    ui.set_vm_name(row.name);
    ui.set_power(row.power);
    let enrolled = state.target(id).ok();
    ui.set_vm_unsupported(row.unsupported);
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
                hyper_gpu_support::ui_text::dashboard::observation::HISTORICAL_PREFIX
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
            pending_summary(
                ui.get_gpu_enabled(),
                &ui.get_values()
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>(),
            )
        } else {
            hyper_gpu_support::ui_text::dashboard::draft::ABSENT.to_owned()
        }
        .into(),
    );
    ui.set_verification(recorded.graphics.into());
    ui.set_support_summary(
        if state.view.needs_readback {
            hyper_gpu_support::ui_text::dashboard::support::HOST_UNAVAILABLE.to_owned()
        } else if recorded.recovery_required
            || state.recovery.is_some()
            || state.enrollment_recovery.is_some()
        {
            hyper_gpu_support::ui_text::dashboard::support::RECOVERY_HOLD.to_owned()
        } else if state.committed.revision(id).is_err() {
            hyper_gpu_support::ui_text::dashboard::support::SAVED_UNAVAILABLE.to_owned()
        } else if vm.gpus.is_empty() {
            hyper_gpu_support::ui_text::dashboard::support::UNATTACHED.to_owned()
        } else {
            hyper_gpu_support::ui_text::dashboard::support::ATTACHED.to_owned()
        }
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
        ui.set_draft_valid(false);
        ui.set_validation(
            hyper_gpu_support::reporting::operator_error(
                hyper_gpu_support::ui_text::dashboard::validation::STAGE,
                &error,
            )
            .into(),
        );
    } else if !ui.get_dirty() {
        let validation = draft_target(ui, state);
        ui.set_draft_valid(validation.is_ok());
        ui.set_validation(validation.err().unwrap_or_default().into());
    }
}

pub(super) fn resource_controls(ui: &AppWindow, state: &State) {
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
                    || hyper_gpu_support::ui_text::dashboard::resources::GPU_REQUIRED.into(),
                    |gpu| {
                        let mut text = gpu.resource_text(resource);
                        if resource == Resource::Vram
                            && let Some(bytes) = gpu.physical_vram_bytes
                        {
                            text.push_str(&format!(
                                "; physical dedicated memory: {}",
                                memory::format_gib(bytes)
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
            ui.set_memory_input(
                memory::request(&values[..3])
                    .ok()
                    .flatten()
                    .map_or_else(String::new, |a| memory::decimal_gb(a.optimal))
                    .into(),
            );
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
            ui.set_memory_input("".into());
            ui.set_memory_minimum(1);
            ui.set_memory_maximum(2);
            ui.set_memory_position(1);
            ui.set_memory_label(hyper_gpu_support::ui_text::common::state::UNAVAILABLE.into());
            ui.set_memory_range_start("—".into());
            ui.set_memory_range_end("—".into());
            ui.set_memory_hint(reason.into());
        }
    }
}

pub(super) fn draft_target(ui: &AppWindow, state: &State) -> Result<Target, String> {
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
#[cfg(test)]
mod tests {
    use super::*;
    use hyper_gpu_support::model::Settings;

    #[test]
    fn pending_summary_uses_exact_draft_memory_and_does_not_guess_invalid_edits() {
        let values = vec![
            "0".to_owned(),
            "6000000001".to_owned(),
            "8278507520".to_owned(),
        ];
        assert!(
            pending_summary(true, &values)
                .contains("enable GPU support. VRAM request: 6.000000001 GB")
        );
        assert!(pending_summary(false, &values).contains("disable GPU support"));
        assert!(
            pending_summary(true, &["invalid".into(), "".into(), "".into()])
                .contains("Correct the VRAM fields")
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
        assert!(row.unsupported);
        assert!(!row.power_running);
        assert!(!row.power_stopped);
        for power in [Power::Running, Power::Off, Power::Other(42)] {
            let mut changed = vm.clone();
            changed.power = power.clone();
            let mut row = vm_row(
                &changed,
                &Discovery {
                    issues: vec![],
                    vms: vec![],
                    gpus: vec![],
                },
            );
            row.power = String::from_iter(['λ', '\n', '}']).into();
            row.status = row.power.clone();
            assert_eq!(row.power_running, power == Power::Running);
            assert_eq!(row.power_stopped, power == Power::Off);
            assert!(row.unsupported);
        }
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
        assert!(current.power_running);
        assert!(!current.power_stopped);
        assert!(!current.unsupported);
        let historical = observed_row(&vm, &inventory, true);
        assert_eq!(historical.id, current.id);
        assert_eq!(historical.name, current.name);
        assert!(!historical.power_running);
        assert!(!historical.power_stopped);
        assert!(!historical.unsupported);
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
        assert!(inventory.recorded_state(&vm.vm_id).recovery_required);
        assert!(inventory.managed[&vm.vm_id].as_ref().unwrap().pending);
    }
}
