[CmdletBinding()]
param([Parameter(Mandatory)][string]$Directory)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
New-Item -ItemType Directory -Path $Directory -Force | Out-Null

function Get-PinnedDownload {
    param([string]$Uri, [string]$Name, [string]$Sha256)
    $destination = Join-Path $Directory $Name
    if (-not (Test-Path -LiteralPath $destination -PathType Leaf)) {
        Invoke-WebRequest -UseBasicParsing -Uri $Uri -OutFile $destination
    }
    if ((Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash -cne $Sha256) {
        throw "SHA-256 mismatch for $Name. Delete the damaged download and retry; do not bypass this check."
    }
    return $destination
}

$innoInstaller = Get-PinnedDownload 'https://github.com/jrsoftware/issrc/releases/download/is-6_7_3/innosetup-6.7.3.exe' 'innosetup-6.7.3.exe' '9C73C3BAE7ED48D44112A0F48E66742C00090BDB5BEF71D9D3C056C66E97B732'
$innoDirectory = Join-Path $Directory 'inno-6.7.3'
if (-not (Test-Path -LiteralPath (Join-Path $innoDirectory 'ISCC.exe') -PathType Leaf)) {
    # Portable mode is confined to the build tool directory; no elevation,
    # shell registration or machine-wide installation is performed.
    $process = Start-Process -FilePath $innoInstaller -ArgumentList @('/PORTABLE=1', '/CURRENTUSER', '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', ('/DIR="{0}"' -f $innoDirectory)) -WindowStyle Hidden -PassThru -Wait
    if ($process.ExitCode -ne 0) { throw "Inno Setup tool provisioning failed: $($process.ExitCode)" }
}
$unpackerArchive = Get-PinnedDownload 'https://raw.githubusercontent.com/jrathlev/InnoUnpacker-Windows-GUI/032d64c782efc6b36766d8f9c3b1b2f042144a58/innounp-2/bin/innounp-267.zip' 'innounp-267.zip' 'AC1D98BBA6588072ADE06163781938DF274F444BEC7318069668608D1E5FAAE8'
$unpackerDirectory = Join-Path $Directory 'innounp-267'
if (-not (Test-Path -LiteralPath (Join-Path $unpackerDirectory 'innounp.exe') -PathType Leaf)) {
    Expand-Archive -LiteralPath $unpackerArchive -DestinationPath $unpackerDirectory
}
$iscc = Join-Path $innoDirectory 'ISCC.exe'
$unpacker = Join-Path $unpackerDirectory 'innounp.exe'
if (-not (Test-Path -LiteralPath $iscc -PathType Leaf) -or -not (Test-Path -LiteralPath $unpacker -PathType Leaf)) { throw 'Build tools were not provisioned.' }
if ($env:GITHUB_ENV) {
    "ISCC_PATH=$iscc" | Out-File -LiteralPath $env:GITHUB_ENV -Append -Encoding utf8
    "INNOUNP_PATH=$unpacker" | Out-File -LiteralPath $env:GITHUB_ENV -Append -Encoding utf8
}
Write-Host "Inno Setup compiler: $iscc"
Write-Host "Non-executing installer unpacker: $unpacker"
