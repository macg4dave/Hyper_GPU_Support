//! Fixed privileged runner contract for the single disposable Hyper-V slot.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::OnceLock;

use serde::Deserialize;
use sha2::{Digest, Sha256};

/// Version-one local named-pipe endpoint. Remote pipe clients are rejected by
/// the Windows transport implementation.
pub const PIPE_NAME: &str = r"\\.\pipe\HyperGpuSupport.Runner.v1";
/// Maximum request or response frame accepted at the privilege boundary.
pub const FRAME_LIMIT: usize = 64 * 1024;

/// Administrator-owned principal enrollment used by both pipe endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Enrollment {
    schema: u32,
    runner_sid: String,
    client_sid: String,
}

impl Enrollment {
    /// SID of the dedicated least-privilege scheduled-task principal.
    #[must_use]
    pub fn runner_sid(&self) -> &str {
        &self.runner_sid
    }

    /// SID of the only unelevated account permitted to submit requests.
    #[must_use]
    pub fn client_sid(&self) -> &str {
        &self.client_sid
    }

    /// SDDL for a single local pipe owned by the runner, client, SYSTEM and
    /// Administrators. Remote clients remain separately rejected by pipe mode.
    #[must_use]
    pub fn pipe_sddl(&self) -> String {
        format!(
            "O:{}D:P(A;;GA;;;{})(A;;0x0012008b;;;{})(A;;GA;;;SY)(A;;GA;;;BA)",
            self.runner_sid, self.runner_sid, self.client_sid
        )
    }
}

/// Parse a strict administrator-owned principal enrollment.
///
/// # Errors
/// Rejects schema drift, unknown fields, duplicate identities and noncanonical
/// SID strings before either endpoint uses them for authentication.
pub fn parse_enrollment(input: &str) -> Result<Enrollment, RunnerError> {
    let enrollment: Enrollment =
        serde_json::from_str(input).map_err(|_| RunnerError::InvalidProtocol)?;
    if enrollment.schema != 1
        || !is_sid(&enrollment.runner_sid)
        || !is_sid(&enrollment.client_sid)
        || enrollment.runner_sid == enrollment.client_sid
    {
        return Err(RunnerError::InvalidProtocol);
    }
    Ok(enrollment)
}

/// Exact immutable runner policy installed outside the repository.
pub const POLICY_V1: &str = include_str!("../config/runner-policy-v1.json");

/// Typed, exact identity policy installed outside the repository.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunnerPolicy {
    schema: u32,
    slot: String,
    vm_id: String,
    vm_name: String,
    parent: String,
    parent_sha256: String,
    child: String,
    gpu_interface: String,
    vm_profile: crate::config::VmProfile,
    resources: crate::vm_settings::GpuResources,
    allowed_operations: Vec<String>,
}

impl RunnerPolicy {
    /// Verify settings pins against the compiled validated project configuration.
    #[must_use]
    pub fn settings_match(&self, project: &crate::config::ProjectConfiguration) -> bool {
        self.vm_profile == project.vm_profile
            && crate::vm_settings::GpuResources::desired(project)
                .is_ok_and(|value| value == self.resources)
    }
    /// Logical disposable-slot identifier.
    #[must_use]
    pub fn slot(&self) -> &str {
        &self.slot
    }

    /// Canonical enrolled VM GUID.
    #[must_use]
    pub fn vm_id(&self) -> &str {
        &self.vm_id
    }

    /// Exact enrolled VM display name.
    #[must_use]
    pub fn vm_name(&self) -> &str {
        &self.vm_name
    }

    /// Exact immutable parent VHDX path.
    #[must_use]
    pub fn parent(&self) -> &str {
        &self.parent
    }

    /// Expected immutable parent SHA-256.
    #[must_use]
    pub fn parent_sha256(&self) -> &str {
        &self.parent_sha256
    }

    /// Exact disposable child VHDX path.
    #[must_use]
    pub fn child(&self) -> &str {
        &self.child
    }

    /// Exact selected host GPU partition interface.
    #[must_use]
    pub fn gpu_interface(&self) -> &str {
        &self.gpu_interface
    }
}

/// Parse and validate a strict runner policy.
///
/// # Errors
/// Rejects schema drift, malformed identities/hashes, duplicate operations or
/// any operation outside the fixed version-one protocol allowlist. Parsing the
/// legacy reset entry does not authorize it; execution also checks the build gate.
pub fn parse_policy(input: &str) -> Result<RunnerPolicy, RunnerError> {
    let policy: RunnerPolicy =
        serde_json::from_str(input).map_err(|_| RunnerError::InvalidProtocol)?;
    policy
        .vm_profile
        .validate()
        .map_err(|_| RunnerError::InvalidProtocol)?;
    policy
        .resources
        .supported_by(&policy.resources)
        .map_err(|_| RunnerError::InvalidProtocol)?;
    if policy.schema != 1
        || policy.slot.is_empty()
        || policy.slot.len() > 128
        || !is_canonical_guid(&policy.vm_id)
        || policy.vm_name.is_empty()
        || policy.vm_name.len() > 128
        || policy.parent == policy.child
        || !is_lower_hex(&policy.parent_sha256, 64)
        || !is_gpu_interface(&policy.gpu_interface)
    {
        return Err(RunnerError::InvalidProtocol);
    }
    let mut operations = BTreeSet::new();
    for value in &policy.allowed_operations {
        let operation = Operation::parse(value).ok_or(RunnerError::InvalidProtocol)?;
        // The enrolled laboratory policy retains its reset entry for compatibility.
        // Executable authorization additionally checks the build's dev-harness gate.
        if !(policy_allows(operation) || operation == Operation::ResetSlot)
            || !operations.insert(operation.as_str())
        {
            return Err(RunnerError::InvalidProtocol);
        }
    }
    if operations.len() != 7 {
        return Err(RunnerError::InvalidProtocol);
    }
    Ok(policy)
}

