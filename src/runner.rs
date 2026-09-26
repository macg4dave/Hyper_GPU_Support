//! Fixed privileged runner contract for the single disposable Hyper-V slot.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Logical name of the only slot accepted by the fixed runner.
pub const SLOT: &str = "gpu-pv-slot-01";
/// Hyper-V identifier of the enrolled disposable VM shell.
pub const VM_ID: &str = "2627e735-5b33-4104-b739-622727dd3a40";

/// Exact immutable runner policy installed outside the repository.
pub const POLICY_V1: &str = concat!(
    "{\n",
    "  \"schema\": 1,\n",
    "  \"slot\": \"gpu-pv-slot-01\",\n",
    "  \"vm_id\": \"2627e735-5b33-4104-b739-622727dd3a40\",\n",
    "  \"vm_name\": \"HyperGpuSupport-Disposable-01\",\n",
    "  \"parent\": \"Z:\\\\HyperGpuSupport\\\\images\\\\golden\\\\win11-pro-25h2-26200.9457-x64-v1\\\\parent.vhdx\",\n",
    "  \"parent_sha256\": \"0fb4dfe6dd51eed64d36e482e4f58d67c19922802e34aa6ae2d1f4ccd5daeb07\",\n",
    "  \"child\": \"Z:\\\\HyperGpuSupport\\\\images\\\\disposable\\\\gpu-pv-slot-01\\\\child.vhdx\",\n",
    "  \"allowed_operations\": [\"reset-slot\"]\n",
    "}\n"
);

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
            Self::StageRuntimeV1 => "stage-runtime-v1",
            Self::RunProbeV1 => "run-probe-v1",
            Self::ReadResult => "read-result",
        }
    }
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
    /// The request plan no longer matches the plan authorized by the caller.
    StalePlan,
    /// The nonce has already been accepted.
    Replay,
}

impl fmt::Display for RunnerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PolicyMismatch => formatter.write_str("installed runner policy mismatch"),
            Self::InvalidProtocol => formatter.write_str("runner adapter returned invalid output"),
            Self::InvalidRequest => formatter.write_str("invalid runner request"),
            Self::IdentityMismatch => formatter.write_str("runner target identity mismatch"),
            Self::StalePlan => formatter.write_str("runner request plan is stale"),
            Self::Replay => formatter.write_str("runner request replay rejected"),
        }
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
    let mut fields = parse_fields(input).map_err(|_| RunnerError::InvalidRequest)?;
    if fields.len() != 7 || fields.remove("schema") != Some("1") {
        return Err(RunnerError::InvalidRequest);
    }
    if fields.remove("slot") != Some(SLOT) || fields.remove("vm_id") != Some(VM_ID) {
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

/// Validate freshness and consume a nonce immediately before an effect.
///
/// The caller must persist `used_nonces` in administrator-owned state before
/// invoking the native adapter. Inserting before the effect deliberately makes
/// an uncertain/interrupted attempt non-retryable under the same nonce.
///
/// # Errors
/// Returns [`RunnerError::StalePlan`] or [`RunnerError::Replay`] without modifying
/// the ledger for a stale plan.
pub fn authorize_request(
    request: &Request,
    authorized_plan_fingerprint: &str,
    used_nonces: &mut BTreeSet<String>,
) -> Result<(), RunnerError> {
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
        || result.vm_id != "2627e735-5b33-4104-b739-622727dd3a40"
        || !result
            .parent_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || result.parent_sha256.len() != 64
    {
        return Err(RunnerError::InvalidProtocol);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        Operation, POLICY_V1, Request, ResetResult, RunnerError, authorize_request, parse_request,
        parse_reset_result, verify_policy,
    };

    const REQUEST: &str = concat!(
        "schema\t1\n",
        "request_id\t0123456789abcdef0123456789abcdef\n",
        "operation\treset-slot\n",
        "slot\tgpu-pv-slot-01\n",
        "vm_id\t2627e735-5b33-4104-b739-622727dd3a40\n",
        "nonce\tfedcba9876543210fedcba9876543210\n",
        "plan_fingerprint\t0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n",
    );

    #[test]
    fn policy_requires_exact_reviewed_bytes() {
        assert_eq!(verify_policy(POLICY_V1), Ok(()));
        assert_eq!(
            verify_policy(&POLICY_V1.replace("reset-slot", "arbitrary-command")),
            Err(RunnerError::PolicyMismatch)
        );
    }

    #[test]
    fn parses_fixed_typed_request() {
        assert_eq!(
            parse_request(REQUEST),
            Ok(Request {
                request_id: "0123456789abcdef0123456789abcdef".into(),
                operation: Operation::ResetSlot,
                nonce: "fedcba9876543210fedcba9876543210".into(),
                plan_fingerprint:
                    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into(),
            })
        );
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
        for request in [
            REQUEST.replace("schema\t1", "schema\t2"),
            REQUEST.replace("operation\treset-slot", "operation\tshell"),
            REQUEST.replace("slot\tgpu-pv-slot-01", "slot\tother"),
            REQUEST.replace(
                "vm_id\t2627e735-5b33-4104-b739-622727dd3a40",
                "vm_id\t00000000-0000-0000-0000-000000000000",
            ),
            format!("{REQUEST}path\tZ:\\other.vhdx\n"),
            format!("{REQUEST}credential\tsecret\n"),
            format!("{REQUEST}nonce\t00000000000000000000000000000000\n"),
            REQUEST.replace(
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
        let request = parse_request(REQUEST).unwrap();
        let mut used = BTreeSet::new();
        assert_eq!(
            authorize_request(&request, &"f".repeat(64), &mut used),
            Err(RunnerError::StalePlan)
        );
        assert!(used.is_empty());
        assert_eq!(
            authorize_request(&request, &request.plan_fingerprint, &mut used),
            Ok(())
        );
        assert_eq!(
            authorize_request(&request, &request.plan_fingerprint, &mut used),
            Err(RunnerError::Replay)
        );
    }

    #[test]
    fn parses_exact_reset_result() {
        let output = concat!(
            "status\tok\n",
            "vm_id\t2627e735-5b33-4104-b739-622727dd3a40\n",
            "child\tZ:\\child.vhdx\n",
            "parent\tZ:\\parent.vhdx\n",
            "parent_sha256\t0fb4dfe6dd51eed64d36e482e4f58d67c19922802e34aa6ae2d1f4ccd5daeb07\n"
        );
        assert_eq!(
            parse_reset_result(output),
            Ok(ResetResult {
                vm_id: "2627e735-5b33-4104-b739-622727dd3a40".into(),
                child: r"Z:\child.vhdx".into(),
                parent: r"Z:\parent.vhdx".into(),
                parent_sha256: "0fb4dfe6dd51eed64d36e482e4f58d67c19922802e34aa6ae2d1f4ccd5daeb07"
                    .into(),
            })
        );
    }

    #[test]
    fn rejects_missing_duplicate_unknown_and_wrong_identity() {
        for output in [
            "",
            "status\tok\n",
            "status\tok\nstatus\tok\n",
            concat!(
                "status\tok\nvm_id\twrong\nchild\tx\nparent\ty\n",
                "parent_sha256\t0000000000000000000000000000000000000000000000000000000000000000\n"
            ),
            concat!(
                "status\tok\nvm_id\t2627e735-5b33-4104-b739-622727dd3a40\n",
                "child\tx\nparent\ty\nunknown\tz\n",
                "parent_sha256\t0000000000000000000000000000000000000000000000000000000000000000\n"
            ),
        ] {
            assert_eq!(
                parse_reset_result(output),
                Err(RunnerError::InvalidProtocol)
            );
        }
    }
}
