//! Typed off-state Hyper-V profile decisions and fresh readback verification.
//! Native effects stay in the fixed runner; values remain opaque integer units.

use serde::{Deserialize, Serialize};

use crate::config::{ConfigError, ProjectConfiguration, ResourceRequest, VmProfile};

/// One exact provider resource triple, without percentage conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Triple {
    /// Minimum resource allocation.
    pub minimum: u64,
    /// Maximum resource allocation.
    pub maximum: u64,
    /// Optimal resource allocation.
    pub optimal: u64,
}

/// The four independent GPU-P resources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GpuResources {
    /// Video memory.
    pub vram: Triple,
    /// Hardware encoding.
    pub encode: Triple,
    /// Hardware decoding.
    pub decode: Triple,
    /// Computation.
    pub compute: Triple,
}

impl GpuResources {
    /// Require an explicit complete profile from validated configuration.
    ///
    /// # Errors
    /// Provider defaults cannot express this operation's exact desired state.
    pub fn desired(project: &ProjectConfiguration) -> Result<Self, ConfigError> {
        fn triple(value: ResourceRequest) -> Result<Triple, ConfigError> {
            match value {
                ResourceRequest::Measured {
                    minimum,
                    maximum,
                    optimal,
                } if minimum <= optimal && optimal <= maximum => Ok(Triple {
                    minimum,
                    maximum,
                    optimal,
                }),
                _ => Err(ConfigError::InvalidResource),
            }
        }
        Ok(Self {
            vram: triple(project.resources.vram)?,
            encode: triple(project.resources.encode)?,
            decode: triple(project.resources.decode)?,
            compute: triple(project.resources.compute)?,
        })
    }

    /// Validate all triples against the freshly discovered provider ranges.
    ///
    /// # Errors
    /// Rejects malformed provider limits or any unsupported requested value.
    pub fn supported_by(&self, limits: &Self) -> Result<(), &'static str> {
        for (value, limit) in [
            (self.vram, limits.vram),
            (self.encode, limits.encode),
            (self.decode, limits.decode),
            (self.compute, limits.compute),
        ] {
            if limit.minimum > limit.maximum
                || value.minimum > value.optimal
                || value.optimal > value.maximum
                || value.minimum < limit.minimum
                || value.maximum > limit.maximum
            {
                return Err("unsupported GPU resource range");
            }
        }
        Ok(())
    }
}

/// Fresh provider observation. Security settings are observed and never modified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsSnapshot {
    /// Enrolled VM GUID.
    pub vm_id: String,
    /// Off is required for both apply and no-op.
    pub state: String,
    /// Exact single adapter interface.
    pub gpu_interface: String,
    /// Reject duplicate or absent adapters.
    pub gpu_adapters: u32,
    /// Microsoft Windows Secure Boot must already be enabled.
    pub secure_boot: bool,
    /// Firmware template identity.
    pub secure_boot_template: String,
    /// Existing vTPM must be enabled.
    pub tpm_enabled: bool,
    /// Dynamic memory is prohibited by the profile.
    pub dynamic_memory: bool,
    /// Exact native checkpoint policy retained for recovery.
    pub checkpoint_type: String,
    /// Exact automatic-checkpoint flag retained for recovery.
    pub automatic_checkpoints: bool,
    /// Exact native automatic stop action retained for recovery.
    pub automatic_stop_action: String,
    /// Fresh VM settings.
    pub profile: VmProfile,
    /// Effective GPU settings.
    pub resources: GpuResources,
    /// Fresh provider capability limits.
    pub limits: GpuResources,
}

