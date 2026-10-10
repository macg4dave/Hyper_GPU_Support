# Privilege: elevated. One Reapply only; continuous durable diagnostics, no retry.
#Requires -RunAsAdministrator
param([Parameter(Mandatory)][string]$ArtifactDirectory, [Parameter(Mandatory)][string]$TestExecutable, [string]$RunDirectory)
$ErrorActionPreference = 'Stop'
$repo = Split-Path (Split-Path $PSScriptRoot)
Set-Location -LiteralPath $repo
$run = if ($RunDirectory) { [IO.Path]::GetFullPath($RunDirectory) } else { Join-Path $repo ('local/evidence/reapply-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ') + '-' + [guid]::NewGuid().ToString('N')) }
if (!$run.StartsWith((Join-Path $repo 'local') + '\', [StringComparison]::OrdinalIgnoreCase) -or (Test-Path -LiteralPath $run)) { throw 'A new evidence directory inside repository local/ is required' }
[void][IO.Directory]::CreateDirectory($run)
$log = [IO.FileStream]::new((Join-Path $run 'continuous.jsonl'), 'CreateNew', 'Write', 'Read')
$clock = [Diagnostics.Stopwatch]::StartNew()
$counters = @{}
$markerOwned = $false
$installRoot = Join-Path ([Environment]::GetFolderPath('ProgramFiles')) 'HyperGpuSupportProduct'
$marker = Join-Path $installRoot 'diagnostics.enabled'
function Record([string]$stage, [string]$status, $context) {
    $line = @{utc=[DateTime]::UtcNow.ToString('o');elapsed_ms=$clock.ElapsedMilliseconds;stage=$stage;status=$status;context=$context} | ConvertTo-Json -Depth 8 -Compress
    $bytes = [Text.Encoding]::UTF8.GetBytes($line + "`n")
    $log.Write($bytes, 0, $bytes.Length)
    $log.Flush($true)
}
function Sample {
    $values = @{}
    foreach ($name in $counters.Keys) { $values[$name] = $counters[$name].NextValue() }
    $all = [Diagnostics.Process]::GetProcesses()
    $selected = @(); $handles = 0
    foreach ($p in $all) {
        try {
            $handles += $p.HandleCount
            if ($p.ProcessName -match '^(hyper-gpu.*|vmwp|vmms|WmiPrvSE|powershell)$') {
                $selected += @{pid=$p.Id;name=$p.ProcessName;handles=$p.HandleCount;working_set=$p.WorkingSet64;private_bytes=$p.PrivateMemorySize64;cpu_ms=$p.TotalProcessorTime.TotalMilliseconds}
            }
        } catch { $values['process_sample_race'] = $_.Exception.HResult }
        finally { $p.Dispose() }
    }
    Record 'heartbeat' 'sample' @{counters=$values;process_count=$all.Count;handle_count=$handles;processes=$selected}
}
function Run-Logged([string]$stage, [string]$executable, [string]$arguments) {
    Record $stage 'start' @{executable=$executable;arguments=$arguments}
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $start = [Diagnostics.ProcessStartInfo]::new($executable)
    $start.Arguments = $arguments
    $start.WorkingDirectory = $repo
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $p = [Diagnostics.Process]::Start($start)
    $out = $p.StandardOutput.ReadLineAsync(); $err = $p.StandardError.ReadLineAsync()
    $nextSample = 0L
    try {
        while (!$p.HasExited -or $out -or $err) {
            if ($out -and $out.IsCompleted) {
                $line = $out.GetAwaiter().GetResult()
                if ($null -eq $line) { $out = $null } else { Record $stage 'stdout' $line; $out = $p.StandardOutput.ReadLineAsync() }
            }
            if ($err -and $err.IsCompleted) {
                $line = $err.GetAwaiter().GetResult()
                if ($null -eq $line) { $err = $null } else { Record $stage 'stderr' $line; $err = $p.StandardError.ReadLineAsync() }
            }
            if (!$p.HasExited -and $timer.ElapsedMilliseconds -ge $nextSample) {
                Sample
                Record $stage 'in-progress' @{pid=$p.Id;duration_ms=$timer.ElapsedMilliseconds;significant_delay=($timer.ElapsedMilliseconds -ge 5000)}
                $nextSample = $timer.ElapsedMilliseconds + 5000
            }
            if (!$p.HasExited -and $timer.Elapsed.TotalSeconds -gt 4500) { throw 'Qualification deadline reached; operation was not retried or assumed stopped' }
            Start-Sleep -Milliseconds 100
        }
        Record $stage 'terminal' @{exit_code=$p.ExitCode;duration_ms=$timer.ElapsedMilliseconds}
        if ($p.ExitCode -ne 0) { throw "$stage failed; no retry" }
    } finally { $p.Dispose() }
}
try {
    Record 'qualification' 'start' @{run=$run;host_build=[Environment]::OSVersion.Version.ToString();architecture=[Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString();native_logs=(Join-Path $installRoot 'diagnostics')}
    foreach ($spec in @(@('cpu_percent','Processor','% Processor Time','_Total'), @('committed_bytes','Memory','Committed Bytes',$null), @('available_mbytes','Memory','Available MBytes',$null), @('disk_seconds_per_transfer','PhysicalDisk','Avg. Disk sec/Transfer','_Total'))) {
        $counter = [Diagnostics.PerformanceCounter]::new($spec[1],$spec[2])
        if ($spec[3]) { $counter.InstanceName = $spec[3] }
        [void]$counter.NextValue()
        $counters[$spec[0]] = $counter
    }
    Sample
    Record 'target-admission' 'start' 'configured disposable VM/GPU/disk'
    . (Join-Path $repo 'scripts/common/project-config.ps1')
    $lab = Import-ProjectConfiguration -Path (Join-Path $repo 'config/project.toml')
    $vmId = [string](Get-ProjectConfigurationValue $lab 'slot.vm_id')
    $gpu = [string](Get-ProjectConfigurationValue $lab 'slot.gpu_interface')
    $child = [string](Get-ProjectConfigurationValue $lab 'slot.child_path')
    $parent = [string](Get-ProjectConfigurationValue $lab 'slot.parent_path')
    Import-Module Hyper-V
    $vm = Get-VM -Id ([guid]$vmId)
    $disks = @(Get-VMHardDiskDrive -VM $vm)
    if ($vm.Generation -ne 2 -or $vm.State -ne 'Off' -or $disks.Count -ne 1 -or $disks[0].Path -ine $child -or (Get-VHD -Path $child).ParentPath -ine $parent) { throw 'Disposable identity/state mismatch' }
    if (@(Get-VMHostPartitionableGpu | Where-Object Name -ceq $gpu).Count -ne 1) { throw 'GPU identity mismatch' }
    Record 'target-admission' 'done' @{vm_id=$vmId;gpu=$gpu;child=$child;parent=$parent}
    # Fixed protected opt-in. Keep earlier logs and any pre-existing marker.
    $directory = Join-Path $installRoot 'diagnostics'
    foreach ($path in @($installRoot,$directory,$marker)) {
        if (Test-Path -LiteralPath $path) {
            $ancestor = $path
            while ($ancestor) {
                if ((Get-Item -LiteralPath $ancestor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Unsafe diagnostic path' }
                $ancestor = Split-Path $ancestor
            }
            if ((Get-Acl -LiteralPath $path).GetOwner([Security.Principal.SecurityIdentifier]).Value -notin @('S-1-5-18','S-1-5-32-544')) { throw 'Untrusted diagnostic path owner' }
        }
    }
    if (!(Test-Path -LiteralPath $directory)) { [void][IO.Directory]::CreateDirectory($directory); Set-Acl -LiteralPath $directory -AclObject (Get-Acl -LiteralPath $installRoot) }
    if (!(Test-Path -LiteralPath $marker)) {
        $file = [IO.File]::Open($marker, 'CreateNew', 'Write', 'Read'); $file.Flush($true); $file.Dispose()
        Set-Acl -LiteralPath $marker -AclObject (Get-Acl -LiteralPath $installRoot)
        $markerOwned = $true
    }
    $config = Join-Path $run 'runtime.toml'
    [IO.File]::WriteAllText($config, "schema = 2`n[[targets]]`nvm_id = '$vmId'`ngpu_interface = '$gpu'`nenabled = true`n", [Text.UTF8Encoding]::new($false))
    Run-Logged 'install-instrumented-artifacts' (Join-Path $ArtifactDirectory 'hyper-gpu-support.exe') "install --config `"$config`""
    $env:HYPER_GPU_SPRINT_OUTPUT = $run
    Record 'diagnostics-ready' 'done' 'durable native/bridge/guest traces plus five-second host telemetry; one Reapply'
    Run-Logged 'Reapply' $TestExecutable '--ignored --exact reapply_with_durable_diagnostics --nocapture'
    Record 'qualification' 'passed' 'Reapply returned success; no post-test investigation'
} catch {
    Record 'qualification' 'failed' $_.Exception.Message
    exit 1
} finally {
    foreach ($counter in $counters.Values) { $counter.Dispose() }
    if ($markerOwned) { [IO.File]::Delete($marker) }
    $log.Dispose()
}
