[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
. (Join-Path $repositoryRoot 'scripts\common\project-config.ps1')
$configuration = Import-ProjectConfiguration -Path (Join-Path $repositoryRoot 'config\project.toml')
$artifactPins = Import-ProjectConfiguration -Path (Join-Path $repositoryRoot 'config\artifact-pins.toml')

foreach ($key in @(
    'slot.name',
    'slot.vm_id',
    'slot.gpu_interface',
    'slot.parent_path',
    'slot.parent_sha256',
    'slot.child_path',
    'runner.data_directory',
    'tooling.cuda_samples_commit',
    'tests.host_probes.output_path'
)) {
    $value = Get-ProjectConfigurationValue -Configuration $configuration -Key $key
    if ($null -eq $value -or [string]::IsNullOrWhiteSpace([string] $value)) {
        throw "Project configuration key '$key' is empty."
    }
}

foreach ($key in @(
    'runner.runner_sha256',
    'runner.client_sha256',
    'runner.rights_sha256',
    'runner.policy_sha256'
)) {
    $value = [string](Get-ProjectConfigurationValue -Configuration $artifactPins -Key $key)
    if ($value -notmatch '^[0-9a-f]{64}$') {
        throw "Generated artifact pin '$key' is not a canonical SHA-256 value."
    }
}

$policy = Get-Content -LiteralPath (Join-Path $repositoryRoot 'config\runner-policy-v1.json') -Raw |
    ConvertFrom-Json
$comparisons = @{
    slot = 'slot.name'
    vm_id = 'slot.vm_id'
    vm_name = 'slot.vm_name'
    parent = 'slot.parent_path'
    parent_sha256 = 'slot.parent_sha256'
    child = 'slot.child_path'
    gpu_interface = 'slot.gpu_interface'
}
foreach ($field in $comparisons.Keys) {
    $expected = [string](Get-ProjectConfigurationValue -Configuration $configuration -Key $comparisons[$field])
    if ([string]$policy.$field -cne $expected) {
        throw "Generated runner policy field '$field' is stale relative to config/project.toml."
    }
}

& (Join-Path $repositoryRoot 'scripts\setup\update-project-pins.ps1') -Check -PolicyOnly

Write-Host "Project configuration parsed and runner policy identity fields match ($($configuration.Count) keys)."
