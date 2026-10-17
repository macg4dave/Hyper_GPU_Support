//! Durable recovery presentation and terminal-operation readback.
use super::{OperationReadback, State};
use hyper_gpu_support::model::Target;

impl State {
    /// Match the recovery path shown in review to the credential decision.
    /// These cached facts only choose presentation; the worker checks them fresh.
    pub fn recovery_without_guest(
        &self,
        record: &hyper_gpu_support::configuration_store::SaveRecord,
    ) -> Result<bool, String> {
        use hyper_gpu_support::configuration_store::SavePhase;
        if matches!(
            record.phase,
            SavePhase::Admitted | SavePhase::Verified(_) | SavePhase::Published(_)
        ) {
            // Admitted proves no guest/Hyper-V effect was dispatched. The worker
            // still requires fresh, unchanged starting facts before ending it;
            // stale presentation must never cause an unnecessary guest sign-in.
            return Ok(true);
        }
        if self.observation_historical(&record.target.vm_id) {
            return Ok(false);
        }
        let vm = self
            .view
            .vm(&record.target.vm_id)
            .ok_or("Recovery VM unavailable")?;
        let inventory = self
            .view
            .inventory
            .as_ref()
            .ok_or("Recovery inventory unavailable")?;
        let journal = inventory
            .managed
            .get(&record.target.vm_id)
            .and_then(Option::as_ref);
        let driver = inventory
            .discovery
            .gpus
            .iter()
            .find(|gpu| gpu.interface == record.target.gpu_interface)
            .map(|gpu| gpu.driver_version.as_str())
            .unwrap_or_default();
        let revision = self.committed.revision(&record.target.vm_id)?;
        Ok(record
            .validate_restored_baseline(vm, driver, &revision, journal)
            .is_ok()
            || record
                .negative_completion(vm, driver, &revision, journal)
                .is_ok_and(|candidate| candidate.is_some()))
    }

    /// A terminal Apply result is distinct from current saved files and observations.
    /// Retain known applied intent even when subsequent protected reads fail.
    pub fn finish_apply(
        &mut self,
        target: &Target,
        operation_id: &str,
        saved: bool,
        readback: Result<OperationReadback, String>,
    ) -> Result<(), String> {
        self.require_authenticated_review = false;
        self.view.needs_readback = true;
        self.approved_plan = None;
        self.view.applied(target)?;
        if saved {
            self.view.published(target)?;
        }
        self.finish_readback(readback)?;
        if let Some(record) = self.recovery_for_vm(&target.vm_id) {
            if record.operation_id != operation_id || record.target != *target {
                self.view.needs_readback = true;
                return Err(
                    "The protected recovery receipt changed after completion. Inspect the loaded operation before any retry.".into(),
                );
            }
        } else if !saved && self.view.unsaved {
            self.view.needs_readback = true;
            return Err("Applied changes were confirmed, but their bound save-only receipt is unavailable. Refresh and inspect recovery; do not repeat Apply.".into());
        }
        if saved
            && (self.committed.revision(&target.vm_id).is_err()
                || self
                    .committed
                    .documents
                    .get(&target.vm_id)
                    .is_none_or(|d| d.target != *target))
        {
            self.view.needs_readback = true;
            return Err("Publication completed, but current saved intent changed or is unreadable. Refresh before editing; do not repeat GPU work.".into());
        }
        Ok(())
    }

    pub fn finish_readback(
        &mut self,
        readback: Result<OperationReadback, String>,
    ) -> Result<(), String> {
        self.view.needs_readback = true;
        self.cancel_review();
        let readback = readback?;
        let mut committed = readback.committed;
        let mut recovery = readback.recovery;
        let mut selected_error = None;
        let (import, enrollment) = if let Some(observation) = readback.observation {
            observation.validate()?;
            let selected = observation
                .provenance
                .selected
                .as_ref()
                .ok_or("Completion identity missing")?;
            if observation.provenance.mode != hyper_gpu_support::observation::Mode::Completion
                || self.view.selected.as_deref() != Some(selected.vm_id.as_str())
            {
                return Err("Completion identity differs from the selected VM".into());
            }
            if !observation
                .inventory
                .discovery
                .vms
                .iter()
                .any(|vm| vm.vm_id == selected.vm_id)
                || !observation
                    .inventory
                    .discovery
                    .gpus
                    .iter()
                    .any(|gpu| gpu.interface == selected.gpu_interface)
            {
                selected_error = Some("Selected VM/GPU completion read is unavailable; Refresh before another operation".to_owned());
            }
            let inventory = observation.inventory_with_display(self.view.inventory.as_ref())?;
            let selected_id = selected.vm_id.clone();
            self.refresh(inventory);
            self.view.selected = Some(selected_id);
            self.worker_available = observation.worker_available;
            self.observation = Some(observation.provenance);
            committed = observation.committed;
            recovery = observation.recovery;
            (observation.import, observation.enrollment)
        } else {
            (
                self.import_recovery.clone(),
                self.enrollment_recovery.clone(),
            )
        };
        let configuration_error = self.load_committed(committed).err();
        self.sync_recovery(recovery, import, enrollment)?;
        if let Some(error) = selected_error.or(configuration_error) {
            self.view.needs_readback = true;
            return Err(error);
        }
        Ok(())
    }

    /// Only the loaded, completed publication receipt can be offered for save-only.
    pub fn save_only_record(
        &self,
    ) -> Result<hyper_gpu_support::configuration_store::SaveRecord, String> {
        use hyper_gpu_support::configuration_store::SavePhase;
        let record = self
            .recovery
            .as_ref()
            .ok_or("No bound save-only receipt is loaded; Refresh and inspect recovery.")?;
        if !record.publication_required
            || !matches!(
                record.phase,
                SavePhase::Verified(_) | SavePhase::Published(_)
            )
        {
            return Err(
                "This operation requires reconciliation, not a configuration save retry.".into(),
            );
        }
        Ok(record.clone())
    }

