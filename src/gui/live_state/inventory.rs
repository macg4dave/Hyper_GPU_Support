//! Inventory refresh, stable selection and target lookup.
use super::State;
use hyper_gpu_support::{gui_model::Inventory, model::Target};

impl State {
    /// A selected completion refreshes only its VM; other rows await a poll.
    pub fn observation_historical(&self, id: &str) -> bool {
        self.view.needs_readback
            || self.observation.as_ref().is_some_and(|provenance| {
                provenance.mode == hyper_gpu_support::observation::Mode::Completion
                    && provenance
                        .selected
                        .as_ref()
                        .is_none_or(|target| target.vm_id != id)
            })
    }

    /// First-install discovery is explicit; it has no protected enrollment to poll.
    pub fn can_poll(&self) -> bool {
        self.view
            .inventory
            .as_ref()
            .is_some_and(|inventory| !inventory.enrolled.is_empty())
    }

    pub fn refresh(&mut self, inventory: Inventory) {
        self.removal_review = None;
        let selected = self.view.selected.clone();
        if let Some(draft) = &self.view.draft
            && self.enrolled(draft)
            && !inventory
                .enrolled
                .iter()
                .any(|t| t.vm_id == draft.vm_id && t.gpu_interface == draft.gpu_interface)
        {
            self.authority_conflict = Some(draft.vm_id.clone());
            self.approved_plan = None;
            self.enrollment_review = None;
            self.setup_artifacts = None;
        }
        self.view.refresh(inventory);
        if self.view.draft.is_some() {
            // A disappeared VM must not silently transfer its draft to another VM.
            self.view.selected = selected;
        }
    }

    pub fn target(&self, id: &str) -> Result<Target, String> {
        self.view
            .configuration
            .targets
            .iter()
            .find(|t| t.vm_id == id)
            .cloned()
            .or_else(|| {
                self.view
                    .inventory
                    .as_ref()?
                    .enrolled
                    .iter()
                    .find(|t| t.vm_id == id)
                    .cloned()
            })
            .or_else(|| {
                // A discovered candidate is unsaved, disabled intent, never authority.
                let inventory = self.view.inventory.as_ref()?;
                let vm = self.view.vm(id)?;
                let gpu = if vm.gpus.is_empty() {
                    inventory.discovery.gpus.first()?
                } else {
                    inventory
                        .discovery
                        .gpus
                        .iter()
                        .find(|g| vm.gpus == [g.interface.clone()])?
                };
                Some(Target {
                    compute: None,
                    encode: None,
                    decode: None,
                    vm_id: id.into(),
                    gpu_interface: gpu.interface.clone(),
                    enabled: false,
                    vram: None,
                })
            })
            .ok_or_else(|| "Select an available VM and physical GPU, then Refresh.".into())
    }

    pub fn enrolled(&self, target: &Target) -> bool {
        self.view.inventory.as_ref().is_some_and(|i| {
            i.enrolled
                .iter()
                .any(|t| t.vm_id == target.vm_id && t.gpu_interface == target.gpu_interface)
        })
    }

    pub fn select_vm(&mut self, id: Option<String>) -> Result<(), String> {
        self.view.selected = id;
        if !self.recoveries.is_empty() {
            self.sync_recovery(
                self.recoveries.clone(),
                self.import_recovery.clone(),
                self.enrollment_recovery.clone(),
            )?;
        }
        Ok(())
    }
}
