//! Bounded Windows PowerShell Direct adapter for the pinned disposable guest.

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{LazyLock, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::config::{DriverManifestConfiguration, GuestConfiguration, SlotConfiguration};
use crate::guest::{GuestCredential, GuestError, GuestTransfer, TransferReceipt, TransferRequest};
use crate::staging::{
    DriverPackageFile, DriverPackageManifest, GuestPackageStager, StageReceipt, StageStatus,
    StagingError,
};

const OUTPUT_LIMIT: usize = 64 * 1024;
const REQUEST_LIMIT: usize = 16 * 1024;
const STAGING_REQUEST_LIMIT: usize = 256 * 1024;
/// Fixed PowerShell Direct transport bound to validated project configuration.
#[derive(Debug, Clone)]
pub struct WindowsGuestTransfer {
    slot: SlotConfiguration,
    guest: GuestConfiguration,
}

impl WindowsGuestTransfer {
    /// Bind the adapter to the already validated VM and guest identities.
    #[must_use]
    pub fn new(slot: SlotConfiguration, guest: GuestConfiguration) -> Self {
        Self { slot, guest }
    }
}

impl GuestTransfer for WindowsGuestTransfer {
    fn transfer(
        &self,
        credential: &GuestCredential,
        request: &TransferRequest,
    ) -> Result<TransferReceipt, GuestError> {
        let staging_root = self.guest.staging_root.to_string_lossy();
        let child_path = self.slot.child_path.to_string_lossy();
        let parent_path = self.slot.parent_path.to_string_lossy();
        let probe = WireRequest {
            mode: "probe",
            vm_id: &self.slot.vm_id,
            vm_name: &self.slot.vm_name,
            child_path: &child_path,
            parent_path: &parent_path,
            computer_name: &self.guest.computer_name,
            machine_guid: &self.guest.machine_guid,
            staging_root: &staging_root,
            source: "",
            destination: "",
            sha256: "",
            username: credential.username(),
            password: credential.password(),
        };
        parse_probe(&run_phase(
            &self.guest.powershell_path,
            &probe,
            self.guest.session_timeout,
        )?)?;

        let source = request.source().to_string_lossy();
        let destination = request.destination().to_string_lossy();
        let transfer = WireRequest {
            mode: "copy",
            vm_id: &self.slot.vm_id,
            vm_name: &self.slot.vm_name,
            child_path: &child_path,
            parent_path: &parent_path,
            computer_name: &self.guest.computer_name,
            machine_guid: &self.guest.machine_guid,
            staging_root: &staging_root,
            source: &source,
            destination: &destination,
            sha256: request.sha256(),
            username: credential.username(),
            password: credential.password(),
        };
        parse_transfer(&run_phase(
            &self.guest.powershell_path,
            &transfer,
            self.guest.transfer_timeout,
        )?)
    }
}

impl GuestPackageStager for WindowsGuestTransfer {
    fn stage(
        &self,
        credential: &GuestCredential,
        configuration: &DriverManifestConfiguration,
        manifest: &DriverPackageManifest,
        manifest_sha256: &str,
    ) -> Result<StageReceipt, StagingError> {
        let staging_root = self.guest.staging_root.to_string_lossy();
        let child_path = self.slot.child_path.to_string_lossy();
        let parent_path = self.slot.parent_path.to_string_lossy();
        let source_root = configuration.source_path.to_string_lossy();
        let request = StageWireRequest {
            vm_id: &self.slot.vm_id,
            vm_name: &self.slot.vm_name,
            child_path: &child_path,
            parent_path: &parent_path,
            computer_name: &self.guest.computer_name,
            machine_guid: &self.guest.machine_guid,
            staging_root: &staging_root,
            source_root: &source_root,
            gpu_interface: &self.slot.gpu_interface,
            gpu_driver_version: &configuration.driver_version,
            host_build: &configuration.host_build,
            manifest_id: &manifest.id,
            manifest_sha256,
            package_directory: &manifest.package_directory,
            signer_thumbprint: &configuration.signer_thumbprint,
            signature_files: &configuration.signature_files,
            allowed_pending_delete_sources: &configuration.allowed_pending_delete_sources,
            files: &manifest.files,
            byte_count: manifest.byte_count,
            username: credential.username(),
            password: credential.password(),
        };
        let payload = Zeroizing::new(
            serde_json::to_vec(&request).map_err(|_| StagingError::InvalidProtocol)?,
        );
        if payload.len() > STAGING_REQUEST_LIMIT {
            return Err(StagingError::InvalidProtocol);
        }
        let output = run_script(
            &self.guest.powershell_path,
            STAGING_SCRIPT.as_str(),
            payload,
            self.guest.staging_timeout,
        )
        .map_err(map_staging_error)?;
        parse_stage_receipt(&output)
    }
}

#[derive(Serialize)]
struct WireRequest<'a> {
    mode: &'a str,
    vm_id: &'a str,
    vm_name: &'a str,
    child_path: &'a str,
    parent_path: &'a str,
    computer_name: &'a str,
    machine_guid: &'a str,
    staging_root: &'a str,
    source: &'a str,
    destination: &'a str,
    sha256: &'a str,
    username: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct StageWireRequest<'a> {
    vm_id: &'a str,
    vm_name: &'a str,
    child_path: &'a str,
    parent_path: &'a str,
    computer_name: &'a str,
    machine_guid: &'a str,
    staging_root: &'a str,
    source_root: &'a str,
    gpu_interface: &'a str,
    gpu_driver_version: &'a str,
    host_build: &'a str,
    manifest_id: &'a str,
    manifest_sha256: &'a str,
    package_directory: &'a str,
    signer_thumbprint: &'a str,
    signature_files: &'a [String],
    allowed_pending_delete_sources: &'a [String],
    files: &'a [DriverPackageFile],
    byte_count: u64,
    username: &'a str,
    password: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireResponse {
    status: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    phase: String,
    #[serde(default)]
    vm_id: String,
    #[serde(default)]
    computer_name: String,
    #[serde(default)]
    machine_guid: String,
    #[serde(default)]
    destination: String,
    #[serde(default)]
    sha256: String,
    #[serde(default)]
    bytes: u64,
    #[serde(default)]
    manifest_id: String,
    #[serde(default)]
    manifest_sha256: String,
    qualified_host_build: Option<String>,
    measured_host_build: Option<String>,
    measured_guest_build: Option<String>,
    pending_delete_count: Option<u32>,
    pending_delete_sources: Option<Vec<String>>,
    #[serde(default)]
    package_destination: String,
    #[serde(default)]
    cuda_alias: String,
    #[serde(default)]
    alias_method: String,
    #[serde(default)]
    files: u32,
}

fn run_phase(
    powershell_path: &Path,
    request: &WireRequest<'_>,
    timeout: Duration,
) -> Result<String, GuestError> {
    let metadata = powershell_path
        .symlink_metadata()
        .map_err(|_| GuestError::IntegrationUnavailable)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(GuestError::IntegrationUnavailable);
    }
    let payload =
        Zeroizing::new(serde_json::to_vec(request).map_err(|_| GuestError::InvalidProtocol)?);
    if payload.len() > REQUEST_LIMIT {
        return Err(GuestError::InvalidProtocol);
    }
    run_script(powershell_path, SCRIPT.as_str(), payload, timeout)
}

fn run_script(
    powershell_path: &Path,
    script: &str,
    payload: Zeroizing<Vec<u8>>,
    timeout: Duration,
) -> Result<String, GuestError> {
    let module_path = powershell_path
        .parent()
        .ok_or(GuestError::IntegrationUnavailable)?
        .join("Modules");
    run_script_with_command(
        Command::new(powershell_path),
        &module_path,
        script,
        payload,
        timeout,
    )
}