/// Parse the exact policy compiled into this runner/client build.
///
/// # Errors
/// Returns [`RunnerError::InvalidProtocol`] if the checked-in generated policy
/// is malformed or inconsistent with the compiled operation set.
pub fn embedded_policy() -> Result<RunnerPolicy, RunnerError> {
    static POLICY: OnceLock<Result<RunnerPolicy, RunnerError>> = OnceLock::new();
    POLICY.get_or_init(|| parse_policy(POLICY_V1)).clone()
}

/// Result of the fixed reset operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetResult {
    /// Enrolled Hyper-V VM identifier.
    pub vm_id: String,
    /// Recreated child path.
    pub child: String,
    /// Verified golden-parent path.
    pub parent: String,
    /// Verified golden-parent SHA-256.
    pub parent_sha256: String,
}

/// Checked read-only observation of the enrolled VM shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectResult {
    /// Enrolled Hyper-V VM identifier.
    pub vm_id: String,
    /// Native Hyper-V state name.
    pub state: String,
    /// Count of attached GPU partition adapters.
    pub gpu_adapters: u32,
    /// Exact attached child path.
    pub child: String,
    /// Exact differencing parent path.
    pub parent: String,
    /// Verified parent SHA-256.
    pub parent_sha256: String,
    /// Exact selected host partition interface.
    pub gpu_interface: String,
}

/// Checked state transition returned by a fixed VM lifecycle operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleResult {
    /// Enrolled Hyper-V VM identifier.
    pub vm_id: String,
    /// State required immediately before the operation.
    pub previous_state: String,
    /// State observed after the operation completed.
    pub state: String,
    /// Count of attached GPU partition adapters preserved by the transition.
    pub gpu_adapters: u32,
    /// Exact attached child path.
    pub child: String,
    /// Exact differencing parent path.
    pub parent: String,
    /// Verified golden-parent SHA-256.
    pub parent_sha256: String,
}

/// Checked adapter-count transition returned by a fixed GPU assignment operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuAssignmentResult {
    /// Enrolled Hyper-V VM identifier.
    pub vm_id: String,
    /// VM state preserved by the operation.
    pub state: String,
    /// Adapter count required immediately before the operation.
    pub previous_gpu_adapters: u32,
    /// Adapter count observed after the operation completed.
    pub gpu_adapters: u32,
    /// Exact attached child path.
    pub child: String,
    /// Exact differencing parent path.
    pub parent: String,
    /// Verified golden-parent SHA-256 from the preceding locked inspection.
    pub parent_sha256: String,
    /// Exact selected host partition interface.
    pub gpu_interface: String,
}

/// Fixed operation names in the version-one runner protocol.
///
/// Availability is additionally restricted by the installed immutable policy;
/// recognizing an operation here does not authorize it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    /// Inspect the enrolled slot without mutation.
    Inspect,
    /// Recreate only the enrolled differencing child.
    ResetSlot,
    /// Start the enrolled VM.
    StartSlot,
    /// Request a graceful shutdown of the enrolled VM.
    ShutdownSlot,
    /// Attach the policy-pinned physical GPU.
    AssignGpu,
    /// Remove the enrolled VM's GPU partition adapter.
    RemoveGpu,
    /// Apply and freshly verify the pinned complete Hyper-V/GPU profile.
    ConfigureSlot,
    /// Stage the policy-pinned runtime manifest.
    StageRuntimeV1,
    /// Run one policy-pinned probe manifest.
    RunProbeV1,
    /// Read an existing result without mutation.
    ReadResult,
}

impl Operation {
    /// Parse an exact version-one operation name.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "inspect" => Some(Self::Inspect),
            "reset-slot" => Some(Self::ResetSlot),
            "start-slot" => Some(Self::StartSlot),
            "shutdown-slot" => Some(Self::ShutdownSlot),
            "assign-gpu" => Some(Self::AssignGpu),
            "remove-gpu" => Some(Self::RemoveGpu),
            "configure-slot" => Some(Self::ConfigureSlot),
            "stage-runtime-v1" => Some(Self::StageRuntimeV1),
            "run-probe-v1" => Some(Self::RunProbeV1),
            "read-result" => Some(Self::ReadResult),
            _ => None,
        }
    }

    /// Return the stable protocol spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inspect => "inspect",
            Self::ResetSlot => "reset-slot",
            Self::StartSlot => "start-slot",
            Self::ShutdownSlot => "shutdown-slot",
            Self::AssignGpu => "assign-gpu",
            Self::RemoveGpu => "remove-gpu",
            Self::ConfigureSlot => "configure-slot",
            Self::StageRuntimeV1 => "stage-runtime-v1",
            Self::RunProbeV1 => "run-probe-v1",
            Self::ReadResult => "read-result",
        }
    }
}

/// Return whether the reviewed policy and build authorize an operation.
///
/// This is intentionally explicit rather than inferred from protocol parsing.
/// Disposable reset additionally requires the non-default dev-harness feature.
#[must_use]
pub const fn policy_allows(operation: Operation) -> bool {
    if matches!(operation, Operation::ResetSlot) {
        return cfg!(feature = "dev-harness");
    }
    matches!(
        operation,
        Operation::Inspect
            | Operation::StartSlot
            | Operation::ShutdownSlot
            | Operation::AssignGpu
            | Operation::RemoveGpu
            | Operation::ConfigureSlot
    )
}

