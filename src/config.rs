//! Version-one configuration and CLI-facing data contracts.
//!
//! The format is a deliberately small `key=value` document. It has no include,
//! environment expansion or credential fields, and unknown fields fail closed.

use std::collections::BTreeMap;
use std::fmt;

/// Current configuration schema.
pub const SCHEMA_VERSION: u32 = 1;

/// Complete desired configuration for the single disposable GPU-PV slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuration {
    /// Exact enrolled Hyper-V VM GUID in canonical lowercase form.
    pub vm_id: String,
    /// Exact GPU-P partition interface, never a friendly name or `Default GPU`.
    pub gpu_interface: String,
    /// Human-auditable immutable manifest identifier.
    pub manifest_id: String,
    /// SHA-256 of the immutable runtime manifest.
    pub manifest_sha256: String,
    /// Requested provider-defined GPU resource settings.
    pub resources: ResourceConfiguration,
}

/// Provider-defined resource requests. Values are opaque units, not percentages.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResourceConfiguration {
    /// Video-memory request.
    pub vram: ResourceRequest,
    /// Encode request.
    pub encode: ResourceRequest,
    /// Decode request.
    pub decode: ResourceRequest,
    /// Compute request.
    pub compute: ResourceRequest,
}

/// One resource request, either omitted for provider defaults or explicitly measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResourceRequest {
    /// Omit the resource fields and record the provider's effective result.
    #[default]
    ProviderDefault,
    /// Exact minimum, maximum and optimal values in provider-defined units.
    Measured {
        /// Minimum value.
        minimum: u64,
        /// Maximum value.
        maximum: u64,
        /// Optimal value within the inclusive range.
        optimal: u64,
    },
}

/// Stable top-level operations exposed by the CLI contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliOperation {
    /// Read host, GPU and VM state.
    Inventory,
    /// Produce a read-only change plan.
    Plan,
    /// Apply a reviewed current plan.
    Apply,
    /// Show effective project-owned state.
    Status,
    /// Validate configuration and prerequisites without mutation.
    Validate,
    /// Remove project-owned GPU assignment/settings.
    Remove,
    /// Discard and recreate the disposable child when state is uncertain.
    Recover,
    /// Start the configured VM.
    Start,
    /// Gracefully shut down the configured VM.
    Shutdown,
    /// Gracefully restart the configured VM.
    Restart,
}

impl CliOperation {
    /// Parse an exact CLI operation name.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "inventory" => Some(Self::Inventory),
            "plan" => Some(Self::Plan),
            "apply" => Some(Self::Apply),
            "status" => Some(Self::Status),
            "validate" => Some(Self::Validate),
            "remove" => Some(Self::Remove),
            "recover" => Some(Self::Recover),
            "start" => Some(Self::Start),
            "shutdown" => Some(Self::Shutdown),
            "restart" => Some(Self::Restart),
            _ => None,
        }
    }

    /// Return the stable CLI spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inventory => "inventory",
            Self::Plan => "plan",
            Self::Apply => "apply",
            Self::Status => "status",
            Self::Validate => "validate",
            Self::Remove => "remove",
            Self::Recover => "recover",
            Self::Start => "start",
            Self::Shutdown => "shutdown",
            Self::Restart => "restart",
        }
    }
}

/// Stable error categories and process exit behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Invalid CLI syntax.
    Usage,
    /// Invalid or stale configuration/plan.
    Configuration,
    /// Required Windows access was denied.
    Permission,
    /// Host, VM or dependency state cannot satisfy the operation.
    Environment,
    /// GPU driver/runtime failure.
    Driver,
    /// Project implementation/protocol failure.
    Implementation,
}

impl ErrorCategory {
    /// Stable process exit code. Zero is reserved for success.
    #[must_use]
    pub const fn exit_code(self) -> u8 {
        match self {
            Self::Usage => 2,
            Self::Configuration => 3,
            Self::Permission => 4,
            Self::Environment => 5,
            Self::Driver => 6,
            Self::Implementation => 70,
        }
    }
}

/// Immutable reference to a reviewed plan and observed environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Operation the plan authorizes.
    pub operation: CliOperation,
    /// SHA-256 of the canonical configuration.
    pub configuration_fingerprint: String,
    /// SHA-256 of the relevant observed host/VM state.
    pub environment_fingerprint: String,
}

