//! Read-only action explanations. Fresh protected reviews still authorize effects.
use super::live_state::State;
use hyper_gpu_support::{model::Power, workflow::verification_power};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct SelectedActions {
    pub verify: String,
    pub repair: String,
    pub remove: String,
    pub forget: String,
    pub recovery: bool,
    pub verify_allowed: bool,
    pub repair_allowed: bool,
    pub remove_allowed: bool,
    pub forget_allowed: bool,
}

impl SelectedActions {
    pub fn allowed(&self, action: &str) -> Option<bool> {
        match action {
            "verify" => Some(self.verify_allowed),
            "repair" => Some(self.repair_allowed),
            "remove-gpu" => Some(self.remove_allowed),
            "forget" => Some(self.forget_allowed),
            _ => None,
        }
    }
    pub fn reason(&self, action: &str) -> Option<&str> {
        match action {
            "verify" => Some(&self.verify),
            "repair" => Some(&self.repair),
            "remove-gpu" => Some(&self.remove),
            "forget" => Some(&self.forget),
            _ => None,
        }
    }
}

impl State {
    /// Explain cached prerequisites without guest access, writes or new discovery.
    /// Typed availability permits requesting a fresh review, never executing effects.
    pub fn selected_actions(&self, gpu_index: i32, values: &[String]) -> SelectedActions {
        let recovery = self.action_recovery_message().is_some();
        let common = self.action_prerequisite(recovery);
        let check = |result: Result<(), String>| {
            let allowed = result.is_ok();
            let reason = result.err().unwrap_or_default();
            let reason = match reason.as_str() {
                "This VM/GPU pair is not authorized. Close this panel and review first-time setup." => {
                    hyper_gpu_support::ui_text::dashboard::actions::PERMISSION_REQUIRED.into()
                }
                _ => reason,
            };
            (allowed, reason)
        };
        let actions = || -> Result<SelectedActions, String> {
            common?;
            let id = self.view.selected.as_deref().ok_or("Select a VM first.")?;
            let target = self.target(id)?;
            let inventory = self.view.inventory.as_ref().ok_or("Use Refresh first.")?;
            let authorized = self.enrolled(&target);
            let pairing =
                "This VM/GPU pair is not authorized. Close this panel and review first-time setup.";
            let (verify_allowed, verify) = check((|| {
                self.editor_eligibility(id)?;
                if self.view.draft.is_some() {
                    return Err(
                        "Review and Apply, or discard the pending edits before Verify.".into(),
                    );
                }
                if !authorized {
                    return Err(pairing.into());
                }
                let vm = self.view.vm(id).ok_or("VM unavailable; use Refresh.")?;
                verification_power(
                    &target,
                    vm,
                    inventory.managed.get(id).and_then(Option::as_ref),
                )
                .map(|_| ())
            })());
            let (repair_allowed, repair) = check((|| {
                if !authorized {
                    return Err(pairing.into());
                }
                self.draft(id, true, gpu_index, values).map(|_| ())
            })());
            let (remove_allowed, remove) = check(if authorized {
                self.removal_draft(id, gpu_index, values).map(|_| ())
            } else {
                Err(pairing.into())
            });
            let (forget_allowed, forget) = check(self.forget_target().map(|_| ()));
            Ok(SelectedActions {
                verify_allowed,
                repair_allowed,
                remove_allowed,
                forget_allowed,
                verify,
                repair,
                remove,
                forget,
                recovery,
            })
        };
        actions().unwrap_or_else(|reason| SelectedActions {
            verify_allowed: false,
            repair_allowed: false,
            remove_allowed: false,
            forget_allowed: false,
            verify: reason.clone(),
            repair: reason.clone(),
            remove: reason.clone(),
            forget: reason,
            recovery,
        })
    }

    fn action_prerequisite(&self, recovery: bool) -> Result<(), String> {
        let id = self.view.selected.as_deref().ok_or("Select a VM first.")?;
        if recovery {
            return Err("An unfinished operation blocks new actions. Choose Review recovery; your edits are kept.".into());
        }
        if self.view.needs_readback {
            return Err(
                "Current VM information is unavailable. Use Refresh; your edits are kept.".into(),
            );
        }
        if self.view.unsaved {
            return Err("The operation outcome is not saved. Use Refresh and inspect recovery before another action.".into());
        }
        if self.view.inventory.as_ref().is_some_and(|inventory| {
            inventory
                .discovery
                .issues
                .iter()
                .any(|issue| issue.target.as_deref() == Some(id))
        }) {
            return Err(
                "Selected VM observations are incomplete. Correct access and use Refresh; your edits are kept.".into(),
            );
        }
        let vm = self
            .view
            .vm(id)
            .ok_or("The selected VM is unavailable. Use Refresh or discard its draft.")?;
        if let Some(inventory) = &self.view.inventory
            && inventory.enrolled.iter().any(|target| target.vm_id == id)
            && !inventory.managed.contains_key(id)
        {
            return Err("Managed operation history is unavailable. Use Refresh before reviewing new actions.".into());
        }
        if vm.generation != 2 {
            return Err(
                "GPU support requires a Generation 2 VM. Close this panel and select a supported VM.".into(),
            );
        }
        if !matches!(vm.power, Power::Off | Power::Running) {
            return Err("The VM is changing power state or is not running/off. Wait for a stable state, then Refresh.".into());
        }
        Ok(())
    }

    /// Enrollment fixes the GPU identity; replacement is not a supported GUI action.
    pub fn fixed_gpu_index(&self) -> Option<i32> {
        let id = self.view.selected.as_deref()?;
        let inventory = self.view.inventory.as_ref()?;
        let pair = inventory
            .enrolled
            .iter()
            .find(|target| target.vm_id == id)?;
        Some(
            inventory
                .discovery
                .gpus
                .iter()
                .position(|gpu| gpu.interface == pair.gpu_interface)
                .map_or(-1, |index| index as i32),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edited_or_empty_reasons_do_not_grant_action_availability() {
        let mut actions = SelectedActions {
            verify: String::new(),
            repair: String::new(),
            remove: String::new(),
            forget: String::new(),
            verify_allowed: false,
            repair_allowed: true,
            remove_allowed: false,
            forget_allowed: false,
            recovery: true,
        };
        for label in [String::new(), String::from_iter(['λ', '\n', '{', '}'])] {
            actions.verify = label.clone();
            actions.repair = label;
            assert_eq!(actions.allowed("verify"), Some(false));
            assert_eq!(actions.allowed("repair"), Some(true));
            assert!(actions.recovery);
        }
        assert_eq!(actions.allowed(""), None);
    }
}
