<#
.SYNOPSIS
Build an unsigned local installer, or stage the two SignPath signing passes.
.DESCRIPTION
Local is the default and needs no signing account. Official automation calls
BuildPayload, PrepareUninstaller and Package -Signed separately, then signs and
verifies the final setup with scripts/release. This script never installs anything.
#>
[CmdletBinding()]
param(
    [ValidateSet('Local', 'BuildPayload', 'PrepareUninstaller', 'Package')]
    [string]$Stage = 'Local',
    [string]$WorkDirectory,
    [string]$OutputDirectory,
    [string]$PayloadDirectory,
    [string]$UninstallerPath,
    [string]$IsccPath,
    [string]$InstallerPathFile,
    [string]$ExpectedPublisher,
    [switch]$Signed,
    [switch]$SkipBuild,
    [switch]$SkipChecks
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path -Parent $PSScriptRoot
$target = 'x86_64-pc-windows-msvc'
$artifacts = @('hyper-gpu-support.exe', 'hyper-gpu-runner.exe', 'hyper-gpu-guest.exe', 'd3d11-probe.exe', 'app-install.exe')
$utf8 = New-Object System.Text.UTF8Encoding($false)

function Invoke-Cargo {
    param([string[]]$Arguments)
    & cargo @Arguments
    if ($LASTEXITCODE -ne 0) { throw "cargo $($Arguments -join ' ') failed with exit $LASTEXITCODE." }
}

function Write-Utf8 {
    param([string]$Path, [string]$Content)
    [System.IO.File]::WriteAllText($Path, $Content, $utf8)
}

function Assert-Payload {
    param([string]$Directory)
    foreach ($name in $artifacts) {
        if (-not (Test-Path -LiteralPath (Join-Path $Directory $name) -PathType Leaf)) {
            throw "Missing installer payload: $name"
        }
    }
    $actual = @(Get-ChildItem -LiteralPath $Directory -Filter '*.exe' -File | ForEach-Object Name | Sort-Object)
    if (@(Compare-Object ($artifacts | Sort-Object) $actual).Count -ne 0) {
        throw 'Payload must contain exactly the five product and maintenance executables.'
    }
    foreach ($relative in @('LICENSE', 'licenses/THIRD_PARTY_NOTICES.md')) {
        if (-not (Test-Path -LiteralPath (Join-Path $Directory $relative) -PathType Leaf)) {
            throw "Missing installer licence material: $relative"
        }
    }
}

function Write-PayloadHashes {
    param([string]$Directory)
    Assert-Payload $Directory
    $lines = foreach ($name in $artifacts) {
        $hash = (Get-FileHash -LiteralPath (Join-Path $Directory $name) -Algorithm SHA256).Hash.ToLowerInvariant()
        "$hash  $name"
    }
    Write-Utf8 (Join-Path $Directory 'SHA256SUMS.txt') (($lines -join "`n") + "`n")
}

function Find-Compiler {
    if ($IsccPath) {
        return (Get-Item -LiteralPath $IsccPath -ErrorAction Stop).FullName
    }
    $command = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    if ($command) { return $command.Source }
    foreach ($base in @(${env:ProgramFiles(x86)}, $env:ProgramFiles, $env:LOCALAPPDATA)) {
        if ($base) {
            $candidate = Join-Path $base 'Inno Setup 6/ISCC.exe'
            if (Test-Path -LiteralPath $candidate -PathType Leaf) { return $candidate }
        }
    }
    throw 'Install Inno Setup 6.7.3 or pass -IsccPath to its ISCC.exe. Building does not require elevation.'
}

function Save-Metadata {
    Write-Utf8 $metadataPath ($metadata | ConvertTo-Json -Depth 4)
}

function Quote-NativeArgument {
    param([string]$Argument)
    # CommandLineToArgvW quoting for ProcessStartInfo on Windows PowerShell 5.1,
    # which does not provide the ArgumentList API available in PowerShell 7.
    $escaped = [regex]::Replace($Argument, '(\\*)"', '$1$1\"')
    $escaped = [regex]::Replace($escaped, '(\\+)$', '$1$1')
    return '"' + $escaped + '"'
}

function Invoke-Compiler {
    param([switch]$Capture)
    $compiler = Find-Compiler
    $fileVersion = $metadata.Version + '.0'
    $parameters = @(
        "/DAppVersion=$($metadata.Version)", "/DFileVersion=$fileVersion",
        "/DPayloadDir=$PayloadDirectory", "/DOutputDir=$OutputDirectory", "/DOutputName=$outputName"
    )
    if ($Signed -or $Capture) {
        $parameters += @('/DSignedBuild', "/DUninstallerDir=$cacheDirectory")
    }
    $parameters += (Join-Path $root 'packaging/HyperGpuSupport.iss')
    # Inno's first external-signing pass deliberately writes an error to stderr.
    # Read both streams without PowerShell converting that expected native output
    # into a terminating error when a caller redirects its log.
    $start = New-Object System.Diagnostics.ProcessStartInfo
    $start.FileName = $compiler
    $start.Arguments = (($parameters | ForEach-Object { Quote-NativeArgument $_ }) -join ' ')
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $start
    try {
        if (-not $process.Start()) { throw 'Inno Setup compiler could not start.' }
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        $process.WaitForExit()
        $compilerExit = $process.ExitCode
        $compilerOutput = $stdout.GetAwaiter().GetResult()
        $compilerError = $stderr.GetAwaiter().GetResult()
    } finally {
        $process.Dispose()
    }
    if ($compilerOutput) { Write-Host $compilerOutput.TrimEnd() }
    if ($compilerError) { Write-Host $compilerError.TrimEnd() }
    if ($Capture) {
        # Documented SignedUninstaller flow: the first compile creates an unsigned
        # cache entry and exits with a request to sign it externally. It never runs
        # Setup. A later compile embeds the signature from the same cache entry.
        $candidates = @(Get-ChildItem -LiteralPath $cacheDirectory -Filter 'uninst-*.e32' -File)
        if ($compilerExit -ne 2 -or $candidates.Count -ne 1) {
            throw "Uninstaller preparation did not produce exactly one cache entry (compiler exit $compilerExit)."
        }
        $candidate = $candidates[0]
        $signingName = [System.IO.Path]::GetFileNameWithoutExtension($candidate.Name) + '.exe'
        Copy-Item -LiteralPath $candidate.FullName -Destination (Join-Path $captureDirectory $signingName)
        $metadata.UninstallerCacheName = $candidate.Name
        $metadata.UninstallerDirectory = $captureDirectory
        Save-Metadata
        return
    }
    if ($compilerExit -ne 0) { throw "Inno Setup compilation failed with exit $compilerExit." }
    if ($Signed) {
        $cachedHash = (Get-FileHash -LiteralPath (Join-Path $cacheDirectory $metadata.UninstallerCacheName) -Algorithm SHA256).Hash
        if ($cachedHash -cne $metadata.SignedUninstallerSha256) {
            throw 'The signed uninstaller cache changed during compilation.'
        }
    }
    $setup = Join-Path $OutputDirectory ($outputName + '.exe')
    if (-not (Test-Path -LiteralPath $setup -PathType Leaf)) { throw "Compiler did not produce $setup." }
    $metadata.SetupPath = $setup
    Save-Metadata
}

Push-Location $root
try {
    if ($Signed -and $Stage -ne 'Package') { throw '-Signed is only valid for the Package stage.' }
    if ($Stage -in @('PrepareUninstaller', 'Package') -and -not $WorkDirectory) {
        throw "-$Stage requires -WorkDirectory from BuildPayload."
    }
    $cargoJson = & cargo metadata --locked --format-version 1 --no-deps
    if ($LASTEXITCODE -ne 0) { throw 'cargo metadata failed.' }
    $cargoMetadata = $cargoJson | ConvertFrom-Json
    $package = @($cargoMetadata.packages | Where-Object { $_.manifest_path -eq (Join-Path $root 'Cargo.toml') })
    if ($package.Count -ne 1) { throw 'Could not identify the root Cargo package.' }
    $version = [string]$package[0].version
    if ($version -notmatch '^\d+\.\d+\.\d+$') { throw "Installer versions must be numeric major.minor.patch Cargo versions: $version" }
    $toolchain = Get-Content -LiteralPath (Join-Path $root 'rust-toolchain.toml') -Raw
    if ($toolchain -notmatch '(?m)^channel\s*=\s*"([^"]+)"') { throw 'Pinned Rust toolchain channel is missing.' }
    $channel = $Matches[1]
    if (-not $WorkDirectory) { $WorkDirectory = Join-Path $cargoMetadata.target_directory ('installer/' + [guid]::NewGuid().ToString('N')) }
    $WorkDirectory = [System.IO.Path]::GetFullPath($WorkDirectory)
    if (-not $OutputDirectory) { $OutputDirectory = Join-Path $root 'dist' }
    $OutputDirectory = [System.IO.Path]::GetFullPath($OutputDirectory)
    New-Item -ItemType Directory -Path $WorkDirectory -Force | Out-Null
    $metadataPath = Join-Path $WorkDirectory 'build-metadata.json'
    if ($Stage -in @('Local', 'BuildPayload')) {
        if (Test-Path -LiteralPath $metadataPath) { throw 'Use a fresh WorkDirectory for a new build.' }
        if (-not $SkipChecks) {
            Invoke-Cargo -Arguments @('fmt', '--all', '--check')
            Invoke-Cargo -Arguments @('clippy', '--locked', '--workspace', '--all-targets', '--', '-D', 'warnings')
            Invoke-Cargo -Arguments @('test', '--locked', '--workspace', '--all-targets')
        }
        if (-not $SkipBuild) {
            Invoke-Cargo -Arguments @('build', '--locked', '--release', '--target', $target, '--package', $package[0].name, '--package', 'app-install', '--bins')
        }
        $releaseDirectory = Join-Path $cargoMetadata.target_directory "$target/release"
        $PayloadDirectory = Join-Path $WorkDirectory 'payload'
        if (Test-Path -LiteralPath $PayloadDirectory) { throw 'Payload staging directory already exists; use a fresh WorkDirectory.' }
        New-Item -ItemType Directory -Path $PayloadDirectory | Out-Null
        foreach ($name in $artifacts) {
            Copy-Item -LiteralPath (Join-Path $releaseDirectory $name) -Destination (Join-Path $PayloadDirectory $name)
        }
        Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $PayloadDirectory
        Copy-Item -LiteralPath (Join-Path $root 'licenses') -Destination (Join-Path $PayloadDirectory 'licenses') -Recurse
        & (Join-Path $root 'scripts/collect-licenses.ps1') -Destination (Join-Path $PayloadDirectory 'licenses/dependencies')
        if (-not $?) { throw 'Dependency licence collection failed.' }
        Write-PayloadHashes $PayloadDirectory
        $metadata = [pscustomobject]@{
            Version = $version; Target = $target; Toolchain = $channel
            PayloadDirectory = $PayloadDirectory; UninstallerDirectory = ''; UninstallerCacheName = ''; SignedUninstallerSha256 = ''; SetupPath = ''
        }
        Save-Metadata
        Write-Host "Staged payload: $PayloadDirectory"
        if ($Stage -eq 'BuildPayload') { return }
    } else {
        $metadata = Get-Content -LiteralPath $metadataPath -Raw | ConvertFrom-Json
        if ($metadata.Version -cne $version -or $metadata.Target -cne $target -or $metadata.Toolchain -cne $channel) {
            throw 'Staged build metadata does not match the current Cargo version, target or toolchain.'
        }
        if (-not $PayloadDirectory) { $PayloadDirectory = $metadata.PayloadDirectory }
    }
    $PayloadDirectory = (Get-Item -LiteralPath $PayloadDirectory).FullName
    Assert-Payload $PayloadDirectory
    New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
    $outputName = "HyperGpuSupport-$version-windows-x64-setup"
    if ($Stage -eq 'PrepareUninstaller') {
        $cacheDirectory = Join-Path $WorkDirectory 'uninstaller-cache'
        $captureDirectory = Join-Path $WorkDirectory 'uninstaller'
        foreach ($directory in @($cacheDirectory, $captureDirectory)) {
            if (Test-Path -LiteralPath $directory) { throw "Uninstaller staging directory already exists: $directory" }
            New-Item -ItemType Directory -Path $directory | Out-Null
        }
        Invoke-Compiler -Capture
        return
    }
    if ($Signed) {
        if (-not $ExpectedPublisher -or -not $UninstallerPath) { throw 'Signed packaging requires -ExpectedPublisher and -UninstallerPath.' }
        $signedUninstaller = Get-Item -LiteralPath $UninstallerPath
        if ($metadata.UninstallerCacheName -notmatch '^uninst-6\.7\.3-[a-fA-F0-9]+\.e32$' -or
            $signedUninstaller.BaseName -cne [System.IO.Path]::GetFileNameWithoutExtension($metadata.UninstallerCacheName)) {
            throw 'Signed uninstaller does not match the cache entry prepared for this build.'
        }
        $verify = Join-Path $root 'scripts/release/verify-signatures.ps1'
        $verifyPaths = @($artifacts | ForEach-Object { Join-Path $PayloadDirectory $_ })
        & $verify -Path $verifyPaths -ExpectedPublisher $ExpectedPublisher -ExpectedVersion $version
        if (-not $?) { throw 'Signed payload verification failed.' }
        $baseline = Join-Path $metadata.UninstallerDirectory ($signedUninstaller.BaseName + '.exe')
        $uninstallerVersion = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($baseline).FileVersion.Trim()
        & $verify -Path @($signedUninstaller.FullName) -ExpectedPublisher $ExpectedPublisher -ExpectedVersion $version -ExpectedFileVersion $uninstallerVersion
        if (-not $?) { throw 'Signed uninstaller verification failed.' }
        $metadata.SignedUninstallerSha256 = (Get-FileHash -LiteralPath $signedUninstaller.FullName -Algorithm SHA256).Hash
        $cacheDirectory = Join-Path $WorkDirectory ('signed-uninstaller-cache-' + [guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $cacheDirectory | Out-Null
        Copy-Item -LiteralPath $signedUninstaller.FullName -Destination (Join-Path $cacheDirectory $metadata.UninstallerCacheName)
    } else {
        $outputName = "HyperGpuSupport-$version-windows-x64-unsigned-setup"
    }
    Write-PayloadHashes $PayloadDirectory
    Invoke-Compiler
    if (-not $Signed) {
        $hash = (Get-FileHash -LiteralPath $metadata.SetupPath -Algorithm SHA256).Hash.ToLowerInvariant()
        Write-Utf8 (Join-Path $OutputDirectory 'SHA256SUMS.txt') ($hash + '  ' + [System.IO.Path]::GetFileName($metadata.SetupPath) + "`n")
        Write-Host "Unsigned local installer: $($metadata.SetupPath)"
    } else {
        Write-Host "Candidate installer (final SignPath signature still required): $($metadata.SetupPath)"
    }
    if ($InstallerPathFile) { Write-Utf8 ([System.IO.Path]::GetFullPath($InstallerPathFile)) $metadata.SetupPath }
} finally {
    Pop-Location
}
