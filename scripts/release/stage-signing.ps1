[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateSet('Prepare', 'Import')][string]$Stage,
    [Parameter(Mandatory)][string]$WorkDirectory,
    [string]$SignedDirectory,
    [string]$ExpectedPublisher,
    [string]$SignToolPath
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'release-common.ps1')
$metadata = Get-ReleaseMetadata $WorkDirectory
$requestDirectory = Join-Path $WorkDirectory 'signing-input'
$uninstallers = @(Get-ChildItem -LiteralPath $metadata.UninstallerDirectory -Filter '*.exe' -File)
if ($uninstallers.Count -ne 1) { throw 'Expected exactly one generated uninstaller.' }
$relativePaths = @($script:ProductExecutables | ForEach-Object { "payload/$_" }) + @("uninstaller/$($uninstallers[0].Name)")
if ($Stage -eq 'Prepare') {
    if (Test-Path -LiteralPath $requestDirectory) { throw 'Signing input already exists; use a fresh work directory.' }
    New-Item -ItemType Directory -Path (Join-Path $requestDirectory 'payload'), (Join-Path $requestDirectory 'uninstaller') | Out-Null
    foreach ($name in $script:ProductExecutables) {
        Copy-Item -LiteralPath (Join-Path $metadata.PayloadDirectory $name) -Destination (Join-Path $requestDirectory 'payload')
    }
    Copy-Item -LiteralPath $uninstallers[0].FullName -Destination (Join-Path $requestDirectory 'uninstaller')
    Assert-ExactFiles $requestDirectory $relativePaths
} else {
    if (-not $SignedDirectory -or -not $ExpectedPublisher) { throw 'SignedDirectory and ExpectedPublisher are required.' }
    Assert-ExactFiles $SignedDirectory $relativePaths
    foreach ($relative in $relativePaths) {
        $signed = Join-Path $SignedDirectory $relative
        $original = Join-Path $requestDirectory $relative
        Assert-SigningOnlyChange $original $signed
        $fileVersion = [Diagnostics.FileVersionInfo]::GetVersionInfo($original).FileVersion.Trim()
        & (Join-Path $PSScriptRoot 'verify-signatures.ps1') -Path $signed -ExpectedPublisher $ExpectedPublisher -ExpectedVersion $metadata.Version -ExpectedFileVersion $fileVersion -SignToolPath $SignToolPath
    }
    $signedPayload = Join-Path $WorkDirectory 'signed-payload'
    if (Test-Path -LiteralPath $signedPayload) { throw 'Signed payload already exists; use a fresh work directory.' }
    Copy-Item -LiteralPath $metadata.PayloadDirectory -Destination $signedPayload -Recurse
    foreach ($name in $script:ProductExecutables) {
        Copy-Item -LiteralPath (Join-Path $SignedDirectory "payload/$name") -Destination (Join-Path $signedPayload $name) -Force
    }
    # Metadata is not accepted from the signing service: retain the local build
    # metadata and copy only the six exact, verified executable results.
    Write-Host "Verified signed payload: $signedPayload"
}
