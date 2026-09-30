//! Standalone GPU probe result contract and hardware-identity validation.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::ProjectConfiguration;

/// Canonical SHA-256 for the 256x256 opaque-magenta RGBA output.
pub const EXPECTED_IMAGE_SHA256: &str =
    "00f88da6c22b46ab45bfc5fbc6659e601ebcefe324d52a3302e614d4a7fb3de4";
/// Required output width.
pub const WIDTH: u32 = 256;
/// Required output height.
pub const HEIGHT: u32 = 256;

/// Stable standalone probe process exit classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitClass {
    /// Workload and correctness checks passed.
    Pass = 0,
    /// Intended hardware adapter was unavailable or mismatched.
    Adapter = 2,
    /// API or runtime was unavailable.
    Runtime = 3,
    /// GPU execution failed.
    Execution = 4,
    /// GPU output differed from the correctness oracle.
    IncorrectOutput = 5,
    /// The bounded operation timed out or was cancelled.
    Timeout = 6,
    /// Input or internal protocol was malformed.
    Internal = 7,
}

impl ExitClass {
    /// Convert the stable exit class into a process exit code.
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }
}

/// Exact selected adapter identity included in every D3D result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterIdentity {
    /// DXGI description with trailing NULs removed.
    pub description: String,
    /// PCI vendor identifier.
    pub vendor_id: u32,
    /// PCI device identifier.
    pub device_id: u32,
    /// PCI subsystem identifier.
    pub subsystem_id: u32,
    /// PCI revision identifier.
    pub revision: u32,
    /// Dedicated video memory reported by DXGI.
    pub dedicated_video_memory: u64,
    /// Windows adapter LUID formatted as fixed hexadecimal halves.
    pub luid: String,
    /// Whether DXGI marked the adapter as software.
    pub software: bool,
    /// Whether D3DKMT identifies an indirect-display adapter.
    pub indirect_display: bool,
    /// Whether D3DKMT identifies a paravirtualized adapter.
    pub paravirtualized: bool,
}

impl AdapterIdentity {
    /// Validate the exact configured hardware selection contract.
    ///
    /// # Errors
    /// Returns [`ProbeContractError::Adapter`] for software, default, ambiguous or
    /// otherwise mismatched identity data.
    pub fn validate(&self) -> Result<(), ProbeContractError> {
        let project = ProjectConfiguration::embedded().map_err(|_| ProbeContractError::Adapter)?;
        let target = project.slot;
        if self.software
            || self.vendor_id != target.gpu_vendor_id
            || self.device_id != target.gpu_device_id
            || self.subsystem_id != target.gpu_subsystem_id
            || self.revision != target.gpu_revision
            || self.dedicated_video_memory == 0
            || self.description != target.gpu_name
            || self.indirect_display
            || !is_luid(&self.luid)
            || self.luid == "00000000:00000000"
        {
            return Err(ProbeContractError::Adapter);
        }
        Ok(())
    }
}

/// Machine-readable successful standalone D3D probe result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeReport {
    /// Result schema version.
    pub schema: u32,
    /// Exact probe identifier (`d3d11-offscreen` or `d3d12-offscreen`).
    pub probe: String,
    /// Always `pass`; failures are emitted on stderr with a nonzero exit.
    pub status: String,
    /// Selected physical/partition adapter identity.
    pub adapter: AdapterIdentity,
    /// Negotiated feature level.
    pub feature_level: String,
    /// Shader profiles used by the pipeline.
    pub shader_profiles: String,
    /// Canonical image width.
    pub width: u32,
    /// Canonical image height.
    pub height: u32,
    /// SHA-256 of tightly packed RGBA output bytes.
    pub output_sha256: String,
    /// Measured wall-clock duration.
    pub duration_ms: u64,
}

/// Probe-result contract failure without echoing untrusted input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeContractError {
    /// JSON or schema fields were malformed.
    Malformed,
    /// Adapter identity did not match the required hardware.
    Adapter,
    /// Output dimensions, status or hash did not match the oracle.
    IncorrectOutput,
}

impl fmt::Display for ProbeContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Malformed => "malformed probe result",
            Self::Adapter => "probe adapter identity mismatch",
            Self::IncorrectOutput => "probe output mismatch",
        })
    }
}

impl std::error::Error for ProbeContractError {}

