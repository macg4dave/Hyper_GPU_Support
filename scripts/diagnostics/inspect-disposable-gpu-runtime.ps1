#Requires -RunAsAdministrator
# Read-only GPU-PV guest diagnosis. Privilege: elevated (Hyper-V PowerShell Direct).
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
. (Join-Path $repositoryRoot 'scripts\common\project-config.ps1')
$configuration = Import-ProjectConfiguration
$vmId = [guid](Get-ProjectConfigurationValue $configuration 'slot.vm_id')
$vmName = [string](Get-ProjectConfigurationValue $configuration 'slot.vm_name')
$child = [string](Get-ProjectConfigurationValue $configuration 'slot.child_path')
$parent = [string](Get-ProjectConfigurationValue $configuration 'slot.parent_path')
$gpuInterface = [string](Get-ProjectConfigurationValue $configuration 'slot.gpu_interface')
$gpuName = [string](Get-ProjectConfigurationValue $configuration 'slot.gpu_name')
$computer = [string](Get-ProjectConfigurationValue $configuration 'guest.computer_name')
$machineGuid = [string](Get-ProjectConfigurationValue $configuration 'guest.machine_guid')
$packageName = Split-Path -Leaf ([string](Get-ProjectConfigurationValue $configuration 'driver_manifest.source_path'))
$infName = [string](Get-ProjectConfigurationValue $configuration 'driver_manifest.inf_name')
$catalogName = [string](Get-ProjectConfigurationValue $configuration 'driver_manifest.catalog_name')
$output = Join-Path $repositoryRoot 'local\evidence\gpu-runtime-diagnostic.json'

$vm = Get-VM -Id $vmId -ErrorAction Stop
$drives = @(Get-VMHardDiskDrive -VM $vm -ErrorAction Stop)
$checkpoints = @(Get-VMSnapshot -VM $vm -ErrorAction SilentlyContinue)
$adapters = @(Get-VMGpuPartitionAdapter -VM $vm -ErrorAction Stop)
if ($vm.Name -cne $vmName -or $vm.State -cne 'Running' -or $drives.Count -ne 1 -or
    $drives[0].Path -ine $child -or $checkpoints.Count -ne 0 -or $adapters.Count -ne 1 -or
    [string]$adapters[0].InstancePath -ine $gpuInterface) {
    throw 'The running disposable VM, disk, checkpoints or GPU assignment differ from the configured target.'
}
$disk = Get-VHD -Path $child -ErrorAction Stop
if ($disk.VhdType -cne 'Differencing' -or $disk.ParentPath -ine $parent) {
    throw 'The disposable child does not reference the configured parent.'
}

