//! Fixed privileged runner contract for the single disposable Hyper-V slot.

use std::collections::BTreeMap;
use std::fmt;

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

/// Runner protocol or policy failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerError {
    /// Installed policy differs from the compiled reviewed policy.
    PolicyMismatch,
    /// The native adapter returned malformed or incomplete output.
    InvalidProtocol,
}

impl fmt::Display for RunnerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PolicyMismatch => formatter.write_str("installed runner policy mismatch"),
            Self::InvalidProtocol => formatter.write_str("runner adapter returned invalid output"),
        }
    }
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
    let mut fields = BTreeMap::new();
    for line in output.lines() {
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
    use super::{POLICY_V1, ResetResult, RunnerError, parse_reset_result, verify_policy};

    #[test]
    fn policy_requires_exact_reviewed_bytes() {
        assert_eq!(verify_policy(POLICY_V1), Ok(()));
        assert_eq!(
            verify_policy(&POLICY_V1.replace("reset-slot", "arbitrary-command")),
            Err(RunnerError::PolicyMismatch)
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
