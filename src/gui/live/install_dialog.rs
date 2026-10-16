//! Display-only installation progress. Worker events and terminal readback retain authority.
use super::progress_state::{Event, Identity, Status};
use crate::gui::{AppWindow, InstallFact, InstallStep, model};
use hyper_gpu_support::{
    model::{Power, Target},
    ui_text::{self, dialogs::install as text},
    workflow::{OperationResult, PreviewAction},
};
use std::time::Duration;

struct Step {
    stage: String,
    label: String,
    status: i32,
    started: Option<Duration>,
    elapsed: Duration,
}

#[derive(Default)]
pub(super) struct Progress {
    steps: Vec<Step>,
    log: Vec<String>,
    last_transfer: bool,
}

impl Progress {
    pub fn new(actions: &[PreviewAction]) -> Self {
        let mut result = Self::default();
        let mut add = |stage, label| {
            result.steps.push(Step {
                stage: String::from(stage),
                label: String::from(label),
                status: 0,
                started: None,
                elapsed: Duration::ZERO,
            });
        };
        add(
            "Prepare protected worker connection",
            ui_text::progress::stage::WORKER_CONNECTION,
        );
        add(
            "Checking reviewed VM and driver files",
            text::CHECKING_FILES,
        );
        add("Read current VM state", ui_text::progress::stage::READ_VM);
        add(
            "Read selected GPU and driver version",
            ui_text::progress::stage::READ_GPU,
        );
        if actions.contains(&PreviewAction::Prepare) {
            add("Prepare current signed payload", text::PREPARING_FILES);
            add("Installing GPU driver files", text::INSTALLING_FILES);
        }
        add("Publish checked VM configuration", text::FINALISING);
        result
    }

    pub fn record(&mut self, event: &Event, stage: &str, label: &str, now: Duration) {
        if event.reportable
            && !event.persistence
            && event.identity != Identity::Transfer
            && !self.steps.iter().any(|step| step.stage == stage)
        {
            self.steps.push(Step {
                stage: stage.into(),
                label: label.into(),
                status: 0,
                started: None,
                elapsed: Duration::ZERO,
            });
        }
        if let Some(step) = self.steps.iter_mut().find(|step| step.stage == stage) {
            match event.status {
                Status::Running => {
                    step.started.get_or_insert(now);
                    step.status = 1;
                }
                Status::Done | Status::Failed => {
                    if let Some(started) = step.started.take() {
                        step.elapsed += now.saturating_sub(started);
                    }
                    step.status = if event.status == Status::Done { 2 } else { 3 };
                }
            }
        }
        if !event.reportable {
            return;
        }
        let status = match event.status {
            Status::Running => "In progress",
            Status::Done => ui_text::progress::status::COMPLETE,
            Status::Failed => ui_text::progress::status::FAILED,
        };
        let line = format!("{}   {label} · {status}", elapsed(now));
        let transfer = event.identity == Identity::Transfer && event.status == Status::Running;
        if transfer && self.last_transfer {
            if let Some(previous) = self.log.last_mut() {
                *previous = line;
            }
        } else {
            self.log.push(line);
        }
        self.last_transfer = transfer;
    }

    pub fn present(&self, ui: &AppWindow, now: Duration) {
        let rows = self
            .steps
            .iter()
            .map(|step| InstallStep {
                label: step.label.as_str().into(),
                status: step.status,
                elapsed: elapsed(
                    step.elapsed
                        + step
                            .started
                            .map_or(Duration::ZERO, |start| now.saturating_sub(start)),
                )
                .into(),
            })
            .collect();
        ui.set_install_steps(super::presentation::update_model(
            ui.get_install_steps(),
            rows,
        ));
        ui.set_dialog_details(self.log.join("\n").into());
    }
}

