//! Live presentation decisions reuse protected enrollment and shared validation.
use hyper_gpu_support::{
    gui_model::{Inventory, View},
    model::{Allocation, Configuration, Resource, Target},
    workflow::Plan,
};

pub(super) struct State {
    pub view: View,
    pub pending_selection: Option<String>,
    pub draft_gpu: Option<String>,
    pub approved_plan: Option<ReviewedPlan>,
    pub removal_review: Option<hyper_gpu_support::enrollment::RemovalReview>,
    pub verification_plan: Option<hyper_gpu_support::workflow::VerificationPlan>,
    pub committed: hyper_gpu_support::configuration_store::StoreSnapshot,
    /// A draft's original committed revision changed; only discard releases it.
    committed_conflict: Option<String>,
    authority_conflict: Option<String>,
    pub recovery: Option<hyper_gpu_support::configuration_store::SaveRecord>,
    pub import_recovery: Option<hyper_gpu_support::configuration_store::ImportRecord>,
    pub enrollment_review: Option<hyper_gpu_support::enrollment::PairReview>,
    pub enrollment_recovery: Option<hyper_gpu_support::enrollment::PendingPair>,
    pub enrollment_recovery_revision: Option<hyper_gpu_support::configuration_store::Revision>,
    pub setup_artifacts: Option<std::collections::BTreeMap<String, String>>,
}

/// The GPU effect review also binds the exact committed intent it was built against.
#[derive(Clone)]
pub(super) struct ReviewedPlan {
    pub plan: Plan,
    pub revision: hyper_gpu_support::configuration_store::Revision,
}

/// Protected reads gathered off the UI thread after an operation attempt.
pub(super) struct OperationReadback {
    pub committed: hyper_gpu_support::configuration_store::StoreSnapshot,
    pub recovery: Option<hyper_gpu_support::configuration_store::SaveRecord>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            view: View::new(Configuration {
                schema: 2,
                targets: vec![],
            }),
            pending_selection: None,
            draft_gpu: None,
            approved_plan: None,
            removal_review: None,
            verification_plan: None,
            committed: Default::default(),
            committed_conflict: None,
            authority_conflict: None,
            recovery: None,
            import_recovery: None,
            enrollment_review: None,
            enrollment_recovery: None,
            enrollment_recovery_revision: None,
            setup_artifacts: None,
        }
    }
}

