//! Operational comparisons exclude presentation data, never missing required facts.
use super::{Allocation, Gpu, Power, Resource, ResourceCapability, Settings, VmState};
use serde::Serialize;

/// Current VM identity, lifecycle, assignment, settings and observed resources.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct VmFacts<'a> {
    vm_id: &'a str,
    power: &'a Power,
    generation: u32,
    gpus: &'a [String],
    settings: &'a Settings,
    allocations: [Option<&'a Allocation>; 4],
}

#[derive(Debug, PartialEq, Eq, Serialize)]
enum Capability<'a> {
    Unknown,
    Unavailable,
    Supported(&'a Allocation),
}

/// Current GPU identity, driver and provider capabilities; no display capacity.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct GpuFacts<'a> {
    interface: &'a str,
    vendor: u32,
    device: u32,
    driver_version: &'a str,
    preparation_supported: bool,
    capabilities: [Capability<'a>; 4],
}

impl VmState {
    /// Compare current operational facts without treating a rename as a mutation.
    pub fn operational(&self) -> VmFacts<'_> {
        VmFacts {
            vm_id: &self.vm_id,
            power: &self.power,
            generation: self.generation,
            gpus: &self.gpus,
            settings: &self.settings,
            allocations: Resource::ALL.map(|r| self.allocation(r)),
        }
    }
}

impl Gpu {
    /// Compare required provider/driver facts independently of optional DXGI data.
    /// Unknown/unavailable capabilities remain distinct from positive capability.
    pub fn operational(&self) -> GpuFacts<'_> {
        GpuFacts {
            interface: &self.interface,
            vendor: self.vendor,
            device: self.device,
            driver_version: &self.driver_version,
            preparation_supported: self.preparation_supported,
            capabilities: [&self.vram, &self.compute, &self.encode, &self.decode].map(
                |c| match c {
                    ResourceCapability::Unknown => Capability::Unknown,
                    ResourceCapability::Unavailable(_) => Capability::Unavailable,
                    ResourceCapability::Supported(value) => Capability::Supported(value),
                },
            ),
        }
    }
}

/// Keep every typed review field while replacing its presentation-bearing facts.
pub(crate) fn review_facts(
    review: &impl Serialize,
    observed: &VmState,
    gpu: &Gpu,
) -> Result<serde_json::Value, String> {
    let mut value = serde_json::to_value(review).map_err(|e| e.to_string())?;
    value["observed"] = serde_json::to_value(observed.operational()).map_err(|e| e.to_string())?;
    value["gpu"] = serde_json::to_value(gpu.operational()).map_err(|e| e.to_string())?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use crate::workflow::Backend;
    use crate::workflow::test_support::{fake, target};

    #[test]
    fn display_changes_do_not_change_operational_equality_but_required_drift_does() {
        let mut backend = fake();
        let vm = backend.inspect(&target()).unwrap();
        let gpu = backend.gpu(&target()).unwrap();
        let mut renamed = vm.clone();
        renamed.name = "another display name".into();
        assert_eq!(vm.operational(), renamed.operational());
        renamed.settings.high_mmio += 1;
        assert_ne!(vm.operational(), renamed.operational());
        let mut optional = gpu.clone();
        optional.name.clear();
        optional.physical_vram_bytes = Some(99);
        optional.total_vram = Some(100);
        assert_eq!(gpu.operational(), optional.operational());
        optional.driver_version.push('x');
        assert_ne!(gpu.operational(), optional.operational());
        optional = gpu.clone();
        optional.vram = super::ResourceCapability::Unknown;
        assert_ne!(gpu.operational(), optional.operational());
        assert!(optional.limits(super::Resource::Vram).is_none());
    }
}
