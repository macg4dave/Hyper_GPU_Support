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

// OS metadata is untrusted display information, never a VM-name heuristic.
fn guest_os_icon(name: Option<&str>) -> &'static str {
    let name = name.unwrap_or_default().trim().to_ascii_lowercase();
    if name.starts_with("microsoft windows") || name.starts_with("windows ") || name == "windows" {
        "windows"
    } else if name.contains("linux")
        || [
            "ubuntu",
            "debian",
            "fedora",
            "centos",
            "rocky",
            "almalinux",
            "suse",
            "opensuse",
            "arch",
            "manjaro",
            "red hat",
            "rhel",
            "kali",
            "gentoo",
            "alpine",
            "nixos",
        ]
        .iter()
        .any(|d| name == *d || name.starts_with(&format!("{d} ")))
    {
        "linux"
    } else {
        "unknown"
    }
}

fn vm_row(vm: &VmState, discovery: &Discovery) -> VmRow {
    let gpu = if vm.gpus.is_empty() {
        hyper_gpu_support::ui_text::dashboard::attachment::absent().to_owned()
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
                            hyper_gpu_support::ui_text::dashboard::attachment::identity_unknown()
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
            Power::Running => hyper_gpu_support::ui_text::common::power::running(),
            Power::Off => hyper_gpu_support::ui_text::common::power::stopped(),
            Power::Other(_) => hyper_gpu_support::ui_text::common::state::unknown(),
        }
        .into(),
        status: if vm.generation == 2 {
            hyper_gpu_support::ui_text::common::state::unknown()
        } else {
            hyper_gpu_support::ui_text::common::eligibility::unsupported()
        }
        .into(),
        gpu: gpu.into(),
        os: guest_os_icon(discovery.guest_os.get(&vm.vm_id).map(String::as_str)).into(),
    }
}

pub(super) fn observed_row(vm: &VmState, inventory: &Inventory, historical: bool) -> VmRow {
    let mut row = vm_row(vm, &inventory.discovery);
    if vm.generation == 2 && inventory.enrolled.iter().any(|t| t.vm_id == vm.vm_id) {
        row.status = if inventory.recorded_state(&vm.vm_id).recovery_required {
            hyper_gpu_support::ui_text::dashboard::state::recovery()
        } else {
            hyper_gpu_support::ui_text::dashboard::state::authorized()
        }
        .into();
    }
    if historical {
        row.power_running = false;
        row.power_stopped = false;
        row.unsupported = false;
        row.power = hyper_gpu_support::ui_text::dashboard::state::status_unavailable().into();
        row.status = hyper_gpu_support::ui_text::dashboard::state::refresh_required().into();
        row.gpu = hyper_gpu_support::ui_text::dashboard::state::status_unavailable().into();
    }
    row
}

pub(in crate::gui) fn present(ui: &AppWindow, state: &State, reset_draft: bool) {
    ui.set_presenting(true);
    if let Some(inventory) = &state.view.inventory {
        ui.set_vms(update_model(
            ui.get_vms(),
            inventory
                .discovery
                .vms
                .iter()
                .map(|vm| observed_row(vm, inventory, state.observation_historical(&vm.vm_id)))
                .collect(),
        ));
    }
    present_inner(ui, state, reset_draft);
    present_actions(ui, state);
    ui.set_presenting(false);
}

