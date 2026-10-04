# Privilege: non-elevated.
[CmdletBinding()]
param(
    [string] $ProjectConfigurationPath = (Join-Path $PSScriptRoot '..\..\config\project.toml')
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
. (Join-Path $repositoryRoot 'scripts\common\project-config.ps1')
$projectConfiguration = Import-ProjectConfiguration -Path $ProjectConfigurationPath
$stagingRelative = [string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.staging_directory')
$cudaRelease = [string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.cuda_release')
$cmakeRelease = [string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.cmake_release')
$samplesRepository = [string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.cuda_samples_repository')
$stagingRoot = Join-Path $repositoryRoot $stagingRelative
$cudaRoot = Join-Path $stagingRoot "cuda-$cudaRelease"
$packageRoot = Join-Path $cudaRoot 'packages'
$extractRoot = Join-Path $cudaRoot 'extract'
$toolkitRoot = Join-Path $cudaRoot 'toolkit'
$cmakeArchive = Join-Path $stagingRoot "cmake-$cmakeRelease-windows-x86_64.zip"
$cmakeRoot = Join-Path $stagingRoot "cmake-$cmakeRelease"
$samplesRoot = Join-Path $stagingRoot 'cuda-samples'
$samplesCommit = [string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.cuda_samples_commit')
$samplesTree = [string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.cuda_samples_tree')

$packageNames = @('cccl', 'cuda_crt', 'cuda_cudart', 'cuda_cuobjdump', 'cuda_nvcc', 'libnvvm')
$packages = @($packageNames | ForEach-Object {
    @{
        Name = $_
        Uri = [string](Get-ProjectConfigurationValue $projectConfiguration "tooling.cuda_packages.$_.uri")
        Sha256 = [string](Get-ProjectConfigurationValue $projectConfiguration "tooling.cuda_packages.$_.sha256")
    }
})

function Invoke-Native {
    param(
        [Parameter(Mandatory)]
        [string] $Executable,

        [Parameter(Mandatory)]
        [string[]] $Arguments
    )

    Write-Host "> $Executable $($Arguments -join ' ')"
    & $Executable @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Executable failed with exit code $LASTEXITCODE."
    }
}

function Assert-Hash {
    param(
        [Parameter(Mandatory)]
        [string] $Path,

        [Parameter(Mandatory)]
        [string] $Expected
    )

    $actual = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $Expected) {
        throw "SHA-256 mismatch for ${Path}: expected $Expected, found $actual."
    }
}

function Get-VerifiedDownload {
    param(
        [Parameter(Mandatory)]
        [string] $Uri,

        [Parameter(Mandatory)]
        [string] $Path,

        [Parameter(Mandatory)]
        [string] $Sha256
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Invoke-Native -Executable 'curl.exe' -Arguments @(
            '-L', '--fail', '--silent', '--show-error', $Uri, '-o', $Path
        )
    }
    Assert-Hash -Path $Path -Expected $Sha256
}

New-Item -ItemType Directory -Force -Path $packageRoot | Out-Null
foreach ($package in $packages) {
    $archive = Join-Path $packageRoot "$($package.Name).zip"
    Get-VerifiedDownload -Uri $package.Uri -Path $archive -Sha256 $package.Sha256
}

$cmakeUri = [string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.cmake_uri')
$cmakeSha256 = [string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.cmake_sha256')
Get-VerifiedDownload -Uri $cmakeUri -Path $cmakeArchive -Sha256 $cmakeSha256

foreach ($path in @($extractRoot, $toolkitRoot, $cmakeRoot)) {
    if (Test-Path -LiteralPath $path) {
        $resolved = (Resolve-Path -LiteralPath $path).Path
        if (-not $resolved.StartsWith($stagingRoot, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to replace a path outside repository staging: $resolved"
        }
        Remove-Item -LiteralPath $resolved -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $path | Out-Null
}

foreach ($package in $packages) {
    $archive = Join-Path $packageRoot "$($package.Name).zip"
    $destination = Join-Path $extractRoot $package.Name
    New-Item -ItemType Directory -Force -Path $destination | Out-Null
    Invoke-Native -Executable '7z.exe' -Arguments @('x', '-y', $archive, "-o$destination")
    $topDirectories = @(Get-ChildItem -LiteralPath $destination -Directory)
    if ($topDirectories.Count -ne 1) {
        throw "Expected one archive root for $($package.Name)."
    }
    Copy-Item -Path (Join-Path $topDirectories[0].FullName '*') -Destination $toolkitRoot -Recurse -Force
}

Invoke-Native -Executable 'tar.exe' -Arguments @('-xf', $cmakeArchive, '-C', $cmakeRoot)

if (-not (Test-Path -LiteralPath (Join-Path $samplesRoot '.git'))) {
    Invoke-Native -Executable 'git.exe' -Arguments @(
        'clone', '--filter=blob:none', '--no-checkout',
        $samplesRepository, $samplesRoot
    )
} else {
    $remote = (& git.exe -C $samplesRoot remote get-url origin).Trim()
    if ($LASTEXITCODE -ne 0 -or $remote -ne $samplesRepository) {
        throw "Existing CUDA samples staging tree has an unexpected origin: $remote"
    }
}
Invoke-Native -Executable 'git.exe' -Arguments @(
    '-C', $samplesRoot, 'fetch', '--depth', '1', 'origin', $samplesCommit
)
Invoke-Native -Executable 'git.exe' -Arguments @(
    '-C', $samplesRoot, 'checkout', '--detach', $samplesCommit
)
$actualCommit = (& git.exe -C $samplesRoot rev-parse HEAD).Trim()
$actualTree = (& git.exe -C $samplesRoot rev-parse 'HEAD^{tree}').Trim()
$sourceChanges = @(& git.exe -C $samplesRoot status --porcelain --untracked-files=no)
if ($actualCommit -ne $samplesCommit -or $actualTree -ne $samplesTree -or $sourceChanges.Count -ne 0) {
    throw "CUDA samples source did not match the pinned clean commit/tree."
}

& (Join-Path $toolkitRoot 'bin\nvcc.exe') --version
if ($LASTEXITCODE -ne 0) {
    throw 'Pinned nvcc failed its version query.'
}
& (Join-Path $cmakeRoot "cmake-$cmakeRelease-windows-x86_64\bin\cmake.exe") --version
if ($LASTEXITCODE -ne 0) {
    throw 'Pinned CMake failed its version query.'
}

Write-Host "Prepared CUDA Toolkit $cudaRelease redistributables and CUDA Samples $samplesCommit under $stagingRoot."