fn run_script_with_command(
    mut command: Command,
    module_path: &Path,
    script: &str,
    payload: Zeroizing<Vec<u8>>,
    timeout: Duration,
) -> Result<String, GuestError> {
    let deadline = Instant::now() + timeout;
    if script.encode_utf16().count() > 30_000 {
        return Err(GuestError::InvalidProtocol);
    }
    configure_powershell_command(&mut command, module_path);
    let mut child = command
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| GuestError::IntegrationUnavailable)?;
    let Some(mut stdin) = child.stdin.take() else {
        terminate(&mut child);
        return Err(GuestError::AdapterFailed);
    };
    let Some(stdout) = child.stdout.take() else {
        terminate(&mut child);
        return Err(GuestError::AdapterFailed);
    };
    let Some(stderr) = child.stderr.take() else {
        terminate(&mut child);
        return Err(GuestError::AdapterFailed);
    };
    let stdout_reader = thread::spawn(move || read_bounded(stdout));
    let stderr_reader = thread::spawn(move || read_bounded(stderr));
    let (writer_sender, writer_receiver) = mpsc::sync_channel(1);
    let writer = thread::spawn(move || {
        let result = stdin.write_all(&payload).and_then(|()| stdin.flush());
        drop(stdin);
        let _ = writer_sender.send(result);
    });
    let mut writer_result = None;
    let status = loop {
        if writer_result.is_none() {
            match writer_receiver.try_recv() {
                Ok(result) => writer_result = Some(result),
                Err(mpsc::TryRecvError::Disconnected) => {
                    terminate(&mut child);
                    finish_threads(writer, stdout_reader, stderr_reader);
                    return Err(GuestError::AdapterFailed);
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if matches!(writer_result, Some(Err(_))) {
            terminate(&mut child);
            finish_threads(writer, stdout_reader, stderr_reader);
            return Err(GuestError::AdapterFailed);
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
            Ok(None) => {
                terminate(&mut child);
                finish_threads(writer, stdout_reader, stderr_reader);
                return Err(GuestError::Interrupted);
            }
            Err(_) => {
                terminate(&mut child);
                finish_threads(writer, stdout_reader, stderr_reader);
                return Err(GuestError::AdapterFailed);
            }
        }
    };
    if writer.join().is_err()
        || writer_result.map_or_else(|| writer_receiver.recv().is_err(), |result| result.is_err())
    {
        let _ = stdout_reader.join();
        let _ = stderr_reader.join();
        return Err(GuestError::AdapterFailed);
    }
    let stdout = stdout_reader
        .join()
        .map_err(|_| GuestError::AdapterFailed)??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| GuestError::AdapterFailed)??;
    if stdout.truncated || stderr.truncated {
        return Err(GuestError::InvalidProtocol);
    }
    let output = String::from_utf8(stdout.bytes).map_err(|_| GuestError::InvalidProtocol)?;
    if !status.success() {
        match serde_json::from_str::<WireResponse>(output.trim()) {
            Ok(response) if response.status == "error" => {}
            _ => return Err(GuestError::AdapterFailed),
        }
    }
    Ok(output)
}

fn configure_powershell_command(command: &mut Command, module_path: &Path) {
    // PowerShell rebuilds its default module path when the variable is absent, so
    // remove caller-controlled discovery by replacing it with the protected system
    // module root. The fixed script separately verifies every Hyper-V command's
    // loaded module path before it reads the credential-bearing request.
    command.env("PSModulePath", module_path);
}

fn terminate(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn finish_threads(
    writer: thread::JoinHandle<()>,
    stdout_reader: thread::JoinHandle<Result<BoundedBytes, GuestError>>,
    stderr_reader: thread::JoinHandle<Result<BoundedBytes, GuestError>>,
) {
    let _ = writer.join();
    let _ = stdout_reader.join();
    let _ = stderr_reader.join();
}

fn parse_probe(output: &str) -> Result<(), GuestError> {
    let response = parse_response(output)?;
    if response.status == "ok" && response.category == "ready" {
        Ok(())
    } else {
        Err(category(&response.category))
    }
}

fn parse_transfer(output: &str) -> Result<TransferReceipt, GuestError> {
    let response = parse_response(output)?;
    if response.status != "ok" || response.category != "transferred" {
        return Err(category(&response.category));
    }
    Ok(TransferReceipt {
        vm_id: response.vm_id,
        computer_name: response.computer_name,
        machine_guid: response.machine_guid,
        destination: response.destination.into(),
        sha256: response.sha256,
        bytes: response.bytes,
    })
}

fn parse_response(output: &str) -> Result<WireResponse, GuestError> {
    serde_json::from_str(output.trim()).map_err(|_| GuestError::InvalidProtocol)
}

fn category(value: &str) -> GuestError {
    match value {
        "credential-denied" => GuestError::CredentialDenied,
        "integration-unavailable" => GuestError::IntegrationUnavailable,
        "unsafe-path" => GuestError::UnsafePath,
        "verification-failed" => GuestError::VerificationFailed,
        "uncertain-state" => GuestError::Interrupted,
        _ => GuestError::AdapterFailed,
    }
}

fn map_staging_error(error: GuestError) -> StagingError {
    match error {
        GuestError::Interrupted | GuestError::AdapterFailed => StagingError::GuestStateUncertain,
        GuestError::CredentialDenied | GuestError::IntegrationUnavailable => {
            StagingError::GuestUnavailable
        }
        GuestError::InvalidProtocol => StagingError::GuestStateUncertain,
        _ => StagingError::VerificationFailed,
    }
}

fn parse_stage_receipt(output: &str) -> Result<StageReceipt, StagingError> {
    let response: WireResponse =
        serde_json::from_str(output.trim()).map_err(|_| StagingError::GuestStateUncertain)?;
    if response.status != "ok" {
        return Err(match response.category.as_str() {
            "invalid-signature" => StagingError::InvalidSignature,
            "credential-denied" | "integration-unavailable" => StagingError::GuestUnavailable,
            "uncertain-state" => StagingError::GuestStateUncertain,
            "verification-failed" => StagingError::VerificationFailed,
            "preflight-failed" if valid_preflight_phase(&response.phase) => {
                StagingError::GuestPreflightFailed {
                    phase: response.phase,
                }
            }
            _ => StagingError::GuestStateUncertain,
        });
    }
    let status = match response.category.as_str() {
        "applied" => StageStatus::Applied,
        "already-applied" => StageStatus::AlreadyApplied,
        _ => return Err(StagingError::GuestStateUncertain),
    };
    let qualified_host_build = response
        .qualified_host_build
        .ok_or(StagingError::GuestStateUncertain)?;
    let measured_host_build = response
        .measured_host_build
        .ok_or(StagingError::GuestStateUncertain)?;
    let measured_guest_build = response
        .measured_guest_build
        .ok_or(StagingError::GuestStateUncertain)?;
    let pending_delete_count = response
        .pending_delete_count
        .ok_or(StagingError::GuestStateUncertain)?;
    let pending_delete_sources = response
        .pending_delete_sources
        .ok_or(StagingError::GuestStateUncertain)?;
    Ok(StageReceipt {
        status,
        vm_id: response.vm_id,
        computer_name: response.computer_name,
        machine_guid: response.machine_guid,
        manifest_id: response.manifest_id,
        manifest_sha256: response.manifest_sha256,
        qualified_host_build,
        measured_host_build,
        measured_guest_build,
        pending_delete_count,
        pending_delete_sources,
        package_destination: response.package_destination.into(),
        cuda_alias: response.cuda_alias.into(),
        alias_method: response.alias_method,
        files: response.files,
        bytes: response.bytes,
    })
}

fn valid_preflight_phase(phase: &str) -> bool {
    matches!(
        phase,
        "request"
            | "host-target"
            | "host-gpu"
            | "host-servicing"
            | "source-signatures"
            | "guest-session"
            | "guest-identity"
            | "guest-paths"
    )
}

struct BoundedBytes {
    bytes: Vec<u8>,
    truncated: bool,
}

fn read_bounded(mut reader: impl Read) -> Result<BoundedBytes, GuestError> {
    let mut bytes = Vec::with_capacity(4096);
    let mut truncated = false;
    let mut buffer = [0_u8; 4096];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|_| GuestError::AdapterFailed)?;
        if count == 0 {
            break;
        }
        let remaining = OUTPUT_LIMIT.saturating_sub(bytes.len());
        bytes.extend_from_slice(&buffer[..count.min(remaining)]);
        truncated |= count > remaining;
    }
    Ok(BoundedBytes { bytes, truncated })
}

