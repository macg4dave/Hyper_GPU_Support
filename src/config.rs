//! Version-one configuration and CLI-facing data contracts.
//!
//! The format is a deliberately small `key=value` document. It has no include,
//! environment expansion or credential fields, and unknown fields fail closed.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};

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

/// Validated project, development-environment and disposable-test configuration.
///
/// This is the typed boundary for `config/project.toml`. Consumers receive this
/// structure after parsing and validation and do not need to know TOML syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectConfiguration {
    /// Fixed guest readiness/workload inputs and finite execution budgets.
    pub validation: ValidationConfiguration,
    /// Desired off-state Hyper-V settings; security devices are retained.
    pub vm_profile: VmProfile,
    /// Exact disposable slot and its pinned storage/GPU identities.
    pub slot: SlotConfiguration,
    /// Desired driver/runtime manifest.
    pub driver_manifest: DriverManifestConfiguration,
    /// Requested provider-defined GPU resource settings.
    pub resources: ResourceConfiguration,
    /// Shared data and test-output locations.
    pub paths: ProjectPaths,
    /// Read-only inventory adapter settings.
    pub inventory: InventoryConfiguration,
    /// PowerShell Direct guest identity, staging root and deadlines.
    pub guest: GuestConfiguration,
    /// Least-privilege runner installation and timeout settings.
    pub runner: RunnerConfiguration,
    /// Pinned development tool inputs.
    pub tooling: ToolingConfiguration,
    /// Hardware-probe harness settings.
    pub tests: TestConfiguration,
}

/// Bounded read-only inventory adapter settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryConfiguration {
    /// Maximum duration of the native inventory query adapter.
    pub timeout: Duration,
}

/// Pinned guest identity and bounded staging settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuestConfiguration {
    /// Exact trusted Windows PowerShell executable used before receiving credentials.
    pub powershell_path: PathBuf,
    /// Computer name reported inside the disposable guest.
    pub computer_name: String,
    /// MachineGuid retained by the fixed disposable VM shell.
    pub machine_guid: String,
    /// Only root under which CORE-008 may create transferred files.
    pub staging_root: PathBuf,
    /// Maximum time to establish and validate a PowerShell Direct session.
    pub session_timeout: Duration,
    /// Maximum time for one verified file transfer.
    pub transfer_timeout: Duration,
    /// Maximum time for one complete manifest-level staging operation.
    pub staging_timeout: Duration,
}

/// Validated identity and disk chain for the one disposable slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotConfiguration {
    /// Logical slot name used by the runner protocol.
    pub name: String,
    /// Canonical lowercase Hyper-V VM GUID.
    pub vm_id: String,
    /// Exact Hyper-V VM display name used as a second identity check.
    pub vm_name: String,
    /// Exact GPU-P partition interface.
    pub gpu_interface: String,
    /// Exact adapter name expected from CUDA.
    pub gpu_name: String,
    /// PCI vendor identifier used for hardware adapter selection.
    pub gpu_vendor_id: u32,
    /// PCI device identifier used for hardware adapter selection.
    pub gpu_device_id: u32,
    /// PCI subsystem identifier required by exact probe validation.
    pub gpu_subsystem_id: u32,
    /// PCI revision required by exact probe validation.
    pub gpu_revision: u32,
    /// Expected CUDA compute-capability major version.
    pub cuda_compute_capability_major: i32,
    /// Expected CUDA compute-capability minor version.
    pub cuda_compute_capability_minor: i32,
    /// Immutable parent VHDX.
    pub parent_path: PathBuf,
    /// Expected immutable parent SHA-256.
    pub parent_sha256: String,
    /// Disposable differencing child VHDX.
    pub child_path: PathBuf,
}

/// Immutable driver/runtime manifest identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverManifestConfiguration {
    /// Deadline for the selected driver's complete native WMI association discovery.
    pub discovery_timeout: Duration,
    /// Human-auditable manifest identifier.
    pub id: String,
    /// Canonical lowercase manifest SHA-256.
    pub sha256: String,
    /// Exact GPU-correlated host DriverStore package root.
    pub source_path: PathBuf,
    /// Display INF filename within the package.
    pub inf_name: String,
    /// Exact `DriverVer` value expected in the INF.
    pub inf_version: String,
    /// Exact selected physical GPU driver version reported by Windows PnP.
    pub driver_version: String,
    /// Host Windows build used as the qualification baseline for this manifest.
    pub host_build: String,
    /// Signed catalog filename within the package.
    pub catalog_name: String,
    /// Expected number of regular files in the complete package tree.
    pub file_count: u32,
    /// Expected aggregate byte length of all package files.
    pub byte_count: u64,
    /// GPU-002 canonical package-tree digest.
    pub package_tree_sha256: String,
    /// Expected SHA-256 of the package catalog.
    pub catalog_sha256: String,
    /// Exact Authenticode signer certificate thumbprint for required signed inputs.
    pub signer_thumbprint: String,
    /// Package-relative files whose Authenticode signatures must validate.
    pub signature_files: Vec<String>,
    /// Exact reviewed delete-on-reboot source records allowed as warnings.
    pub allowed_pending_delete_sources: Vec<String>,
}

