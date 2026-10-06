#Requires -RunAsAdministrator
# Privilege: elevated; Rust owns target guards, guest effects and workload validation.
[CmdletBinding()]
param([switch]$KeepOpen, [string]$PreparedRun)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Set-Location -LiteralPath $root
. (Join-Path $root 'scripts\common\project-config.ps1')
$config = Import-ProjectConfiguration
$pins = Import-ProjectConfiguration -Path (Join-Path $root 'config\artifact-pins.toml')
function Config([string]$Key) { Get-ProjectConfigurationValue $config $Key }
$client = Join-Path (Config 'runner.install_directory') 'hyper-gpu-client.exe'
$artifacts = Config 'runner.artifacts.directory'
$stage = Join-Path $artifacts 'hyper-gpu-stage.exe'
$cli = Join-Path $artifacts 'hyper-gpu-support.exe'
$outputRoot = Join-Path $root (Config 'paths.test_output')
$run = Join-Path $outputRoot ('gpu006-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffZ'))
New-Item -ItemType Directory -Path $run | Out-Null
Copy-Item -LiteralPath $PSCommandPath -Destination (Join-Path $run 'qualification-harness.ps1')
$statusPath = Join-Path $outputRoot 'gpu006-live.status.json'
$report = [ordered]@{
    schema = 1; task = 'GPU-006'; success = $false; phase = 'preflight'
    started_utc = [DateTime]::UtcNow.ToString('o'); completed_utc = $null
    revision = [string](& git rev-parse HEAD)
    working_diff_sha256 = $null
    harness_sha256 = (Get-FileHash (Join-Path $run 'qualification-harness.ps1')).Hash.ToLowerInvariant()
    configuration_sha256 = (Get-FileHash (Join-Path $root 'config\project.toml')).Hash.ToLowerInvariant()
    vm_id = Config 'slot.vm_id'; gpu_interface = Config 'slot.gpu_interface'
    parent_sha256 = Config 'slot.parent_sha256'
    artifacts = @(); operations = @(); staging = $null; reapply = $null
    validation = $null; diagnostic = $null
    preparation_report_sha256 = $null
}
& git diff --binary HEAD | Set-Content -LiteralPath (Join-Path $run 'working.diff') -Encoding UTF8
$report.working_diff_sha256 = (Get-FileHash (Join-Path $run 'working.diff')).Hash.ToLowerInvariant()

function Save-Report([string]$Phase) {
    $report.phase = $Phase
    $report | ConvertTo-Json -Depth 40 | Set-Content -LiteralPath (Join-Path $run 'report.json') -Encoding UTF8
    @{phase=$Phase; success=$report.success; report=(Join-Path $run 'report.json'); diagnostic=$report.diagnostic} |
        ConvertTo-Json | Set-Content -LiteralPath $statusPath -Encoding UTF8
}

function Invoke-Runner([string]$Operation) {
    Save-Report $Operation
    Write-Host "Fixed Rust runner: $Operation"
    $lines = @(& $client $Operation | Tee-Object -FilePath (Join-Path $run ("$($report.operations.Count)-$Operation.log")))
    if ($LASTEXITCODE -ne 0) { throw "Runner operation failed: $Operation; reconcile retained audit/results before retry." }
    # ensure-gpu verifies its result internally and prints key/value output.
    if ($Operation -eq 'ensure-gpu') {
        $report.operations += @{operation=$Operation; output=$lines}
        return
    }
    $response = @{}
    foreach ($line in $lines) {
        if ($line -match '^([a-z_]+)\t(.+)$') { $response[$Matches[1]] = $Matches[2] }
    }
    if ($response.status -cne 'succeeded' -or [string]$response.operation_id -notmatch '^\d+-\d+$') {
        throw 'Invalid successful runner response.'
    }
    $resultPath = Join-Path (Config 'runner.data_directory') ("results\$($response.operation_id).json")
    if ((Get-Item -LiteralPath $resultPath).Length -gt 65536) { throw 'Runner result exceeds protocol bound.' }
    $result = Get-Content -LiteralPath $resultPath -Raw | ConvertFrom-Json
    if ($result.request_id -cne $response.request_id -or $result.operation_id -cne $response.operation_id -or
        $result.operation -cne $Operation -or $result.status -cne 'succeeded') { throw 'Runner result identity mismatch.' }
    $report.operations += $result
    Save-Report $Operation
    return $result
}

function Invoke-Staging([string]$Expected) {
    Save-Report "staging-$Expected"
    Write-Host 'Enter guest username now, then password at the Rust masked prompt. Credentials are never captured.'
    $stderr = Join-Path $run "staging-$Expected.stderr.txt"
    # rpassword uses CONOUT$/CONIN$ directly; redirecting stderr retains native
    # diagnostics without capturing its password prompt or password feedback.
    $priorPreference = $ErrorActionPreference
    try {
        # Windows PowerShell 5.1 wraps redirected native stderr as error records,
        # including ordinary progress. The native exit code is authoritative.
        $ErrorActionPreference = 'Continue'
        $lines = @(& $stage apply --interactive 2> $stderr | Tee-Object -FilePath (Join-Path $run "staging-$Expected.log"))
        $nativeExit = $LASTEXITCODE
    } finally { $ErrorActionPreference = $priorPreference }
    if ($nativeExit -ne 0) { throw "Rust staging failed: $Expected. See $stderr" }
    $values = [ordered]@{}
    foreach ($line in $lines) {
        if ($line -match '^([^=]+)=(.*)$') {
            # Multiple pending-delete entries are recorded in the original output.
            $values[$Matches[1]] = $Matches[2]
        }
    }
    if ($values.status -cne $Expected -or $values.vm_id -cne (Config 'slot.vm_id') -or
        $values.manifest_sha256 -notmatch '^[0-9a-f]{64}$' -or [uint64]$values.files -eq 0) {
        throw 'Unexpected staging result.'
    }
    return $values
}

