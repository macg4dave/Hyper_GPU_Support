# Privilege: non-elevated.
[CmdletBinding()]
param(
    [string] $DxcRoot,
    [string] $ProjectConfigurationPath = (Join-Path $PSScriptRoot '..\..\config\project.toml')
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Invoke-Compiler {
    param(
        [Parameter(Mandatory)]
        [string] $Executable,

        [Parameter(Mandatory)]
        [string[]] $Arguments
    )

    Write-Host "> $Executable $($Arguments -join ' ')"
    & $Executable @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Shader compiler failed with exit code $LASTEXITCODE."
    }
}

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
. (Join-Path $repositoryRoot 'scripts\common\project-config.ps1')
$projectConfiguration = Import-ProjectConfiguration -Path $ProjectConfigurationPath
if ([string]::IsNullOrWhiteSpace($DxcRoot)) {
    $DxcRoot = [string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.dxc_directory')
}
$windowsSdkVersion = [string](Get-ProjectConfigurationValue $projectConfiguration 'tooling.windows_sdk_version')
$source = Join-Path $repositoryRoot 'probes\shaders\offscreen.hlsl'
$output = Join-Path $repositoryRoot 'probes\shaders\compiled'
$fxc = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin\$windowsSdkVersion\x64\fxc.exe"
$resolvedDxcRoot = (Resolve-Path (Join-Path $repositoryRoot $DxcRoot)).Path
$dxc = Join-Path $resolvedDxcRoot 'bin\x64\dxc.exe'

foreach ($path in @($source, $fxc, $dxc)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Required shader input/tool is missing: $path"
    }
}

New-Item -ItemType Directory -Force -Path $output | Out-Null
Invoke-Compiler -Executable $fxc -Arguments @('/nologo', '/Ges', '/WX', '/T', 'vs_5_0', '/E', 'VSMain', '/Fo', (Join-Path $output 'd3d11-vs.dxbc'), $source)
Invoke-Compiler -Executable $fxc -Arguments @('/nologo', '/Ges', '/WX', '/T', 'ps_5_0', '/E', 'PSMain', '/Fo', (Join-Path $output 'd3d11-ps.dxbc'), $source)
Invoke-Compiler -Executable $dxc -Arguments @('-Ges', '-WX', '-T', 'vs_6_0', '-E', 'VSMain', '-Fo', (Join-Path $output 'd3d12-vs.dxil'), $source)
Invoke-Compiler -Executable $dxc -Arguments @('-Ges', '-WX', '-T', 'ps_6_0', '-E', 'PSMain', '-Fo', (Join-Path $output 'd3d12-ps.dxil'), $source)

Get-FileHash -Algorithm SHA256 -LiteralPath @(
    $source,
    (Join-Path $output 'd3d11-vs.dxbc'),
    (Join-Path $output 'd3d11-ps.dxbc'),
    (Join-Path $output 'd3d12-vs.dxil'),
    (Join-Path $output 'd3d12-ps.dxil')
) | Select-Object Path, Hash
