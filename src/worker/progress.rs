//! Observable backend stages, guest phases and progress-delivery failure handling.
use super::protocol::{Progress, StageStatus};
use crate::{
    model::{Allocation, Gpu, Power, Resource, Settings, Target, VmState},
    workflow::{self, Backend, Journal},
};

pub(super) struct ObservedBackend<'a, B, F> {
    pub(super) backend: &'a mut B,
    pub(super) notify: &'a mut F,
    pub(super) before_effect: Option<&'a mut dyn FnMut() -> Result<(), String>>,
}

/// Phase boundaries come from the fixed worker, never elapsed time or byte percentage.
fn guest_progress(
    event: workflow::PreparationProgress,
    current: &mut Option<&'static str>,
    notify: &mut impl FnMut(Progress) -> Result<(), String>,
) -> Result<(), String> {
    use workflow::PreparationProgress;
    let stage = match event {
        PreparationProgress::Copying {
            bytes,
            total,
            files,
            count,
        } => {
            return notify(Progress {
                stage: crate::reporting::transfer_progress(bytes, total, files, count),
                status: StageStatus::Running,
            });
        }
        PreparationProgress::CopyComplete => "File copying complete; waiting for guest checks",
        PreparationProgress::CheckingCopiedFiles => "Checking copied driver files",
        PreparationProgress::InstallingFiles => "Installing GPU driver files",
        PreparationProgress::CheckingInstalledFiles => "Checking installed driver files",
        PreparationProgress::CheckingGuestFiles => "Checking current guest files and setup record",
        PreparationProgress::CheckingGraphics => "Checking device health and hardware rendering",
    };
    if let Some(previous) = *current {
        notify(Progress {
            stage: previous.into(),
            status: StageStatus::Done,
        })?;
    }
    notify(Progress {
        stage: stage.into(),
        status: StageStatus::Running,
    })?;
    *current = Some(stage);
    Ok(())
}

fn finish_guest_progress(
    current: Option<&str>,
    succeeded: bool,
    notify: &mut impl FnMut(Progress) -> Result<(), String>,
) -> Result<(), String> {
    if let Some(stage) = current {
        notify(Progress {
            stage: stage.into(),
            status: if succeeded {
                StageStatus::Done
            } else {
                StageStatus::Failed
            },
        })?;
    }
    Ok(())
}

/// Report preparation before blocking reads/elevation; never imply a final outcome.
pub(super) fn preparation_stage<T>(
    name: &str,
    notify: &mut impl FnMut(Progress) -> Result<(), String>,
    action: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    notify(Progress {
        stage: name.into(),
        status: StageStatus::Running,
    })?;
    let result = action();
    let delivery = notify(Progress {
        stage: name.into(),
        status: if result.is_ok() {
            StageStatus::Done
        } else {
            StageStatus::Failed
        },
    });
    finish_progress(result, delivery)
}