/// SHA-256 fingerprint bound to requests for the exact compiled policy bytes.
#[must_use]
pub fn policy_fingerprint() -> String {
    Sha256::digest(POLICY_V1.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Fully validated request at the unelevated-to-runner trust boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// Caller-generated identifier used to correlate audit and result records.
    pub request_id: String,
    /// Fixed typed operation.
    pub operation: Operation,
    /// Single-use nonce used for replay rejection.
    pub nonce: String,
    /// Fingerprint of the reviewed plan being applied.
    pub plan_fingerprint: String,
}

impl Request {
    /// Encode the canonical version-one wire representation.
    ///
    /// # Errors
    /// Returns [`RunnerError::InvalidProtocol`] if the compiled generated policy
    /// cannot supply the fixed slot identity.
    pub fn encode(&self) -> Result<String, RunnerError> {
        let policy = embedded_policy()?;
        Ok(format!(
            concat!(
                "schema\t1\nrequest_id\t{}\noperation\t{}\n",
                "slot\t{}\nvm_id\t{}\nnonce\t{}\nplan_fingerprint\t{}\n"
            ),
            self.request_id,
            self.operation.as_str(),
            policy.slot(),
            policy.vm_id(),
            self.nonce,
            self.plan_fingerprint
        ))
    }
}

/// Minimal response returned over the local runner pipe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    /// Correlates the response to the caller's request.
    pub request_id: String,
    /// Stable result category (`succeeded` or `failed`).
    pub status: String,
    /// Administrator-generated operation identifier, when execution began.
    pub operation_id: Option<String>,
    /// Bounded, secret-free diagnostic category.
    pub diagnostic: String,
}

impl Response {
    /// Encode the canonical version-one wire representation.
    #[must_use]
    pub fn encode(&self) -> String {
        format!(
            concat!(
                "schema\t1\nrequest_id\t{}\nstatus\t{}\noperation_id\t{}\n",
                "diagnostic\t{}\n"
            ),
            self.request_id,
            self.status,
            self.operation_id.as_deref().unwrap_or("none"),
            self.diagnostic
        )
    }
}

/// Runner protocol or policy failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerError {
    /// Installed policy differs from the compiled reviewed policy.
    PolicyMismatch,
    /// The native adapter returned malformed or incomplete output.
    InvalidProtocol,
    /// The request schema, field set or value encoding is invalid.
    InvalidRequest,
    /// The request names an identity other than the compiled fixed slot.
    IdentityMismatch,
    /// The request operation differs from the reviewed operation.
    OperationMismatch,
    /// The request plan no longer matches the plan authorized by the caller.
    StalePlan,
    /// The nonce has already been accepted.
    Replay,
    /// A request/response frame exceeds the fixed transport bound.
    FrameTooLarge,
}

impl fmt::Display for RunnerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PolicyMismatch => formatter.write_str("installed runner policy mismatch"),
            Self::InvalidProtocol => formatter.write_str("runner adapter returned invalid output"),
            Self::InvalidRequest => formatter.write_str("invalid runner request"),
            Self::IdentityMismatch => formatter.write_str("runner target identity mismatch"),
            Self::OperationMismatch => formatter.write_str("runner operation mismatch"),
            Self::StalePlan => formatter.write_str("runner request plan is stale"),
            Self::Replay => formatter.write_str("runner request replay rejected"),
            Self::FrameTooLarge => formatter.write_str("runner protocol frame exceeds limit"),
        }
    }
}

/// Parse a complete version-one response.
///
/// # Errors
/// Returns [`RunnerError::InvalidProtocol`] for an unknown field, malformed
/// identifier, unrecognized status or an unsafe diagnostic.
pub fn parse_response(input: &str) -> Result<Response, RunnerError> {
    let mut fields = parse_fields(input)?;
    if fields.len() != 5 || fields.remove("schema") != Some("1") {
        return Err(RunnerError::InvalidProtocol);
    }
    let request_id = take_field(&mut fields, "request_id")?;
    let status = take_field(&mut fields, "status")?;
    let operation_id = take_field(&mut fields, "operation_id")?;
    let diagnostic = take_field(&mut fields, "diagnostic")?;
    if !fields.is_empty()
        || !is_lower_hex(request_id, 32)
        || !matches!(status, "succeeded" | "failed")
        || (operation_id != "none" && !is_operation_id(operation_id))
        || diagnostic.len() > 512
        || diagnostic
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b'\t')
    {
        return Err(RunnerError::InvalidProtocol);
    }
    Ok(Response {
        request_id: request_id.to_owned(),
        status: status.to_owned(),
        operation_id: (operation_id != "none").then(|| operation_id.to_owned()),
        diagnostic: diagnostic.to_owned(),
    })
}

/// Atomically consume a nonce in administrator-owned state.
///
/// One create-new marker per canonical nonce makes replay rejection survive
/// process exits and crashes. The caller must create and ACL the directory
/// during installation; this function deliberately never creates it.
///
/// # Errors
/// Returns [`RunnerError::InvalidRequest`] for a malformed nonce,
/// [`RunnerError::Replay`] when the marker already exists, and the underlying
/// I/O error for unavailable or unwritable trusted state.
pub fn consume_nonce(directory: &Path, nonce: &str) -> Result<(), PersistentStateError> {
    if !is_lower_hex(nonce, 32) {
        return Err(PersistentStateError::Protocol(RunnerError::InvalidRequest));
    }
    let path = directory.join(format!("{nonce}.used"));
    let mut file = match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(PersistentStateError::Protocol(RunnerError::Replay));
        }
        Err(error) => return Err(PersistentStateError::Io(error)),
    };
    file.write_all(b"consumed\n")?;
    file.sync_all()?;
    Ok(())
}

/// Persistent state failure with protocol and native I/O categories separated.
#[derive(Debug)]
pub enum PersistentStateError {
    /// A stable protocol failure.
    Protocol(RunnerError),
    /// Native filesystem failure from administrator-owned state.
    Io(std::io::Error),
}

