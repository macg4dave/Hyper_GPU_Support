//! Durable recovery presentation and terminal-operation readback.
use super::State;
use hyper_gpu_support::model::Target;
use hyper_gpu_support::observation::{Bundle, Mode};

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
            .ok_or(hyper_gpu_support::ui_text::operations::recovery::vm_record_unavailable())?;
        let inventory = self
            .view
            .inventory
            .as_ref()
            .ok_or(hyper_gpu_support::ui_text::operations::recovery::inventory_unavailable())?;
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
        readback: Result<Bundle, String>,
    ) -> Result<(), String> {
        self.view.needs_readback = true;
        // A terminal result belongs to the submitted intent, not a newer edit.
        let draft = self.view.draft.clone().filter(|draft| draft != target);
        self.view.applied(target)?;
        if saved {
            self.view.published(target)?;
        }
        if draft.is_some() {
            self.view.draft = draft;
        }
        self.finish_readback(readback)?;
        if let Some(record) = self.recovery_for_vm(&target.vm_id) {
            if record.operation_id != operation_id || record.target != *target {
                self.view.needs_readback = true;
                return Err(
                    hyper_gpu_support::ui_text::operations::recovery::receipt_changed().into(),
                );
            }
        } else if !saved && self.view.unsaved {
            self.view.needs_readback = true;
            return Err(
                hyper_gpu_support::ui_text::operations::recovery::receipt_unavailable().into(),
            );
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
            return Err(
                hyper_gpu_support::ui_text::operations::recovery::saved_intent_changed().into(),
            );
        }
        Ok(())
    }

    pub fn finish_readback(&mut self, readback: Result<Bundle, String>) -> Result<(), String> {
        self.view.needs_readback = true;
        self.result_notice = None;
        let observation = readback?;
        let inventory = observation.inventory_with_display(self.view.inventory.as_ref())?;
        let selected = observation
            .provenance
            .selected
            .as_ref()
            .ok_or(hyper_gpu_support::ui_text::operations::recovery::identity_missing())?;
        if observation.provenance.mode != Mode::Completion
            || self.view.selected.as_deref() != Some(selected.vm_id.as_str())
        {
            return Err(
                hyper_gpu_support::ui_text::operations::recovery::identity_differs().into(),
            );
        }
        let selected_error = (!observation
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
                .any(|gpu| gpu.interface == selected.gpu_interface))
        .then(|| {
            hyper_gpu_support::ui_text::operations::recovery::completion_unavailable().to_owned()
        });
        let selected_id = selected.vm_id.clone();
        self.refresh(inventory);
        self.view.selected = Some(selected_id);
        self.worker_available = observation.worker_available;
        self.observation = Some(observation.provenance);
        let configuration_error = self.load_committed(observation.committed).err();
        self.sync_recovery(
            observation.recovery,
            observation.import,
            observation.enrollment,
        )?;
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
            .recovery_for_vm(self.view.selected.as_deref().unwrap_or_default())
            .ok_or(hyper_gpu_support::ui_text::operations::recovery::no_receipt())?;
        if !record.publication_required
            || !matches!(
                record.phase,
                SavePhase::Verified(_) | SavePhase::Published(_)
            )
        {
            return Err(
                hyper_gpu_support::ui_text::operations::recovery::reconciliation_only().into(),
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
        self.recoveries = recovery.into_iter().collect();
        self.import_recovery = import_recovery;
        self.enrollment_recovery = enrollment_recovery;
        self.update_recovery()
    }

    /// Reconcile selection-dependent presentation from the one receipt collection.
    pub(super) fn update_recovery(&mut self) -> Result<(), String> {
        use hyper_gpu_support::configuration_store::SavePhase;
        if let Some(record) =
            self.recovery_for_vm(self.view.selected.as_deref().unwrap_or_default())
        {
            let unsaved = (record.publication_required
                && matches!(record.phase, SavePhase::Verified(_)))
            .then(|| record.target.clone());
            self.view.unsaved = unsaved.is_some();
            self.view.verified_unsaved = unsaved;
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
                    hyper_gpu_support::ui_text::operations::recovery::intent_unconfirmed().into(),
                );
            }
        } else if self.view.unsaved {
            self.view.needs_readback = true;
            return Err(
                hyper_gpu_support::ui_text::operations::recovery::no_verified_target().into(),
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
        self.recoveries
            .iter()
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
            || hyper_gpu_support::ui_text::operations::recovery::import_unfinished().to_owned(),
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
        Some(
            hyper_gpu_support::ui_text::operations::recovery::review_prompt(&operation)
                .into_string(),
        )
    }

    pub fn applied_text(&self, id: &str) -> String {
        use hyper_gpu_support::configuration_store::SavePhase;
        if let Some(record) = self.recovery_for_vm(id) {
            return match record.phase {
                SavePhase::Admitted | SavePhase::EffectsStarted => {
                    hyper_gpu_support::ui_text::operations::outcome::uncertain()
                }
                SavePhase::Verified(_) if record.publication_required => {
                    hyper_gpu_support::ui_text::operations::outcome::publication_pending()
                }
                SavePhase::Published(_) => {
                    hyper_gpu_support::ui_text::operations::outcome::finalization_pending()
                }
                SavePhase::Verified(_) => {
                    hyper_gpu_support::ui_text::operations::outcome::verify_finalization_pending()
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
            hyper_gpu_support::ui_text::operations::outcome::save_unknown().into()
        } else {
            hyper_gpu_support::ui_text::operations::outcome::absent().into()
        }
    }
}
