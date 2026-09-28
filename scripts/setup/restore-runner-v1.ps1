#Requires -RunAsAdministrator
[CmdletBinding(SupportsShouldProcess, ConfirmImpact = 'High')]
param()

$ErrorActionPreference = 'Stop'
$installDirectory = 'C:\Program Files\HyperGpuSupport\Runner'
$dataDirectory = 'C:\ProgramData\HyperGpuSupport\Runner'
$runnerTarget = Join-Path $installDirectory 'hyper-gpu-runner.exe'
$clientTarget = Join-Path $installDirectory 'hyper-gpu-client.exe'
$rightsTarget = Join-Path $installDirectory 'hyper-gpu-rights.exe'
$policyTarget = Join-Path $dataDirectory 'policy-v1.json'
$enrollmentTarget = Join-Path $dataDirectory 'enrollment-v1.json'
$backupRoot = Join-Path $dataDirectory 'install-backup-v1'
$stagingRoot = Join-Path $dataDirectory 'install-staging-v1'
$taskPath = '\HyperGpuSupport\'
$taskName = 'Runner-v1'
$oldTaskName = 'ResetSlot-v1'
$accountName = 'HyperGpuRunner'
$recoveryPath = Join-Path $backupRoot 'recovery-v1.json'
$oldRunner = Join-Path $backupRoot 'hyper-gpu-runner.exe'
$oldPolicy = Join-Path $backupRoot 'policy-v1.json'
$oldTaskXml = Join-Path $backupRoot 'ResetSlot-v1.xml'
$backupRights = Join-Path $backupRoot 'hyper-gpu-rights.exe'
$runnerSidRecord = Join-Path $backupRoot 'runner-sid-v1.txt'

function Assert-Hash([string]$Path, [string]$Expected) {
    $actual = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
    if ($actual -ne $Expected) { throw "Hash mismatch for $Path (actual $actual)" }
}

foreach ($path in $recoveryPath,$oldRunner,$oldPolicy,$oldTaskXml,$backupRights) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Missing rollback preimage: $path" }
}
$recovery = Get-Content -LiteralPath $recoveryPath -Raw | ConvertFrom-Json
if ($recovery.schema -ne 1 -or [string]::IsNullOrWhiteSpace($recovery.client_sid) -or
    [string]::IsNullOrWhiteSpace($recovery.runner_sha256) -or [string]::IsNullOrWhiteSpace($recovery.policy_sha256) -or
    [string]::IsNullOrWhiteSpace($recovery.rights_helper_sha256) -or
    [string]::IsNullOrWhiteSpace($recovery.old_task_sddl) -or $null -eq $recovery.old_task_enabled -or
    $null -eq $recovery.acls) { throw 'Rollback recovery metadata is invalid' }
Assert-Hash $oldRunner $recovery.runner_sha256
Assert-Hash $oldPolicy $recovery.policy_sha256
Assert-Hash $backupRights $recovery.rights_helper_sha256
$runnerUser = Get-LocalUser -Name $accountName -ErrorAction SilentlyContinue
$runnerSid = if (Test-Path -LiteralPath $runnerSidRecord -PathType Leaf) {
    (Get-Content -LiteralPath $runnerSidRecord -Raw).Trim()
} elseif (Test-Path -LiteralPath $enrollmentTarget -PathType Leaf) {
    [string](Get-Content -LiteralPath $enrollmentTarget -Raw | ConvertFrom-Json).runner_sid
} else { $null }
if (-not [string]::IsNullOrWhiteSpace($runnerSid)) {
    $runnerSidObject = [Security.Principal.SecurityIdentifier]::new($runnerSid)
    if ($runnerSidObject.Value -ne $runnerSid) { throw 'Authoritative runner SID is not canonical' }
    if ($null -ne $runnerUser -and [string]$runnerUser.SID -ne $runnerSid) {
        throw "Refusing recovery: $accountName now has a different SID"
    }
    if (Test-Path -LiteralPath $rightsTarget -PathType Leaf) {
        Assert-Hash $rightsTarget $recovery.rights_helper_sha256
        $rightsHelper = $rightsTarget
    } else {
        $rightsHelper = $backupRights
    }
} elseif ($null -ne $runnerUser -or (Test-Path -LiteralPath $rightsTarget)) {
    throw 'Refusing recovery without an authoritative enrolled runner SID'
}

$effect = "disable and remove only $taskPath$taskName, quiesce and restore $taskPath$oldTaskName, remove only the five fixed runner account rights from the authoritative enrolled SID, restore captured runner/policy and ACL preimages, remove that exact-SID local account, and retain audit/results/backup"
if (-not $PSCmdlet.ShouldProcess('one enrolled GPU-PV runner slot', $effect)) { return }

