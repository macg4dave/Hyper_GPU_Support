//! Operator intent and fresh observations; no laboratory or driver integrity pins.
use serde::{Deserialize, Serialize};

/// Runtime operator configuration. Enrollment is separately administrator protected.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    /// Configuration format version.
    pub schema: u32,
    /// Independently selected existing VMs, with one GPU each.
    pub targets: Vec<Target>,
}
/// Desired state for one existing VM.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    /// Stable Hyper-V VM GUID, independent of its display name.
    pub vm_id: String,
    /// Exact provider partitionable GPU interface.
    pub gpu_interface: String,
    /// Whether to enable GPU-PV.
    pub enabled: bool,
    /// Optional provider-defined VRAM triple; absent requests no allocation write.
    #[serde(default)]
    pub vram: Option<Allocation>,
}
/// Provider-defined values, not a claim of bytes, percentages or a hard limit.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Allocation {
    /// Requested minimum.
    pub minimum: u64,
    /// Requested maximum.
    pub maximum: u64,
    /// Requested optimal value.
    pub optimal: u64,
}
impl Allocation {
    /// Validate ordering and the currently reported provider range.
    pub fn validate(&self, limits: &Self) -> Result<(), String> {
        if self.minimum > self.optimal
            || self.optimal > self.maximum
            || self.minimum < limits.minimum
            || self.maximum > limits.maximum
        {
            return Err("VRAM request is outside the provider's reported range".into());
        }
        Ok(())
    }
}
impl Configuration {
    /// Prepare canonical per-VM documents for an explicit schema-2 import.
    /// Reuses the production serializer and validates programmatically supplied
    /// intent. This is a read-only transformation: no publication, enrollment or
    /// VM operation occurs. Destination conflict/trust checks belong to the writer.
    pub fn vm_documents(&self) -> Result<std::collections::BTreeMap<String, String>, String> {
        self.validate()?;
        self.targets
            .iter()
            .map(|target| {
                let mut target = target.clone();
                target.vm_id.make_ascii_lowercase();
                let filename = format!("{}.toml", target.vm_id);
                let single = Self {
                    schema: self.schema,
                    targets: vec![target],
                };
                let text = toml::to_string_pretty(&single)
                    .map_err(|error| format!("serialize VM configuration: {error}"))?;
                Ok((filename, text))
            })
            .collect()
    }

    /// Parse a GUID-keyed per-VM file with the existing production schema-2 parser.
    /// The filename is canonical lowercase `<vm-guid>.toml`; it must match the
    /// sole target. This validates intent syntax, not enrollment or provider state.
    pub fn parse_vm_file(text: &str, filename: &str) -> Result<Self, String> {
        let value = Self::parse(text)?;
        if value.targets.len() != 1 {
            return Err("a per-VM configuration file requires exactly one target".into());
        }
        let expected = format!("{}.toml", value.targets[0].vm_id);
        if filename != expected {
            return Err("configuration filename must match the canonical target VM GUID".into());
        }
        Ok(value)
    }

