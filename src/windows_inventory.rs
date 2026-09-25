//! Bounded Windows process adapter for fixed, read-only inventory queries.

use std::collections::BTreeMap;
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::inventory::{
    Fact, FactStatus, InventoryError, InventoryReport, InventorySource, parse_protocol,
};

const TIMEOUT: Duration = Duration::from_secs(15);
const MAX_OUTPUT: usize = 64 * 1024;
const TARGET_PCI_PREFIX: &str = "VEN_10DE&DEV_2D05";

/// Executes a fixed set of non-mutating Windows and Hyper-V queries.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsInventory;

impl InventorySource for WindowsInventory {
    fn collect(&self) -> Result<InventoryReport, InventoryError> {
        let raw = run_bounded(
            "powershell.exe",
            &[
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                SCRIPT,
            ],
            TIMEOUT,
            MAX_OUTPUT,
        )?;
        normalize(parse_protocol(&raw)?)
    }
}

fn run_bounded(
    program: &str,
    arguments: &[&str],
    timeout: Duration,
    max_output: usize,
) -> Result<String, InventoryError> {
    let mut child = Command::new(program)
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| InventoryError::AdapterLaunch {
            kind: error.kind(),
            code: error.raw_os_error(),
        })?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| InventoryError::AdapterExit(None, "stdout pipe unavailable".to_owned()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| InventoryError::AdapterExit(None, "stderr pipe unavailable".to_owned()))?;
    let stdout_reader = thread::spawn(move || read_bounded(stdout, max_output));
    let stderr_reader = thread::spawn(move || read_bounded(stderr, max_output));
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(InventoryError::AdapterTimeout);
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(InventoryError::AdapterExit(
                    None,
                    format!(
                        "wait failed: {}; OS code {:?}",
                        error.kind(),
                        error.raw_os_error()
                    ),
                ));
            }
        }
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| InventoryError::AdapterExit(status.code(), "stdout reader failed".into()))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| InventoryError::AdapterExit(status.code(), "stderr reader failed".into()))??;
    if stdout.truncated || stderr.truncated {
        return Err(InventoryError::AdapterOutputTooLarge);
    }
    if !status.success() {
        return Err(InventoryError::AdapterExit(
            status.code(),
            sanitize_diagnostic(&stderr.bytes),
        ));
    }
    String::from_utf8(stdout.bytes).map_err(|_| InventoryError::InvalidProtocol)
}

struct BoundedBytes {
    bytes: Vec<u8>,
    truncated: bool,
}

fn read_bounded(mut reader: impl Read, limit: usize) -> Result<BoundedBytes, InventoryError> {
    let mut bytes = Vec::with_capacity(limit.min(4096));
    let mut truncated = false;
    let mut buffer = [0_u8; 4096];
    loop {
        let count = reader.read(&mut buffer).map_err(|error| {
            InventoryError::AdapterExit(
                None,
                format!(
                    "read failed: {}; OS code {:?}",
                    error.kind(),
                    error.raw_os_error()
                ),
            )
        })?;
        if count == 0 {
            break;
        }
        let remaining = limit.saturating_sub(bytes.len());
        bytes.extend_from_slice(&buffer[..count.min(remaining)]);
        truncated |= count > remaining;
    }
    Ok(BoundedBytes { bytes, truncated })
}

fn sanitize_diagnostic(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .chars()
        .filter(|character| !character.is_control() || *character == ' ')
        .take(256)
        .collect::<String>()
        .trim()
        .to_owned()
}