foreach ($candidate in $taskName,$oldTaskName) {
    if (Get-ScheduledTask -TaskPath $taskPath -TaskName $candidate -ErrorAction SilentlyContinue) {
        Disable-ScheduledTask -TaskPath $taskPath -TaskName $candidate | Out-Null
    }
}
foreach ($candidate in $taskName,$oldTaskName) {
    $task = Get-ScheduledTask -TaskPath $taskPath -TaskName $candidate -ErrorAction SilentlyContinue
    if ($null -ne $task -and $task.State -eq 'Running') {
        throw "$taskPath$candidate is still running; it remains disabled. Retry recovery after it exits; do not force-stop it."
    }
}
foreach ($candidate in $taskName,$oldTaskName) {
    if (Get-ScheduledTask -TaskPath $taskPath -TaskName $candidate -ErrorAction SilentlyContinue) {
        Unregister-ScheduledTask -TaskPath $taskPath -TaskName $candidate -Confirm:$false
    }
}

Copy-Item -LiteralPath $oldRunner -Destination $runnerTarget -Force
Copy-Item -LiteralPath $oldPolicy -Destination $policyTarget -Force
Assert-Hash $runnerTarget $recovery.runner_sha256
Assert-Hash $policyTarget $recovery.policy_sha256
if (Test-Path -LiteralPath $clientTarget) {
    Remove-Item -LiteralPath $clientTarget -Force -ErrorAction Stop
    if (Test-Path -LiteralPath $clientTarget) { throw 'Runner client remains after removal' }
}
if (Test-Path -LiteralPath $stagingRoot) {
    Remove-Item -LiteralPath $stagingRoot -Recurse -Force -ErrorAction Stop
    if (Test-Path -LiteralPath $stagingRoot) { throw 'Runner staging remains after removal' }
}
$oldTaskDefinition = [xml](Get-Content -LiteralPath $oldTaskXml -Raw)
$settingsNode = $oldTaskDefinition.SelectSingleNode("/*[local-name()='Task']/*[local-name()='Settings']")
if ($null -eq $settingsNode) { throw 'Captured task XML has no Settings element' }
$enabledNode = $settingsNode.SelectSingleNode("*[local-name()='Enabled']")
if ($null -eq $enabledNode) {
    $enabledNode = $oldTaskDefinition.CreateElement('Enabled', $oldTaskDefinition.DocumentElement.NamespaceURI)
    $settingsNode.AppendChild($enabledNode) | Out-Null
}
$enabledNode.InnerText = 'false'
Register-ScheduledTask -TaskPath $taskPath -TaskName $oldTaskName -Xml $oldTaskDefinition.OuterXml | Out-Null
$taskService = New-Object -ComObject 'Schedule.Service'
$taskService.Connect()
$taskFolder = $taskService.GetFolder($taskPath.TrimEnd('\'))
$restoredTask = $taskFolder.GetTask($oldTaskName)
$restoredTask.SetSecurityDescriptor([string]$recovery.old_task_sddl, 0)

if (-not [string]::IsNullOrWhiteSpace($runnerSid)) {
    & $rightsHelper revoke $runnerSid
    if ($LASTEXITCODE -ne 0) { throw 'Failed to remove exact runner account rights' }
}
if (Test-Path -LiteralPath $rightsTarget -PathType Leaf) {
    & icacls.exe $rightsTarget /grant:r '*S-1-5-32-544:F' | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Failed to make the account-right helper removable' }
    Remove-Item -LiteralPath $rightsTarget -Force -ErrorAction Stop
    if (Test-Path -LiteralPath $rightsTarget) { throw 'Account-right helper remains after removal' }
}
if (Test-Path -LiteralPath $enrollmentTarget) {
    Remove-Item -LiteralPath $enrollmentTarget -Force -ErrorAction Stop
    if (Test-Path -LiteralPath $enrollmentTarget) { throw 'Runner enrollment remains after removal' }
}
if (-not [string]::IsNullOrWhiteSpace($runnerSid) -and (Get-LocalUser -SID $runnerSidObject -ErrorAction SilentlyContinue)) {
    Remove-LocalUser -SID $runnerSidObject
}
foreach ($property in $recovery.acls.PSObject.Properties) {
    $path = $property.Name
    if (-not (Test-Path -LiteralPath $path)) { throw "Cannot restore ACL for missing path: $path" }
    $acl = Get-Acl -LiteralPath $path
    $acl.SetSecurityDescriptorSddlForm([string]$property.Value)
    Set-Acl -LiteralPath $path -AclObject $acl
}
if ([bool]$recovery.old_task_enabled) {
    Enable-ScheduledTask -TaskPath $taskPath -TaskName $oldTaskName | Out-Null
}
$archive = "$backupRoot-restored-$([DateTimeOffset]::UtcNow.ToUnixTimeSeconds())"
Move-Item -LiteralPath $backupRoot -Destination $archive

[ordered]@{
    schema = 1
    restored_task = "$taskPath$oldTaskName"
    restored_runner_sha256 = $recovery.runner_sha256
    restored_policy_sha256 = $recovery.policy_sha256
    retained_audit = (Join-Path $dataDirectory 'audit')
    retained_results = (Join-Path $dataDirectory 'results')
    retained_backup = $archive
} | ConvertTo-Json
