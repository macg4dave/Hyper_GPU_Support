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
$stagingRoot = Join-Path $repositoryRoot ([string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.staging_directory'))
$cudaRelease = [string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.cuda_release')
$cmakeRelease = [string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.cmake_release')
$toolkitRoot = Join-Path $stagingRoot "cuda-$cudaRelease\toolkit"
$samplesRoot = Join-Path $stagingRoot 'cuda-samples'
$sourceRoot = Join-Path $samplesRoot 'cpp\0_Introduction\vectorAddDrv'
$buildRoot = Join-Path $stagingRoot 'cuda-vector-add-build'
$outputRoot = Join-Path $repositoryRoot 'probes\cuda\compiled'
$upstreamRoot = Join-Path $repositoryRoot 'probes\cuda\upstream'
$cmake = Join-Path $stagingRoot "cmake-$cmakeRelease\cmake-$cmakeRelease-windows-x86_64\bin\cmake.exe"
$nvcc = Join-Path $toolkitRoot 'bin\nvcc.exe'
$cuobjdump = Join-Path $toolkitRoot 'bin\cuobjdump.exe'
$samplesCommit = [string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.cuda_samples_commit')
$samplesTree = [string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.cuda_samples_tree')
$developerShell = [string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.visual_studio_developer_shell')
$cudaArchitecture = '{0}{1}' -f @(
    Get-ProjectConfigurationValue $projectConfiguration 'slot.cuda_compute_capability_major'
    Get-ProjectConfigurationValue $projectConfiguration 'slot.cuda_compute_capability_minor'
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

foreach ($path in @($cmake, $nvcc, $cuobjdump, $developerShell, (Join-Path $sourceRoot 'CMakeLists.txt'))) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Required prepared input is missing: $path. Run scripts/setup/prepare-cuda-probe.ps1."
    }
}

$actualCommit = (& git.exe -C $samplesRoot rev-parse HEAD).Trim()
$actualTree = (& git.exe -C $samplesRoot rev-parse 'HEAD^{tree}').Trim()
$sourceChanges = @(& git.exe -C $samplesRoot status --porcelain --untracked-files=no)
if ($actualCommit -ne $samplesCommit -or $actualTree -ne $samplesTree -or $sourceChanges.Count -ne 0) {
    throw 'CUDA samples source is not the pinned clean commit/tree.'
}

if (Test-Path -LiteralPath $buildRoot) {
    $resolvedBuild = (Resolve-Path -LiteralPath $buildRoot).Path
    if (-not $resolvedBuild.StartsWith($stagingRoot, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to replace a build path outside repository staging: $resolvedBuild"
    }
    Remove-Item -LiteralPath $resolvedBuild -Recurse -Force
}

& $developerShell `
    -Arch amd64 -HostArch amd64 -SkipAutomaticLocation
if ($LASTEXITCODE -ne 0) {
    throw 'Visual Studio developer environment initialization failed.'
}

Invoke-Native -Executable $cmake -Arguments @(
    '-S', $sourceRoot,
    '-B', $buildRoot,
    '-G', 'NMake Makefiles',
    '-DCMAKE_BUILD_TYPE=Release',
    "-DCMAKE_CUDA_ARCHITECTURES=$cudaArchitecture",
    "-DCUDAToolkit_ROOT=$toolkitRoot",
    "-DCMAKE_CUDA_COMPILER=$nvcc"
)
Invoke-Native -Executable $cmake -Arguments @(
    '--build', $buildRoot, '--config', 'Release', '--target', 'vectorAddDrv'
)

$executable = Join-Path $buildRoot 'vectorAddDrv.exe'
$fatbin = Join-Path $buildRoot 'vectorAdd_kernel64.fatbin'
$elf = @(& $cuobjdump --dump-elf $fatbin 2>&1)
if ($LASTEXITCODE -ne 0 -or ($elf -join "`n") -notmatch "(?m)^arch = sm_$cudaArchitecture\s*$") {
    throw "cuobjdump did not confirm an sm_$cudaArchitecture image in the generated FATBIN."
}

New-Item -ItemType Directory -Force -Path $outputRoot, $upstreamRoot | Out-Null
Copy-Item -LiteralPath $executable -Destination (Join-Path $outputRoot 'vectorAddDrv.exe') -Force
Copy-Item -LiteralPath $fatbin -Destination (Join-Path $outputRoot 'vectorAdd_kernel64.fatbin') -Force
Copy-Item -LiteralPath (Join-Path $samplesRoot 'LICENSE') -Destination (Join-Path $upstreamRoot 'LICENSE.cuda-samples') -Force
Copy-Item -LiteralPath (Join-Path $sourceRoot 'CMakeLists.txt') -Destination $upstreamRoot -Force
Copy-Item -LiteralPath (Join-Path $sourceRoot 'README.md') -Destination $upstreamRoot -Force
Copy-Item -LiteralPath (Join-Path $sourceRoot 'vectorAddDrv.cpp') -Destination $upstreamRoot -Force
Copy-Item -LiteralPath (Join-Path $sourceRoot 'vectorAdd_kernel.cu') -Destination $upstreamRoot -Force

Write-Host 'cuobjdump architecture evidence:'
$elf | Select-String 'arch =|CUDA Virtual SM|Tool Command Line Arguments'
Write-Host "CUDA Samples commit: $actualCommit"
Write-Host "CUDA Samples tree:   $actualTree"
Get-FileHash -Algorithm SHA256 -LiteralPath @(
    (Join-Path $upstreamRoot 'vectorAddDrv.cpp'),
    (Join-Path $upstreamRoot 'vectorAdd_kernel.cu'),
    (Join-Path $outputRoot 'vectorAddDrv.exe'),
    (Join-Path $outputRoot 'vectorAdd_kernel64.fatbin')
) | Select-Object Path, Hash
