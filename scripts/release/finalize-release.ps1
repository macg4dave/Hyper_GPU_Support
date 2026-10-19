[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$WorkDirectory,
    [Parameter(Mandatory)][string]$UnpackerPath,
    [Parameter(Mandatory)][string]$OutputDirectory
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'release-common.ps1')
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Release output already exists; never reuse a previous run output.' }
$metadata = Get-ReleaseMetadata $WorkDirectory
$setupName = "HyperGpuSupport-$($metadata.Version)-windows-x64-unsigned-setup.exe"
$zipName = "HyperGpuSupport-$($metadata.Version)-windows-x64-unsigned-binaries.zip"
$candidate = Split-Path -Parent ([IO.Path]::GetFullPath($metadata.SetupPath))
if ([IO.Path]::GetFullPath($metadata.SetupPath) -cne (Join-Path $candidate $setupName) -or
    [IO.Path]::GetFullPath($metadata.BinaryZipPath) -cne (Join-Path $candidate $zipName)) { throw 'Package paths do not match the expected unsigned release names and directory.' }
Assert-ExactFiles $candidate @($setupName, $zipName, 'SHA256SUMS.txt')
Assert-FileHashes $candidate @($setupName, $zipName)
$payload = (Get-Item -LiteralPath $metadata.PayloadDirectory).FullName
$payloadFiles = @(Get-ArtifactFiles $payload)
foreach ($required in @($script:ProductExecutables) + @('LICENSE', 'licenses/THIRD_PARTY_NOTICES.md', 'SHA256SUMS.txt')) {
    if ($payloadFiles -cnotcontains $required) { throw "Missing required payload file: $required" }
}
$executables = @($payloadFiles | Where-Object { $_.EndsWith('.exe', [StringComparison]::OrdinalIgnoreCase) })
if ($executables.Count -ne $script:ProductExecutables.Count -or @(Compare-Object @($script:ProductExecutables | Sort-Object) @($executables | Sort-Object) -CaseSensitive).Count -ne 0) {
    throw 'Payload must contain exactly the five product and maintenance executables.'
}
Assert-FileHashes $payload $script:ProductExecutables
foreach ($name in $script:ProductExecutables) { Assert-UnsignedProduct (Join-Path $payload $name) $metadata.Version -RequireX64 }
Assert-UnsignedProduct $metadata.SetupPath $metadata.Version
Assert-ZipPayload $metadata.BinaryZipPath $payload $payloadFiles
$extraction = Join-Path $WorkDirectory 'verified-extraction'
if (Test-Path -LiteralPath $extraction) { throw 'Extraction directory already exists; use a fresh work directory.' }
if (-not (Test-Path -LiteralPath $UnpackerPath -PathType Leaf)) { throw 'Pinned innounp is required.' }
& $UnpackerPath -x -b -y -a ('-d' + $extraction) $metadata.SetupPath
if ($LASTEXITCODE -ne 0) { throw "Non-executing installer extraction failed: $LASTEXITCODE" }
$app = Join-Path $extraction '{app}'
Assert-ExactFiles $app $payloadFiles
foreach ($relative in $payloadFiles) {
    if ((Get-FileHash -LiteralPath (Join-Path $payload $relative) -Algorithm SHA256).Hash -cne (Get-FileHash -LiteralPath (Join-Path $app $relative) -Algorithm SHA256).Hash) {
        throw "Embedded installer payload differs: $relative"
    }
}
# Inno's dontcopy files run installation guards before installed files change.
$temporary = Join-Path $extraction '{tmp}'
Assert-ExactFiles $temporary (@($script:ProductExecutables) + @('SHA256SUMS.txt'))
foreach ($name in @($script:ProductExecutables) + @('SHA256SUMS.txt')) {
    if ((Get-FileHash -LiteralPath (Join-Path $temporary $name) -Algorithm SHA256).Hash -cne (Get-FileHash -LiteralPath (Join-Path $payload $name) -Algorithm SHA256).Hash) {
        throw "Embedded temporary file differs: $name"
    }
}
# innounp also generates its own reconstructed script alongside extracted files.
Assert-ExactFiles $extraction @(@($payloadFiles | ForEach-Object { '{app}/' + $_ }) + @(@($script:ProductExecutables) + @('SHA256SUMS.txt') | ForEach-Object { '{tmp}/' + $_ }) + @('install_script.iss'))

$outputParent = Split-Path -Parent ([IO.Path]::GetFullPath($OutputDirectory))
New-Item -ItemType Directory -Path $outputParent -Force | Out-Null
$staging = Join-Path $outputParent ('.HyperGpuSupport-release-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $staging | Out-Null
$resolvedStaging = (Get-Item -LiteralPath $staging).FullName
if ((Split-Path -Parent $resolvedStaging) -cne (Get-Item -LiteralPath $outputParent).FullName -or
    [IO.Path]::GetFileName($resolvedStaging) -notmatch '^\.HyperGpuSupport-release-[a-f0-9]{32}$') { throw 'Refusing release staging outside the intended output parent.' }
try {
    $lines = foreach ($name in @($setupName, $zipName)) {
        $destination = Join-Path $staging $name
        Copy-Item -LiteralPath (Join-Path $candidate $name) -Destination $destination
        $hash = (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($hash -cne (Get-FileHash -LiteralPath (Join-Path $candidate $name) -Algorithm SHA256).Hash.ToLowerInvariant()) { throw "Final copy differs: $name" }
        "$hash  $name"
    }
    [IO.File]::WriteAllText((Join-Path $staging 'SHA256SUMS.txt'), ($lines -join "`n") + "`n", [Text.UTF8Encoding]::new($false))
    Assert-FileHashes $staging @($setupName, $zipName)
    Move-Item -LiteralPath $resolvedStaging -Destination ([IO.Path]::GetFullPath($OutputDirectory))
} finally {
    if (Test-Path -LiteralPath $resolvedStaging) { Remove-Item -LiteralPath $resolvedStaging -Recurse -Force }
}
Write-Host "Verified unsigned installer, binary ZIP and checksums: $OutputDirectory"
