#Requires -RunAsAdministrator
# Read-only, identity-pinned GPU-PV and VMBus evidence collection.
[CmdletBinding()]
param([ValidateSet('initial', 'repeat')][string]$Window = 'initial')

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
. (Join-Path $root 'scripts\common\project-config.ps1')
$config = Import-ProjectConfiguration
$vmId = [guid](Get-ProjectConfigurationValue $config 'slot.vm_id')
$vmName = [string](Get-ProjectConfigurationValue $config 'slot.vm_name')
$child = [string](Get-ProjectConfigurationValue $config 'slot.child_path')
$parent = [string](Get-ProjectConfigurationValue $config 'slot.parent_path')
$interface = [string](Get-ProjectConfigurationValue $config 'slot.gpu_interface')
$computer = [string](Get-ProjectConfigurationValue $config 'guest.computer_name')
$machineGuid = [string](Get-ProjectConfigurationValue $config 'guest.machine_guid')
if ($Window -eq 'repeat') {
    $startUtc = [datetime]'2026-10-04T20:18:00Z'
    $endUtc = [datetime]'2026-10-04T20:22:00Z'
    $output = Join-Path $root 'local\evidence\gpu009-repeat-events.json'
} else {
    $startUtc = [datetime]'2026-10-04T19:55:00Z'
    $endUtc = [datetime]'2026-10-04T20:10:00Z'
    $output = Join-Path $root 'local\evidence\gpu009-timeline-vmbus.json'
}

$vm = Get-VM -Id $vmId -ErrorAction Stop
$drives = @(Get-VMHardDiskDrive -VM $vm -ErrorAction Stop)
$checkpoints = @(Get-VMSnapshot -VM $vm -ErrorAction SilentlyContinue)
$adapters = @(Get-VMGpuPartitionAdapter -VM $vm -ErrorAction Stop)
if ($vm.Name -cne $vmName -or $vm.State -cne 'Running' -or $drives.Count -ne 1 -or
    $drives[0].Path -ine $child -or $checkpoints.Count -ne 0 -or $adapters.Count -ne 1 -or
    [string]$adapters[0].InstancePath -ine $interface) {
    throw 'Disposable VM identity, state, disk, checkpoints or GPU assignment mismatch.'
}
$disk = Get-VHD -Path $child -ErrorAction Stop
if ($disk.VhdType -cne 'Differencing' -or $disk.ParentPath -ine $parent) {
    throw 'Disposable child-parent identity mismatch.'
}

