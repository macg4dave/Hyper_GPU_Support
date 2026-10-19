//! Display-only installation progress. Worker events and terminal readback retain authority.
use super::progress_state::{Event, Identity, Status};
use crate::gui::{AppWindow, InstallFact, InstallStep, model};
use hyper_gpu_support::{
    model::{Power, Target},
    ui_text::{self, dialogs::install as text},
    workflow::{OperationResult, PreviewAction},
};

struct Step {
    stage: String,
    label: String,
    status: i32,
    log_slot: usize,
}

#[derive(Default)]
pub(super) struct Progress {
    steps: Vec<Step>,
    log: Vec<String>,
    last_transfer: bool,
}

impl Progress {
    /// Append stages in first-observed execution order. A later execution of
    /// the same stage is a distinct row; completion never moves existing rows.
    pub fn record(&mut self, event: &Event, stage: &str, label: &str) {
        let mut log_slot = None;
        let mut failed_step = false;
        if event.reportable && !event.persistence && event.identity != Identity::Transfer {
            let active = self
                .steps
                .iter()
                .rposition(|step| step.stage == stage && step.status == 1);
            let row = active.or_else(|| {
                if event.status != Status::Running {
                    self.steps.iter().rposition(|step| step.stage == stage)
                } else {
                    None
                }
            });
            let index = row.unwrap_or_else(|| {
                self.steps.push(Step {
                    stage: stage.into(),
                    label: label.into(),
                    status: 1,
                    log_slot: self.log.len(),
                });
                self.steps.len() - 1
            });
            let step = &mut self.steps[index];
            log_slot = Some(step.log_slot);
            step.status = match event.status {
                Status::Running => 1,
                Status::Done if step.status != 3 => 2,
                Status::Done | Status::Failed => 3,
            };
            failed_step = step.status == 3;
        }
        if !event.reportable {
            return;
        }
        let status = match if failed_step {
            Status::Failed
        } else {
            event.status
        } {
            Status::Running => ui_text::progress::status::in_progress(),
            Status::Done => ui_text::progress::status::complete(),
            Status::Failed => ui_text::progress::status::failed(),
        };
        let line = ui_text::progress::status::line(label, status).into_string();
        let transfer = event.identity == Identity::Transfer && event.status == Status::Running;
        if let Some(slot) = log_slot {
            // A start and its terminal status describe one execution. A fresh
            // start after completion creates a new step and a new log slot.
            if slot < self.log.len() {
                self.log[slot] = line;
            } else {
                self.log.push(line);
            }
        } else if transfer && self.last_transfer {
            if let Some(previous) = self.log.last_mut() {
                *previous = line;
            }
        } else {
            self.log.push(line);
        }
        self.last_transfer = transfer;
    }

    pub fn present(&self, ui: &AppWindow) {
        use slint::Model;
        let rows = self
            .steps
            .iter()
            .map(|step| InstallStep {
                label: step.label.as_str().into(),
                status: step.status,
            })
            .collect::<Vec<_>>();
        let current = ui.get_install_steps();
        if let Some(vector) = current
            .as_any()
            .downcast_ref::<slint::VecModel<InstallStep>>()
        {
            // Appending a discovered stage must not recreate existing rows.
            for (index, row) in rows.into_iter().enumerate() {
                if index == vector.row_count() {
                    vector.push(row);
                } else if vector.row_data(index).as_ref() != Some(&row) {
                    vector.set_row_data(index, row);
                }
            }
            while vector.row_count() > self.steps.len() {
                vector.remove(vector.row_count() - 1);
            }
        } else {
            ui.set_install_steps(model(rows));
        }
        ui.set_dialog_details(self.log.join("\n").into());
    }
}

pub(super) fn prepare(ui: &AppWindow, vm_name: &str, actions: &[PreviewAction]) -> Progress {
    ui.set_install_vm_name(vm_name.into());
    ui.set_install_guest_work(
        actions.contains(&PreviewAction::Verify) || actions.contains(&PreviewAction::Prepare),
    );
    ui.set_install_transfer("".into());
    ui.set_install_facts(model(Vec::new()));
    let progress = Progress::default();
    progress.present(ui);
    progress
}

/// Bind display context to the operation target, never to later selection changes.
pub(super) fn context(ui: &AppWindow, gpu_name: &str, target: &Target) {
    ui.set_install_gpu_name(gpu_name.into());
    ui.set_install_vram(
        target
            .vram
            .as_ref()
            .map_or_else(
                || ui_text::common::allocation::provider_default().to_owned(),
                |vram| crate::gui::memory::format_gb(vram.optimal),
            )
            .into(),
    );
}

