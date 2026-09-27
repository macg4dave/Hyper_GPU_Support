[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$stagingRoot = Join-Path $repositoryRoot 'data\staging'
$cudaRoot = Join-Path $stagingRoot 'cuda-13.4.1'
$packageRoot = Join-Path $cudaRoot 'packages'
$extractRoot = Join-Path $cudaRoot 'extract'
$toolkitRoot = Join-Path $cudaRoot 'toolkit'
$cmakeArchive = Join-Path $stagingRoot 'cmake-4.4.3-windows-x86_64.zip'
$cmakeRoot = Join-Path $stagingRoot 'cmake-4.4.3'
$samplesRoot = Join-Path $stagingRoot 'cuda-samples'
$samplesCommit = '5443602d89ed99aede2e4b7bf329daddeadb320e'
$samplesTree = '532bbdd145b2a9dc49638b18eb2ab696b73ae57c'

$packages = @(
    @{
        Name = 'cccl'
        Uri = 'https://developer.download.nvidia.com/compute/cuda/redist/cccl/windows-x86_64/cccl-windows-x86_64-13.3.4.2.1-archive.zip'
        Sha256 = 'ea3ebdd98d4d98819cc26b66bc5a0931004ed35ff18eea8b93e0bd2c8bfa5d7c'
    },
    @{
        Name = 'cuda_crt'
        Uri = 'https://developer.download.nvidia.com/compute/cuda/redist/cuda_crt/windows-x86_64/cuda_crt-windows-x86_64-13.4.59-archive.zip'
        Sha256 = 'f969a0e3b086a48f940563cd1b965cb5197fd9540c79cc952b1abfa479f9ede7'
    },
    @{
        Name = 'cuda_cudart'
        Uri = 'https://developer.download.nvidia.com/compute/cuda/redist/cuda_cudart/windows-x86_64/cuda_cudart-windows-x86_64-13.4.49-archive.zip'
        Sha256 = 'e6663f3d3e8949eedc2d5ab92c7c5b9fa3f2a222086d91c42bfc5a38bf2b0225'
    },
    @{
        Name = 'cuda_cuobjdump'
        Uri = 'https://developer.download.nvidia.com/compute/cuda/redist/cuda_cuobjdump/windows-x86_64/cuda_cuobjdump-windows-x86_64-13.4.49-archive.zip'
        Sha256 = '9d1aeb5a25ea4be1abb9ae34985ce9734332d686c314ce597b45d79967286a42'
    },
    @{
        Name = 'cuda_nvcc'
        Uri = 'https://developer.download.nvidia.com/compute/cuda/redist/cuda_nvcc/windows-x86_64/cuda_nvcc-windows-x86_64-13.4.59-archive.zip'
        Sha256 = '06a4fe6ec543030c5e5a7b85493cda90612015d9fff2a54ed0227162c937ea46'
    },
    @{
        Name = 'libnvvm'
        Uri = 'https://developer.download.nvidia.com/compute/cuda/redist/libnvvm/windows-x86_64/libnvvm-windows-x86_64-13.4.59-archive.zip'
        Sha256 = 'a7a07bcc21cc05bce83eedebcbcd9b9f5418c58316b0fa28a57cb2c548b29841'
    }
)

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

$cmakeUri = 'https://github.com/Kitware/CMake/releases/download/v4.4.3/cmake-4.4.3-windows-x86_64.zip'
$cmakeSha256 = '4d52ebab7193a698651639ed80d8d04fd903358843572cf44c7fd234cb7c26ab'
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
        'https://github.com/NVIDIA/cuda-samples.git', $samplesRoot
    )
} else {
    $remote = (& git.exe -C $samplesRoot remote get-url origin).Trim()
    if ($LASTEXITCODE -ne 0 -or $remote -ne 'https://github.com/NVIDIA/cuda-samples.git') {
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
& (Join-Path $cmakeRoot 'cmake-4.4.3-windows-x86_64\bin\cmake.exe') --version
if ($LASTEXITCODE -ne 0) {
    throw 'Pinned CMake failed its version query.'
}

Write-Host "Prepared CUDA Toolkit 13.4.1 redistributables and CUDA Samples $samplesCommit under $stagingRoot."
