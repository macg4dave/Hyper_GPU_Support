//! Fixed Configure steps shared by preview and guarded execution. Never a wire format.
use super::planning::{Decision, PreviewAction};
use crate::model::{Allocation, Power, Resource, Settings, Target};

pub(super) enum Step<'a> {
    BeginPreparation,
    Power(Power),
    Assign(bool),
    Prepare,
    RecordPreparation,
    Settings(&'a Settings),
    OwnSettings,
    Allocation(Resource, &'a Allocation),
    Verify,
    Restore(&'a Power, bool),
}

impl Step<'_> {
    pub(super) fn preview(&self) -> Option<PreviewAction> {
        Some(match self {
            Self::Power(Power::Off) | Self::Restore(Power::Off, true) => PreviewAction::Shutdown,
            Self::Power(Power::Running) | Self::Restore(Power::Running, true) => {
                PreviewAction::Start
            }
            Self::Assign(false) => PreviewAction::Detach,
            Self::Assign(true) => PreviewAction::Attach,
            Self::Prepare => PreviewAction::Prepare,
            Self::Settings(_) => PreviewAction::Settings,
            Self::Allocation(..) => PreviewAction::Allocation,
            Self::Verify => PreviewAction::Verify,
            _ => return None,
        })
    }
}

impl Decision {
    pub(super) fn steps<'a>(&'a self, target: &'a Target) -> Vec<Step<'a>> {
        let mut steps = Vec::new();
        let mut power = self.initial.power.clone();
        if self.needs_changes {
            if power == Power::Running && (!self.stale || self.assignment) {
                steps.push(Step::Power(Power::Off));
                power = Power::Off;
            }
            if self.stale {
                steps.push(Step::BeginPreparation);
                if self.assignment {
                    steps.push(Step::Assign(false));
                }
                if power != Power::Running {
                    steps.push(Step::Power(Power::Running));
                }
                steps.extend([
                    Step::Prepare,
                    Step::Power(Power::Off),
                    Step::RecordPreparation,
                ]);
                power = Power::Off;
            }
            if target.enabled {
                if self.initial.settings != self.desired {
                    steps.push(Step::Settings(&self.desired));
                }
                steps.push(Step::OwnSettings);
                if !self.assignment || self.stale {
                    steps.push(Step::Assign(true));
                }
                for resource in &self.allocation_writes {
                    if let Some(value) = target.allocation(*resource) {
                        steps.push(Step::Allocation(*resource, value));
                    }
                }
            } else {
                if self.assignment {
                    steps.push(Step::Assign(false));
                }
                if self.initial.settings != self.disabled_settings {
                    steps.push(Step::Settings(&self.disabled_settings));
                }
            }
        }
        if self.setup_required {
            if power != Power::Running {
                steps.push(Step::Power(Power::Running));
                power = Power::Running;
            }
            steps.push(Step::Verify);
        }
        steps.push(Step::Restore(
            &self.restore_power,
            power != self.restore_power,
        ));
        steps
    }
}