impl State {
    /// First-install discovery is explicit; it has no protected enrollment to poll.
    pub fn can_poll(&self) -> bool {
        self.view
            .inventory
            .as_ref()
            .is_some_and(|inventory| !inventory.enrolled.is_empty())
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
        self.view.needs_readback = true;
        self.approved_plan = None;
        self.view.applied(target)?;
        if saved {
            self.view.published(target)?;
        }
        self.finish_readback(readback)?;
        if let Some(record) = &self.recovery {
            if record.operation_id != operation_id || record.target != *target {
                return Err("The protected recovery receipt changed after completion. Inspect the loaded operation before any retry.".into());
            }
        } else if !saved && self.view.unsaved {
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
        let configuration_error = self.load_committed(readback.committed).err();
        self.sync_recovery(
            readback.recovery,
            self.import_recovery.clone(),
            self.enrollment_recovery.clone(),
        )?;
        configuration_error.map_or(Ok(()), Err)
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

    pub fn applied_text(&self, id: &str) -> String {
        use hyper_gpu_support::configuration_store::SavePhase;
        if let Some(record) = self.recovery.as_ref().filter(|r| r.target.vm_id == id) {
            return match record.phase {
                SavePhase::Admitted | SavePhase::EffectsStarted => "Operation outcome uncertain; inspect protected recovery.",
                SavePhase::Verified(_) if record.publication_required => "Applied and checked; configuration publication is pending. Use the bound save-only retry.",
                SavePhase::Published(_) => "Applied and published; protected recovery finalization is pending.",
                SavePhase::Verified(_) => "Graphics check completed; protected recovery finalization is pending. No configuration publication is authorized.",
            }.into();
        }
        if self
            .view
            .verified_unsaved
            .as_ref()
            .is_some_and(|t| t.vm_id == id)
        {
            "Applied changes were checked; saving is not confirmed. Refresh for the bound receipt."
                .into()
        } else {
            "No unpublished operation is recorded; see current observations and saved intent separately.".into()
        }
    }

    /// Current readable publication, separately from an earlier terminal reply.
    pub fn publication_confirmed(&self, target: &Target) -> bool {
        !self.view.unsaved
            && self.committed.revision(&target.vm_id).is_ok()
            && self
                .committed
                .documents
                .get(&target.vm_id)
                .is_some_and(|d| d.target == *target)
    }
    pub fn accept_review(
        &mut self,
        plan: Plan,
        current: &Target,
        revision: hyper_gpu_support::configuration_store::Revision,
    ) -> Result<(), String> {
        self.approved_plan = None;
        self.editor_eligibility(&current.vm_id)?;
        self.view.eligibility(current)?;
        if self.view.selected.as_deref() != Some(current.vm_id.as_str())
            || plan.desired != *current
            || plan.observed.vm_id != current.vm_id
            || self.committed.revision(&current.vm_id)? != revision
        {
            return Err(
                "The target, draft or saved settings changed while planning. Review again.".into(),
            );
        }
        self.approved_plan = Some(ReviewedPlan { plan, revision });
        Ok(())
    }

    pub fn reviewed_plan(&self, current: &Target) -> Result<ReviewedPlan, String> {
        self.editor_eligibility(&current.vm_id)?;
        self.view.eligibility(current)?;
        let reviewed = self
            .approved_plan
            .as_ref()
            .ok_or("Review the current draft before Apply")?;
        if self.view.selected.as_deref() != Some(current.vm_id.as_str())
            || *current != reviewed.plan.desired
            || self.committed.revision(&current.vm_id)? != reviewed.revision
        {
            return Err(
                "The target, draft or saved settings changed after review. Review again.".into(),
            );
        }
        Ok(reviewed.clone())
    }

    pub fn cancel_review(&mut self) {
        self.removal_review = None;
        self.approved_plan = None;
        self.enrollment_review = None;
        self.setup_artifacts = None;
        self.verification_plan = None;
        self.enrollment_recovery_revision = None;
    }

    /// Native byte requests; physical capacity never rescales provider intent.
    pub fn memory_slider(
        &self,
        gpu_index: i32,
        values: &[String],
    ) -> Result<super::memory::MemorySlider, String> {
        let gpu = self
            .view
            .inventory
            .as_ref()
            .and_then(|inventory| {
                usize::try_from(gpu_index)
                    .ok()
                    .and_then(|index| inventory.discovery.gpus.get(index))
            })
            .ok_or("Selected GPU is unavailable. Use Refresh to retry.")?;
        let limits = gpu
            .limits(Resource::Vram)
            .ok_or("VRAM bounds are unavailable. Use Refresh to retry.")?;
        let current =
            super::memory::request(values.get(..3).ok_or("VRAM fields are unavailable.")?)?;
        let capacity = gpu.physical_vram_bytes.filter(|bytes| *bytes > 0)
            .ok_or("Physical VRAM capacity could not be matched to the selected GPU. Use raw Advanced Allocation within the provider range.")?;
        super::memory::MemorySlider::new(limits, current.as_ref(), capacity)
    }

    /// Edit only the VRAM request; saved intent and compatibility categories stay intact.
    pub fn memory_values(
        &self,
        id: &str,
        gpu_index: i32,
        values: &[String],
        position: i32,
    ) -> Result<Vec<String>, String> {
        self.editor_eligibility(id)?;
        if values.len() != 12 {
            return Err("Expected four complete allocation triples".into());
        }
        let allocation = self.memory_slider(gpu_index, values)?.select(position)?;
        let mut edited = values.to_vec();
        edited[0] = allocation.minimum.to_string();
        edited[1] = allocation.optimal.to_string();
        edited[2] = allocation.maximum.to_string();
        Ok(edited)
    }

    /// Compare intent without treating validation, navigation or formatting as edits.
    pub fn draft_changed(
        &self,
        baseline: &Target,
        enabled: bool,
        gpu: Option<&str>,
        values: &[String],
    ) -> bool {
        let baseline_values = allocation_values(baseline);
        let same_values = values.len() == baseline_values.len()
            && values.iter().zip(&baseline_values).all(|(a, b)| {
                a.trim() == b
                    || a.trim()
                        .parse::<u64>()
                        .ok()
                        .zip(b.parse::<u64>().ok())
                        .is_some_and(|(a, b)| a == b)
            });
        enabled != baseline.enabled || gpu != Some(baseline.gpu_interface.as_str()) || !same_values
    }

    /// Synchronize presentation only after successful inventory/store/record reads.
    /// Record absence alone cannot prove that verified intent was published.
    pub fn sync_recovery(
        &mut self,
        recovery: Option<hyper_gpu_support::configuration_store::SaveRecord>,
        import_recovery: Option<hyper_gpu_support::configuration_store::ImportRecord>,
        enrollment_recovery: Option<hyper_gpu_support::enrollment::PendingPair>,
    ) -> Result<(), String> {
        use hyper_gpu_support::configuration_store::SavePhase;
        self.removal_review = None;
        self.recovery = recovery;
        self.import_recovery = import_recovery;
        self.enrollment_recovery = enrollment_recovery;
        self.approved_plan = None;
        self.verification_plan = None;
        if let Some(record) = &self.recovery {
            self.view.unsaved =
                record.publication_required && matches!(record.phase, SavePhase::Verified(_));
            self.view.verified_unsaved = self.view.unsaved.then(|| record.target.clone());
            return Ok(());
        }
        if let Some(target) = &self.view.verified_unsaved {
            let published = self.committed.revision(&target.vm_id).is_ok()
                && self
                    .committed
                    .documents
                    .get(&target.vm_id)
                    .is_some_and(|document| document.target == *target);
            if !published {
                self.view.needs_readback = true;
                return Err("Recovery record is absent, but verified intent is not confirmed in readable committed configuration. Refresh and inspect before another operation.".into());
            }
        } else if self.view.unsaved {
            self.view.needs_readback = true;
            return Err("Unsaved outcome has no verified target. Refresh and inspect before another operation.".into());
        }
        // Do not use View::published here: an external save must preserve any draft.
        self.view.unsaved = false;
        self.view.verified_unsaved = None;
        Ok(())
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

    fn proposal_eligibility(&self, target: &Target) -> Result<(), String> {
        if !self.enrolled(target)
            && self
                .view
                .inventory
                .as_ref()
                .is_some_and(|i| i.enrolled.iter().any(|t| t.vm_id == target.vm_id))
        {
            return Err("This selected GPU is not enrolled. Changing an enrolled GPU requires the separate replacement transition, which is not connected yet.".into());
        }
        self.view.draft_eligibility(target)
    }

    /// Explain recovery before offering actions that would be blocked by it.
    pub fn action_recovery_message(&self) -> Option<String> {
        let (vm_id, fallback) = if let Some(record) = &self.enrollment_recovery {
            (Some(record.review.vm_id.as_str()), None)
        } else if self.import_recovery.is_some() {
            (None, None)
        } else if let Some(record) = &self.recovery {
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
            || "A configuration import did not finish.".to_owned(),
            |id| {
                let name = self
                    .view
                    .vm(id)
                    .map(|vm| vm.name.as_str())
                    .or(fallback)
                    .unwrap_or(id);
                format!("An earlier operation for {name} did not finish.")
            },
        );
        Some(format!(
            "{operation}\n\nBefore verifying, repairing, removing GPU support or forgetting a pairing, review what happened. Choose Review recovery to see the recorded operation and the available next step. Your edits are kept. Reviewing recovery does not make changes; any available recovery action needs separate confirmation."
        ))
    }

    pub fn editor_eligibility(&self, id: &str) -> Result<(), String> {
        if self.authority_conflict.as_deref() == Some(id) {
            return Err("Protected enrollment changed. Draft preserved; discard and Refresh before a new enrollment review.".into());
        }
        if self.committed_conflict.as_deref() == Some(id) {
            return Err("Committed configuration changed or became unavailable. Draft preserved; discard it to reload the latest committed intent.".into());
        }
        if self.recovery.is_some()
            || self.import_recovery.is_some()
            || self.enrollment_recovery.is_some()
        {
            return Err(
                "A host-wide operation requires manual reconciliation or save-only retry.".into(),
            );
        }
        if self
            .view
            .inventory
            .as_ref()
            .is_some_and(|inventory| inventory.recorded_state(id).recovery_required)
        {
            return Err("The selected VM has pending or inconsistent managed recovery. Inspect and reconcile it before editing or reviewing new changes.".into());
        }
        self.committed.revision(id)?;
        let mut pair = self.target(id)?;
        // Pair eligibility must allow correcting a now-invalid old allocation or
        // disabling an unsupported preparation adapter. Draft validation is separate.
        pair.enabled = false;
        pair.clear_allocations();
        self.proposal_eligibility(&pair)
    }
    /// Accept fresh readable intent; an error means the retained draft is blocked,
    /// not that the new store snapshot or unrelated inventory should be discarded.
    pub fn load_committed(
        &mut self,
        committed: hyper_gpu_support::configuration_store::StoreSnapshot,
    ) -> Result<(), String> {
        if let Some(draft) = &self.view.draft {
            let unchanged = match (
                self.committed.revision(&draft.vm_id),
                committed.revision(&draft.vm_id),
            ) {
                (Ok(previous), Ok(current)) => previous == current,
                _ => false,
            };
            if !unchanged {
                self.committed_conflict = Some(draft.vm_id.clone());
                self.approved_plan = None;
            }
        }
        // Refresh all readable intent even when one draft conflicts. This never
        // changes the draft or grants enrollment, and the conflict remains latched.
        self.view.configuration = committed.configuration();
        self.committed = committed;
        if let Some(id) = &self.committed_conflict {
            return self.editor_eligibility(id);
        }
        Ok(())
    }

    pub fn discard(&mut self) {
        self.view.discard();
        self.committed_conflict = None;
        self.authority_conflict = None;
        self.cancel_review();
        self.draft_gpu = None;
    }

    pub fn saved_desired_text(&self, id: &str) -> String {
        match self.committed.revision(id) {
            Ok(_) => self.view.saved_desired_text(id),
            Err(_) => "Committed configuration unavailable; saved intent is unknown. Editing is blocked until the file can be read and validated.".into(),
        }
    }

    pub fn host_configuration_text(&self, id: &str) -> String {
        super::host_configuration::comparison(self, id)
    }

    pub fn draft(
        &self,
        id: &str,
        enabled: bool,
        gpu_index: i32,
        values: &[String],
    ) -> Result<Target, String> {
        self.editor_eligibility(id)?;
        let mut target = self.target(id)?;
        let inventory = self
            .view
            .inventory
            .as_ref()
            .ok_or("Refresh inventory first.")?;
        let gpu = usize::try_from(gpu_index)
            .ok()
            .and_then(|i| inventory.discovery.gpus.get(i))
            .ok_or("Select an available physical GPU.")?;
        target.gpu_interface = gpu.interface.clone();
        target.enabled = enabled;
        if values.len() != 12 {
            return Err("Expected four complete allocation triples".into());
        }
        for (resource, values) in Resource::ALL.into_iter().zip(values.chunks_exact(3)) {
            let value = if values.iter().all(|v| v.trim().is_empty()) {
                None
            } else {
                let parsed = values.iter().map(|v| v.trim().parse::<u64>()
                    .map_err(|_| format!("Enter all three {} values as unsigned provider integers, or leave all three empty.", resource.name())))
                    .collect::<Result<Vec<_>, _>>()?;
                Some(Allocation {
                    minimum: parsed[0],
                    optimal: parsed[1],
                    maximum: parsed[2],
                })
            };
            target.set_allocation(resource, value);
        }
        self.proposal_eligibility(&target)?;
        Ok(target)
    }

    /// Review host removal using the ordinary disabled draft and exact enrolled pair.
    pub fn removal_draft(
        &self,
        id: &str,
        gpu_index: i32,
        values: &[String],
    ) -> Result<Target, String> {
        if self.view.selected.as_deref() != Some(id) {
            return Err("The selected VM changed; review removal again.".into());
        }
        let target = self.draft(id, false, gpu_index, values)?;
        if !self.enrolled(&target) {
            return Err("GPU removal requires this exact authorized VM/GPU pair; no enrollment or guest work was performed.".into());
        }
        Ok(target)
    }

    /// Profile removal is separate from drafts and requires independently inactive host support.
    pub fn forget_target(&self) -> Result<Target, String> {
        let id = self.view.selected.as_deref().ok_or("Select a VM first.")?;
        if self.view.draft.is_some() || self.view.unsaved || self.view.needs_readback {
            return Err(
                "Discard or complete pending edits and Refresh before Forget pairing.".into(),
            );
        }
        if self.recovery.is_some()
            || self.import_recovery.is_some()
            || self.enrollment_recovery.is_some()
            || self.authority_conflict.is_some()
            || self.committed_conflict.is_some()
            || !self.committed.errors.is_empty()
        {
            return Err(
                "Resolve recovery or conflicting/unreadable records before Forget pairing.".into(),
            );
        }
        let inventory = self.view.inventory.as_ref().ok_or("Refresh first.")?;
        let mut target = inventory
            .enrolled
            .iter()
            .find(|t| t.vm_id == id)
            .cloned()
            .ok_or("This VM has no authorized pairing; Refresh for current state.")?;
        let vm = self.view.vm(id).ok_or("VM unavailable; Refresh first.")?;
        if inventory.recorded_state(id).recovery_required || !inventory.managed.contains_key(id) {
            return Err(
                "Complete GPU removal and Refresh its managed history before Forget pairing."
                    .into(),
            );
        }
        target.enabled = false;
        target.clear_allocations();
        hyper_gpu_support::enrollment::validate_inactive_profile(
            &target,
            vm,
            inventory.managed.get(id).and_then(Option::as_ref),
            self.committed.documents.get(id).map(|d| &d.target),
        )?;
        Ok(target)
    }

    fn check_forget_review(
        &self,
        review: &hyper_gpu_support::enrollment::RemovalReview,
    ) -> Result<(), String> {
        let target = self.forget_target()?;
        let mut revisions = self
            .committed
            .documents
            .iter()
            .map(|(id, d)| (id.clone(), d.revision.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        revisions
            .entry(target.vm_id.clone())
            .or_insert(hyper_gpu_support::configuration_store::Revision::Missing);
        if review.pair.target()? != target || review.configurations != revisions {
            return Err(
                "Selection, saved profiles or pairing changed; review Forget pairing again.".into(),
            );
        }
        Ok(())
    }

    pub fn accept_forget_review(
        &mut self,
        review: hyper_gpu_support::enrollment::RemovalReview,
    ) -> Result<(), String> {
        self.removal_review = None;
        self.check_forget_review(&review)?;
        self.removal_review = Some(review);
        Ok(())
    }

    pub fn reviewed_forget(&self) -> Result<hyper_gpu_support::enrollment::RemovalReview, String> {
        let review = self
            .removal_review
            .as_ref()
            .ok_or("Open a fresh Forget pairing review first.")?;
        self.check_forget_review(review)?;
        Ok(review.clone())
    }
}

pub(super) fn allocation_values(target: &Target) -> Vec<String> {
    let mut values = vec![String::new(); 12];
    for (index, resource) in Resource::ALL.into_iter().enumerate() {
        if let Some(value) = target.allocation(resource) {
            values[index * 3] = value.minimum.to_string();
            values[index * 3 + 1] = value.optimal.to_string();
            values[index * 3 + 2] = value.maximum.to_string();
        }
    }
    values
}

/// Review exact VRAM byte requests, separately from allocation enforcement.
pub(super) fn memory_review(plan: &Plan) -> String {
    if let Some(write) = &plan.preview.allocation {
        format!(
            "GPU memory (bytes: minimum / optimal / maximum): {} → {}\n",
            allocation_description(plan.observed.vram.as_ref(), "Unknown"),
            allocation_description(Some(write), "Unknown"),
        )
    } else if let Some(request) = &plan.desired.vram {
        format!(
            "GPU memory request (bytes: minimum / optimal / maximum): {}. No VRAM write is planned.\n",
            allocation_description(Some(request), "Unknown"),
        )
    } else {
        "GPU memory: no explicit request.\n".into()
    }
}

pub(super) fn review_text(plan: &Plan) -> String {
    let requested = allocation_description(
        plan.desired.vram.as_ref(),
        "Unspecified; no explicit VRAM request",
    );
    let planned = allocation_description(
        plan.preview.allocation.as_ref(),
        "No allocation write planned; existing/provider values are not explicitly reset",
    );
    let extended = [
        (Resource::Compute, plan.preview.compute.as_ref()),
        (Resource::Encode, plan.preview.encode.as_ref()),
        (Resource::Decode, plan.preview.decode.as_ref()),
    ]
    .into_iter()
    .map(|(resource, planned)| {
        format!(
            "{} draft (Minimum / Optimal / Maximum): {}\nPlanned {} write: {}",
            resource.name(),
            allocation_description(
                plan.desired.allocation(resource),
                "Unspecified; no explicit request"
            ),
            resource.name(),
            allocation_description(
                planned,
                "No explicit category write; new/recreated attachments use provider defaults"
            ),
        )
    })
    .collect::<Vec<_>>()
    .join("\n");
    format!(
        "VM: {}\nVM ID: {}\nGPU interface: {}\nDesired GPU support: {}\nVRAM draft (Minimum / Optimal / Maximum): {requested}\nPlanned allocation write: {planned}\n{extended}\n\n{}\n\nAllocation units: {}\nConfiguration publication: schema 3, retaining schema-2 input compatibility.\n\nPreview from the shared CLI planner. No VM or guest changes have been performed during review.",
        plan.observed.name,
        plan.desired.vm_id,
        plan.desired.gpu_interface,
        if plan.desired.enabled {
            "Enabled"
        } else {
            "Disabled"
        },
        plan.preview.summary.join("\n"),
        plan.allocation_units,
    )
}

/// User-facing before/after changes, derived from the same approved effect plan.
pub(super) fn review_summary(plan: &Plan) -> String {
    use hyper_gpu_support::workflow::PreviewAction;
    let mut lines = vec![
        format!("VM: {}", plan.observed.name),
        format!(
            "GPU support: {} → {}",
            if plan.observed.gpus.is_empty() {
                "Disabled"
            } else {
                "Enabled"
            },
            if plan.desired.enabled {
                "Enabled"
            } else {
                "Disabled"
            }
        ),
    ];
    if plan.reinstall {
        lines.insert(1, "Repair / Reinstall: run complete signed driver preparation even with matching history, verify current payload and selected-hardware rendering, then leave the VM off.".into());
    }
    if !plan.desired.enabled {
        lines.insert(1, "Remove GPU support: detach the exact managed GPU if present, restore still-owned compatibility values and leave the VM off. Keep guest driver files, authorization and setup history. No guest credentials or graphics check.".into());
    }
    if plan.initial_setup && !plan.reinstall {
        lines.insert(1, "Initial GPU setup: check hardware rendering, then gracefully leave the VM off. Start it through Hyper-V after setup passes.".into());
    }
    if !plan.initial_setup && plan.preview.credentials_required && !plan.reinstall {
        lines.insert(1, "GPU setup/update required: preparation changed or is invalid. Review driver work, hardware rendering and final shutdown before confirming.".into());
    } else if !plan.preview.credentials_required {
        lines.insert(1, "Ordinary GPU change: no guest access or graphics check. Leave the VM off after success; start it through Hyper-V when ready.".into());
    }
    for (resource, write) in [
        (Resource::Vram, plan.preview.allocation.as_ref()),
        (Resource::Compute, plan.preview.compute.as_ref()),
        (Resource::Encode, plan.preview.encode.as_ref()),
        (Resource::Decode, plan.preview.decode.as_ref()),
    ] {
        if let Some(write) = write {
            lines.push(format!(
                "{} (Minimum / Optimal / Maximum, raw provider values): {} → {}",
                resource.name(),
                allocation_description(plan.observed.allocation(resource), "Unknown"),
                allocation_description(Some(write), "Unknown")
            ));
        }
    }
    if let Some(settings) = &plan.preview.settings {
        for (name, before, after) in [
            (
                "Low MMIO (MiB)",
                settings.before.low_mmio.to_string(),
                settings.after.low_mmio.to_string(),
            ),
            (
                "High MMIO (MiB)",
                settings.before.high_mmio.to_string(),
                settings.after.high_mmio.to_string(),
            ),
            (
                "Guest cache types",
                settings.before.cache_types.to_string(),
                settings.after.cache_types.to_string(),
            ),
            (
                "Automatic checkpoints",
                settings.before.automatic_checkpoints.to_string(),
                settings.after.automatic_checkpoints.to_string(),
            ),
        ] {
            if before != after {
                lines.push(format!("{name}: {before} → {after}"));
            }
        }
    }
    lines.push("\nPlanned steps, in order:".into());
    if plan.preview.actions.is_empty() {
        lines.push("No VM or guest changes are needed. Save the reviewed intent.".into());
    }
    lines.extend(
        plan.preview
            .actions
            .iter()
            .enumerate()
            .map(|(index, action)| {
                let step = match action {
                    PreviewAction::Shutdown => "Gracefully stop the VM.",
                    PreviewAction::Start => "Start the VM.",
                    PreviewAction::Detach => "Remove the GPU attachment.",
                    PreviewAction::Prepare => {
                        "Prepare the guest with the current signed GPU driver files."
                    }
                    PreviewAction::Settings => "Update or restore GPU compatibility settings.",
                    PreviewAction::Attach => "Attach the selected physical GPU.",
                    PreviewAction::Allocation => "Set the GPU resources shown above.",
                    PreviewAction::Verify => "Check guest graphics health and hardware rendering.",
                };
                format!("{}. {step}", index + 1)
            }),
    );
    lines.push(format!(
        "\nExpected final VM power after successful completion: {}.",
        match plan.preview.restore_power {
            hyper_gpu_support::model::Power::Running => "Running",
            hyper_gpu_support::model::Power::Off => "Stopped",
            hyper_gpu_support::model::Power::Other(_) => "Unknown",
        }
    ));
    if plan.preview.guest_downtime {
        lines.push("The running VM will be interrupted. Save work inside it before Apply. Approval below is required.".into());
    }
    if plan.preview.actions.contains(&PreviewAction::Prepare) {
        lines.push("Driver preparation and file transfer can take several minutes.".into());
    }
    if plan.preview.actions.contains(&PreviewAction::Start)
        || plan.preview.actions.contains(&PreviewAction::Shutdown)
    {
        lines.push("Allow time for Windows to start or shut down gracefully. Shutdown refusal stops further changes.".into());
    }
    if plan.preview.actions.contains(&PreviewAction::Verify) {
        lines.push("The graphics check may take additional time and requires Windows credentials inside this VM.".into());
    }
    if !plan.preview.actions.is_empty() {
        lines.push("Completion time varies; no exact duration is promised. Keep the application open until it reports an outcome.".into());
    }
    if plan.preview.settings_preserved {
        lines.push("Externally changed compatibility settings will be preserved.".into());
    }
    if plan.preview.pending_recovery {
        lines.push("Interrupted work requires recovery before further changes.".into());
    }
    lines.join("\n")
}

pub(super) fn allocation_description(value: Option<&Allocation>, absent: &str) -> String {
    value.map_or_else(
        || absent.to_owned(),
        |v| format!("{} / {} / {}", v.minimum, v.optimal, v.maximum),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper_gpu_support::model::{Discovery, Gpu, Power, Settings, VmState};

    fn inventory() -> Inventory {
        let target = Target {
            compute: None,
            encode: None,
            decode: None,
            vm_id: "12345678-1234-1234-1234-123456789abc".into(),
            gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#fixture\GPUPARAV".into(),
            enabled: true,
            vram: None,
        };
        Inventory {
            discovery: Discovery {
                issues: vec![],
                vms: vec![VmState {
                    compute: None,
                    encode: None,
                    decode: None,
                    vm_id: target.vm_id.clone(),
                    name: "Renamable VM".into(),
                    power: Power::Off,
                    generation: 2,
                    gpus: vec![],
                    vram: None,
                    settings: Settings {
                        low_mmio: 0,
                        high_mmio: 0,
                        cache_types: false,
                        automatic_checkpoints: true,
                    },
                }],
                gpus: vec![Gpu {
                    compute: Default::default(),
                    encode: Default::default(),
                    decode: Default::default(),
                    interface: target.gpu_interface.clone(),
                    name: "Identical GPU name".into(),
                    vendor: 0x10de,
                    device: 1,
                    driver_version: "fixture".into(),
                    physical_vram_bytes: None,
                    total_vram: None,
                    preparation_supported: true,
                    vram: Allocation {
                        minimum: 10,
                        optimal: 50,
                        maximum: 100,
                    }
                    .into(),
                }],
            },
            enrolled: vec![target],
            managed: Default::default(),
        }
    }
    fn state() -> State {
        let mut s = State::default();
        s.refresh(inventory());
        s
    }

    #[test]
    fn draft_tracking_ignores_unchanged_values_and_retains_invalid_edits() {
        let s = state();
        let mut baseline = s.target(id(&s)).unwrap();
        baseline.vram = Some(Allocation {
            minimum: 10,
            optimal: 50,
            maximum: 100,
        });
        let mut values = allocation_values(&baseline);
        let gpu = Some(baseline.gpu_interface.as_str());
        assert!(!s.draft_changed(&baseline, baseline.enabled, gpu, &values));
        values[1] = " 050 ".into();
        assert!(!s.draft_changed(&baseline, baseline.enabled, gpu, &values));
        assert!(s.draft_changed(&baseline, !baseline.enabled, gpu, &values));
        values[1] = "invalid".into();
        assert!(s.draft_changed(&baseline, baseline.enabled, gpu, &values));
        values[1] = "50".into();
        assert!(!s.draft_changed(&baseline, baseline.enabled, gpu, &values));
    }

    #[test]
    fn retained_inventory_allows_memory_drafts_but_cannot_authorize_apply() {
        let mut s = state();
        s.view.inventory.as_mut().unwrap().discovery.gpus[0].vram = Allocation {
            minimum: 0,
            optimal: 1_000_000_000,
            maximum: 8_000_000_000,
        }
        .into();
        s.view.inventory.as_mut().unwrap().discovery.gpus[0].physical_vram_bytes =
            Some(8_000_000_000);
        s.view.needs_readback = true;
        assert!(s.memory_slider(0, &vec![String::new(); 12]).is_ok());
        let target = s.target(id(&s)).unwrap();
        assert!(s.editor_eligibility(id(&s)).is_ok());
        assert!(s.view.eligibility(&target).is_err());
    }

    #[test]
    fn memory_selection_and_advanced_fields_produce_the_same_draft() {
        let mut s = state();
        s.view.inventory.as_mut().unwrap().discovery.gpus[0].vram = Allocation {
            minimum: 1,
            optimal: 4_000_000_000,
            maximum: 8_000_000_000,
        }
        .into();
        s.view.inventory.as_mut().unwrap().discovery.gpus[0].physical_vram_bytes =
            Some(8_000_000_000);
        s.view.inventory.as_mut().unwrap().discovery.gpus[0].total_vram = Some(8_000_000_000);
        let mut raw = values("2000000000", "4000000000", "6000000000");
        let selected = s.memory_slider(0, &raw).unwrap().select(55).unwrap();
        raw = s.memory_values(id(&s), 0, &raw, 55).unwrap();
        let draft = s.draft(id(&s), true, 0, &raw).unwrap();
        assert_eq!(draft.vram, Some(selected));
        assert!(s.draft_changed(
            &s.target(id(&s)).unwrap(),
            true,
            Some(&draft.gpu_interface),
            &raw
        ));
        // Unsupported/incomplete raw text must remain intact rather than being
        // replaced with guessed defaults from the slider.
        raw[1] = "invalid".into();
        assert!(s.memory_slider(0, &raw).is_err());
        assert_eq!(raw[1], "invalid");
    }

    #[test]
    fn slider_capacity_follows_selected_gpu_and_unknown_capacity_stays_unknown() {
        let mut s = state();
        let gpus = &mut s.view.inventory.as_mut().unwrap().discovery.gpus;
        gpus[0].vram = Allocation {
            minimum: 0,
            optimal: 1_000_000_000,
            maximum: 1_000_000_000,
        }
        .into();
        gpus[0].physical_vram_bytes = Some(8_000_000_000);
        let mut second = gpus[0].clone();
        second.interface = "second physical GPU".into();
        second.physical_vram_bytes = Some(16_000_000_000);
        gpus.push(second);
        let raw = vec![String::new(); 12];
        assert_eq!(s.memory_slider(0, &raw).unwrap().range_start, "1 GB");
        assert_eq!(s.memory_slider(1, &raw).unwrap().range_start, "1 GB");
        assert_eq!(s.memory_slider(0, &raw).unwrap().maximum, 80);
        assert_eq!(s.memory_slider(1, &raw).unwrap().maximum, 160);
        assert_eq!(
            s.memory_slider(1, &raw)
                .unwrap()
                .select(20)
                .unwrap()
                .optimal,
            2_000_000_000
        );
        s.view.inventory.as_mut().unwrap().discovery.gpus[1].physical_vram_bytes = None;
        assert!(s.memory_slider(1, &raw).is_err());
        s.view.inventory.as_mut().unwrap().discovery.gpus[1].physical_vram_bytes = Some(0);
        assert!(s.memory_slider(1, &raw).is_err());
    }

    #[test]
    fn memory_edits_preserve_saved_schemas_other_categories_and_other_vms() {
        for schema in [2, 3] {
            let mut s = state();
            let vm_id = id(&s).to_owned();
            let gpu = &mut s.view.inventory.as_mut().unwrap().discovery.gpus[0];
            gpu.vram = Allocation {
                minimum: 0,
                optimal: 4_000_000_000,
                maximum: 8_000_000_000,
            }
            .into();
            gpu.physical_vram_bytes = Some(8_000_000_000);
            let compatible = Allocation {
                minimum: 10,
                optimal: 50,
                maximum: 100,
            };
            gpu.compute = compatible.clone().into();
            gpu.encode = compatible.clone().into();
            gpu.decode = compatible.clone().into();
            let mut saved = s.target(&vm_id).unwrap();
            saved.vram = Some(Allocation {
                minimum: 0,
                optimal: 0,
                maximum: 0,
            });
            if schema == 3 {
                saved.compute = Some(compatible.clone());
                saved.encode = Some(compatible.clone());
                saved.decode = Some(compatible);
            }
            let mut other = saved.clone();
            other.vm_id = "87654321-4321-4321-4321-cba987654321".into();
            let configuration = Configuration {
                schema,
                targets: vec![saved.clone(), other.clone()],
            };
            let mut store = hyper_gpu_support::configuration_store::StoreSnapshot::default();
            for (filename, source) in configuration.vm_documents().unwrap() {
                store.insert(&filename, Ok(source));
            }
            s.load_committed(store).unwrap();
            let source = s.committed.documents[&vm_id].source.clone();
            let original = allocation_values(&saved);
            assert_eq!(s.memory_slider(0, &original).unwrap().label, "0 GB");
            assert!(s.memory_values(&vm_id, 0, &original, 9).is_err());
            let edited = s.memory_values(&vm_id, 0, &original, 10).unwrap();
            assert_eq!(&edited[3..], &original[3..]);
            let enabled = s.draft(&vm_id, true, 0, &edited).unwrap();
            assert!(enabled.enabled);
            assert_eq!(enabled.vram.as_ref().unwrap().optimal, 1_000_000_000);
            assert_eq!(enabled.compute, saved.compute);
            assert_eq!(enabled.encode, saved.encode);
            assert_eq!(enabled.decode, saved.decode);
            // A zero or omitted request does not disable GPU support.
            assert!(s.draft(&vm_id, true, 0, &original).unwrap().enabled);
            assert!(
                s.draft(&vm_id, true, 0, &vec![String::new(); 12])
                    .unwrap()
                    .enabled
            );
            assert!(!s.draft(&vm_id, false, 0, &edited).unwrap().enabled);
            assert_eq!(s.target(&vm_id).unwrap(), saved);
            assert_eq!(s.committed.documents[&vm_id].source, source);
            assert_eq!(s.committed.documents[&other.vm_id].target, other);
            assert_eq!(s.view.vm(&vm_id).unwrap().power, Power::Off);
            assert!(s.view.vm(&vm_id).unwrap().gpus.is_empty());
            s.view.needs_readback = true;
            assert!(s.memory_values(&vm_id, 0, &original, 10).is_ok());
            assert!(s.view.eligibility(&enabled).is_err());
        }
    }

    #[test]
    fn new_vm_can_be_drafted_but_proposal_never_becomes_execution_authority() {
        let mut s = State::default();
        let mut i = inventory();
        i.enrolled.clear();
        s.refresh(i);
        let vm_id = id(&s).to_owned();
        let target = s
            .draft(&vm_id, true, 0, &values("10", "50", "100"))
            .unwrap();
        assert!(!s.enrolled(&target));
        assert!(s.view.configuration.targets.is_empty());
        assert!(!s.target(&vm_id).unwrap().enabled);
        assert!(
            s.view
                .eligibility(&target)
                .unwrap_err()
                .contains("not enrolled")
        );
        assert!(s.view.inventory.as_ref().unwrap().enrolled.is_empty());
        s.view.draft = Some(target.clone());
        s.refresh(inventory());
        assert_eq!(s.view.draft, Some(target.clone()));
        assert!(s.enrolled(&target));
        assert!(s.view.configuration.targets.is_empty());
        assert!(s.view.eligibility(&target).is_ok());
    }

    #[test]
    fn saved_intent_remains_distinct_from_enrollment_observation_and_draft() {
        let mut s = state();
        assert!(s.view.configuration.targets.is_empty());
        assert!(s.view.saved_desired_text(id(&s)).contains("No saved"));
        let mut candidate = s.target(id(&s)).unwrap();
        candidate.enabled = false;
        let configuration = Configuration {
            schema: 2,
            targets: vec![candidate.clone()],
        };
        s.view.configuration = configuration;
        assert!(!s.target(id(&s)).unwrap().enabled);
        assert!(s.view.inventory.as_ref().unwrap().enrolled[0].enabled);
        s.view.draft = Some(
            s.draft(id(&s), true, 0, &values("10", "50", "100"))
                .unwrap(),
        );
        s.refresh(inventory());
        assert_eq!(s.view.configuration.targets[0], candidate);
        assert!(s.view.draft.as_ref().unwrap().enabled);
        assert!(s.view.draft.is_some());
    }

    fn comparison_state(enabled: bool) -> (State, String, Target) {
        let mut s = state();
        let vm_id = id(&s).to_owned();
        let mut saved = s.target(&vm_id).unwrap();
        saved.enabled = enabled;
        saved.vram = Some(Allocation {
            minimum: 10,
            optimal: 50,
            maximum: 100,
        });
        s.load_committed(committed(vec![saved.clone()], "comparison"))
            .unwrap();
        (s, vm_id, saved)
    }

    #[test]
    fn host_comparison_uses_saved_intent_and_preserves_missing_attachment_repair() {
        let (mut s, vm_id, saved) = comparison_state(true);
        let before = s.committed.documents[&vm_id].source.clone();
        s.view.draft = Some(Target {
            enabled: false,
            ..saved.clone()
        });
        let text = s.host_configuration_text(&vm_id);
        assert!(text.contains(&format!("expected {}; observed none", saved.gpu_interface)));
        assert!(text.contains("observed Unknown"));
        assert!(s.editor_eligibility(&vm_id).is_ok());
        assert!(s.view.eligibility(&saved).is_ok());
        assert_eq!(s.committed.documents[&vm_id].source, before);
        assert!(!s.view.draft.as_ref().unwrap().enabled);
        assert!(s.view.vm(&vm_id).unwrap().gpus.is_empty());
        let inventory = s.view.inventory.as_mut().unwrap();
        inventory.discovery.vms[0].gpus = vec!["unrelated-gpu".into()];
        assert!(
            s.host_configuration_text(&vm_id)
                .contains("observed unrelated-gpu")
        );
        assert!(s.view.eligibility(&saved).is_err());
    }

    #[test]
    fn host_comparison_reports_exact_resources_and_managed_settings() {
        let (mut s, vm_id, saved) = comparison_state(true);
        let inventory = s.view.inventory.as_mut().unwrap();
        let vm = &mut inventory.discovery.vms[0];
        vm.gpus = vec![saved.gpu_interface.clone()];
        vm.vram = saved.vram.clone();
        let original = vm.settings.clone();
        vm.settings = original.for_gpu();
        inventory.managed.insert(
            vm_id.clone(),
            Some(hyper_gpu_support::workflow::Journal {
                schema: 1,
                vm_id: vm_id.clone(),
                gpu_interface: saved.gpu_interface.clone(),
                original,
                applied: vm.settings.clone(),
                prepared: Some("digest".into()),
                pending: false,
                restore_power: Power::Off,
                verification_only: false,
                last_verified: None,
                latest_check: None,
                preparation_started: None,
                check_restored: false,
                settings_released: false,
            }),
        );
        let text = s.host_configuration_text(&vm_id);
        assert!(text.contains("compared values match"));
        assert!(text.contains("Compute has no explicit saved request"));
        let vm = &mut s.view.inventory.as_mut().unwrap().discovery.vms[0];
        vm.vram.as_mut().unwrap().optimal = 60;
        vm.settings.low_mmio = 1;
        vm.settings.high_mmio = 2;
        vm.settings.cache_types = false;
        vm.settings.automatic_checkpoints = true;
        let text = s.host_configuration_text(&vm_id);
        assert!(text.contains("VRAM: expected 10 / 50 / 100; observed 10 / 60 / 100"));
        for name in [
            "Low MMIO",
            "High MMIO",
            "Guest cache types",
            "Automatic checkpoints",
        ] {
            assert!(text.contains(name), "{text}");
        }
        // A confirmed guest rollback mismatch invalidates payload eligibility,
        // not the independently recorded host settings baseline.
        let journal = s
            .view
            .inventory
            .as_mut()
            .unwrap()
            .managed
            .get_mut(&vm_id)
            .unwrap()
            .as_mut()
            .unwrap();
        journal.prepared = None;
        journal.latest_check = Some(hyper_gpu_support::workflow::GuestCheck::PayloadMismatch);
        assert!(s.host_configuration_text(&vm_id).contains("Low MMIO"));
        s.view
            .inventory
            .as_mut()
            .unwrap()
            .managed
            .get_mut(&vm_id)
            .unwrap()
            .as_mut()
            .unwrap()
            .pending = true;
        let text = s.host_configuration_text(&vm_id);
        assert!(text.contains("baseline is unavailable"));
        assert!(!text.contains("Low MMIO"));
    }

    #[test]
    fn host_comparison_handles_disabled_missing_malformed_and_stale_reads() {
        let (mut s, vm_id, _) = comparison_state(false);
        assert_eq!(
            s.host_configuration_text(&vm_id),
            "Host configuration: compared values match the saved intent."
        );
        s.view.needs_readback = true;
        assert!(s.host_configuration_text(&vm_id).contains("historical"));
        s.refresh(inventory());
        assert!(
            s.host_configuration_text(&vm_id)
                .contains("compared values match")
        );
        s.view.inventory.as_mut().unwrap().discovery.vms.clear();
        assert!(s.host_configuration_text(&vm_id).contains("Unknown"));
        s.view.inventory = None;
        assert!(
            s.host_configuration_text(&vm_id)
                .contains("inventory is unavailable")
        );
        s.load_committed(Default::default()).unwrap();
        assert!(
            s.host_configuration_text(&vm_id)
                .contains("no saved intent")
        );
        let mut store = hyper_gpu_support::configuration_store::StoreSnapshot::default();
        store.insert(&format!("{vm_id}.toml"), Ok("malformed".into()));
        s.load_committed(store).unwrap();
        assert!(
            s.host_configuration_text(&vm_id)
                .contains("saved intent is unreadable")
        );
    }

    #[test]
    fn host_comparison_isolates_incomplete_observations_and_requires_fresh_review() {
        let (mut s, vm_id, saved) = comparison_state(true);
        let diagnostic = hyper_gpu_support::reporting::Diagnostic::observation;
        let mut fresh = inventory();
        fresh.discovery.issues.push(diagnostic(
            "VM settings",
            Some("other-vm".into()),
            "access denied",
        ));
        s.refresh(fresh.clone());
        assert!(
            s.host_configuration_text(&vm_id)
                .contains("GPU attachment: expected")
        );
        fresh.discovery.issues.push(diagnostic(
            "VM settings",
            Some(vm_id.clone()),
            "access denied",
        ));
        s.refresh(fresh);
        assert!(
            s.host_configuration_text(&vm_id)
                .contains("observations are incomplete")
        );
        let text = s.host_configuration_text(&vm_id);
        assert!(!text.contains("configuration differences"));
        s.refresh(inventory());
        assert!(
            s.host_configuration_text(&vm_id)
                .contains("configuration differences")
        );
        s.view.needs_readback = true;
        assert!(s.view.eligibility(&saved).is_err());
        assert!(s.reviewed_plan(&saved).is_err());
        assert_eq!(s.committed.documents[&vm_id].target, saved);
    }
    fn id(s: &State) -> &str {
        s.view.selected.as_deref().unwrap()
    }
    fn values(a: &str, b: &str, c: &str) -> Vec<String> {
        let mut v = vec![String::new(); 12];
        v[0] = a.into();
        v[1] = b.into();
        v[2] = c.into();
        v
    }

    fn committed(
        targets: Vec<Target>,
        comment: &str,
    ) -> hyper_gpu_support::configuration_store::StoreSnapshot {
        let configuration = Configuration { schema: 2, targets };
        let mut store = hyper_gpu_support::configuration_store::StoreSnapshot::default();
        for (filename, source) in configuration.vm_documents().unwrap() {
            store.insert(&filename, Ok(format!("{source}\n# {comment}\n")));
        }
        store
    }

    fn save_record(s: &State) -> hyper_gpu_support::configuration_store::SaveRecord {
        use hyper_gpu_support::configuration_store::{Revision, SavePhase, SaveRecord};
        let observed = s.view.vm(id(s)).unwrap().clone();
        SaveRecord {
            schema: 1,
            retirement: None,
            operation_id: "test-operation".into(),
            target: s.target(id(s)).unwrap(),
            expected: Revision::Missing,
            driver_version: "test-driver".into(),
            payload_digest: None,
            plan_revision: Revision::Missing,
            intended_source: String::new(),
            publication_required: true,
            initial: observed.clone(),
            expected_power: None,
            graphics_required: None,
            initial_journal: None,
            expected_settings: observed.settings.clone(),
            phase: SavePhase::Verified(observed),
        }
    }

    #[test]
    fn unfinished_operation_explains_recovery_without_discarding_edits_or_releasing_hold() {
        use hyper_gpu_support::configuration_store::SavePhase;
        let mut s = state();
        assert!(s.action_recovery_message().is_none());
        let draft = s.target(id(&s)).unwrap();
        s.view.draft = Some(draft.clone());
        let mut record = save_record(&s);
        record.phase = SavePhase::EffectsStarted;
        s.recovery = Some(record);
        let body = s.action_recovery_message().unwrap();
        assert!(body.contains("Renamable VM"));
        assert!(body.contains("Review recovery"));
        assert!(body.contains("separate confirmation"));
        assert_eq!(s.view.draft, Some(draft));
        assert!(matches!(
            s.recovery.as_ref().unwrap().phase,
            SavePhase::EffectsStarted
        ));
        assert!(s.editor_eligibility(id(&s)).is_err());
        // A host-wide hold still needs an explanation when another VM is selected.
        s.view.selected = Some("another-vm".into());
        assert!(
            s.action_recovery_message()
                .unwrap()
                .contains("Renamable VM")
        );
        s.recovery = None;
        assert!(s.action_recovery_message().is_none());
    }

    #[test]
    fn external_save_clears_save_only_state_and_preserves_conflicting_draft() {
        let mut s = state();
        let target = s.target(id(&s)).unwrap();
        s.view.applied(&target).unwrap();
        let mut draft = target.clone();
        draft.enabled = !draft.enabled;
        s.view.draft = Some(draft.clone());
        assert!(
            s.load_committed(committed(vec![target], "external save"))
                .is_err()
        );
        s.refresh(inventory());
        s.sync_recovery(None, None, None).unwrap();
        assert!(!s.view.unsaved);
        assert!(s.view.verified_unsaved.is_none());
        assert_eq!(s.view.draft, Some(draft));
        assert!(s.editor_eligibility(id(&s)).is_err());
    }

    #[test]
    fn absent_recovery_requires_matching_readable_committed_intent() {
        for unreadable in [false, true] {
            let mut s = state();
            let target = s.target(id(&s)).unwrap();
            s.view.applied(&target).unwrap();
            if unreadable {
                s.committed
                    .errors
                    .insert("store".into(), "access denied".into());
            }
            s.refresh(inventory());
            assert!(s.sync_recovery(None, None, None).is_err());
            assert!(s.view.unsaved);
            assert_eq!(s.view.verified_unsaved, Some(target));
            assert!(s.view.needs_readback);
        }
    }

    #[test]
    fn standalone_verify_does_not_offer_publication_and_external_reconcile_releases_hold() {
        let mut s = state();
        let mut record = save_record(&s);
        record.publication_required = false;
        s.sync_recovery(Some(record), None, None).unwrap();
        assert!(!s.view.unsaved);
        assert!(s.view.verified_unsaved.is_none());
        assert!(s.editor_eligibility(id(&s)).is_err());
        s.refresh(inventory());
        s.sync_recovery(None, None, None).unwrap();
        assert!(s.editor_eligibility(id(&s)).is_ok());
        assert!(s.view.configuration.targets.is_empty());
    }

    #[test]
    fn verified_record_derives_save_only_state_and_import_holds_edits() {
        let mut s = state();
        let record = save_record(&s);
        let target = record.target.clone();
        s.sync_recovery(Some(record), None, None).unwrap();
        assert!(s.view.unsaved);
        assert_eq!(s.view.verified_unsaved, Some(target.clone()));
        s.load_committed(committed(vec![target], "external save"))
            .unwrap();
        let import = hyper_gpu_support::configuration_store::ImportRecord {
            schema: 1,
            operation_id: "test-import".into(),
            source_revision: hyper_gpu_support::configuration_store::Revision::Missing,
            documents: Default::default(),
        };
        s.refresh(inventory());
        s.sync_recovery(None, Some(import), None).unwrap();
        assert!(!s.view.unsaved);
        assert!(s.editor_eligibility(id(&s)).is_err());
        s.refresh(inventory());
        s.sync_recovery(None, None, None).unwrap();
        assert!(s.editor_eligibility(id(&s)).is_ok());
    }

    #[test]
    fn first_install_inventory_stays_current_until_explicit_refresh() {
        let mut s = state();
        assert!(s.can_poll());
        let mut first_install = inventory();
        first_install.enrolled.clear();
        s.refresh(first_install);
        assert!(!s.can_poll());
        assert!(!s.view.needs_readback);
        assert!(s.editor_eligibility(id(&s)).is_ok());
        s.refresh(inventory());
        assert!(s.can_poll());
    }

    #[test]
    fn completed_unsaved_apply_keeps_saved_observed_and_applied_state_distinct() {
        let mut s = state();
        let original = s.target(id(&s)).unwrap();
        let store = committed(vec![original.clone()], "saved before operation");
        s.load_committed(store.clone()).unwrap();
        let mut applied = original.clone();
        applied.enabled = false;
        s.view.draft = Some(applied.clone());
        let observed = s.view.vm(id(&s)).unwrap().clone();
        let mut record = save_record(&s);
        record.target = applied.clone();
        s.finish_apply(
            &applied,
            "test-operation",
            false,
            Ok(OperationReadback {
                committed: store,
                recovery: Some(record),
            }),
        )
        .unwrap();
        assert_eq!(s.target(&original.vm_id).unwrap(), original);
        assert_eq!(
            serde_json::to_value(s.view.vm(&applied.vm_id).unwrap()).unwrap(),
            serde_json::to_value(observed).unwrap()
        );
        assert_eq!(s.view.verified_unsaved, Some(applied.clone()));
        assert!(s.view.draft.is_none());
        assert!(s.view.needs_readback);
        assert!(
            s.applied_text(&applied.vm_id)
                .contains("publication is pending")
        );
        assert_eq!(s.save_only_record().unwrap().target, applied);
        assert!(s.editor_eligibility(id(&s)).is_err());
    }

    #[test]
    fn completed_apply_read_failure_retains_known_unsaved_intent_without_retry_authority() {
        let mut s = state();
        let target = s.target(id(&s)).unwrap();
        s.view.draft = Some(target.clone());
        assert!(
            s.finish_apply(
                &target,
                "test-operation",
                false,
                Err("protected read failed".into())
            )
            .is_err()
        );
        assert_eq!(s.view.verified_unsaved, Some(target));
        assert!(s.view.unsaved && s.view.needs_readback);
        assert!(s.approved_plan.is_none());
        assert!(s.save_only_record().is_err());
    }

    #[test]
    fn completed_apply_rejects_a_replaced_operation_receipt() {
        for changed_target in [false, true] {
            let mut s = state();
            let target = s.target(id(&s)).unwrap();
            let mut record = save_record(&s);
            if changed_target {
                record.target.enabled = !target.enabled;
            } else {
                record.operation_id = "another-operation".into();
            }
            assert!(
                s.finish_apply(
                    &target,
                    "test-operation",
                    false,
                    Ok(OperationReadback {
                        committed: Default::default(),
                        recovery: Some(record),
                    })
                )
                .unwrap_err()
                .contains("receipt changed")
            );
            assert!(s.view.needs_readback);
            assert!(s.editor_eligibility(id(&s)).is_err());
        }
    }

    #[test]
    fn save_only_completion_updates_selected_intent_and_preserves_another_vm_draft() {
        let mut s = state();
        let mut applied = s.target(id(&s)).unwrap();
        applied.enabled = false;
        let mut other = applied.clone();
        other.vm_id = "12345678-1234-1234-1234-000000000002".into();
        let mut other_draft = other.clone();
        other_draft.enabled = true;
        let store = committed(vec![applied.clone(), other.clone()], "published");
        s.load_committed(store.clone()).unwrap();
        s.view.draft = Some(other_draft.clone());
        s.finish_apply(
            &applied,
            "test-operation",
            true,
            Ok(OperationReadback {
                committed: store.clone(),
                recovery: None,
            }),
        )
        .unwrap();
        assert_eq!(s.view.draft, Some(other_draft));
        assert_eq!(s.target(&applied.vm_id).unwrap(), applied);
        assert_eq!(s.target(&other.vm_id).unwrap(), other);
        assert!(!s.view.unsaved);
        assert!(s.view.verified_unsaved.is_none());
        let mut reopened = State::default();
        reopened.load_committed(store).unwrap();
        reopened.refresh(inventory());
        assert_eq!(
            reopened.target(id(&reopened)).unwrap(),
            s.target(id(&s)).unwrap()
        );
        assert!(reopened.view.draft.is_none());
    }

    #[test]
    fn lost_terminal_reply_uses_durable_phase_without_inventing_completion() {
        use hyper_gpu_support::configuration_store::SavePhase;
        let mut s = state();
        let target = s.target(id(&s)).unwrap();
        s.view.draft = Some(target.clone());
        let mut record = save_record(&s);
        record.phase = SavePhase::EffectsStarted;
        s.finish_readback(Ok(OperationReadback {
            committed: Default::default(),
            recovery: Some(record),
        }))
        .unwrap();
        assert!(s.view.needs_readback);
        assert_eq!(s.view.draft, Some(target));
        assert!(s.applied_text(id(&s)).contains("uncertain"));
        assert!(s.save_only_record().is_err());
        assert!(s.editor_eligibility(id(&s)).is_err());
    }

    #[test]
    fn save_retry_is_available_only_for_publication_receipts() {
        use hyper_gpu_support::configuration_store::SavePhase;
        let mut s = state();
        let mut record = save_record(&s);
        let effective = record.initial.clone();
        for phase in [SavePhase::Admitted, SavePhase::EffectsStarted] {
            record.phase = phase;
            s.sync_recovery(Some(record.clone()), None, None).unwrap();
            assert!(s.save_only_record().is_err());
        }
        record.phase = SavePhase::Published(effective);
        s.sync_recovery(Some(record.clone()), None, None).unwrap();
        assert!(s.save_only_record().is_ok());
        assert!(!s.view.unsaved);
        assert!(s.applied_text(id(&s)).contains("published"));
        record.publication_required = false;
        s.sync_recovery(Some(record), None, None).unwrap();
        assert!(s.save_only_record().is_err());
    }

    #[test]
    fn committed_conflict_preserves_draft_but_refreshes_other_intent_and_inventory() {
        let mut s = state();
        let vm_id = id(&s).to_owned();
        let original = s.target(&vm_id).unwrap();
        s.load_committed(committed(vec![original.clone()], "original"))
            .unwrap();
        let draft = s.draft(&vm_id, false, 0, &values("", "", "")).unwrap();
        s.view.draft = Some(draft.clone());
        s.load_committed(committed(vec![original.clone()], "original"))
            .unwrap();

        let mut updated = original.clone();
        updated.vram = Some(Allocation {
            minimum: 10,
            optimal: 50,
            maximum: 100,
        });
        let mut other = original;
        other.vm_id = "12345678-1234-1234-1234-000000000002".into();
        let latest = committed(vec![updated.clone(), other.clone()], "external edit");
        assert!(s.load_committed(latest.clone()).is_err());
        let mut fresh = inventory();
        fresh.discovery.vms[0].name = "Fresh VM name".into();
        s.refresh(fresh);
        assert_eq!(s.view.vm(&vm_id).unwrap().name, "Fresh VM name");
        assert!(!s.view.needs_readback);
        assert_eq!(s.view.draft, Some(draft));
        assert_eq!(s.target(&vm_id).unwrap(), updated);
        assert_eq!(s.target(&other.vm_id).unwrap(), other);
        assert!(s.saved_desired_text(&vm_id).contains("10 / 50 / 100"));
        assert!(s.draft(&vm_id, true, 0, &values("", "", "")).is_err());
        // A second Refresh must not silently accept the conflicting revision.
        assert!(s.load_committed(latest).is_err());
        s.discard();
        assert!(s.view.draft.is_none());
        assert!(s.editor_eligibility(&vm_id).is_ok());
    }

    #[test]
    fn unreadable_committed_intent_is_unknown_and_draft_stays_blocked_after_repair() {
        let mut s = state();
        let vm_id = id(&s).to_owned();
        let target = s.target(&vm_id).unwrap();
        let original = committed(vec![target.clone()], "original");
        s.load_committed(original.clone()).unwrap();
        s.view.draft = Some(target.clone());
        let mut unreadable = hyper_gpu_support::configuration_store::StoreSnapshot::default();
        unreadable.insert(&format!("{vm_id}.toml"), Err("access denied".into()));
        assert!(s.load_committed(unreadable).is_err());
        assert!(s.saved_desired_text(&vm_id).contains("unknown"));
        assert!(!s.saved_desired_text(&vm_id).contains("No saved"));
        assert_eq!(s.view.draft, Some(target));
        assert!(s.load_committed(original).is_err());
        assert!(s.editor_eligibility(&vm_id).is_err());
        s.discard();
        assert!(s.editor_eligibility(&vm_id).is_ok());
    }

    #[test]
    fn removed_committed_file_does_not_become_a_new_draft_baseline_until_discard() {
        let mut s = state();
        let vm_id = id(&s).to_owned();
        let target = s.target(&vm_id).unwrap();
        s.load_committed(committed(vec![target.clone()], "original"))
            .unwrap();
        s.view.draft = Some(target.clone());
        assert!(s.load_committed(Default::default()).is_err());
        assert!(s.saved_desired_text(&vm_id).contains("No saved"));
        assert_eq!(s.view.draft, Some(target));
        assert!(s.draft(&vm_id, false, 0, &values("", "", "")).is_err());
        s.discard();
        assert!(s.draft(&vm_id, false, 0, &values("", "", "")).is_ok());
    }

    #[test]
    fn drafts_validate_raw_vram_and_never_mutate_observation_or_enrollment() {
        let s = state();
        let target = s
            .draft(id(&s), true, 0, &values("10", "50", "100"))
            .unwrap();
        assert_eq!(target.vram.unwrap().optimal, 50);
        assert!(s.view.vm(id(&s)).unwrap().gpus.is_empty());
        assert!(s.view.configuration.targets.is_empty());
        assert!(
            s.draft(id(&s), false, 0, &values("", "", ""))
                .unwrap()
                .vram
                .is_none()
        );
        assert_eq!(
            allocation_values(&s.target(id(&s)).unwrap()),
            values("", "", "")
        );
    }
    #[test]
    fn vram_draft_above_one_gb_metadata_does_not_change_saved_state() {
        let mut s = state();
        let gpu = &mut s.view.inventory.as_mut().unwrap().discovery.gpus[0];
        gpu.vram = Allocation {
            minimum: 0,
            optimal: 1_000_000_000,
            maximum: 1_000_000_000,
        }
        .into();
        gpu.physical_vram_bytes = Some(16_000_000_000);
        let saved = s.target(id(&s)).unwrap();
        let raw = s
            .memory_values(id(&s), 0, &vec![String::new(); 12], 160)
            .unwrap();
        let draft = s.draft(id(&s), true, 0, &raw).unwrap();
        assert_eq!(draft.vram.as_ref().unwrap().optimal, 16_000_000_000);
        assert_eq!(draft.vram.as_ref().unwrap().maximum, 16_000_000_000);
        assert_eq!(s.target(id(&s)).unwrap(), saved);
        assert!(s.view.vm(id(&s)).unwrap().gpus.is_empty());
        assert!(s.view.configuration.targets.is_empty());
    }

    #[test]
    fn incomplete_overflow_out_of_bounds_and_unsupported_fields_are_blocked() {
        let s = state();
        for v in [
            values("10", "", "100"),
            values("-1", "50", "100"),
            values("10", "50", "18446744073709551616"),
            values("10", "101", "100"),
        ] {
            assert!(s.draft(id(&s), true, 0, &v).is_err(), "{v:?}");
        }
        let mut v = values("10", "50", "100");
        v[4] = "50".into();
        assert!(
            s.draft(id(&s), true, 0, &v)
                .unwrap_err()
                .contains("Compute")
        );
        assert!(s.draft(id(&s), true, 0, &v[..3]).is_err());
    }

    #[test]
    fn supported_compute_draft_survives_unavailable_vram_and_preserves_observations() {
        use hyper_gpu_support::model::ResourceCapability;
        let mut s = state();
        let gpu = &mut s.view.inventory.as_mut().unwrap().discovery.gpus[0];
        gpu.vram = ResourceCapability::Unavailable("VRAM range unreadable".into());
        gpu.compute = ResourceCapability::Supported(Allocation {
            minimum: 10,
            optimal: 50,
            maximum: 100,
        });
        let mut raw = values("", "", "");
        raw[3..6].clone_from_slice(&["10".into(), "50".into(), "100".into()]);
        let target = s.draft(id(&s), true, 0, &raw).unwrap();
        assert_eq!(allocation_values(&target), raw);
        assert_eq!(target.compute.as_ref().unwrap().optimal, 50);
        assert!(target.vram.is_none());
        assert!(s.view.vm(id(&s)).unwrap().compute.is_none());
        assert!(
            s.view.inventory.as_ref().unwrap().enrolled[0]
                .compute
                .is_none()
        );
        let mut invalid = raw.clone();
        invalid[4] = "101".into();
        assert!(s.draft(id(&s), true, 0, &invalid).is_err());
        invalid[3] = "".into();
        assert!(s.draft(id(&s), true, 0, &invalid).is_err());
        raw[6..9].clone_from_slice(&["10".into(), "50".into(), "100".into()]);
        assert!(
            s.draft(id(&s), true, 0, &raw)
                .unwrap_err()
                .contains("Encode")
        );
        assert!(s.draft(id(&s), false, 0, &raw).is_ok());
        raw[6..9].fill(String::new());
        assert!(s.draft(id(&s), true, 0, &raw).is_ok());
    }
    #[test]
    fn duplicate_gpu_names_do_not_grant_a_different_pair() {
        let mut s = state();
        let i = s.view.inventory.as_mut().unwrap();
        let mut other = i.discovery.gpus[0].clone();
        other.interface = r"\\?\PCI#VEN_10DE&DEV_2D05#other\GPUPARAV".into();
        i.discovery.gpus.push(other);
        assert!(
            s.draft(id(&s), true, 1, &values("", "", ""))
                .unwrap_err()
                .contains("not enrolled")
        );
        assert!(s.draft(id(&s), true, -1, &values("", "", "")).is_err());
    }
    #[test]
    fn failed_reads_and_removed_enrollment_block_preview_without_losing_drafts() {
        let mut s = state();
        let original = s
            .draft(id(&s), true, 0, &values("10", "50", "100"))
            .unwrap();
        s.view.draft = Some(original.clone());
        s.view.needs_readback = true;
        assert!(s.draft(id(&s), true, 0, &values("10", "50", "100")).is_ok());
        assert!(s.view.eligibility(&original).is_err());
        let mut next = inventory();
        next.enrolled.clear();
        s.refresh(next);
        assert_eq!(s.view.draft, Some(original));
        assert!(
            s.draft(id(&s), true, 0, &values("10", "50", "100"))
                .is_err()
        );
    }

    #[test]
    fn pending_journal_blocks_new_reviews_without_a_save_record_and_retains_the_draft() {
        for verification_only in [false, true] {
            let mut s = state();
            let vm_id = id(&s).to_owned();
            let draft = s
                .draft(&vm_id, true, 0, &values("10", "50", "100"))
                .unwrap();
            s.view.draft = Some(draft.clone());
            let mut next = inventory();
            let settings = next.discovery.vms[0].settings.clone();
            next.managed.insert(
                vm_id.clone(),
                Some(hyper_gpu_support::workflow::Journal {
                    schema: 1,
                    vm_id: vm_id.clone(),
                    gpu_interface: draft.gpu_interface.clone(),
                    original: settings.clone(),
                    applied: settings,
                    prepared: None,
                    pending: true,
                    restore_power: Power::Off,
                    verification_only,
                    last_verified: None,
                    latest_check: None,
                    preparation_started: None,
                    check_restored: false,
                    settings_released: false,
                }),
            );
            s.refresh(next.clone());
            assert!(s.recovery.is_none());
            assert!(
                s.action_recovery_message()
                    .unwrap()
                    .contains("Renamable VM")
            );
            assert!(
                s.editor_eligibility(&vm_id)
                    .unwrap_err()
                    .contains("recovery")
            );
            assert!(
                s.draft(&vm_id, true, 0, &values("10", "50", "100"))
                    .is_err()
            );
            assert!(
                s.accept_review(
                    review_plan(),
                    &draft,
                    hyper_gpu_support::configuration_store::Revision::Missing,
                )
                .is_err()
            );
            assert!(s.approved_plan.is_none());
            assert_eq!(s.view.draft, Some(draft));
            assert!(
                s.view.inventory.as_ref().unwrap().managed[&vm_id]
                    .as_ref()
                    .unwrap()
                    .pending
            );
            next.managed
                .get_mut(&vm_id)
                .unwrap()
                .as_mut()
                .unwrap()
                .pending = false;
            s.refresh(next);
            assert!(s.editor_eligibility(&vm_id).is_ok());
        }
    }
    #[test]
    fn disappearing_vm_keeps_its_draft_identity_until_discard() {
        let mut s = state();
        s.view.draft = Some(s.target(id(&s)).unwrap());
        let selected = s.view.selected.clone();
        let mut next = inventory();
        next.discovery.vms[0].vm_id = "12345678-1234-1234-1234-000000000002".into();
        s.refresh(next);
        assert_eq!(s.view.selected, selected);
        assert!(s.view.vm(id(&s)).is_none());
        assert!(s.draft(id(&s), true, 0, &values("", "", "")).is_err());
        s.view.discard();
        s.refresh(inventory());
        assert!(s.view.vm(id(&s)).is_some());
    }

    #[test]
    fn changed_provider_bounds_do_not_reject_existing_vram_byte_requests() {
        let mut s = state();
        s.view.configuration.targets = s.view.inventory.as_ref().unwrap().enrolled.clone();
        s.view.configuration.targets[0].vram = Some(Allocation {
            minimum: 10,
            optimal: 150,
            maximum: 200,
        });
        assert!(s.editor_eligibility(id(&s)).is_ok());
        let old = allocation_values(&s.target(id(&s)).unwrap());
        assert!(s.draft(id(&s), true, 0, &old).is_ok());
        assert!(s.draft(id(&s), true, 0, &values("10", "50", "100")).is_ok());
        assert!(s.draft(id(&s), false, 0, &values("", "", "")).is_ok());
    }
    #[test]
    fn malformed_committed_configuration_cannot_echo_secrets_into_validation() {
        let mut s = state();
        let vm_id = id(&s).to_owned();
        s.committed.insert(
            &format!("{vm_id}.toml"),
            Ok("password='private-parser-secret'".into()),
        );
        let error = s.editor_eligibility(&vm_id).unwrap_err();
        assert!(error.contains("Committed configuration"));
        assert!(!error.contains("private-parser-secret"));
    }

    fn review_plan() -> Plan {
        use hyper_gpu_support::workflow::EffectPreview;
        let s = state();
        let target = s
            .draft(id(&s), true, 0, &values("10", "50", "100"))
            .unwrap();
        Plan {
            reinstall: false,
            initial_setup: false,
            desired: target.clone(),
            observed: s.view.vm(id(&s)).unwrap().clone(),
            managed: None,
            preparation: None,
            preview: EffectPreview {
                compute: None,
                encode: None,
                decode: None,
                actions: vec![],
                settings: None,
                settings_preserved: false,
                allocation: target.vram,
                credentials_required: true,
                guest_downtime: false,
                restore_power: Power::Off,
                pending_recovery: false,
                summary: vec!["Apply requested raw VRAM values".into()],
            },
            allocation_units: "provider-defined".into(),
            automatic_guest_restart: false,
        }
    }

    #[test]
    fn review_distinguishes_requested_triples_and_no_write_semantics() {
        let mut plan = review_plan();
        let first = review_text(&plan);
        let summary = review_summary(&plan);
        assert!(summary.contains("GPU support: Disabled → Enabled"));
        assert!(summary.contains("Unknown → 10 / 50 / 100"));
        assert!(!summary.contains("GPU interface"));
        assert!(first.contains("VRAM draft (Minimum / Optimal / Maximum): 10 / 50 / 100"));
        assert!(first.contains("Planned allocation write: 10 / 50 / 100"));
        plan.desired.vram = Some(Allocation {
            minimum: 20,
            optimal: 60,
            maximum: 90,
        });
        plan.preview.allocation = plan.desired.vram.clone();
        assert_ne!(first, review_text(&plan));
        plan.preview.allocation = None;
        assert!(review_text(&plan).contains("No allocation write planned"));
        plan.desired.vram = None;
        assert!(review_text(&plan).contains("Unspecified; no explicit VRAM request"));
    }

    #[test]
    fn memory_review_preserves_exact_observed_and_requested_bytes() {
        let mut plan = review_plan();
        plan.observed.vram = Some(Allocation {
            minimum: 0,
            optimal: 1_000_000_000,
            maximum: 1_000_000_000,
        });
        plan.preview.allocation = Some(Allocation {
            minimum: 1,
            optimal: 500_000_001,
            maximum: 900_000_003,
        });
        let text = memory_review(&plan);
        assert!(text.contains("0 / 1000000000 / 1000000000 → 1 / 500000001 / 900000003"));
        assert!(text.contains("bytes: minimum / optimal / maximum"));
        assert!(!text.contains("1 GB"));
        assert!(!text.contains("0.500000001 GB"));
        // Unknown readback cannot become a default or zero observation.
        plan.observed.vram = None;
        assert!(memory_review(&plan).contains("Unknown → 1 / 500000001 / 900000003"));
    }

    #[test]
    fn memory_review_distinguishes_saved_request_from_a_planned_write() {
        let mut plan = review_plan();
        plan.preview.allocation = None;
        let text = memory_review(&plan);
        assert!(text.contains("10 / 50 / 100"));
        assert!(text.contains("No VRAM write is planned"));
        assert!(!text.contains('→'));
        plan.desired.vram = None;
        let text = memory_review(&plan);
        assert!(text.contains("no explicit request"));
        assert!(!text.contains("bytes:"));
    }

    #[test]
    fn forget_pairing_review_is_bound_cancelable_and_keeps_other_profiles() {
        use hyper_gpu_support::{
            configuration_store::Revision,
            enrollment::{PairReview, RemovalReview},
        };
        let mut s = state();
        let vm_id = id(&s).to_owned();
        s.view
            .inventory
            .as_mut()
            .unwrap()
            .managed
            .insert(vm_id.clone(), None);
        let target = s.forget_target().unwrap();
        let review = RemovalReview {
            pair: PairReview {
                vm_id: vm_id.clone(),
                gpu_interface: target.gpu_interface.clone(),
                expected: Revision::of(b"policy"),
                operator_sid: "S-1-5-21-1".into(),
            },
            configurations: std::collections::BTreeMap::from([(vm_id.clone(), Revision::Missing)]),
            journals: std::collections::BTreeMap::from([(vm_id.clone(), Revision::Missing)]),
        };
        s.accept_forget_review(review.clone()).unwrap();
        assert_eq!(s.reviewed_forget().unwrap(), review);
        s.cancel_review();
        assert!(s.reviewed_forget().is_err());
        assert!(s.enrolled(&target));
        assert!(s.committed.documents.is_empty());
        s.accept_forget_review(review.clone()).unwrap();
        s.view.selected = Some("another-vm".into());
        assert!(s.reviewed_forget().is_err());
        s.view.selected = Some(vm_id.clone());
        let mut changed = target.clone();
        changed.enabled = false;
        s.committed.documents.insert(
            vm_id.clone(),
            hyper_gpu_support::configuration_store::SavedDocument {
                target: changed,
                source: "new disabled intent".into(),
                revision: Revision::of(b"new disabled intent"),
            },
        );
        assert!(s.reviewed_forget().is_err());
        assert!(s.accept_forget_review(review).is_err());
    }

    #[test]
    fn forget_pairing_refuses_active_unknown_recovery_or_absent_authority() {
        for scenario in 0..6 {
            let mut s = state();
            let vm_id = id(&s).to_owned();
            let gpu = s.target(&vm_id).unwrap().gpu_interface;
            s.view
                .inventory
                .as_mut()
                .unwrap()
                .managed
                .insert(vm_id.clone(), None);
            match scenario {
                0 => s.view.inventory.as_mut().unwrap().discovery.vms[0].gpus = vec![gpu],
                1 => s.view.inventory.as_mut().unwrap().discovery.vms[0].power = Power::Running,
                2 => s.recovery = Some(save_record(&s)),
                3 => {
                    s.view.inventory.as_mut().unwrap().managed.remove(&vm_id);
                }
                4 => s.view.inventory.as_mut().unwrap().enrolled.clear(),
                _ => s.view.needs_readback = true,
            }
            assert!(s.forget_target().is_err());
            assert!(s.removal_review.is_none());
        }
    }

    #[test]
    fn removal_is_reviewable_without_preparation_or_attachment_and_preserves_requests() {
        let mut s = state();
        let vm_id = id(&s).to_owned();
        let mut requested = s.target(&vm_id).unwrap();
        requested.compute = Some(Allocation {
            minimum: 10,
            optimal: 50,
            maximum: 90,
        });
        requested.vram = Some(Allocation {
            minimum: 10,
            optimal: 50,
            maximum: 100,
        });
        let values = allocation_values(&requested);
        s.view
            .inventory
            .as_mut()
            .unwrap()
            .managed
            .insert(vm_id.clone(), None);
        s.view.inventory.as_mut().unwrap().discovery.vms[0]
            .gpus
            .clear();
        let removed = s.removal_draft(&vm_id, 0, &values).unwrap();
        assert!(!removed.enabled);
        assert_eq!(removed.compute, requested.compute);
        assert_eq!(removed.vram, requested.vram);
        let mut plan = review_plan();
        plan.desired = removed.clone();
        plan.preview.credentials_required = false;
        let revision = s.committed.revision(&vm_id).unwrap();
        assert!(review_summary(&plan).contains("Remove GPU support"));
        assert!(review_summary(&plan).contains("Keep guest driver files, authorization"));
        s.accept_review(plan, &removed, revision).unwrap();
        assert!(!s.reviewed_plan(&removed).unwrap().plan.desired.enabled);
        s.cancel_review();
        assert!(s.reviewed_plan(&removed).is_err());
        assert!(s.enrolled(&removed));
    }

    #[test]
    fn removal_does_not_bypass_recovery_authority_or_selection() {
        for scenario in 0..3 {
            let mut s = state();
            let vm_id = id(&s).to_owned();
            match scenario {
                0 => s.view.inventory.as_mut().unwrap().enrolled.clear(),
                1 => s.recovery = Some(save_record(&s)),
                _ => s.view.selected = Some("another-vm".into()),
            }
            assert!(
                s.removal_draft(&vm_id, 0, &values("10", "50", "100"))
                    .is_err()
            );
            assert!(s.approved_plan.is_none());
        }
    }

    #[test]
    fn reinstall_review_is_explicit_cancelable_and_bound_to_selection_and_revision() {
        let mut s = state();
        let mut plan = review_plan();
        plan.reinstall = true;
        plan.preview.credentials_required = true;
        plan.preview.actions = vec![
            hyper_gpu_support::workflow::PreviewAction::Prepare,
            hyper_gpu_support::workflow::PreviewAction::Verify,
            hyper_gpu_support::workflow::PreviewAction::Shutdown,
        ];
        let target = plan.desired.clone();
        let revision = s.committed.revision(&target.vm_id).unwrap();
        let original = target.clone();
        assert!(review_summary(&plan).contains("Repair / Reinstall"));
        assert!(review_summary(&plan).contains("matching history"));
        s.accept_review(plan.clone(), &target, revision.clone())
            .unwrap();
        assert!(s.reviewed_plan(&target).unwrap().plan.reinstall);
        assert_eq!(s.reviewed_plan(&target).unwrap().plan.desired, original);
        s.view.selected = Some("another-vm".into());
        assert!(s.reviewed_plan(&target).is_err());
        s.view.selected = Some(target.vm_id.clone());
        s.cancel_review();
        assert!(s.reviewed_plan(&target).is_err());
        s.accept_review(plan, &target, revision).unwrap();
        s.committed.documents.remove(&target.vm_id);
        s.committed
            .errors
            .insert(format!("{}.toml", target.vm_id), "unreadable".into());
        assert!(s.reviewed_plan(&target).is_err());
    }

    #[test]
    fn review_binds_target_draft_and_exact_saved_revision() {
        use hyper_gpu_support::configuration_store::{Revision, SavedDocument};
        let mut s = state();
        let plan = review_plan();
        let target = plan.desired.clone();
        let revision = s.committed.revision(&target.vm_id).unwrap();
        s.accept_review(plan.clone(), &target, revision.clone())
            .unwrap();
        assert_eq!(s.reviewed_plan(&target).unwrap().revision, revision);

        let mut edited = target.clone();
        edited.enabled = false;
        assert!(s.reviewed_plan(&edited).is_err());
        edited = target.clone();
        edited.vram.as_mut().unwrap().optimal += 1;
        assert!(s.reviewed_plan(&edited).is_err());
        s.view.selected = Some("another-vm".into());
        assert!(s.reviewed_plan(&target).is_err());
        assert!(
            s.accept_review(plan.clone(), &target, revision.clone())
                .is_err()
        );
        s.view.selected = Some(target.vm_id.clone());
        s.accept_review(plan.clone(), &target, revision.clone())
            .unwrap();
        s.committed.documents.insert(
            target.vm_id.clone(),
            SavedDocument {
                target: target.clone(),
                source: "externally changed bytes".into(),
                revision: Revision::of(b"externally changed bytes"),
            },
        );
        assert!(s.reviewed_plan(&target).is_err());
        assert!(s.accept_review(plan, &target, revision).is_err());
        assert!(s.approved_plan.is_none());
    }

    #[test]
    fn late_plan_reply_cannot_approve_an_edited_draft_or_different_observed_vm() {
        let mut s = state();
        let mut plan = review_plan();
        let target = plan.desired.clone();
        let revision = s.committed.revision(&target.vm_id).unwrap();
        let mut edited = target.clone();
        edited.enabled = false;
        assert!(
            s.accept_review(plan.clone(), &edited, revision.clone())
                .is_err()
        );
        assert!(s.approved_plan.is_none());
        plan.observed.vm_id = "another-vm".into();
        assert!(s.accept_review(plan, &target, revision).is_err());
        assert!(s.approved_plan.is_none());
    }

    #[test]
    fn cancel_returns_to_draft_and_refresh_requires_new_review() {
        let mut s = state();
        let plan = review_plan();
        let target = plan.desired.clone();
        s.view.draft = Some(target.clone());
        let revision = s.committed.revision(&target.vm_id).unwrap();
        s.accept_review(plan.clone(), &target, revision.clone())
            .unwrap();
        s.cancel_review();
        assert_eq!(s.view.draft.as_ref(), Some(&target));
        assert!(s.reviewed_plan(&target).is_err());
        s.accept_review(plan, &target, revision).unwrap();
        s.sync_recovery(None, None, None).unwrap();
        assert!(s.reviewed_plan(&target).is_err());
        assert_eq!(s.view.draft.as_ref(), Some(&target));
    }

    #[test]
    fn cancel_and_discard_clear_verification_and_authority_approvals() {
        use hyper_gpu_support::{configuration_store::Revision, workflow::VerificationPlan};
        for discard in [false, true] {
            let mut s = state();
            let plan = review_plan();
            let target = plan.desired.clone();
            let saved = s.view.configuration.targets.clone();
            s.view.draft = Some(target.clone());
            let revision = s.committed.revision(&target.vm_id).unwrap();
            s.accept_review(plan.clone(), &target, revision).unwrap();
            s.verification_plan = Some(VerificationPlan {
                target: target.clone(),
                observed: plan.observed,
                gpu: s.view.inventory.as_ref().unwrap().discovery.gpus[0].clone(),
                managed: None,
                payload_digest: "current".into(),
            });
            s.enrollment_recovery_revision = Some(Revision::of(b"reviewed authority"));
            if discard {
                s.discard();
                assert!(s.view.draft.is_none());
            } else {
                s.cancel_review();
                assert_eq!(s.view.draft.as_ref(), Some(&target));
            }
            assert!(s.approved_plan.is_none());
            assert!(s.verification_plan.is_none());
            assert!(s.enrollment_recovery_revision.is_none());
            assert_eq!(s.view.configuration.targets, saved);
            assert!(s.view.vm(&target.vm_id).unwrap().gpus.is_empty());
        }
    }

    #[test]
    fn review_reports_ordered_power_steps_changed_settings_and_time_warnings() {
        use hyper_gpu_support::workflow::{PreviewAction, SettingsPreview};
        let mut plan = review_plan();
        plan.observed.power = Power::Running;
        plan.preview.guest_downtime = true;
        plan.preview.restore_power = Power::Running;
        plan.preview.actions = vec![
            PreviewAction::Shutdown,
            PreviewAction::Start,
            PreviewAction::Prepare,
            PreviewAction::Shutdown,
            PreviewAction::Settings,
            PreviewAction::Attach,
            PreviewAction::Start,
            PreviewAction::Verify,
        ];
        let before = plan.observed.settings.clone();
        let mut after = before.clone();
        after.high_mmio = 32768;
        plan.preview.settings = Some(SettingsPreview { before, after });
        let summary = review_summary(&plan);
        assert!(summary.contains("VM: Renamable VM"));
        assert!(summary.contains("1. Gracefully stop the VM."));
        assert!(summary.contains("2. Start the VM."));
        assert!(summary.contains("4. Gracefully stop the VM."));
        assert!(summary.contains("Expected final VM power after successful completion: Running."));
        assert!(summary.contains("High MMIO (MiB): 0 → 32768"));
        assert!(!summary.contains("Low MMIO (MiB)"));
        assert!(summary.contains("Save work inside it"));
        assert!(summary.contains("several minutes"));
        assert!(summary.contains("Shutdown refusal"));
        assert!(summary.contains("graphics check may take additional time"));
        plan.preview.actions.clear();
        plan.preview.guest_downtime = false;
        plan.preview.settings = None;
        let summary = review_summary(&plan);
        assert!(summary.contains("No VM or guest changes are needed"));
        assert!(!summary.contains("several minutes"));
        assert!(!summary.contains("Save work inside it"));
        assert!(!summary.contains("graphics check may take additional time"));
    }

    #[test]
    fn initial_setup_review_includes_graphics_and_successful_final_off() {
        use hyper_gpu_support::workflow::PreviewAction;
        let mut plan = review_plan();
        plan.initial_setup = true;
        plan.preview.restore_power = Power::Off;
        plan.preview.actions = vec![
            PreviewAction::Start,
            PreviewAction::Verify,
            PreviewAction::Shutdown,
        ];
        let summary = review_summary(&plan);
        assert!(summary.contains("Initial GPU setup"));
        assert!(summary.contains("hardware rendering"));
        assert!(summary.contains("Expected final VM power after successful completion: Stopped."));
        assert!(summary.contains("3. Gracefully stop the VM."));
        assert!(summary.contains("Start it through Hyper-V"));
    }

    #[test]
    fn ordinary_review_has_final_shutdown_without_guest_work_or_restart() {
        use hyper_gpu_support::workflow::PreviewAction;
        let mut plan = review_plan();
        plan.preview.credentials_required = false;
        plan.preview.restore_power = Power::Off;
        plan.preview.actions = vec![PreviewAction::Shutdown, PreviewAction::Allocation];
        let summary = review_summary(&plan);
        assert!(summary.contains("Ordinary GPU change"));
        assert!(summary.contains("no guest access or graphics check"));
        assert!(summary.contains("Expected final VM power after successful completion: Stopped."));
        assert!(!summary.contains("several minutes"));
        assert!(!summary.contains("graphics check may take additional time"));
        assert!(!summary.contains("Start the VM."));
    }

    #[test]
    fn driver_update_review_explicitly_names_required_setup_and_time() {
        use hyper_gpu_support::workflow::PreviewAction;
        let mut plan = review_plan();
        plan.preview.credentials_required = true;
        plan.preview.actions = vec![
            PreviewAction::Prepare,
            PreviewAction::Verify,
            PreviewAction::Shutdown,
        ];
        plan.preview.restore_power = Power::Off;
        let summary = review_summary(&plan);
        assert!(summary.contains("GPU setup/update required"));
        assert!(summary.contains("several minutes"));
        assert!(summary.contains("graphics check may take additional time"));
        assert!(!summary.contains("Ordinary GPU change"));
    }
}
