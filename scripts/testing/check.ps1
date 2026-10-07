# Privilege: non-elevated.
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Invoke-CheckedCommand {
    param(
        [Parameter(Mandatory)]
        [string] $FilePath,

        [Parameter(Mandatory)]
        [string[]] $ArgumentList
    )

    Write-Host "> $FilePath $($ArgumentList -join ' ')"
    & $FilePath @ArgumentList
    if ($LASTEXITCODE -ne 0) {
        throw "Command '$FilePath' failed with exit code $LASTEXITCODE."
    }
}

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$oldRustFlags = $env:RUSTFLAGS
$oldRustDocFlags = $env:RUSTDOCFLAGS

Push-Location $repositoryRoot
try {
    $env:RUSTFLAGS = '-Dwarnings'
    $env:RUSTDOCFLAGS = '-Dwarnings'

    Invoke-CheckedCommand -FilePath 'cargo' -ArgumentList @('fmt', '--all', '--', '--check')
    # Root workspace contains only the product. The laboratory has separate gates.
    Invoke-CheckedCommand -FilePath 'cargo' -ArgumentList @('clippy', '--locked', '--workspace', '--all-targets', '--', '-D', 'warnings')
    Invoke-CheckedCommand -FilePath 'cargo' -ArgumentList @('test', '--locked', '--workspace')
    Invoke-CheckedCommand -FilePath 'cargo' -ArgumentList @('build', '--locked', '--workspace', '--all-features')
    Invoke-CheckedCommand -FilePath 'cargo' -ArgumentList @('doc', '--locked', '--workspace', '--all-features', '--no-deps')
    & (Join-Path $repositoryRoot 'scripts\testing\check-docs.ps1')
    if ($LASTEXITCODE -ne 0) {
        throw "Documentation check failed with exit code $LASTEXITCODE."
    }
}
catch {
    Write-Error $_ -ErrorAction Continue
    exit 1
}
finally {
    $env:RUSTFLAGS = $oldRustFlags
    $env:RUSTDOCFLAGS = $oldRustDocFlags
    Pop-Location
}
