//! Allocation editing and draft eligibility; proposals never grant execution authority.
use super::State;
use crate::gui::memory;
use hyper_gpu_support::model::{Allocation, Resource, Target};

impl State {
    /// Native byte requests; physical capacity never rescales provider intent.
    pub fn memory_slider(
        &self,
        gpu_index: i32,
        values: &[String],
    ) -> Result<memory::MemorySlider, String> {
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
        let current = memory::request(values.get(..3).ok_or("VRAM fields are unavailable.")?)?;
        let capacity = gpu
            .physical_vram_bytes
            .filter(|bytes| *bytes > 0)
            .ok_or("Physical VRAM capacity could not be matched to the selected GPU. Use raw Advanced Allocation within the provider range.")?;
        memory::MemorySlider::new(limits, current.as_ref(), capacity)
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

    /// Commit a numeric GB edit through the same allocation and eligibility rules.
    pub fn memory_text_values(
        &self,
        id: &str,
        gpu_index: i32,
        values: &[String],
        text: &str,
    ) -> Result<Vec<String>, String> {
        self.editor_eligibility(id)?;
        if values.len() != 12 {
            return Err("Expected four complete allocation triples".into());
        }
        let allocation = self.memory_slider(gpu_index, values)?.select_gb(text)?;
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

    fn proposal_eligibility(&self, target: &Target) -> Result<(), String> {
        if !self.enrolled(target)
            && self
                .view
                .inventory
                .as_ref()
                .is_some_and(|i| i.enrolled.iter().any(|t| t.vm_id == target.vm_id))
        {
            return Err(
                "This selected GPU is not enrolled. Changing an enrolled GPU requires the separate replacement transition, which is not connected yet.".into(),
            );
        }
        self.view.draft_eligibility(target)
    }

    pub fn editor_eligibility(&self, id: &str) -> Result<(), String> {
        if !self.view.needs_readback && self.observation_historical(id) {
            return Err(
                "Current VM status is unavailable. Refresh before editing or reviewing changes."
                    .into(),
            );
        }
        if self.authority_conflict.as_deref() == Some(id) {
            return Err("Protected enrollment changed. Draft preserved; discard and Refresh before a new enrollment review.".into());
        }
        if self.committed_conflict.as_deref() == Some(id) {
            return Err("Committed configuration changed or became unavailable. Draft preserved; discard it to reload the latest committed intent.".into());
        }
        if self.blocks_vm(id) {
            return Err(
                "This VM or shared authority requires reconciliation or a save-only retry.".into(),
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

    pub fn discard(&mut self) {
        self.view.discard();
        self.committed_conflict = None;
        self.authority_conflict = None;
        self.result_notice = None;
        self.draft_gpu = None;
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
            return Err(
                "GPU removal requires this exact authorized VM/GPU pair; no enrollment or guest work was performed.".into(),
            );
        }
        Ok(target)
    }
}

pub(in crate::gui) fn allocation_values(target: &Target) -> Vec<String> {
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
