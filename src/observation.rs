//! Revision-bearing observations for presentation; never execution attestation.
use crate::{
    configuration_store::{ImportRecord, Revision, SaveRecord, StoreSnapshot},
    enrollment::PendingPair,
    gui_model::Inventory,
};
use serde::{Deserialize, Serialize};
#[cfg(test)]
mod tests;

/// Explicit metadata refresh, lightweight inventory poll, or selected completion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    /// Refresh topology and optional display metadata.
    Full,
    /// Read current topology/operational state without DXGI/KVP display work.
    Poll,
    /// Read the selected target after a terminal operation.
    Completion,
}

/// Fresh authority identity and observation provenance, not permission to execute.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Provenance {
    /// Snapshot contract version.
    pub schema: u32,
    /// Exact enrolled-policy revision read under the operation lock.
    pub authority: Revision,
    /// Revision of the complete observed store snapshot, including read errors.
    pub configuration: Revision,
    /// Observation breadth and metadata semantics.
    pub mode: Mode,
    /// Selected completion identity; unrelated inventory remains separately polled.
    pub selected: Option<crate::model::Target>,
}

/// One locked read of inventory, saved intent, recovery and enrolled authority.
#[derive(Clone, Serialize, Deserialize)]
pub struct Bundle {
    /// Current selected/all inventory according to provenance.
    pub inventory: Inventory,
    /// Independently read committed configuration with per-document revisions.
    pub committed: StoreSnapshot,
    /// Current VM-scoped holds; unreadable/global holds fail the whole read.
    pub recovery: Vec<SaveRecord>,
    /// Unfinished shared import, if present.
    pub import: Option<ImportRecord>,
    /// Unfinished shared pairing, if present.
    pub enrollment: Option<PendingPair>,
    /// Whether the freshly authenticated policy includes the protected worker.
    /// Worker admission still authenticates current files independently.
    pub worker_available: bool,
    /// Revisions and scope of this observation.
    pub provenance: Provenance,
}

/// A prepared plan and readiness from the same authenticated runner invocation.
#[derive(Serialize, Deserialize)]
pub struct Prepared<T> {
    /// Existing typed plan/review, unchanged execution contract.
    pub value: T,
    /// Fresh read policy identity, never reusable admission evidence.
    pub authority: Revision,
    /// Current protected-worker membership from this invocation's authentication.
    pub worker_available: bool,
}

impl Bundle {
    /// Verify internal scope/revision consistency before publication.
    pub fn validate(&self) -> Result<(), String> {
        if self.provenance.schema != 1
            || (self.provenance.mode == Mode::Completion) != self.provenance.selected.is_some()
        {
            return Err("Invalid observation scope".into());
        }
        let revision =
            Revision::of(&serde_json::to_vec(&self.committed).map_err(|e| e.to_string())?);
        if revision != self.provenance.configuration {
            return Err("Observation configuration revision differs".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for vm in &self.inventory.discovery.vms {
            if !ids.insert(&vm.vm_id) {
                return Err("Duplicate observation VM identity".into());
            }
            if self
                .provenance
                .selected
                .as_ref()
                .is_some_and(|target| target.vm_id != vm.vm_id)
            {
                return Err("Completion observation belongs to another VM".into());
            }
        }
        ids.clear();
        for gpu in &self.inventory.discovery.gpus {
            if !ids.insert(&gpu.interface) {
                return Err("Duplicate observation GPU identity".into());
            }
            if self
                .provenance
                .selected
                .as_ref()
                .is_some_and(|target| target.gpu_interface != gpu.interface)
            {
                return Err("Completion observation belongs to another GPU".into());
            }
        }
        Ok(())
    }

    /// Merge only presentation metadata by stable identity. Missing/changed
    /// operational rows never inherit operational facts from a previous poll.
    pub fn inventory_with_display(
        &self,
        previous: Option<&Inventory>,
    ) -> Result<Inventory, String> {
        self.validate()?;
        let mut inventory = self.inventory.clone();
        let Some(previous) = previous else {
            return Ok(inventory);
        };
        if self.provenance.mode == Mode::Full {
            return Ok(inventory);
        }
        for gpu in &mut inventory.discovery.gpus {
            if let Some(old) = previous.discovery.gpus.iter().find(|old| {
                old.interface == gpu.interface
                    && old.vendor == gpu.vendor
                    && old.device == gpu.device
                    && old.driver_version == gpu.driver_version
            }) {
                gpu.name = old.name.clone();
                gpu.physical_vram_bytes = old.physical_vram_bytes;
                gpu.total_vram = old.total_vram;
            }
        }
        for vm in &inventory.discovery.vms {
            if let Some(os) = previous.discovery.guest_os.get(&vm.vm_id) {
                inventory
                    .discovery
                    .guest_os
                    .insert(vm.vm_id.clone(), os.clone());
            }
        }
        if let Some(target) = &self.provenance.selected {
            inventory.discovery.vms =
                replace_selected(inventory.discovery.vms, &previous.discovery.vms, |vm| {
                    vm.vm_id == target.vm_id
                });
            inventory.discovery.gpus =
                replace_selected(inventory.discovery.gpus, &previous.discovery.gpus, |gpu| {
                    gpu.interface == target.gpu_interface
                });
            inventory.discovery.guest_os.extend(
                previous
                    .discovery
                    .guest_os
                    .iter()
                    .filter(|(id, _)| **id != target.vm_id)
                    .map(|(id, os)| (id.clone(), os.clone())),
            );
        }
        Ok(inventory)
    }
}

fn replace_selected<T: Clone>(
    fresh: Vec<T>,
    previous: &[T],
    selected: impl Fn(&T) -> bool,
) -> Vec<T> {
    let mut current = fresh.into_iter().next();
    let mut rows = Vec::new();
    for old in previous {
        if selected(old) {
            if let Some(value) = current.take() {
                rows.push(value);
            }
        } else {
            rows.push(old.clone());
        }
    }
    if let Some(value) = current {
        rows.push(value);
    }
    rows
}