    /// Read a candidate per-VM file without writes or Windows/backend queries.
    /// Returns exact source text for external-edit conflict detection. This reader
    /// does not establish trusted ownership/ACLs; protected-store admission remains
    /// a separate worker responsibility. Input is bounded to 64 KiB UTF-8 TOML.
    pub fn read_vm_file(path: &std::path::Path) -> Result<(Self, String), String> {
        use std::io::Read;
        const MAX_BYTES: u64 = 64 * 1024;
        let filename = path.file_name().and_then(|name| name.to_str())
            .ok_or("configuration requires a UTF-8 GUID filename")?;
        let file = std::fs::File::open(path)
            .map_err(|error| format!("read VM configuration: {error}"))?;
        let mut bytes = Vec::new();
        file.take(MAX_BYTES + 1).read_to_end(&mut bytes)
            .map_err(|error| format!("read VM configuration: {error}"))?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err("per-VM configuration exceeds the 64 KiB input limit".into());
        }
        let text = String::from_utf8(bytes)
            .map_err(|_| "VM configuration must be UTF-8 TOML".to_owned())?;
        Ok((Self::parse_vm_file(&text, filename)?, text))
    }

    /// Parse and validate exactly once at the application boundary.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut value: Self = toml::from_str(text).map_err(|e| format!("configuration: {e}"))?;
        value.validate()?;
        for target in &mut value.targets {
            target.vm_id.make_ascii_lowercase();
        }
        Ok(value)
    }

    /// Validate runtime intent, including values constructed by native callers.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != 2 || self.targets.is_empty() || self.targets.len() > 128 {
            return Err("configuration requires schema 2 and 1..128 targets".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for target in &self.targets {
            target.validate()?;
            if !ids.insert(target.vm_id.to_ascii_lowercase()) {
                return Err("each VM may occur only once".into());
            }
        }
        Ok(())
    }
}
impl Target {
    /// Validate target syntax before queries or privileged effects.
    pub fn validate(&self) -> Result<(), String> {
        let id = self.vm_id.as_bytes();
        if id.len() != 36
            || id.iter().enumerate().any(|(i, b)| {
                if [8, 13, 18, 23].contains(&i) {
                    *b != b'-'
                } else {
                    !b.is_ascii_hexdigit()
                }
            })
            || !self.gpu_interface.starts_with(r"\\?\PCI#")
            || !self.gpu_interface.ends_with(r"\GPUPARAV")
            || self.gpu_interface.len() > 512
            || self
                .gpu_interface
                .chars()
                .any(|c| c.is_control() || ['\'', '"'].contains(&c))
        {
            return Err("invalid VM GUID or GPU partition interface".into());
        }
        if self
            .vram
            .as_ref()
            .is_some_and(|v| v.minimum > v.optimal || v.optimal > v.maximum)
        {
            return Err("invalid VRAM allocation ordering".into());
        }
        Ok(())
    }
}
/// Current VM power state. Transitional/saved states are rejected before mutation.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Power {
    /// Powered off.
    Off,
    /// Running.
    Running,
    /// Saved or transitional provider state; refused for mutations.
    Other(u64),
}
/// Product-owned compatibility settings; CPU and RAM quantities are never changed.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Settings {
    /// Low MMIO gap in provider MiB units.
    pub low_mmio: u64,
    /// High MMIO gap in provider MiB units.
    pub high_mmio: u64,
    /// Guest cache types.
    pub cache_types: bool,
    /// Existing automatic-checkpoint policy.
    pub automatic_checkpoints: bool,
}
impl Settings {
    /// Conservative NVIDIA compatibility changes derived from the proven recipe.
    pub fn for_gpu(&self) -> Self {
        Self {
            low_mmio: self.low_mmio.max(3072),
            high_mmio: self.high_mmio.max(32768),
            cache_types: true,
            automatic_checkpoints: false,
        }
    }
}
/// Fresh independent Hyper-V state.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VmState {
    /// Stable identity.
    pub vm_id: String,
    /// Display name for users, never an authorization key.
    pub name: String,
    /// Current power state.
    pub power: Power,
    /// Generation supported by this product.
    pub generation: u32,
    /// Currently attached GPU interfaces.
    pub gpus: Vec<String>,
    /// Current compatibility settings.
    pub settings: Settings,
    /// Effective partition VRAM settings if assigned.
    pub vram: Option<Allocation>,
}
/// Partitionable host GPU discovery.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Gpu {
    /// Provider interface.
    pub interface: String,
    /// Installed PnP display name.
    pub name: String,
    /// PCI vendor ID.
    pub vendor: u32,
    /// PCI device ID.
    pub device: u32,
    /// Current driver version.
    pub driver_version: String,
    /// Reported VRAM allocation range.
    pub vram: Allocation,
    /// Whether automatic guest preparation is implemented for this vendor.
    pub preparation_supported: bool,
}
/// Discovery data used identically by CLI and GUI.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Discovery {
    /// Existing VMs; discovery itself does not enroll them.
    pub vms: Vec<VmState>,
    /// Partitionable GPUs, including vendors awaiting preparation adapters.
    pub gpus: Vec<Gpu>,
}

