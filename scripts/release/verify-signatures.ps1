[CmdletBinding()]
param(
    [Parameter(Mandatory)][string[]]$Path,
    [Parameter(Mandatory)][ValidateNotNullOrEmpty()][string]$ExpectedPublisher,
    [Parameter(Mandatory)][ValidateNotNullOrEmpty()][string]$ExpectedVersion,
    [string]$ExpectedFileVersion,
    [string]$SignToolPath
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if ($ExpectedVersion -notmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$') {
    throw 'ExpectedVersion must be a Cargo semantic version.'
}
$numericVersion = '{0}.{1}.{2}.0' -f $Matches[1], $Matches[2], $Matches[3]
if ($ExpectedFileVersion) { $numericVersion = $ExpectedFileVersion }
if (-not $SignToolPath) {
    $sdk = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
    $candidates = @(Get-ChildItem -LiteralPath $sdk -Directory | Sort-Object Name -Descending | ForEach-Object {
        $candidate = Join-Path $_.FullName 'x64\signtool.exe'
        if (Test-Path -LiteralPath $candidate -PathType Leaf) { $candidate }
    })
    if ($candidates.Count -eq 0) { throw 'Windows SDK x64 signtool.exe is required for release verification.' }
    $SignToolPath = $candidates[0]
}
if (-not (Test-Path -LiteralPath $SignToolPath -PathType Leaf)) { throw 'signtool.exe was not found.' }

function Test-CertificateUsage {
    param([System.Security.Cryptography.X509Certificates.X509Certificate2]$Certificate, [string]$Oid)
    foreach ($extension in $Certificate.Extensions) {
        if ($extension -is [System.Security.Cryptography.X509Certificates.X509EnhancedKeyUsageExtension]) {
            foreach ($usage in $extension.EnhancedKeyUsages) {
                if ($usage.Value -eq $Oid) { return $true }
            }
        }
    }
    return $false
}

foreach ($file in $Path) {
    $resolved = (Get-Item -LiteralPath $file -Force).FullName
    if ((Get-Item -LiteralPath $resolved).Attributes -band [IO.FileAttributes]::ReparsePoint) {
        throw "Release input must be a regular file: $resolved"
    }
    $signature = Get-AuthenticodeSignature -LiteralPath $resolved
    if ($signature.Status -ne [System.Management.Automation.SignatureStatus]::Valid -or -not $signature.SignerCertificate) {
        throw "Authenticode validation failed for ${resolved}: $($signature.Status)"
    }
    if (-not [StringComparer]::Ordinal.Equals($signature.SignerCertificate.Subject, $ExpectedPublisher)) {
        throw "Unexpected Authenticode publisher for $resolved"
    }
    if (-not (Test-CertificateUsage $signature.SignerCertificate '1.3.6.1.5.5.7.3.3')) {
        throw "Signer lacks the Code Signing EKU: $resolved"
    }
    if (-not $signature.TimeStamperCertificate -or -not (Test-CertificateUsage $signature.TimeStamperCertificate '1.3.6.1.5.5.7.3.8')) {
        throw "A trusted timestamp with the Time Stamping EKU is required: $resolved"
    }
    $version = [Diagnostics.FileVersionInfo]::GetVersionInfo($resolved)
    if ($version.ProductName.Trim() -cne 'Hyper GPU Support' -or $version.ProductVersion.Trim() -cne $ExpectedVersion -or $version.FileVersion.Trim() -cne $numericVersion) {
        throw "Product name/version metadata mismatch: $resolved"
    }
    # WinTrust checks the Windows Authenticode trust policy; SignTool additionally
    # verifies every signature and its timestamp. /tw warnings fail this gate too.
    & $SignToolPath verify /pa /all /tw $resolved
    if ($LASTEXITCODE -ne 0) { throw "SignTool verification failed for $resolved (exit $LASTEXITCODE)." }
    Write-Host "Verified signed and timestamped: $([IO.Path]::GetFileName($resolved))"
}
