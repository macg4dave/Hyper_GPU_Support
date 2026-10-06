# Privilege: non-elevated.
[CmdletBinding()]
param(
    [string] $ProjectConfigurationPath = (Join-Path $PSScriptRoot '..\..\config\project.toml'),
    [string] $ArtifactPinsPath = (Join-Path $PSScriptRoot '..\..\config\artifact-pins.toml'),
    [switch] $Check,
    [switch] $PolicyOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
. (Join-Path $repositoryRoot 'scripts\common\project-config.ps1')
$resolvedConfigurationPath = (Resolve-Path -LiteralPath $ProjectConfigurationPath).Path
$configuration = Import-ProjectConfiguration -Path $resolvedConfigurationPath
$resolvedArtifactPinsPath = (Resolve-Path -LiteralPath $ArtifactPinsPath).Path
$artifactPins = Import-ProjectConfiguration -Path $resolvedArtifactPinsPath
$policyPath = Join-Path $repositoryRoot 'config\runner-policy-v1.json'
$utf8NoBom = New-Object Text.UTF8Encoding($false)

function ConvertTo-JsonString([string] $Value) {
    return ($Value | ConvertTo-Json -Compress)
}

$allowedOperations = @(
    'inspect',
    'reset-slot',
    'start-slot',
    'shutdown-slot',
    'assign-gpu',
    'remove-gpu'
    'configure-slot'
)
$operationJson = ($allowedOperations | ForEach-Object { ConvertTo-JsonString $_ }) -join ', '
$policyText = @(
    '{'
    '  "schema": 1,'
    ('  "slot": {0},' -f (ConvertTo-JsonString ([string](Get-ProjectConfigurationValue $configuration 'slot.name'))))
    ('  "vm_id": {0},' -f (ConvertTo-JsonString ([string](Get-ProjectConfigurationValue $configuration 'slot.vm_id'))))
    ('  "vm_name": {0},' -f (ConvertTo-JsonString ([string](Get-ProjectConfigurationValue $configuration 'slot.vm_name'))))
    ('  "parent": {0},' -f (ConvertTo-JsonString ([string](Get-ProjectConfigurationValue $configuration 'slot.parent_path'))))
    ('  "parent_sha256": {0},' -f (ConvertTo-JsonString ([string](Get-ProjectConfigurationValue $configuration 'slot.parent_sha256'))))
    ('  "child": {0},' -f (ConvertTo-JsonString ([string](Get-ProjectConfigurationValue $configuration 'slot.child_path'))))
    ('  "gpu_interface": {0},' -f (ConvertTo-JsonString ([string](Get-ProjectConfigurationValue $configuration 'slot.gpu_interface'))))
    '  "vm_profile": {'
    ('    "memory_bytes": {0},' -f (Get-ProjectConfigurationValue $configuration 'vm_profile.memory_bytes'))
    ('    "processors": {0},' -f (Get-ProjectConfigurationValue $configuration 'vm_profile.processors'))
    ('    "low_mmio_bytes": {0},' -f (Get-ProjectConfigurationValue $configuration 'vm_profile.low_mmio_bytes'))
    ('    "high_mmio_bytes": {0},' -f (Get-ProjectConfigurationValue $configuration 'vm_profile.high_mmio_bytes'))
    ('    "guest_controlled_cache_types": {0},' -f ([string](Get-ProjectConfigurationValue $configuration 'vm_profile.guest_controlled_cache_types')).ToLowerInvariant())
    ('    "expose_virtualization_extensions": {0},' -f ([string](Get-ProjectConfigurationValue $configuration 'vm_profile.expose_virtualization_extensions')).ToLowerInvariant())
    ('    "checkpoints_disabled": {0},' -f ([string](Get-ProjectConfigurationValue $configuration 'vm_profile.checkpoints_disabled')).ToLowerInvariant())
    ('    "automatic_stop_guest_shutdown": {0}' -f ([string](Get-ProjectConfigurationValue $configuration 'vm_profile.automatic_stop_guest_shutdown')).ToLowerInvariant())
    '  },'
    '  "resources": {'
    foreach ($resource in @('vram', 'encode', 'decode', 'compute')) {
        $triple = ([string](Get-ProjectConfigurationValue $configuration "resources.$resource")).Split(',')
        if ($triple.Count -ne 3 -or @($triple | Where-Object { $_ -notmatch '^\d+$' }).Count -ne 0) { throw 'Explicit integer resource triples required.' }
        $suffix = if ($resource -eq 'compute') { '' } else { ',' }
        '    "{0}": {{"minimum": {1}, "maximum": {2}, "optimal": {3}}}{4}' -f $resource, $triple[0], $triple[1], $triple[2], $suffix
    }
    '  },'
    ('  "allowed_operations": [{0}]' -f $operationJson)
    '}'
) -join "`n"
$policyText += "`n"

if ($Check) {
    if ((Get-Content -LiteralPath $policyPath -Raw) -cne $policyText) {
        throw 'config/runner-policy-v1.json is stale relative to config/project.toml.'
    }
} else {
    [IO.File]::WriteAllText($policyPath, $policyText, $utf8NoBom)
}

$expectedHashes = [ordered]@{
    policy_sha256 = (Get-FileHash -LiteralPath $policyPath -Algorithm SHA256).Hash.ToLowerInvariant()
}
if (-not $PolicyOnly) {
    $artifactDirectory = [string](Get-ProjectConfigurationValue $configuration 'runner.artifacts.directory')
    foreach ($artifact in @(
        @{ Key = 'runner_sha256'; File = 'hyper-gpu-runner.exe' },
        @{ Key = 'client_sha256'; File = 'hyper-gpu-client.exe' },
        @{ Key = 'rights_sha256'; File = 'hyper-gpu-rights.exe' }
    )) {
        $artifactPath = Join-Path $artifactDirectory $artifact.File
        if (-not (Test-Path -LiteralPath $artifactPath -PathType Leaf)) {
            throw "Required release artifact is missing: $artifactPath"
        }
        $expectedHashes[$artifact.Key] = (Get-FileHash -LiteralPath $artifactPath -Algorithm SHA256).Hash.ToLowerInvariant()
    }
}

$artifactPinsText = [IO.File]::ReadAllText($resolvedArtifactPinsPath)
foreach ($key in $expectedHashes.Keys) {
    $current = [string](Get-ProjectConfigurationValue $artifactPins "runner.$key")
    $expected = [string]$expectedHashes[$key]
    if ($Check) {
        if ($current -cne $expected) {
            throw "Project pin '$key' is stale: configured $current, actual $expected."
        }
        continue
    }
    $pattern = '(?m)^' + [regex]::Escape($key) + ' = "[0-9a-f]{64}"$'
    $replacement = $key + ' = "' + $expected + '"'
    $updated = [regex]::Replace($artifactPinsText, $pattern, $replacement)
    if ($updated -ceq $artifactPinsText -and $current -cne $expected) {
        throw "Could not update project pin '$key'."
    }
    $artifactPinsText = $updated
}
if (-not $Check) {
    [IO.File]::WriteAllText($resolvedArtifactPinsPath, $artifactPinsText, $utf8NoBom)
    Write-Host "Updated generated runner policy and $($expectedHashes.Count) artifact pin(s)."
} else {
    Write-Host "Generated runner policy and $($expectedHashes.Count) artifact pin(s) are current."
}
