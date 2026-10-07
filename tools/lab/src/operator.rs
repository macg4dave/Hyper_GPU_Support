//! Read-only operator views for the configured existing VM and selected GPU.
//! These views consume bounded native inventory; they never inspect VM disks,
//! stage guest files, or imply readiness from host GPU enumeration.

use crate::{
    config::{ConfigError, ProjectConfiguration, VmProfile},
    inventory::{FactStatus, InventoryReport},
    vm_settings::GpuResources,
};
use serde::Serialize;

/// Configured operator intent, distinct from observed provider identities.
#[derive(Debug, Serialize)]
pub struct Target {
    /// Selected existing VM GUID.
    pub vm_id: String,
    /// Selected existing VM name.
    pub vm_name: String,
    /// Selected physical GPU model.
    pub gpu_name: String,
    /// Selected GPU-P interface.
    pub gpu_interface: String,
}
impl From<&ProjectConfiguration> for Target {
    fn from(p: &ProjectConfiguration) -> Self {
        Self {
            vm_id: p.slot.vm_id.clone(),
            vm_name: p.slot.vm_name.clone(),
            gpu_name: p.slot.gpu_name.clone(),
            gpu_interface: p.slot.gpu_interface.clone(),
        }
    }
}

/// A known value or explicit explanation of why it is unknown.
#[derive(Debug, Serialize)]
pub struct Observation {
    /// Inventory access/result classification.
    pub status: FactStatus,
    /// Present only when the provider returned a value.
    pub value: Option<String>,
    /// Missing, denied or unavailable explanation.
    pub detail: Option<String>,
}
impl Observation {
    fn read(report: &InventoryReport, key: &str) -> Self {
        match report.facts().iter().find(|fact| fact.key() == key) {
            Some(fact) if fact.status() == FactStatus::Known => Self {
                status: FactStatus::Known,
                value: Some(fact.value().into()),
                detail: None,
            },
            Some(fact) => Self {
                status: fact.status(),
                value: None,
                detail: Some(fact.value().into()),
            },
            None => Self::unobserved(),
        }
    }
    fn unobserved() -> Self {
        Self {
            status: FactStatus::Unavailable,
            value: None,
            detail: Some("not observed".into()),
        }
    }
}

/// Current observed state; unknown guest checks never become successful claims.
#[derive(Debug, Serialize)]
pub struct StatusReport {
    /// Operator report version.
    pub schema: u32,
    /// Configured target.
    pub target: Target,
    /// Exact VM selection observation.
    pub vm_selection: Observation,
    /// Native VM lifecycle state.
    pub vm_state: Observation,
    /// Native VM generation.
    pub vm_generation: Observation,
    /// Selected host GPU model.
    pub gpu_model: Observation,
    /// Selected host driver version.
    pub driver_version: Observation,
    /// Guest GPU assignment, independent of host partitionability.
    pub gpu_assignment: Observation,
    /// Guest driver provisioning, independent of host driver installation.
    pub driver_staging: Observation,
    /// Guest readiness, requiring a guest observation.
    pub guest_readiness: Observation,
}

/// Construct status without reading laboratory images, receipts or artifacts.
pub fn status(p: &ProjectConfiguration, inventory: &InventoryReport) -> StatusReport {
    StatusReport {
        schema: 1,
        target: p.into(),
        vm_selection: Observation::read(inventory, "vm.selection"),
        vm_state: Observation::read(inventory, "vm.state"),
        vm_generation: Observation::read(inventory, "vm.generation"),
        gpu_model: Observation::read(inventory, "gpu.model"),
        driver_version: Observation::read(inventory, "gpu.driverversion"),
        gpu_assignment: Observation::unobserved(),
        driver_staging: Observation::unobserved(),
        guest_readiness: Observation::unobserved(),
    }
}

/// One setup prerequisite; access failure is distinct from a known mismatch.
#[derive(Debug, Serialize)]
pub struct Prerequisite {
    /// Expected environment fact.
    pub name: &'static str,
    /// True/false only when the inventory returned a value.
    pub satisfied: Option<bool>,
    /// Supporting provider observation.
    pub observed: Observation,
}

