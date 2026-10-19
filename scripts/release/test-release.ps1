[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'release-common.ps1')

function Assert-Fails {
    param([scriptblock]$Action, [string]$Expected)
    $failed = $false
    try { & $Action } catch {
        $failed = $true
        if ($_.Exception.Message -notlike "*$Expected*") { throw }
    }
    if (-not $failed) { throw "Expected failure containing: $Expected" }
}

$fixture = Join-Path ([IO.Path]::GetTempPath()) ('HyperGpuSupport-release-tests-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $fixture | Out-Null
try {
    $files = Join-Path $fixture 'files'
    New-Item -ItemType Directory -Path $files | Out-Null
    [IO.File]::WriteAllText((Join-Path $files 'expected.exe'), 'fixture')
    Assert-ExactFiles $files @('expected.exe')
    [IO.File]::WriteAllText((Join-Path $files 'unexpected.txt'), 'extra')
    Assert-Fails { Assert-ExactFiles $files @('expected.exe') } 'missing or unexpected'
    Assert-Fails { Assert-ExactFiles $files @('expected.exe', 'absent.exe') } 'missing or unexpected'
    New-Item -ItemType Directory -Path (Join-Path $files 'licenses') | Out-Null
    [IO.File]::WriteAllText((Join-Path $files 'licenses/notice.txt'), 'licence fixture')
    Assert-ExactFiles $files @('expected.exe', 'unexpected.txt', 'licenses/notice.txt')

    # This is only a PE-content-preservation fixture, never a signature fixture.
    # The real signature verifier below must reject it before any release output.
    $before = [byte[]]::new(512)
    $before[0] = 0x4d; $before[1] = 0x5a
    [BitConverter]::GetBytes([int]128).CopyTo($before, 0x3c)
    [BitConverter]::GetBytes([uint32]0x4550).CopyTo($before, 128)
    [BitConverter]::GetBytes([uint16]0x20b).CopyTo($before, 152)
    $before[400] = 42
    $after = [byte[]]::new(528)
    $before.CopyTo($after, 0)
    [BitConverter]::GetBytes([uint32]512).CopyTo($after, 296)
    [BitConverter]::GetBytes([uint32]16).CopyTo($after, 300)
    [BitConverter]::GetBytes([uint32]1234).CopyTo($after, 216)
    $unsigned = Join-Path $fixture 'unsigned.exe'
    $signedLayout = Join-Path $fixture 'signed-layout.exe'
    [IO.File]::WriteAllBytes($unsigned, $before)
    [IO.File]::WriteAllBytes($signedLayout, $after)
    Assert-SigningOnlyChange $unsigned $signedLayout
    $after[400] = 43
    [IO.File]::WriteAllBytes($signedLayout, $after)
    Assert-Fails { Assert-SigningOnlyChange $unsigned $signedLayout } 'changed executable content'
    $after[400] = 42
    [BitConverter]::GetBytes([uint32]504).CopyTo($after, 296)
    [IO.File]::WriteAllBytes($signedLayout, $after)
    Assert-Fails { Assert-SigningOnlyChange $unsigned $signedLayout } 'changed the PE layout'
    [IO.File]::WriteAllText($signedLayout, 'not an executable')
    Assert-Fails { Assert-SigningOnlyChange $unsigned $signedLayout } 'not a PE image'

    $metadata = @{ Version = '0.1.0'; Target = 'x86_64-pc-windows-msvc'; SetupPath = $unsigned } | ConvertTo-Json
    [IO.File]::WriteAllText((Join-Path $fixture 'build-metadata.json'), $metadata)
    $readback = Get-ReleaseMetadata $fixture
    if ($readback.Version -cne '0.1.0') { throw 'Metadata readback failed.' }
    $metadata.Replace('x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc') | Set-Content -LiteralPath (Join-Path $fixture 'build-metadata.json')
    Assert-Fails { Get-ReleaseMetadata $fixture } 'Windows x64'
    $metadata.Replace('0.1.0', 'bad-version') | Set-Content -LiteralPath (Join-Path $fixture 'build-metadata.json')
    Assert-Fails { Get-ReleaseMetadata $fixture } 'valid Cargo version'
    Assert-Fails { & (Join-Path $PSScriptRoot 'verify-signatures.ps1') -Path $unsigned -ExpectedPublisher 'CN=Fixture' -ExpectedVersion '0.1.0' } 'Authenticode validation failed'
    [IO.File]::WriteAllText((Join-Path $fixture 'build-metadata.json'), $metadata)
    $unsignedSetupDirectory = Join-Path $fixture 'unsigned-setup'
    New-Item -ItemType Directory -Path $unsignedSetupDirectory | Out-Null
    Copy-Item -LiteralPath $unsigned -Destination (Join-Path $unsignedSetupDirectory 'HyperGpuSupport-0.1.0-windows-x64-setup.exe')
    $releaseOutput = Join-Path $fixture 'release-ready'
    Assert-Fails {
        & (Join-Path $PSScriptRoot 'finalize-release.ps1') -WorkDirectory $fixture -SignedSetupDirectory $unsignedSetupDirectory -SignedUninstallerPath $unsigned -ExpectedPublisher 'CN=Fixture' -UnpackerPath $unsigned -OutputDirectory $releaseOutput
    } 'changed the PE layout or omitted its certificate'
    if (Test-Path -LiteralPath $releaseOutput) { throw 'Unsigned setup produced release-ready output.' }
    Write-Host 'Release staging negative checks passed; no signing service or setup executable was run.'
} finally {
    $resolved = [IO.Path]::GetFullPath($fixture)
    $temp = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\') + '\'
    if (-not $resolved.StartsWith($temp, [StringComparison]::OrdinalIgnoreCase)) { throw 'Refusing fixture cleanup outside temporary directory.' }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