impl Plan {
    /// Construct a plan reference from lowercase SHA-256 fingerprints.
    ///
    /// # Errors
    /// Returns [`ConfigError::InvalidFingerprint`] when either fingerprint is not a
    /// canonical lowercase SHA-256 value.
    pub fn new(
        operation: CliOperation,
        configuration_fingerprint: String,
        environment_fingerprint: String,
    ) -> Result<Self, ConfigError> {
        if !is_lower_hex(&configuration_fingerprint, 64)
            || !is_lower_hex(&environment_fingerprint, 64)
        {
            return Err(ConfigError::InvalidFingerprint);
        }
        Ok(Self {
            operation,
            configuration_fingerprint,
            environment_fingerprint,
        })
    }
}

/// Machine-readable outcome of one declared operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationReport {
    /// Report schema version.
    pub schema: u32,
    /// Operation that produced the report.
    pub operation: CliOperation,
    /// Explicit outcome; untested or blocked is never success.
    pub outcome: Outcome,
    /// Stable failure category, absent only for success.
    pub error_category: Option<ErrorCategory>,
}

/// Explicit operation result state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Operation completed and its required verification passed.
    Succeeded,
    /// Operation ran and failed.
    Failed,
    /// A prerequisite prevented execution.
    Blocked,
    /// Operation was not attempted.
    Untested,
}

impl OperationReport {
    /// Construct a report while enforcing success/error consistency.
    ///
    /// # Errors
    /// Returns [`ConfigError::InvalidSyntax`] when success has an error category
    /// or a non-success outcome has no category.
    pub fn new(
        operation: CliOperation,
        outcome: Outcome,
        error_category: Option<ErrorCategory>,
    ) -> Result<Self, ConfigError> {
        if (outcome == Outcome::Succeeded) != error_category.is_none() {
            return Err(ConfigError::InvalidSyntax);
        }
        Ok(Self {
            schema: SCHEMA_VERSION,
            operation,
            outcome,
            error_category,
        })
    }
}

/// Configuration parsing or validation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    /// A line is not a single non-empty `key=value` pair.
    InvalidSyntax,
    /// A field is not defined by schema version one.
    UnknownField,
    /// A field occurs more than once.
    DuplicateField,
    /// The schema is absent or unsupported.
    UnsupportedSchema,
    /// The VM GUID is absent or noncanonical.
    InvalidVmId,
    /// The GPU identity is absent, ambiguous or not a GPU-P interface.
    InvalidGpuIdentity,
    /// Manifest identity/hash is absent or invalid.
    InvalidManifest,
    /// A plan fingerprint is not a canonical SHA-256 value.
    InvalidFingerprint,
    /// A resource request is malformed or has an invalid range.
    InvalidResource,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidSyntax => "invalid configuration syntax",
            Self::UnknownField => "unknown configuration field",
            Self::DuplicateField => "duplicate configuration field",
            Self::UnsupportedSchema => "missing or unsupported configuration schema",
            Self::InvalidVmId => "invalid VM identity",
            Self::InvalidGpuIdentity => "invalid or ambiguous GPU identity",
            Self::InvalidManifest => "invalid manifest identity",
            Self::InvalidFingerprint => "invalid plan fingerprint",
            Self::InvalidResource => "invalid GPU resource range",
        })
    }
}

impl std::error::Error for ConfigError {}

