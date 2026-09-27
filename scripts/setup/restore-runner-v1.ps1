#Requires -RunAsAdministrator
[CmdletBinding(SupportsShouldProcess, ConfirmImpact = 'High')]
param()

$ErrorActionPreference = 'Stop'
$installDirectory = 'C:\Program Files\HyperGpuSupport\Runner'
$dataDirectory = 'C:\ProgramData\HyperGpuSupport\Runner'
$runnerTarget = Join-Path $installDirectory 'hyper-gpu-runner.exe'
$clientTarget = Join-Path $installDirectory 'hyper-gpu-client.exe'
$policyTarget = Join-Path $dataDirectory 'policy-v1.json'
$enrollmentTarget = Join-Path $dataDirectory 'enrollment-v1.json'
$backupRoot = Join-Path $dataDirectory 'install-backup-v1'
$taskPath = '\HyperGpuSupport\'
$taskName = 'Runner-v1'
$oldTaskName = 'ResetSlot-v1'
$accountName = 'HyperGpuRunner'
$parentDirectory = 'Z:\HyperGpuSupport\images\golden\win11-pro-25h2-26200.9457-x64-v1'

if (-not (Test-Path -LiteralPath $backupRoot -PathType Container)) { throw "Missing rollback backup: $backupRoot" }
$enrollment = Get-Content -LiteralPath $enrollmentTarget -Raw | ConvertFrom-Json
if ($enrollment.schema -ne 1 -or [string]::IsNullOrWhiteSpace($enrollment.runner_sid) -or [string]::IsNullOrWhiteSpace($enrollment.client_sid)) {
    throw 'Installed enrollment is invalid; refusing broad cleanup'
}
$runnerSid = [string]$enrollment.runner_sid
$clientSid = [string]$enrollment.client_sid

$effect = "remove only $taskPath$taskName and $env:COMPUTERNAME\$accountName, restore the backed-up reset-only runner/task/policy, remove the runner parent ACL, and retain audit/results"
if (-not $PSCmdlet.ShouldProcess('one enrolled GPU-PV runner slot', $effect)) { return }

if (Get-ScheduledTask -TaskPath $taskPath -TaskName $taskName -ErrorAction SilentlyContinue) {
    Unregister-ScheduledTask -TaskPath $taskPath -TaskName $taskName -Confirm:$false
}
& icacls.exe $parentDirectory /remove "*$runnerSid" | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Failed to remove runner access from the fixed parent' }
if (Get-LocalUser -Name $accountName -ErrorAction SilentlyContinue) {
    Remove-LocalUser -Name $accountName
}

Remove-Item -LiteralPath $clientTarget -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $enrollmentTarget -Force -ErrorAction SilentlyContinue
Copy-Item -LiteralPath (Join-Path $backupRoot 'hyper-gpu-runner.exe') -Destination $runnerTarget -Force
Copy-Item -LiteralPath (Join-Path $backupRoot 'policy-v1.json') -Destination $policyTarget -Force
$oldXml = Join-Path $backupRoot 'ResetSlot-v1.xml'
if (Test-Path -LiteralPath $oldXml -PathType Leaf) {
    Register-ScheduledTask -TaskPath $taskPath -TaskName $oldTaskName -Xml (Get-Content -LiteralPath $oldXml -Raw) -Force | Out-Null
}

& icacls.exe $installDirectory /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' "*$clientSid`:(OI)(CI)RX" | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Failed to restore executable directory ACL' }
& icacls.exe $dataDirectory /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' "*$clientSid`:(OI)(CI)RX" | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Failed to restore runner data ACL' }

[ordered]@{
    schema = 1
    restored_task = "$taskPath$oldTaskName"
    retained_audit = (Join-Path $dataDirectory 'audit')
    retained_results = (Join-Path $dataDirectory 'results')
    removed_runner_sid = $runnerSid
} | ConvertTo-Json
