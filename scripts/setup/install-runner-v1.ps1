# Privilege: elevated.
#Requires -RunAsAdministrator
[CmdletBinding(SupportsShouldProcess, ConfirmImpact = 'High')]
param(
    [string] $ProjectConfigurationPath = (Join-Path $PSScriptRoot '..\..\config\project.toml'),
    [string] $ArtifactPinsPath = (Join-Path $PSScriptRoot '..\..\config\artifact-pins.toml')
)

$ErrorActionPreference = 'Stop'
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
. (Join-Path $repositoryRoot 'scripts\common\project-config.ps1')
$projectConfiguration = Import-ProjectConfiguration -Path $ProjectConfigurationPath
$artifactPins = Import-ProjectConfiguration -Path $ArtifactPinsPath
$artifactDirectory = [string](Get-ProjectConfigurationValue $projectConfiguration 'runner.artifacts.directory')
$runnerSource = Join-Path $artifactDirectory 'hyper-gpu-runner.exe'
$clientSource = Join-Path $artifactDirectory 'hyper-gpu-client.exe'
$rightsSource = Join-Path $artifactDirectory 'hyper-gpu-rights.exe'
$policySource = Join-Path $repositoryRoot 'config\runner-policy-v1.json'
$runnerHash = [string](Get-ProjectConfigurationValue $artifactPins 'runner.runner_sha256')
$clientHash = [string](Get-ProjectConfigurationValue $artifactPins 'runner.client_sha256')
$rightsHash = [string](Get-ProjectConfigurationValue $artifactPins 'runner.rights_sha256')
$policyHash = [string](Get-ProjectConfigurationValue $artifactPins 'runner.policy_sha256')
$installDirectory = [string](Get-ProjectConfigurationValue $projectConfiguration 'runner.install_directory')
$dataDirectory = [string](Get-ProjectConfigurationValue $projectConfiguration 'runner.data_directory')
$runnerTarget = Join-Path $installDirectory 'hyper-gpu-runner.exe'
$clientTarget = Join-Path $installDirectory 'hyper-gpu-client.exe'
$rightsTarget = Join-Path $installDirectory 'hyper-gpu-rights.exe'
$policyTarget = Join-Path $dataDirectory 'policy-v1.json'
$enrollmentTarget = Join-Path $dataDirectory 'enrollment-v1.json'
$accountName = [string](Get-ProjectConfigurationValue $projectConfiguration 'runner.account_name')
$taskPath = [string](Get-ProjectConfigurationValue $projectConfiguration 'runner.task_path')
$taskName = [string](Get-ProjectConfigurationValue $projectConfiguration 'runner.task_name')
$oldTaskName = [string](Get-ProjectConfigurationValue $projectConfiguration 'runner.legacy_task_name')
$parentDirectory = Split-Path -Parent ([string](Get-ProjectConfigurationValue $projectConfiguration 'slot.parent_path'))
$childDirectory = Split-Path -Parent ([string](Get-ProjectConfigurationValue $projectConfiguration 'slot.child_path'))
$taskExecutionTimeLimit = 'PT{0}S' -f [uint64](Get-ProjectConfigurationValue $projectConfiguration 'runner.task_execution_timeout_seconds')
$backupRoot = Join-Path $dataDirectory 'install-backup-v1'
$backupPreparing = Join-Path $dataDirectory 'install-backup-v1.preparing'
$stagingRoot = Join-Path $dataDirectory 'install-staging-v1'
$failureLog = Join-Path $dataDirectory 'install-failure-v1.txt'
$progressLog = Join-Path $dataDirectory 'install-progress-v1.txt'

trap {
    [IO.File]::WriteAllText($failureLog, ($_ | Out-String), (New-Object Text.UTF8Encoding($false)))
    exit 1
}

function Assert-Hash([string]$Path, [string]$Expected) {
    $actual = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
    if ($actual -ne $Expected) { throw "Hash mismatch for $Path (actual $actual)" }
}

foreach ($item in @(
    @{ Path = $runnerSource; Hash = $runnerHash },
    @{ Path = $clientSource; Hash = $clientHash },
    @{ Path = $rightsSource; Hash = $rightsHash },
    @{ Path = $policySource; Hash = $policyHash }
)) { Assert-Hash $item.Path $item.Hash }

