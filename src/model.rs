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
    /// Optional VRAM byte triple; absent requests no allocation write.
    #[serde(default)]
    pub vram: Option<Allocation>,
    /// Optional raw compute request; omission performs no compute write.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compute: Option<Allocation>,
    /// Optional raw encode request; omission performs no encode write.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encode: Option<Allocation>,
    /// Optional raw decode request; omission performs no decode write.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decode: Option<Allocation>,
}
/// Fixed provider resource categories, in the approved editor order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Resource {
    /// Video memory byte requests.
    Vram,
    /// Compute engines, in raw provider units.
    Compute,
    /// Encode engines, in raw provider units.
    Encode,
    /// Decode engines, in raw provider units.
    Decode,
}
impl Resource {
    /// All resource categories in editor order.
    pub const ALL: [Self; 4] = [Self::Vram, Self::Compute, Self::Encode, Self::Decode];
    /// Provider property suffix and operator label.
    pub fn name(self) -> &'static str {
        match self {
            Self::Vram => "VRAM",
            Self::Compute => "Compute",
            Self::Encode => "Encode",
            Self::Decode => "Decode",
        }
    }
}
/// Capability evidence; unavailable is distinct from an omitted request.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum ResourceCapability {
    /// Older captures or missing provider evidence cannot authorize writes.
    #[default]
    Unknown,
    /// A complete ordered native provider range was read.
    Supported(Allocation),
    /// The category could not be read/validated; other categories remain usable.
    Unavailable(String),
}
impl ResourceCapability {
    /// Report usable raw limits only for positively observed capability.
    pub fn limits(&self) -> Option<&Allocation> {
        match self {
            Self::Supported(limits) if limits.validate(limits).is_ok() => Some(limits),
            _ => None,
        }
    }
}
impl From<Allocation> for ResourceCapability {
    fn from(value: Allocation) -> Self {
        Self::Supported(value)
    }
}
fn deserialize_vram_capability<'de, D>(deserializer: D) -> Result<ResourceCapability, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Historical captures stored the positive VRAM range directly. Missing
    // capability remains Unknown; no range is inferred for other categories.
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Recorded {
        Capability(ResourceCapability),
        Legacy(Allocation),
    }
    Ok(match Recorded::deserialize(deserializer)? {
        Recorded::Capability(value) => value,
        Recorded::Legacy(value) => value.into(),
    })
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
    /// Validate a request using category-specific admission. VRAM values are
    /// byte requests: reported partition metadata is not a physical-card ceiling.
    /// Hyper-V decides whether a VRAM request is supported; callers must read it
    /// back independently. Other categories retain their reported range checks.
    pub fn validate_request(&self, resource: Resource, limits: &Self) -> Result<(), String> {
        limits.validate(limits)?;
        if resource == Resource::Vram {
            self.validate(self)
        } else {
            self.validate(limits)
        }
    }

    /// Validate ordering and the currently reported provider range.
    pub fn validate(&self, limits: &Self) -> Result<(), String> {
        if self.minimum > self.optimal || self.optimal > self.maximum {
            return Err("resource request requires minimum <= optimal <= maximum".into());
        }
        if self.minimum < limits.minimum || self.maximum > limits.maximum {
            return Err("resource request is outside the provider's reported range".into());
        }
        Ok(())
    }
}
impl Configuration {
    /// Explicit read-only upgrade; existing files are never rewritten by parsing.
    /// New category intent may be added to the returned schema-3 configuration.
    pub fn into_schema3(mut self) -> Result<Self, String> {
        self.validate()?;
        self.schema = 3;
        Ok(self)
    }
    /// Prepare canonical per-VM documents, preserving the declared schema version.
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

