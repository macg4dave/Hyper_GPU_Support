# Privilege: elevated; bounded product installation qualification on the designated disposable VM.
#Requires -RunAsAdministrator
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$ArtifactDirectory,
    [Parameter(Mandatory)][string]$OutputDirectory
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repository = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$output = [IO.Path]::GetFullPath($OutputDirectory)
$localRoot = [IO.Path]::GetFullPath((Join-Path $repository 'local')) + '\'
if (-not $output.StartsWith($localRoot, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Qualification output must be inside repository local/.'
}
[void][IO.Directory]::CreateDirectory($output)
$reportPath = Join-Path $output 'enrollment-result.json'
$report = [ordered]@{ success = $false; phase = 'identity'; error = $null; checks = @() }
$cli = Join-Path (Resolve-Path -LiteralPath $ArtifactDirectory).Path 'hyper-gpu-support.exe'

function Publish {
    $report | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $reportPath -Encoding UTF8
}
function Invoke-Product([string]$Executable, [string[]]$Arguments, [int]$ExpectedExit = 0) {
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $Executable
    $start.Arguments = ($Arguments | ForEach-Object { '"' + $_ + '"' }) -join ' '
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $process = [Diagnostics.Process]::Start($start)
    try {
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        if (-not $process.WaitForExit(600000)) {
            throw 'Product command exceeded qualification deadline; inspect task state before retry.'
        }
        $value = [ordered]@{ command = $Arguments[0]; exit = $process.ExitCode; stdout = $stdout.Result; stderr = $stderr.Result }
        $report.checks += $value
        Publish
        if ($process.ExitCode -ne $ExpectedExit) { throw "Unexpected product exit: $($stderr.Result)" }
        $value
    } finally { $process.Dispose() }
}
function Write-Intent([string]$Path, [string]$Id, [string]$Interface) {
    if ($Id.Contains("'") -or $Interface.Contains("'") -or $Interface.Contains("`n")) { throw 'Unsafe fixture identity' }
    [IO.File]::WriteAllText($Path, "schema = 2`n[[targets]]`nvm_id = '$Id'`ngpu_interface = '$Interface'`nenabled = true`n", [Text.UTF8Encoding]::new($false))
}
function Get-DesignatedSnapshot {
    $vm = Get-VM -Id ([guid]$vmId) -ErrorAction Stop
    $disks = @(Get-VMHardDiskDrive -VM $vm)
    if ($vm.Generation -ne 2 -or $disks.Count -ne 1 -or $disks[0].Path -ine $child -or
        (Get-VHD -Path $child).ParentPath -ine $parent) { throw 'Designated VM/disk identity mismatch' }
    [ordered]@{
        vm_id = $vm.Id.ToString(); name = $vm.Name; generation = $vm.Generation
        state = [string]$vm.State; disk = $disks[0].Path; parent = $parent
        processors = (Get-VMProcessor -VM $vm).Count; memory = $vm.MemoryStartup
        dynamic_memory = $vm.DynamicMemoryEnabled; secure_boot = [string](Get-VMFirmware -VM $vm).SecureBoot
        tpm = (Get-VMSecurity -VM $vm).TpmEnabled
        gpu = @(Get-VMGpuPartitionAdapter -VM $vm | Select-Object InstancePath, MinPartitionVRAM, MaxPartitionVRAM, OptimalPartitionVRAM)
    }
}
try {
    . (Join-Path $repository 'scripts\common\project-config.ps1')
    $lab = Import-ProjectConfiguration -Path (Join-Path $repository 'config\project.toml')
    $vmId = [string](Get-ProjectConfigurationValue $lab 'slot.vm_id')
    $gpu = [string](Get-ProjectConfigurationValue $lab 'slot.gpu_interface')
    $child = [string](Get-ProjectConfigurationValue $lab 'slot.child_path')
    $parent = [string](Get-ProjectConfigurationValue $lab 'slot.parent_path')
    $report.before = Get-DesignatedSnapshot
    $inventory = (Invoke-Product $cli @('inventory')).stdout | ConvertFrom-Json
    if (@($inventory.vms | Where-Object { $_.vm_id -eq $vmId -and $_.generation -eq 2 }).Count -ne 1 -or
        @($inventory.gpus | Where-Object { $_.interface -ceq $gpu }).Count -ne 1) { throw 'Native VM/GPU identity mismatch' }
    $report.hostBuild = [Environment]::OSVersion.Version.ToString()
    $report.architecture = [Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
    $report.gpu = @($inventory.gpus | Where-Object { $_.interface -ceq $gpu })[0]
    $configPath = Join-Path $output 'target.toml'
    Write-Intent $configPath $vmId $gpu
    $installRoot = Join-Path ([Environment]::GetFolderPath('ProgramFiles')) 'HyperGpuSupportProduct'
    $stateRoot = Join-Path ([Environment]::GetFolderPath('CommonApplicationData')) 'HyperGpuSupportProduct'
    $pending = Join-Path $stateRoot 'installation-pending.json'

    $report.phase = 'install'
    [void](Get-DesignatedSnapshot)
    [void](Invoke-Product $cli @('install', '--config', $configPath))
    $enrollmentPath = Join-Path $stateRoot 'enrollment.json'
    $baseline = (Get-FileHash -LiteralPath $enrollmentPath -Algorithm SHA256).Hash
    $report.phase = 'invalid-enrollment-refusal'
    $invalid = Join-Path $output 'invalid-gpu.toml'
    Write-Intent $invalid $vmId '\\?\PCI#VEN_0000&DEV_0000#not-installed\GPUPARAV'
    [void](Get-DesignatedSnapshot)
    $refusal = Invoke-Product $cli @('install', '--config', $invalid) 1
    if (-not $refusal.stderr.Contains('enrollment GPU') -or
        (Get-FileHash -LiteralPath $enrollmentPath -Algorithm SHA256).Hash -ne $baseline -or
        (Test-Path -LiteralPath $pending)) { throw 'Invalid enrollment altered installed policy or did not fail as expected' }

    # Exercise documented interrupted-install recovery using an incomplete local bundle.
    # No guest operation is launched; the valid installation below repairs admission.
    $report.phase = 'interrupted-install'
    $incomplete = Join-Path $output 'incomplete-bundle'
    [void][IO.Directory]::CreateDirectory($incomplete)
    $incompleteCli = Join-Path $incomplete 'hyper-gpu-support.exe'
    Copy-Item -LiteralPath $cli -Destination $incompleteCli
    [void](Get-DesignatedSnapshot)
    [void](Invoke-Product $incompleteCli @('install', '--config', $configPath) 1)
    if (-not (Test-Path -LiteralPath $pending) -or
        (Get-FileHash -LiteralPath $enrollmentPath -Algorithm SHA256).Hash -ne $baseline) { throw 'Interrupted installation did not retain policy and pending marker' }
    $refusal = Invoke-Product $cli @('status', '--config', $configPath) 1
    if (-not $refusal.stderr.Contains('installation was interrupted')) { throw 'Interrupted installation admitted an operation' }
    $report.phase = 'installation-recovery'
    [void](Get-DesignatedSnapshot)
    [void](Invoke-Product $cli @('install', '--config', $configPath))
    if (Test-Path -LiteralPath $pending) { throw 'Successful reinstall retained installation marker' }

    $report.phase = 'installed-status-and-preview'
    $report.status = (Invoke-Product $cli @('status', '--config', $configPath)).stdout | ConvertFrom-Json
    $report.preview = (Invoke-Product $cli @('plan', '--config', $configPath)).stdout | ConvertFrom-Json
    if ($report.status.observed.vm_id -ne $vmId -or $report.status.observed.generation -ne 2) { throw 'Installed runner observed a different VM' }
    $enrollment = Get-Content -LiteralPath $enrollmentPath -Raw | ConvertFrom-Json
    if ($enrollment.schema -ne 2 -or @($enrollment.targets).Count -ne 1 -or
        $enrollment.targets[0].vm_id -ne $vmId -or $enrollment.targets[0].gpu_interface -cne $gpu) { throw 'Installed policy identity mismatch' }
    $report.artifacts = @()
    foreach ($property in $enrollment.artifacts.PSObject.Properties) {
        $path = Join-Path $installRoot $property.Name
        $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($hash -cne $property.Value) { throw 'Installed artifact hash mismatch' }
        $report.artifacts += @{ name = $property.Name; sha256 = $hash; owner = (Get-Acl -LiteralPath $path).Owner }
    }
    $task = Get-ScheduledTask -TaskName 'HyperGpuSupport-Product-v2' -TaskPath '\'
    if ($task.Principal.UserId -notin @('SYSTEM', 'S-1-5-18') -or @($task.Actions).Count -ne 1 -or
        $task.Actions[0].Execute -ine (Join-Path $installRoot 'hyper-gpu-runner.exe') -or
        -not [string]::IsNullOrEmpty($task.Actions[0].Arguments)) { throw 'Installed task does not have its fixed SYSTEM action' }
    $report.task = @{ user = $task.Principal.UserId; executable = $task.Actions[0].Execute; arguments = $task.Actions[0].Arguments }
    $report.after = Get-DesignatedSnapshot
    if (($report.before | ConvertTo-Json -Depth 10 -Compress) -cne ($report.after | ConvertTo-Json -Depth 10 -Compress)) { throw 'M1 qualification changed VM configuration' }
    $report.phase = 'complete'
    $report.success = $true
} catch { $report.error = $_.Exception.Message }
Publish
if (-not $report.success) { exit 1 }