/// Never infer the completion summary from stage rows or copy percentages.
pub(super) fn completion(
    target: &Target,
    operation: Option<&OperationResult>,
    finished: bool,
    published: bool,
    gpu_name: &str,
    final_power: &Power,
    checks_required: bool,
) -> Result<(String, Vec<InstallFact>), String> {
    let operation = operation
        .filter(|operation| {
            finished
                && published
                && (!checks_required || operation.verified)
                && operation.effective.vm_id == target.vm_id
                && if target.enabled {
                    operation.effective.gpus == [target.gpu_interface.clone()]
                        && target.allocations_match(&operation.effective)
                } else {
                    operation.effective.gpus.is_empty()
                }
                && operation.effective.power == *final_power
                && matches!(final_power, Power::Off | Power::Running)
        })
        .ok_or(ui_text::dialogs::install::unconfirmed())?;
    let fact = |label: &str, value: String| InstallFact {
        label: label.into(),
        value: value.into(),
    };
    let mut facts = vec![
        fact(
            text::support(),
            if target.enabled {
                ui_text::common::support::enabled()
            } else {
                ui_text::common::support::disabled()
            }
            .into(),
        ),
        fact(text::gpu(), gpu_name.into()),
        fact(
            text::preparation(),
            if operation.prepared {
                text::prepared()
            } else if checks_required {
                text::unchanged()
            } else {
                text::not_required()
            }
            .into(),
        ),
        fact(
            text::checks(),
            if operation.verified {
                text::prepared()
            } else {
                text::not_required()
            }
            .into(),
        ),
    ];
    if target.enabled {
        facts.insert(
            2,
            fact(
                text::memory(),
                target.vram.as_ref().map_or_else(
                    || ui_text::common::allocation::provider_default().into(),
                    |value| crate::gui::memory::format_gb(value.optimal),
                ),
            ),
        );
    }
    Ok((
        if operation.effective.power == Power::Off {
            text::success_off()
        } else {
            text::success_running()
        }
        .into(),
        facts,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_requires_terminal_success_saved_readback_and_matching_effective_state() {
        use hyper_gpu_support::model::{Allocation, Settings, VmState};
        let target = Target {
            vm_id: "12345678-1234-1234-1234-123456789abc".into(),
            gpu_interface: "test-gpu".into(),
            enabled: true,
            vram: Some(Allocation {
                minimum: 7_000_000_000,
                optimal: 7_000_000_000,
                maximum: 7_000_000_000,
            }),
            compute: None,
            encode: None,
            decode: None,
        };
        let mut operation = OperationResult {
            effective: VmState {
                vm_id: target.vm_id.clone(),
                name: "VM1".into(),
                power: Power::Off,
                generation: 2,
                gpus: vec![target.gpu_interface.clone()],
                vram: target.vram.clone(),
                compute: None,
                encode: None,
                decode: None,
                settings: Settings {
                    low_mmio: 3072,
                    high_mmio: 32768,
                    cache_types: true,
                    automatic_checkpoints: false,
                },
            },
            prepared: true,
            verified: true,
            settings_preserved: false,
        };
        for (finished, saved) in [(false, false), (true, false), (false, true)] {
            assert!(
                completion(
                    &target,
                    Some(&operation),
                    finished,
                    saved,
                    "GPU",
                    &Power::Off,
                    true
                )
                .is_err()
            );
        }
        assert!(completion(&target, None, true, true, "GPU", &Power::Off, true).is_err());
        let (body, facts) = completion(
            &target,
            Some(&operation),
            true,
            true,
            "GPU {name}",
            &Power::Off,
            true,
        )
        .unwrap();
        assert_eq!(body, text::success_off());
        assert_eq!(facts[1].value, "GPU {name}");
        assert_eq!(facts[2].value, "7 GB");
        operation.verified = false;
        assert!(
            completion(
                &target,
                Some(&operation),
                true,
                true,
                "GPU",
                &Power::Off,
                true
            )
            .is_err()
        );
        // Ordinary allocation changes do not run guest preparation or graphics checks.
        operation.prepared = false;
        let (_, ordinary) = completion(
            &target,
            Some(&operation),
            true,
            true,
            "GPU",
            &Power::Off,
            false,
        )
        .unwrap();
        assert_eq!(ordinary[3].value, text::not_required());
        assert_eq!(ordinary[4].value, text::not_required());
        assert!(
            completion(
                &target,
                Some(&operation),
                false,
                true,
                "GPU",
                &Power::Off,
                false
            )
            .is_err()
        );
        let mut disabled = target.clone();
        disabled.enabled = false;
        let mut removed = operation.clone();
        removed.effective.gpus.clear();
        let (_, facts) = completion(
            &disabled,
            Some(&removed),
            true,
            true,
            "GPU",
            &Power::Off,
            false,
        )
        .unwrap();
        assert_eq!(facts[0].value, ui_text::common::support::disabled());
        assert!(!facts.iter().any(|fact| fact.label == text::memory()));
        assert!(
            completion(
                &disabled,
                Some(&operation),
                true,
                true,
                "GPU",
                &Power::Off,
                false
            )
            .is_err()
        );
        operation.verified = true;
        operation.effective.power = Power::Running;
        assert!(
            completion(
                &target,
                Some(&operation),
                true,
                true,
                "GPU",
                &Power::Off,
                true
            )
            .is_err()
        );
        operation.effective.power = Power::Off;
        operation.effective.vram.as_mut().unwrap().optimal -= 1;
        assert!(
            completion(
                &target,
                Some(&operation),
                true,
                true,
                "GPU",
                &Power::Off,
                true
            )
            .is_err()
        );
        operation.effective.vram = target.vram.clone();
        operation.effective.vm_id = "another-vm".into();
        assert!(
            completion(
                &target,
                Some(&operation),
                true,
                true,
                "GPU",
                &Power::Off,
                true
            )
            .is_err()
        );
    }

    #[test]
    fn nested_and_repeated_stages_keep_execution_order_and_model_identity() {
        use slint::Model;
        slint::platform::set_platform(Box::new(crate::gui::catalogue_tests::Headless)).unwrap();
        let ui = AppWindow::new().unwrap();
        let mut progress = Progress::default();
        progress.present(&ui);
        let original = ui.get_install_steps();
        let events = [
            ("Checking reviewed VM and driver files", Status::Running),
            ("Read current VM state", Status::Running),
            ("Read current VM state", Status::Done),
            ("Checking reviewed VM and driver files", Status::Done),
            ("Read current VM state", Status::Running),
            ("Read current VM state", Status::Running),
            ("Read current VM state", Status::Failed),
            ("Read current VM state", Status::Done),
            ("Start guest", Status::Running),
            ("Start guest", Status::Done),
            ("Graceful guest shutdown", Status::Running),
            ("Graceful guest shutdown", Status::Done),
            ("Start guest", Status::Running),
        ];
        let mut prefix = Vec::new();
        for (stage, status) in events {
            progress.record(&Event::from_worker(stage, status), stage, stage);
            let current: Vec<_> = progress
                .steps
                .iter()
                .map(|step| step.stage.clone())
                .collect();
            assert!(current.starts_with(&prefix));
            prefix = current;
            progress.present(&ui);
            assert_eq!(ui.get_install_steps(), original);
        }
        assert_eq!(
            prefix,
            [
                "Checking reviewed VM and driver files",
                "Read current VM state",
                "Read current VM state",
                "Start guest",
                "Graceful guest shutdown",
                "Start guest"
            ]
        );
        assert_eq!(original.row_count(), 6);
        assert_eq!(
            (
                original.row_data(0).unwrap().status,
                original.row_data(1).unwrap().status,
                original.row_data(2).unwrap().status,
                original.row_data(5).unwrap().status
            ),
            (2, 2, 3, 1)
        );
        assert!(progress.log.iter().all(|line| !line.starts_with("00:")));
        assert_eq!(progress.log.len(), 6, "one line per actual execution");
        assert!(progress.log[2].ends_with(ui_text::progress::status::failed()));
        assert!(progress.log[5].ends_with("In progress"));
    }

    #[test]
    fn transfer_updates_one_log_line_without_creating_tasks_or_exposing_private_events() {
        let mut progress = Progress::default();
        for bytes in [20, 80, 100] {
            let stage = hyper_gpu_support::reporting::transfer_progress(bytes, 100, 1, 1);
            progress.record(&Event::from_worker(&stage, Status::Running), &stage, &stage);
        }
        assert_eq!(progress.log.len(), 1);
        assert!(progress.log[0].contains("100%"));
        assert!(progress.steps.is_empty());
        let stage = "credential secret fixture";
        progress.record(&Event::from_worker(stage, Status::Failed), stage, stage);
        assert_eq!(progress.log.len(), 1);
        assert!(progress.steps.is_empty());
    }
}