try {
    Save-Report 'preflight'
    if ((Get-FileHash -LiteralPath $client).Hash.ToLowerInvariant() -cne
        (Get-ProjectConfigurationValue $pins 'runner.client_sha256')) { throw 'Installed client pin mismatch.' }
    foreach ($executable in @($client,$stage,$cli,(Join-Path $artifacts 'hyper-gpu-validation-worker.exe'))) {
        $item = Get-Item -LiteralPath $executable
        $report.artifacts += @{name=$item.Name; bytes=$item.Length; sha256=(Get-FileHash -LiteralPath $executable).Hash.ToLowerInvariant()}
    }
    if ($PreparedRun) {
        $prepared = Get-Content -LiteralPath $PreparedRun -Raw | ConvertFrom-Json
        if ($prepared.schema -ne 1 -or $prepared.task -cne 'GPU-006' -or
            $prepared.configuration_sha256 -cne $report.configuration_sha256 -or $prepared.vm_id -cne $report.vm_id -or
            $null -ne $prepared.staging -or $null -ne $prepared.validation -or
            @($prepared.operations | Where-Object { $_.operation -eq 'reset-slot' -and $_.status -eq 'succeeded' }).Count -ne 1 -or
            $prepared.operations[-1].operation -cne 'start-slot' -or $prepared.operations[-1].status -cne 'succeeded' -or
            $prepared.operations[-1].state -cne 'Running' -or
            $prepared.operations[-1].gpu_adapters -ne 0) { throw 'Resume requires a matching clean-child preparation report without a staging receipt.' }
        Copy-Item -LiteralPath $PreparedRun -Destination (Join-Path $run 'preparation-report.json')
        $report.preparation_report_sha256 = (Get-FileHash (Join-Path $run 'preparation-report.json')).Hash.ToLowerInvariant()
        $report.operations = @($prepared.operations)
    }
    $initial = Invoke-Runner 'inspect'
    if ($initial.vm_id -cne (Config 'slot.vm_id') -or $initial.child -ine (Config 'slot.child_path') -or
        $initial.parent -ine (Config 'slot.parent_path') -or $initial.parent_sha256 -cne (Config 'slot.parent_sha256') -or
        $initial.gpu_interface -ine (Config 'slot.gpu_interface')) { throw 'Initial enrolled target mismatch.' }
    if ($PreparedRun) {
        if ($initial.state -cne 'Running' -or $initial.gpu_adapters -ne 0) { throw 'Prepared child must still be running without a GPU.' }
        # Rust rechecks its exclusive staging lock, prior receipt and every byte.
        # An uncertain/partial earlier write is refused and requires recreation.
    } else {
        if ($initial.state -eq 'Running') { $null = Invoke-Runner 'shutdown-slot' }
        elseif ($initial.state -ne 'Off') { throw 'Initial VM state is unsafe.' }
        $null = Invoke-Runner 'remove-gpu'
        $null = Invoke-Runner 'reset-slot'
        $null = Invoke-Runner 'start-slot'
    }
    $report.staging = Invoke-Staging 'applied'
    $report.reapply = Invoke-Staging 'already-applied'
    foreach ($key in @('manifest_sha256','files','bytes','vm_id','computer_name','machine_guid','windows_root')) {
        if ($report.staging[$key] -cne $report.reapply[$key]) { throw "Staging reapply differs: $key." }
    }
    $null = Invoke-Runner 'shutdown-slot'
    $null = Invoke-Runner 'ensure-gpu'
    $null = Invoke-Runner 'configure-slot'
    $settingsReapply = Invoke-Runner 'configure-slot'
    if ($settingsReapply.settings_status -cne 'already-applied') { throw 'Settings reapply was not a verified no-op.' }
    $null = Invoke-Runner 'ensure-gpu'
    $null = Invoke-Runner 'start-slot'
    Save-Report 'validation-credentials-and-workloads'
    Write-Host 'Enter guest username now, then password at the Rust prompt for validation.'
    $priorPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        $validationText = @(& $cli validate 2> (Join-Path $run 'validation.stderr.txt') | Tee-Object -FilePath (Join-Path $run 'validation.json'))
        $validationExit = $LASTEXITCODE
    } finally { $ErrorActionPreference = $priorPreference }
    $report.validation = ($validationText -join "`n") | ConvertFrom-Json
    Save-Report 'validation-result'
    if ($validationExit -ne 0) { throw 'Public Rust validation did not pass every essential check; see the per-check report.' }
    $null = Invoke-Runner 'shutdown-slot'
    $final = Invoke-Runner 'inspect'
    if ($final.state -cne 'Off' -or $final.gpu_adapters -ne 1) { throw 'Graceful shutdown final readback mismatch.' }
    $report.success = $true
    $report.completed_utc = [DateTime]::UtcNow.ToString('o')
    Save-Report 'completed'
    Write-Host "GPU-006 passed. Report: $run\report.json" -ForegroundColor Green
} catch {
    $report.diagnostic = $_.Exception.Message
    $report.completed_utc = [DateTime]::UtcNow.ToString('o')
    Save-Report 'failed'
    Write-Host "$($report.diagnostic) Report: $run\report.json" -ForegroundColor Red
    if ($KeepOpen) { $null = Read-Host 'Press Enter to close' }
    exit 1
}
if ($KeepOpen) { $null = Read-Host 'Press Enter to close' }