impl Configuration {
    /// Parse and validate a strict version-one configuration.
    ///
    /// # Errors
    /// Returns a non-secret-bearing category for malformed, unknown, duplicate or
    /// unsafe input. Unknown fields include any attempted credential or path field.
    pub fn parse(input: &str) -> Result<Self, ConfigError> {
        if input.len() > 64 * 1024 {
            return Err(ConfigError::InvalidSyntax);
        }
        let mut fields = BTreeMap::new();
        for line in input.lines() {
            if line.is_empty() {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(ConfigError::InvalidSyntax);
            };
            if key.is_empty()
                || value.is_empty()
                || value.contains('=')
                || !key
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
            {
                return Err(ConfigError::InvalidSyntax);
            }
            if !matches!(
                key,
                "schema"
                    | "vm_id"
                    | "gpu_interface"
                    | "manifest_id"
                    | "manifest_sha256"
                    | "vram"
                    | "encode"
                    | "decode"
                    | "compute"
            ) {
                return Err(ConfigError::UnknownField);
            }
            if fields.insert(key, value).is_some() {
                return Err(ConfigError::DuplicateField);
            }
        }
        if fields.remove("schema") != Some("1") {
            return Err(ConfigError::UnsupportedSchema);
        }
        let vm_id = fields.remove("vm_id").ok_or(ConfigError::InvalidVmId)?;
        if !is_canonical_guid(vm_id) {
            return Err(ConfigError::InvalidVmId);
        }
        let gpu_interface = fields
            .remove("gpu_interface")
            .ok_or(ConfigError::InvalidGpuIdentity)?;
        if !is_gpu_interface(gpu_interface) {
            return Err(ConfigError::InvalidGpuIdentity);
        }
        let manifest_id = fields
            .remove("manifest_id")
            .ok_or(ConfigError::InvalidManifest)?;
        let manifest_sha256 = fields
            .remove("manifest_sha256")
            .ok_or(ConfigError::InvalidManifest)?;
        if !is_manifest_id(manifest_id) || !is_lower_hex(manifest_sha256, 64) {
            return Err(ConfigError::InvalidManifest);
        }
        let resources = ResourceConfiguration {
            vram: parse_resource(fields.remove("vram"))?,
            encode: parse_resource(fields.remove("encode"))?,
            decode: parse_resource(fields.remove("decode"))?,
            compute: parse_resource(fields.remove("compute"))?,
        };
        debug_assert!(fields.is_empty());
        Ok(Self {
            vm_id: vm_id.to_owned(),
            gpu_interface: gpu_interface.to_owned(),
            manifest_id: manifest_id.to_owned(),
            manifest_sha256: manifest_sha256.to_owned(),
            resources,
        })
    }

    /// Render the canonical version-one representation.
    #[must_use]
    pub fn render(&self) -> String {
        format!(
            concat!(
                "schema=1\nvm_id={}\ngpu_interface={}\nmanifest_id={}\n",
                "manifest_sha256={}\nvram={}\nencode={}\ndecode={}\ncompute={}\n"
            ),
            self.vm_id,
            self.gpu_interface,
            self.manifest_id,
            self.manifest_sha256,
            render_resource(self.resources.vram),
            render_resource(self.resources.encode),
            render_resource(self.resources.decode),
            render_resource(self.resources.compute),
        )
    }
}

fn parse_resource(value: Option<&str>) -> Result<ResourceRequest, ConfigError> {
    let Some(value) = value else {
        return Ok(ResourceRequest::ProviderDefault);
    };
    if value == "provider-default" {
        return Ok(ResourceRequest::ProviderDefault);
    }
    let mut parts = value.split(',');
    let minimum = parse_u64(parts.next())?;
    let maximum = parse_u64(parts.next())?;
    let optimal = parse_u64(parts.next())?;
    if parts.next().is_some() || maximum == 0 || minimum > optimal || optimal > maximum {
        return Err(ConfigError::InvalidResource);
    }
    Ok(ResourceRequest::Measured {
        minimum,
        maximum,
        optimal,
    })
}

fn parse_u64(value: Option<&str>) -> Result<u64, ConfigError> {
    let value = value.ok_or(ConfigError::InvalidResource)?;
    if value.is_empty() || (value.len() > 1 && value.starts_with('0')) {
        return Err(ConfigError::InvalidResource);
    }
    value.parse().map_err(|_| ConfigError::InvalidResource)
}

fn render_resource(value: ResourceRequest) -> String {
    match value {
        ResourceRequest::ProviderDefault => "provider-default".to_owned(),
        ResourceRequest::Measured {
            minimum,
            maximum,
            optimal,
        } => format!("{minimum},{maximum},{optimal}"),
    }
}

fn is_canonical_guid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
            }
        })
}

fn is_gpu_interface(value: &str) -> bool {
    value.len() <= 1024
        && value.is_ascii()
        && value.starts_with(r"\\?\PCI#")
        && value.ends_with(r"\GPUPARAV")
        && value.contains("VEN_")
        && value.contains("DEV_")
        && !value.bytes().any(|byte| byte.is_ascii_control())
}

