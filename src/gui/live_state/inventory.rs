//! Inventory refresh, stable selection and target lookup.
use super::State;
use hyper_gpu_support::{gui_model::Inventory, model::Target};

impl State {
    /// Editing is available after completed setup for this exact VM/GPU pair.
    /// Enrollment or a host attachment alone does not establish guest setup.
    pub fn initial_setup_required(&self, id: &str) -> bool {
        let Ok(target) = self.target(id) else {
            return true;
        };
        !self.view.inventory.as_ref().is_some_and(|inventory| {
            inventory
                .managed
                .get(id)
                .and_then(Option::as_ref)
                .is_some_and(|journal| {
                    journal.vm_id == target.vm_id
                        && journal.gpu_interface == target.gpu_interface
                        && journal.prepared.is_some()
                        && journal.last_verified.is_some()
                })
        })
    }

    /// A snapshot supplies a proposal revision only; privileged enrollment still
    /// authenticates current artifacts/policy and rejects stale authority.
    pub fn permission_review(
        &self,
        target: &Target,
        operator: String,
    ) -> Result<Option<hyper_gpu_support::enrollment::PairReview>, String> {
        self.editor_eligibility(&target.vm_id)?;
        self.view.proposal_eligibility(target)?;
        let Some(observation) = &self.observation else {
            return Ok(None);
        };
        if self.observation_historical(&target.vm_id) {
            return Err("Inventory is historical. Refresh before requesting permissions.".into());
        }
        let inventory = self
            .view
            .inventory
            .as_ref()
            .ok_or("Inventory unavailable")?;
        let review = hyper_gpu_support::enrollment::PairReview {
            vm_id: target.vm_id.clone(),
            gpu_interface: target.gpu_interface.clone(),
            expected: observation.authority.clone(),
            operator_sid: operator,
        };
        review.target()?;
        if matches!(
            review.expected,
            hyper_gpu_support::configuration_store::Revision::Missing
        ) || inventory
            .enrolled
            .iter()
            .any(|current| current.vm_id.eq_ignore_ascii_case(&target.vm_id))
        {
            return Err("Pair authority is absent or this VM is already enrolled. Refresh before requesting permissions.".into());
        }
        Ok(Some(review))
    }

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
        let selected = self.view.selected.clone();
        if let Some(draft) = &self.view.draft
            && self.enrolled(draft)
            && !inventory
                .enrolled
                .iter()
                .any(|t| t.vm_id == draft.vm_id && t.gpu_interface == draft.gpu_interface)
        {
            self.authority_conflict = Some(draft.vm_id.clone());
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