pub(in crate::gui) fn recovery_summary(state: &State, selected: &str) -> Option<String> {
    let (id, fallback, operation) = if let Some(record) = state
        .recovery_for_vm(selected)
        .or_else(|| state.recoveries.first())
    {
        (
            record.target.vm_id.as_str(),
            record.initial.name.as_str(),
            if record.publication_required {
                hyper_gpu_support::ui_text::dashboard::recovery_summary::gpu_change()
            } else {
                hyper_gpu_support::ui_text::dashboard::recovery_summary::gpu_verification()
            },
        )
    } else if let Some(record) = &state.enrollment_recovery {
        (
            record.review.vm_id.as_str(),
            record.review.vm_id.as_str(),
            hyper_gpu_support::ui_text::dashboard::recovery_summary::permission_change(),
        )
    } else if state.import_recovery.is_some() {
        return Some(hyper_gpu_support::ui_text::dashboard::recovery_summary::import().into());
    } else {
        return None;
    };
    let name = state.view.vm(id).map_or(fallback, |vm| vm.name.as_str());
    Some(if id == selected {
        hyper_gpu_support::ui_text::dashboard::recovery_summary::selected(name, operation)
            .into_string()
    } else if state.import_recovery.is_some() || state.enrollment_recovery.is_some() {
        hyper_gpu_support::ui_text::dashboard::recovery_summary::shared(name, id).into_string()
    } else {
        hyper_gpu_support::ui_text::dashboard::recovery_summary::other_vm(name, id, operation)
            .into_string()
    })
}

/// VM recovery presentation follows the same identity as protected effect admission.
pub(in crate::gui) fn vm_recovery(state: &State, id: &str) -> bool {
    !id.is_empty()
        && (state.recovery_for_vm(id).is_some()
            || state
                .enrollment_recovery
                .as_ref()
                .is_some_and(|r| r.review.vm_id == id)
            || state
                .view
                .inventory
                .as_ref()
                .is_some_and(|inventory| inventory.recorded_state(id).recovery_required))
}

fn vm_has_issue(state: &State, id: &str) -> bool {
    use hyper_gpu_support::workflow::GuestCheck;
    vm_recovery(state, id)
        || state.committed.revision(id).is_err()
        || state.view.vm(id).is_none_or(|vm| vm.generation != 2)
        || super::super::host_configuration::has_current_problem(state, id)
        || state.view.inventory.as_ref().is_some_and(|inventory| {
            inventory
                .discovery
                .issues
                .iter()
                .any(|issue| issue.target.as_deref() == Some(id))
                || inventory
                    .managed
                    .get(id)
                    .and_then(Option::as_ref)
                    .is_some_and(|journal| {
                        matches!(
                            journal.latest_check,
                            Some(
                                GuestCheck::Interrupted
                                    | GuestCheck::PayloadMismatch
                                    | GuestCheck::Failed
                                    | GuestCheck::Unavailable
                            )
                        )
                    })
        })
}

pub(super) fn present_actions(ui: &AppWindow, state: &State) {
    let selected = ui.get_selected_id();
    ui.set_initial_setup_required(state.initial_setup_required(selected.as_str()));
    // Enrollment and retained driver receipts do not establish host attachment.
    ui.set_setup_required(
        !state.observation_historical(selected.as_str())
            && state
                .view
                .vm(selected.as_str())
                .is_some_and(|vm| vm.generation == 2 && vm.gpus.is_empty()),
    );
    ui.set_has_recovery(
        !state.recoveries.is_empty()
            || state.import_recovery.is_some()
            || state.enrollment_recovery.is_some()
            || vm_recovery(state, selected.as_str()),
    );
    ui.set_selected_recovery(vm_recovery(state, selected.as_str()));
    ui.set_recovery_summary(
        recovery_summary(state, ui.get_selected_id().as_str())
            .unwrap_or_else(|| hyper_gpu_support::ui_text::dashboard::recovery::banner().to_owned())
            .into(),
    );
    ui.set_vm_details(
        if !selected.is_empty() && vm_has_issue(state, selected.as_str()) {
            diagnostics::selected_vm_details(state)
        } else {
            String::new()
        }
        .into(),
    );
    let values = ui
        .get_values()
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    let actions = state.selected_actions(ui.get_gpu_index(), &values);
    ui.set_verify_allowed(actions.verify_allowed);
    ui.set_gpu_problem(
        state
            .view
            .selected
            .as_deref()
            .is_some_and(|id| super::super::host_configuration::has_current_problem(state, id)),
    );
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
        let result = super::partial_removal::partial_removal_candidate(state);
        ui.set_reset_allowed(result.is_ok());
        ui.set_reset_unavailable(result.err().unwrap_or_default().into());
    }
    #[cfg(not(windows))]
    {
        ui.set_reset_allowed(false);
        ui.set_reset_unavailable(
            hyper_gpu_support::ui_text::dashboard::actions::partial_removal::windows_required()
                .into(),
        );
    }
    ui.set_gpu_selection_fixed(state.fixed_gpu_index().is_some());
}

