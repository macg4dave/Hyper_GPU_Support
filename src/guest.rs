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
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

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
}
#[derive(Serialize)]
struct Artifact {
    name: String,
    hash: String,
}
#[derive(Deserialize)]
struct BridgeResult {
    output: String,
}
fn bridge(
    t: &Target,
    credential: &Credential,
    mode: &str,
    manifest: Option<&Manifest>,
    gpu: Option<&Gpu>,
) -> Result<String, String> {
    let install = runner::install_directory()?;
    let artifacts = ["hyper-gpu-guest.exe", "d3d11-probe.exe"]
        .into_iter()
        .map(|name| {
            Ok(Artifact {
                name: name.into(),
                hash: payload::hash_file(&install.join(name))?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let request = BridgeRequest {
        vm_id: &t.vm_id,
        username: &credential.username,
        password: &credential.password,
        install,
        mode,
        manifest,
        digest: manifest
            .map(Manifest::digest)
            .transpose()?
            .unwrap_or_else(|| "verify".into()),
        vendor: gpu.map_or(0, |g| g.vendor),
        device: gpu.map_or(0, |g| g.device),
        artifacts,
    };
    let bytes = zeroize::Zeroizing::new(serde_json::to_vec(&request).map_err(|e| e.to_string())?);
    let windows = windows_paths::windows_directory().map_err(|e| e.to_string())?;
    let module = windows.join("System32/WindowsPowerShell/v1.0/Modules");
    // Inbox modules may use versioned subdirectories. Importing the trusted
    // module directory lets PowerShell's loader select its installed manifest.
    crate::security::verify_system(&module.join("Hyper-V"))?;
    let module_literal = module.join("Hyper-V").to_string_lossy().replace('\'', "''");
    let script = include_str!("guest_transport.ps1").replace("__HYPERV_MODULE__", &module_literal);
    let mut command =
        Command::new(windows_paths::windows_powershell_executable().map_err(|e| e.to_string())?);
    command.env("PSModulePath", &module).args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        &script,
    ]);
    let result = process::bounded_process_with_limit(
        command,
        Duration::from_secs(3600),
        1024 * 1024,
        &bytes,
    )
    .map_err(|error| {
        format!("guest transport supervision failed: {error}; preparation may be partial")
    })?;
    if result.exit_code != Some(0) {
        return Err("guest transport/worker failed; reconcile the selected VM before retry".into());
    }
    let reply: BridgeResult = serde_json::from_str(result.stdout.trim())
        .map_err(|_| "malformed guest transport result")?;
    Ok(reply.output)
}
/// Copy and independently verify the current complete payload.
pub fn prepare(t: &Target, manifest: &Manifest, credential: &Credential) -> Result<String, String> {
    let text = bridge(t, credential, "prepare", Some(manifest), None)?;
    let receipt: Receipt =
        serde_json::from_str(text.trim()).map_err(|_| "invalid guest preparation receipt")?;
    let digest = manifest.digest()?;
    if receipt.digest != digest
        || receipt.files != manifest.files.len()
        || receipt.machine_guid.is_empty()
    {
        return Err("guest preparation receipt mismatch".into());
    }
    Ok(digest)
}
/// Require PnP Code 0 and checked D3D11 rendering on the selected hardware.
pub fn verify(t: &Target, gpu: &Gpu, credential: &Credential) -> Result<(), String> {
    let text = bridge(t, credential, "verify", None, Some(gpu))?;
    let report = crate::probe::parse_success_report(text.trim(), "d3d11-offscreen")
        .map_err(|e| e.to_string())?;
    if report.adapter.vendor_id != gpu.vendor
        || report.adapter.device_id != gpu.device
        || !report.adapter.paravirtualized
    {
        return Err("guest graphics probe selected the wrong GPU".into());
    }
    Ok(())
}
/// Fixed guest worker entry point. Sources/destinations derive only from a protected transport bundle.
pub fn worker(mode: &str, bundle: &Path, vendor: u32, device: u32) -> Result<String, String> {
    let _deadline = process::WorkerDeadline::start(Duration::from_secs(3500));
    if !process::is_elevated()? {
        return Err("guest worker requires administrator rights".into());
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
        health()?;
        let mut command = Command::new(root.join("d3d11-probe.exe"));
        command.args([vendor.to_string(), device.to_string()]);
        let out = process::bounded_process(command, Duration::from_secs(60))?;
        if out.exit_code != Some(0) {
            return Err("hardware rendering failed".into());
        }
        let report = crate::probe::parse_success_report(out.stdout.trim(), "d3d11-offscreen")
            .map_err(|e| e.to_string())?;
        if report.adapter.vendor_id != vendor
            || report.adapter.device_id != device
            || !report.adapter.paravirtualized
        {
            return Err("wrong guest GPU selected".into());
        }
        return Ok(out.stdout);
    }
    if mode != "prepare" {
        return Err("unknown guest operation".into());
    }
    let input = fs::read(bundle.join("manifest.json")).map_err(|e| e.to_string())?;
    if input.len() > 32 * 1024 * 1024 {
        return Err("preparation manifest exceeds safety limit".into());
    }
    let manifest: Manifest = serde_json::from_slice(&input).map_err(|e| e.to_string())?;
    if manifest.schema != 1 || manifest.files.is_empty() || manifest.files.len() > 65536 {
        return Err("invalid preparation manifest".into());
    }
    let digest = manifest.digest()?;
    if bundle.file_name().and_then(|n| n.to_str()) != Some(&digest) {
        return Err("preparation bundle identity mismatch".into());
    }
    let windows = windows_paths::windows_directory().map_err(|e| e.to_string())?;
    let lock_path = root.join("preparation.lock");
    if lock_path.exists() {
        crate::security::verify(&lock_path)?;
    }
    use std::os::windows::fs::OpenOptionsExt;
    let _lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .share_mode(0)
        .open(&lock_path)
        .map_err(|_| "another guest preparation is active")?;
    crate::runner::protect(&lock_path, "S-1-5-32-544")?;
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
        std::io::copy(&mut input, &mut output)
            .and_then(|_| output.sync_all())
            .map_err(|e| e.to_string())?;
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
fn health() -> Result<(), String> {
    let _a = crate::windows_com::Apartment::initialize().map_err(|e| e.to_string())?;
    let cim = crate::windows_com::connect(r"ROOT\cimv2").map_err(|e| e.to_string())?;
    let deadline = std::time::Instant::now() + Duration::from_secs(120);
    loop {
        let rows=crate::windows_com::query(&cim,"SELECT DeviceID FROM Win32_PnPSignedDriver WHERE InfName='vrd.inf' AND DeviceClass='DISPLAY'",&["DeviceID"],deadline).map_err(|e|e.to_string())?;
        if rows.len() == 1 {
            let id = rows[0].get("DeviceID").ok_or("guest device ID missing")?;
            let pnp = crate::windows_com::query(
                &cim,
                &format!(
                    "SELECT ConfigManagerErrorCode FROM Win32_PnPEntity WHERE DeviceID='{}'",
                    crate::windows_com::quoted(id)
                ),
                &["ConfigManagerErrorCode"],
                deadline,
            )
            .map_err(|e| e.to_string())?;
            if pnp.len() == 1
                && pnp[0]
                    .get("ConfigManagerErrorCode")
                    .is_some_and(|s| s == "0")
            {
                return Ok(());
            }
        }
        if std::time::Instant::now() >= deadline {
            return Err("guest GPU did not reach Code 0".into());
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_transport_refuses_malformed_input_before_guest_effects() {
        let module = windows_paths::windows_directory()
            .unwrap()
            .join("System32/WindowsPowerShell/v1.0/Modules/Hyper-V")
            .to_string_lossy()
            .replace('\'', "''");
        let script = include_str!("guest_transport.ps1").replace("__HYPERV_MODULE__", &module);
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
}