/// Shared project data locations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectPaths {
    /// Canonical absolute root for external machine-local data.
    pub data_root: PathBuf,
    /// Repository-relative directory for non-promoted test evidence.
    pub test_output: PathBuf,
}

/// Fixed runner installation identifiers and bounded operation deadlines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerConfiguration {
    /// Administrator-owned executable directory.
    pub install_directory: PathBuf,
    /// Administrator-owned policy, state, result and audit directory.
    pub data_directory: PathBuf,
    /// Dedicated local service account name.
    pub account_name: String,
    /// Scheduled Task folder, including leading and trailing separators.
    pub task_path: String,
    /// Current runner task name.
    pub task_name: String,
    /// Previous task name retained only for the reviewed migration/recovery path.
    pub legacy_task_name: String,
    /// Maximum reset operation duration.
    pub reset_timeout: Duration,
    /// Maximum inspection duration.
    pub inspect_timeout: Duration,
    /// Maximum VM-start duration.
    pub start_timeout: Duration,
    /// Maximum graceful-shutdown duration.
    pub shutdown_timeout: Duration,
    /// Maximum GPU attach/detach duration.
    pub gpu_assignment_timeout: Duration,
    /// Scheduled Task execution ceiling covering preflight plus one operation.
    pub task_execution_timeout: Duration,
    /// Reviewed build artifacts and their expected hashes.
    pub artifacts: RunnerArtifactConfiguration,
}

/// Hash-pinned binaries installed at the privilege boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerArtifactConfiguration {
    /// Directory containing release runner/client/rights binaries.
    pub directory: PathBuf,
}

/// Pinned local build-tool and upstream-source settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolingConfiguration {
    /// Repository-relative staging directory.
    pub staging_directory: PathBuf,
    /// CUDA toolkit release label used in derived directories.
    pub cuda_release: String,
    /// CMake release label used in derived directories.
    pub cmake_release: String,
    /// Windows SDK version containing the required shader compiler.
    pub windows_sdk_version: String,
    /// Absolute path to the Visual Studio developer-shell initializer.
    pub visual_studio_developer_shell: PathBuf,
    /// Repository-relative extracted DXC directory.
    pub dxc_directory: PathBuf,
    /// Exact CUDA Samples upstream URL.
    pub cuda_samples_repository: String,
    /// Pinned CUDA Samples commit.
    pub cuda_samples_commit: String,
    /// Pinned CUDA Samples tree.
    pub cuda_samples_tree: String,
    /// Pinned CMake archive URL.
    pub cmake_uri: String,
    /// Pinned CMake archive SHA-256.
    pub cmake_sha256: String,
    /// Named CUDA redistributable archives and hashes.
    pub cuda_packages: BTreeMap<String, DownloadConfiguration>,
}

/// One pinned download input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadConfiguration {
    /// HTTPS source URL.
    pub uri: String,
    /// Canonical lowercase archive SHA-256.
    pub sha256: String,
}

/// Settings for test harnesses that legitimately vary by environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestConfiguration {
    /// Standalone host probe settings.
    pub host_probes: HostProbeConfiguration,
}

/// Bounded host-probe harness settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostProbeConfiguration {
    /// Repository-local evidence output.
    pub output_path: PathBuf,
    /// Repository-local self-test evidence output.
    pub self_test_output_path: PathBuf,
    /// D3D/CUDA identity process timeout.
    pub process_timeout: Duration,
    /// CUDA workload process timeout.
    pub cuda_timeout: Duration,
    /// Whole-suite timeout.
    pub suite_timeout: Duration,
    /// Measured repetitions after the warm-up.
    pub repetitions: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProjectConfiguration {
    validation: ValidationConfiguration,
    vm_profile: VmProfile,
    schema: u32,
    slot: RawSlotConfiguration,
    driver_manifest: RawDriverManifestConfiguration,
    resources: RawResourceConfiguration,
    paths: RawProjectPaths,
    inventory: RawInventoryConfiguration,
    guest: RawGuestConfiguration,
    runner: RawRunnerConfiguration,
    tooling: RawToolingConfiguration,
    tests: RawTestConfiguration,
}

/// Guest-validation configuration, shared by host transport and Rust worker.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationConfiguration {
    /// NVIDIA runtime version expected from nvidia-smi for this driver recipe.
    pub nvidia_driver_version: String,
    /// App-local Microsoft x64 CRT source directory for the retained CUDA sample.
    pub crt_directory: PathBuf,
    /// Retained CUDA sample/FATBIN directory, relative to the repository.
    pub cuda_directory: PathBuf,
    /// Time allowed to obtain sustained readiness.
    pub readiness_timeout_seconds: u64,
    /// Consecutive Code 0 observation duration.
    pub stable_seconds: u64,
    /// PnP sampling interval.
    pub sample_milliseconds: u64,
    /// Bound on each workload and identity process.
    pub process_timeout_seconds: u64,
    /// Total worker execution bound, including readiness.
    pub worker_timeout_seconds: u64,
}

