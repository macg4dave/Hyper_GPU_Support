<# Compile and inspect isolated fixtures only; never launch Setup or a helper. #>
[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$IsccPath)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$builder = Join-Path $root 'scripts/build-installer.ps1'
$fixture = Join-Path ([System.IO.Path]::GetTempPath()) ('HyperGpuSupport-installer-tests-' + [guid]::NewGuid().ToString('N'))
$payload = Join-Path $fixture 'payload'
New-Item -ItemType Directory -Path (Join-Path $payload 'licenses') -Force | Out-Null
$names = @('hyper-gpu-support.exe', 'hyper-gpu-runner.exe', 'hyper-gpu-guest.exe', 'd3d11-probe.exe', 'app-install.exe')
foreach ($name in $names) {
    [System.IO.File]::WriteAllText((Join-Path $payload $name), "Isolated packaging fixture: $name")
}
[System.IO.File]::WriteAllText((Join-Path $payload 'LICENSE'), 'Fixture licence')
[System.IO.File]::WriteAllText((Join-Path $payload 'licenses/THIRD_PARTY_NOTICES.md'), 'Fixture notices')
$cargoJson = & cargo metadata --locked --format-version 1 --no-deps --manifest-path (Join-Path $root 'Cargo.toml')
if ($LASTEXITCODE -ne 0) { throw 'cargo metadata failed.' }
$cargoMetadata = $cargoJson | ConvertFrom-Json
$package = @($cargoMetadata.packages | Where-Object { $_.name -eq 'hyper-gpu-support' })[0]
$toolchainText = Get-Content -LiteralPath (Join-Path $root 'rust-toolchain.toml') -Raw
if ($toolchainText -notmatch '(?m)^channel\s*=\s*"([^"]+)"') { throw 'Missing toolchain.' }
$metadataPath = Join-Path $fixture 'build-metadata.json'
@{
    Version = $package.version; Target = 'x86_64-pc-windows-msvc'; Toolchain = $Matches[1]
    PayloadDirectory = $payload; UninstallerDirectory = ''; UninstallerCacheName = ''; SignedUninstallerSha256 = ''; SetupPath = ''
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
$sum = (Get-Content -LiteralPath (Join-Path $output 'SHA256SUMS.txt') -Raw).Trim()
$expectedHash = (Get-FileHash -LiteralPath $metadata.SetupPath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($sum -cne ($expectedHash + '  ' + [System.IO.Path]::GetFileName($metadata.SetupPath))) { throw 'Local installer checksum is incorrect.' }
if (@(Get-Content -LiteralPath (Join-Path $payload 'SHA256SUMS.txt')).Count -ne 5) { throw 'All five payload hashes are required.' }

$uninstallerOutput = Join-Path $fixture 'uninstaller-compile'
& $builder -Stage PrepareUninstaller -WorkDirectory $fixture -OutputDirectory $uninstallerOutput -IsccPath $IsccPath
$metadata = Get-Content -LiteralPath $metadataPath -Raw | ConvertFrom-Json
$uninstallers = @(Get-ChildItem -LiteralPath $metadata.UninstallerDirectory -File)
if ($uninstallers.Count -ne 1 -or $uninstallers[0].Extension -cne '.exe' -or $metadata.UninstallerCacheName -notmatch '\.e32$') {
    throw 'External uninstaller signing must have one executable and preserve its cache name.'
}
if (@(Get-ChildItem -LiteralPath $uninstallerOutput -Filter '*.exe' -File).Count -ne 0) { throw 'Uninstaller preparation must not produce a release installer.' }

$signedOutput = Join-Path $fixture 'signed-candidate'
Expect-Failure {
    & $builder -Stage Package -Signed -WorkDirectory $fixture -OutputDirectory $signedOutput -IsccPath $IsccPath
} 'Official packaging must reject missing signing configuration.'
if (@(Get-ChildItem -LiteralPath $signedOutput -Filter '*.exe' -File).Count -ne 0) { throw 'Signing configuration failure produced an installer.' }
Expect-Failure {
    & $builder -Stage Package -Signed -ExpectedPublisher 'CN=Untrusted fixture' -UninstallerPath $uninstallers[0].FullName -WorkDirectory $fixture -OutputDirectory $signedOutput -IsccPath $IsccPath
} 'Official packaging must reject unsigned payloads even with signing configuration.'
if (@(Get-ChildItem -LiteralPath $signedOutput -Filter '*.exe' -File).Count -ne 0) { throw 'Unsigned payload rejection produced an installer.' }

$missing = Join-Path $fixture 'missing-payload'
New-Item -ItemType Directory -Path $missing | Out-Null
Expect-Failure {
    & $builder -Stage Package -WorkDirectory $fixture -PayloadDirectory $missing -OutputDirectory (Join-Path $fixture 'missing-output') -IsccPath $IsccPath
} 'Packaging must reject incomplete payloads.'
Write-Host "Installer fixture checks passed. Fixtures remain isolated at $fixture"
