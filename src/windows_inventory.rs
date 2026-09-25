//! Windows PowerShell adapter for fixed, read-only inventory queries.

use std::process::Command;

use crate::inventory::{InventoryError, InventoryReport, InventorySource, parse_protocol};

/// Executes a fixed set of non-mutating Windows and Hyper-V queries.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsInventory;

impl InventorySource for WindowsInventory {
    fn collect(&self) -> Result<InventoryReport, InventoryError> {
        let output = Command::new("powershell.exe")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                SCRIPT,
            ])
            .output()
            .map_err(|_| InventoryError::AdapterFailed)?;
        if !output.status.success() {
            return Err(InventoryError::AdapterFailed);
        }
        let stdout =
            String::from_utf8(output.stdout).map_err(|_| InventoryError::InvalidProtocol)?;
        parse_protocol(&stdout)
    }
}

// Values use UTF-8 hex so device paths and localized text cannot break records.
// The script accepts no user input and contains queries only.
const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
function Emit([string]$Key, [string]$Status, [string]$Value = '') {
    $bytes = [Text.Encoding]::UTF8.GetBytes($Value)
    $hex = [BitConverter]::ToString($bytes).Replace('-', '')
    [Console]::Out.WriteLine("$Key`t$Status`t$hex")
}
function Query([string]$Key, [scriptblock]$Action) {
    try {
        $value = & $Action
        if ($null -eq $value -or [string]::IsNullOrWhiteSpace([string]$value)) {
            Emit $Key 'missing'
        } else {
            Emit $Key 'known' ([string]$value)
        }
    } catch {
        $message = $_.Exception.Message
        if ($_.Exception.HResult -eq -2147024891 -or $message -match '(?i)access.*denied|required permission') {
            Emit $Key 'denied'
        } else {
            Emit $Key 'unavailable' $_.Exception.GetType().FullName
        }
    }
}
$os = Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
Emit 'host.edition' 'known' ([string]$os.EditionID)
Emit 'host.version' 'known' ([string]$os.DisplayVersion)
Emit 'host.build' 'known' ("$($os.CurrentBuild).$($os.UBR)")
Emit 'host.architecture' 'known' ([Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString())
Query 'hyperv.moduleversion' { (Get-Module -ListAvailable Hyper-V | Select-Object -First 1).Version }
$gpu = Get-CimInstance Win32_PnPSignedDriver | Where-Object { $_.DeviceID -like 'PCI\VEN_10DE&DEV_2D05*' } | Select-Object -First 1
if ($null -eq $gpu) {
    Emit 'gpu.model' 'missing'
    Emit 'gpu.pciid' 'missing'
    Emit 'gpu.driverversion' 'missing'
    Emit 'gpu.driverinf' 'missing'
} else {
    Emit 'gpu.model' 'known' ([string]$gpu.DeviceName)
    Emit 'gpu.pciid' 'known' (([string]$gpu.DeviceID).Split('\')[1])
    Emit 'gpu.driverversion' 'known' ([string]$gpu.DriverVersion)
    Emit 'gpu.driverinf' 'known' ([string]$gpu.InfName)
}
Query 'gpup.interface' { (Get-VMHostPartitionableGpu | Select-Object -First 1).Name }
try {
    $vms = @(Get-VM)
    Emit 'vm.count' 'known' ([string]$vms.Count)
    if ($vms.Count -eq 0) {
        Emit 'vm.selection' 'missing' 'no registered VM'
    } elseif ($vms.Count -gt 1) {
        Emit 'vm.selection' 'unavailable' 'multiple registered VMs; explicit selection required'
    } else {
        $vm = $vms[0]
        Emit 'vm.selection' 'known' ([string]$vm.Id)
        Emit 'vm.state' 'known' ([string]$vm.State)
        Emit 'vm.generation' 'known' ([string]$vm.Generation)
        Emit 'vm.version' 'known' ([string]$vm.Version)
    }
} catch {
    $message = $_.Exception.Message
    $status = if ($_.Exception.HResult -eq -2147024891 -or $message -match '(?i)access.*denied|required permission') { 'denied' } else { 'unavailable' }
    Emit 'vm.count' $status
    Emit 'vm.selection' $status
}
"#;

#[cfg(test)]
mod tests {
    use super::{SCRIPT, WindowsInventory};

    #[test]
    fn adapter_is_constructible_and_script_has_no_mutating_hyper_v_verbs() {
        let _source = WindowsInventory;
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
