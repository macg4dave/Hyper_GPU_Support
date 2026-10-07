//! Fixed guest validation decisions; Windows effects are behind one narrow adapter.

use crate::{
    config::ProjectConfiguration,
    probe::{ProbeReport, parse_success_report},
};
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Duration};

/// Exact test artifact allowlist. Shaders are embedded in the Rust D3D probes.
pub const INPUT_NAMES: [&str; 9] = [
    "hyper-gpu-validation-worker.exe",
    "d3d11-probe.exe",
    "d3d12-probe.exe",
    "cuda-identity.exe",
    "vectorAddDrv.exe",
    "vectorAdd_kernel64.fatbin",
    "vcruntime140.dll",
    "vcruntime140_1.dll",
    "msvcp140.dll",
];
/// Ordered checks, including the CUDA selection prerequisite.
pub const CHECK_NAMES: [&str; 7] = [
    "inputs",
    "readiness",
    "nvidia-smi",
    "d3d11",
    "d3d12",
    "cuda-identity",
    "cuda",
];

/// One verified transported artifact, without credentials or caller-selected commands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputIdentity {
    /// Fixed flat filename.
    pub name: String,
    /// Exact byte length.
    pub bytes: u64,
    /// SHA-256 measured at the host and checked in the guest.
    pub sha256: String,
}

/// Per-check outcome, preserving absence and lack of execution separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckStatus {
    /// Checked output passed.
    Pass,
    /// Execution or output failed.
    Fail,
    /// A prerequisite was missing or invalid.
    Blocked,
    /// The check was not executed.
    Untested,
}
/// Exact process evidence, bounded before parsing or publication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessOutput {
    /// Native exit code; absent if terminated or launch failed.
    pub exit_code: Option<i32>,
    /// Bounded UTF-8 standard output.
    pub stdout: String,
    /// Bounded UTF-8 standard error.
    pub stderr: String,
}
/// Selected virtual-render PnP devnode, never a software/display fallback.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PnpSample {
    /// Monotonic time since worker startup.
    pub elapsed_ms: u64,
    /// Exact devnode identity selected from vrd.inf.
    pub device_id: String,
    /// Observed name.
    pub name: String,
    /// ConfigManagerErrorCode, including nonzero errors.
    pub problem: u32,
}
/// Named check and its actual evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckResult {
    /// Fixed check name.
    pub name: String,
    /// Outcome of this check.
    pub status: CheckStatus,
    /// Bounded diagnostic without credentials.
    pub detail: String,
    /// Process evidence, if executed.
    pub process: Option<ProcessOutput>,
}
/// Complete reusable guest-validation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationReport {
    /// Report schema.
    pub schema: u32,
    /// Enrolled VM.
    pub vm_id: String,
    /// Guest OS build and architecture measured by the worker.
    pub guest_build: String,
    /// Windows x64 is required.
    pub architecture: String,
    /// Verified input identities.
    pub inputs: Vec<InputIdentity>,
    /// Bounded readiness evidence.
    pub samples: Vec<PnpSample>,
    /// Every fixed check, even when blocked or untested.
    pub checks: Vec<CheckResult>,
}
impl ValidationReport {
    /// Create a report before any effects.
    #[must_use]
    pub fn new(project: &ProjectConfiguration) -> Self {
        Self {
            schema: 1,
            vm_id: project.slot.vm_id.clone(),
            guest_build: String::new(),
            architecture: String::new(),
            inputs: vec![],
            samples: vec![],
            checks: CHECK_NAMES
                .iter()
                .map(|name| CheckResult {
                    name: (*name).into(),
                    status: CheckStatus::Untested,
                    detail: String::new(),
                    process: None,
                })
                .collect(),
        }
    }
    /// Exit 0 requires every essential check, including input and CUDA selection checks.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        if self.checks.len() == CHECK_NAMES.len()
            && self
                .checks
                .iter()
                .zip(CHECK_NAMES)
                .all(|(c, n)| c.name == n && c.status == CheckStatus::Pass)
        {
            0
        } else {
            1
        }
    }
    /// Classify a preflight failure without claiming workload execution.
    pub fn block(&mut self, detail: impl Into<String>) {
        self.checks[0].status = CheckStatus::Blocked;
        self.checks[0].detail = detail.into();
    }
}

