#Requires -RunAsAdministrator
[CmdletBinding(SupportsShouldProcess, ConfirmImpact = 'High')]
param()

$ErrorActionPreference = 'Stop'
$runnerSource = 'C:\Users\dave\.cargo-target\text-game\x86_64-pc-windows-msvc\release\hyper-gpu-runner.exe'
$clientSource = 'C:\Users\dave\.cargo-target\text-game\x86_64-pc-windows-msvc\release\hyper-gpu-client.exe'
$policySource = 'C:\Users\dave\github\Hyper_GPU_Support\config\runner-policy-v1.json'
$runnerHash = '274ba2548dcc352a9591ea8e8806e21390a86bee16b95d5d64a103cbbc3ac1e6'
$clientHash = '649eccc2c937f5baa3ad838dbbcc3c0e8966f6246e69aae5b5ba278cde2b4a92'
$policyHash = 'b4178d0a5b72520c3769cf371a0da824d1434c909df4c25b769b7a26e8f5916f'
$installDirectory = 'C:\Program Files\HyperGpuSupport\Runner'
$dataDirectory = 'C:\ProgramData\HyperGpuSupport\Runner'
$runnerTarget = Join-Path $installDirectory 'hyper-gpu-runner.exe'
$clientTarget = Join-Path $installDirectory 'hyper-gpu-client.exe'
$policyTarget = Join-Path $dataDirectory 'policy-v1.json'
$enrollmentTarget = Join-Path $dataDirectory 'enrollment-v1.json'
$accountName = 'HyperGpuRunner'
$taskPath = '\HyperGpuSupport\'
$taskName = 'Runner-v1'
$oldTaskPath = '\HyperGpuSupport\'
$oldTaskName = 'ResetSlot-v1'
$parentDirectory = 'Z:\HyperGpuSupport\images\golden\win11-pro-25h2-26200.9457-x64-v1'
$backupRoot = Join-Path $dataDirectory 'install-backup-v1'

function Assert-Hash([string]$Path, [string]$Expected) {
    $actual = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $Expected) { throw "Hash mismatch for $Path (actual $actual)" }
}

foreach ($item in @(
    @{ Path = $runnerSource; Hash = $runnerHash },
    @{ Path = $clientSource; Hash = $clientHash },
    @{ Path = $policySource; Hash = $policyHash }
)) {
    Assert-Hash $item.Path $item.Hash
}

$clientSid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$qualifiedRunner = "$env:COMPUTERNAME\$accountName"
$existing = Get-LocalUser -Name $accountName -ErrorAction SilentlyContinue
if ($null -ne $existing) { throw "Refusing to reuse existing local account $qualifiedRunner" }
if ((Get-ScheduledTask -TaskPath $taskPath -TaskName $taskName -ErrorAction SilentlyContinue)) {
    throw "Refusing to replace existing task $taskPath$taskName"
}
if (Test-Path -LiteralPath $backupRoot) { throw "Install backup already exists: $backupRoot" }

$effect = "install fixed runner/client/policy/enrollment, create $qualifiedRunner in Hyper-V Administrators, replace only $oldTaskPath$oldTaskName, grant the runner read access to the fixed parent, and register $taskPath$taskName"
if (-not $PSCmdlet.ShouldProcess('one enrolled GPU-PV runner slot', $effect)) { return }

New-Item -ItemType Directory -Path $backupRoot -Force | Out-Null
if (Test-Path -LiteralPath $runnerTarget) { Copy-Item -LiteralPath $runnerTarget -Destination (Join-Path $backupRoot 'hyper-gpu-runner.exe') }
if (Test-Path -LiteralPath $policyTarget) { Copy-Item -LiteralPath $policyTarget -Destination (Join-Path $backupRoot 'policy-v1.json') }
$oldTask = Get-ScheduledTask -TaskPath $oldTaskPath -TaskName $oldTaskName -ErrorAction SilentlyContinue
if ($null -ne $oldTask) {
    Export-ScheduledTask -TaskPath $oldTaskPath -TaskName $oldTaskName | Set-Content -LiteralPath (Join-Path $backupRoot 'ResetSlot-v1.xml') -Encoding Unicode
}