fn is_manifest_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':'))
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::{
        CliOperation, ConfigError, Configuration, ErrorCategory, OperationReport, Outcome, Plan,
        ResourceRequest,
    };

    const MINIMAL: &str = concat!(
        "schema=1\n",
        "vm_id=2627e735-5b33-4104-b739-622727dd3a40\n",
        "gpu_interface=\\\\?\\PCI#VEN_10DE&DEV_2D05#example\\GPUPARAV\n",
        "manifest_id=nvidia-616.92-baseline-v1\n",
        "manifest_sha256=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n",
    );

    #[test]
    fn defaults_and_canonical_round_trip() {
        let config = Configuration::parse(MINIMAL).unwrap();
        assert_eq!(config.resources.vram, ResourceRequest::ProviderDefault);
        assert_eq!(config.resources.compute, ResourceRequest::ProviderDefault);
        assert_eq!(Configuration::parse(&config.render()), Ok(config));
    }

    #[test]
    fn measured_resources_round_trip_without_percentage_semantics() {
        let input = format!(
            "{MINIMAL}vram=0,1000000000,1000000000\nencode=0,18446744073709551615,18446744073709551615\ndecode=0,1000000000,1000000000\ncompute=0,1000000000,1000000000\n"
        );
        let config = Configuration::parse(&input).unwrap();
        assert_eq!(
            config.resources.vram,
            ResourceRequest::Measured {
                minimum: 0,
                maximum: 1_000_000_000,
                optimal: 1_000_000_000,
            }
        );
        assert_eq!(Configuration::parse(&config.render()), Ok(config));
    }

    #[test]
    fn rejects_unknown_duplicate_credentials_and_ambiguous_targets() {
        for (input, expected) in [
            (
                MINIMAL.replace("schema=1", "schema=2"),
                ConfigError::UnsupportedSchema,
            ),
            (
                format!("{MINIMAL}password=secret\n"),
                ConfigError::UnknownField,
            ),
            (
                format!("{MINIMAL}vm_id=2627e735-5b33-4104-b739-622727dd3a40\n"),
                ConfigError::DuplicateField,
            ),
            (
                MINIMAL.replace(
                    "2627e735-5b33-4104-b739-622727dd3a40",
                    "2627E735-5B33-4104-B739-622727DD3A40",
                ),
                ConfigError::InvalidVmId,
            ),
            (
                MINIMAL.replace(r"\\?\PCI#VEN_10DE&DEV_2D05#example\GPUPARAV", "Default GPU"),
                ConfigError::InvalidGpuIdentity,
            ),
            (
                format!("{MINIMAL}vram=10,9,10\n"),
                ConfigError::InvalidResource,
            ),
            (
                format!("{MINIMAL}vram=0,10,11\n"),
                ConfigError::InvalidResource,
            ),
            (
                format!("{MINIMAL}vram=00,10,10\n"),
                ConfigError::InvalidResource,
            ),
        ] {
            assert_eq!(Configuration::parse(&input), Err(expected));
        }
    }

    #[test]
    fn operations_and_error_codes_are_stable_and_distinct() {
        for name in [
            "inventory",
            "plan",
            "apply",
            "status",
            "validate",
            "remove",
            "recover",
            "start",
            "shutdown",
            "restart",
        ] {
            assert_eq!(
                CliOperation::parse(name).map(CliOperation::as_str),
                Some(name)
            );
        }
        assert_eq!(CliOperation::parse("shell"), None);
        assert_eq!(ErrorCategory::Usage.exit_code(), 2);
        assert_eq!(ErrorCategory::Configuration.exit_code(), 3);
        assert_eq!(ErrorCategory::Permission.exit_code(), 4);
        assert_eq!(ErrorCategory::Environment.exit_code(), 5);
        assert_eq!(ErrorCategory::Driver.exit_code(), 6);
        assert_eq!(ErrorCategory::Implementation.exit_code(), 70);
    }

    #[test]
    fn plan_and_report_keep_stale_and_unknown_results_explicit() {
        let fingerprint =
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        assert!(
            Plan::new(
                CliOperation::Apply,
                fingerprint.into(),
                fingerprint.into()
            )
            .is_ok()
        );
        assert_eq!(
            Plan::new(
                CliOperation::Apply,
                "not-a-hash".into(),
                fingerprint.into()
            ),
            Err(ConfigError::InvalidFingerprint)
        );
        assert!(OperationReport::new(CliOperation::Apply, Outcome::Succeeded, None).is_ok());
        assert!(
            OperationReport::new(
                CliOperation::Apply,
                Outcome::Blocked,
                Some(ErrorCategory::Environment)
            )
            .is_ok()
        );
        assert_eq!(
            OperationReport::new(CliOperation::Apply, Outcome::Untested, None),
            Err(ConfigError::InvalidSyntax)
        );
        assert_eq!(
            OperationReport::new(
                CliOperation::Apply,
                Outcome::Succeeded,
                Some(ErrorCategory::Driver)
            ),
            Err(ConfigError::InvalidSyntax)
        );
    }
}
