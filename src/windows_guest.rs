//! Bounded Windows PowerShell Direct adapter for the pinned disposable guest.

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{LazyLock, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::config::{GuestConfiguration, SlotConfiguration};
use crate::guest::{GuestCredential, GuestError, GuestTransfer, TransferReceipt, TransferRequest};

const OUTPUT_LIMIT: usize = 64 * 1024;
const REQUEST_LIMIT: usize = 16 * 1024;
const SYSTEM_MODULE_PATH: &str = r"C:\Windows\System32\WindowsPowerShell\v1.0\Modules";

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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireResponse {
    status: String,
    #[serde(default)]
    category: String,
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
    run_script_with_command(Command::new(powershell_path), script, payload, timeout)
}

fn run_script_with_command(
    mut command: Command,
    script: &str,
    payload: Zeroizing<Vec<u8>>,
    timeout: Duration,
) -> Result<String, GuestError> {
    let deadline = Instant::now() + timeout;
    configure_powershell_command(&mut command);
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

fn configure_powershell_command(command: &mut Command) {
    // PowerShell rebuilds its default module path when the variable is absent, so
    // remove caller-controlled discovery by replacing it with the protected system
    // module root. The fixed script separately verifies every Hyper-V command's
    // loaded module path before it reads the credential-bearing request.
    command.env("PSModulePath", SYSTEM_MODULE_PATH);
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
"#;

static SCRIPT: LazyLock<String> = LazyLock::new(|| {
    SCRIPT_TEMPLATE
        .replace("__HYPERV_BOOTSTRAP__", HYPERV_BOOTSTRAP)
        .replace("__ACL_VALIDATOR__", ACL_VALIDATOR)
});

const HYPERV_BOOTSTRAP: &str = r#"
try {
    $trustedModuleRoot = [IO.Path]::GetFullPath('C:\Windows\System32\WindowsPowerShell\v1.0\Modules').TrimEnd('\')
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use std::time::Duration;

    use super::{
        ACL_VALIDATOR, SCRIPT, SCRIPT_TEMPLATE, category, parse_probe, parse_transfer, run_script,
        run_script_with_command,
    };
    use crate::guest::GuestError;
    use zeroize::Zeroizing;

    const POWERSHELL: &str = r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe";

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
            Path::new(POWERSHELL),
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
                Path::new(POWERSHELL),
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

        let mut command = Command::new(POWERSHELL);
        command.env("PSModulePath", &shadow_root);
        let output = run_script_with_command(
            command,
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
                Path::new(POWERSHELL),
                "[Console]::Out.Write('{\"status\":\"ok\",\"category\":\"ready\"}'); exit 1",
                Zeroizing::new(Vec::new()),
                Duration::from_secs(5),
            ),
            Err(GuestError::AdapterFailed)
        );
    }

    #[test]
    fn powershell_parser_accepts_fixed_adapter() {
        assert_eq!(
            run_script(
                Path::new(POWERSHELL),
                "[scriptblock]::Create([Console]::In.ReadToEnd()) | Out-Null",
                Zeroizing::new(SCRIPT.as_bytes().to_vec()),
                Duration::from_secs(5)
            ),
            Ok(String::new())
        );
    }

    #[test]
    fn utf8_stream_round_trips_non_ascii_input() {
        let value = "Zażółć 🔒";
        assert_eq!(
            run_script(
                Path::new(POWERSHELL),
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
                Path::new(POWERSHELL),
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
                Path::new(POWERSHELL),
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
            Path::new(POWERSHELL),
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
            Path::new(POWERSHELL),
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
            Path::new(POWERSHELL),
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
    fn acl_validator_rejects_untrusted_access_and_replacement() {
        assert_eq!(acl_case("baseline"), "accepted");
        for case in ["writer", "ancestor", "owner", "replacement"] {
            assert!(acl_case(case).starts_with("rejected:"), "case {case}");
        }
    }
}