$credential = Get-Credential -Message 'Enter the disposable guest administrator credential for read-only GPU diagnostics.'
$job = Invoke-Command -VMId $vmId -Credential $credential -AsJob -ArgumentList @(
    $computer, $machineGuid, $gpuName, $packageName, $infName, $catalogName
) -ScriptBlock {
    param($ExpectedComputer, $ExpectedGuid, $ExpectedGpu, $PackageName, $InfName, $CatalogName)
    $actualGuid = (Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Cryptography' -Name MachineGuid).MachineGuid
    if ($env:COMPUTERNAME -ine $ExpectedComputer -or $actualGuid -cne $ExpectedGuid) {
        throw 'Guest identity mismatch.'
    }
    $drivers = @(Get-CimInstance Win32_PnPSignedDriver | Where-Object {
        $_.DeviceClass -ieq 'DISPLAY' -and $_.DeviceName -ieq $ExpectedGpu
    })
    $gpuDeviceId = if ($drivers.Count -eq 1) { [string]$drivers[0].DeviceID } else { $null }
    $problemStatus = $null
    if ($gpuDeviceId) {
        $problemProperty = Get-PnpDeviceProperty -InstanceId $gpuDeviceId -KeyName 'DEVPKEY_Device_ProblemStatus' -ErrorAction SilentlyContinue
        if ($problemProperty) { $problemStatus = [string]$problemProperty.Data }
    }
    $devices = @(Get-CimInstance Win32_VideoController | ForEach-Object {
        [ordered]@{
            name = [string]$_.Name
            pnp_id = [string]$_.PNPDeviceID
            driver_version = [string]$_.DriverVersion
            problem = [uint32]$_.ConfigManagerErrorCode
            status = [string]$_.Status
        }
    })
    $package = Join-Path $env:SystemRoot "System32\HostDriverStore\FileRepository\$PackageName"
    $paths = @(
        (Join-Path $package $InfName),
        (Join-Path $package $CatalogName),
        (Join-Path $package 'nvcuda_loader64.dll'),
        (Join-Path $package 'nvcuda64.dll'),
        (Join-Path $package 'nvwgf2umx.dll'),
        (Join-Path $env:SystemRoot 'System32\nvcuda.dll')
    )
    $files = @($paths | ForEach-Object {
        [ordered]@{
            path = $_
            exists = Test-Path -LiteralPath $_ -PathType Leaf
            sha256 = if (Test-Path -LiteralPath $_ -PathType Leaf) {
                (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash.ToLowerInvariant()
            } else { $null }
        }
    })
    $events = @(Get-WinEvent -FilterHashtable @{
        LogName = 'System'
        StartTime = (Get-Date).AddHours(-4)
    } -MaxEvents 250 -ErrorAction SilentlyContinue | Where-Object {
        $_.ProviderName -match 'Kernel-PnP|Display|nvlddmkm|Hyper-V' -or
        $_.Message -match 'VEN_1414&DEV_008E|NVIDIA GeForce RTX 5060|vrd.inf'
    } | Select-Object -First 30 | ForEach-Object {
        [ordered]@{
            time = $_.TimeCreated.ToUniversalTime().ToString('o')
            provider = $_.ProviderName
            id = $_.Id
            level = $_.LevelDisplayName
            message = ([string]$_.Message).Substring(0, [Math]::Min(1200, ([string]$_.Message).Length))
        }
    })
    $pnpEvents = @(Get-WinEvent -FilterHashtable @{
        LogName = 'Microsoft-Windows-Kernel-PnP/Configuration'
        StartTime = (Get-Date).AddHours(-4)
    } -MaxEvents 150 -ErrorAction SilentlyContinue | Where-Object {
        $gpuDeviceId -and $_.Message -like "*$gpuDeviceId*"
    } | Select-Object -First 20 | ForEach-Object {
        [ordered]@{
            time = $_.TimeCreated.ToUniversalTime().ToString('o')
            id = $_.Id
            level = $_.LevelDisplayName
            message = ([string]$_.Message).Substring(0, [Math]::Min(1600, ([string]$_.Message).Length))
        }
    })
    $smiPath = Join-Path $package 'nvidia-smi.exe'
    $smi = [ordered]@{ present = Test-Path -LiteralPath $smiPath -PathType Leaf; exit_code = $null; output = $null }
    if ($smi.present) {
        $stdout = Join-Path $env:TEMP ('gpu-diag-' + [guid]::NewGuid().ToString('N') + '.out')
        $stderr = $stdout + '.err'
        try {
            $process = Start-Process -FilePath $smiPath -ArgumentList @('--query-gpu=name,driver_version', '--format=csv,noheader') -WindowStyle Hidden -RedirectStandardOutput $stdout -RedirectStandardError $stderr -PassThru
            if (-not $process.WaitForExit(15000)) {
                $process.Kill()
                $smi.output = 'timeout'
            } else {
                $smi.exit_code = $process.ExitCode
                $combined = ((Get-Content -LiteralPath $stdout -Raw -ErrorAction SilentlyContinue) + (Get-Content -LiteralPath $stderr -Raw -ErrorAction SilentlyContinue)).Trim()
                $smi.output = $combined.Substring(0, [Math]::Min(1200, $combined.Length))
            }
        } finally {
            Remove-Item -LiteralPath $stdout, $stderr -Force -ErrorAction SilentlyContinue
        }
    }
    [ordered]@{
        computer_name = $env:COMPUTERNAME
        machine_guid = $actualGuid
        display_drivers = @($drivers | ForEach-Object {
            [ordered]@{
                device_id = [string]$_.DeviceID
                inf = [string]$_.InfName
                version = [string]$_.DriverVersion
                manufacturer = [string]$_.Manufacturer
            }
        })
        video_controllers = $devices
        pnp_problem_status = $problemStatus
        files = $files
        nvidia_smi = $smi
        recent_system_events = $events
        recent_pnp_events = $pnpEvents
    }
}
if (-not (Wait-Job -Job $job -Timeout 90)) {
    Stop-Job -Job $job -ErrorAction SilentlyContinue
    Remove-Job -Job $job -Force -ErrorAction SilentlyContinue
    throw 'Guest diagnosis timed out.'
}
try {
    $guest = Receive-Job -Job $job -ErrorAction Stop
} finally {
    Remove-Job -Job $job -Force -ErrorAction SilentlyContinue
}
[ordered]@{
    checked_at = (Get-Date).ToUniversalTime().ToString('o')
    vm_id = [string]$vmId
    vm_state = [string]$vm.State
    child = $drives[0].Path
    parent = $disk.ParentPath
    gpu_interface = [string]$adapters[0].InstancePath
    guest = $guest
} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $output -Encoding UTF8
Write-Host "Read-only GPU diagnostic saved to $output"