impl SettingsSnapshot {
    /// Check target, security and supported resource ranges before any effect.
    ///
    /// # Errors
    /// Wrong state/adapter, weakened security and unsupported resources fail closed.
    pub fn validate(&self, project: &ProjectConfiguration) -> Result<(), &'static str> {
        if !matches!(
            self.checkpoint_type.as_str(),
            "Disabled" | "Standard" | "Production" | "ProductionOnly"
        ) || !matches!(
            self.automatic_stop_action.as_str(),
            "Save" | "TurnOff" | "ShutDown"
        ) || self.profile.checkpoints_disabled
            != (self.checkpoint_type == "Disabled" && !self.automatic_checkpoints)
            || self.profile.automatic_stop_guest_shutdown
                != (self.automatic_stop_action == "ShutDown")
        {
            return Err("inconsistent native checkpoint/stop observation");
        }
        if self.vm_id != project.slot.vm_id
            || self.state != "Off"
            || self.gpu_adapters != 1
            || !self
                .gpu_interface
                .eq_ignore_ascii_case(&project.slot.gpu_interface)
        {
            return Err("settings target/state/adapter mismatch");
        }
        if !self.secure_boot || self.secure_boot_template != "MicrosoftWindows" || !self.tpm_enabled
        {
            return Err("existing Microsoft Windows Secure Boot and vTPM required");
        }
        GpuResources::desired(project)
            .map_err(|_| "explicit resource profile required")?
            .supported_by(&self.limits)
    }

    /// Compare all owned settings after a fresh validated observation.
    ///
    /// # Errors
    /// Invalid observations cannot be classified as a no-op.
    pub fn matches(&self, project: &ProjectConfiguration) -> Result<bool, &'static str> {
        self.validate(project)?;
        Ok(!self.dynamic_memory
            && self.profile == project.vm_profile
            && self.resources
                == GpuResources::desired(project)
                    .map_err(|_| "explicit resource profile required")?)
    }

    /// Verify post-effect observation, preserving security and provider identities.
    ///
    /// # Errors
    /// Partial updates, stale effective values or capability changes require reconciliation.
    pub fn verify(
        &self,
        before: &Self,
        project: &ProjectConfiguration,
    ) -> Result<(), &'static str> {
        if !self.matches(project)? || self.limits != before.limits {
            return Err("fresh settings readback mismatch; reconciliation required");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(project: &ProjectConfiguration) -> SettingsSnapshot {
        let resources = GpuResources::desired(project).unwrap();
        SettingsSnapshot {
            vm_id: project.slot.vm_id.clone(),
            state: "Off".into(),
            gpu_interface: project.slot.gpu_interface.clone(),
            gpu_adapters: 1,
            secure_boot: true,
            secure_boot_template: "MicrosoftWindows".into(),
            tpm_enabled: true,
            dynamic_memory: false,
            checkpoint_type: "Disabled".into(),
            automatic_checkpoints: false,
            automatic_stop_action: "ShutDown".into(),
            profile: project.vm_profile.clone(),
            limits: resources.clone(),
            resources,
        }
    }

    #[test]
    fn exact_profile_and_u64_encode_round_trip() {
        let project = ProjectConfiguration::embedded().unwrap();
        let before = snapshot(&project);
        assert_eq!(before.resources.encode.minimum, 1 << 63);
        let after: SettingsSnapshot =
            serde_json::from_str(&serde_json::to_string(&before).unwrap()).unwrap();
        assert!(after.matches(&project).unwrap());
        after.verify(&before, &project).unwrap();
    }

    #[test]
    fn partial_update_and_stale_readback_fail() {
        let project = ProjectConfiguration::embedded().unwrap();
        let mut before = snapshot(&project);
        before.profile.processors = 2;
        assert!(!before.matches(&project).unwrap());
        assert!(before.verify(&before, &project).is_err());
        let mut after = snapshot(&project);
        after.resources.compute.optimal -= 1;
        assert!(after.verify(&before, &project).is_err());
        after = snapshot(&project);
        after.dynamic_memory = true;
        assert!(after.verify(&before, &project).is_err());
    }

    #[test]
    fn unsafe_targets_security_and_ranges_fail_before_noop() {
        let project = ProjectConfiguration::embedded().unwrap();
        let good = snapshot(&project);
        for bad in [
            SettingsSnapshot {
                state: "Running".into(),
                ..good.clone()
            },
            SettingsSnapshot {
                vm_id: "wrong".into(),
                ..good.clone()
            },
            SettingsSnapshot {
                gpu_interface: "wrong".into(),
                ..good.clone()
            },
            SettingsSnapshot {
                gpu_adapters: 2,
                ..good.clone()
            },
            SettingsSnapshot {
                secure_boot: false,
                ..good.clone()
            },
            SettingsSnapshot {
                tpm_enabled: false,
                ..good.clone()
            },
            SettingsSnapshot {
                secure_boot_template: "MicrosoftUEFICertificateAuthority".into(),
                ..good.clone()
            },
        ] {
            assert!(bad.matches(&project).is_err());
        }
        let mut bad = good.clone();
        bad.limits.encode.maximum -= 1;
        assert!(bad.validate(&project).is_err());
        bad = good;
        bad.limits.vram.minimum = bad.limits.vram.maximum + 1;
        assert!(bad.validate(&project).is_err());
    }

    #[test]
    fn malformed_profile_and_default_resources_are_rejected() {
        let mut project = ProjectConfiguration::embedded().unwrap();
        project.vm_profile.memory_bytes += 1;
        assert!(project.vm_profile.validate().is_err());
        project.resources.compute = ResourceRequest::ProviderDefault;
        assert!(GpuResources::desired(&project).is_err());
        let text = include_str!("../config/project.toml");
        for (from, to) in [
            ("processors = 4", "processors = 0"),
            (
                "checkpoints_disabled = true",
                "checkpoints_disabled = false",
            ),
            ("memory_bytes = 8589934592", "memory_bytes = -1"),
        ] {
            assert!(ProjectConfiguration::parse(&text.replace(from, to)).is_err());
        }
    }
}