/// Preview of intent and prerequisites, without discovering/copying driver files.
#[derive(Debug, Serialize)]
pub struct PlanReport {
    /// Operator report version.
    pub schema: u32,
    /// Configured existing VM and GPU.
    pub target: Target,
    /// Desired VM settings.
    pub vm_profile: VmProfile,
    /// Desired opaque GPU resource units.
    pub gpu_resources: GpuResources,
    /// Initial environment checks, not a mutation authorization.
    pub prerequisites: Vec<Prerequisite>,
    /// Ordinary setup actions; no reset/recreation operation is planned.
    pub setup_steps: Vec<&'static str>,
}

/// Preview setup intent from typed configuration and native inventory.
///
/// # Errors
/// Rejects configuration without complete explicit GPU resource values.
pub fn plan(
    p: &ProjectConfiguration,
    inventory: &InventoryReport,
) -> Result<PlanReport, ConfigError> {
    let checks = [
        ("Windows x64 host", "host.architecture", "X64"),
        (
            "selected existing VM",
            "vm.selection",
            p.slot.vm_id.as_str(),
        ),
        ("Generation 2 VM", "vm.generation", "2"),
        (
            "selected partitionable GPU",
            "gpup.interface",
            p.slot.gpu_interface.as_str(),
        ),
    ];
    let prerequisites = checks
        .into_iter()
        .map(|(name, key, expected)| {
            let observed = Observation::read(inventory, key);
            let satisfied = observed
                .value
                .as_ref()
                .map(|value| value.eq_ignore_ascii_case(expected));
            Prerequisite {
                name,
                satisfied,
                observed,
            }
        })
        .collect();
    Ok(PlanReport {
        schema: 1,
        target: p.into(),
        vm_profile: p.vm_profile.clone(),
        gpu_resources: GpuResources::desired(p)?,
        prerequisites,
        setup_steps: vec![
            "Verify current VM, GPU and guest access before changes",
            "Discover and provision the selected host driver's guest payload",
            "Apply the VM profile and selected GPU partition while the VM is off",
            "Read back effective settings and report guest GPU readiness",
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::Fact;
    #[test]
    fn denied_vm_is_unknown_and_host_gpu_does_not_prove_guest_readiness() {
        let p = ProjectConfiguration::embedded().unwrap();
        let inventory = InventoryReport::new(vec![
            Fact::new(
                "vm.selection",
                FactStatus::Denied,
                "management access denied",
            )
            .unwrap(),
            Fact::new("gpu.model", FactStatus::Known, &p.slot.gpu_name).unwrap(),
        ])
        .unwrap();
        let report = status(&p, &inventory);
        assert_eq!(report.vm_selection.status, FactStatus::Denied);
        assert!(report.vm_selection.value.is_none());
        assert_eq!(
            report.vm_selection.detail.as_deref(),
            Some("management access denied")
        );
        assert_eq!(
            report.gpu_model.value.as_deref(),
            Some(p.slot.gpu_name.as_str())
        );
        assert!(report.gpu_assignment.value.is_none());
        assert!(report.driver_staging.value.is_none());
        assert!(report.guest_readiness.value.is_none());
    }
    #[test]
    fn plan_preserves_unknown_and_wrong_target_prerequisites() {
        let p = ProjectConfiguration::embedded().unwrap();
        let inventory = InventoryReport::new(vec![
            Fact::new("host.architecture", FactStatus::Known, "X64").unwrap(),
            Fact::new("vm.selection", FactStatus::Known, "unrelated VM").unwrap(),
            Fact::new("vm.generation", FactStatus::Denied, "denied").unwrap(),
        ])
        .unwrap();
        let report = plan(&p, &inventory).unwrap();
        assert_eq!(
            report
                .prerequisites
                .iter()
                .map(|p| p.satisfied)
                .collect::<Vec<_>>(),
            [Some(true), Some(false), None, None]
        );
        assert_eq!(report.vm_profile, p.vm_profile);
        assert_eq!(report.gpu_resources.encode.optimal, 1 << 63);
        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("sha256"));
        assert!(!json.contains("parent_path"));
    }
}