$passwordBytes = New-Object byte[] 48
$random = [Security.Cryptography.RandomNumberGenerator]::Create()
$random.GetBytes($passwordBytes)
$random.Dispose()
$passwordText = [Convert]::ToBase64String($passwordBytes)
$password = ConvertTo-SecureString $passwordText -AsPlainText -Force
$runnerUser = New-LocalUser -Name $accountName -Password $password -AccountNeverExpires -PasswordNeverExpires -UserMayNotChangePassword -Description 'HyperGpuSupport fixed Hyper-V runner'
$passwordText = $null
[Array]::Clear($passwordBytes, 0, $passwordBytes.Length)
Add-LocalGroupMember -Group 'Hyper-V Administrators' -Member $runnerUser
$runnerSid = $runnerUser.SID.Value

New-Item -ItemType Directory -Path $installDirectory,$dataDirectory -Force | Out-Null
foreach ($leaf in 'state','state\nonces','results','audit') {
    New-Item -ItemType Directory -Path (Join-Path $dataDirectory $leaf) -Force | Out-Null
}
Copy-Item -LiteralPath $runnerSource -Destination $runnerTarget -Force
Copy-Item -LiteralPath $clientSource -Destination $clientTarget -Force
Copy-Item -LiteralPath $policySource -Destination $policyTarget -Force
$enrollment = [ordered]@{ schema = 1; runner_sid = $runnerSid; client_sid = $clientSid }
$enrollmentJson = $enrollment | ConvertTo-Json
[IO.File]::WriteAllText($enrollmentTarget, $enrollmentJson, (New-Object Text.UTF8Encoding($false)))

& icacls.exe $installDirectory /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' "*$runnerSid`:(OI)(CI)RX" "*$clientSid`:(OI)(CI)RX" | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Failed to secure runner executable directory' }
& icacls.exe $dataDirectory /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' "*$runnerSid`:(OI)(CI)RX" "*$clientSid`:(OI)(CI)RX" | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Failed to secure runner data root' }
foreach ($leaf in 'state','results','audit') {
    $path = Join-Path $dataDirectory $leaf
    & icacls.exe $path /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' "*$runnerSid`:(OI)(CI)M" "*$clientSid`:(OI)(CI)R" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Failed to secure $path" }
}
& icacls.exe $parentDirectory /grant "*$runnerSid`:(OI)(CI)RX" | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Failed to grant fixed parent read access' }

$action = New-ScheduledTaskAction -Execute $runnerTarget -Argument 'serve-once'
$principal = New-ScheduledTaskPrincipal -UserId $qualifiedRunner -LogonType S4U -RunLevel Limited
$settings = New-ScheduledTaskSettingsSet -ExecutionTimeLimit (New-TimeSpan -Minutes 6) -MultipleInstances IgnoreNew -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
Register-ScheduledTask -TaskPath $taskPath -TaskName $taskName -Action $action -Principal $principal -Settings $settings -Description 'Fixed one-shot HyperGpuSupport runner v1' | Out-Null

$service = New-Object -ComObject 'Schedule.Service'
$service.Connect()
$folder = $service.GetFolder($taskPath.TrimEnd('\'))
$registered = $folder.GetTask($taskName)
$registered.SetSecurityDescriptor("D:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;GRGX;;;$clientSid)", 0)
if ($null -ne $oldTask) { Unregister-ScheduledTask -TaskPath $oldTaskPath -TaskName $oldTaskName -Confirm:$false }

Assert-Hash $runnerTarget $runnerHash
Assert-Hash $clientTarget $clientHash
Assert-Hash $policyTarget $policyHash
$result = [ordered]@{
    schema = 1
    runner_sha256 = $runnerHash
    client_sha256 = $clientHash
    policy_sha256 = $policyHash
    runner_sid = $runnerSid
    client_sid = $clientSid
    task = "$taskPath$taskName"
    backup = $backupRoot
}
$result | ConvertTo-Json