const ACL_VALIDATOR: &str = r#"
function Assert-ProtectedTree([string]$FullPath, [string]$Anchor = [Environment]::GetFolderPath('ProgramFiles')) {
    $full = [IO.Path]::GetFullPath($FullPath).TrimEnd('\')
    $anchorFull = [IO.Path]::GetFullPath($Anchor).TrimEnd('\')
    if ($full -ine $anchorFull -and -not $full.StartsWith($anchorFull + '\', [StringComparison]::OrdinalIgnoreCase)) {
        throw 'unsafe path'
    }
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    $trustedWriters = @(
        $currentUser,
        'S-1-3-0',
        'S-1-5-18',
        'S-1-5-32-544',
        'S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464'
    )
    $trustedOwners = @(
        $currentUser,
        'S-1-5-18',
        'S-1-5-32-544',
        'S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464'
    )
    $writeMask = [int](
        [Security.AccessControl.FileSystemRights]::WriteData -bor
        [Security.AccessControl.FileSystemRights]::AppendData -bor
        [Security.AccessControl.FileSystemRights]::WriteExtendedAttributes -bor
        [Security.AccessControl.FileSystemRights]::WriteAttributes -bor
        [Security.AccessControl.FileSystemRights]::DeleteSubdirectoriesAndFiles -bor
        [Security.AccessControl.FileSystemRights]::Delete -bor
        [Security.AccessControl.FileSystemRights]::ChangePermissions -bor
        [Security.AccessControl.FileSystemRights]::TakeOwnership)
    $current = $full
    while ($true) {
        $item = Get-Item -LiteralPath $current -Force
        if (-not $item.PSIsContainer -or (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0)) {
            throw 'unsafe path'
        }
        $acl = Get-Acl -LiteralPath $current
        $owner = $acl.GetOwner([Security.Principal.SecurityIdentifier]).Value
        if ($owner -notin $trustedOwners) { throw 'unsafe path' }
        foreach ($rule in $acl.Access) {
            if ($rule.AccessControlType -ne [Security.AccessControl.AccessControlType]::Allow) { continue }
            $rights = [int]$rule.FileSystemRights
            if (($rights -band $writeMask) -eq 0) { continue }
            $sid = $rule.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value
            if ($sid -notin $trustedWriters) { throw 'unsafe path' }
        }
        if ($current -ieq $anchorFull) { break }
        $current = Split-Path -Parent $current
        if ([string]::IsNullOrEmpty($current)) { throw 'unsafe path' }
    }
}
function Assert-ProtectedFile([string]$FullPath) {
    $item = Get-Item -LiteralPath $FullPath -Force
    if ($item.PSIsContainer -or (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0)) {
        throw 'unsafe path'
    }
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    $trustedWriters = @(
        $currentUser,
        'S-1-3-0',
        'S-1-5-18',
        'S-1-5-32-544',
        'S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464'
    )
    $trustedOwners = @(
        $currentUser,
        'S-1-5-18',
        'S-1-5-32-544',
        'S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464'
    )
    $writeMask = [int](
        [Security.AccessControl.FileSystemRights]::WriteData -bor
        [Security.AccessControl.FileSystemRights]::AppendData -bor
        [Security.AccessControl.FileSystemRights]::WriteExtendedAttributes -bor
        [Security.AccessControl.FileSystemRights]::WriteAttributes -bor
        [Security.AccessControl.FileSystemRights]::Delete -bor
        [Security.AccessControl.FileSystemRights]::ChangePermissions -bor
        [Security.AccessControl.FileSystemRights]::TakeOwnership)
    $acl = Get-Acl -LiteralPath $FullPath
    $owner = $acl.GetOwner([Security.Principal.SecurityIdentifier]).Value
    if ($owner -notin $trustedOwners) { throw 'unsafe path' }
    foreach ($rule in $acl.Access) {
        if ($rule.AccessControlType -ne [Security.AccessControl.AccessControlType]::Allow) { continue }
        $rights = [int]$rule.FileSystemRights
        if (($rights -band $writeMask) -eq 0) { continue }
        $sid = $rule.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value
        if ($sid -notin $trustedWriters) { throw 'unsafe path' }
    }
}
"#;

static SCRIPT: LazyLock<String> = LazyLock::new(|| {
    SCRIPT_TEMPLATE
        .replace("__HYPERV_BOOTSTRAP__", HYPERV_BOOTSTRAP)
        .replace("__ACL_VALIDATOR__", ACL_VALIDATOR)
});

static STAGING_SCRIPT: LazyLock<String> = LazyLock::new(|| {
    let staging_acl = ACL_VALIDATOR.replace("function Assert-", "function global:Assert-");
    STAGING_SCRIPT_TEMPLATE
        .replace("__HYPERV_BOOTSTRAP__", HYPERV_BOOTSTRAP)
        .replace("__STAGING_ACL_VALIDATOR__", &staging_acl)
        .replace("__PENDING_RENAME_VALIDATOR__", PENDING_RENAME_VALIDATOR)
        .replace("__ACL_VALIDATOR__", "")
});

const PENDING_RENAME_VALIDATOR: &str = r#"
function Get-ReviewedPendingDeletes($Entries, $Allowed) {
    $values = if ($null -eq $Entries) { @() } else { @($Entries) }
    $allowedValues = @($Allowed)
    if (($values.Count % 2) -ne 0) { throw 'host servicing state' }
    $reviewed = [Collections.Generic.List[string]]::new()
    for ($index = 0; $index -lt $values.Count; $index += 2) {
        $source = [string]$values[$index]
        $destination = [string]$values[$index + 1]
        if ([string]::IsNullOrWhiteSpace($source) -or
            -not [string]::IsNullOrEmpty($destination) -or
            $source -cnotin $allowedValues) { throw 'host servicing state' }
        $reviewed.Add($source)
    }
    [pscustomobject]@{ count = [uint32]$reviewed.Count; sources = [string[]]$reviewed.ToArray() }
}
"#;

const HYPERV_BOOTSTRAP: &str = r#"
try {
    $trustedModuleRoot = [IO.Path]::GetFullPath((Join-Path $PSHOME 'Modules')).TrimEnd('\')
    Import-Module -Name 'Hyper-V' -Force -ErrorAction Stop
    $getVmCommand = Get-Command -Name 'Get-VM' -Module 'Hyper-V' -CommandType Cmdlet -ErrorAction Stop
    $getVmHardDiskDriveCommand = Get-Command -Name 'Get-VMHardDiskDrive' -Module 'Hyper-V' -CommandType Cmdlet -ErrorAction Stop
    $getVmSnapshotCommand = Get-Command -Name 'Get-VMSnapshot' -Module 'Hyper-V' -CommandType Cmdlet -ErrorAction Stop
    $getVhdCommand = Get-Command -Name 'Get-VHD' -Module 'Hyper-V' -CommandType Cmdlet -ErrorAction Stop
    foreach ($command in @($getVmCommand, $getVmHardDiskDriveCommand, $getVmSnapshotCommand, $getVhdCommand)) {
        $modulePath = [IO.Path]::GetFullPath($command.Module.Path)
        if (-not $modulePath.StartsWith($trustedModuleRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
            throw 'untrusted Hyper-V module'
        }
    }
} catch {
    Emit 'error' 'integration-unavailable'; exit 12
}
"#;

const SCRIPT_TEMPLATE: &str = r#"
$utf8 = [Text.UTF8Encoding]::new($false)
[Console]::InputEncoding = $utf8
[Console]::OutputEncoding = $utf8
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
function Emit([string]$Status, [string]$Category, $Data = @{}) {
    $result = [ordered]@{ status = $Status; category = $Category }
    foreach ($entry in $Data.GetEnumerator()) { $result[$entry.Key] = $entry.Value }
    [Console]::Out.Write(($result | ConvertTo-Json -Compress))
}
__HYPERV_BOOTSTRAP__
function Open-FixedSession($Request, $Credential) {
    try { New-PSSession -VMId ([guid]$Request.vm_id) -Credential $Credential -ErrorAction Stop }
    catch {
        $text = [string]$_.Exception.Message
        $category = [string]$_.CategoryInfo.Category
        if ($category -in @('AuthenticationError', 'SecurityError') -or $text -match '(?i)credential|user name or password|logon failure') {
            Emit 'error' 'credential-denied'; exit 11
        }
        Emit 'error' 'integration-unavailable'; exit 12
    }
}
try {
    $request = [Console]::In.ReadToEnd() | ConvertFrom-Json -ErrorAction Stop
    $vm = & $getVmCommand -Id ([guid]$request.vm_id) -ErrorAction Stop
    $drives = @(& $getVmHardDiskDriveCommand -VM $vm -ErrorAction Stop)
    $checkpoints = @(& $getVmSnapshotCommand -VM $vm -ErrorAction Stop)
    if ($vm.Name -cne [string]$request.vm_name -or $vm.State -cne 'Running' -or
        $drives.Count -ne 1 -or $drives[0].Path -ine [string]$request.child_path -or
        $checkpoints.Count -ne 0) { throw 'target identity mismatch' }
    $vhd = & $getVhdCommand -Path $drives[0].Path -ErrorAction Stop
    if ($vhd.VhdType -cne 'Differencing' -or $vhd.ParentPath -ine [string]$request.parent_path) {
        throw 'target identity mismatch'
    }
    $secure = ConvertTo-SecureString ([string]$request.password) -AsPlainText -Force
    $credential = [pscredential]::new([string]$request.username, $secure)
    $request.password = $null
    $session = Open-FixedSession $request $credential
    try {
        $identity = Invoke-Command -Session $session -ArgumentList @($request.computer_name, $request.machine_guid) -ScriptBlock {
            param($ExpectedComputer, $ExpectedGuid)
            $windowsIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
            $principal = [Security.Principal.WindowsPrincipal]::new($windowsIdentity)
            if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'identity mismatch' }
            $actualGuid = (Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Cryptography' -Name MachineGuid).MachineGuid
            if ($env:COMPUTERNAME -ine $ExpectedComputer -or $actualGuid -cne $ExpectedGuid) { throw 'identity mismatch' }
            [pscustomobject]@{ computer_name = $env:COMPUTERNAME; machine_guid = $actualGuid }
        }
        if ($request.mode -ceq 'probe') { Emit 'ok' 'ready'; exit 0 }
        if ($request.mode -cne 'copy') { Emit 'error' 'unsafe-path'; exit 13 }

        $sourceItem = Get-Item -LiteralPath ([string]$request.source) -Force
        if (-not $sourceItem.PSIsContainer -and (($sourceItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -eq 0)) {
            $sourceHash = (Get-FileHash -LiteralPath $sourceItem.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        } else { Emit 'error' 'unsafe-path'; exit 13 }
        if ($sourceHash -cne [string]$request.sha256) { Emit 'error' 'verification-failed'; exit 14 }

        $prepared = Invoke-Command -Session $session -ArgumentList @($request.staging_root, $request.destination) -ScriptBlock {
            param($Root, $Relative)
            if ([IO.Path]::IsPathRooted($Relative) -or $Relative -match '(^|[\\/])\.\.([\\/]|$)' -or $Relative -match ':') { throw 'unsafe path' }
            $rootFull = [IO.Path]::GetFullPath($Root).TrimEnd('\')
            $target = [IO.Path]::GetFullPath([IO.Path]::Combine($rootFull, $Relative))
            if (-not $target.StartsWith($rootFull + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'unsafe path' }
            function Ensure-SafeDirectory([string]$FullPath) {
                $full = [IO.Path]::GetFullPath($FullPath).TrimEnd('\')
                $volume = [IO.Path]::GetPathRoot($full).TrimEnd('\')
                $current = $volume + '\'
                $remaining = $full.Substring($current.Length)
                foreach ($segment in $remaining.Split('\', [StringSplitOptions]::RemoveEmptyEntries)) {
                    $current = [IO.Path]::Combine($current, $segment)
                    if (Test-Path -LiteralPath $current) {
                        $item = Get-Item -LiteralPath $current -Force
                        if (-not $item.PSIsContainer -or (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0)) { throw 'unsafe path' }
                    } else {
                        $item = New-Item -ItemType Directory -Path $current
                        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'unsafe path' }
                    }
                }
            }
            __ACL_VALIDATOR__
            $existing = $rootFull
            while (-not (Test-Path -LiteralPath $existing)) {
                $existing = Split-Path -Parent $existing
                if ([string]::IsNullOrEmpty($existing)) { throw 'unsafe path' }
            }
            Assert-ProtectedTree $existing
            Ensure-SafeDirectory $rootFull
            $parent = Split-Path -Parent $target
            if ($parent -ine $rootFull) { throw 'unsafe path' }
            Assert-ProtectedTree $rootFull
            if (Test-Path -LiteralPath $target) { throw 'destination exists' }
            if (Get-ChildItem -LiteralPath $parent -Filter ((Split-Path -Leaf $target) + '.partial-*') -Force) { throw 'uncertain state' }
            [pscustomobject]@{ target = $target; temporary = $target + '.partial-' + [guid]::NewGuid().ToString('N') }
        }
        Copy-Item -LiteralPath $sourceItem.FullName -Destination $prepared.temporary -ToSession $session -ErrorAction Stop
        $result = Invoke-Command -Session $session -ArgumentList @($prepared.temporary, $prepared.target, $request.sha256) -ScriptBlock {
            param($Temporary, $Target, $ExpectedHash)
            $root = Split-Path -Parent $Target
            __ACL_VALIDATOR__
            Assert-ProtectedTree $root
            $temporaryHash = (Get-FileHash -LiteralPath $Temporary -Algorithm SHA256).Hash.ToLowerInvariant()
            if ($temporaryHash -cne $ExpectedHash) { throw 'hash mismatch' }
            Move-Item -LiteralPath $Temporary -Destination $Target
            $item = Get-Item -LiteralPath $Target -Force
            $finalHash = (Get-FileHash -LiteralPath $Target -Algorithm SHA256).Hash.ToLowerInvariant()
            [pscustomobject]@{ destination = $item.FullName; sha256 = $finalHash; bytes = [uint64]$item.Length }
        }
        Emit 'ok' 'transferred' ([ordered]@{
            vm_id = [string]$request.vm_id; computer_name = [string]$identity.computer_name
            machine_guid = [string]$identity.machine_guid; destination = [string]$result.destination
            sha256 = [string]$result.sha256; bytes = [uint64]$result.bytes
        })
    } finally { if ($session) { Remove-PSSession -Session $session -ErrorAction SilentlyContinue } }
} catch {
    $text = [string]$_.Exception.Message
    if ($text -match '(?i)unsafe path|destination exists') { Emit 'error' 'unsafe-path'; exit 13 }
    if ($text -match '(?i)uncertain state') { Emit 'error' 'uncertain-state'; exit 15 }
    if ($text -match '(?i)identity mismatch|hash mismatch') { Emit 'error' 'verification-failed'; exit 14 }
    Emit 'error' 'uncertain-state'; exit 15
}
"#;

const STAGING_SCRIPT_TEMPLATE: &str = r#"
$utf8 = [Text.UTF8Encoding]::new($false)
[Console]::InputEncoding = $utf8
[Console]::OutputEncoding = $utf8
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
function Emit([string]$Status, [string]$Category, $Data = @{}) {
    $result = [ordered]@{ status = $Status; category = $Category }
    foreach ($entry in $Data.GetEnumerator()) { $result[$entry.Key] = $entry.Value }
    [Console]::Out.Write(($result | ConvertTo-Json -Compress))
}
__HYPERV_BOOTSTRAP__
$aclValidatorSource = @'
__STAGING_ACL_VALIDATOR__
'@
__PENDING_RENAME_VALIDATOR__
function Open-FixedSession($Request, $Credential) {
    try { New-PSSession -VMId ([guid]$Request.vm_id) -Credential $Credential -ErrorAction Stop }
    catch {
        $text = [string]$_.Exception.Message
        $category = [string]$_.CategoryInfo.Category
        if ($category -in @('AuthenticationError', 'SecurityError') -or $text -match '(?i)credential|user name or password|logon failure') {
            Emit 'error' 'credential-denied'; exit 11
        }
        Emit 'error' 'integration-unavailable'; exit 12
    }
}
function Assert-SafeRelative([string]$Relative) {
    if ([string]::IsNullOrWhiteSpace($Relative) -or [IO.Path]::IsPathRooted($Relative) -or
        $Relative -match '(^|[\\/])\.\.([\\/]|$)' -or $Relative -match ':' -or
        $Relative.EndsWith('.') -or $Relative.EndsWith(' ')) { throw 'unsafe path' }
}
function Ensure-SafeDirectory([string]$FullPath) {
    $full = [IO.Path]::GetFullPath($FullPath).TrimEnd('\')
    $volume = [IO.Path]::GetPathRoot($full).TrimEnd('\')
    $current = $volume + '\'
    $remaining = $full.Substring($current.Length)
    foreach ($segment in $remaining.Split('\', [StringSplitOptions]::RemoveEmptyEntries)) {
        $current = [IO.Path]::Combine($current, $segment)
        if (Test-Path -LiteralPath $current) {
            $item = Get-Item -LiteralPath $current -Force
            if (-not $item.PSIsContainer -or (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0)) { throw 'unsafe path' }
        } else {
            $item = New-Item -ItemType Directory -Path $current
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'unsafe path' }
        }
    }
}
function Assert-SafeSourceFile([string]$Root, [string]$Path) {
    $rootFull = [IO.Path]::GetFullPath($Root).TrimEnd('\')
    $pathFull = [IO.Path]::GetFullPath($Path)
    if (-not $pathFull.StartsWith($rootFull + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'unsafe path' }
    $current = $pathFull
    while ($true) {
        $item = Get-Item -LiteralPath $current -Force
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'unsafe path' }
        if ($current -ieq $rootFull) { break }
        $current = Split-Path -Parent $current
        if ([string]::IsNullOrEmpty($current)) { throw 'unsafe path' }
    }
}
function Test-Package($Session, $Root, $Files, [uint64]$ExpectedBytes) {
    $result = Invoke-Command -Session $Session -ArgumentList @($Root, $Files, $ExpectedBytes) -ScriptBlock {
        param($Root, $Files, [uint64]$ExpectedBytes)
        __ACL_VALIDATOR__
        $rootFull = [IO.Path]::GetFullPath($Root).TrimEnd('\')
        $rootItem = Get-Item -LiteralPath $rootFull -Force
        if (-not $rootItem.PSIsContainer -or (($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0)) { throw 'verification failed' }
        Assert-ProtectedTree $rootFull $env:SystemRoot
        $allItems = @(Get-ChildItem -LiteralPath $rootFull -Recurse -Force)
        if (@($allItems | Where-Object { ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 }).Count -ne 0) { throw 'verification failed' }
        foreach ($directory in @($allItems | Where-Object { $_.PSIsContainer })) {
            Assert-ProtectedTree $directory.FullName $env:SystemRoot
        }
        $actual = @($allItems | Where-Object { -not $_.PSIsContainer })
        if ($actual.Count -ne $Files.Count) { throw 'verification failed' }
        [uint64]$total = 0
        foreach ($entry in $Files) {
            $target = [IO.Path]::GetFullPath([IO.Path]::Combine($rootFull, [string]$entry.relative_path))
            if (-not $target.StartsWith($rootFull + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'verification failed' }
            $item = Get-Item -LiteralPath $target -Force
            if ($item.PSIsContainer -or (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) -or [uint64]$item.Length -ne [uint64]$entry.bytes) { throw 'verification failed' }
            Assert-ProtectedFile $target
            $hash = (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant()
            if ($hash -cne [string]$entry.sha256) { throw 'verification failed' }
            $total += [uint64]$item.Length
        }
        if ($total -ne $ExpectedBytes) { throw 'verification failed' }
        [pscustomobject]@{ files = [uint32]$actual.Count; bytes = $total }
    }
    return $result
}
$mutationStarted = $false
$phase = 'request'
try {
    $request = [Console]::In.ReadToEnd() | ConvertFrom-Json -ErrorAction Stop
    $phase = 'host-target'
    $vm = & $getVmCommand -Id ([guid]$request.vm_id) -ErrorAction Stop
    $drives = @(& $getVmHardDiskDriveCommand -VM $vm -ErrorAction Stop)
    $checkpoints = @(& $getVmSnapshotCommand -VM $vm -ErrorAction Stop)
    if ($vm.Name -cne [string]$request.vm_name -or $vm.State -cne 'Running' -or
        $drives.Count -ne 1 -or $drives[0].Path -ine [string]$request.child_path -or
        $checkpoints.Count -ne 0) { throw 'target identity mismatch' }
    $vhd = & $getVhdCommand -Path $drives[0].Path -ErrorAction Stop
    if ($vhd.VhdType -cne 'Differencing' -or $vhd.ParentPath -ine [string]$request.parent_path) { throw 'target identity mismatch' }

    $phase = 'host-gpu'
    $hostVersion = Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
    $measuredBuild = [string]$hostVersion.CurrentBuildNumber + '.' + [string]$hostVersion.UBR
    $gpuPath = [string]$request.gpu_interface
    $interfaceEnd = $gpuPath.IndexOf('#{', [StringComparison]::Ordinal)
    if (-not $gpuPath.StartsWith('\\?\PCI#', [StringComparison]::OrdinalIgnoreCase) -or $interfaceEnd -le 4) { throw 'host GPU drift' }
    $deviceId = $gpuPath.Substring(4, $interfaceEnd - 4).Replace('#', '\')
    $activeDrivers = @(Get-CimInstance Win32_PnPSignedDriver | Where-Object {
        $_.DeviceClass -ieq 'DISPLAY' -and $_.DeviceID -ieq $deviceId
    })
    if ($activeDrivers.Count -ne 1 -or [string]$activeDrivers[0].DriverVersion -cne [string]$request.gpu_driver_version) { throw 'host GPU drift' }
    $phase = 'host-servicing'
    $pendingReboot = (Test-Path -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending') -or
        (Test-Path -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired')
    $pendingRenameProperty = Get-ItemProperty -LiteralPath 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager' -Name PendingFileRenameOperations -ErrorAction SilentlyContinue
    $pendingRenameEntries = if ($null -eq $pendingRenameProperty) { @() } else { @($pendingRenameProperty.PendingFileRenameOperations) }
    $pendingDeletes = Get-ReviewedPendingDeletes $pendingRenameEntries $request.allowed_pending_delete_sources
    $setup = Get-ItemProperty -LiteralPath 'HKLM:\SYSTEM\Setup' -Name SystemSetupInProgress -ErrorAction SilentlyContinue
    if ($pendingReboot -or (Test-Path -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Installer\InProgress') -or
        ($null -ne $setup -and [int]$setup.SystemSetupInProgress -ne 0)) { throw 'host servicing state' }

    $phase = 'source-signatures'
    $sourceRoot = [IO.Path]::GetFullPath([string]$request.source_root).TrimEnd('\')
    $sourceItem = Get-Item -LiteralPath $sourceRoot -Force
    if (-not $sourceItem.PSIsContainer -or (($sourceItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0)) { throw 'unsafe path' }
    foreach ($relative in $request.signature_files) {
        Assert-SafeRelative ([string]$relative)
        $path = [IO.Path]::GetFullPath([IO.Path]::Combine($sourceRoot, [string]$relative))
        Assert-SafeSourceFile $sourceRoot $path
        $signature = Get-AuthenticodeSignature -LiteralPath $path
        if ([string]$signature.Status -cne 'Valid' -or $null -eq $signature.SignerCertificate -or
            $signature.SignerCertificate.Thumbprint.ToLowerInvariant() -cne [string]$request.signer_thumbprint) {
            Emit 'error' 'invalid-signature'; exit 16
        }
    }

    $phase = 'guest-session'
    $secure = ConvertTo-SecureString ([string]$request.password) -AsPlainText -Force
    $credential = [pscredential]::new([string]$request.username, $secure)
    $request.password = $null
    $session = Open-FixedSession $request $credential
    try {
        Invoke-Command -Session $session -ArgumentList @($aclValidatorSource) -ScriptBlock {
            param($Source)
            . ([scriptblock]::Create($Source))
        }
        $phase = 'guest-identity'
        $identity = Invoke-Command -Session $session -ArgumentList @($request.computer_name, $request.machine_guid) -ScriptBlock {
            param($ExpectedComputer, $ExpectedGuid)
            $windowsIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
            $principal = [Security.Principal.WindowsPrincipal]::new($windowsIdentity)
            if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'identity mismatch' }
            $actualGuid = (Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Cryptography' -Name MachineGuid).MachineGuid
            if ($env:COMPUTERNAME -ine $ExpectedComputer -or $actualGuid -cne $ExpectedGuid) { throw 'identity mismatch' }
            $guestVersion = Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
            $guestBuild = [string]$guestVersion.CurrentBuildNumber + '.' + [string]$guestVersion.UBR
            [pscustomobject]@{ computer_name = $env:COMPUTERNAME; machine_guid = $actualGuid; build = $guestBuild }
        }
        $phase = 'guest-paths'
        $paths = Invoke-Command -Session $session -ArgumentList @($request.staging_root, $request.package_directory, $request.manifest_id) -ScriptBlock {
            param($StagingRoot, $PackageDirectory, $ManifestId)
            $windowsRoot = [IO.Path]::GetFullPath($env:SystemRoot).TrimEnd('\')
            $packageParent = [IO.Path]::Combine($windowsRoot, 'System32\HostDriverStore\FileRepository')
            $package = [IO.Path]::Combine($packageParent, $PackageDirectory)
            $alias = [IO.Path]::Combine($windowsRoot, 'System32\nvcuda.dll')
            $receipt = [IO.Path]::Combine([IO.Path]::GetFullPath($StagingRoot).TrimEnd('\'), 'applied-' + $ManifestId + '.json')
            [pscustomobject]@{ package_parent=$packageParent; package=$package; alias=$alias; receipt=$receipt }
        }
        Invoke-Command -Session $session -ArgumentList @($paths.package_parent, $request.staging_root, $paths.alias) -ScriptBlock {
            param($PackageParent, $StagingRoot, $Alias)
            __ACL_VALIDATOR__
            foreach ($entry in @(
                [pscustomobject]@{ path=$PackageParent; anchor=$env:SystemRoot },
                [pscustomobject]@{ path=$StagingRoot; anchor=[Environment]::GetFolderPath('ProgramFiles') },
                [pscustomobject]@{ path=(Split-Path -Parent $Alias); anchor=$env:SystemRoot }
            )) {
                $existing = [IO.Path]::GetFullPath([string]$entry.path).TrimEnd('\')
                while (-not (Test-Path -LiteralPath $existing)) {
                    $existing = Split-Path -Parent $existing
                    if ([string]::IsNullOrEmpty($existing)) { throw 'unsafe path' }
                }
                Assert-ProtectedTree $existing ([string]$entry.anchor)
            }
        }
        $mutationStarted = $true
        $lockPath = Invoke-Command -Session $session -ArgumentList @($paths.package_parent, $request.staging_root, $request.manifest_sha256) -ScriptBlock {
            param($PackageParent, $StagingRoot, $ManifestSha256)
            __ACL_VALIDATOR__
            function Ensure-SafeDirectory([string]$FullPath) {
                $full = [IO.Path]::GetFullPath($FullPath).TrimEnd('\')
                $volume = [IO.Path]::GetPathRoot($full).TrimEnd('\')
                $current = $volume + '\'
                $remaining = $full.Substring($current.Length)
                foreach ($segment in $remaining.Split('\', [StringSplitOptions]::RemoveEmptyEntries)) {
                    $current = [IO.Path]::Combine($current, $segment)
                    if (Test-Path -LiteralPath $current) {
                        $item = Get-Item -LiteralPath $current -Force
                        if (-not $item.PSIsContainer -or (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0)) { throw 'unsafe path' }
                    } else {
                        $item = New-Item -ItemType Directory -Path $current
                        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'unsafe path' }
                    }
                }
            }
            Ensure-SafeDirectory $PackageParent
            Ensure-SafeDirectory $StagingRoot
            Assert-ProtectedTree $PackageParent $env:SystemRoot
            Assert-ProtectedTree $StagingRoot
            $lock = [IO.Path]::Combine([IO.Path]::GetFullPath($StagingRoot).TrimEnd('\'), 'stage-runtime-v1.lock')
            $stream = [IO.File]::Open($lock, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
            try {
                $bytes = [Text.Encoding]::ASCII.GetBytes($ManifestSha256 + "`n")
                $stream.Write($bytes, 0, $bytes.Length)
                $stream.Flush()
            } finally { $stream.Dispose() }
            $lock
        }
        $present = Invoke-Command -Session $session -ArgumentList @($paths.package, $paths.alias, $paths.receipt) -ScriptBlock {
            param($Package, $Alias, $Receipt)
            $packageParent = Split-Path -Parent $Package
            $packageLeaf = Split-Path -Leaf $Package
            $receiptParent = Split-Path -Parent $Receipt
            $receiptLeaf = Split-Path -Leaf $Receipt
            $partials = @(Get-ChildItem -LiteralPath $packageParent -Filter ($packageLeaf + '.partial-*') -Force -ErrorAction SilentlyContinue).Count +
                @(Get-ChildItem -LiteralPath $receiptParent -Filter ($receiptLeaf + '.partial-*') -Force -ErrorAction SilentlyContinue).Count
            [pscustomobject]@{ package=(Test-Path -LiteralPath $Package); alias=(Test-Path -LiteralPath $Alias); receipt=(Test-Path -LiteralPath $Receipt); partials=$partials }
        }
        if ([int]$present.partials -ne 0) { $mutationStarted = $true; throw 'uncertain state' }
        if ($present.package -or $present.alias -or $present.receipt) {
            $mutationStarted = $true
            if (-not ($present.package -and $present.alias -and $present.receipt)) { throw 'uncertain state' }
            $receipt = Invoke-Command -Session $session -ArgumentList @($paths.receipt) -ScriptBlock { param($Path) Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json }
            if ([string]$receipt.manifest_id -cne [string]$request.manifest_id -or
                [string]$receipt.manifest_sha256 -cne [string]$request.manifest_sha256 -or
                [string]$receipt.alias_method -notin @('hardlink','copy')) { throw 'uncertain state' }
            $verified = Test-Package $session $paths.package $request.files ([uint64]$request.byte_count)
            $loader = $request.files | Where-Object { [string]$_.relative_path -ieq 'nvcuda_loader64.dll' }
            $aliasHash = Invoke-Command -Session $session -ArgumentList @($paths.alias, $paths.receipt, $lockPath) -ScriptBlock {
                param($Alias, $Receipt, $Lock)
                __ACL_VALIDATOR__
                foreach ($path in @($Alias, $Receipt, $Lock)) {
                    $item = Get-Item -LiteralPath $path -Force
                    if ($item.PSIsContainer -or (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0)) { throw 'verification failed' }
                    Assert-ProtectedFile $path
                    $anchor = if ($path -ieq $Alias) { $env:SystemRoot } else { [Environment]::GetFolderPath('ProgramFiles') }
                    Assert-ProtectedTree (Split-Path -Parent $path) $anchor
                }
                (Get-FileHash -LiteralPath $Alias -Algorithm SHA256).Hash.ToLowerInvariant()
            }
            if ($null -eq $loader -or $aliasHash -cne [string]$loader.sha256) { throw 'uncertain state' }
            Invoke-Command -Session $session -ArgumentList @($lockPath) -ScriptBlock { param($Lock) Remove-Item -LiteralPath $Lock -Force }
            Emit 'ok' 'already-applied' ([ordered]@{
                vm_id=[string]$request.vm_id; computer_name=[string]$identity.computer_name; machine_guid=[string]$identity.machine_guid
                manifest_id=[string]$request.manifest_id; manifest_sha256=[string]$request.manifest_sha256
                qualified_host_build=[string]$request.host_build; measured_host_build=$measuredBuild
                measured_guest_build=[string]$identity.build
                pending_delete_count=[uint32]$pendingDeletes.count; pending_delete_sources=@($pendingDeletes.sources)
                package_destination=[string]$paths.package; cuda_alias=[string]$paths.alias; alias_method=[string]$receipt.alias_method
                files=[uint32]$verified.files; bytes=[uint64]$verified.bytes
            }); exit 0
        }

        $temporary = Invoke-Command -Session $session -ArgumentList @($paths.package_parent, $request.package_directory) -ScriptBlock {
            param($PackageParent, $PackageDirectory)
            __ACL_VALIDATOR__
            Assert-ProtectedTree $PackageParent $env:SystemRoot
            $path = [IO.Path]::Combine($PackageParent, $PackageDirectory + '.partial-' + [guid]::NewGuid().ToString('N'))
            New-Item -ItemType Directory -Path $path | Out-Null
            $path
        }
        foreach ($entry in $request.files) {
            Assert-SafeRelative ([string]$entry.relative_path)
            $source = [IO.Path]::GetFullPath([IO.Path]::Combine($sourceRoot, [string]$entry.relative_path))
            Assert-SafeSourceFile $sourceRoot $source
            $item = Get-Item -LiteralPath $source -Force
            if ($item.PSIsContainer -or (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) -or [uint64]$item.Length -ne [uint64]$entry.bytes) { throw 'source changed' }
            if ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -cne [string]$entry.sha256) { throw 'source changed' }
            $target = Invoke-Command -Session $session -ArgumentList @($temporary, [string]$entry.relative_path) -ScriptBlock {
                param($Root, $Relative)
                $target = [IO.Path]::GetFullPath([IO.Path]::Combine($Root, $Relative))
                if (-not $target.StartsWith([IO.Path]::GetFullPath($Root).TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'unsafe path' }
                $parent = Split-Path -Parent $target
                New-Item -ItemType Directory -Path $parent -Force | Out-Null
                $target
            }
            Copy-Item -LiteralPath $source -Destination $target -ToSession $session -ErrorAction Stop
        }
        $verified = Test-Package $session $temporary $request.files ([uint64]$request.byte_count)
        $final = Invoke-Command -Session $session -ArgumentList @($temporary, $paths.package, $paths.alias, $paths.receipt, $request.manifest_id, $request.manifest_sha256) -ScriptBlock {
            param($Temporary, $Package, $Alias, $Receipt, $ManifestId, $ManifestSha256)
            if ((Test-Path -LiteralPath $Package) -or (Test-Path -LiteralPath $Alias) -or (Test-Path -LiteralPath $Receipt)) { throw 'uncertain state' }
            [IO.Directory]::Move($Temporary, $Package)
            $loader = [IO.Path]::Combine($Package, 'nvcuda_loader64.dll')
            $method = 'hardlink'
            try { New-Item -ItemType HardLink -Path $Alias -Target $loader -ErrorAction Stop | Out-Null }
            catch {
                if (Test-Path -LiteralPath $Alias) { throw 'uncertain state' }
                $method = 'copy'
                [IO.File]::Copy($loader, $Alias, $false)
            }
            $receiptValue = [ordered]@{ schema=1; manifest_id=$ManifestId; manifest_sha256=$ManifestSha256; alias_method=$method }
            $temporaryReceipt = $Receipt + '.partial-' + [guid]::NewGuid().ToString('N')
            $receiptValue | ConvertTo-Json -Compress | Set-Content -LiteralPath $temporaryReceipt -Encoding UTF8
            [IO.File]::Move($temporaryReceipt, $Receipt)
            [pscustomobject]@{ alias_method=$method }
        }
        $loader = $request.files | Where-Object { [string]$_.relative_path -ieq 'nvcuda_loader64.dll' }
        $verified = Test-Package $session $paths.package $request.files ([uint64]$request.byte_count)
        $aliasHash = Invoke-Command -Session $session -ArgumentList @($paths.alias, $paths.receipt, $lockPath) -ScriptBlock {
            param($Alias, $Receipt, $Lock)
            __ACL_VALIDATOR__
            foreach ($path in @($Alias, $Receipt, $Lock)) {
                $item = Get-Item -LiteralPath $path -Force
                if ($item.PSIsContainer -or (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0)) { throw 'verification failed' }
                Assert-ProtectedFile $path
                $anchor = if ($path -ieq $Alias) { $env:SystemRoot } else { [Environment]::GetFolderPath('ProgramFiles') }
                Assert-ProtectedTree (Split-Path -Parent $path) $anchor
            }
            (Get-FileHash -LiteralPath $Alias -Algorithm SHA256).Hash.ToLowerInvariant()
        }
        if ($null -eq $loader -or $aliasHash -cne [string]$loader.sha256) { throw 'verification failed' }
        Invoke-Command -Session $session -ArgumentList @($lockPath) -ScriptBlock { param($Lock) Remove-Item -LiteralPath $Lock -Force }
        Emit 'ok' 'applied' ([ordered]@{
            vm_id=[string]$request.vm_id; computer_name=[string]$identity.computer_name; machine_guid=[string]$identity.machine_guid
            manifest_id=[string]$request.manifest_id; manifest_sha256=[string]$request.manifest_sha256
            qualified_host_build=[string]$request.host_build; measured_host_build=$measuredBuild
            measured_guest_build=[string]$identity.build
            pending_delete_count=[uint32]$pendingDeletes.count; pending_delete_sources=@($pendingDeletes.sources)
            package_destination=[string]$paths.package; cuda_alias=[string]$paths.alias; alias_method=[string]$final.alias_method
            files=[uint32]$verified.files; bytes=[uint64]$verified.bytes
        })
    } finally { if ($session) { Remove-PSSession -Session $session -ErrorAction SilentlyContinue } }
} catch {
    $text = [string]$_.Exception.Message
    if ($mutationStarted -or $text -match '(?i)uncertain state') { Emit 'error' 'uncertain-state'; exit 15 }
    if ($text -match '(?i)signature') { Emit 'error' 'invalid-signature'; exit 16 }
    if ($text -match '(?i)credential|logon failure') { Emit 'error' 'credential-denied'; exit 11 }
    if ($text -match '(?i)target identity|identity mismatch|verification failed|source changed|unsafe path|host GPU drift|host servicing state') { Emit 'error' 'verification-failed'; exit 14 }
    if (-not $mutationStarted) { Emit 'error' 'preflight-failed' ([ordered]@{ phase=$phase }); exit 14 }
    Emit 'error' 'uncertain-state'; exit 15
}
"#;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::Duration;

    use super::{
        ACL_VALIDATOR, PENDING_RENAME_VALIDATOR, SCRIPT, SCRIPT_TEMPLATE, STAGING_SCRIPT, category,
        parse_probe, parse_stage_receipt, parse_transfer, run_script, run_script_with_command,
    };
    use crate::guest::GuestError;
    use crate::staging::{StageStatus, StagingError};
    use zeroize::Zeroizing;

    fn powershell() -> PathBuf {
        crate::windows_paths::windows_powershell_executable().unwrap()
    }

    fn fixed_script_fixture(new_session_body: &str, parent_path: &str) -> String {
        let test_bootstrap = r#"
$getVmCommand = Get-Command -Name 'Get-VM' -CommandType Function
$getVmHardDiskDriveCommand = Get-Command -Name 'Get-VMHardDiskDrive' -CommandType Function
$getVmSnapshotCommand = Get-Command -Name 'Get-VMSnapshot' -CommandType Function
$getVhdCommand = Get-Command -Name 'Get-VHD' -CommandType Function
"#;
        let script = SCRIPT_TEMPLATE
            .replace("__HYPERV_BOOTSTRAP__", test_bootstrap)
            .replace("__ACL_VALIDATOR__", ACL_VALIDATOR);
        format!(
            r#"
function Get-VM {{ param($Id, $ErrorAction) [pscustomobject]@{{ Name='HyperGpuSupport-Disposable-01'; State='Running' }} }}
function Get-VMHardDiskDrive {{ param($VM, $ErrorAction) [pscustomobject]@{{ Path='Z:\HyperGpuSupport\images\disposable\gpu-pv-slot-01\child.vhdx' }} }}
function Get-VMSnapshot {{ param($VM, $ErrorAction) }}
function Get-VHD {{ param($Path, $ErrorAction) [pscustomobject]@{{ VhdType='Differencing'; ParentPath='{parent_path}' }} }}
function New-PSSession {{ [CmdletBinding()]param($VMId, $Credential) {new_session_body} }}
{script}
"#
        )
    }

    fn probe_payload() -> Zeroizing<Vec<u8>> {
        Zeroizing::new(
            serde_json::to_vec(&serde_json::json!({
                "mode": "probe",
                "vm_id": "2627e735-5b33-4104-b739-622727dd3a40",
                "vm_name": "HyperGpuSupport-Disposable-01",
                "child_path": r"Z:\HyperGpuSupport\images\disposable\gpu-pv-slot-01\child.vhdx",
                "parent_path": r"Z:\HyperGpuSupport\images\golden\win11-pro-25h2-26200.9457-x64-v1\parent.vhdx",
                "computer_name": "TESTVM",
                "machine_guid": "046edc35-4c8f-4910-9c53-574681e623af",
                "staging_root": r"C:\Program Files\HyperGpuSupport\Staging",
                "source": "",
                "destination": "",
                "sha256": "",
                "username": "guest",
                "password": "secret"
            }))
            .unwrap(),
        )
    }

    fn acl_case(case: &str) -> String {
        let script = format!(
            r#"
{ACL_VALIDATOR}
$ErrorActionPreference = 'Stop'
$case = [Console]::In.ReadToEnd()
$currentSid = [Security.Principal.WindowsIdentity]::GetCurrent().User
function New-TestAcl($Owner, $BadSid, $BadRights) {{
    $acl = [Security.AccessControl.DirectorySecurity]::new()
    $acl.SetOwner($Owner)
    $rule = [Security.AccessControl.FileSystemAccessRule]::new(
        $currentSid, 'FullControl', 'ContainerInherit,ObjectInherit', 'None', 'Allow')
    $acl.AddAccessRule($rule) | Out-Null
    if ($BadSid) {{
        $badRule = [Security.AccessControl.FileSystemAccessRule]::new(
            $BadSid, $BadRights, 'ContainerInherit,ObjectInherit', 'None', 'Allow')
        $acl.AddAccessRule($badRule) | Out-Null
    }}
    $acl
}}
$localService = [Security.Principal.SecurityIdentifier]::new('S-1-5-19')
$users = [Security.Principal.SecurityIdentifier]::new('S-1-5-32-545')
$goodAcl = New-TestAcl $currentSid $null $null
$badWriterAcl = New-TestAcl $currentSid $localService 'Modify'
$badAncestorAcl = New-TestAcl $currentSid $localService 'DeleteSubdirectoriesAndFiles'
$badOwnerAcl = New-TestAcl $users $null $null
function Get-Item {{
    param($LiteralPath, [switch]$Force)
    $attributes = if ($case -eq 'replacement' -and $LiteralPath -like '*\Staging') {{
        [IO.FileAttributes]::Directory -bor [IO.FileAttributes]::ReparsePoint
    }} else {{ [IO.FileAttributes]::Directory }}
    [pscustomobject]@{{ PSIsContainer = $true; Attributes = $attributes }}
}}
function Get-Acl {{
    param($LiteralPath)
    if ($case -eq 'writer' -and $LiteralPath -like '*\Staging') {{ return $badWriterAcl }}
    if ($case -eq 'ancestor' -and $LiteralPath -like '*\Project') {{ return $badAncestorAcl }}
    if ($case -eq 'owner' -and $LiteralPath -like '*\Staging') {{ return $badOwnerAcl }}
    $goodAcl
}}
try {{ Assert-ProtectedTree 'C:\Program Files\Project\Staging' 'C:\Program Files'; [Console]::Out.Write('accepted') }}
catch {{ [Console]::Out.Write('rejected:' + $_.Exception.Message) }}
"#
        );
        run_script(
            &powershell(),
            &script,
            Zeroizing::new(case.as_bytes().to_vec()),
            Duration::from_secs(5),
        )
        .unwrap()
    }

    #[test]
    fn parses_success_without_accepting_extra_or_missing_fields() {
        assert_eq!(parse_probe(r#"{"status":"ok","category":"ready"}"#), Ok(()));
        let receipt = parse_transfer(concat!(
            r#"{"status":"ok","category":"transferred","vm_id":"vm","#,
            r#""computer_name":"TESTVM","machine_guid":"guid","destination":"C:\\stage\\x","#,
            r#""sha256":"0000000000000000000000000000000000000000000000000000000000000000","bytes":7}"#
        ))
        .unwrap();
        assert_eq!(receipt.bytes, 7);
        assert!(parse_probe(r#"{"status":"ok","category":"ready","secret":"x"}"#).is_err());
    }

    #[test]
    fn categorizes_denial_unavailable_and_interruption() {
        assert_eq!(category("credential-denied"), GuestError::CredentialDenied);
        assert_eq!(
            category("integration-unavailable"),
            GuestError::IntegrationUnavailable
        );
        assert_eq!(category("uncertain-state"), GuestError::Interrupted);
        assert_eq!(category("unexpected"), GuestError::AdapterFailed);
    }

    #[test]
    fn process_deadline_kills_and_reaps_stalled_adapter() {
        assert_eq!(
            run_script(
                &powershell(),
                "[Console]::In.ReadToEnd() | Out-Null; Start-Sleep -Seconds 30",
                Zeroizing::new(b"{}".to_vec()),
                Duration::from_millis(20)
            ),
            Err(GuestError::Interrupted)
        );
    }

    #[test]
    fn caller_module_path_cannot_shadow_hyper_v() {
        let shadow_root =
            std::env::temp_dir().join(format!("hyper-gpu-shadow-module-{}", std::process::id()));
        let module_root = shadow_root.join("Hyper-V");
        let marker = shadow_root.join("loaded.txt");
        let _ = fs::remove_dir_all(&shadow_root);
        fs::create_dir_all(&module_root).unwrap();
        fs::write(
            module_root.join("Hyper-V.psm1"),
            format!(
                "Set-Content -LiteralPath '{}' -Value loaded; function Get-VM {{}}; Export-ModuleMember Get-VM",
                marker.display()
            ),
        )
        .unwrap();

        let powershell = powershell();
        let module_path = powershell.parent().unwrap().join("Modules");
        let mut command = Command::new(&powershell);
        command.env("PSModulePath", &shadow_root);
        let output = run_script_with_command(
            command,
            &module_path,
            SCRIPT.as_str(),
            Zeroizing::new(Vec::new()),
            Duration::from_secs(5),
        )
        .unwrap();

        assert!(!marker.exists());
        assert!(matches!(
            parse_probe(&output),
            Err(GuestError::IntegrationUnavailable | GuestError::Interrupted)
        ));
        fs::remove_dir_all(shadow_root).unwrap();
    }

    #[test]
    fn nonzero_process_exit_cannot_report_success() {
        assert_eq!(
            run_script(
                &powershell(),
                "[Console]::Out.Write('{\"status\":\"ok\",\"category\":\"ready\"}'); exit 1",
                Zeroizing::new(Vec::new()),
                Duration::from_secs(5),
            ),
            Err(GuestError::AdapterFailed)
        );
    }

    #[test]
    fn powershell_parser_accepts_fixed_adapter() {
        assert!(!SCRIPT.contains("__"));
        assert!(!STAGING_SCRIPT.contains("__"));
        assert_eq!(
            run_script(
                &powershell(),
                "[scriptblock]::Create([Console]::In.ReadToEnd()) | Out-Null",
                Zeroizing::new(SCRIPT.as_bytes().to_vec()),
                Duration::from_secs(5)
            ),
            Ok(String::new())
        );
        assert_eq!(
            run_script(
                &powershell(),
                "[scriptblock]::Create([Console]::In.ReadToEnd()) | Out-Null",
                Zeroizing::new(STAGING_SCRIPT.as_bytes().to_vec()),
                Duration::from_secs(5)
            ),
            Ok(String::new())
        );
    }

    #[test]
    fn bounded_fixed_script_launches_through_command_line_transport() {
        let mut script = "# padding\n".repeat(2_500);
        script.push_str("[Console]::Out.Write('launched')");
        assert!(script.len() > 20_000);
        assert!(script.len() < 30_000);
        assert_eq!(
            run_script(
                &powershell(),
                &script,
                Zeroizing::new(Vec::new()),
                Duration::from_secs(5)
            ),
            Ok("launched".into())
        );
    }

    #[test]
    fn utf8_stream_round_trips_non_ascii_input() {
        let value = "Zażółć 🔒";
        assert_eq!(
            run_script(
                &powershell(),
                concat!(
                    "$utf8=[Text.UTF8Encoding]::new($false);",
                    "[Console]::InputEncoding=$utf8;[Console]::OutputEncoding=$utf8;",
                    "[Console]::Out.Write([Console]::In.ReadToEnd())"
                ),
                Zeroizing::new(value.as_bytes().to_vec()),
                Duration::from_secs(5)
            ),
            Ok(value.to_owned())
        );
    }

    #[test]
    fn deadline_covers_stalled_large_stdin_write() {
        assert_eq!(
            run_script(
                &powershell(),
                "Start-Sleep -Seconds 30",
                Zeroizing::new(vec![b'x'; 1024 * 1024]),
                Duration::from_millis(20)
            ),
            Err(GuestError::Interrupted)
        );
    }

    #[test]
    fn interrupted_copy_retains_partial_marker_for_reconciliation() {
        let marker = std::env::temp_dir().join(format!(
            "hyper-gpu-copy-{}.partial-test",
            std::process::id()
        ));
        let _ = fs::remove_file(&marker);
        assert_eq!(
            run_script(
                &powershell(),
                "$path=[Console]::In.ReadToEnd(); Set-Content -LiteralPath $path -Value partial -NoNewline; Start-Sleep -Seconds 30",
                Zeroizing::new(marker.to_string_lossy().as_bytes().to_vec()),
                // Several adapter tests launch Windows PowerShell in parallel. Give
                // process startup enough headroom while retaining a bounded stall.
                Duration::from_secs(5)
            ),
            Err(GuestError::Interrupted)
        );
        assert_eq!(fs::read_to_string(&marker).unwrap(), "partial");
        fs::remove_file(marker).unwrap();
    }

    #[test]
    fn fixed_script_classifies_denied_credentials() {
        let script = fixed_script_fixture(
            concat!(
                "$record=[Management.Automation.ErrorRecord]::new(",
                "[UnauthorizedAccessException]::new('denied'),'denied',",
                "[Management.Automation.ErrorCategory]::AuthenticationError,$null);",
                "$PSCmdlet.ThrowTerminatingError($record)"
            ),
            r"Z:\HyperGpuSupport\images\golden\win11-pro-25h2-26200.9457-x64-v1\parent.vhdx",
        );
        let output = run_script(
            &powershell(),
            &script,
            probe_payload(),
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(parse_probe(&output), Err(GuestError::CredentialDenied));
    }

    #[test]
    fn fixed_script_classifies_unavailable_integration() {
        let script = fixed_script_fixture(
            "throw [InvalidOperationException]::new('integration unavailable')",
            r"Z:\HyperGpuSupport\images\golden\win11-pro-25h2-26200.9457-x64-v1\parent.vhdx",
        );
        let output = run_script(
            &powershell(),
            &script,
            probe_payload(),
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(
            parse_probe(&output),
            Err(GuestError::IntegrationUnavailable)
        );
    }

    #[test]
    fn fixed_script_rejects_wrong_parent_before_session() {
        let script = fixed_script_fixture("throw 'must not open session'", r"Z:\wrong\parent.vhdx");
        let output = run_script(
            &powershell(),
            &script,
            probe_payload(),
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(parse_probe(&output), Err(GuestError::VerificationFailed));
    }

    #[test]
    fn fixed_script_enforces_flat_protected_staging_root() {
        assert!(SCRIPT.contains("if ($parent -ine $rootFull) { throw 'unsafe path' }"));
        assert!(SCRIPT.matches("Assert-ProtectedTree").count() >= 5);
        assert!(SCRIPT.contains("DeleteSubdirectoriesAndFiles"));
        assert!(SCRIPT.contains("$owner -notin $trustedOwners"));
        assert!(SCRIPT.contains("$sid -notin $trustedWriters"));
    }

    #[test]
    fn manifest_stager_is_fixed_verified_and_recoverable() {
        assert!(STAGING_SCRIPT.encode_utf16().count() < 30_000);
        assert!(STAGING_SCRIPT.contains("function global:Assert-ProtectedTree"));
        assert!(STAGING_SCRIPT.contains(". ([scriptblock]::Create($Source))"));
        assert!(STAGING_SCRIPT.contains("Get-AuthenticodeSignature"));
        assert!(STAGING_SCRIPT.contains(".partial-"));
        assert!(STAGING_SCRIPT.contains("Test-Package"));
        assert!(STAGING_SCRIPT.contains("'already-applied'"));
        assert!(STAGING_SCRIPT.contains("HostDriverStore\\FileRepository"));
        assert!(STAGING_SCRIPT.contains("System32\\nvcuda.dll"));
        assert!(STAGING_SCRIPT.contains("[IO.FileMode]::CreateNew"));
        assert!(STAGING_SCRIPT.contains("[IO.Directory]::Move($Temporary, $Package)"));
        assert!(STAGING_SCRIPT.contains("[IO.File]::Copy($loader, $Alias, $false)"));
        assert!(STAGING_SCRIPT.contains("host servicing state"));
        assert!(!STAGING_SCRIPT.contains("throw 'host Windows drift'"));
        assert!(STAGING_SCRIPT.contains("function Get-ReviewedPendingDeletes"));
        assert!(STAGING_SCRIPT.contains("$source -cnotin $allowedValues"));
        assert!(STAGING_SCRIPT.contains("pending_delete_sources=@($pendingDeletes.sources)"));
        assert!(STAGING_SCRIPT.contains("$guestVersion = Get-ItemProperty"));
        assert!(STAGING_SCRIPT.contains("measured_guest_build=[string]$identity.build"));
        assert!(STAGING_SCRIPT.contains("Win32_PnPSignedDriver"));
        assert!(!STAGING_SCRIPT.contains("Remove-Item -LiteralPath $Package"));
    }

    #[test]
    fn stage_receipt_parser_preserves_noop_and_uncertain_protocol() {
        let output = r#"{"status":"ok","category":"already-applied","vm_id":"vm","computer_name":"guest","machine_guid":"guid","manifest_id":"manifest","manifest_sha256":"hash","qualified_host_build":"26200.9457","measured_host_build":"26300.9457","measured_guest_build":"26200.9457","pending_delete_count":2,"pending_delete_sources":["cleanup-a","cleanup-b"],"package_destination":"C:\\package","cuda_alias":"C:\\alias","alias_method":"hardlink","files":217,"bytes":2850973044}"#;
        let receipt = parse_stage_receipt(output).unwrap();
        assert_eq!(receipt.status, StageStatus::AlreadyApplied);
        assert_eq!(receipt.files, 217);
        assert!(receipt.has_host_qualification_drift());
        assert!(receipt.has_host_guest_build_drift());
        assert!(receipt.has_pending_delete_cleanup());
        assert_eq!(
            parse_stage_receipt("not-json"),
            Err(StagingError::GuestStateUncertain)
        );
        assert_eq!(
            parse_stage_receipt(
                r#"{"status":"error","category":"preflight-failed","phase":"guest-paths"}"#
            ),
            Err(StagingError::GuestPreflightFailed {
                phase: "guest-paths".into()
            })
        );
        assert_eq!(
            parse_stage_receipt(
                r#"{"status":"error","category":"preflight-failed","phase":"native error text"}"#
            ),
            Err(StagingError::GuestStateUncertain)
        );
        let missing_cleanup_evidence = output.replace(
            r#","pending_delete_count":2,"pending_delete_sources":["cleanup-a","cleanup-b"]"#,
            "",
        );
        assert_eq!(
            parse_stage_receipt(&missing_cleanup_evidence),
            Err(StagingError::GuestStateUncertain)
        );
    }

    #[test]
    fn pending_rename_validator_accepts_only_reviewed_delete_pairs() {
        let cases = [
            ("@()", "@('reviewed')", "accepted:0:"),
            ("@('reviewed','')", "@('reviewed')", "accepted:1:reviewed"),
            ("@('reviewed')", "@('reviewed')", "rejected"),
            ("@('','')", "@('reviewed')", "rejected"),
            ("@('reviewed','replacement')", "@('reviewed')", "rejected"),
            ("@('unknown','')", "@('reviewed')", "rejected"),
        ];
        for (entries, allowed, expected) in cases {
            let script = format!(
                "{PENDING_RENAME_VALIDATOR}\ntry{{$result=Get-ReviewedPendingDeletes {entries} {allowed};[Console]::Out.Write('accepted:'+[string]$result.count+':'+(@($result.sources)-join ','))}}catch{{[Console]::Out.Write('rejected')}}"
            );
            let output = run_script(
                &powershell(),
                &script,
                Zeroizing::new(Vec::new()),
                Duration::from_secs(5),
            )
            .unwrap();
            assert_eq!(output, expected, "entries {entries}");
        }
        let absent_registry_script = format!(
            "{PENDING_RENAME_VALIDATOR}\n$entries=if($true){{@()}}else{{@('reviewed','')}};$result=Get-ReviewedPendingDeletes $entries @('reviewed');[Console]::Out.Write('accepted:'+[string]$result.count+':'+(@($result.sources)-join ','))"
        );
        let output = run_script(
            &powershell(),
            &absent_registry_script,
            Zeroizing::new(Vec::new()),
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(output, "accepted:0:");
    }

    #[test]
    fn publication_primitives_refuse_existing_destinations() {
        let output = run_script(
            &powershell(),
            concat!(
                "$root=Join-Path ([IO.Path]::GetTempPath()) ('hyper-gpu-publish-' + [guid]::NewGuid().ToString('N'));",
                "$a=Join-Path $root 'a';$b=Join-Path $root 'b';New-Item -ItemType Directory -Path $a,$b|Out-Null;",
                "$directoryRejected=$false;try{[IO.Directory]::Move($a,$b)}catch{$directoryRejected=$true};",
                "$source=Join-Path $root 'source';$target=Join-Path $root 'target';",
                "[IO.File]::WriteAllText($source,'source');[IO.File]::WriteAllText($target,'target');",
                "$fileRejected=$false;try{[IO.File]::Copy($source,$target,$false)}catch{$fileRejected=$true};",
                "Remove-Item -LiteralPath $root -Recurse -Force;",
                "if(-not $directoryRejected -or -not $fileRejected){exit 1}"
            ),
            Zeroizing::new(Vec::new()),
            Duration::from_secs(5),
        );
        assert_eq!(output, Ok(String::new()));
    }

    #[test]
    fn acl_validator_rejects_untrusted_access_and_replacement() {
        assert_eq!(acl_case("baseline"), "accepted");
        for case in ["writer", "ancestor", "owner", "replacement"] {
            assert!(acl_case(case).starts_with("rejected:"), "case {case}");
        }
    }
}
