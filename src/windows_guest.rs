//! Bounded Windows PowerShell Direct adapter for the pinned disposable guest.

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::config::{GuestConfiguration, SlotConfiguration};
use crate::guest::{GuestCredential, GuestError, GuestTransfer, TransferReceipt, TransferRequest};

const OUTPUT_LIMIT: usize = 64 * 1024;
const REQUEST_LIMIT: usize = 16 * 1024;

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
    let payload =
        Zeroizing::new(serde_json::to_vec(request).map_err(|_| GuestError::InvalidProtocol)?);
    if payload.len() > REQUEST_LIMIT {
        return Err(GuestError::InvalidProtocol);
    }
    run_script(powershell_path, SCRIPT, payload, timeout)
}

fn run_script(
    powershell_path: &Path,
    script: &str,
    payload: Zeroizing<Vec<u8>>,
    timeout: Duration,
) -> Result<String, GuestError> {
    let deadline = Instant::now() + timeout;
    let mut child = Command::new(powershell_path)
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
    if !status.success() && output.trim().is_empty() {
        return Err(GuestError::AdapterFailed);
    }
    Ok(output)
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

const SCRIPT: &str = r#"
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
    $vm = Get-VM -Id ([guid]$request.vm_id) -ErrorAction Stop
    $drives = @(Get-VMHardDiskDrive -VM $vm -ErrorAction Stop)
    $checkpoints = @(Get-VMSnapshot -VM $vm -ErrorAction Stop)
    if ($vm.Name -cne [string]$request.vm_name -or $vm.State -cne 'Running' -or
        $drives.Count -ne 1 -or $drives[0].Path -ine [string]$request.child_path -or
        $checkpoints.Count -ne 0) { throw 'target identity mismatch' }
    $vhd = Get-VHD -Path $drives[0].Path -ErrorAction Stop
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
            Ensure-SafeDirectory $rootFull
            $parent = Split-Path -Parent $target
            Ensure-SafeDirectory $parent
            if (Test-Path -LiteralPath $target) { throw 'destination exists' }
            if (Get-ChildItem -LiteralPath $parent -Filter ((Split-Path -Leaf $target) + '.partial-*') -Force) { throw 'uncertain state' }
            [pscustomobject]@{ target = $target; temporary = $target + '.partial-' + [guid]::NewGuid().ToString('N') }
        }
        Copy-Item -LiteralPath $sourceItem.FullName -Destination $prepared.temporary -ToSession $session -ErrorAction Stop
        $result = Invoke-Command -Session $session -ArgumentList @($prepared.temporary, $prepared.target, $request.sha256) -ScriptBlock {
            param($Temporary, $Target, $ExpectedHash)
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
    use std::path::Path;
    use std::time::Duration;

    use super::{SCRIPT, category, parse_probe, parse_transfer, run_script};
    use crate::guest::GuestError;
    use zeroize::Zeroizing;

    const POWERSHELL: &str = r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe";

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
}