/// Parse and validate a successful probe result for the expected probe kind.
///
/// # Errors
/// Rejects unknown/missing JSON fields, fallback identities, non-pass status,
/// wrong dimensions or a noncanonical image hash.
pub fn parse_success_report(
    input: &str,
    expected_probe: &str,
) -> Result<ProbeReport, ProbeContractError> {
    let report: ProbeReport =
        serde_json::from_str(input).map_err(|_| ProbeContractError::Malformed)?;
    report.adapter.validate()?;
    if report.schema != 1 || report.probe != expected_probe || report.status != "pass" {
        return Err(ProbeContractError::Malformed);
    }
    if report.width != WIDTH
        || report.height != HEIGHT
        || report.output_sha256 != EXPECTED_IMAGE_SHA256
    {
        return Err(ProbeContractError::IncorrectOutput);
    }
    Ok(report)
}

/// Hash bytes into lowercase SHA-256 hexadecimal.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn is_luid(value: &str) -> bool {
    value.len() == 17
        && value.bytes().enumerate().all(|(index, byte)| {
            if index == 8 {
                byte == b':'
            } else {
                byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
            }
        })
}

#[cfg(test)]
mod tests {
    use super::{
        AdapterIdentity, EXPECTED_IMAGE_SHA256, ExitClass, ProbeContractError, ProbeReport,
        parse_success_report, sha256_hex,
    };

    fn report() -> ProbeReport {
        let target = crate::config::ProjectConfiguration::embedded()
            .unwrap()
            .slot;
        ProbeReport {
            schema: 1,
            probe: "d3d11-offscreen".into(),
            status: "pass".into(),
            adapter: AdapterIdentity {
                description: target.gpu_name,
                vendor_id: target.gpu_vendor_id,
                device_id: target.gpu_device_id,
                subsystem_id: target.gpu_subsystem_id,
                revision: target.gpu_revision,
                dedicated_video_memory: 8_000_000_000,
                luid: "01234567:89abcdef".into(),
                software: false,
                indirect_display: false,
                paravirtualized: false,
            },
            feature_level: "12_1".into(),
            shader_profiles: "vs_5_0,ps_5_0".into(),
            width: 256,
            height: 256,
            output_sha256: EXPECTED_IMAGE_SHA256.into(),
            duration_ms: 1,
        }
    }

    #[test]
    fn parses_exact_success_report() {
        let report = report();
        let json = serde_json::to_string(&report).unwrap();
        assert_eq!(parse_success_report(&json, "d3d11-offscreen"), Ok(report));
    }

    #[test]
    fn rejects_software_indirect_wrong_hardware_and_missing_luid() {
        let mut report = report();
        for mutate in [
            |adapter: &mut AdapterIdentity| adapter.software = true,
            |adapter: &mut AdapterIdentity| adapter.indirect_display = true,
            |adapter: &mut AdapterIdentity| adapter.vendor_id = 0x1414,
            |adapter: &mut AdapterIdentity| adapter.device_id = 0xffff,
            |adapter: &mut AdapterIdentity| adapter.luid = "00000000:00000000".into(),
        ] {
            let mut candidate = report.clone();
            mutate(&mut candidate.adapter);
            let json = serde_json::to_string(&candidate).unwrap();
            assert_eq!(
                parse_success_report(&json, "d3d11-offscreen"),
                Err(ProbeContractError::Adapter)
            );
        }
        report.adapter.description = "Microsoft Basic Render Driver".into();
        let json = serde_json::to_string(&report).unwrap();
        assert_eq!(
            parse_success_report(&json, "d3d11-offscreen"),
            Err(ProbeContractError::Adapter)
        );
    }

    #[test]
    fn rejects_malformed_unknown_and_incorrect_output() {
        assert_eq!(
            parse_success_report("{}", "d3d11-offscreen"),
            Err(ProbeContractError::Malformed)
        );
        let json = serde_json::to_string(&report()).unwrap();
        let unknown = json.replacen('{', "{\"unknown\":1,", 1);
        assert_eq!(
            parse_success_report(&unknown, "d3d11-offscreen"),
            Err(ProbeContractError::Malformed)
        );
        let mut wrong = report();
        wrong.output_sha256 = "0".repeat(64);
        let json = serde_json::to_string(&wrong).unwrap();
        assert_eq!(
            parse_success_report(&json, "d3d11-offscreen"),
            Err(ProbeContractError::IncorrectOutput)
        );
    }

    #[test]
    fn canonical_image_hash_matches_specification() {
        let rgba = [255_u8, 0, 255, 255].repeat(256 * 256);
        assert_eq!(sha256_hex(&rgba), EXPECTED_IMAGE_SHA256);
    }

    #[test]
    fn exit_classes_are_stable_and_distinct() {
        assert_eq!(
            [
                ExitClass::Pass.code(),
                ExitClass::Adapter.code(),
                ExitClass::Runtime.code(),
                ExitClass::Execution.code(),
                ExitClass::IncorrectOutput.code(),
                ExitClass::Timeout.code(),
                ExitClass::Internal.code(),
            ],
            [0, 2, 3, 4, 5, 6, 7]
        );
    }
}