#[cfg(test)]
mod tests {
    use super::*;
    const CONFIG: &str = r#"schema = 2
[[targets]]
vm_id = "ABCDEF01-2345-6789-ABCD-EF0123456789"
gpu_interface = '\\?\PCI#VEN_10DE&DEV_2D05#test\GPUPARAV'
enabled = true
"#;
    #[test]
    fn normalizes_vm_ids_and_rejects_case_insensitive_duplicates() {
        let configuration = Configuration::parse(CONFIG).unwrap();
        assert_eq!(
            configuration.targets[0].vm_id,
            "abcdef01-2345-6789-abcd-ef0123456789"
        );
        let duplicate = format!(
            "{CONFIG}\n{}",
            CONFIG.split_once('\n').unwrap().1.replace(
                "ABCDEF01-2345-6789-ABCD-EF0123456789",
                "abcdef01-2345-6789-abcd-ef0123456789"
            )
        );
        assert!(
            Configuration::parse(&duplicate)
                .unwrap_err()
                .contains("each VM")
        );
    }
    #[test]
    fn rejects_laboratory_and_discovered_inventory_inputs() {
        for field in [
            "parent_path",
            "disk_path",
            "vm_name",
            "driver_version",
            "package_path",
            "file_count",
            "sha256",
            "password",
        ] {
            assert!(
                Configuration::parse(&format!("{CONFIG}\n{field} = 'not operator intent'\n"))
                    .is_err(),
                "accepted {field}"
            );
        }
        assert!(
            Configuration::parse(&CONFIG.replacen(
                "schema = 2",
                "schema = 2\nparent_path = 'golden'",
                1
            ))
            .is_err()
        );
    }
    #[test]
    fn validates_programmatically_constructed_configuration() {
        let mut configuration = Configuration::parse(CONFIG).unwrap();
        configuration.schema = 1;
        assert!(configuration.validate().is_err());
        configuration.schema = 2;
        configuration.targets.clear();
        assert!(configuration.validate().is_err());
        let target = Configuration::parse(CONFIG).unwrap().targets.remove(0);
        configuration.targets = vec![target.clone(); 129];
        assert!(configuration.validate().is_err());
        configuration.targets = vec![target.clone(), target];
        assert!(configuration.validate().unwrap_err().contains("each VM"));
        configuration.targets.truncate(1);
        configuration.targets[0].vm_id = "invalid".into();
        assert!(configuration.validate().is_err());
    }
    #[test]
    fn accepts_shared_gpu_without_lab_or_driver_inputs() {
        let t = r#"schema = 2
[[targets]]
vm_id = "11111111-1111-1111-1111-111111111111"
gpu_interface = '\\?\PCI#VEN_10DE&DEV_2D05#test\GPUPARAV'
enabled = true
[[targets]]
vm_id = "22222222-2222-2222-2222-222222222222"
gpu_interface = '\\?\PCI#VEN_10DE&DEV_2D05#test\GPUPARAV'
enabled = false
"#;
        assert_eq!(Configuration::parse(t).unwrap().targets.len(), 2);
        assert!(
            Configuration::parse(&t.replace(
                "22222222-2222-2222-2222-222222222222",
                "11111111-1111-1111-1111-111111111111"
            ))
            .is_err()
        );
        assert!(Configuration::parse(&format!("{t}\nparent_path = 'golden'\n")).is_err());
    }
    #[test]
    fn rejects_allocation_outside_actual_provider_range() {
        let range = Allocation {
            minimum: 10,
            optimal: 20,
            maximum: 30,
        };
        assert!(range.validate(&range).is_ok());
        assert!(
            Allocation {
                minimum: 1,
                ..range.clone()
            }
            .validate(&range)
            .is_err()
        );
        assert!(
            Allocation {
                optimal: 40,
                ..range.clone()
            }
            .validate(&range)
            .is_err()
        );
    }
}