    /// Parse a GUID-keyed per-VM file with the shared production schema parser.
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
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("configuration requires a UTF-8 GUID filename")?;
        let file =
            std::fs::File::open(path).map_err(|error| format!("read VM configuration: {error}"))?;
        let mut bytes = Vec::new();
        file.take(MAX_BYTES + 1)
            .read_to_end(&mut bytes)
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
        if !matches!(self.schema, 2 | 3) || self.targets.is_empty() || self.targets.len() > 128 {
            return Err("configuration requires schema 2 or 3 and 1..128 targets".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for target in &self.targets {
            target.validate()?;
            if self.schema == 2 && target.has_extended_resources() {
                return Err("Compute/Encode/Decode intent requires explicit schema 3".into());
            }
            if !ids.insert(target.vm_id.to_ascii_lowercase()) {
                return Err("each VM may occur only once".into());
            }
        }
        Ok(())
    }
}
impl Target {
    /// Read a requested category; omission always means no explicit write.
    pub fn allocation(&self, resource: Resource) -> Option<&Allocation> {
        match resource {
            Resource::Vram => self.vram.as_ref(),
            Resource::Compute => self.compute.as_ref(),
            Resource::Encode => self.encode.as_ref(),
            Resource::Decode => self.decode.as_ref(),
        }
    }
    /// Stage a typed request without effects or configuration publication.
    pub fn set_allocation(&mut self, resource: Resource, value: Option<Allocation>) {
        match resource {
            Resource::Vram => self.vram = value,
            Resource::Compute => self.compute = value,
            Resource::Encode => self.encode = value,
            Resource::Decode => self.decode = value,
        }
    }
    /// Extended requests require schema 3 when explicitly published/imported.
    pub fn has_extended_resources(&self) -> bool {
        self.compute.is_some() || self.encode.is_some() || self.decode.is_some()
    }
    /// Clear requests only for identity/proposal checks, never committed intent.
    pub fn clear_allocations(&mut self) {
        for resource in Resource::ALL {
            self.set_allocation(resource, None);
        }
    }
    /// Independently compare every requested category; omitted categories are ignored.
    pub fn allocations_match(&self, state: &VmState) -> bool {
        Resource::ALL.into_iter().all(|resource| {
            self.allocation(resource)
                .is_none_or(|value| state.allocation(resource) == Some(value))
        })
    }
    /// Validate each requested category against its fresh provider evidence.
    pub fn validate_allocations(&self, gpu: &Gpu) -> Result<(), String> {
        for resource in Resource::ALL {
            if let Some(value) = self.allocation(resource) {
                let limits = gpu.limits(resource).ok_or_else(|| {
                    format!("{} capability is unavailable or unknown", resource.name())
                })?;
                value
                    .validate_request(resource, limits)
                    .map_err(|error| format!("{}: {error}", resource.name()))?;
            }
        }
        Ok(())
    }
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
        for resource in Resource::ALL {
            if self
                .allocation(resource)
                .is_some_and(|v| v.minimum > v.optimal || v.optimal > v.maximum)
            {
                return Err(format!("invalid {} allocation ordering", resource.name()));
            }
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
    /// Observed compute triple; absent means not observed/unset, never zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compute: Option<Allocation>,
    /// Observed encode triple.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encode: Option<Allocation>,
    /// Observed decode triple.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decode: Option<Allocation>,
}
impl VmState {
    /// Read an independently observed resource triple.
    pub fn allocation(&self, resource: Resource) -> Option<&Allocation> {
        match resource {
            Resource::Vram => self.vram.as_ref(),
            Resource::Compute => self.compute.as_ref(),
            Resource::Encode => self.encode.as_ref(),
            Resource::Decode => self.decode.as_ref(),
        }
    }
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
    /// Physical dedicated memory from an unambiguous native hardware identity.
    /// This is inventory, not the Hyper-V allocation range or an enforced quota.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical_vram_bytes: Option<u64>,
    /// Driver-reported total VRAM allocation value, distinct from card capacity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_vram: Option<u64>,
    /// VRAM capability; historical positive ranges remain readable.
    #[serde(default, deserialize_with = "deserialize_vram_capability")]
    pub vram: ResourceCapability,
    /// Compute capability; missing historical fields default to unknown.
    #[serde(default)]
    pub compute: ResourceCapability,
    /// Encode capability.
    #[serde(default)]
    pub encode: ResourceCapability,
    /// Decode capability.
    #[serde(default)]
    pub decode: ResourceCapability,
    /// Whether automatic guest preparation is implemented for this vendor.
    pub preparation_supported: bool,
}
impl Gpu {
    /// Complete validated limits, without inventing units or missing capability.
    pub fn limits(&self, resource: Resource) -> Option<&Allocation> {
        match resource {
            Resource::Vram => self.vram.limits(),
            Resource::Compute => self.compute.limits(),
            Resource::Encode => self.encode.limits(),
            Resource::Decode => self.decode.limits(),
        }
    }
    /// Present actual category bounds or the reason capability is unavailable.
    pub fn resource_text(&self, resource: Resource) -> String {
        if let Some(limits) = self.limits(resource) {
            return format!(
                "{} raw provider range: {} .. {} (optimal {}); no physical GB or enforcement claim",
                resource.name(),
                limits.minimum,
                limits.maximum,
                limits.optimal
            );
        }
        let capability = match resource {
            Resource::Vram => &self.vram,
            Resource::Compute => &self.compute,
            Resource::Encode => &self.encode,
            Resource::Decode => &self.decode,
        };
        let reason = match capability {
            ResourceCapability::Unknown => {
                "Capability unknown; new nonempty requests unavailable; clear existing values to omit"
            }
            ResourceCapability::Unavailable(reason) => reason.as_str(),
            ResourceCapability::Supported(_) => "Invalid provider range; editing unavailable",
        };
        format!(
            "{}: {reason}. Clear an existing request to omit this category.",
            resource.name()
        )
    }
}
/// Discovery data used identically by CLI and GUI.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Discovery {
    /// Guest-reported OS names from host KVP inventory, for presentation only.
    /// Missing/unreadable metadata stays unknown and never affects authorisation.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub guest_os: std::collections::BTreeMap<String, String>,
    /// Isolated failures; omitted observations are unknown, never detached/off.
    #[serde(default)]
    pub issues: Vec<crate::reporting::Diagnostic>,
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
    fn historical_gpu_ranges_remain_readable_and_missing_categories_are_unknown() {
        let raw = serde_json::json!({
            "interface": "synthetic", "name": "GPU", "vendor": 4318, "device": 1,
            "driver_version": "historical", "preparation_supported": true,
            "vram": {"minimum": 10, "optimal": 50, "maximum": 100}
        });
        let historical: Gpu = serde_json::from_value(raw).unwrap();
        assert!(historical.limits(Resource::Vram).is_some());
        assert!(historical.limits(Resource::Compute).is_none());
        assert_eq!(historical.compute, ResourceCapability::Unknown);
        let mut current = historical;
        current.vram = ResourceCapability::Unavailable("VRAM unknown".into());
        current.compute = Allocation {
            minimum: 10,
            optimal: 50,
            maximum: 100,
        }
        .into();
        let round_trip: Gpu =
            serde_json::from_slice(&serde_json::to_vec(&current).unwrap()).unwrap();
        assert!(round_trip.limits(Resource::Vram).is_none());
        assert!(round_trip.limits(Resource::Compute).is_some());
    }
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
    fn vram_requests_do_not_inherit_reported_partition_ceilings() {
        let reported = Allocation {
            minimum: 0,
            optimal: 1_000_000_000,
            maximum: 1_000_000_000,
        };
        let request = Allocation {
            minimum: 2_000_000_000,
            optimal: 8_000_000_000,
            maximum: 16_000_000_000,
        };
        assert!(request.validate_request(Resource::Vram, &reported).is_ok());
        for resource in [Resource::Compute, Resource::Encode, Resource::Decode] {
            assert!(request.validate_request(resource, &reported).is_err());
        }
        let invalid = Allocation {
            optimal: 17_000_000_000,
            ..request
        };
        assert!(invalid.validate_request(Resource::Vram, &reported).is_err());
        assert!(
            invalid
                .validate_request(Resource::Vram, &reported)
                .unwrap_err()
                .contains("minimum <= optimal <= maximum")
        );
        let malformed = Allocation {
            optimal: 2_000_000_000,
            ..reported
        };
        assert!(
            invalid
                .validate_request(Resource::Vram, &malformed)
                .is_err()
        );
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