    /// Synchronize presentation only after successful inventory/store/record reads.
    /// Record absence alone cannot prove that verified intent was published.
    pub fn sync_recovery(
        &mut self,
        recovery: impl IntoIterator<Item = hyper_gpu_support::configuration_store::SaveRecord>,
        import_recovery: Option<hyper_gpu_support::configuration_store::ImportRecord>,
        enrollment_recovery: Option<hyper_gpu_support::enrollment::PendingPair>,
    ) -> Result<(), String> {
        use hyper_gpu_support::configuration_store::SavePhase;
        self.removal_review = None;
        self.recoveries = recovery.into_iter().collect();
        self.recovery = self
            .recoveries
            .iter()
            .find(|r| Some(r.target.vm_id.as_str()) == self.view.selected.as_deref())
            .or_else(|| self.recoveries.first())
            .cloned();
        self.import_recovery = import_recovery;
        self.enrollment_recovery = enrollment_recovery;
        self.approved_plan = None;
        self.verification_plan = None;
        if let Some(record) = self
            .recovery_for_vm(self.view.selected.as_deref().unwrap_or_default())
            .cloned()
        {
            self.view.unsaved =
                record.publication_required && matches!(record.phase, SavePhase::Verified(_));
            self.view.verified_unsaved = self.view.unsaved.then(|| record.target.clone());
            return Ok(());
        }
        if let Some(target) = &self.view.verified_unsaved {
            if self.recovery_for_vm(&target.vm_id).is_some() {
                // Its durable receipt retains this VM's save-only hold while another is selected.
                self.view.unsaved = false;
                self.view.verified_unsaved = None;
                return Ok(());
            }
            let published = self.committed.revision(&target.vm_id).is_ok()
                && self
                    .committed
                    .documents
                    .get(&target.vm_id)
                    .is_some_and(|document| document.target == *target);
            if !published {
                self.view.needs_readback = true;
                return Err(
                    "Recovery record is absent, but verified intent is not confirmed in readable committed configuration. Refresh and inspect before another operation.".into(),
                );
            }
        } else if self.view.unsaved {
            self.view.needs_readback = true;
            return Err(
                "Unsaved outcome has no verified target. Refresh and inspect before another operation.".into(),
            );
        }
        // Do not use View::published here: an external save must preserve any draft.
        self.view.unsaved = false;
        self.view.verified_unsaved = None;
        Ok(())
    }

    pub fn recovery_for_vm(
        &self,
        id: &str,
    ) -> Option<&hyper_gpu_support::configuration_store::SaveRecord> {
        self.recovery
            .iter()
            .chain(self.recoveries.iter())
            .find(|r| r.target.vm_id.eq_ignore_ascii_case(id))
    }

    pub fn blocks_vm(&self, id: &str) -> bool {
        self.recovery_for_vm(id).is_some()
            || self.import_recovery.is_some()
            || self.enrollment_recovery.is_some()
    }

    /// Explain recovery before offering actions that would be blocked by it.
    pub fn action_recovery_message(&self) -> Option<String> {
        let (vm_id, fallback) = if let Some(record) = &self.enrollment_recovery {
            (Some(record.review.vm_id.as_str()), None)
        } else if self.import_recovery.is_some() {
            (None, None)
        } else if let Some(record) =
            self.recovery_for_vm(self.view.selected.as_deref().unwrap_or_default())
        {
            (
                Some(record.target.vm_id.as_str()),
                Some(record.initial.name.as_str()),
            )
        } else {
            let id = self.view.selected.as_deref()?;
            if !self
                .view
                .inventory
                .as_ref()?
                .recorded_state(id)
                .recovery_required
            {
                return None;
            }
            (Some(id), None)
        };
        let operation = vm_id.map_or_else(
            || hyper_gpu_support::ui_text::operations::recovery::IMPORT_UNFINISHED.to_owned(),
            |id| {
                let name = self
                    .view
                    .vm(id)
                    .map(|vm| vm.name.as_str())
                    .or(fallback)
                    .unwrap_or(id);
                hyper_gpu_support::ui_text::operations::recovery::vm_unfinished(name).into_string()
            },
        );
        Some(format!(
            "{operation}\n\nBefore verifying, repairing, removing GPU support or deleting an app profile, review what happened. Choose Review recovery to see the recorded operation and the available next step. Your edits are kept. Reviewing recovery does not make changes; any available recovery action needs separate confirmation."
        ))
    }

    pub fn applied_text(&self, id: &str) -> String {
        use hyper_gpu_support::configuration_store::SavePhase;
        if let Some(record) = self.recovery_for_vm(id) {
            return match record.phase {
                SavePhase::Admitted | SavePhase::EffectsStarted => {
                    hyper_gpu_support::ui_text::operations::outcome::UNCERTAIN
                }
                SavePhase::Verified(_) if record.publication_required => {
                    hyper_gpu_support::ui_text::operations::outcome::PUBLICATION_PENDING
                }
                SavePhase::Published(_) => {
                    hyper_gpu_support::ui_text::operations::outcome::FINALIZATION_PENDING
                }
                SavePhase::Verified(_) => {
                    hyper_gpu_support::ui_text::operations::outcome::VERIFY_FINALIZATION_PENDING
                }
            }
            .into();
        }
        if self
            .view
            .verified_unsaved
            .as_ref()
            .is_some_and(|t| t.vm_id == id)
        {
            hyper_gpu_support::ui_text::operations::outcome::SAVE_UNKNOWN.into()
        } else {
            hyper_gpu_support::ui_text::operations::outcome::ABSENT.into()
        }
    }
}