fn elapsed(duration: Duration) -> String {
    let seconds = duration.as_secs();
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

pub(super) fn prepare(ui: &AppWindow, vm_name: &str, actions: &[PreviewAction]) -> Progress {
    ui.set_install_vm_name(vm_name.into());
    ui.set_install_guest_work(
        actions.contains(&PreviewAction::Verify) || actions.contains(&PreviewAction::Prepare),
    );
    ui.set_install_transfer("".into());
    ui.set_install_facts(model(Vec::new()));
    let progress = Progress::new(actions);
    progress.present(ui, Duration::ZERO);
    progress
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
    let operation = operation.filter(|operation| {
        finished && published && (!checks_required || operation.verified)
            && operation.effective.vm_id == target.vm_id
            && if target.enabled { operation.effective.gpus == [target.gpu_interface.clone()] && target.allocations_match(&operation.effective) } else { operation.effective.gpus.is_empty() }
            && operation.effective.power == *final_power
            && matches!(final_power, Power::Off | Power::Running)
    }).ok_or("The terminal installation result could not be confirmed. Refresh and inspect recovery before trying again; do not repeat Apply.")?;
    let fact = |label: &str, value: String| InstallFact {
        label: label.into(),
        value: value.into(),
    };
    let mut facts = vec![
        fact(
            text::SUPPORT,
            if target.enabled {
                ui_text::common::support::ENABLED
            } else {
                ui_text::common::support::DISABLED
            }
            .into(),
        ),
        fact(text::GPU, gpu_name.into()),
        fact(
            text::PREPARATION,
            if operation.prepared {
                text::PREPARED
            } else if checks_required {
                text::UNCHANGED
            } else {
                text::NOT_REQUIRED
            }
            .into(),
        ),
        fact(
            text::CHECKS,
            if operation.verified {
                text::PREPARED
            } else {
                text::NOT_REQUIRED
            }
            .into(),
        ),
    ];
    if target.enabled {
        facts.insert(
            2,
            fact(
                text::MEMORY,
                target.vram.as_ref().map_or_else(
                    || ui_text::common::allocation::PROVIDER_DEFAULT.into(),
                    |value| crate::gui::memory::format_gb(value.optimal),
                ),
            ),
        );
    }
    Ok((
        if operation.effective.power == Power::Off {
            text::SUCCESS_OFF
        } else {
            text::SUCCESS_RUNNING
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
        assert_eq!(body, text::SUCCESS_OFF);
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
        assert_eq!(ordinary[3].value, text::NOT_REQUIRED);
        assert_eq!(ordinary[4].value, text::NOT_REQUIRED);
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
        assert_eq!(facts[0].value, ui_text::common::support::DISABLED);
        assert!(!facts.iter().any(|fact| fact.label == text::MEMORY));
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
    fn pending_steps_follow_the_review_and_only_matching_events_complete_them() {
        let mut progress = Progress::new(&[PreviewAction::Verify]);
        assert!(
            !progress
                .steps
                .iter()
                .any(|step| step.stage == "Installing GPU driver files")
        );
        assert!(progress.steps.iter().all(|step| step.status == 0));
        let stage = "Prepare protected worker connection";
        for (status, seconds) in [
            (Status::Running, 2),
            (Status::Running, 4),
            (Status::Done, 6),
        ] {
            progress.record(
                &Event::from_worker(stage, status),
                stage,
                stage,
                Duration::from_secs(seconds),
            );
        }
        assert_eq!(progress.steps[0].status, 2);
        assert_eq!(progress.steps[0].elapsed, Duration::from_secs(4));
        assert_eq!(progress.steps.last().unwrap().status, 0);
        // Repeated admission/final reads reopen the step and accumulate actual time.
        progress.record(
            &Event::from_worker(stage, Status::Running),
            stage,
            stage,
            Duration::from_secs(10),
        );
        progress.record(
            &Event::from_worker(stage, Status::Failed),
            stage,
            stage,
            Duration::from_secs(13),
        );
        assert_eq!(progress.steps[0].status, 3);
        assert_eq!(progress.steps[0].elapsed, Duration::from_secs(7));
    }

    #[test]
    fn copying_updates_one_log_line_and_never_complete_pending_steps() {
        let mut progress = Progress::new(&[PreviewAction::Prepare, PreviewAction::Verify]);
        for bytes in [20, 80, 100] {
            let stage = hyper_gpu_support::reporting::transfer_progress(bytes, 100, 1, 1);
            progress.record(
                &Event::from_worker(&stage, Status::Running),
                &stage,
                &stage,
                Duration::from_secs(bytes),
            );
        }
        assert_eq!(progress.log.len(), 1);
        assert!(progress.log[0].contains("100%"));
        assert!(progress.steps.iter().all(|step| step.status == 0));
        let stage = "credential secret fixture";
        progress.record(
            &Event::from_worker(stage, Status::Failed),
            stage,
            stage,
            Duration::ZERO,
        );
        assert_eq!(progress.log.len(), 1);
    }
}