impl fmt::Display for PersistentStateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(error) => error.fmt(formatter),
            Self::Io(error) => write!(formatter, "runner replay state failure: {error}"),
        }
    }
}

impl std::error::Error for PersistentStateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Protocol(error) => Some(error),
            Self::Io(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for PersistentStateError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Write one little-endian length-prefixed protocol frame.
///
/// # Errors
/// Returns [`RunnerError::FrameTooLarge`] through [`FrameError`] when the frame
/// exceeds [`FRAME_LIMIT`], or preserves the underlying I/O error.
pub fn write_frame(writer: &mut impl Write, value: &[u8]) -> Result<(), FrameError> {
    if value.len() > FRAME_LIMIT {
        return Err(FrameError::Protocol(RunnerError::FrameTooLarge));
    }
    let length = u32::try_from(value.len()).expect("FRAME_LIMIT fits u32");
    writer.write_all(&length.to_le_bytes())?;
    writer.write_all(value)?;
    writer.flush()?;
    Ok(())
}

/// Read one little-endian length-prefixed protocol frame.
///
/// # Errors
/// Rejects oversized declarations before allocating and preserves short/native
/// I/O failures.
pub fn read_frame(reader: &mut impl Read) -> Result<Vec<u8>, FrameError> {
    let mut header = [0_u8; 4];
    reader.read_exact(&mut header)?;
    let length = u32::from_le_bytes(header) as usize;
    if length > FRAME_LIMIT {
        return Err(FrameError::Protocol(RunnerError::FrameTooLarge));
    }
    let mut value = vec![0_u8; length];
    reader.read_exact(&mut value)?;
    Ok(value)
}

/// Framed transport failure.
#[derive(Debug)]
pub enum FrameError {
    /// Stable protocol category.
    Protocol(RunnerError),
    /// Native stream error.
    Io(std::io::Error),
}

impl fmt::Display for FrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(error) => error.fmt(formatter),
            Self::Io(error) => write!(formatter, "runner transport failure: {error}"),
        }
    }
}

impl std::error::Error for FrameError {}

impl From<std::io::Error> for FrameError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Parse a complete tab-separated version-one request.
///
/// The fixed target identity remains mandatory even though callers cannot select
/// another target. Unknown and duplicate fields are rejected, which also prevents
/// adding paths, credentials, scripts or enrollment data to this protocol.
///
/// # Errors
/// Returns a structured error for malformed fields or a non-enrolled target.
pub fn parse_request(input: &str) -> Result<Request, RunnerError> {
    let policy = embedded_policy()?;
    let mut fields = parse_fields(input).map_err(|_| RunnerError::InvalidRequest)?;
    if fields.len() != 7 || fields.remove("schema") != Some("1") {
        return Err(RunnerError::InvalidRequest);
    }
    if fields.remove("slot") != Some(policy.slot())
        || fields.remove("vm_id") != Some(policy.vm_id())
    {
        return Err(RunnerError::IdentityMismatch);
    }
    let request_id = take_field(&mut fields, "request_id")?;
    let operation = Operation::parse(take_field(&mut fields, "operation")?)
        .ok_or(RunnerError::InvalidRequest)?;
    let nonce = take_field(&mut fields, "nonce")?;
    let plan_fingerprint = take_field(&mut fields, "plan_fingerprint")?;
    if !fields.is_empty()
        || !is_lower_hex(request_id, 32)
        || !is_lower_hex(nonce, 32)
        || !is_lower_hex(plan_fingerprint, 64)
    {
        return Err(RunnerError::InvalidRequest);
    }
    Ok(Request {
        request_id: request_id.to_owned(),
        operation,
        nonce: nonce.to_owned(),
        plan_fingerprint: plan_fingerprint.to_owned(),
    })
}

/// Validate operation and plan freshness, then consume a nonce before an effect.
///
/// The caller must persist `used_nonces` in administrator-owned state before
/// invoking the native adapter. Inserting before the effect deliberately makes
/// an uncertain/interrupted attempt non-retryable under the same nonce.
///
/// # Errors
/// Returns [`RunnerError::OperationMismatch`], [`RunnerError::StalePlan`] or
/// [`RunnerError::Replay`]. An operation/plan mismatch does not modify the ledger.
pub fn authorize_request(
    request: &Request,
    authorized_operation: Operation,
    authorized_plan_fingerprint: &str,
    used_nonces: &mut BTreeSet<String>,
) -> Result<(), RunnerError> {
    if request.operation != authorized_operation {
        return Err(RunnerError::OperationMismatch);
    }
    if request.plan_fingerprint != authorized_plan_fingerprint {
        return Err(RunnerError::StalePlan);
    }
    if !used_nonces.insert(request.nonce.clone()) {
        return Err(RunnerError::Replay);
    }
    Ok(())
}

fn parse_fields(input: &str) -> Result<BTreeMap<&str, &str>, RunnerError> {
    let mut fields = BTreeMap::new();
    for line in input.lines() {
        let Some((key, value)) = line.split_once('\t') else {
            return Err(RunnerError::InvalidProtocol);
        };
        if key.is_empty()
            || value.is_empty()
            || !key
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
            || fields.insert(key, value).is_some()
        {
            return Err(RunnerError::InvalidProtocol);
        }
    }
    Ok(fields)
}

fn take_field<'a>(fields: &mut BTreeMap<&str, &'a str>, key: &str) -> Result<&'a str, RunnerError> {
    fields.remove(key).ok_or(RunnerError::InvalidRequest)
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
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
    value.starts_with(r"\\?\PCI#")
        && value.ends_with(r"\GPUPARAV")
        && !value.chars().any(char::is_control)
        && value.len() <= 512
}