/// Fixed native effects used by the worker and deterministic test fakes.
pub trait ValidationAdapter {
    /// Monotonic elapsed time.
    fn elapsed(&self) -> Duration;
    /// Wait no longer than the finite deadline permits.
    fn wait(&mut self, duration: Duration);
    /// Read exactly one vrd.inf devnode; missing/duplicate/query failure is an error.
    fn sample(&mut self) -> Result<PnpSample, String>;
    /// Run only the named fixed workload with its fixed arguments.
    fn run(&mut self, name: &str, timeout: Duration) -> Result<ProcessOutput, String>;
}

/// Observe sustained Code 0 then execute and check the fixed essential workloads.
/// Input integrity must already have passed; failures do not erase successful checks.
pub fn validate_workloads(
    project: &ProjectConfiguration,
    report: &mut ValidationReport,
    adapter: &mut impl ValidationAdapter,
) {
    if report.checks[0].status != CheckStatus::Pass {
        return;
    }
    let cfg = &project.validation;
    let end = adapter.elapsed() + Duration::from_secs(cfg.readiness_timeout_seconds);
    let mut since = None;
    let mut identity = None;
    let mut ready = false;
    while adapter.elapsed() < end {
        let sample = adapter.sample();
        if adapter.elapsed() > end {
            report.checks[1].detail = "readiness query exceeded deadline".into();
            break;
        }
        match sample {
            Ok(sample)
                if sample.name == project.slot.gpu_name
                    && !sample.device_id.is_empty()
                    && sample.device_id.len() <= 128 =>
            {
                if identity.as_ref().is_some_and(|id| id != &sample.device_id) {
                    report.checks[1].detail =
                        "virtual-render devnode changed during observation".into();
                    break;
                }
                identity = Some(sample.device_id.clone());
                if sample.problem == 0 {
                    let observed = Duration::from_millis(sample.elapsed_ms);
                    let start = *since.get_or_insert(observed);
                    ready =
                        observed.saturating_sub(start) >= Duration::from_secs(cfg.stable_seconds);
                } else {
                    since = None;
                }
                report.samples.push(sample);
                if ready {
                    break;
                }
            }
            Ok(_) => {
                report.checks[1].detail = "virtual-render identity mismatch".into();
                break;
            }
            Err(error) => {
                report.checks[1].detail = error;
                break;
            }
        }
        adapter.wait(
            Duration::from_millis(cfg.sample_milliseconds)
                .min(end.saturating_sub(adapter.elapsed())),
        );
    }
    report.checks[1].status = if ready {
        CheckStatus::Pass
    } else {
        CheckStatus::Fail
    };
    if !ready {
        if report.checks[1].detail.is_empty() {
            report.checks[1].detail = "sustained Code 0 deadline expired".into();
        }
        for check in &mut report.checks[2..] {
            check.status = CheckStatus::Blocked;
            check.detail = "GPU readiness did not pass".into();
        }
        return;
    }
    let worker_end = Duration::from_secs(cfg.worker_timeout_seconds);
    let mut graphics: Option<ProbeReport> = None;
    let mut cuda_selected = false;
    for (index, name) in CHECK_NAMES.iter().copied().enumerate().skip(2) {
        if name == "cuda" && !cuda_selected {
            report.checks[index].status = CheckStatus::Blocked;
            report.checks[index].detail = "CUDA device selection did not pass".into();
            continue;
        }
        let remaining = worker_end.saturating_sub(adapter.elapsed());
        if remaining.is_zero() {
            report.checks[index].status = CheckStatus::Blocked;
            report.checks[index].detail = "worker deadline exhausted".into();
            continue;
        }
        let outcome = adapter.run(
            name,
            remaining.min(Duration::from_secs(cfg.process_timeout_seconds)),
        );
        let checked = match &outcome {
            Ok(output) if output.exit_code == Some(0) => match name {
                "nvidia-smi" => check_smi(&output.stdout, project),
                "d3d11" | "d3d12" => {
                    let kind = if name == "d3d11" {
                        "d3d11-offscreen"
                    } else {
                        "d3d12-offscreen"
                    };
                    parse_success_report(output.stdout.trim(), kind)
                        .map_err(|e| e.to_string())
                        .and_then(|result| {
                            if !result.adapter.paravirtualized {
                                return Err("guest probe did not select the GPU partition".into());
                            }
                            if graphics
                                .as_ref()
                                .is_some_and(|other| other.adapter != result.adapter)
                            {
                                return Err("D3D adapter identities differ".into());
                            }
                            if name == "d3d11" {
                                graphics = Some(result);
                            }
                            Ok(())
                        })
                }
                "cuda-identity" => check_cuda_identity(&output.stdout, project).map(|_| {
                    cuda_selected = true;
                }),
                "cuda" => check_cuda_output(&output.stdout, project),
                _ => Err("unknown check".into()),
            },
            Ok(_) => Err("process exited unsuccessfully".into()),
            Err(error) => Err(error.clone()),
        };
        report.checks[index].status = if checked.is_ok() {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        };
        report.checks[index].detail = checked.err().unwrap_or_default();
        report.checks[index].process = outcome.ok();
    }
}

