<#
.SYNOPSIS
Build unsigned Windows x64 installer and binary ZIP packages.
.DESCRIPTION
Local is the default for source builds and release automation. BuildPayload and
Package allow inspecting the shared staging directory before packaging. All
packages are explicitly labelled unsigned. This script never installs anything.
#>
[CmdletBinding()]
param(
    [ValidateSet('Local', 'BuildPayload', 'Package')]
    [string]$Stage = 'Local',
    [string]$WorkDirectory,
    [string]$OutputDirectory,
    [string]$PayloadDirectory,
    [string]$IsccPath,
    [string]$InstallerPathFile,
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
    $allowedRootNames = $artifacts + @('LICENSE', 'SHA256SUMS.txt', 'licenses')
    foreach ($entry in @(Get-Item -LiteralPath $Directory) + @(Get-ChildItem -LiteralPath $Directory -Force -Recurse)) {
        if (($entry.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Payload must not contain symbolic links or reparse points: $($entry.FullName)"
        }
    }
    foreach ($entry in Get-ChildItem -LiteralPath $Directory -Force) {
        if ($allowedRootNames -cnotcontains $entry.Name -or ($entry.PSIsContainer -and $entry.Name -cne 'licenses')) {
            throw "Unexpected payload entry: $($entry.Name)"
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
    $compiler = Find-Compiler
    $fileVersion = $metadata.Version + '.0'
    $parameters = @(
        "/DAppVersion=$($metadata.Version)", "/DFileVersion=$fileVersion",
        "/DPayloadDir=$PayloadDirectory", "/DOutputDir=$OutputDirectory", "/DOutputName=$outputName"
    )
    $parameters += (Join-Path $root 'packaging/HyperGpuSupport.iss')
    # Read both streams concurrently so verbose compiler output cannot deadlock
    # and failures remain visible when the caller redirects its build log.
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
    if ($compilerExit -ne 0) { throw "Inno Setup compilation failed with exit $compilerExit." }
    $setup = Join-Path $OutputDirectory ($outputName + '.exe')
    if (-not (Test-Path -LiteralPath $setup -PathType Leaf)) { throw "Compiler did not produce $setup." }
    $metadata.SetupPath = $setup
    Save-Metadata
}

Push-Location $root
try {
    if ($Stage -eq 'Package' -and -not $WorkDirectory) {
        throw 'Package requires -WorkDirectory from BuildPayload.'
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
            & (Join-Path $root 'scripts/testing/test-installer.ps1') -IsccPath (Find-Compiler)
            if (-not $?) { throw 'Installer regression checks failed.' }
            & (Join-Path $root 'scripts/release/test-release.ps1')
            if (-not $?) { throw 'Release regression checks failed.' }
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
            PayloadDirectory = $PayloadDirectory; SetupPath = ''; BinaryZipPath = ''
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
    $metadata.PayloadDirectory = $PayloadDirectory
    $payloadPrefix = $PayloadDirectory.TrimEnd([System.IO.Path]::DirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar
    if ($OutputDirectory.Equals($PayloadDirectory, [StringComparison]::OrdinalIgnoreCase) -or
        $OutputDirectory.StartsWith($payloadPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'OutputDirectory must be outside the payload staging directory.'
    }
    New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
    $outputName = "HyperGpuSupport-$version-windows-x64-unsigned-setup"
    $zipPath = Join-Path $OutputDirectory "HyperGpuSupport-$version-windows-x64-unsigned-binaries.zip"
    $installInstructions = (Get-Item -LiteralPath (Join-Path $root 'packaging/install.txt') -ErrorAction Stop).FullName
    Write-PayloadHashes $PayloadDirectory
    Invoke-Compiler
    Add-Type -AssemblyName System.IO.Compression, System.IO.Compression.FileSystem
    $zipTemporaryPath = Join-Path $WorkDirectory ('binaries-' + [guid]::NewGuid().ToString('N') + '.zip')
    $archive = [System.IO.Compression.ZipFile]::Open($zipTemporaryPath, [System.IO.Compression.ZipArchiveMode]::Create)
    try {
        # Windows PowerShell 5.1's CreateFromDirectory writes backslashes. Use
        # explicit portable entry names so local and Actions ZIPs agree.
        foreach ($file in Get-ChildItem -LiteralPath $PayloadDirectory -File -Recurse | Sort-Object FullName) {
            $entryName = $file.FullName.Substring($PayloadDirectory.Length + 1).Replace('\', '/')
            [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, $file.FullName, $entryName, [System.IO.Compression.CompressionLevel]::Optimal) | Out-Null
        }
        [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, $installInstructions, 'install.txt', [System.IO.Compression.CompressionLevel]::Optimal) | Out-Null
    } finally { $archive.Dispose() }
    Move-Item -LiteralPath $zipTemporaryPath -Destination $zipPath -Force
    $metadata.BinaryZipPath = $zipPath
    Save-Metadata
    $packageHashes = foreach ($path in @($metadata.SetupPath, $metadata.BinaryZipPath)) {
        $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
        $hash + '  ' + [System.IO.Path]::GetFileName($path)
    }
    Write-Utf8 (Join-Path $OutputDirectory 'SHA256SUMS.txt') (($packageHashes -join "`n") + "`n")
    Write-Host "Unsigned installer: $($metadata.SetupPath)"
    Write-Host "Unsigned binaries: $($metadata.BinaryZipPath)"
    if ($InstallerPathFile) { Write-Utf8 ([System.IO.Path]::GetFullPath($InstallerPathFile)) $metadata.SetupPath }
} finally {
    Pop-Location
}
