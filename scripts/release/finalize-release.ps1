[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$WorkDirectory,
    [Parameter(Mandatory)][string]$SignedSetupDirectory,
    [Parameter(Mandatory)][string]$SignedUninstallerPath,
    [Parameter(Mandatory)][string]$ExpectedPublisher,
    [Parameter(Mandatory)][string]$UnpackerPath,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [string]$SignToolPath
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'release-common.ps1')
$metadata = Get-ReleaseMetadata $WorkDirectory
$setupName = "HyperGpuSupport-$($metadata.Version)-windows-x64-setup.exe"
Assert-ExactFiles $SignedSetupDirectory @($setupName)
$signedSetup = Join-Path $SignedSetupDirectory $setupName
Assert-SigningOnlyChange $metadata.SetupPath $signedSetup
& (Join-Path $PSScriptRoot 'verify-signatures.ps1') -Path $signedSetup -ExpectedPublisher $ExpectedPublisher -ExpectedVersion $metadata.Version -SignToolPath $SignToolPath
$payload = Join-Path $WorkDirectory 'signed-payload'
$extraction = Join-Path $WorkDirectory 'verified-extraction'
if (Test-Path -LiteralPath $extraction) { throw 'Extraction directory already exists; use a fresh work directory.' }
if (-not (Test-Path -LiteralPath $UnpackerPath -PathType Leaf)) { throw 'Pinned innounp is required.' }
& $UnpackerPath -x -m -b -y -a ('-d' + $extraction) $signedSetup
if ($LASTEXITCODE -ne 0) { throw "Non-executing installer extraction failed: $LASTEXITCODE" }
$app = Join-Path $extraction '{app}'
$payloadFiles = @(Get-ChildItem -LiteralPath $payload -Recurse -File | ForEach-Object { $_.FullName.Substring($payload.Length + 1).Replace('\', '/') })
Assert-ExactFiles $app $payloadFiles
foreach ($relative in $payloadFiles) {
    $expected = (Get-FileHash -LiteralPath (Join-Path $payload $relative) -Algorithm SHA256).Hash
    $actual = (Get-FileHash -LiteralPath (Join-Path $app $relative) -Algorithm SHA256).Hash
    if ($expected -cne $actual) { throw "Embedded installer payload differs: $relative" }
}
& (Join-Path $PSScriptRoot 'verify-signatures.ps1') -Path @($script:ProductExecutables | ForEach-Object { Join-Path $app $_ }) -ExpectedPublisher $ExpectedPublisher -ExpectedVersion $metadata.Version -SignToolPath $SignToolPath
$uninstallerHash = (Get-FileHash -LiteralPath $SignedUninstallerPath -Algorithm SHA256).Hash
if ($metadata.SignedUninstallerSha256 -cne $uninstallerHash) { throw 'Signed uninstaller differs from the cache supplied to the compiler.' }
$uninstallerName = [IO.Path]::GetFileName($SignedUninstallerPath)
$originalUninstaller = Join-Path $metadata.UninstallerDirectory $uninstallerName
Assert-SigningOnlyChange $originalUninstaller $SignedUninstallerPath
$uninstallerVersion = [Diagnostics.FileVersionInfo]::GetVersionInfo($originalUninstaller).FileVersion.Trim()
& (Join-Path $PSScriptRoot 'verify-signatures.ps1') -Path $SignedUninstallerPath -ExpectedPublisher $ExpectedPublisher -ExpectedVersion $metadata.Version -ExpectedFileVersion $uninstallerVersion -SignToolPath $SignToolPath
# Inno reconstructs its uninstaller from the signed runtime cache when installing.
# innounp does not expose that runtime. The pinned compiler enforces its cache
# identity; Package records its unchanged signed input hash. The outer PE-content
# comparison above protects the resulting compiler output through outer signing.
# dontcopy files are used by the setup helper before installation. Validate
# every executable copy as well as the normal application-directory payload.
$temporary = Join-Path $extraction '{tmp}'
Assert-ExactFiles $temporary (@($script:ProductExecutables) + @('SHA256SUMS.txt'))
foreach ($temporaryName in @($script:ProductExecutables) + @('SHA256SUMS.txt')) {
    if ((Get-FileHash -LiteralPath (Join-Path $temporary $temporaryName) -Algorithm SHA256).Hash -cne (Get-FileHash -LiteralPath (Join-Path $payload $temporaryName) -Algorithm SHA256).Hash) {
        throw "Embedded temporary file differs: $temporaryName"
    }
}
foreach ($file in Get-ChildItem -LiteralPath $extraction -Recurse -File -Filter '*.exe') {
    if ($script:ProductExecutables -cnotcontains $file.Name) { throw "Unexpected executable embedded in Setup: $($file.FullName)" }
    if ((Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash -cne (Get-FileHash -LiteralPath (Join-Path $payload $file.Name) -Algorithm SHA256).Hash) {
        throw "Embedded temporary executable differs: $($file.Name)"
    }
}
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Release output already exists; never reuse a previous run output.' }
New-Item -ItemType Directory -Path $OutputDirectory | Out-Null
Copy-Item -LiteralPath $signedSetup -Destination (Join-Path $OutputDirectory $setupName)
$hash = (Get-FileHash -LiteralPath (Join-Path $OutputDirectory $setupName) -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText((Join-Path $OutputDirectory 'SHA256SUMS.txt'), "$hash  $setupName`n", [Text.UTF8Encoding]::new($false))
Write-Host "Release-ready signed installer and final checksum: $OutputDirectory"