pub(super) fn finish_progress<T>(
    result: Result<T, String>,
    delivery: Result<(), String>,
) -> Result<T, String> {
    match (result, delivery) {
        (Err(primary), Err(delivery)) => {
            Err(format!("Progress delivery failed: {delivery}; {primary}"))
        }
        (Err(primary), Ok(())) => Err(primary),
        (Ok(_), Err(delivery)) => Err(format!(
            "Stage completed but progress delivery failed; outcome uncertain: {delivery}"
        )),
        (Ok(value), Ok(())) => Ok(value),
    }
}
impl<B: Backend, F: FnMut(Progress) -> Result<(), String>> ObservedBackend<'_, B, F> {
    fn effect<T>(
        &mut self,
        name: &str,
        action: impl FnOnce(&mut B) -> Result<T, String>,
    ) -> Result<T, String> {
        if let Some(before) = &mut self.before_effect {
            before()?;
        }
        self.stage(name, action)
    }
    pub(super) fn stage<T>(
        &mut self,
        name: &str,
        action: impl FnOnce(&mut B) -> Result<T, String>,
    ) -> Result<T, String> {
        self.stage_observed(name, |observed| action(observed.backend))
    }
    // Orchestration stages retain this wrapper so nested reads remain observable.
    pub(super) fn stage_observed<T>(
        &mut self,
        name: &str,
        action: impl FnOnce(&mut Self) -> Result<T, String>,
    ) -> Result<T, String> {
        (self.notify)(Progress {
            stage: name.into(),
            status: StageStatus::Running,
        })?;
        let result = crate::diagnostics::run(name, "approved-worker", || action(self));
        let delivery = (self.notify)(Progress {
            stage: name.into(),
            status: if result.is_ok() {
                StageStatus::Done
            } else {
                StageStatus::Failed
            },
        });
        finish_progress(
            result.map_err(|error| crate::reporting::failure_at_stage(error, name)),
            delivery,
        )
    }
}
impl<B: Backend, F: FnMut(Progress) -> Result<(), String>> Backend for ObservedBackend<'_, B, F> {
    fn check_installation_os(&mut self, t: &Target) -> Result<(), String> {
        self.stage("Check guest OS compatibility for installation", |b| {
            b.check_installation_os(t)
        })
    }
    fn inspect(&mut self, t: &Target) -> Result<VmState, String> {
        self.stage("Read current VM state", |b| b.inspect(t))
    }
    fn gpu(&mut self, t: &Target) -> Result<Gpu, String> {
        self.stage("Read selected GPU and driver version", |b| b.gpu(t))
    }
    fn payload(&mut self, t: &Target) -> Result<String, String> {
        self.stage("Authenticate current signed driver files", |b| b.payload(t))
    }
    fn journal(&mut self, t: &Target) -> Result<Option<Journal>, String> {
        self.backend.journal(t)
    }
    fn save(&mut self, t: &Target, j: &Journal) -> Result<(), String> {
        self.stage("Persist operation recovery", |b| b.save(t, j))
    }
    fn power(&mut self, t: &Target, p: Power) -> Result<(), String> {
        self.effect(
            if p == Power::Off {
                "Graceful guest shutdown"
            } else {
                "Start guest"
            },
            |b| b.power(t, p),
        )
    }
    fn assign(&mut self, t: &Target, e: bool) -> Result<(), String> {
        self.effect(
            if e {
                "Attach selected GPU"
            } else {
                "Detach selected GPU"
            },
            |b| b.assign(t, e),
        )
    }
    fn settings(&mut self, t: &Target, s: &Settings) -> Result<(), String> {
        self.effect("Compatibility settings", |b| b.settings(t, s))
    }
    fn allocation(&mut self, t: &Target, resource: Resource, a: &Allocation) -> Result<(), String> {
        self.effect("Provider allocation", |b| b.allocation(t, resource, a))
    }
    fn prepare(&mut self, t: &Target) -> Result<String, String> {
        if let Some(before) = &mut self.before_effect {
            before()?;
        }
        (self.notify)(Progress {
            stage: "Prepare current signed payload".into(),
            status: StageStatus::Running,
        })?;
        let mut phase = None;
        let result =
            crate::diagnostics::run("Prepare current signed payload", "approved-worker", || {
                self.backend.prepare_with_progress(t, &mut |event| {
                    guest_progress(event, &mut phase, self.notify)
                })
            });
        let phase_delivery = finish_guest_progress(phase, result.is_ok(), self.notify);
        let delivery = (self.notify)(Progress {
            stage: "Prepare current signed payload".into(),
            status: if result.is_ok() {
                StageStatus::Done
            } else {
                StageStatus::Failed
            },
        });
        finish_progress(
            result.map_err(|error| {
                crate::reporting::failure_at_stage(error, "Prepare current signed payload")
            }),
            phase_delivery.and(delivery),
        )
    }
    fn verify(&mut self, t: &Target, g: &Gpu) -> Result<(), String> {
        if let Some(before) = &mut self.before_effect {
            before()?;
        }
        let stage = "Check guest health and hardware rendering";
        (self.notify)(Progress {
            stage: stage.into(),
            status: StageStatus::Running,
        })?;
        let mut phase = None;
        let result = crate::diagnostics::run(stage, "approved-worker", || {
            self.backend.verify_with_progress(t, g, &mut |event| {
                guest_progress(event, &mut phase, self.notify)
            })
        });
        let phase_delivery = finish_guest_progress(phase, result.is_ok(), self.notify);
        let delivery = (self.notify)(Progress {
            stage: stage.into(),
            status: if result.is_ok() {
                StageStatus::Done
            } else {
                StageStatus::Failed
            },
        });
        finish_progress(
            result.map_err(|error| crate::reporting::failure_at_stage(error, stage)),
            phase_delivery.and(delivery),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::ordinary_disabled_plan;
    use super::*;
    use crate::runner::NativeBackend;

    #[test]
    fn worker_preparation_reports_order_failure_and_stops_on_lost_progress() {
        use std::cell::RefCell;
        for action_fails in [false, true] {
            for fail_delivery in [None, Some(0), Some(1)] {
                let trace = RefCell::new(Vec::new());
                let mut delivered = 0;
                let result = preparation_stage(
                    "Check protected worker admission",
                    &mut |event| {
                        trace.borrow_mut().push(match event.status {
                            StageStatus::Running => "running",
                            StageStatus::Done => "done",
                            StageStatus::Failed => "failed",
                        });
                        let fail = fail_delivery == Some(delivered);
                        delivered += 1;
                        if fail {
                            Err("progress disconnected".into())
                        } else {
                            Ok(())
                        }
                    },
                    || {
                        trace.borrow_mut().push("work");
                        if action_fails {
                            Err("artifact check failed".into())
                        } else {
                            Ok(())
                        }
                    },
                );
                if fail_delivery == Some(0) {
                    assert_eq!(*trace.borrow(), ["running"]);
                } else {
                    assert_eq!(
                        *trace.borrow(),
                        [
                            "running",
                            "work",
                            if action_fails { "failed" } else { "done" }
                        ]
                    );
                }
                assert_eq!(result.is_ok(), !action_fails && fail_delivery.is_none());
                if action_fails && fail_delivery != Some(0) {
                    assert!(result.unwrap_err().contains("artifact check failed"));
                }
            }
        }
    }

    #[test]
    fn guest_phase_failures_retain_the_actual_stage_and_delivery_failure_prevents_next_stage_notification()
     {
        let mut events = Vec::new();
        let mut phase = None;
        let mut notify = |event: Progress| {
            events.push(event);
            Ok(())
        };
        guest_progress(
            workflow::PreparationProgress::CheckingCopiedFiles,
            &mut phase,
            &mut notify,
        )
        .unwrap();
        guest_progress(
            workflow::PreparationProgress::InstallingFiles,
            &mut phase,
            &mut notify,
        )
        .unwrap();
        finish_guest_progress(phase, false, &mut notify).unwrap();
        assert_eq!(events.len(), 4);
        assert_eq!(events[1].status, StageStatus::Done);
        assert_eq!(events[3].stage, "Installing GPU driver files");
        assert_eq!(events[3].status, StageStatus::Failed);
        let mut delivered = 0;
        let result = guest_progress(
            workflow::PreparationProgress::CheckingInstalledFiles,
            &mut phase,
            &mut |_| {
                delivered += 1;
                Err("pipe disconnected".into())
            },
        );
        assert!(result.is_err());
        assert_eq!(delivered, 1);
        assert_eq!(phase, Some("Installing GPU driver files"));
    }

    #[test]
    fn admission_progress_is_visible_before_work_and_delivery_failure_prevents_work() {
        for fail_delivery in [false, true] {
            let mut backend = NativeBackend {
                reads: None,
                credential: None,
                manifest: None,
                payload_hold: Vec::new(),
                data: std::path::PathBuf::new(),
            };
            let mut updates = Vec::new();
            let mut notify = |update: Progress| {
                updates.push(update.status);
                if fail_delivery {
                    Err("disconnected".into())
                } else {
                    Ok(())
                }
            };
            let mut called = false;
            let result = ObservedBackend {
                backend: &mut backend,
                notify: &mut notify,
                before_effect: None,
            }
            .stage("Checking reviewed VM and driver files", |_| {
                called = true;
                Err::<(), _>("stale review".into())
            });
            assert!(result.is_err());
            assert_eq!(called, !fail_delivery);
            assert_eq!(
                updates,
                if fail_delivery {
                    vec![StageStatus::Running]
                } else {
                    vec![StageStatus::Running, StageStatus::Failed]
                }
            );
        }
    }

    #[test]
    fn read_progress_delivery_failure_prevents_native_reads_and_reports_the_read_identity() {
        let target = ordinary_disabled_plan().desired;
        for (read, stage) in [
            (0, "Read current VM state"),
            (1, "Read selected GPU and driver version"),
            (2, "Authenticate current signed driver files"),
        ] {
            // No native read may run: the progress sink refuses the start event.
            let mut backend = NativeBackend {
                reads: None,
                credential: None,
                manifest: None,
                payload_hold: Vec::new(),
                data: std::path::PathBuf::new(),
            };
            let mut updates = Vec::new();
            let mut notify = |update: Progress| {
                updates.push((update.stage, update.status));
                Err("disconnected before read".into())
            };
            let mut observed = ObservedBackend {
                backend: &mut backend,
                notify: &mut notify,
                before_effect: None,
            };
            let result = match read {
                0 => observed.inspect(&target).map(|_| ()),
                1 => observed.gpu(&target).map(|_| ()),
                _ => observed.payload(&target).map(|_| ()),
            };
            assert_eq!(result.unwrap_err(), "disconnected before read");
            assert_eq!(updates.len(), 1);
            assert_eq!(updates[0].0, stage);
            assert!(matches!(updates[0].1, StageStatus::Running));
            assert!(backend.manifest.is_none());
        }
    }

    #[test]
    fn parent_check_contains_nested_read_outcomes_in_order_and_preserves_failure() {
        for fail_read in [false, true] {
            let mut backend = NativeBackend {
                reads: None,
                credential: None,
                manifest: None,
                payload_hold: Vec::new(),
                data: std::path::PathBuf::new(),
            };
            let mut updates = Vec::new();
            let mut notify = |update: Progress| {
                updates.push((update.stage, update.status));
                Ok(())
            };
            let result = ObservedBackend {
                backend: &mut backend,
                notify: &mut notify,
                before_effect: None,
            }
            .stage_observed(
                "Confirm final VM state and signed driver files",
                |observed| {
                    observed.stage("Authenticate current signed driver files", |_| {
                        if fail_read {
                            Err("driver file changed".into())
                        } else {
                            Ok(())
                        }
                    })
                },
            );
            assert_eq!(result.is_err(), fail_read);
            if let Err(error) = result {
                assert!(error.contains("driver file changed"));
            }
            assert_eq!(updates.len(), 4);
            assert_eq!(
                updates[0].0,
                "Confirm final VM state and signed driver files"
            );
            assert_eq!(updates[1].0, "Authenticate current signed driver files");
            assert_eq!(updates[2].0, updates[1].0);
            assert_eq!(updates[3].0, updates[0].0);
            assert!(matches!(updates[0].1, StageStatus::Running));
            assert!(matches!(updates[1].1, StageStatus::Running));
            for (_, status) in &updates[2..] {
                assert!(matches!(
                    (status, fail_read),
                    (StageStatus::Failed, true) | (StageStatus::Done, false)
                ));
            }
        }
    }

    #[test]
    fn failed_progress_delivery_preserves_the_adapter_failure_and_uncertainty() {
        let both = finish_progress::<()>(
            Err("adapter failed".into()),
            Err("pipe disconnected".into()),
        )
        .unwrap_err();
        assert!(both.contains("adapter failed") && both.contains("pipe disconnected"));
        let lost = finish_progress(Ok(()), Err("pipe disconnected".into())).unwrap_err();
        assert!(lost.contains("outcome uncertain"));
        assert!(finish_progress(Ok(()), Ok(())).is_ok());
    }
}