/// Recheck every claimed pass at the host result boundary.
/// # Errors
/// Rejects partial/malformed evidence, fabricated readiness and changed adapter identity.
pub fn verify_report_evidence(
    report: &ValidationReport,
    project: &ProjectConfiguration,
) -> Result<(), String> {
    if report.samples.len() > 181
        || report.checks.len() != CHECK_NAMES.len()
        || report.checks.iter().zip(CHECK_NAMES).any(|(c, n)| {
            c.name != n
                || c.detail.len() > 4096
                || c.process
                    .as_ref()
                    .is_some_and(|p| p.stdout.len() > 4096 || p.stderr.len() > 4096)
        })
    {
        return Err("validation evidence exceeds contract".into());
    }
    if report.checks[1].status == CheckStatus::Pass {
        let window = report
            .samples
            .iter()
            .rev()
            .take_while(|s| s.problem == 0)
            .collect::<Vec<_>>();
        let first = window.last().ok_or("missing readiness evidence")?;
        let last = window.first().ok_or("missing readiness evidence")?;
        if window.len() < 2
            || last.elapsed_ms.saturating_sub(first.elapsed_ms)
                < project.validation.stable_seconds * 1000
            || report.samples.windows(2).any(|s| {
                s[0].elapsed_ms >= s[1].elapsed_ms
                    || s[1].elapsed_ms - s[0].elapsed_ms
                        > project.validation.sample_milliseconds + 5000
                    || s[0].device_id != s[1].device_id
            })
            || report.samples.iter().any(|s| {
                s.device_id.is_empty()
                    || s.name != project.slot.gpu_name
                    || s.elapsed_ms > project.validation.readiness_timeout_seconds * 1000
            })
        {
            return Err("invalid sustained readiness evidence".into());
        }
    }
    let mut graphics = None;
    for (index, name) in CHECK_NAMES.iter().enumerate().skip(2) {
        let check = &report.checks[index];
        if check.status != CheckStatus::Pass {
            continue;
        }
        if report.checks[0].status != CheckStatus::Pass
            || report.checks[1].status != CheckStatus::Pass
        {
            return Err("workload passed without prerequisites".into());
        }
        let output = check.process.as_ref().ok_or("missing process evidence")?;
        if output.exit_code != Some(0) {
            return Err("successful check has failing exit".into());
        }
        match *name {
            "nvidia-smi" => check_smi(&output.stdout, project)?,
            "d3d11" | "d3d12" => {
                let p = parse_success_report(
                    output.stdout.trim(),
                    if *name == "d3d11" {
                        "d3d11-offscreen"
                    } else {
                        "d3d12-offscreen"
                    },
                )
                .map_err(|e| e.to_string())?;
                if !p.adapter.paravirtualized || graphics.as_ref().is_some_and(|a| a != &p.adapter)
                {
                    return Err("guest graphics adapter mismatch".into());
                }
                graphics = Some(p.adapter);
            }
            "cuda-identity" => {
                let identity = check_cuda_identity(&output.stdout, project)?;
                if graphics
                    .as_ref()
                    .is_some_and(|a| a.luid != identity.dxgi_luid)
                {
                    return Err("CUDA companion graphics identity mismatch".into());
                }
            }
            "cuda" => {
                if report.checks[5].status != CheckStatus::Pass {
                    return Err("CUDA computation lacks safe selection".into());
                }
                check_cuda_output(&output.stdout, project)?;
            }
            _ => return Err("unknown workload evidence".into()),
        }
    }
    Ok(())
}