fn is_operation_id(value: &str) -> bool {
    let Some((seconds, nanos)) = value.split_once('-') else {
        return false;
    };
    !seconds.is_empty()
        && seconds.bytes().all(|byte| byte.is_ascii_digit())
        && nanos.len() == 9
        && nanos.bytes().all(|byte| byte.is_ascii_digit())
}

fn is_sid(value: &str) -> bool {
    let mut parts = value.split('-');
    parts.next() == Some("S")
        && parts.next() == Some("1")
        && parts.clone().count() >= 2
        && parts.all(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part == "0" || !part.starts_with('0'))
        })
}

impl std::error::Error for RunnerError {}

/// Verify the installed policy byte-for-byte.
///
/// # Errors
/// Returns [`RunnerError::PolicyMismatch`] when an administrator-installed policy
/// does not exactly match the reviewed policy compiled into this runner.
pub fn verify_policy(installed: &str) -> Result<(), RunnerError> {
    if installed == POLICY_V1 {
        Ok(())
    } else {
        Err(RunnerError::PolicyMismatch)
    }
}

/// Parse the fixed tab-separated reset result.
///
/// # Errors
/// Returns [`RunnerError::InvalidProtocol`] for unknown, duplicate, missing or
/// unsafe fields.
pub fn parse_reset_result(output: &str) -> Result<ResetResult, RunnerError> {
    let policy = embedded_policy()?;
    let mut fields = parse_fields(output)?;
    if fields.len() != 5 || fields.remove("status") != Some("ok") {
        return Err(RunnerError::InvalidProtocol);
    }
    let take = |fields: &mut BTreeMap<&str, &str>, key| {
        fields
            .remove(key)
            .map(str::to_owned)
            .ok_or(RunnerError::InvalidProtocol)
    };
    let result = ResetResult {
        vm_id: take(&mut fields, "vm_id")?,
        child: take(&mut fields, "child")?,
        parent: take(&mut fields, "parent")?,
        parent_sha256: take(&mut fields, "parent_sha256")?,
    };
    if !fields.is_empty()
        || result.vm_id != policy.vm_id()
        || result.child != policy.child()
        || result.parent != policy.parent()
        || result.parent_sha256 != policy.parent_sha256()
    {
        return Err(RunnerError::InvalidProtocol);
    }
    Ok(result)
}

/// Parse and identity-check the fixed read-only inspection result.
///
/// # Errors
/// Returns [`RunnerError::InvalidProtocol`] for missing, duplicate, unknown or
/// target-mismatched fields.
pub fn parse_inspect_result(output: &str) -> Result<InspectResult, RunnerError> {
    let policy = embedded_policy()?;
    let mut fields = parse_fields(output)?;
    if fields.len() != 8 || fields.remove("status") != Some("ok") {
        return Err(RunnerError::InvalidProtocol);
    }
    let result = InspectResult {
        vm_id: take_field(&mut fields, "vm_id")?.to_owned(),
        state: take_field(&mut fields, "state")?.to_owned(),
        gpu_adapters: take_field(&mut fields, "gpu_adapters")?
            .parse()
            .map_err(|_| RunnerError::InvalidProtocol)?,
        child: take_field(&mut fields, "child")?.to_owned(),
        parent: take_field(&mut fields, "parent")?.to_owned(),
        parent_sha256: take_field(&mut fields, "parent_sha256")?.to_owned(),
        gpu_interface: take_field(&mut fields, "gpu_interface")?.to_owned(),
    };
    if !fields.is_empty()
        || result.vm_id != policy.vm_id()
        || !matches!(result.state.as_str(), "Off" | "Running")
        || result.gpu_adapters > 1
        || result.child != policy.child()
        || result.parent != policy.parent()
        || result.parent_sha256 != policy.parent_sha256()
        || result.gpu_interface != policy.gpu_interface()
    {
        return Err(RunnerError::InvalidProtocol);
    }
    Ok(result)
}

/// Parse and identity-check a fixed start or graceful-shutdown result.
///
/// # Errors
/// Returns [`RunnerError::InvalidProtocol`] for an inapplicable operation or
/// missing, duplicate, unknown, malformed or target-mismatched fields.
pub fn parse_lifecycle_result(
    output: &str,
    operation: Operation,
) -> Result<LifecycleResult, RunnerError> {
    let policy = embedded_policy()?;
    let (expected_previous, expected_state) = match operation {
        Operation::StartSlot => ("Off", "Running"),
        Operation::ShutdownSlot => ("Running", "Off"),
        _ => return Err(RunnerError::InvalidProtocol),
    };
    let mut fields = parse_fields(output)?;
    if fields.len() != 8 || fields.remove("status") != Some("ok") {
        return Err(RunnerError::InvalidProtocol);
    }
    let result = LifecycleResult {
        vm_id: take_field(&mut fields, "vm_id")?.to_owned(),
        previous_state: take_field(&mut fields, "previous_state")?.to_owned(),
        state: take_field(&mut fields, "state")?.to_owned(),
        gpu_adapters: take_field(&mut fields, "gpu_adapters")?
            .parse()
            .map_err(|_| RunnerError::InvalidProtocol)?,
        child: take_field(&mut fields, "child")?.to_owned(),
        parent: take_field(&mut fields, "parent")?.to_owned(),
        parent_sha256: take_field(&mut fields, "parent_sha256")?.to_owned(),
    };
    if !fields.is_empty()
        || result.vm_id != policy.vm_id()
        || result.previous_state != expected_previous
        || result.state != expected_state
        || result.gpu_adapters > 1
        || result.child != policy.child()
        || result.parent != policy.parent()
        || result.parent_sha256 != policy.parent_sha256()
    {
        return Err(RunnerError::InvalidProtocol);
    }
    Ok(result)
}

