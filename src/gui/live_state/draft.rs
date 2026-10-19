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
            .ok_or(hyper_gpu_support::ui_text::dashboard::editing::gpu_unavailable())?;
        let limits = gpu
            .limits(Resource::Vram)
            .ok_or(hyper_gpu_support::ui_text::dashboard::editing::vram_bounds_unavailable())?;
        let current =
            memory::request(values.get(..3).ok_or(
                hyper_gpu_support::ui_text::dashboard::editing::vram_fields_unavailable(),
            )?)?;
        let capacity = gpu
            .physical_vram_bytes
            .filter(|bytes| *bytes > 0)
            .ok_or(hyper_gpu_support::ui_text::dashboard::editing::capacity_unmatched())?;
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
            return Err(hyper_gpu_support::ui_text::dashboard::editing::triples_expected().into());
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
            return Err(hyper_gpu_support::ui_text::dashboard::editing::triples_expected().into());
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
            return Err(hyper_gpu_support::ui_text::dashboard::editing::gpu_not_enrolled().into());
        }
        self.view.draft_eligibility(target)
    }

    pub fn editor_eligibility(&self, id: &str) -> Result<(), String> {
        if !self.view.needs_readback && self.observation_historical(id) {
            return Err(
                hyper_gpu_support::ui_text::dashboard::editing::status_unavailable().into(),
            );
        }
        if self.authority_conflict.as_deref() == Some(id) {
            return Err(
                hyper_gpu_support::ui_text::dashboard::editing::enrollment_changed().into(),
            );
        }
        if self.committed_conflict.as_deref() == Some(id) {
            return Err(
                hyper_gpu_support::ui_text::dashboard::editing::configuration_changed().into(),
            );
        }
        if self.blocks_vm(id) {
            return Err(
                hyper_gpu_support::ui_text::dashboard::editing::reconciliation_required().into(),
            );
        }
        if self
            .view
            .inventory
            .as_ref()
            .is_some_and(|inventory| inventory.recorded_state(id).recovery_required)
        {
            return Err(hyper_gpu_support::ui_text::dashboard::editing::recovery_pending().into());
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
            .ok_or(hyper_gpu_support::ui_text::dashboard::eligibility::refresh_inventory())?;
        let gpu = usize::try_from(gpu_index)
            .ok()
            .and_then(|i| inventory.discovery.gpus.get(i))
            .ok_or(hyper_gpu_support::ui_text::dashboard::editing::select_gpu())?;
        target.gpu_interface = gpu.interface.clone();
        target.enabled = enabled;
        if values.len() != 12 {
            return Err(hyper_gpu_support::ui_text::dashboard::editing::triples_expected().into());
        }
        for (resource, values) in Resource::ALL.into_iter().zip(values.chunks_exact(3)) {
            let value = if values.iter().all(|v| v.trim().is_empty()) {
                None
            } else {
                let parsed = values
                    .iter()
                    .map(|v| {
                        v.trim().parse::<u64>().map_err(|_| {
                            hyper_gpu_support::ui_text::dashboard::editing::integers_required(
                                resource.name(),
                            )
                            .into_string()
                        })
                    })
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
            return Err(
                hyper_gpu_support::ui_text::dashboard::editing::removal_vm_changed().into(),
            );
        }
        let target = self.draft(id, false, gpu_index, values)?;
        if !self.enrolled(&target) {
            return Err(
                hyper_gpu_support::ui_text::dashboard::editing::removal_pair_required().into(),
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