fn normalize(raw: InventoryReport) -> Result<InventoryReport, InventoryError> {
    let facts = raw
        .facts()
        .iter()
        .map(|fact| (fact.key().to_owned(), fact.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut output = facts
        .values()
        .filter(|fact| {
            !fact.key().starts_with("gpu.candidate.")
                && !fact.key().starts_with("gpup.candidate.")
                && !fact.key().starts_with("vm.candidate.")
                && !matches!(fact.key(), "gpu.query" | "gpup.query" | "vm.query")
        })
        .cloned()
        .collect::<Vec<_>>();
    select_gpu(&facts, &mut output)?;
    select_gpup(&facts, &mut output)?;
    select_vm(&facts, &mut output)?;
    InventoryReport::new(output)
}

fn select_gpu(
    facts: &BTreeMap<String, Fact>,
    output: &mut Vec<Fact>,
) -> Result<(), InventoryError> {
    let query = required(facts, "gpu.query")?;
    if query.status() != FactStatus::Known {
        for key in [
            "gpu.model",
            "gpu.pciid",
            "gpu.driverversion",
            "gpu.driverinf",
        ] {
            output.push(Fact::new(key, query.status(), query.value())?);
        }
        return Ok(());
    }
    let matches = candidate_indices(facts, "gpu.candidate.", ".pciid")?
        .into_iter()
        .filter(|index| {
            facts[&format!("gpu.candidate.{index}.pciid")]
                .value()
                .to_ascii_uppercase()
                .starts_with(TARGET_PCI_PREFIX)
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        let (status, reason) = if matches.is_empty() {
            (FactStatus::Missing, "target GPU not found")
        } else {
            (FactStatus::Unavailable, "target GPU is ambiguous")
        };
        for key in [
            "gpu.model",
            "gpu.pciid",
            "gpu.driverversion",
            "gpu.driverinf",
        ] {
            output.push(Fact::new(key, status, reason)?);
        }
        return Ok(());
    }
    let index = matches[0];
    for (source, target) in [
        ("model", "gpu.model"),
        ("pciid", "gpu.pciid"),
        ("driverversion", "gpu.driverversion"),
        ("driverinf", "gpu.driverinf"),
    ] {
        let fact = required(facts, &format!("gpu.candidate.{index}.{source}"))?;
        output.push(Fact::new(target, fact.status(), fact.value())?);
    }
    Ok(())
}

fn select_gpup(
    facts: &BTreeMap<String, Fact>,
    output: &mut Vec<Fact>,
) -> Result<(), InventoryError> {
    let query = required(facts, "gpup.query")?;
    if query.status() != FactStatus::Known {
        output.push(Fact::new("gpup.interface", query.status(), query.value())?);
        return Ok(());
    }
    let pci = output
        .iter()
        .find(|fact| fact.key() == "gpu.pciid" && fact.status() == FactStatus::Known)
        .map(Fact::value);
    let Some(pci) = pci else {
        output.push(Fact::new(
            "gpup.interface",
            FactStatus::Unavailable,
            "target GPU identity unavailable",
        )?);
        return Ok(());
    };
    let needle = format!("#{}#", pci.to_ascii_uppercase());
    let matches = candidate_indices(facts, "gpup.candidate.", ".interface")?
        .into_iter()
        .filter_map(|index| {
            let fact = &facts[&format!("gpup.candidate.{index}.interface")];
            fact.value()
                .to_ascii_uppercase()
                .contains(&needle)
                .then_some(fact)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [fact] => output.push(Fact::new(
            "gpup.interface",
            FactStatus::Known,
            fact.value(),
        )?),
        [] => output.push(Fact::new(
            "gpup.interface",
            FactStatus::Missing,
            "target interface not found",
        )?),
        _ => output.push(Fact::new(
            "gpup.interface",
            FactStatus::Unavailable,
            "target interface is ambiguous",
        )?),
    }
    Ok(())
}

fn select_vm(facts: &BTreeMap<String, Fact>, output: &mut Vec<Fact>) -> Result<(), InventoryError> {
    let query = required(facts, "vm.query")?;
    if query.status() != FactStatus::Known {
        output.push(Fact::new("vm.count", query.status(), query.value())?);
        output.push(Fact::new("vm.selection", query.status(), query.value())?);
        return Ok(());
    }
    let indices = candidate_indices(facts, "vm.candidate.", ".id")?;
    output.push(Fact::new(
        "vm.count",
        FactStatus::Known,
        indices.len().to_string(),
    )?);
    match indices.as_slice() {
        [] => output.push(Fact::new(
            "vm.selection",
            FactStatus::Missing,
            "no registered VM",
        )?),
        [index] => {
            for (source, target) in [
                ("id", "vm.selection"),
                ("state", "vm.state"),
                ("generation", "vm.generation"),
                ("version", "vm.version"),
            ] {
                let fact = required(facts, &format!("vm.candidate.{index}.{source}"))?;
                output.push(Fact::new(target, fact.status(), fact.value())?);
            }
        }
        _ => output.push(Fact::new(
            "vm.selection",
            FactStatus::Unavailable,
            "multiple registered VMs; explicit selection required",
        )?),
    }
    Ok(())
}

fn required<'a>(facts: &'a BTreeMap<String, Fact>, key: &str) -> Result<&'a Fact, InventoryError> {
    facts.get(key).ok_or(InventoryError::InvalidProtocol)
}

fn candidate_indices(
    facts: &BTreeMap<String, Fact>,
    prefix: &str,
    suffix: &str,
) -> Result<Vec<usize>, InventoryError> {
    let mut indices = Vec::new();
    for key in facts.keys() {
        let Some(text) = key
            .strip_prefix(prefix)
            .and_then(|rest| rest.strip_suffix(suffix))
        else {
            continue;
        };
        let index = text
            .parse::<usize>()
            .map_err(|_| InventoryError::InvalidProtocol)?;
        if index.to_string() != text || indices.contains(&index) {
            return Err(InventoryError::InvalidProtocol);
        }
        indices.push(index);
    }
    Ok(indices)
}

// Query transport only: Rust owns validation, target correlation, selection,
// error semantics, lifetime and rendering. DEC-013 records why the cancellable
// process boundary is retained for these WMI/Hyper-V providers.
const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
function Emit([string]$Key, [string]$Status, [string]$Value = '') {
    $hex = [BitConverter]::ToString([Text.Encoding]::UTF8.GetBytes($Value)).Replace('-', '')
    [Console]::Out.WriteLine("$Key`t$Status`t$hex")
}
function Failure([string]$Key, $Record) {
    $error = $Record.Exception
    $denied = $Record.CategoryInfo.Category -eq 'PermissionDenied' -or $error.HResult -eq -2147024891
    while ($null -ne $error -and -not $denied) {
        $issue = $error.PSObject.Properties['Issue']
        $denied = $null -ne $issue -and [string]$issue.Value -eq 'AccessDenied'
        $error = $error.InnerException
    }
    if ($denied) { Emit $Key 'denied' 'access.denied' } else { Emit $Key 'unavailable' ([string]$Record.FullyQualifiedErrorId) }
}
$os = Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
Emit 'host.edition' 'known' ([string]$os.EditionID)
Emit 'host.version' 'known' ([string]$os.DisplayVersion)
Emit 'host.build' 'known' ("$($os.CurrentBuild).$($os.UBR)")
Emit 'host.architecture' 'known' ([Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString())
$module = Get-Module -ListAvailable Hyper-V | Select-Object -First 1
if ($null -eq $module) { Emit 'hyperv.moduleversion' 'missing' 'module not installed' } else { Emit 'hyperv.moduleversion' 'known' ([string]$module.Version) }
try {
    $gpus = @(Get-CimInstance Win32_PnPSignedDriver | Where-Object { $_.DeviceClass -eq 'DISPLAY' })
    Emit 'gpu.query' 'known'
    for ($i = 0; $i -lt $gpus.Count; $i++) {
        Emit "gpu.candidate.$i.model" 'known' ([string]$gpus[$i].DeviceName)
        Emit "gpu.candidate.$i.pciid" 'known' (([string]$gpus[$i].DeviceID).Split('\')[1])
        Emit "gpu.candidate.$i.driverversion" 'known' ([string]$gpus[$i].DriverVersion)
        Emit "gpu.candidate.$i.driverinf" 'known' ([string]$gpus[$i].InfName)
    }
} catch { Failure 'gpu.query' $_ }
if ($null -eq (Get-Command Get-VMHostPartitionableGpu -ErrorAction SilentlyContinue)) {
    Emit 'gpup.query' 'missing' 'cmdlet not installed'
} else {
    try {
        $interfaces = @(Get-VMHostPartitionableGpu)
        Emit 'gpup.query' 'known'
        for ($i = 0; $i -lt $interfaces.Count; $i++) { Emit "gpup.candidate.$i.interface" 'known' ([string]$interfaces[$i].Name) }
    } catch { Failure 'gpup.query' $_ }
}
if ($null -eq (Get-Command Get-VM -ErrorAction SilentlyContinue)) {
    Emit 'vm.query' 'missing' 'cmdlet not installed'
} else {
    try {
        $vms = @(Get-VM)
        Emit 'vm.query' 'known'
        for ($i = 0; $i -lt $vms.Count; $i++) {
            Emit "vm.candidate.$i.id" 'known' ([string]$vms[$i].Id)
            Emit "vm.candidate.$i.state" 'known' ([string]$vms[$i].State)
            Emit "vm.candidate.$i.generation" 'known' ([string]$vms[$i].Generation)
            Emit "vm.candidate.$i.version" 'known' ([string]$vms[$i].Version)
        }
    } catch { Failure 'vm.query' $_ }
}
"#;

#[cfg(test)]
mod tests {
    use super::{SCRIPT, normalize, run_bounded};
    use crate::inventory::{FactStatus, InventoryError, parse_protocol};
    use std::time::Duration;

    fn hex(value: &str) -> String {
        value
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect()
    }

    fn line(key: &str, status: &str, value: &str) -> String {
        format!("{key}\t{status}\t{}\n", hex(value))
    }

    #[test]
    fn correlates_target_interface_when_unrelated_interface_is_first() {
        let mut protocol = line("gpu.query", "known", "");
        for (key, value) in [
            ("model", "NVIDIA GeForce RTX 5060"),
            ("pciid", "VEN_10DE&DEV_2D05&SUBSYS_8A151043&REV_A1"),
            ("driverversion", "32.0.16.1692"),
            ("driverinf", "oem59.inf"),
        ] {
            protocol.push_str(&line(&format!("gpu.candidate.0.{key}"), "known", value));
        }
        protocol.push_str(&line("gpup.query", "known", ""));
        protocol.push_str(&line(
            "gpup.candidate.0.interface",
            "known",
            r"\\?\PCI#VEN_1234&DEV_0001#A#{GUID}\GPUPARAV",
        ));
        protocol.push_str(&line(
            "gpup.candidate.1.interface",
            "known",
            r"\\?\PCI#VEN_10DE&DEV_2D05&SUBSYS_8A151043&REV_A1#B#{GUID}\GPUPARAV",
        ));
        protocol.push_str(&line("vm.query", "known", ""));
        protocol.push_str(&line("host.build", "known", "26200.9457"));
        let report = normalize(parse_protocol(&protocol).unwrap()).unwrap();
        let interface = report
            .facts()
            .iter()
            .find(|fact| fact.key() == "gpup.interface")
            .unwrap();
        assert_eq!(interface.status(), FactStatus::Known);
        assert!(interface.value().contains("VEN_10DE&DEV_2D05"));
        assert_eq!(
            report
                .facts()
                .iter()
                .find(|fact| fact.key() == "vm.selection")
                .unwrap()
                .status(),
            FactStatus::Missing
        );
    }

    #[test]
    fn preserves_missing_denied_and_ambiguous_states() {
        let mut protocol = line("gpu.query", "missing", "provider absent");
        protocol.push_str(&line("gpup.query", "denied", "permission.denied"));
        protocol.push_str(&line("vm.query", "known", ""));
        for index in 0..2 {
            protocol.push_str(&line(
                &format!("vm.candidate.{index}.id"),
                "known",
                &index.to_string(),
            ));
        }
        let report = normalize(parse_protocol(&protocol).unwrap()).unwrap();
        assert_eq!(
            report
                .facts()
                .iter()
                .find(|f| f.key() == "gpu.model")
                .unwrap()
                .status(),
            FactStatus::Missing
        );
        assert_eq!(
            report
                .facts()
                .iter()
                .find(|f| f.key() == "gpup.interface")
                .unwrap()
                .status(),
            FactStatus::Denied
        );
        assert_eq!(
            report
                .facts()
                .iter()
                .find(|f| f.key() == "vm.selection")
                .unwrap()
                .status(),
            FactStatus::Unavailable
        );
    }

    #[test]
    fn propagates_provider_failure_and_duplicate_target_ambiguity() {
        let mut unavailable = line("gpu.query", "unavailable", "wmi.provider.failed");
        unavailable.push_str(&line("gpup.query", "missing", "cmdlet not installed"));
        unavailable.push_str(&line("vm.query", "denied", "access.denied"));
        let report = normalize(parse_protocol(&unavailable).unwrap()).unwrap();
        assert_eq!(
            report
                .facts()
                .iter()
                .find(|fact| fact.key() == "gpu.model")
                .unwrap()
                .status(),
            FactStatus::Unavailable
        );
        assert_eq!(
            report
                .facts()
                .iter()
                .find(|fact| fact.key() == "gpup.interface")
                .unwrap()
                .status(),
            FactStatus::Missing
        );
        assert_eq!(
            report
                .facts()
                .iter()
                .find(|fact| fact.key() == "vm.selection")
                .unwrap()
                .status(),
            FactStatus::Denied
        );

        let mut duplicate = line("gpu.query", "known", "");
        for index in 0..2 {
            duplicate.push_str(&line(
                &format!("gpu.candidate.{index}.pciid"),
                "known",
                "VEN_10DE&DEV_2D05&SUBSYS_8A151043&REV_A1",
            ));
        }
        duplicate.push_str(&line("gpup.query", "known", ""));
        duplicate.push_str(&line("vm.query", "known", ""));
        let report = normalize(parse_protocol(&duplicate).unwrap()).unwrap();
        assert_eq!(
            report
                .facts()
                .iter()
                .find(|fact| fact.key() == "gpu.model")
                .unwrap()
                .status(),
            FactStatus::Unavailable
        );
    }

    #[test]
    fn selects_one_vm_and_reports_no_matching_target_interface() {
        let mut protocol = line("gpu.query", "known", "");
        for (key, value) in [
            ("model", "NVIDIA GeForce RTX 5060"),
            ("pciid", "VEN_10DE&DEV_2D05&SUBSYS_8A151043&REV_A1"),
            ("driverversion", "32.0.16.1692"),
            ("driverinf", "oem59.inf"),
        ] {
            protocol.push_str(&line(&format!("gpu.candidate.0.{key}"), "known", value));
        }
        protocol.push_str(&line("gpup.query", "known", ""));
        protocol.push_str(&line(
            "gpup.candidate.0.interface",
            "known",
            r"\\?\PCI#VEN_1234&DEV_0001#A#{GUID}\GPUPARAV",
        ));
        protocol.push_str(&line("vm.query", "known", ""));
        for (key, value) in [
            ("id", "00000000-0000-0000-0000-000000000001"),
            ("state", "Off"),
            ("generation", "2"),
            ("version", "12.0"),
        ] {
            protocol.push_str(&line(&format!("vm.candidate.0.{key}"), "known", value));
        }
        let report = normalize(parse_protocol(&protocol).unwrap()).unwrap();
        assert_eq!(
            report
                .facts()
                .iter()
                .find(|fact| fact.key() == "gpup.interface")
                .unwrap()
                .status(),
            FactStatus::Missing
        );
        assert_eq!(
            report
                .facts()
                .iter()
                .find(|fact| fact.key() == "vm.generation")
                .unwrap()
                .value(),
            "2"
        );
        assert_eq!(
            report
                .facts()
                .iter()
                .find(|fact| fact.key() == "vm.selection")
                .unwrap()
                .status(),
            FactStatus::Known
        );
    }

    #[test]
    fn rejects_noncanonical_candidate_indices_without_panicking() {
        let mut protocol = line("gpu.query", "known", "");
        protocol.push_str(&line(
            "gpu.candidate.00.pciid",
            "known",
            "VEN_10DE&DEV_2D05",
        ));
        protocol.push_str(&line("gpup.query", "known", ""));
        protocol.push_str(&line("vm.query", "known", ""));
        assert_eq!(
            normalize(parse_protocol(&protocol).unwrap()),
            Err(InventoryError::InvalidProtocol)
        );
    }

    #[test]
    fn process_runner_enforces_timeout_and_output_limit() {
        let timeout = run_bounded(
            "powershell.exe",
            &["-NoProfile", "-Command", "Start-Sleep -Seconds 5"],
            Duration::from_millis(100),
            1024,
        )
        .unwrap_err();
        assert!(matches!(timeout, InventoryError::AdapterTimeout));
        let oversized = run_bounded(
            "powershell.exe",
            &["-NoProfile", "-Command", "[Console]::Out.Write('A' * 4096)"],
            Duration::from_secs(5),
            128,
        )
        .unwrap_err();
        assert!(matches!(oversized, InventoryError::AdapterOutputTooLarge));
    }

    #[test]
    fn process_runner_preserves_launch_and_exit_context() {
        let launch = run_bounded(
            "definitely-not-a-real-inventory-program.exe",
            &[],
            Duration::from_secs(1),
            128,
        )
        .unwrap_err();
        assert!(matches!(
            launch,
            InventoryError::AdapterLaunch {
                kind: std::io::ErrorKind::NotFound,
                ..
            }
        ));
        let exit = run_bounded(
            "powershell.exe",
            &[
                "-NoProfile",
                "-Command",
                "[Console]::Error.Write('safe diagnostic'); exit 7",
            ],
            Duration::from_secs(5),
            128,
        )
        .unwrap_err();
        assert_eq!(
            exit,
            InventoryError::AdapterExit(Some(7), "safe diagnostic".to_owned())
        );
    }

    #[test]
    fn script_contains_queries_only() {
        for verb in [
            "Add-VM",
            "Set-VM",
            "Remove-VM",
            "New-VM",
            "Start-VM",
            "Stop-VM",
        ] {
            assert!(!SCRIPT.contains(verb));
        }
        assert!(SCRIPT.contains("Get-VMHostPartitionableGpu"));
    }
}