/// Project-owned Hyper-V settings, applied only while the enrolled VM is off.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VmProfile {
    /// Static guest memory in bytes.
    pub memory_bytes: u64,
    /// Virtual processor count.
    pub processors: u32,
    /// Low MMIO aperture in bytes.
    pub low_mmio_bytes: u64,
    /// High MMIO aperture in bytes.
    pub high_mmio_bytes: u64,
    /// Permit guest-controlled cache types.
    pub guest_controlled_cache_types: bool,
    /// Expose virtualization extensions to the guest.
    pub expose_virtualization_extensions: bool,
    /// Disable Hyper-V checkpoints.
    pub checkpoints_disabled: bool,
    /// Request guest shutdown when the host stops this VM.
    pub automatic_stop_guest_shutdown: bool,
}

impl VmProfile {
    /// Validate bounded project settings without weakening the checkpoint/stop contract.
    ///
    /// # Errors
    /// Rejects zero, unaligned or excessive allocations and unsupported policies.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !(1 << 30..=1 << 40).contains(&self.memory_bytes)
            || !self.memory_bytes.is_multiple_of(1 << 20)
            || !(1..=64).contains(&self.processors)
            || self.low_mmio_bytes > 4 << 30
            || self.high_mmio_bytes > 1 << 40
            || !self.low_mmio_bytes.is_multiple_of(1 << 20)
            || !self.high_mmio_bytes.is_multiple_of(1 << 20)
            || !self.checkpoints_disabled
            || !self.automatic_stop_guest_shutdown
        {
            return Err(ConfigError::InvalidProjectSetting);
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSlotConfiguration {
    name: String,
    vm_id: String,
    vm_name: String,
    gpu_interface: String,
    gpu_name: String,
    gpu_vendor_id: u32,
    gpu_device_id: u32,
    gpu_subsystem_id: u32,
    gpu_revision: u32,
    cuda_compute_capability_major: i32,
    cuda_compute_capability_minor: i32,
    parent_path: PathBuf,
    parent_sha256: String,
    child_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDriverManifestConfiguration {
    discovery_timeout_seconds: u64,
    id: String,
    sha256: String,
    source_path: PathBuf,
    inf_name: String,
    inf_version: String,
    driver_version: String,
    host_build: String,
    catalog_name: String,
    file_count: u32,
    byte_count: u64,
    package_tree_sha256: String,
    catalog_sha256: String,
    signer_thumbprint: String,
    signature_files: Vec<String>,
    allowed_pending_delete_sources: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawResourceConfiguration {
    vram: String,
    encode: String,
    decode: String,
    compute: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProjectPaths {
    data_root: PathBuf,
    test_output: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawInventoryConfiguration {
    timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGuestConfiguration {
    computer_name: String,
    machine_guid: String,
    staging_root: PathBuf,
    session_timeout_seconds: u64,
    transfer_timeout_seconds: u64,
    staging_timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRunnerConfiguration {
    install_directory: PathBuf,
    data_directory: PathBuf,
    account_name: String,
    task_path: String,
    task_name: String,
    legacy_task_name: String,
    reset_timeout_seconds: u64,
    inspect_timeout_seconds: u64,
    start_timeout_seconds: u64,
    shutdown_timeout_seconds: u64,
    gpu_assignment_timeout_seconds: u64,
    task_execution_timeout_seconds: u64,
    artifacts: RawRunnerArtifactConfiguration,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRunnerArtifactConfiguration {
    directory: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawToolingConfiguration {
    staging_directory: PathBuf,
    cuda_release: String,
    cmake_release: String,
    windows_sdk_version: String,
    visual_studio_developer_shell: PathBuf,
    dxc_directory: PathBuf,
    cuda_samples_repository: String,
    cuda_samples_commit: String,
    cuda_samples_tree: String,
    cmake_uri: String,
    cmake_sha256: String,
    cuda_packages: BTreeMap<String, RawDownloadConfiguration>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDownloadConfiguration {
    uri: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTestConfiguration {
    host_probes: RawHostProbeConfiguration,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHostProbeConfiguration {
    output_path: PathBuf,
    self_test_output_path: PathBuf,
    process_timeout_seconds: u64,
    cuda_timeout_seconds: u64,
    suite_timeout_seconds: u64,
    repetitions: u32,
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
    /// A project path is not absolute/relative as required or escapes its root.
    InvalidPath,
    /// A bounded operation or suite timeout is zero or internally inconsistent.
    InvalidTimeout,
    /// A project/tool identity, URL, revision or pinned hash is malformed.
    InvalidProjectSetting,
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
            Self::InvalidPath => "invalid project configuration path",
            Self::InvalidTimeout => "invalid project configuration timeout",
            Self::InvalidProjectSetting => "invalid project configuration setting",
        })
    }
}

impl std::error::Error for ConfigError {}

impl ProjectConfiguration {
    /// Parse and validate the authoritative project TOML document.
    ///
    /// # Errors
    /// Returns a non-secret-bearing category for malformed TOML, unknown fields,
    /// invalid identities, unsafe paths, unbounded timing or invalid pins.
    pub fn parse(input: &str) -> Result<Self, ConfigError> {
        if input.len() > 64 * 1024 {
            return Err(ConfigError::InvalidSyntax);
        }
        let raw: RawProjectConfiguration =
            toml::from_str(input).map_err(|_| ConfigError::InvalidSyntax)?;
        if raw.schema != SCHEMA_VERSION {
            return Err(ConfigError::UnsupportedSchema);
        }
        validate_identifier(&raw.slot.name)?;
        if !is_canonical_guid(&raw.slot.vm_id) {
            return Err(ConfigError::InvalidVmId);
        }
        validate_identifier(&raw.slot.vm_name)?;
        if !is_gpu_interface(&raw.slot.gpu_interface) {
            return Err(ConfigError::InvalidGpuIdentity);
        }
        if raw.slot.gpu_name.is_empty()
            || raw.slot.gpu_name.len() > 128
            || raw.slot.gpu_name.chars().any(char::is_control)
            || raw.slot.gpu_vendor_id == 0
            || raw.slot.gpu_device_id == 0
            || raw.slot.gpu_subsystem_id == 0
            || raw.slot.gpu_vendor_id > u16::MAX.into()
            || raw.slot.gpu_device_id > u16::MAX.into()
            || raw.slot.gpu_revision > u8::MAX.into()
            || raw.slot.cuda_compute_capability_major <= 0
            || raw.slot.cuda_compute_capability_minor < 0
        {
            return Err(ConfigError::InvalidGpuIdentity);
        }
        validate_absolute_windows_path(&raw.slot.parent_path)?;
        validate_absolute_windows_path(&raw.slot.child_path)?;
        if raw.slot.parent_path == raw.slot.child_path || !is_lower_hex(&raw.slot.parent_sha256, 64)
        {
            return Err(ConfigError::InvalidPath);
        }
        if !is_manifest_id(&raw.driver_manifest.id)
            || !is_lower_hex(&raw.driver_manifest.sha256, 64)
            || raw.driver_manifest.sha256.bytes().all(|byte| byte == b'0')
        {
            return Err(ConfigError::InvalidManifest);
        }
        validate_absolute_windows_path(&raw.driver_manifest.source_path)?;
        for filename in [
            &raw.driver_manifest.inf_name,
            &raw.driver_manifest.catalog_name,
        ] {
            validate_flat_filename(filename)?;
        }
        if raw.driver_manifest.signature_files.is_empty()
            || raw.driver_manifest.signature_files.len() > 16
        {
            return Err(ConfigError::InvalidManifest);
        }
        for filename in &raw.driver_manifest.signature_files {
            validate_flat_filename(filename)?;
        }
        if raw
            .driver_manifest
            .allowed_pending_delete_sources
            .is_empty()
            || raw.driver_manifest.allowed_pending_delete_sources.len() > 16
        {
            return Err(ConfigError::InvalidManifest);
        }
        let mut pending_delete_sources = BTreeSet::new();
        for source in &raw.driver_manifest.allowed_pending_delete_sources {
            let Some(path) = source.strip_prefix(r"*1\??\") else {
                return Err(ConfigError::InvalidManifest);
            };
            validate_absolute_windows_path(Path::new(path))?;
            if !pending_delete_sources.insert(source.to_ascii_lowercase()) {
                return Err(ConfigError::InvalidManifest);
            }
        }
        if raw.driver_manifest.discovery_timeout_seconds == 0
            || raw.driver_manifest.discovery_timeout_seconds > 300
            || raw.driver_manifest.inf_version.is_empty()
            || raw.driver_manifest.inf_version.len() > 128
            || raw
                .driver_manifest
                .inf_version
                .chars()
                .any(char::is_control)
            || raw.driver_manifest.driver_version.is_empty()
            || raw.driver_manifest.driver_version.len() > 64
            || !raw
                .driver_manifest
                .driver_version
                .bytes()
                .all(|byte| byte.is_ascii_digit() || byte == b'.')
            || raw.driver_manifest.file_count == 0
            || raw.driver_manifest.byte_count == 0
            || !is_lower_hex(&raw.driver_manifest.package_tree_sha256, 64)
            || !is_lower_hex(&raw.driver_manifest.catalog_sha256, 64)
            || !is_lower_hex(&raw.driver_manifest.signer_thumbprint, 40)
        {
            return Err(ConfigError::InvalidManifest);
        }
        validate_version_label(&raw.driver_manifest.host_build)?;
        validate_absolute_windows_path(&raw.paths.data_root)?;
        validate_relative_path(&raw.paths.test_output)?;
        if !path_is_within(&raw.slot.parent_path, &raw.paths.data_root)
            || !path_is_within(&raw.slot.child_path, &raw.paths.data_root)
        {
            return Err(ConfigError::InvalidPath);
        }
        if raw.inventory.timeout_seconds == 0 || raw.inventory.timeout_seconds > 300 {
            return Err(ConfigError::InvalidTimeout);
        }
        validate_identifier(&raw.guest.computer_name)?;
        if !is_canonical_guid(&raw.guest.machine_guid) {
            return Err(ConfigError::InvalidProjectSetting);
        }
        validate_absolute_windows_path(&raw.guest.staging_root)?;
        let powershell_path = crate::windows_paths::windows_powershell_executable()
            .map_err(|_| ConfigError::InvalidPath)?;
        if raw.guest.session_timeout_seconds == 0
            || raw.guest.transfer_timeout_seconds == 0
            || raw.guest.staging_timeout_seconds == 0
            || raw.guest.session_timeout_seconds > 300
            || raw.guest.transfer_timeout_seconds > 3600
            || raw.guest.staging_timeout_seconds > 7200
        {
            return Err(ConfigError::InvalidTimeout);
        }

        validate_absolute_windows_path(&raw.runner.install_directory)?;
        validate_absolute_windows_path(&raw.runner.data_directory)?;
        validate_absolute_windows_path(&raw.runner.artifacts.directory)?;
        for value in [
            &raw.runner.account_name,
            &raw.runner.task_name,
            &raw.runner.legacy_task_name,
        ] {
            validate_identifier(value)?;
        }
        if raw.runner.task_path.len() < 3
            || !raw.runner.task_path.starts_with('\\')
            || !raw.runner.task_path.ends_with('\\')
            || raw.runner.task_path.contains("..")
            || raw.runner.task_path.chars().any(char::is_control)
        {
            return Err(ConfigError::InvalidProjectSetting);
        }
        let runner_timeouts = [
            raw.runner.reset_timeout_seconds,
            raw.runner.inspect_timeout_seconds,
            raw.runner.start_timeout_seconds,
            raw.runner.shutdown_timeout_seconds,
            raw.runner.gpu_assignment_timeout_seconds,
            raw.runner.task_execution_timeout_seconds,
        ];
        if runner_timeouts.contains(&0)
            || raw.runner.inspect_timeout_seconds + raw.runner.start_timeout_seconds
                >= raw.runner.task_execution_timeout_seconds
            || raw.runner.inspect_timeout_seconds + raw.runner.shutdown_timeout_seconds
                >= raw.runner.task_execution_timeout_seconds
            || raw.runner.inspect_timeout_seconds + raw.runner.gpu_assignment_timeout_seconds
                >= raw.runner.task_execution_timeout_seconds
        {
            return Err(ConfigError::InvalidTimeout);
        }

        validate_relative_path(&raw.tooling.staging_directory)?;
        validate_relative_path(&raw.tooling.dxc_directory)?;
        validate_absolute_windows_path(&raw.tooling.visual_studio_developer_shell)?;
        for value in [
            &raw.tooling.cuda_release,
            &raw.tooling.cmake_release,
            &raw.tooling.windows_sdk_version,
        ] {
            validate_version_label(value)?;
        }
        validate_https(&raw.tooling.cuda_samples_repository)?;
        validate_https(&raw.tooling.cmake_uri)?;
        if !is_lower_hex(&raw.tooling.cuda_samples_commit, 40)
            || !is_lower_hex(&raw.tooling.cuda_samples_tree, 40)
            || !is_lower_hex(&raw.tooling.cmake_sha256, 64)
            || raw.tooling.cuda_packages.is_empty()
        {
            return Err(ConfigError::InvalidProjectSetting);
        }
        let cuda_packages = raw
            .tooling
            .cuda_packages
            .into_iter()
            .map(|(name, package)| {
                validate_identifier(&name)?;
                validate_https(&package.uri)?;
                if !is_lower_hex(&package.sha256, 64) {
                    return Err(ConfigError::InvalidProjectSetting);
                }
                Ok((
                    name,
                    DownloadConfiguration {
                        uri: package.uri,
                        sha256: package.sha256,
                    },
                ))
            })
            .collect::<Result<BTreeMap<_, _>, ConfigError>>()?;

        validate_relative_path(&raw.tests.host_probes.output_path)?;
        validate_relative_path(&raw.tests.host_probes.self_test_output_path)?;
        let test_timeouts = [
            raw.tests.host_probes.process_timeout_seconds,
            raw.tests.host_probes.cuda_timeout_seconds,
            raw.tests.host_probes.suite_timeout_seconds,
        ];
        if test_timeouts.contains(&0)
            || raw.tests.host_probes.repetitions == 0
            || raw.tests.host_probes.process_timeout_seconds
                > raw.tests.host_probes.suite_timeout_seconds
            || raw.tests.host_probes.cuda_timeout_seconds
                > raw.tests.host_probes.suite_timeout_seconds
        {
            return Err(ConfigError::InvalidTimeout);
        }

        let resources = ResourceConfiguration {
            vram: parse_resource(Some(&raw.resources.vram))?,
            encode: parse_resource(Some(&raw.resources.encode))?,
            decode: parse_resource(Some(&raw.resources.decode))?,
            compute: parse_resource(Some(&raw.resources.compute))?,
        };
        raw.vm_profile.validate()?;
        validate_absolute_windows_path(&raw.validation.crt_directory)?;
        validate_relative_path(&raw.validation.cuda_directory)?;
        if !(1..=600).contains(&raw.validation.readiness_timeout_seconds)
            || raw.validation.nvidia_driver_version.is_empty()
            || !raw
                .validation
                .nvidia_driver_version
                .bytes()
                .all(|b| b.is_ascii_digit() || b == b'.')
            || !(1..=raw.validation.readiness_timeout_seconds)
                .contains(&raw.validation.stable_seconds)
            || !(100..=5000).contains(&raw.validation.sample_milliseconds)
            || raw.validation.sample_milliseconds > raw.validation.stable_seconds * 1000
            || raw.validation.readiness_timeout_seconds * 1000 / raw.validation.sample_milliseconds
                > 180
            || !(1..=60).contains(&raw.validation.process_timeout_seconds)
            || raw.validation.worker_timeout_seconds
                < raw.validation.readiness_timeout_seconds
                    + 5 * raw.validation.process_timeout_seconds
            || raw.validation.worker_timeout_seconds > 900
        {
            return Err(ConfigError::InvalidProjectSetting);
        }
        Ok(Self {
            validation: raw.validation,
            vm_profile: raw.vm_profile,
            slot: SlotConfiguration {
                name: raw.slot.name,
                vm_id: raw.slot.vm_id,
                vm_name: raw.slot.vm_name,
                gpu_interface: raw.slot.gpu_interface,
                gpu_name: raw.slot.gpu_name,
                gpu_vendor_id: raw.slot.gpu_vendor_id,
                gpu_device_id: raw.slot.gpu_device_id,
                gpu_subsystem_id: raw.slot.gpu_subsystem_id,
                gpu_revision: raw.slot.gpu_revision,
                cuda_compute_capability_major: raw.slot.cuda_compute_capability_major,
                cuda_compute_capability_minor: raw.slot.cuda_compute_capability_minor,
                parent_path: raw.slot.parent_path,
                parent_sha256: raw.slot.parent_sha256,
                child_path: raw.slot.child_path,
            },
            driver_manifest: DriverManifestConfiguration {
                discovery_timeout: Duration::from_secs(
                    raw.driver_manifest.discovery_timeout_seconds,
                ),
                id: raw.driver_manifest.id,
                sha256: raw.driver_manifest.sha256,
                source_path: raw.driver_manifest.source_path,
                inf_name: raw.driver_manifest.inf_name,
                inf_version: raw.driver_manifest.inf_version,
                driver_version: raw.driver_manifest.driver_version,
                host_build: raw.driver_manifest.host_build,
                catalog_name: raw.driver_manifest.catalog_name,
                file_count: raw.driver_manifest.file_count,
                byte_count: raw.driver_manifest.byte_count,
                package_tree_sha256: raw.driver_manifest.package_tree_sha256,
                catalog_sha256: raw.driver_manifest.catalog_sha256,
                signer_thumbprint: raw.driver_manifest.signer_thumbprint,
                signature_files: raw.driver_manifest.signature_files,
                allowed_pending_delete_sources: raw.driver_manifest.allowed_pending_delete_sources,
            },
            resources,
            paths: ProjectPaths {
                data_root: raw.paths.data_root,
                test_output: raw.paths.test_output,
            },
            inventory: InventoryConfiguration {
                timeout: Duration::from_secs(raw.inventory.timeout_seconds),
            },
            guest: GuestConfiguration {
                powershell_path,
                computer_name: raw.guest.computer_name,
                machine_guid: raw.guest.machine_guid,
                staging_root: raw.guest.staging_root,
                session_timeout: Duration::from_secs(raw.guest.session_timeout_seconds),
                transfer_timeout: Duration::from_secs(raw.guest.transfer_timeout_seconds),
                staging_timeout: Duration::from_secs(raw.guest.staging_timeout_seconds),
            },
            runner: RunnerConfiguration {
                install_directory: raw.runner.install_directory,
                data_directory: raw.runner.data_directory,
                account_name: raw.runner.account_name,
                task_path: raw.runner.task_path,
                task_name: raw.runner.task_name,
                legacy_task_name: raw.runner.legacy_task_name,
                reset_timeout: Duration::from_secs(raw.runner.reset_timeout_seconds),
                inspect_timeout: Duration::from_secs(raw.runner.inspect_timeout_seconds),
                start_timeout: Duration::from_secs(raw.runner.start_timeout_seconds),
                shutdown_timeout: Duration::from_secs(raw.runner.shutdown_timeout_seconds),
                gpu_assignment_timeout: Duration::from_secs(
                    raw.runner.gpu_assignment_timeout_seconds,
                ),
                task_execution_timeout: Duration::from_secs(
                    raw.runner.task_execution_timeout_seconds,
                ),
                artifacts: RunnerArtifactConfiguration {
                    directory: raw.runner.artifacts.directory,
                },
            },
            tooling: ToolingConfiguration {
                staging_directory: raw.tooling.staging_directory,
                cuda_release: raw.tooling.cuda_release,
                cmake_release: raw.tooling.cmake_release,
                windows_sdk_version: raw.tooling.windows_sdk_version,
                visual_studio_developer_shell: raw.tooling.visual_studio_developer_shell,
                dxc_directory: raw.tooling.dxc_directory,
                cuda_samples_repository: raw.tooling.cuda_samples_repository,
                cuda_samples_commit: raw.tooling.cuda_samples_commit,
                cuda_samples_tree: raw.tooling.cuda_samples_tree,
                cmake_uri: raw.tooling.cmake_uri,
                cmake_sha256: raw.tooling.cmake_sha256,
                cuda_packages,
            },
            tests: TestConfiguration {
                host_probes: HostProbeConfiguration {
                    output_path: raw.tests.host_probes.output_path,
                    self_test_output_path: raw.tests.host_probes.self_test_output_path,
                    process_timeout: Duration::from_secs(
                        raw.tests.host_probes.process_timeout_seconds,
                    ),
                    cuda_timeout: Duration::from_secs(raw.tests.host_probes.cuda_timeout_seconds),
                    suite_timeout: Duration::from_secs(raw.tests.host_probes.suite_timeout_seconds),
                    repetitions: raw.tests.host_probes.repetitions,
                },
            },
        })
    }

    /// Parse the checked-in authoritative project configuration.
    ///
    /// # Errors
    /// Returns the same validation categories as [`Self::parse`].
    pub fn embedded() -> Result<Self, ConfigError> {
        static CONFIGURATION: OnceLock<Result<ProjectConfiguration, ConfigError>> = OnceLock::new();
        CONFIGURATION
            .get_or_init(|| Self::parse(include_str!("../config/project.toml")))
            .clone()
    }

    /// Return the desired-state subset consumed by product planning/apply code.
    #[must_use]
    pub fn desired_state(&self) -> Configuration {
        Configuration {
            vm_id: self.slot.vm_id.clone(),
            gpu_interface: self.slot.gpu_interface.clone(),
            manifest_id: self.driver_manifest.id.clone(),
            manifest_sha256: self.driver_manifest.sha256.clone(),
            resources: self.resources.clone(),
        }
    }
}

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

fn validate_identifier(value: &str) -> Result<(), ConfigError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(ConfigError::InvalidProjectSetting);
    }
    Ok(())
}

fn validate_version_label(value: &str) -> Result<(), ConfigError> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(ConfigError::InvalidProjectSetting);
    }
    Ok(())
}

fn validate_flat_filename(value: &str) -> Result<(), ConfigError> {
    let path = Path::new(value);
    let mut components = path.components();
    if value.is_empty()
        || value.len() > 255
        || value.ends_with([' ', '.'])
        || value.contains([':', '\0'])
        || value.chars().any(char::is_control)
        || !matches!(components.next(), Some(Component::Normal(_)))
        || components.next().is_some()
    {
        return Err(ConfigError::InvalidManifest);
    }
    Ok(())
}

fn validate_https(value: &str) -> Result<(), ConfigError> {
    if value.len() > 2048 || !value.starts_with("https://") || value.chars().any(char::is_control) {
        return Err(ConfigError::InvalidProjectSetting);
    }
    Ok(())
}

fn validate_absolute_windows_path(value: &Path) -> Result<(), ConfigError> {
    if !value.is_absolute()
        || value.as_os_str().is_empty()
        || value
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(ConfigError::InvalidPath);
    }
    Ok(())
}

fn validate_relative_path(value: &Path) -> Result<(), ConfigError> {
    if value.as_os_str().is_empty()
        || value.is_absolute()
        || value.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ConfigError::InvalidPath);
    }
    Ok(())
}

fn path_is_within(path: &Path, root: &Path) -> bool {
    path.starts_with(root) && path != root
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
    let Some(value) = value.strip_prefix(r"\\?\PCI#") else {
        return false;
    };
    let Some(value) = value.strip_suffix(r"\GPUPARAV") else {
        return false;
    };
    let mut segments = value.split('#');
    let (Some(hardware), Some(instance), Some(class)) =
        (segments.next(), segments.next(), segments.next())
    else {
        return false;
    };
    if segments.next().is_some()
        || class != "{064092b3-625e-43bf-9eb5-dc845897dd59}"
        || instance.is_empty()
        || instance.len() > 128
        || !instance
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'&' | b'_' | b'-'))
    {
        return false;
    }
    let mut identifiers = hardware.split('&');
    matches!(identifiers.next(), Some(value) if is_tagged_upper_hex(value, "VEN_", 4))
        && matches!(identifiers.next(), Some(value) if is_tagged_upper_hex(value, "DEV_", 4))
        && matches!(identifiers.next(), Some(value) if is_tagged_upper_hex(value, "SUBSYS_", 8))
        && matches!(identifiers.next(), Some(value) if is_tagged_upper_hex(value, "REV_", 2))
        && identifiers.next().is_none()
}

fn is_tagged_upper_hex(value: &str, tag: &str, digits: usize) -> bool {
    let Some(value) = value.strip_prefix(tag) else {
        return false;
    };
    value.len() == digits
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte))
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
        ProjectConfiguration, ResourceRequest,
    };

    const MINIMAL: &str = concat!(
        "schema=1\n",
        "vm_id=2627e735-5b33-4104-b739-622727dd3a40\n",
        "gpu_interface=\\\\?\\PCI#VEN_10DE&DEV_2D05&SUBSYS_8A151043&REV_A1#95B0EB63032DB04800#{064092b3-625e-43bf-9eb5-dc845897dd59}\\GPUPARAV\n",
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
    fn project_toml_deserializes_validates_and_exposes_desired_state() {
        let config = ProjectConfiguration::embedded().unwrap();
        assert!(!config.slot.name.is_empty());
        assert!(!config.runner.reset_timeout.is_zero());
        assert!(!config.guest.session_timeout.is_zero());
        assert!(config.guest.staging_root.is_absolute());
        assert!(config.tests.host_probes.repetitions > 0);
        assert!(!config.tooling.cuda_packages.is_empty());
        let desired = config.desired_state();
        assert_eq!(desired.vm_id, config.slot.vm_id);
        assert_eq!(desired.gpu_interface, config.slot.gpu_interface);
        assert_eq!(desired.manifest_id, config.driver_manifest.id);
    }

    #[test]
    fn project_toml_rejects_secrets_unsafe_paths_bad_pins_and_deadlines() {
        let valid = include_str!("../config/project.toml");
        let project = ProjectConfiguration::embedded().unwrap();
        for invalid in [
            valid.replace("schema = 1", "schema = 2"),
            valid.replace("discovery_timeout_seconds = 300", "discovery_timeout_seconds = 0"),
            valid.replace("discovery_timeout_seconds = 300", "discovery_timeout_seconds = 301"),
            valid.replace(
                &format!("vm_name = \"{}\"", project.slot.vm_name),
                &format!(
                    "vm_name = \"{}\"\npassword = \"secret\"",
                    project.slot.vm_name
                ),
            ),
            valid.replace(
                &format!("parent_path = '''{}'''", project.slot.parent_path.display()),
                "parent_path = '''..\\outside.vhdx'''",
            ),
            valid.replace(
                &format!("cmake_sha256 = \"{}\"", project.tooling.cmake_sha256),
                "cmake_sha256 = \"not-a-hash\"",
            ),
            valid.replace(
                &format!("machine_guid = \"{}\"", project.guest.machine_guid),
                "machine_guid = \"not-a-guid\"",
            ),
            valid.replace("file_count = 217", "file_count = 0"),
            valid.replace(
                &project.driver_manifest.sha256,
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            valid.replace(
                "catalog_name = \"NV_DISP.CAT\"",
                "catalog_name = \"nested\\\\NV_DISP.CAT\"",
            ),
            valid.replace(
                &format!(
                    "signer_thumbprint = \"{}\"",
                    project.driver_manifest.signer_thumbprint
                ),
                "signer_thumbprint = \"not-a-thumbprint\"",
            ),
            valid.replace(
                "signature_files = [\"NV_DISP.CAT\", \"nv_dispi.inf\", \"nvcuda64.dll\", \"nvwgf2umx.dll\"]",
                "signature_files = [\"nested\\\\NV_DISP.CAT\"]",
            ),
            valid.replace(
                "computer_name = \"TESTVM\"",
                "powershell_path = '''C:\\Temp\\powershell.exe'''\ncomputer_name = \"TESTVM\"",
            ),
            valid.replace(
                &format!(
                    "staging_root = '''{}'''",
                    project.guest.staging_root.display()
                ),
                "staging_root = '''..\\outside'''",
            ),
            valid.replace(
                &format!(
                    "session_timeout_seconds = {}",
                    project.guest.session_timeout.as_secs()
                ),
                "session_timeout_seconds = 0",
            ),
            valid.replace(
                &format!(
                    "staging_timeout_seconds = {}",
                    project.guest.staging_timeout.as_secs()
                ),
                "staging_timeout_seconds = 0",
            ),
            valid.replace(
                &format!(
                    "task_execution_timeout_seconds = {}",
                    project.runner.task_execution_timeout.as_secs()
                ),
                "task_execution_timeout_seconds = 300",
            ),
        ] {
            assert!(ProjectConfiguration::parse(&invalid).is_err());
        }
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
                MINIMAL.replace(
                    r"\\?\PCI#VEN_10DE&DEV_2D05&SUBSYS_8A151043&REV_A1#95B0EB63032DB04800#{064092b3-625e-43bf-9eb5-dc845897dd59}\GPUPARAV",
                    "Default GPU",
                ),
                ConfigError::InvalidGpuIdentity,
            ),
            (
                MINIMAL.replace(
                    r"\\?\PCI#VEN_10DE&DEV_2D05&SUBSYS_8A151043&REV_A1#95B0EB63032DB04800#{064092b3-625e-43bf-9eb5-dc845897dd59}\GPUPARAV",
                    r"\\?\PCI#VEN_*&DEV_*#*\GPUPARAV",
                ),
                ConfigError::InvalidGpuIdentity,
            ),
            (
                MINIMAL.replace("95B0EB63032DB04800", "95B0 EB63032DB04800"),
                ConfigError::InvalidGpuIdentity,
            ),
            (
                MINIMAL.replace("95B0EB63032DB04800", r"95B0EB63\..\child"),
                ConfigError::InvalidGpuIdentity,
            ),
            (
                MINIMAL.replace("VEN_10DE&DEV_2D05", "VEN_DEV_"),
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
        let fingerprint = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        assert!(Plan::new(CliOperation::Apply, fingerprint.into(), fingerprint.into()).is_ok());
        assert_eq!(
            Plan::new(CliOperation::Apply, "not-a-hash".into(), fingerprint.into()),
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