$credential = Get-Credential -Message 'Enter the disposable guest administrator credential for read-only VMBus evidence collection.'
$job = Invoke-Command -VMId $vmId -Credential $credential -AsJob -ArgumentList @(
    $computer, $machineGuid, $startUtc, $endUtc
) -ScriptBlock {
    param($ExpectedComputer, $ExpectedGuid, $StartUtc, $EndUtc)
    $actualGuid = (Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Cryptography' -Name MachineGuid).MachineGuid
    if ($env:COMPUTERNAME -ine $ExpectedComputer -or $actualGuid -cne $ExpectedGuid) {
        throw 'Guest identity mismatch.'
    }
    function Read-Properties([string]$InstanceId) {
        $names = @(
            'DEVPKEY_Device_HardwareIds', 'DEVPKEY_Device_CompatibleIds',
            'DEVPKEY_Device_ClassGuid', 'DEVPKEY_Device_Parent',
            'DEVPKEY_Device_LocationInfo', 'DEVPKEY_Device_LocationPaths',
            'DEVPKEY_Device_ProblemCode', 'DEVPKEY_Device_ProblemStatus',
            'DEVPKEY_Device_Driver', 'DEVPKEY_Device_DriverInfPath',
            'DEVPKEY_Device_Service', 'DEVPKEY_Device_ContainerId',
            'DEVPKEY_Device_BusReportedDeviceDesc', 'DEVPKEY_Device_DeviceDesc',
            'DEVPKEY_Device_EnumeratorName', 'DEVPKEY_Device_Manufacturer',
            'DEVPKEY_Device_InstallDate', 'DEVPKEY_Device_LastArrivalDate'
        )
        $values = [ordered]@{}
        foreach ($name in $names) {
            $property = Get-PnpDeviceProperty -InstanceId $InstanceId -KeyName $name -ErrorAction SilentlyContinue
            if ($null -ne $property) {
                $values[$name] = @($property.Data | ForEach-Object { [string]$_ })
            }
        }
        $values
    }
    function Read-Events([string]$LogName, [datetime]$FromUtc, [datetime]$ToUtc, [string[]]$Ids) {
        $log = Get-WinEvent -ListLog $LogName -ErrorAction SilentlyContinue
        if ($null -eq $log -or -not $log.IsEnabled) { return @() }
        $events = @(Get-WinEvent -FilterHashtable @{
            LogName=$LogName; StartTime=$FromUtc.ToLocalTime(); EndTime=$ToUtc.ToLocalTime()
        } -ErrorAction SilentlyContinue)
        @($events | Where-Object {
            $message = [string]$_.Message
            $xml = $_.ToXml()
            foreach ($id in $Ids) {
                if ($message.IndexOf($id, [StringComparison]::OrdinalIgnoreCase) -ge 0 -or
                    $xml.IndexOf($id, [StringComparison]::OrdinalIgnoreCase) -ge 0) { return $true }
            }
            $false
        } | ForEach-Object {
            [ordered]@{
                utc=$_.TimeCreated.ToUniversalTime().ToString('o')
                record_id=$_.RecordId
                provider=$_.ProviderName
                id=$_.Id
                level=$_.LevelDisplayName
                message=[string]$_.Message
                xml=$_.ToXml()
            }
        })
    }

    $allVmbus = @(Get-PnpDevice | Where-Object { $_.InstanceId -like 'VMBUS\*' })
    $unknown = @($allVmbus | Where-Object {
        $_.FriendlyName -match 'Unknown' -or $_.Class -eq 'Unknown' -or $_.Status -ne 'OK'
    })
    $gpu = @(Get-PnpDevice | Where-Object { $_.InstanceId -like 'PCI\VEN_1414&DEV_008E*' })
    $detailIds = @($unknown.InstanceId) + @($gpu.InstanceId)
    $details = @($detailIds | Select-Object -Unique | ForEach-Object {
        $device = Get-PnpDevice -InstanceId $_ -ErrorAction Stop
        $signed = @(Get-CimInstance Win32_PnPSignedDriver | Where-Object { $_.DeviceID -ieq $device.InstanceId })
        [ordered]@{
            instance_id=[string]$device.InstanceId
            friendly_name=[string]$device.FriendlyName
            class=[string]$device.Class
            status=[string]$device.Status
            properties=Read-Properties ([string]$device.InstanceId)
            signed_driver=@($signed | ForEach-Object {
                [ordered]@{
                    inf=[string]$_.InfName; manufacturer=[string]$_.Manufacturer
                    version=[string]$_.DriverVersion; provider=[string]$_.DriverProviderName
                }
            })
        }
    })
    $eventIds = @($detailIds) + @('VEN_1414&DEV_008E', 'VirtualRender', 'vrd.inf',
        'VMBUS', 'GPU-PV', 'GpuPartition')
    $logs = @(
        'Microsoft-Windows-Kernel-PnP/Configuration',
        'Microsoft-Windows-DeviceSetupManager/Admin',
        'Microsoft-Windows-DeviceSetupManager/Operational',
        'System'
    )
    $history = [ordered]@{}
    foreach ($logName in $logs) {
        $history[$logName] = @(Read-Events $logName $StartUtc $EndUtc $eventIds)
    }
    $setupPath = Join-Path $env:SystemRoot 'INF\setupapi.dev.log'
    $setupMatches = @()
    if (Test-Path -LiteralPath $setupPath -PathType Leaf) {
        $setupMatches = @(Select-String -LiteralPath $setupPath -Pattern @(
            'dde9cbc0', 'VEN_1414&DEV_008E', 'VirtualRender', '13:05', '13:19', '13:20'
        ) -Context 1,2 -ErrorAction SilentlyContinue | Select-Object -Last 80 | ForEach-Object {
            [ordered]@{
                line_number=$_.LineNumber
                before=@($_.Context.PreContext)
                line=$_.Line
                after=@($_.Context.PostContext)
            }
        })
    }
    [ordered]@{
        collected_at_utc=(Get-Date).ToUniversalTime().ToString('o')
        computer_name=$env:COMPUTERNAME
        machine_guid=$actualGuid
        vmbus_devices=@($allVmbus | ForEach-Object {
            [ordered]@{
                instance_id=[string]$_.InstanceId; friendly_name=[string]$_.FriendlyName
                class=[string]$_.Class; status=[string]$_.Status
            }
        })
        details=$details
        events=$history
        setupapi_log_modified_utc=if (Test-Path -LiteralPath $setupPath -PathType Leaf) {
            (Get-Item -LiteralPath $setupPath).LastWriteTimeUtc.ToString('o')
        } else { $null }
        setupapi_matches=$setupMatches
    }
}
if (-not (Wait-Job -Job $job -Timeout 180)) {
    Stop-Job -Job $job -ErrorAction SilentlyContinue
    Remove-Job -Job $job -Force -ErrorAction SilentlyContinue
    throw 'Read-only guest evidence collection timed out.'
}
try { $guest = Receive-Job -Job $job -ErrorAction Stop }
finally { Remove-Job -Job $job -Force -ErrorAction SilentlyContinue }
[ordered]@{
    collected_at_utc=(Get-Date).ToUniversalTime().ToString('o')
    vm_id=[string]$vmId
    vm_state=[string]$vm.State
    child=$drives[0].Path
    parent=$disk.ParentPath
    gpu_interface=[string]$adapters[0].InstancePath
    guest=$guest
} | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $output -Encoding UTF8
Write-Host "Read-only VMBus evidence saved to $output"