fn check_smi(text: &str, project: &ProjectConfiguration) -> Result<(), String> {
    let lines: Vec<_> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() != 1 {
        return Err("nvidia-smi returned absent/ambiguous GPUs".into());
    }
    let fields: Vec<_> = lines[0].split(',').map(str::trim).collect();
    if fields.len() != 3
        || fields[0] != project.slot.gpu_name
        || fields[1] != project.validation.nvidia_driver_version
        || !valid_uuid(fields[2].strip_prefix("GPU-").unwrap_or(""))
    {
        return Err("nvidia-smi identity/driver mismatch".into());
    }
    Ok(())
}
/// Checked CUDA identity with LUID comparison retained only as a diagnostic.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CudaIdentity {
    /// Schema version.
    pub schema: u32,
    /// Fixed companion identifier.
    pub probe: String,
    /// Checked hardware-selection result.
    pub status: String,
    /// Explicit selected ordinal.
    pub ordinal: i32,
    /// Native CUDA enumeration count; one is required for safe guest sample selection.
    pub device_count: i32,
    /// Observed model.
    pub name: String,
    /// Native UUID.
    pub uuid: String,
    /// Native compute capability.
    pub compute_capability: String,
    /// CUDA namespace LUID.
    pub luid: String,
    /// CUDA device node mask.
    pub device_node_mask: u32,
    /// CUDA Driver API runtime version.
    pub driver_version: i32,
    /// Strictly selected D3D partition's LUID, for diagnostics.
    pub dxgi_luid: String,
}
fn valid_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
        && value.bytes().any(|b| b.is_ascii_hexdigit() && b != b'0')
}
/// Validate the one-GPU CUDA selection without requiring cross-namespace LUID equality.
/// # Errors
/// Malformed output, multiple devices or mismatched hardware are refused.
pub fn check_cuda_identity(
    text: &str,
    project: &ProjectConfiguration,
) -> Result<CudaIdentity, String> {
    let value: CudaIdentity =
        serde_json::from_str(text.trim()).map_err(|_| "malformed CUDA identity")?;
    if value.schema != 1
        || value.probe != "cuda-identity"
        || value.status != "pass"
        || value.ordinal != 0
        || value.device_count != 1
        || value.name != project.slot.gpu_name
        || !valid_uuid(&value.uuid)
        || value.compute_capability
            != format!(
                "{}.{}",
                project.slot.cuda_compute_capability_major,
                project.slot.cuda_compute_capability_minor
            )
        || value.driver_version <= 0
        || !crate::probe::is_luid(&value.luid)
        || !crate::probe::is_luid(&value.dxgi_luid)
        || value.dxgi_luid == "00000000:00000000"
    {
        return Err("CUDA identity/selection mismatch".into());
    }
    Ok(value)
}
fn check_cuda_output(text: &str, project: &ProjectConfiguration) -> Result<(), String> {
    if text
        .lines()
        .filter(|line| line.trim() == "Result = PASS")
        .count()
        != 1
        || text.contains("Result = FAIL")
        || text
            .lines()
            .filter(|line| {
                line.trim() == format!("> Using CUDA Device [0]: {}", project.slot.gpu_name)
            })
            .count()
            != 1
    {
        return Err("CUDA checked output/selected device mismatch".into());
    }
    Ok(())
}

