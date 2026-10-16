//! Rust guest preparation with a fixed PowerShell Direct session/transfer/launch bridge.
use crate::{
    credentials::Credential,
    model::{Gpu, Target},
    payload::{self, Manifest},
    process, runner, windows_paths,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::{Duration, Instant},
};

const MANIFEST_LIMIT: u64 = 32 * 1024 * 1024;
mod os;

fn read_preparation_manifest(input: impl Read) -> Result<Manifest, String> {
    let mut bytes = Vec::new();
    input
        .take(MANIFEST_LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MANIFEST_LIMIT {
        return Err("preparation manifest exceeds safety limit".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

/// Receipt from a fully verified guest preparation.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    /// Current complete manifest digest.
    pub digest: String,
    /// Discovered guest identity, not a contributor configuration pin.
    pub machine_guid: String,
    /// Verified file count for this run.
    pub files: usize,
}
/// Authenticate every discovered file through embedded signatures or catalog membership.
pub fn validate_trust(manifest: &Manifest) -> Result<(), String> {
    crate::trust::validate(manifest)
}
#[derive(Serialize)]
struct BridgeRequest<'a> {
    vm_id: &'a str,
    username: &'a str,
    password: &'a str,
    install: PathBuf,
    mode: &'a str,
    manifest: Option<&'a Manifest>,
    digest: String,
    vendor: u32,
    device: u32,
    artifacts: Vec<Artifact>,
    diagnostics: bool,
    verification: Option<String>,
    windows_11_builds: &'static [u32],
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct VerificationInput {
    target: Target,
    manifest: Manifest,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct VerificationReport {
    payload: Receipt,
    graphics: crate::probe::ProbeReport,
}
#[derive(Serialize)]
struct Artifact {
    name: String,
    hash: String,
    bytes: u64,
}
#[derive(Deserialize)]
struct BridgeResult {
    output: String,
    #[serde(default)]
    worker_exit: Option<i32>,
    #[serde(default)]
    transport_hresult: Option<i32>,
}

fn transport_script(module_literal: &str) -> String {
    include_str!("guest_transport.ps1")
        .replace("__HYPERV_MODULE__", module_literal)
        .replace(
            "__TRANSFER_CHUNK_BYTES__",
            &crate::guest_transfer::CHUNK_BYTES.to_string(),
        )
}

fn supervision_failure(error: process::BoundedProcessError) -> String {
    use crate::reporting::{Component, ComponentFailure, FailureCause, FailureCode};
    let (cause, code) = match error {
        process::BoundedProcessError::Timeout => (FailureCause::TimedOut, None),
        process::BoundedProcessError::Launch { kind, code } => (
            if kind == std::io::ErrorKind::PermissionDenied {
                FailureCause::AccessDenied
            } else {
                FailureCause::Unavailable
            },
            code.map(|code| FailureCode::HResult(0x80070000 | (code as u32 & 0xffff))),
        ),
        process::BoundedProcessError::OutputTooLarge => (FailureCause::InvalidReply, None),
        process::BoundedProcessError::Execution(error) => {
            if let Some(failure) = ComponentFailure::decode(&error) {
                return failure.into_error();
            }
            (FailureCause::Unavailable, None)
        }
    };
    ComponentFailure {
        stage: None,
        component: Component::GuestTransport,
        cause,
        code,
    }
    .into_error()
}

fn bridge(
    t: &Target,
    credential: &Credential,
    mode: &str,
    manifest: Option<&Manifest>,
    gpu: Option<&Gpu>,
    progress: &mut dyn FnMut(crate::workflow::PreparationProgress) -> Result<(), String>,
) -> Result<String, String> {
    crate::diagnostics::run(
        "guest-transport",
        &format!("vm={} mode={mode}", t.vm_id),
        || {
            let install = runner::install_directory()?;
            let artifacts = ["hyper-gpu-guest.exe", "d3d11-probe.exe"]
                .into_iter()
                .filter(|_| mode != "compatibility")
                .map(|name| {
                    Ok(Artifact {
                        name: name.into(),
                        hash: payload::hash_file(&install.join(name))?,
                        bytes: fs::metadata(install.join(name))
                            .map_err(|e| e.to_string())?
                            .len(),
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            let sizes = artifacts
                .iter()
                .map(|a| a.bytes)
                .chain(
                    manifest
                        .filter(|_| mode == "prepare")
                        .into_iter()
                        .flat_map(|m| m.files.iter().map(|f| f.bytes)),
                )
                .collect();
            let monitor = Arc::new(crate::guest_transfer::Monitor::for_operation(
                sizes,
                Instant::now(),
                mode == "verify",
            )?);
            let request = BridgeRequest {
                vm_id: &t.vm_id,
                username: &credential.username,
                password: &credential.password,
                install,
                mode,
                manifest: manifest.filter(|_| mode == "prepare"),
                digest: manifest
                    .map(Manifest::digest)
                    .transpose()?
                    .map(|digest| {
                        if mode == "verify" {
                            format!("verify-{digest}")
                        } else {
                            digest
                        }
                    })
                    .or_else(|| (mode == "compatibility").then(String::new))
                    .ok_or("current manifest required for guest operation")?,
                vendor: gpu.map_or(0, |g| g.vendor),
                device: gpu.map_or(0, |g| g.device),
                artifacts,
                diagnostics: crate::diagnostics::enabled()?,
                verification: if mode == "verify" {
                    Some(
                        serde_json::to_string(&VerificationInput {
                            target: t.clone(),
                            manifest: manifest
                                .ok_or("verification requires current manifest")?
                                .clone(),
                        })
                        .map_err(|e| e.to_string())?,
                    )
                } else {
                    None
                },
                windows_11_builds: crate::guest_compatibility::WINDOWS_11_BUILDS,
            };
            let bytes =
                zeroize::Zeroizing::new(serde_json::to_vec(&request).map_err(|e| e.to_string())?);
            let windows = windows_paths::windows_directory().map_err(|e| e.to_string())?;
            let module = windows.join("System32/WindowsPowerShell/v1.0/Modules");
            // Inbox modules may use versioned subdirectories. Importing the trusted
            // module directory lets PowerShell's loader select its installed manifest.
            crate::security::verify_system(&module.join("Hyper-V"))?;
            let module_literal = module.join("Hyper-V").to_string_lossy().replace('\'', "''");
            let script = transport_script(&module_literal);
            let mut command = Command::new(
                windows_paths::windows_powershell_executable().map_err(|e| e.to_string())?,
            );
            command.env("PSModulePath", &module).args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &script,
            ]);
            let result = if mode == "compatibility" {
                process::bounded_process_with_limit(
                    command,
                    Duration::from_secs(210),
                    16384,
                    &bytes,
                )
            } else {
                process::bounded_process_with_monitor(
                    command,
                    Duration::from_secs(3600),
                    1024 * 1024,
                    &bytes,
                    monitor.clone(),
                    &mut || {
                        if let Some(p) = monitor.take_progress(Instant::now())? {
                            progress(crate::workflow::PreparationProgress::Copying {
                                bytes: p.bytes,
                                total: p.total,
                                files: p.files,
                                count: p.count,
                            })?;
                            if p.done {
                                progress(crate::workflow::PreparationProgress::CopyComplete)?;
                            }
                        }
                        for phase in monitor.take_phases()? {
                            progress(phase)?;
                        }
                        Ok(())
                    },
                )
            }
            .map_err(supervision_failure)?;
            if result.exit_code != Some(0) {
                if let Ok(reply) = serde_json::from_str::<BridgeResult>(result.stdout.trim())
                    && let Some(failure) = crate::reporting::ComponentFailure::decode(&reply.output)
                {
                    return Err(failure.into_error());
                }
                if let Ok(reply) = serde_json::from_str::<BridgeResult>(result.stdout.trim())
                    && let Some(code) = reply.transport_hresult
                {
                    return Err(crate::reporting::ComponentFailure {
                        stage: None,
                        component: crate::reporting::Component::GuestTransport,
                        cause: crate::reporting::FailureCause::NativeCall,
                        code: Some(crate::reporting::FailureCode::HResult(code as u32)),
                    }
                    .into_error());
                }
                return Err(crate::reporting::ComponentFailure {
                    stage: None,
                    component: crate::reporting::Component::GuestTransport,
                    cause: crate::reporting::FailureCause::ProcessFailed,
                    code: result.exit_code.map(crate::reporting::FailureCode::Exit),
                }
                .into_error());
            }
            let reply: BridgeResult = serde_json::from_str(result.stdout.trim()).map_err(|_| {
                crate::reporting::ComponentFailure {
                    stage: None,
                    component: crate::reporting::Component::GuestTransport,
                    cause: crate::reporting::FailureCause::InvalidReply,
                    code: None,
                }
                .into_error()
            })?;
            if let Some(code) = reply.worker_exit
                && code != 0
            {
                if let Some(failure) = crate::reporting::ComponentFailure::decode(&reply.output) {
                    return Err(failure.into_error());
                }
                return Err(crate::reporting::ComponentFailure {
                    stage: None,
                    component: crate::reporting::Component::GuestTransport,
                    cause: crate::reporting::FailureCause::ProcessFailed,
                    code: Some(crate::reporting::FailureCode::Exit(code)),
                }
                .into_error());
            }
            if reply.worker_exit != Some(0) {
                return Err(crate::reporting::ComponentFailure {
                    stage: None,
                    component: crate::reporting::Component::GuestTransport,
                    cause: crate::reporting::FailureCause::InvalidReply,
                    code: None,
                }
                .into_error());
            }
            if mode != "compatibility" {
                monitor.require_worker_phases().map_err(|_| {
                    crate::reporting::ComponentFailure {
                        stage: None,
                        component: crate::reporting::Component::GuestTransport,
                        cause: crate::reporting::FailureCause::InvalidReply,
                        code: None,
                    }
                    .into_error()
                })?;
            }
            Ok(reply.output)
        },
    )
}
/// Read OS identity over an authenticated session to an already running VM.
/// This transport mode returns before guest bootstrap, file transfer or launch.
pub(crate) fn check_installation_os(t: &Target, credential: &Credential) -> Result<(), String> {
    let text = bridge(t, credential, "compatibility", None, None, &mut |_| Ok(()))
        .map_err(|_| crate::guest_compatibility::unknown())?;
    let identity =
        serde_json::from_str(&text).map_err(|_| crate::guest_compatibility::unknown())?;
    crate::guest_compatibility::require_supported(&identity)
}

/// Copy and independently verify the current complete payload.
pub fn prepare(t: &Target, manifest: &Manifest, credential: &Credential) -> Result<String, String> {
    prepare_with_progress(t, manifest, credential, &mut |_| Ok(()))
}
/// Prepare with real acknowledged transport totals, before independent receipt verification.
/// Events carry acknowledged counters and the post-copy integrity/installation boundary.
pub fn prepare_with_progress(
    t: &Target,
    manifest: &Manifest,
    credential: &Credential,
    progress: &mut dyn FnMut(crate::workflow::PreparationProgress) -> Result<(), String>,
) -> Result<String, String> {
    // The prepare bridge rechecks the running OS in its own authenticated
    // session before bootstrap writes, including for direct callers.
    let text = bridge(t, credential, "prepare", Some(manifest), None, progress)?;
    let receipt: Receipt =
        serde_json::from_str(text.trim()).map_err(|_| "invalid guest preparation receipt")?;
    let digest = manifest.digest()?;
    if receipt.digest != digest
        || receipt.files != manifest.files.len()
        || receipt.machine_guid.is_empty()
    {
        return Err(crate::reporting::ComponentFailure {
            stage: None,
            component: crate::reporting::Component::GuestReceipt,
            cause: crate::reporting::FailureCause::ReadbackMismatch,
            code: None,
        }
        .into_error());
    }
    Ok(digest)
}
/// Require current driver payload/receipt parity, PnP Code 0 and selected-hardware rendering.
pub fn verify(
    t: &Target,
    gpu: &Gpu,
    manifest: &Manifest,
    credential: &Credential,
) -> Result<(), String> {
    verify_with_progress(t, gpu, manifest, credential, &mut |_| Ok(()))
}

/// Explicit verification with real file-check and graphics phase events.
pub fn verify_with_progress(
    t: &Target,
    gpu: &Gpu,
    manifest: &Manifest,
    credential: &Credential,
    progress: &mut dyn FnMut(crate::workflow::PreparationProgress) -> Result<(), String>,
) -> Result<(), String> {
    let text = bridge(t, credential, "verify", Some(manifest), Some(gpu), progress)?;
    let checked: VerificationReport = serde_json::from_str(text.trim()).map_err(|_| {
        crate::reporting::ComponentFailure {
            stage: None,
            component: crate::reporting::Component::GuestPayload,
            cause: crate::reporting::FailureCause::InvalidReply,
            code: None,
        }
        .into_error()
    })?;
    if checked.payload.digest != manifest.digest()?
        || checked.payload.files != manifest.files.len()
        || checked.payload.machine_guid.is_empty()
    {
        return Err(crate::reporting::ComponentFailure {
            stage: None,
            component: crate::reporting::Component::GuestPayload,
            cause: crate::reporting::FailureCause::ReadbackMismatch,
            code: None,
        }
        .into_error());
    }
    let report = crate::probe::parse_success_report(
        &serde_json::to_string(&checked.graphics).map_err(|e| e.to_string())?,
        "d3d11-offscreen",
    )
    .map_err(|_| {
        crate::reporting::ComponentFailure {
            stage: None,
            component: crate::reporting::Component::GuestGraphics,
            cause: crate::reporting::FailureCause::InvalidReply,
            code: None,
        }
        .into_error()
    })?;
    if report.adapter.vendor_id != gpu.vendor
        || report.adapter.device_id != gpu.device
        || !report.adapter.paravirtualized
    {
        return Err(crate::reporting::ComponentFailure {
            stage: None,
            component: crate::reporting::Component::GuestGraphics,
            cause: crate::reporting::FailureCause::ReadbackMismatch,
            code: None,
        }
        .into_error());
    }
    Ok(())
}
/// Fixed guest worker entry point. Sources/destinations derive only from a protected transport bundle.
pub fn worker(mode: &str, bundle: &Path, vendor: u32, device: u32) -> Result<String, String> {
    worker_with_progress(mode, bundle, vendor, device, &mut |_| Ok(()))
}

/// Fixed worker entry with bounded, non-secret phase events emitted before work.
/// Failed delivery refuses entry into the next phase; effects already begun remain uncertain.
pub fn worker_with_progress(
    mode: &str,
    bundle: &Path,
    vendor: u32,
    device: u32,
    progress: &mut dyn FnMut(crate::workflow::PreparationProgress) -> Result<(), String>,
) -> Result<String, String> {
    let _deadline = process::WorkerDeadline::start(Duration::from_secs(3500));
    if !process::is_elevated()? {
        return Err("guest worker requires administrator rights".into());
    }
    if mode == "prepare" {
        os::require_local()?;
    }
    let expected = crate::runner::folder(&windows::Win32::UI::Shell::FOLDERID_ProgramFiles)?
        .join("HyperGpuSupport/Guest");
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let root = executable.parent().ok_or("guest worker location missing")?;
    if root != expected {
        return Err("guest worker is outside its protected installation".into());
    }
    crate::security::verify(root)?;
    crate::security::verify(&executable)?;
    if bundle.parent() != Some(root)
        || !bundle.file_name().is_some_and(|n| {
            n.to_string_lossy()
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
    {
        return Err("invalid transport bundle".into());
    }
    crate::security::verify(bundle)?;
    if mode == "verify" {
        use std::{io::Read, os::windows::fs::OpenOptionsExt};
        let request_path = bundle.join("verification.json");
        crate::security::verify(&request_path)?;
        let mut input = Vec::new();
        fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&request_path)
            .map_err(|e| e.to_string())?
            .take(32 * 1024 * 1024 + 1)
            .read_to_end(&mut input)
            .map_err(|e| e.to_string())?;
        if input.len() > 32 * 1024 * 1024 {
            return Err("verification manifest exceeds safety limit".into());
        }
        let request: VerificationInput =
            serde_json::from_slice(&input).map_err(|_| "invalid verification input")?;
        request.target.validate()?;
        crate::guest_payload::validate(&request.manifest, &request.target.gpu_interface)?;
        if bundle.file_name().and_then(|n| n.to_str())
            != Some(format!("verify-{}", request.manifest.digest()?).as_str())
        {
            return Err("verification bundle identity mismatch".into());
        }
        let windows = windows_paths::windows_directory().map_err(|e| e.to_string())?;
        let _lock = preparation_lock(root)?;
        progress(crate::workflow::PreparationProgress::CheckingGuestFiles)?;
        let payload = crate::diagnostics::run(
            "guest-current-payload",
            "current destination hashes and preparation receipt",
            || {
                crate::guest_payload::check(
                    &request.manifest,
                    &windows,
                    &root.join("prepared.json"),
                    &machine_guid().map_err(|_| {
                        crate::reporting::ComponentFailure {
                            stage: None,
                            component: crate::reporting::Component::GuestPayload,
                            cause: crate::reporting::FailureCause::Unavailable,
                            code: None,
                        }
                        .into_error()
                    })?,
                )
            },
        )?;
        progress(crate::workflow::PreparationProgress::CheckingGraphics)?;
        crate::diagnostics::run("guest-pnp-health", "selected GPU", health)?;
        let mut command = Command::new(root.join("d3d11-probe.exe"));
        command.args([vendor.to_string(), device.to_string()]);
        let out = crate::diagnostics::run(
            "guest-d3d11-probe",
            &format!("vendor={vendor} device={device}"),
            || process::bounded_process(command, Duration::from_secs(60)),
        )?;
        if out.exit_code != Some(0) {
            return Err(crate::reporting::ComponentFailure {
                stage: None,
                component: crate::reporting::Component::GuestGraphics,
                cause: crate::reporting::FailureCause::RenderingFailed,
                code: out.exit_code.map(crate::reporting::FailureCode::Exit),
            }
            .into_error());
        }
        let report = crate::probe::parse_success_report(out.stdout.trim(), "d3d11-offscreen")
            .map_err(|_| {
                crate::reporting::ComponentFailure {
                    stage: None,
                    component: crate::reporting::Component::GuestGraphics,
                    cause: crate::reporting::FailureCause::InvalidReply,
                    code: None,
                }
                .into_error()
            })?;
        if report.adapter.vendor_id != vendor
            || report.adapter.device_id != device
            || !report.adapter.paravirtualized
        {
            return Err(crate::reporting::ComponentFailure {
                stage: None,
                component: crate::reporting::Component::GuestGraphics,
                cause: crate::reporting::FailureCause::ReadbackMismatch,
                code: None,
            }
            .into_error());
        }
        return serde_json::to_string(&VerificationReport {
            payload,
            graphics: report,
        })
        .map_err(|e| e.to_string());
    }
    if mode != "prepare" {
        return Err("unknown guest operation".into());
    }
    let manifest = read_preparation_manifest(
        fs::File::open(bundle.join("manifest.json")).map_err(|e| e.to_string())?,
    )?;
    if manifest.schema != 1 || manifest.files.is_empty() || manifest.files.len() > 65536 {
        return Err("invalid preparation manifest".into());
    }
    let digest = manifest.digest()?;
    if bundle.file_name().and_then(|n| n.to_str()) != Some(&digest) {
        return Err("preparation bundle identity mismatch".into());
    }
    let windows = windows_paths::windows_directory().map_err(|e| e.to_string())?;
    let _lock = preparation_lock(root)?;
    progress(crate::workflow::PreparationProgress::CheckingCopiedFiles)?;
    let mut destinations = std::collections::BTreeSet::new();
    for (index, file) in manifest.files.iter().enumerate() {
        payload::validate_relative(&file.destination)?;
        if !destinations.insert(file.destination.to_ascii_lowercase()) {
            return Err("duplicate guest destination".into());
        }
        let source = bundle.join(index.to_string());
        payload::no_reparse(&source)?;
        if fs::metadata(&source).map_err(|e| e.to_string())?.len() != file.bytes
            || payload::hash_file(&source)? != file.sha256
        {
            return Err("transferred preparation source changed".into());
        }
    }
    progress(crate::workflow::PreparationProgress::InstallingFiles)?;
    for (index, file) in manifest.files.iter().enumerate() {
        let target = windows.join(&file.destination);
        ensure_directory(target.parent().ok_or("destination parent missing")?)?;
        if target.exists() {
            crate::security::verify_system(&target)?;
            if fs::metadata(&target).map_err(|e| e.to_string())?.len() == file.bytes
                && payload::hash_file(&target)? == file.sha256
            {
                continue;
            }
        }
        let partial = target.with_extension("hyper-gpu-partial");
        if partial.exists() {
            crate::security::verify_system(&partial)?;
            fs::remove_file(&partial).map_err(|e| e.to_string())?;
        }
        let mut input =
            fs::File::open(bundle.join(index.to_string())).map_err(|e| e.to_string())?;
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&partial)
            .map_err(|e| e.to_string())?;
        crate::diagnostics::run(
            "guest-file-copy",
            &format!(
                "index={index} destination={} bytes={}",
                file.destination, file.bytes
            ),
            || {
                std::io::copy(&mut input, &mut output)
                    .and_then(|_| output.sync_all())
                    .map_err(|e| e.to_string())
            },
        )?;
        drop(output);
        // Runtime files retain vetted Windows-parent read/execute inheritance so
        // ordinary guest users can load them; only private state uses runner ACLs.
        crate::security::set_administrator_owner(&partial)?;
        crate::security::verify_system(&partial)?;
        if payload::hash_file(&partial)? != file.sha256 {
            return Err("guest copy hash mismatch".into());
        }
        replace_file(&partial, &target)?;
        if payload::hash_file(&target)? != file.sha256 {
            return Err("published guest file verification failed".into());
        }
    }
    progress(crate::workflow::PreparationProgress::CheckingInstalledFiles)?;
    for file in &manifest.files {
        let target = windows.join(&file.destination);
        payload::no_reparse(&target)?;
        crate::security::verify_system(&target)?;
        if fs::metadata(&target).map_err(|e| e.to_string())?.len() != file.bytes
            || payload::hash_file(&target)? != file.sha256
        {
            return Err("installed driver file verification failed".into());
        }
    }
    let receipt = Receipt {
        digest,
        machine_guid: machine_guid()?,
        files: manifest.files.len(),
    };
    let text = serde_json::to_string(&receipt).map_err(|e| e.to_string())?;
    let receipt_path = root.join("prepared.json");
    let partial = root.join("prepared.partial");
    if partial.exists() {
        crate::security::verify(&partial)?;
        fs::remove_file(&partial).map_err(|e| e.to_string())?;
    }
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&partial)
        .map_err(|e| e.to_string())?;
    use std::io::Write;
    output
        .write_all(text.as_bytes())
        .and_then(|_| output.sync_all())
        .map_err(|e| e.to_string())?;
    drop(output);
    crate::runner::protect(&partial, "S-1-5-32-544")?;
    if receipt_path.exists() {
        crate::security::verify(&receipt_path)?;
    }
    replace_file(&partial, &receipt_path)?;
    Ok(text)
}
fn ensure_directory(path: &Path) -> Result<(), String> {
    if !path.exists() {
        ensure_directory(path.parent().ok_or("destination volume missing")?)?;
        fs::create_dir(path).map_err(|e| e.to_string())?;
        crate::security::set_administrator_owner(path)?;
    }
    crate::security::verify_system(path)
}
#[allow(unsafe_code)]
pub(crate) fn replace_file(from: &Path, to: &Path) -> Result<(), String> {
    use windows::{
        Win32::Storage::FileSystem::{
            MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
        },
        core::PCWSTR,
    };
    let from: Vec<_> = from
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let to: Vec<_> = to.to_string_lossy().encode_utf16().chain(Some(0)).collect();
    // SAFETY: terminated validated local paths; atomic replacement on the same volume.
    unsafe {
        MoveFileExW(
            PCWSTR(from.as_ptr()),
            PCWSTR(to.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    }
    .map_err(|e| e.to_string())
}
fn preparation_lock(root: &Path) -> Result<fs::File, String> {
    use std::os::windows::fs::OpenOptionsExt;
    let path = root.join("preparation.lock");
    if path.exists() {
        crate::security::verify(&path)?;
    }
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .share_mode(0)
        .open(&path)
        .map_err(|_| "another guest preparation or verification is active")?;
    crate::runner::protect(&path, "S-1-5-32-544")?;
    Ok(lock)
}

fn machine_guid() -> Result<String, String> {
    let _a = crate::windows_com::Apartment::initialize().map_err(|e| e.to_string())?;
    let cim = crate::windows_com::connect(r"ROOT\cimv2").map_err(|e| e.to_string())?;
    let rows = crate::windows_com::query(
        &cim,
        "SELECT UUID FROM Win32_ComputerSystemProduct",
        &["UUID"],
        std::time::Instant::now() + Duration::from_secs(15),
    )
    .map_err(|e| e.to_string())?;
    if rows.len() != 1 {
        return Err("guest machine identity is ambiguous".into());
    }
    rows[0]
        .get("UUID")
        .cloned()
        .ok_or("guest machine identity missing".into())
}
fn health_failure(
    cause: crate::reporting::FailureCause,
    code: Option<crate::reporting::FailureCode>,
) -> String {
    crate::reporting::ComponentFailure {
        stage: None,
        component: crate::reporting::Component::GuestGraphics,
        cause,
        code,
    }
    .into_error()
}
fn health_query_failure(error: crate::windows_com::DriverDiscoveryError) -> String {
    use crate::reporting::{FailureCause, FailureCode};
    use crate::windows_com::DriverDiscoveryError;
    let (cause, code) = match error {
        DriverDiscoveryError::Native { source, .. } => (
            if matches!(source.code().0 as u32, 0x80070005 | 0x80041003) {
                FailureCause::AccessDenied
            } else {
                FailureCause::Unavailable
            },
            Some(FailureCode::HResult(source.code().0 as u32)),
        ),
        DriverDiscoveryError::Timeout => (FailureCause::TimedOut, None),
        _ => (FailureCause::InvalidReply, None),
    };
    health_failure(cause, code)
}
fn health_status(status: Option<u32>) -> Result<bool, String> {
    use crate::reporting::{FailureCause, FailureCode};
    match status {
        Some(0) => Ok(true),
        Some(code) => Err(health_failure(
            FailureCause::DeviceUnhealthy,
            Some(FailureCode::DeviceStatus(code)),
        )),
        None => Err(health_failure(FailureCause::TimedOut, None)),
    }
}
fn health() -> Result<(), String> {
    let _a = crate::windows_com::Apartment::initialize().map_err(health_query_failure)?;
    let cim = crate::windows_com::connect(r"ROOT\cimv2").map_err(health_query_failure)?;
    let deadline = std::time::Instant::now() + Duration::from_secs(120);
    loop {
        let mut last_status = None;
        let rows = crate::windows_com::query(
            &cim,
            "SELECT DeviceID FROM Win32_PnPSignedDriver WHERE InfName='vrd.inf' AND DeviceClass='DISPLAY'",
            &["DeviceID"],
            deadline,
        ).map_err(health_query_failure)?;
        if rows.len() == 1 {
            let id = rows[0].get("DeviceID").ok_or_else(|| {
                health_failure(crate::reporting::FailureCause::InvalidReply, None)
            })?;
            let pnp = crate::windows_com::query(
                &cim,
                &format!(
                    "SELECT ConfigManagerErrorCode FROM Win32_PnPEntity WHERE DeviceID='{}'",
                    crate::windows_com::quoted(id)
                ),
                &["ConfigManagerErrorCode"],
                deadline,
            )
            .map_err(health_query_failure)?;
            if pnp.len() == 1 {
                let status = pnp[0]
                    .get("ConfigManagerErrorCode")
                    .and_then(|value| value.parse::<u32>().ok());
                last_status = status;
                if status == Some(0) {
                    return Ok(());
                }
                if status.is_none() {
                    return Err(health_failure(
                        crate::reporting::FailureCause::InvalidReply,
                        None,
                    ));
                }
            }
        }
        if std::time::Instant::now() >= deadline {
            return health_status(last_status).map(|_| ());
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actual_transport_checks_current_os_before_bootstrap_without_metadata_or_disk_access() {
        // Run the production bridge with all guest/Hyper-V entrypoints replaced.
        // The second remoting invocation is stopped before executing bootstrap.
        let mocks = r#"
function Import-Module {}
function New-PSSession { 'isolated-test-session' }
function Remove-PSSession {}
function Get-CimInstance { $r.facts }
function Get-ItemPropertyValue { $r.kind }
function Invoke-Command {
    param($Session, [scriptblock]$ScriptBlock)
    if ($script:queried) {
        [Console]::Out.WriteLine('BOOTSTRAP_BOUNDARY')
        throw 'Test stops before bootstrap'
    }
    $script:queried = $true
    & $ScriptBlock
}
"#;
        for (version, build, product_type, kind, expected) in [
            ("10.0.26100", "26100", 1, "Client", None),
            (
                "10.0.19045",
                "19045",
                1,
                "Client",
                Some(crate::reporting::FailureCause::UnsupportedGuest),
            ),
            (
                "10.0.26100",
                "26100",
                3,
                "Server",
                Some(crate::reporting::FailureCause::UnsupportedGuest),
            ),
            (
                "10.0.26100",
                "26100",
                1,
                "WindowsPE",
                Some(crate::reporting::FailureCause::UnsupportedGuest),
            ),
            (
                "10.0.26100",
                "22631",
                1,
                "Client",
                Some(crate::reporting::FailureCause::Unavailable),
            ),
            (
                "10.0",
                "26100",
                1,
                "Client",
                Some(crate::reporting::FailureCause::Unavailable),
            ),
            (
                "10.0.26100",
                "26100",
                0,
                "Client",
                Some(crate::reporting::FailureCause::Unavailable),
            ),
        ] {
            let input = serde_json::to_vec(&serde_json::json!({
                "vm_id": "11111111-1111-1111-1111-111111111111",
                "username": "test-placeholder", "password": "test-placeholder",
                "mode": "prepare", "diagnostics": false,
                "windows_11_builds": crate::guest_compatibility::WINDOWS_11_BUILDS,
                "kind": kind,
                "facts": {"Version": version, "BuildNumber": build, "ProductType": product_type}
            }))
            .unwrap();
            let mut command = Command::new(windows_paths::windows_powershell_executable().unwrap());
            command.args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!("{mocks}\n{}", transport_script("unused")),
            ]);
            let result = process::bounded_process_with_limit(
                command,
                Duration::from_secs(20),
                65536,
                &input,
            )
            .unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(result.exit_code, Some(1));
            if let Some(expected) = expected {
                assert!(
                    !result.stdout.contains("BOOTSTRAP_BOUNDARY"),
                    "{version}/{kind}"
                );
                let reply: BridgeResult = serde_json::from_str(result.stdout.trim()).unwrap();
                let failure = crate::reporting::ComponentFailure::decode(&reply.output).unwrap();
                assert!(matches!(
                    failure.component,
                    crate::reporting::Component::GuestCompatibility
                ));
                assert_eq!(
                    std::mem::discriminant(&failure.cause),
                    std::mem::discriminant(&expected)
                );
            } else {
                assert!(
                    result.stdout.contains("BOOTSTRAP_BOUNDARY"),
                    "{}",
                    result.stdout
                );
            }
        }
    }

    #[test]
    fn preparation_manifest_read_is_bounded_before_parsing() {
        let source = br#"{"schema":1,"device_id":"device","driver_version":"version","catalogs":[],"files":[]}"#;
        let manifest = read_preparation_manifest(source.as_slice()).unwrap();
        assert_eq!(manifest.device_id, "device");
        let exact_limit = source
            .as_slice()
            .chain(std::io::repeat(b' ').take(MANIFEST_LIMIT - source.len() as u64));
        assert!(read_preparation_manifest(exact_limit).is_ok());
        let mut oversized = std::io::repeat(b' ').take(MANIFEST_LIMIT + 4096);
        assert_eq!(
            read_preparation_manifest(&mut oversized).unwrap_err(),
            "preparation manifest exceeds safety limit"
        );
        assert_eq!(
            oversized.limit(),
            4095,
            "stop after one byte beyond the limit"
        );
        assert!(read_preparation_manifest(b"invalid JSON".as_slice()).is_err());
    }

    #[test]
    fn device_health_preserves_status_and_query_failure_without_private_text() {
        use crate::reporting::{ComponentFailure, FailureCause, FailureCode};
        assert!(health_status(Some(0)).unwrap());
        let unhealthy = ComponentFailure::decode(&health_status(Some(43)).unwrap_err()).unwrap();
        assert!(matches!(unhealthy.cause, FailureCause::DeviceUnhealthy));
        assert!(matches!(
            unhealthy.code,
            Some(FailureCode::DeviceStatus(43))
        ));
        let missing = ComponentFailure::decode(&health_status(None).unwrap_err()).unwrap();
        assert!(matches!(missing.cause, FailureCause::TimedOut));
        let unavailable = health_query_failure(crate::windows_com::DriverDiscoveryError::Invalid(
            "private adapter output",
        ));
        assert!(!unavailable.contains("private adapter output"));
        assert!(matches!(
            ComponentFailure::decode(&unavailable).unwrap().cause,
            FailureCause::InvalidReply
        ));
        let denied = health_query_failure(crate::windows_com::native(
            "private operation context",
            windows::core::Error::from_hresult(windows::core::HRESULT(0x80070005u32 as i32)),
        ));
        let denied = ComponentFailure::decode(&denied).unwrap();
        assert!(matches!(denied.cause, FailureCause::AccessDenied));
        assert!(matches!(
            denied.code,
            Some(FailureCode::HResult(0x80070005))
        ));
        let timeout = health_query_failure(crate::windows_com::DriverDiscoveryError::Timeout);
        assert!(matches!(
            ComponentFailure::decode(&timeout).unwrap().cause,
            FailureCause::TimedOut
        ));
    }

    #[test]
    fn supervision_diagnostics_retain_typed_facts_without_adapter_text() {
        use crate::reporting::{ComponentFailure, FailureCause};
        let timeout =
            ComponentFailure::decode(&supervision_failure(process::BoundedProcessError::Timeout))
                .unwrap();
        assert!(matches!(timeout.cause, FailureCause::TimedOut));
        let error = supervision_failure(process::BoundedProcessError::Execution(
            "private adapter output".into(),
        ));
        assert!(!error.contains("private adapter output"));
        assert!(matches!(
            ComponentFailure::decode(&error).unwrap().cause,
            FailureCause::Unavailable
        ));
    }

    #[test]
    fn fixed_transport_refuses_malformed_input_before_guest_effects() {
        let module = windows_paths::windows_directory()
            .unwrap()
            .join("System32/WindowsPowerShell/v1.0/Modules/Hyper-V")
            .to_string_lossy()
            .replace('\'', "''");
        let script = transport_script(&module);
        let mut command = Command::new(windows_paths::windows_powershell_executable().unwrap());
        command.args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &script,
        ]);
        let result = process::bounded_process_with_limit(
            command,
            Duration::from_secs(15),
            1024 * 1024,
            &vec![b'x'; 131072],
        )
        .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(result.exit_code, Some(1));
        assert_eq!(
            result.stderr.trim(),
            "Fixed guest transport failed; reconcile before retry."
        );
    }

    #[test]
    fn stream_sender_uses_four_mib_chunks_exact_tails_and_stops_on_bad_ack() {
        // Execute only the copy function against local temporary files. The
        // substitute never imports Hyper-V, opens a guest session or uses credentials.
        let script = transport_script("unused");
        let definitions = script.split_once("\ntry {").unwrap().0;
        let fixture = r#"
function Invoke-Command {
    param($Session, [object[]]$ArgumentList, [scriptblock]$ScriptBlock)
    $copying = $ArgumentList.Count -eq 2
    if ($copying) {
        if ($ArgumentList[0].Length -ne $ArgumentList[1]) { throw 'Padded chunk' }
        if ($ArgumentList[0].Length -gt 4194304) { throw 'Oversized chunk' }
        $script:lengths.Add([int]$ArgumentList[1])
    }
    $result = & $ScriptBlock @ArgumentList
    if ($copying -and $script:badAck) { return ([long]$result + 1) }
    return $result
}
$root = Join-Path ([IO.Path]::GetTempPath()) ('hypergpu-chunks-' + [guid]::NewGuid())
[void][IO.Directory]::CreateDirectory($root)
try {
    foreach ($size in @(0, 37, 4194304, 4194341, 8388645)) {
        $source = Join-Path $root 'source'
        $destination = Join-Path $root 'destination'
        $file = [IO.File]::Open($source, 'Create', 'Write')
        try {
            $file.SetLength($size)
            if ($size -gt 0) {
                $file.WriteByte(165)
                $file.Position = $size - 1
                $file.WriteByte(90)
            }
        } finally { $file.Dispose() }
        $script:lengths = [Collections.Generic.List[int]]::new()
        $script:badAck = $false
        Copy-Logged $source $destination $size
        if ((Get-FileHash $source).Hash -ne (Get-FileHash $destination).Hash) { throw 'Copy mismatch' }
        if ($script:lengths.Count -ne [Math]::Ceiling($size / 4194304.0)) { throw 'Wrong chunk count' }
        if ($size -gt 0) {
            $tail = $size % 4194304
            if ($tail -eq 0) { $tail = 4194304 }
            if ($script:lengths[$script:lengths.Count - 1] -ne $tail) { throw 'Wrong tail size' }
        }
    }
    $script:lengths.Clear()
    $script:badAck = $true
    $rejected = $false
    try { Copy-Logged $source $destination $size } catch {
        if ($_.Exception.Message -ne 'Transfer acknowledgement mismatch') { throw }
        $rejected = $true
    }
    if (!$rejected -or $script:lengths.Count -ne 1) { throw 'Bad acknowledgement did not stop transfer' }
} finally {
    if ($global:hyperGpuTransfer) { $global:hyperGpuTransfer.Dispose() }
    [IO.File]::Delete((Join-Path $root 'source'))
    [IO.File]::Delete((Join-Path $root 'destination'))
    [IO.Directory]::Delete($root)
}
Write-Output 'Chunk checks passed'
"#;
        let mut command = Command::new(windows_paths::windows_powershell_executable().unwrap());
        command.args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!("{definitions}\n{fixture}"),
        ]);
        let result =
            process::bounded_process_with_limit(command, Duration::from_secs(30), 1024 * 1024, &[])
                .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(result.exit_code, Some(0), "{}", result.stderr);
        assert_eq!(result.stdout.trim(), "Chunk checks passed");
    }
}