$stateDirectory = Join-Path $dataDirectory 'state'
$resultDirectory = Join-Path $dataDirectory 'results'
$auditDirectory = Join-Path $dataDirectory 'audit'
foreach ($path in $installDirectory,$dataDirectory,$stateDirectory,$resultDirectory,$auditDirectory,$parentDirectory,$childDirectory) {
    if (-not (Test-Path -LiteralPath $path -PathType Container)) { throw "Required recovery path is missing: $path" }
}
foreach ($path in $runnerTarget,$policyTarget) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Required installed preimage is missing: $path" }
}
foreach ($path in $clientTarget,$rightsTarget,$enrollmentTarget) {
    if (Test-Path -LiteralPath $path) { throw "Refusing to overwrite unexpected candidate artifact: $path" }
}
if (Get-LocalUser -Name $accountName -ErrorAction SilentlyContinue) { throw "Refusing to reuse existing local account $accountName" }
if (Get-ScheduledTask -TaskPath $taskPath -TaskName $taskName -ErrorAction SilentlyContinue) { throw "Refusing to replace existing task $taskPath$taskName" }
$oldTask = Get-ScheduledTask -TaskPath $taskPath -TaskName $oldTaskName -ErrorAction Stop
if ($oldTask.State -eq 'Running') { throw "Refusing migration while $taskPath$oldTaskName is running" }
$taskService = New-Object -ComObject 'Schedule.Service'
$taskService.Connect()
$taskFolder = $taskService.GetFolder($taskPath.TrimEnd('\'))
$oldRegisteredTask = $taskFolder.GetTask($oldTaskName)
$oldTaskSddl = $oldRegisteredTask.GetSecurityDescriptor(7)
if (Test-Path -LiteralPath $backupRoot) { throw "Install backup already exists: $backupRoot" }
if (Test-Path -LiteralPath $stagingRoot) { throw "Install staging already exists: $stagingRoot" }

$clientSid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$qualifiedRunner = "$env:COMPUTERNAME\$accountName"
$effect = "stage and hash artifacts under administrator-only ACLs; disable and replace only $taskPath$oldTaskName; create $qualifiedRunner in built-in Hyper-V Administrators; add only SeBatchLogonRight plus deny network/interactive/remote-interactive/service logon to its exact SID through the fixed Rust LSA helper; grant exact parent read/child modify access; register disabled, verify, then enable $taskPath$taskName; retain complete ACL/task/file recovery preimages"
if (-not $PSCmdlet.ShouldProcess('one enrolled GPU-PV runner slot', $effect)) { return }

foreach ($path in $failureLog,$progressLog) {
    if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force -ErrorAction Stop }
}
if (Test-Path -LiteralPath $backupPreparing) {
    Remove-Item -LiteralPath $backupPreparing -Recurse -Force -ErrorAction Stop
    if (Test-Path -LiteralPath $backupPreparing) { throw 'Stale recovery preparation remains after removal' }
}
New-Item -ItemType Directory -Path $backupPreparing -Force | Out-Null
& icacls.exe $backupPreparing /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Failed to secure runner recovery directory' }
$recovery = [ordered]@{
    schema = 1
    client_sid = $clientSid
    account_name = $accountName
    runner_sha256 = (Get-FileHash -LiteralPath $runnerTarget -Algorithm SHA256).Hash
    policy_sha256 = (Get-FileHash -LiteralPath $policyTarget -Algorithm SHA256).Hash
    rights_helper_sha256 = $rightsHash
    old_task_sddl = $oldTaskSddl
    old_task_enabled = [bool]$oldTask.Settings.Enabled
    acls = [ordered]@{}
}
foreach ($path in $installDirectory,$dataDirectory,$stateDirectory,$resultDirectory,$auditDirectory,$parentDirectory,$childDirectory,$runnerTarget,$policyTarget) {
    $recovery.acls[$path] = (Get-Acl -LiteralPath $path).Sddl
}
[IO.File]::WriteAllText((Join-Path $backupPreparing 'recovery-v1.json'), ($recovery | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding($false)))
Copy-Item -LiteralPath $runnerTarget -Destination (Join-Path $backupPreparing 'hyper-gpu-runner.exe')
Copy-Item -LiteralPath $policyTarget -Destination (Join-Path $backupPreparing 'policy-v1.json')
Copy-Item -LiteralPath $rightsSource -Destination (Join-Path $backupPreparing 'hyper-gpu-rights.exe')
Assert-Hash (Join-Path $backupPreparing 'hyper-gpu-rights.exe') $rightsHash
Export-ScheduledTask -TaskPath $taskPath -TaskName $oldTaskName | Set-Content -LiteralPath (Join-Path $backupPreparing 'ResetSlot-v1.xml') -Encoding Unicode
Move-Item -LiteralPath $backupPreparing -Destination $backupRoot

try {
    New-Item -ItemType Directory -Path $stagingRoot -Force | Out-Null
    & icacls.exe $stagingRoot /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Failed to secure runner staging directory' }
    $stagedRunner = Join-Path $stagingRoot 'hyper-gpu-runner.exe'
    $stagedClient = Join-Path $stagingRoot 'hyper-gpu-client.exe'
    $stagedRights = Join-Path $stagingRoot 'hyper-gpu-rights.exe'
    $stagedPolicy = Join-Path $stagingRoot 'policy-v1.json'
    Copy-Item -LiteralPath $runnerSource -Destination $stagedRunner
    Copy-Item -LiteralPath $clientSource -Destination $stagedClient
    Copy-Item -LiteralPath $rightsSource -Destination $stagedRights
    Copy-Item -LiteralPath $policySource -Destination $stagedPolicy
    Assert-Hash $stagedRunner $runnerHash
    Assert-Hash $stagedClient $clientHash
    Assert-Hash $stagedRights $rightsHash
    Assert-Hash $stagedPolicy $policyHash

    $passwordBytes = New-Object byte[] 48
    $random = [Security.Cryptography.RandomNumberGenerator]::Create()
    $random.GetBytes($passwordBytes)
    $random.Dispose()
    $passwordText = [Convert]::ToBase64String($passwordBytes)
    $password = ConvertTo-SecureString $passwordText -AsPlainText -Force
    $runnerUser = New-LocalUser -Name $accountName -Password $password -AccountNeverExpires -PasswordNeverExpires -UserMayNotChangePassword -Description 'HyperGpuSupport fixed Hyper-V runner'
    [Array]::Clear($passwordBytes, 0, $passwordBytes.Length)
    $runnerSid = $runnerUser.SID.Value
    $runnerSidRecord = Join-Path $backupRoot 'runner-sid-v1.txt'
    [IO.File]::WriteAllText($runnerSidRecord, $runnerSid, (New-Object Text.UTF8Encoding($false)))
    & icacls.exe $runnerSidRecord /inheritance:r /grant:r '*S-1-5-18:F' '*S-1-5-32-544:F' | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Failed to secure authoritative runner SID record' }
    $enrollment = [ordered]@{ schema = 1; runner_sid = $runnerSid; client_sid = $clientSid }
    [IO.File]::WriteAllText($enrollmentTarget, ($enrollment | ConvertTo-Json), (New-Object Text.UTF8Encoding($false)))
    $hyperVAdministrators = Get-LocalGroup -SID ([Security.Principal.SecurityIdentifier]::new('S-1-5-32-578'))
    Add-LocalGroupMember -Group $hyperVAdministrators -Member $runnerUser

    Move-Item -LiteralPath $stagedRights -Destination $rightsTarget -Force
    & icacls.exe $rightsTarget /inheritance:r /grant:r '*S-1-5-18:F' '*S-1-5-32-544:F' | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Failed to secure account-right helper' }
    Assert-Hash $rightsTarget $rightsHash
    & $rightsTarget grant $runnerSid
    if ($LASTEXITCODE -ne 0) { throw 'Failed to apply exact runner account rights' }

    New-Item -ItemType Directory -Path (Join-Path $stateDirectory 'nonces') -Force | Out-Null
    & icacls.exe $installDirectory /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' "*$runnerSid`:(OI)(CI)RX" "*$clientSid`:(OI)(CI)RX" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Failed to secure runner executable directory' }
    & icacls.exe $dataDirectory /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' "*$runnerSid`:(OI)(CI)RX" "*$clientSid`:(OI)(CI)RX" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Failed to secure runner data root' }
    foreach ($path in $stateDirectory,$resultDirectory,$auditDirectory) {
        & icacls.exe $path /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' "*$runnerSid`:(OI)(CI)M" "*$clientSid`:(OI)(CI)R" | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "Failed to secure $path" }
    }
    & icacls.exe $parentDirectory /grant "*$runnerSid`:(OI)(CI)RX" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Failed to grant fixed parent read access' }
    & icacls.exe $childDirectory /grant "*$runnerSid`:(OI)(CI)M" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Failed to grant fixed child-directory modify access' }

    Disable-ScheduledTask -TaskPath $taskPath -TaskName $oldTaskName | Out-Null
    if ((Get-ScheduledTask -TaskPath $taskPath -TaskName $oldTaskName).State -eq 'Running') { throw 'Old reset task began running during migration; it remains disabled for recovery' }
    Move-Item -LiteralPath $stagedRunner -Destination $runnerTarget -Force
    Move-Item -LiteralPath $stagedClient -Destination $clientTarget -Force
    Move-Item -LiteralPath $stagedPolicy -Destination $policyTarget -Force
    foreach ($path in $runnerTarget,$clientTarget) {
        & icacls.exe $path /inheritance:r /grant:r '*S-1-5-18:F' '*S-1-5-32-544:F' "*$runnerSid`:RX" "*$clientSid`:RX" | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "Failed to secure executable $path" }
    }
    foreach ($path in $policyTarget,$enrollmentTarget) {
        & icacls.exe $path /inheritance:r /grant:r '*S-1-5-18:F' '*S-1-5-32-544:F' "*$runnerSid`:R" "*$clientSid`:R" | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "Failed to secure runner data file $path" }
    }
    Assert-Hash $runnerTarget $runnerHash
    Assert-Hash $clientTarget $clientHash
    Assert-Hash $rightsTarget $rightsHash
    Assert-Hash $policyTarget $policyHash

    $service = New-Object -ComObject 'Schedule.Service'
    $service.Connect()
    $folder = $service.GetFolder($taskPath.TrimEnd('\'))
    $definition = $service.NewTask(0)
    $definition.RegistrationInfo.Description = 'Fixed one-shot HyperGpuSupport runner v1'
    $definition.Principal.UserId = $qualifiedRunner
    $definition.Principal.LogonType = 2 # TASK_LOGON_S4U
    $definition.Principal.RunLevel = 0 # TASK_RUNLEVEL_LUA
    $definition.Settings.Enabled = $false
    $definition.Settings.ExecutionTimeLimit = $taskExecutionTimeLimit
    $definition.Settings.MultipleInstances = 2 # TASK_INSTANCES_IGNORE_NEW
    $definition.Settings.DisallowStartIfOnBatteries = $false
    $definition.Settings.StopIfGoingOnBatteries = $false
    $action = $definition.Actions.Create(0) # TASK_ACTION_EXEC
    $action.Path = $runnerTarget
    $action.Arguments = 'serve-once'
    $registered = $folder.RegisterTaskDefinition($taskName, $definition, 2, $qualifiedRunner, $passwordText, 2, $null)
    $passwordText = $null
    [IO.File]::WriteAllText(
        $progressLog,
        "task_path=$($registered.Path)`ntask_name=$($registered.Name)`nstate=$($registered.State)`n",
        (New-Object Text.UTF8Encoding($false))
    )
    $registered.SetSecurityDescriptor("D:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;GRGX;;;$clientSid)", 0)
    $registeredTask = Get-ScheduledTask -TaskPath $taskPath -TaskName $taskName -ErrorAction Stop
    if ($registeredTask.Settings.Enabled) { throw 'New runner task must remain disabled during verification' }
    $registeredExecutionLimit = [Xml.XmlConvert]::ToTimeSpan([string]$registeredTask.Settings.ExecutionTimeLimit)
    $expectedExecutionLimit = [Xml.XmlConvert]::ToTimeSpan($taskExecutionTimeLimit)
    if ($registeredExecutionLimit -ne $expectedExecutionLimit) { throw 'Runner task execution limit mismatch' }
    Assert-Hash $runnerTarget $runnerHash
    Assert-Hash $clientTarget $clientHash
    Assert-Hash $rightsTarget $rightsHash
    Assert-Hash $policyTarget $policyHash
    Unregister-ScheduledTask -TaskPath $taskPath -TaskName $oldTaskName -Confirm:$false
    Remove-Item -LiteralPath $stagingRoot -Recurse -Force
    Enable-ScheduledTask -TaskPath $taskPath -TaskName $taskName | Out-Null
} catch {
    $cause = $_
    $passwordText = $null
    foreach ($candidate in $taskName,$oldTaskName) {
        if (Get-ScheduledTask -TaskPath $taskPath -TaskName $candidate -ErrorAction SilentlyContinue) {
            Disable-ScheduledTask -TaskPath $taskPath -TaskName $candidate -ErrorAction SilentlyContinue | Out-Null
        }
    }
    throw "Runner installation stopped fail-closed. Do not trigger either task. Run scripts/setup/restore-runner-v1.ps1 from an administrator session to restore the captured preimage. Cause: $($cause.Exception.Message) Location: $($cause.InvocationInfo.PositionMessage) Stack: $($cause.ScriptStackTrace)"
}

[ordered]@{ schema = 1; runner_sha256 = $runnerHash; client_sha256 = $clientHash; rights_helper_sha256 = $rightsHash; policy_sha256 = $policyHash; runner_sid = $runnerSid; client_sid = $clientSid; task = "$taskPath$taskName"; backup = $backupRoot } | ConvertTo-Json