/// Parse and identity-check a fixed GPU attach or detach result.
///
/// # Errors
/// Returns [`RunnerError::InvalidProtocol`] for an inapplicable operation or
/// missing, duplicate, unknown, malformed or target-mismatched fields.
pub fn parse_gpu_assignment_result(
    output: &str,
    operation: Operation,
) -> Result<GpuAssignmentResult, RunnerError> {
    let policy = embedded_policy()?;
    let (expected_previous, expected_count) = match operation {
        Operation::AssignGpu => (0, 1),
        Operation::RemoveGpu => (1, 0),
        _ => return Err(RunnerError::InvalidProtocol),
    };
    let mut fields = parse_fields(output)?;
    if fields.len() != 9 || fields.remove("status") != Some("ok") {
        return Err(RunnerError::InvalidProtocol);
    }
    let result = GpuAssignmentResult {
        vm_id: take_field(&mut fields, "vm_id")?.to_owned(),
        state: take_field(&mut fields, "state")?.to_owned(),
        previous_gpu_adapters: take_field(&mut fields, "previous_gpu_adapters")?
            .parse()
            .map_err(|_| RunnerError::InvalidProtocol)?,
        gpu_adapters: take_field(&mut fields, "gpu_adapters")?
            .parse()
            .map_err(|_| RunnerError::InvalidProtocol)?,
        child: take_field(&mut fields, "child")?.to_owned(),
        parent: take_field(&mut fields, "parent")?.to_owned(),
        parent_sha256: take_field(&mut fields, "parent_sha256")?.to_owned(),
        gpu_interface: take_field(&mut fields, "gpu_interface")?.to_owned(),
    };
    if !fields.is_empty()
        || result.vm_id != policy.vm_id()
        || result.state != "Off"
        || result.previous_gpu_adapters != expected_previous
        || result.gpu_adapters != expected_count
        || result.child != policy.child()
        || result.parent != policy.parent()
        || result.parent_sha256 != policy.parent_sha256()
        || result.gpu_interface != policy.gpu_interface()
    {
        return Err(RunnerError::InvalidProtocol);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::io::Cursor;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        Enrollment, FRAME_LIMIT, GpuAssignmentResult, InspectResult, LifecycleResult, Operation,
        POLICY_V1, PersistentStateError, Request, ResetResult, Response, RunnerError,
        authorize_request, consume_nonce, embedded_policy, parse_enrollment,
        parse_gpu_assignment_result, parse_inspect_result, parse_lifecycle_result, parse_request,
        parse_reset_result, parse_response, policy_allows, policy_fingerprint, read_frame,
        verify_policy, write_frame,
    };

    fn request_fixture() -> String {
        Request {
            request_id: "0123456789abcdef0123456789abcdef".into(),
            operation: Operation::ResetSlot,
            nonce: "fedcba9876543210fedcba9876543210".into(),
            plan_fingerprint: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .into(),
        }
        .encode()
        .unwrap()
    }

    #[test]
    fn policy_requires_exact_reviewed_bytes() {
        assert_eq!(verify_policy(POLICY_V1), Ok(()));
        let policy = embedded_policy().unwrap();
        let project = crate::config::ProjectConfiguration::embedded().unwrap();
        assert_eq!(policy.gpu_interface(), project.slot.gpu_interface);
        assert_eq!(policy.vm_id(), project.slot.vm_id);
        assert_eq!(policy.parent(), project.slot.parent_path.to_string_lossy());
        assert_eq!(policy_fingerprint().len(), 64);
        assert_eq!(
            policy_allows(Operation::ResetSlot),
            cfg!(feature = "dev-harness")
        );
        assert!(policy_allows(Operation::Inspect));
        assert!(policy_allows(Operation::StartSlot));
        assert!(policy_allows(Operation::ShutdownSlot));
        assert!(policy_allows(Operation::AssignGpu));
        assert!(policy_allows(Operation::RemoveGpu));
        assert!(!policy_allows(Operation::StageRuntimeV1));
        assert_eq!(
            verify_policy(&POLICY_V1.replace("reset-slot", "arbitrary-command")),
            Err(RunnerError::PolicyMismatch)
        );
    }

    #[test]
    fn enrollment_is_strict_and_builds_pinned_pipe_acl() {
        let text = concat!(
            "{\"schema\":1,",
            "\"runner_sid\":\"S-1-5-21-1-2-3-1001\",",
            "\"client_sid\":\"S-1-5-21-1-2-3-1000\"}"
        );
        let enrollment = parse_enrollment(text).unwrap();
        assert_eq!(
            enrollment,
            Enrollment {
                schema: 1,
                runner_sid: "S-1-5-21-1-2-3-1001".into(),
                client_sid: "S-1-5-21-1-2-3-1000".into(),
            }
        );
        assert_eq!(
            enrollment.pipe_sddl(),
            concat!(
                "O:S-1-5-21-1-2-3-1001D:P(A;;GA;;;S-1-5-21-1-2-3-1001)",
                "(A;;0x0012008b;;;S-1-5-21-1-2-3-1000)(A;;GA;;;SY)(A;;GA;;;BA)"
            )
        );
        for bad in [
            text.replace("\"schema\":1", "\"schema\":2"),
            text.replace("S-1-5-21-1-2-3-1000", "S-1-5-21-1-2-3-1001"),
            text.replace("S-1-5-21-1-2-3-1000", "S-01-5"),
            text.replace("}", ",\"path\":\"C:\\\\temp\"}"),
        ] {
            assert_eq!(parse_enrollment(&bad), Err(RunnerError::InvalidProtocol));
        }
    }

    #[test]
    fn parses_fixed_typed_request() {
        let expected = Request {
            request_id: "0123456789abcdef0123456789abcdef".into(),
            operation: Operation::ResetSlot,
            nonce: "fedcba9876543210fedcba9876543210".into(),
            plan_fingerprint: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .into(),
        };
        let encoded = request_fixture();
        assert_eq!(parse_request(&encoded), Ok(expected.clone()));
        assert_eq!(expected.encode().unwrap(), encoded);
        for name in [
            "inspect",
            "reset-slot",
            "start-slot",
            "shutdown-slot",
            "assign-gpu",
            "remove-gpu",
            "stage-runtime-v1",
            "run-probe-v1",
            "read-result",
        ] {
            assert_eq!(Operation::parse(name).map(Operation::as_str), Some(name));
        }
        assert_eq!(Operation::parse("arbitrary-command"), None);
    }

    #[test]
    fn rejects_unknown_duplicate_sensitive_and_wrong_identity_fields() {
        let request = request_fixture();
        let policy = embedded_policy().unwrap();
        for request in [
            request.replace("schema\t1", "schema\t2"),
            request.replace("operation\treset-slot", "operation\tshell"),
            request.replace(&format!("slot\t{}", policy.slot()), "slot\tother"),
            request.replace(
                &format!("vm_id\t{}", policy.vm_id()),
                "vm_id\t00000000-0000-0000-0000-000000000000",
            ),
            format!("{request}path\tZ:\\other.vhdx\n"),
            format!("{request}credential\tsecret\n"),
            format!("{request}nonce\t00000000000000000000000000000000\n"),
            request.replace(
                "nonce\tfedcba9876543210fedcba9876543210",
                "nonce\tFEDCBA9876543210FEDCBA9876543210",
            ),
        ] {
            assert!(matches!(
                parse_request(&request),
                Err(RunnerError::InvalidRequest | RunnerError::IdentityMismatch)
            ));
        }
    }

    #[test]
    fn stale_plan_does_not_consume_nonce_and_replay_is_rejected() {
        let request = parse_request(&request_fixture()).unwrap();
        let mut used = BTreeSet::new();
        assert_eq!(
            authorize_request(&request, Operation::ResetSlot, &"f".repeat(64), &mut used),
            Err(RunnerError::StalePlan)
        );
        assert!(used.is_empty());
        assert_eq!(
            authorize_request(
                &request,
                Operation::ResetSlot,
                &request.plan_fingerprint,
                &mut used
            ),
            Ok(())
        );
        assert_eq!(
            authorize_request(
                &request,
                Operation::ResetSlot,
                &request.plan_fingerprint,
                &mut used
            ),
            Err(RunnerError::Replay)
        );
    }

    #[test]
    fn substituted_operation_is_rejected_without_consuming_nonce() {
        let mut request = parse_request(&request_fixture()).unwrap();
        request.operation = Operation::AssignGpu;
        let mut used = BTreeSet::new();
        assert_eq!(
            authorize_request(
                &request,
                Operation::ResetSlot,
                &request.plan_fingerprint,
                &mut used
            ),
            Err(RunnerError::OperationMismatch)
        );
        assert!(used.is_empty());
    }

    #[test]
    fn parses_exact_reset_result() {
        let policy = embedded_policy().unwrap();
        let output = format!(
            "status\tok\nvm_id\t{}\nchild\t{}\nparent\t{}\nparent_sha256\t{}\n",
            policy.vm_id(),
            policy.child(),
            policy.parent(),
            policy.parent_sha256()
        );
        assert_eq!(
            parse_reset_result(&output),
            Ok(ResetResult {
                vm_id: policy.vm_id().into(),
                child: policy.child().into(),
                parent: policy.parent().into(),
                parent_sha256: policy.parent_sha256().into(),
            })
        );
    }

    #[test]
    fn parses_identity_checked_inspection() {
        let policy = embedded_policy().unwrap();
        let output = format!(
            concat!(
                "status\tok\nvm_id\t{}\nstate\tOff\ngpu_adapters\t0\n",
                "child\t{}\nparent\t{}\nparent_sha256\t{}\ngpu_interface\t{}\n"
            ),
            policy.vm_id(),
            policy.child(),
            policy.parent(),
            policy.parent_sha256(),
            policy.gpu_interface()
        );
        assert_eq!(
            parse_inspect_result(&output),
            Ok(InspectResult {
                vm_id: policy.vm_id().into(),
                state: "Off".into(),
                gpu_adapters: 0,
                child: policy.child().into(),
                parent: policy.parent().into(),
                parent_sha256: policy.parent_sha256().into(),
                gpu_interface: policy.gpu_interface().into(),
            })
        );
        assert_eq!(
            parse_inspect_result(&output.replace("state\tOff", "state\tSaved")),
            Err(RunnerError::InvalidProtocol)
        );
    }

    #[test]
    fn parses_exact_lifecycle_transitions() {
        let policy = embedded_policy().unwrap();
        let output = format!(
            concat!(
                "status\tok\nvm_id\t{}\nprevious_state\tOff\nstate\tRunning\n",
                "gpu_adapters\t1\nchild\t{}\nparent\t{}\nparent_sha256\t{}\n"
            ),
            policy.vm_id(),
            policy.child(),
            policy.parent(),
            policy.parent_sha256()
        );
        assert_eq!(
            parse_lifecycle_result(&output, Operation::StartSlot),
            Ok(LifecycleResult {
                vm_id: policy.vm_id().into(),
                previous_state: "Off".into(),
                state: "Running".into(),
                gpu_adapters: 1,
                child: policy.child().into(),
                parent: policy.parent().into(),
                parent_sha256: policy.parent_sha256().into(),
            })
        );
        let shutdown = output
            .replace("state\tRunning", "state\tOff")
            .replace("previous_state\tOff", "previous_state\tRunning");
        assert!(parse_lifecycle_result(&shutdown, Operation::ShutdownSlot).is_ok());

        for invalid in [
            output.replace("state\tRunning", "state\tOff"),
            output.replace("gpu_adapters\t1", "gpu_adapters\t2"),
            output.replace(policy.vm_id(), "00000000-0000-0000-0000-000000000000"),
        ] {
            assert_eq!(
                parse_lifecycle_result(&invalid, Operation::StartSlot),
                Err(RunnerError::InvalidProtocol)
            );
        }
        assert_eq!(
            parse_lifecycle_result(&output, Operation::AssignGpu),
            Err(RunnerError::InvalidProtocol)
        );
    }

    #[test]
    fn parses_exact_gpu_assignment_transitions() {
        let policy = embedded_policy().unwrap();
        let output = format!(
            concat!(
                "status\tok\nvm_id\t{}\nstate\tOff\nprevious_gpu_adapters\t0\n",
                "gpu_adapters\t1\nchild\t{}\nparent\t{}\nparent_sha256\t{}\n",
                "gpu_interface\t{}\n"
            ),
            policy.vm_id(),
            policy.child(),
            policy.parent(),
            policy.parent_sha256(),
            policy.gpu_interface()
        );
        assert_eq!(
            parse_gpu_assignment_result(&output, Operation::AssignGpu),
            Ok(GpuAssignmentResult {
                vm_id: policy.vm_id().into(),
                state: "Off".into(),
                previous_gpu_adapters: 0,
                gpu_adapters: 1,
                child: policy.child().into(),
                parent: policy.parent().into(),
                parent_sha256: policy.parent_sha256().into(),
                gpu_interface: policy.gpu_interface().into(),
            })
        );
        let removed = output
            .replace("previous_gpu_adapters\t0", "previous_gpu_adapters\t1")
            .replace("\ngpu_adapters\t1", "\ngpu_adapters\t0");
        assert!(parse_gpu_assignment_result(&removed, Operation::RemoveGpu).is_ok());
        for invalid in [
            output.replace("state\tOff", "state\tRunning"),
            output.replace("gpu_adapters\t1", "gpu_adapters\t2"),
            output.replace("GPUPARAV", "wrong"),
        ] {
            assert_eq!(
                parse_gpu_assignment_result(&invalid, Operation::AssignGpu),
                Err(RunnerError::InvalidProtocol)
            );
        }
        assert_eq!(
            parse_gpu_assignment_result(&output, Operation::StartSlot),
            Err(RunnerError::InvalidProtocol)
        );
    }

    #[test]
    fn rejects_missing_duplicate_unknown_and_wrong_identity() {
        let configured_identity_with_unknown = format!(
            concat!(
                "status\tok\nvm_id\t{}\nchild\tx\nparent\ty\nunknown\tz\n",
                "parent_sha256\t{}\n"
            ),
            embedded_policy().unwrap().vm_id(),
            "0".repeat(64)
        );
        for output in [
            "".to_owned(),
            "status\tok\n".to_owned(),
            "status\tok\nstatus\tok\n".to_owned(),
            concat!(
                "status\tok\nvm_id\twrong\nchild\tx\nparent\ty\n",
                "parent_sha256\t0000000000000000000000000000000000000000000000000000000000000000\n"
            )
            .to_owned(),
            configured_identity_with_unknown,
        ] {
            assert_eq!(
                parse_reset_result(&output),
                Err(RunnerError::InvalidProtocol)
            );
        }
    }

    #[test]
    fn response_round_trips_and_rejects_unsafe_diagnostics() {
        let response = Response {
            request_id: "0123456789abcdef0123456789abcdef".into(),
            status: "succeeded".into(),
            operation_id: Some("1790391920-577596700".into()),
            diagnostic: "none".into(),
        };
        assert_eq!(parse_response(&response.encode()), Ok(response));
        assert_eq!(
            parse_response(concat!(
                "schema\t1\nrequest_id\t0123456789abcdef0123456789abcdef\n",
                "status\tfailed\noperation_id\tnone\ndiagnostic\tsecret\nvalue\n"
            )),
            Err(RunnerError::InvalidProtocol)
        );
    }

    #[test]
    fn framed_transport_is_bounded_and_exact() {
        let request = request_fixture();
        let mut bytes = Vec::new();
        write_frame(&mut bytes, request.as_bytes()).unwrap();
        assert_eq!(
            read_frame(&mut Cursor::new(bytes)).unwrap(),
            request.as_bytes()
        );

        let error = write_frame(&mut Vec::new(), &vec![0; FRAME_LIMIT + 1]).unwrap_err();
        assert!(matches!(
            error,
            super::FrameError::Protocol(RunnerError::FrameTooLarge)
        ));
        let declared = u32::try_from(FRAME_LIMIT + 1).unwrap().to_le_bytes();
        let error = read_frame(&mut Cursor::new(declared)).unwrap_err();
        assert!(matches!(
            error,
            super::FrameError::Protocol(RunnerError::FrameTooLarge)
        ));
    }

    #[test]
    fn persistent_nonce_survives_process_local_state() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "hyper-gpu-runner-nonce-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();
        let nonce = "fedcba9876543210fedcba9876543210";
        consume_nonce(&directory, nonce).unwrap();
        assert!(matches!(
            consume_nonce(&directory, nonce),
            Err(PersistentStateError::Protocol(RunnerError::Replay))
        ));
        assert!(matches!(
            consume_nonce(&directory, "not-a-nonce"),
            Err(PersistentStateError::Protocol(RunnerError::InvalidRequest))
        ));
        fs::remove_dir_all(directory).unwrap();
    }
}