/// Verify the complete flat allowlist and ordinary files immediately before execution.
/// # Errors
/// Missing, changed, duplicate, oversized and reparse inputs fail closed.
pub fn verify_inputs(root: &Path, inputs: &[InputIdentity]) -> Result<(), String> {
    if inputs.len() != INPUT_NAMES.len() {
        return Err("incomplete validation inputs".into());
    }
    for name in INPUT_NAMES {
        let found: Vec<_> = inputs.iter().filter(|i| i.name == name).collect();
        if found.len() != 1 {
            return Err("duplicate/missing validation input".into());
        }
        let input = found[0];
        if input.bytes == 0
            || input.bytes > 64 * 1024 * 1024
            || input.sha256.len() != 64
            || !input
                .sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err("malformed validation input".into());
        }
        let file = root.join(name);
        let metadata = std::fs::symlink_metadata(&file)
            .map_err(|e| format!("missing validation input {name}: {e}"))?;
        if !metadata.is_file()
            || unsafe_path_with_anchor(&file, root)?
            || metadata.len() != input.bytes
            || crate::guest::hash_file(&file).map_err(|e| e.to_string())? != input.sha256
        {
            return Err(format!("validation input integrity mismatch: {name}"));
        }
    }
    Ok(())
}
pub(crate) fn unsafe_path(path: &Path) -> Result<bool, String> {
    let root = path.ancestors().last().ok_or("path root missing")?;
    unsafe_path_with_anchor(path, root)
}
pub(crate) fn unsafe_path_with_anchor(path: &Path, anchor: &Path) -> Result<bool, String> {
    if !path.starts_with(anchor) {
        return Ok(true);
    }
    for ancestor in path.ancestors() {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        let m = std::fs::symlink_metadata(ancestor).map_err(|e| e.to_string())?;
        if m.file_type().is_symlink() {
            return Ok(true);
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if m.file_attributes() & 0x400 != 0 {
                return Ok(true);
            }
        }
        if ancestor == anchor {
            return Ok(false);
        }
    }
    Err("source anchor missing".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, VecDeque};
    struct Fake {
        time: Duration,
        codes: VecDeque<Result<u32, String>>,
        outputs: BTreeMap<String, Result<ProcessOutput, String>>,
        calls: Vec<String>,
    }
    impl ValidationAdapter for Fake {
        fn elapsed(&self) -> Duration {
            self.time
        }
        fn wait(&mut self, d: Duration) {
            self.time += d;
        }
        fn sample(&mut self) -> Result<PnpSample, String> {
            Ok(PnpSample {
                elapsed_ms: self.time.as_millis() as u64,
                device_id: "virtual-render".into(),
                name: ProjectConfiguration::embedded().unwrap().slot.gpu_name,
                problem: self.codes.pop_front().unwrap_or(Ok(0))?,
            })
        }
        fn run(&mut self, n: &str, _: Duration) -> Result<ProcessOutput, String> {
            self.calls.push(n.into());
            self.outputs
                .remove(n)
                .unwrap_or_else(|| Err("absent workload".into()))
        }
    }
    fn project() -> ProjectConfiguration {
        let mut p = ProjectConfiguration::embedded().unwrap();
        p.validation.stable_seconds = 2;
        p.validation.readiness_timeout_seconds = 5;
        p
    }
    fn d3d(project: &ProjectConfiguration, probe: &str) -> String {
        serde_json::json!({"schema":1,"probe":probe,"status":"pass","adapter":{"description":project.slot.gpu_name,"vendor_id":project.slot.gpu_vendor_id,"device_id":project.slot.gpu_device_id,"subsystem_id":project.slot.gpu_subsystem_id,"revision":project.slot.gpu_revision,"dedicated_video_memory":8000000000_u64,"luid":"01234567:89abcdef","software":false,"indirect_display":false,"paravirtualized":true},"feature_level":"12_1","shader_profiles":if probe == "d3d11-offscreen" { "vs_5_0,ps_5_0" } else { "vs_6_0,ps_6_0,max_6_9" },"width":256,"height":256,"output_sha256":crate::probe::EXPECTED_IMAGE_SHA256,"duration_ms":1}).to_string()
    }
    fn cuda(project: &ProjectConfiguration) -> String {
        serde_json::to_string(&CudaIdentity {
            schema: 1,
            probe: "cuda-identity".into(),
            status: "pass".into(),
            ordinal: 0,
            device_count: 1,
            name: project.slot.gpu_name.clone(),
            uuid: "12345678-1234-1234-1234-123456789abc".into(),
            compute_capability: "12.0".into(),
            luid: "abcdef00:00000001".into(),
            device_node_mask: 1,
            driver_version: 13040,
            dxgi_luid: "01234567:89abcdef".into(),
        })
        .unwrap()
    }
    fn fake(project: &ProjectConfiguration) -> Fake {
        let outputs = [
            (
                "nvidia-smi",
                format!(
                    "{}, {}, GPU-12345678-1234-1234-1234-123456789abc",
                    project.slot.gpu_name, project.validation.nvidia_driver_version
                ),
            ),
            ("d3d11", d3d(project, "d3d11-offscreen")),
            ("d3d12", d3d(project, "d3d12-offscreen")),
            ("cuda-identity", cuda(project)),
            (
                "cuda",
                format!(
                    "> Using CUDA Device [0]: {}\nResult = PASS\n",
                    project.slot.gpu_name
                ),
            ),
        ]
        .into_iter()
        .map(|(name, stdout)| {
            (
                name.into(),
                Ok(ProcessOutput {
                    exit_code: Some(0),
                    stdout,
                    stderr: String::new(),
                }),
            )
        })
        .collect();
        Fake {
            time: Duration::ZERO,
            codes: VecDeque::new(),
            outputs,
            calls: vec![],
        }
    }
    fn ready_report(project: &ProjectConfiguration) -> ValidationReport {
        let mut report = ValidationReport::new(project);
        report.checks[0].status = CheckStatus::Pass;
        report
    }
    #[test]
    fn sustained_readiness_and_every_workload_pass_with_distinct_cuda_luid() {
        let p = project();
        let mut adapter = fake(&p);
        let mut report = ready_report(&p);
        validate_workloads(&p, &mut report, &mut adapter);
        assert_eq!(report.exit_code(), 0);
        assert_eq!(adapter.calls.len(), 5);
        assert_eq!(report.samples.len(), 3);
        verify_report_evidence(&report, &p).unwrap();
        let mut wrong = report.clone();
        wrong.samples.last_mut().unwrap().elapsed_ms = 1001;
        assert!(verify_report_evidence(&wrong, &p).is_err());
        wrong = report;
        wrong.checks[6].process.as_mut().unwrap().exit_code = Some(1);
        assert!(verify_report_evidence(&wrong, &p).is_err());
    }
    #[test]
    fn nonzero_resets_window_and_timeout_or_absence_blocks_workloads() {
        let p = project();
        let mut adapter = fake(&p);
        adapter.codes = VecDeque::from([Ok(0), Ok(43), Ok(0), Ok(0), Ok(0)]);
        let mut report = ready_report(&p);
        validate_workloads(&p, &mut report, &mut adapter);
        assert_eq!(report.exit_code(), 0);
        assert_eq!(report.samples.len(), 5);
        verify_report_evidence(&report, &p).unwrap();
        for codes in [
            VecDeque::from([Err("absent or duplicate vrd.inf device".into())]),
            VecDeque::from([const { Ok(43) }; 5]),
        ] {
            let mut adapter = fake(&p);
            adapter.codes = codes;
            let mut report = ready_report(&p);
            validate_workloads(&p, &mut report, &mut adapter);
            assert_eq!(report.checks[1].status, CheckStatus::Fail);
            assert!(adapter.calls.is_empty());
            assert!(
                report.checks[2..]
                    .iter()
                    .all(|c| c.status == CheckStatus::Blocked)
            );
        }
    }
    #[test]
    fn malformed_identity_and_timeout_do_not_execute_cuda_and_preserve_other_results() {
        let p = project();
        for output in [
            Ok(ProcessOutput {
                exit_code: Some(0),
                stdout: "{}".into(),
                stderr: String::new(),
            }),
            Err("deadline expired".into()),
        ] {
            let mut adapter = fake(&p);
            adapter.outputs.insert("cuda-identity".into(), output);
            let mut report = ready_report(&p);
            validate_workloads(&p, &mut report, &mut adapter);
            assert_eq!(report.exit_code(), 1);
            assert_eq!(report.checks[2].status, CheckStatus::Pass);
            assert_eq!(report.checks[5].status, CheckStatus::Fail);
            assert_eq!(report.checks[6].status, CheckStatus::Blocked);
            assert!(!adapter.calls.iter().any(|n| n == "cuda"));
        }
    }
    #[test]
    fn rejects_wrong_cuda_selection_and_noncanonical_success_lines() {
        let p = project();
        let good = cuda(&p);
        for key in ["device_count", "ordinal"] {
            let mut value: serde_json::Value = serde_json::from_str(&good).unwrap();
            value[key] = 2.into();
            assert!(check_cuda_identity(&value.to_string(), &p).is_err());
        }
        let good = format!(
            "> Using CUDA Device [0]: {}\nResult = PASS",
            p.slot.gpu_name
        );
        assert!(check_cuda_output(&good, &p).is_ok());
        for bad in [
            "Result = PASS".to_string(),
            good.replace("[0]", "[1]"),
            format!("{good}\nResult = PASS"),
            format!("{good}\nResult = FAIL"),
        ] {
            assert!(check_cuda_output(&bad, &p).is_err());
        }
        assert!(
            check_smi(
                &format!(
                    "{}, {}",
                    p.slot.gpu_name, p.validation.nvidia_driver_version
                ),
                &p
            )
            .is_err()
        );
        assert!(
            check_smi(
                "software, 616.92, GPU-12345678-1234-1234-1234-123456789abc",
                &p
            )
            .is_err()
        );
    }
    #[test]
    fn input_failure_does_not_claim_execution_and_config_rejects_bad_budgets() {
        let p = project();
        let mut report = ValidationReport::new(&p);
        report.block("missing runtime");
        let mut adapter = fake(&p);
        validate_workloads(&p, &mut report, &mut adapter);
        assert_eq!(report.exit_code(), 1);
        assert!(adapter.calls.is_empty());
        assert!(
            report.checks[1..]
                .iter()
                .all(|c| c.status == CheckStatus::Untested)
        );
        let text = include_str!("../../../config/project.toml");
        for (from, to) in [
            ("stable_seconds = 120", "stable_seconds = 181"),
            ("sample_milliseconds = 1000", "sample_milliseconds = 100"),
            ("worker_timeout_seconds = 360", "worker_timeout_seconds = 1"),
        ] {
            assert!(ProjectConfiguration::parse(&text.replace(from, to)).is_err());
        }
    }
    #[test]
    fn ordinary_inputs_reject_tampering_missing_files_and_duplicates() {
        let root = std::env::current_dir()
            .unwrap()
            .canonicalize()
            .unwrap()
            .join("local")
            .join(format!("validation-input-test-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let inputs = INPUT_NAMES
            .iter()
            .map(|name| {
                std::fs::write(root.join(name), b"test").unwrap();
                InputIdentity {
                    name: (*name).into(),
                    bytes: 4,
                    sha256: crate::probe::sha256_hex(b"test"),
                }
            })
            .collect::<Vec<_>>();
        verify_inputs(&root, &inputs).unwrap();
        std::fs::write(root.join(INPUT_NAMES[0]), b"evil").unwrap();
        assert!(verify_inputs(&root, &inputs).is_err());
        std::fs::write(root.join(INPUT_NAMES[0]), b"test").unwrap();
        let mut duplicate = inputs.clone();
        duplicate[1] = duplicate[0].clone();
        assert!(verify_inputs(&root, &duplicate).is_err());
        std::fs::remove_file(root.join(INPUT_NAMES[1])).unwrap();
        assert!(verify_inputs(&root, &inputs).is_err());
        std::fs::remove_dir_all(&root).unwrap();
    }
}
