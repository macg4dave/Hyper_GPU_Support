<# Compile packages and exercise helper launching with isolated fixtures only. #>
[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$IsccPath)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
& (Join-Path $PSScriptRoot 'test-installer-helper.ps1') -IsccPath $IsccPath
if (-not $?) { throw 'Installer helper execution checks failed.' }
$builder = Join-Path $root 'scripts/build-installer.ps1'
$fixture = Join-Path ([System.IO.Path]::GetTempPath()) ('HyperGpuSupport-installer-tests-' + [guid]::NewGuid().ToString('N'))
$payload = Join-Path $fixture 'payload'
New-Item -ItemType Directory -Path (Join-Path $payload 'licenses') -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $payload 'licenses/nested') | Out-Null
$names = @('hyper-gpu-support.exe', 'hyper-gpu-runner.exe', 'hyper-gpu-guest.exe', 'd3d11-probe.exe', 'app-install.exe')
foreach ($name in $names) {
    [System.IO.File]::WriteAllText((Join-Path $payload $name), "Isolated packaging fixture: $name")
}
[System.IO.File]::WriteAllText((Join-Path $payload 'LICENSE'), 'Fixture licence')
[System.IO.File]::WriteAllText((Join-Path $payload 'licenses/THIRD_PARTY_NOTICES.md'), 'Fixture notices')
[System.IO.File]::WriteAllText((Join-Path $payload 'licenses/nested/LICENSE-MIT'), 'Nested fixture licence')
$cargoJson = & cargo metadata --locked --format-version 1 --no-deps --manifest-path (Join-Path $root 'Cargo.toml')
if ($LASTEXITCODE -ne 0) { throw 'cargo metadata failed.' }
$cargoMetadata = $cargoJson | ConvertFrom-Json
$package = @($cargoMetadata.packages | Where-Object { $_.name -eq 'hyper-gpu-support' })[0]
$toolchainText = Get-Content -LiteralPath (Join-Path $root 'rust-toolchain.toml') -Raw
if ($toolchainText -notmatch '(?m)^channel\s*=\s*"([^"]+)"') { throw 'Missing toolchain.' }
$metadataPath = Join-Path $fixture 'build-metadata.json'
@{
    Version = $package.version; Target = 'x86_64-pc-windows-msvc'; Toolchain = $Matches[1]
    PayloadDirectory = $payload; SetupPath = ''; BinaryZipPath = ''
} | ConvertTo-Json | Set-Content -LiteralPath $metadataPath -Encoding UTF8

function Expect-Failure {
    param([scriptblock]$Action, [string]$Message)
    $failed = $false
    try { & $Action } catch { $failed = $true }
    if (-not $failed) { throw $Message }
}

$output = Join-Path $fixture 'unsigned'
& $builder -Stage Package -WorkDirectory $fixture -OutputDirectory $output -IsccPath $IsccPath
$metadata = Get-Content -LiteralPath $metadataPath -Raw | ConvertFrom-Json
if ([System.IO.Path]::GetFileName($metadata.SetupPath) -cne "HyperGpuSupport-$($package.version)-windows-x64-unsigned-setup.exe") {
    throw 'Local installer must be explicitly labelled unsigned.'
}
if ([System.IO.Path]::GetFileName($metadata.BinaryZipPath) -cne "HyperGpuSupport-$($package.version)-windows-x64-unsigned-binaries.zip") {
    throw 'Binary ZIP must be explicitly labelled unsigned.'
}
$sums = @(Get-Content -LiteralPath (Join-Path $output 'SHA256SUMS.txt'))
if ($sums.Count -ne 2) { throw 'Both unsigned package checksums are required.' }
foreach ($path in @($metadata.SetupPath, $metadata.BinaryZipPath)) {
    $expectedHash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($sums -cnotcontains ($expectedHash + '  ' + [System.IO.Path]::GetFileName($path))) {
        throw "Package checksum is incorrect: $path"
    }
}
if (@(Get-Content -LiteralPath (Join-Path $payload 'SHA256SUMS.txt')).Count -ne 5) { throw 'All five payload hashes are required.' }
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [System.IO.Compression.ZipFile]::OpenRead($metadata.BinaryZipPath)
try {
    $entries = @($zip.Entries | Where-Object { $_.Name })
    $expectedPaths = @($names) + @('LICENSE', 'SHA256SUMS.txt', 'licenses/THIRD_PARTY_NOTICES.md', 'licenses/nested/LICENSE-MIT', 'install.txt')
    $actualPaths = @($entries | ForEach-Object { $_.FullName })
    if (@($actualPaths | Where-Object { $_.Contains('\') }).Count -ne 0) {
        throw 'Binary ZIP entry paths must use portable forward slashes.'
    }
    if (@(Compare-Object ($expectedPaths | Sort-Object) ($actualPaths | Sort-Object)).Count -ne 0) {
        throw 'Binary ZIP must contain the complete staged payload directly at its root.'
    }
    foreach ($entry in $entries) {
        $stream = $entry.Open()
        $sha = [System.Security.Cryptography.SHA256]::Create()
        try { $zipHash = ([BitConverter]::ToString($sha.ComputeHash($stream))).Replace('-', '') }
        finally { $sha.Dispose(); $stream.Dispose() }
        $source = if ($entry.FullName -ceq 'install.txt') { Join-Path $root 'packaging/install.txt' } else { Join-Path $payload $entry.FullName }
        $sourceHash = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
        if ($zipHash -cne $sourceHash) { throw "Binary ZIP changed staged content: $($entry.FullName)" }
    }
} finally { $zip.Dispose() }

# Local packaging must remain independent of signing accounts and configuration.
# These obsolete parameters/stages must fail during argument binding.
Expect-Failure { & $builder -Stage PrepareUninstaller } 'Removed signing stage is still accepted.'
Expect-Failure { & $builder -Signed } 'Removed signing switch is still accepted.'

$missing = Join-Path $fixture 'missing-payload'
New-Item -ItemType Directory -Path $missing | Out-Null
Expect-Failure {
    & $builder -Stage Package -WorkDirectory $fixture -PayloadDirectory $missing -OutputDirectory (Join-Path $fixture 'missing-output') -IsccPath $IsccPath
} 'Packaging must reject incomplete payloads.'
$unexpected = Join-Path $payload 'unexpected.txt'
[System.IO.File]::WriteAllText($unexpected, 'Must not enter only one package')
Expect-Failure {
    & $builder -Stage Package -WorkDirectory $fixture -OutputDirectory (Join-Path $fixture 'unexpected-output') -IsccPath $IsccPath
} 'Packaging must reject files the installer would omit from the binary ZIP.'
Remove-Item -LiteralPath $unexpected
$compilerFailureOutput = Join-Path $fixture 'missing-compiler-output'
Expect-Failure {
    & $builder -Stage Package -WorkDirectory $fixture -OutputDirectory $compilerFailureOutput -IsccPath (Join-Path $fixture 'missing-iscc.exe')
} 'Packaging must reject a missing compiler.'
if (@(Get-ChildItem -LiteralPath $compilerFailureOutput -File).Count -ne 0) { throw 'Compiler failure produced release packages.' }
Expect-Failure {
    & $builder -Stage Package -WorkDirectory $fixture -OutputDirectory (Join-Path $payload 'invalid-output') -IsccPath $IsccPath
} 'Packaging must reject output inside the binary ZIP source.'
if (Test-Path -LiteralPath (Join-Path $payload 'invalid-output')) { throw 'Invalid output created files inside the staged payload.' }
Write-Host "Installer fixture checks passed. Fixtures remain isolated at $fixture"