pub(super) fn pending_summary(enabled: bool, baseline: Option<bool>, values: &[String]) -> String {
    let request = values
        .get(..3)
        .ok_or_else(|| {
            hyper_gpu_support::ui_text::dashboard::pending::fields_unavailable().to_owned()
        })
        .and_then(memory::request);
    match request {
        Ok(request) => hyper_gpu_support::ui_text::dashboard::pending::summary(
            match (baseline, enabled) {
                (Some(true), true) => {
                    hyper_gpu_support::ui_text::dashboard::pending::stays_enabled()
                }
                (Some(false), false) => {
                    hyper_gpu_support::ui_text::dashboard::pending::stays_disabled()
                }
                (_, true) => hyper_gpu_support::ui_text::dashboard::pending::enable(),
                (_, false) => hyper_gpu_support::ui_text::dashboard::pending::disable(),
            },
            &request.map_or_else(
                || hyper_gpu_support::ui_text::common::allocation::provider_default().to_owned(),
                |v| memory::format_gb(v.optimal),
            ),
        )
        .into_string(),
        Err(_) => hyper_gpu_support::ui_text::dashboard::pending::invalid().into(),
    }
}

fn present_inner(ui: &AppWindow, state: &State, reset_draft: bool) {
    let Some(id) = state.view.selected.as_ref() else {
        ui.set_selected_id("".into());
        ui.set_vm_name(hyper_gpu_support::ui_text::dashboard::selection::absent().into());
        ui.set_power(hyper_gpu_support::ui_text::common::state::unknown().into());
        ui.set_status(hyper_gpu_support::ui_text::common::state::unknown().into());
        ui.set_vm_unsupported(false);
        ui.set_observed(hyper_gpu_support::ui_text::dashboard::selection::absent().into());
        ui.set_desired(hyper_gpu_support::ui_text::dashboard::selection::absent().into());
        ui.set_applied(hyper_gpu_support::ui_text::dashboard::selection::absent().into());
        ui.set_pending(hyper_gpu_support::ui_text::dashboard::draft::absent().into());
        ui.set_verification(
            hyper_gpu_support::ui_text::dashboard::verification::selection_absent().into(),
        );
        ui.set_support_summary(hyper_gpu_support::ui_text::dashboard::selection::prompt().into());
        ui.set_eligible(false);
        ui.set_recovery(state.blocks_vm(ui.get_selected_id().as_str()));
        ui.set_dirty(false);
        ui.set_draft_valid(false);
        ui.set_validation("".into());
        return;
    };
    ui.set_selected_id(id.clone().into());
    let Some(vm) = state.view.vm(id) else {
        ui.set_eligible(false);
        ui.set_vm_name(hyper_gpu_support::ui_text::dashboard::selection::vm_unavailable().into());
        ui.set_power(hyper_gpu_support::ui_text::common::state::unknown().into());
        ui.set_status(hyper_gpu_support::ui_text::common::state::unavailable().into());
        ui.set_vm_unsupported(false);
        ui.set_support_summary(state.host_configuration_text(id).into());
        ui.set_observed(hyper_gpu_support::ui_text::dashboard::observation::vm_missing().into());
        ui.set_verification(
            hyper_gpu_support::ui_text::dashboard::verification::vm_unavailable().into(),
        );
        ui.set_desired(state.saved_desired_text(id).into());
        ui.set_applied(state.applied_text(id).into());
        ui.set_pending(
            if ui.get_dirty() {
                hyper_gpu_support::ui_text::dashboard::draft::vm_unavailable()
            } else {
                hyper_gpu_support::ui_text::dashboard::draft::absent()
            }
            .into(),
        );
        ui.set_recovery(state.blocks_vm(ui.get_selected_id().as_str()));
        ui.set_draft_valid(false);
        ui.set_validation(hyper_gpu_support::ui_text::dashboard::draft::vm_removed().into());
        return;
    };
    let Some(inventory) = state.view.inventory.as_ref() else {
        return;
    };
    let historical = state.observation_historical(id);
    let row = observed_row(vm, inventory, historical);
    ui.set_vm_name(row.name);
    ui.set_power(row.power);
    let enrolled = state.target(id).ok();
    ui.set_vm_unsupported(row.unsupported);
    ui.set_status(row.status);
    let recorded = inventory.recorded_state(id);
    ui.set_recovery(recorded.recovery_required || state.blocks_vm(id));
    ui.set_observed(
        hyper_gpu_support::ui_text::dashboard::observation::summary(
            if historical {
                hyper_gpu_support::ui_text::dashboard::observation::historical_prefix()
            } else {
                ""
            },
            &row.gpu,
            &Resource::ALL
                .into_iter()
                .map(|resource| {
                    vm.allocation(resource).map_or_else(
                        || {
                            hyper_gpu_support::ui_text::dashboard::observation::resource_default(
                                resource.name(),
                            )
                            .into_string()
                        },
                        |value| {
                            hyper_gpu_support::ui_text::dashboard::observation::resource_readback(
                                resource.name(),
                                &value.minimum.to_string(),
                                &value.optimal.to_string(),
                                &value.maximum.to_string(),
                            )
                            .into_string()
                        },
                    )
                })
                .collect::<Vec<_>>()
                .join(hyper_gpu_support::ui_text::common::list::separator()),
            &recorded.preparation,
        )
        .into(),
    );
    ui.set_desired(state.saved_desired_text(id).into());
    ui.set_applied(state.applied_text(id).into());
    ui.set_pending(
        if !reset_draft && ui.get_dirty() {
            pending_summary(
                ui.get_gpu_enabled(),
                enrolled.as_ref().map(|target| target.enabled),
                &ui.get_values()
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>(),
            )
        } else {
            hyper_gpu_support::ui_text::dashboard::draft::absent().to_owned()
        }
        .into(),
    );
    ui.set_verification(recorded.graphics.into());
    ui.set_support_summary(
        if historical {
            hyper_gpu_support::ui_text::dashboard::support::host_unavailable().to_owned()
        } else if vm_recovery(state, id) {
            recovery_summary(state, id).unwrap_or_else(|| {
                hyper_gpu_support::ui_text::dashboard::support::recovery_hold().to_owned()
            })
        } else if state.committed.revision(id).is_err() {
            hyper_gpu_support::ui_text::dashboard::support::saved_unavailable().to_owned()
        } else if vm.gpus.is_empty() {
            hyper_gpu_support::ui_text::dashboard::support::unattached().to_owned()
        } else {
            hyper_gpu_support::ui_text::dashboard::support::attached().to_owned()
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
                hyper_gpu_support::ui_text::dashboard::validation::stage(),
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
                    || hyper_gpu_support::ui_text::dashboard::resources::gpu_required().into(),
                    |gpu| {
                        let mut text = gpu.resource_text(resource);
                        if resource == Resource::Vram
                            && let Some(bytes) = gpu.physical_vram_bytes
                        {
                            text.push_str(
                                &hyper_gpu_support::ui_text::dashboard::resources::physical_memory(
                                    &memory::format_gib(bytes),
                                ),
                            );
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
            ui.set_memory_capacity(memory::decimal_gb(slider.physical_capacity).into());
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
            ui.set_memory_hint("".into());
        }
        Err(reason) => {
            ui.set_memory_available(false);
            ui.set_memory_capacity("".into());
            ui.set_memory_input("".into());
            ui.set_memory_minimum(1);
            ui.set_memory_maximum(2);
            ui.set_memory_position(1);
            ui.set_memory_label(hyper_gpu_support::ui_text::common::state::unavailable().into());
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
    fn os_icons_classify_guest_metadata_and_preserve_unknowns() {
        for name in ["Microsoft Windows 11 Pro", "Windows Server 2025", "windows"] {
            assert_eq!(guest_os_icon(Some(name)), "windows");
        }
        for name in [
            "Ubuntu 24.04 LTS",
            "Debian GNU/Linux",
            "Red Hat Enterprise Linux",
            "Fedora Linux 43",
            "SUSE Linux Enterprise Server",
            "Alpine v3.22",
            "Arch",
        ] {
            assert_eq!(guest_os_icon(Some(name)), "linux");
        }
        for name in [
            None,
            Some(""),
            Some("FreeBSD"),
            Some("Oracle Solaris 11"),
            Some("my Windows VM"),
        ] {
            assert_eq!(guest_os_icon(name), "unknown");
        }
    }

    #[test]
    fn refresh_updates_os_icons_in_the_existing_model_and_removes_stale_detection() {
        let vm = VmState {
            vm_id: "12345678-1234-1234-1234-123456789abc".into(),
            name: "Unchanged VM".into(),
            power: Power::Off,
            generation: 2,
            gpus: vec![],
            vram: None,
            compute: None,
            encode: None,
            decode: None,
            settings: Settings {
                low_mmio: 0,
                high_mmio: 0,
                cache_types: false,
                automatic_checkpoints: true,
            },
        };
        let mut discovery = Discovery {
            guest_os: Default::default(),
            issues: vec![],
            vms: vec![vm.clone()],
            gpus: vec![],
        };
        let mut rows = model(vec![vm_row(&vm, &discovery)]);
        let original = rows.clone();
        for (name, expected) in [
            (Some("Microsoft Windows 11 Pro"), "windows"),
            (Some("Ubuntu 24.04"), "linux"),
            (Some("FreeBSD"), "unknown"),
            (None, "unknown"),
        ] {
            discovery.guest_os.clear();
            if let Some(name) = name {
                discovery.guest_os.insert(vm.vm_id.clone(), name.into());
            }
            rows = update_model(rows, vec![vm_row(&vm, &discovery)]);
            assert_eq!(original.row_data(0).unwrap().os, expected);
            assert_eq!(rows.row_data(0).unwrap().id, vm.vm_id);
        }
    }

    #[test]
    fn saved_os_keeps_stopped_vm_icon_after_reopen_and_tracks_os_changes() {
        let id = "12345678-1234-1234-1234-123456789abc";
        let directory =
            std::env::temp_dir().join(format!("hypergpu-os-icons-{}", std::process::id()));
        let mut discovery = Discovery {
            guest_os: Default::default(),
            issues: vec![],
            gpus: vec![],
            vms: vec![VmState {
                vm_id: id.into(),
                name: "Linux name must not decide the icon".into(),
                power: Power::Running,
                generation: 2,
                gpus: vec![],
                vram: None,
                compute: None,
                encode: None,
                decode: None,
                settings: Settings {
                    low_mmio: 0,
                    high_mmio: 0,
                    cache_types: false,
                    automatic_checkpoints: true,
                },
            }],
        };
        let mut rows = model(vec![vm_row(&discovery.vms[0], &discovery)]);
        let original = rows.clone();
        for (fresh, expected) in [
            (None, "unknown"),
            (Some("Microsoft Windows 11 Pro"), "windows"),
            (None, "windows"),
            (Some("Ubuntu 24.04"), "linux"),
            (None, "linux"),
            (Some("FreeBSD"), "unknown"),
            (None, "unknown"),
            (Some("Microsoft Windows 11 Pro"), "windows"),
            (None, "windows"),
        ] {
            discovery.guest_os.clear();
            discovery.vms[0].power = if fresh.is_some() {
                Power::Running
            } else {
                Power::Off
            };
            discovery.vms[0].name = "Renamed VM".into();
            if let Some(name) = fresh {
                discovery.guest_os.insert(id.into(), name.into());
            }
            // The disk reader has no session state, also covering an app reopen.
            crate::gui::os_metadata::remember_in(&directory, &mut discovery).unwrap();
            rows = update_model(rows, vec![vm_row(&discovery.vms[0], &discovery)]);
            assert_eq!(original.row_data(0).unwrap().os, expected);
            assert_eq!(rows.row_data(0).unwrap().id, id);
        }
        // Recreating a VM with the same name but a different GUID inherits nothing.
        discovery.vms[0].vm_id = "87654321-4321-4321-4321-cba987654321".into();
        discovery.guest_os.clear();
        crate::gui::os_metadata::remember_in(&directory, &mut discovery).unwrap();
        assert_eq!(vm_row(&discovery.vms[0], &discovery).os, "unknown");
        std::fs::remove_file(directory.join(format!("{id}.txt"))).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn memory_only_pending_summary_preserves_saved_support_state() {
        let values = vec!["0".into(), "4000000000".into(), "8000000000".into()];
        for (enabled, support) in [
            (true, "GPU support stays enabled"),
            (false, "GPU support stays disabled"),
        ] {
            assert_eq!(
                pending_summary(enabled, Some(enabled), &values),
                format!("Pending: {support}. VRAM request: 4 GB. Review before applying.")
            );
            // Returning the toggle to its saved state leaves only the memory edit.
            let changed = pending_summary(!enabled, Some(enabled), &values);
            assert!(changed.contains(if enabled {
                "disable GPU support"
            } else {
                "enable GPU support"
            }));
            assert!(!changed.contains("stays"));
            assert_eq!(
                pending_summary(enabled, Some(enabled), &["".into(), "".into(), "".into()]),
                format!(
                    "Pending: {support}. VRAM request: {}. Review before applying.",
                    hyper_gpu_support::ui_text::common::allocation::provider_default()
                )
            );
        }
    }

    #[test]
    fn pending_summary_uses_exact_draft_memory() {
        let values = vec![
            "0".to_owned(),
            "6000000001".to_owned(),
            "8278507520".to_owned(),
        ];
        assert!(
            pending_summary(true, Some(false), &values)
                .contains("enable GPU support. VRAM request: 6.000000001 GB")
        );
        assert!(pending_summary(false, Some(true), &values).contains("disable GPU support"));
    }

    #[test]
    fn invalid_pending_memory_does_not_claim_an_enable_or_disable_action() {
        for values in [
            vec!["invalid".into(), "".into(), "".into()],
            vec!["0".into(), "4000000000".into()],
            vec!["0".into(), "4000000000".into(), "".into()],
        ] {
            for enabled in [true, false] {
                assert_eq!(
                    pending_summary(enabled, Some(enabled), &values),
                    "Pending edits retained. Correct the VRAM fields before reviewing."
                );
            }
        }
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
                guest_os: Default::default(),
                issues: vec![],
                vms: vec![],
                gpus: vec![],
            },
        );
        assert_eq!(row.id, vm.vm_id);
        assert_eq!(row.name, vm.name);
        assert_eq!(row.os, "unknown");
        let mut discovery = Discovery {
            guest_os: Default::default(),
            issues: vec![],
            vms: vec![],
            gpus: vec![],
        };
        discovery
            .guest_os
            .insert(vm.vm_id.clone(), "Ubuntu 24.04".into());
        let mut renamed = vm.clone();
        renamed.name = "Windows VM".into();
        assert_eq!(vm_row(&renamed, &discovery).os, "linux");
        assert!(row.unsupported);
        assert!(!row.power_running);
        assert!(!row.power_stopped);
        for power in [Power::Running, Power::Off, Power::Other(42)] {
            let mut changed = vm.clone();
            changed.power = power.clone();
            let mut row = vm_row(
                &changed,
                &Discovery {
                    guest_os: Default::default(),
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
                guest_os: Default::default(),
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
