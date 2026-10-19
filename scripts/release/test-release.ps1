[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'release-common.ps1')
Add-Type -AssemblyName System.IO.Compression.FileSystem
Add-Type -AssemblyName System.IO.Compression

function Assert-Fails {
    param([scriptblock]$Action, [string]$Expected)
    $failed = $false
    try { & $Action } catch {
        $failed = $true
        if ($_.Exception.Message -notlike "*$Expected*") { throw }
    }
    if (-not $failed) { throw "Expected failure containing: $Expected" }
}

function Write-Hashes {
    param([string]$Directory, [string[]]$Names)
    $lines = foreach ($name in $Names) { (Get-FileHash -LiteralPath (Join-Path $Directory $name) -Algorithm SHA256).Hash.ToLowerInvariant() + '  ' + $name }
    [IO.File]::WriteAllText((Join-Path $Directory 'SHA256SUMS.txt'), ($lines -join "`n") + "`n")
}

function Write-Zip {
    param([string]$Path, [string[]]$Names, [string[]]$Contents)
    $archive = [IO.Compression.ZipFile]::Open($Path, [IO.Compression.ZipArchiveMode]::Create)
    try {
        for ($index = 0; $index -lt $Names.Count; $index++) {
            $entry = $archive.CreateEntry($Names[$index])
            $writer = [IO.StreamWriter]::new($entry.Open(), [Text.UTF8Encoding]::new($false))
            try { $writer.Write($Contents[$index]) } finally { $writer.Dispose() }
        }
    } finally { $archive.Dispose() }
}

$fixture = Join-Path ([IO.Path]::GetTempPath()) ('HyperGpuSupport-release-tests-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $fixture | Out-Null
try {
    $files = Join-Path $fixture 'files'
    New-Item -ItemType Directory -Path (Join-Path $files 'licenses') -Force | Out-Null
    [IO.File]::WriteAllText((Join-Path $files 'expected.exe'), 'fixture')
    [IO.File]::WriteAllText((Join-Path $files 'licenses/notice.txt'), 'licence fixture')
    Assert-ExactFiles $files @('expected.exe', 'licenses/notice.txt')
    Assert-Fails { Assert-ExactFiles $files @('expected.exe') } 'missing or unexpected'
    Assert-Fails { Assert-ExactFiles $files @('expected.exe', 'absent.exe') } 'missing or unexpected'
    Write-Hashes $files @('expected.exe')
    Assert-FileHashes $files @('expected.exe')
    Assert-Fails { Assert-FileHashes $files @('absent.exe') } 'exactly the expected'
    $hashText = [IO.File]::ReadAllText((Join-Path $files 'SHA256SUMS.txt'))
    [IO.File]::WriteAllText((Join-Path $files 'SHA256SUMS.txt'), $hashText + $hashText)
    Assert-Fails { Assert-FileHashes $files @('expected.exe') } 'Duplicate checksum'
    [IO.File]::WriteAllText((Join-Path $files 'SHA256SUMS.txt'), $hashText.Replace('expected.exe', '../expected.exe'))
    Assert-Fails { Assert-FileHashes $files @('expected.exe') } 'Malformed checksum'
    [IO.File]::WriteAllText((Join-Path $files 'SHA256SUMS.txt'), $hashText)
    [IO.File]::WriteAllText((Join-Path $files 'expected.exe'), 'modified')
    Assert-Fails { Assert-FileHashes $files @('expected.exe') } 'Checksum mismatch'
    [IO.File]::WriteAllText((Join-Path $files 'expected.exe'), 'fixture')
    Assert-Fails { Assert-UnsignedProduct (Join-Path $files 'expected.exe') '0.1.0' } 'not a PE image'
    # A certificate directory is forbidden even when WinTrust cannot validate it.
    # This minimal PE-header fixture is never executed or accepted as a product.
    $pe = [byte[]]::new(512)
    $pe[0] = 0x4d; $pe[1] = 0x5a
    [BitConverter]::GetBytes([int]128).CopyTo($pe, 0x3c)
    [BitConverter]::GetBytes([uint32]0x4550).CopyTo($pe, 128)
    [BitConverter]::GetBytes([uint16]0x20b).CopyTo($pe, 152)
    [BitConverter]::GetBytes([uint32]512).CopyTo($pe, 296)
    [BitConverter]::GetBytes([uint32]16).CopyTo($pe, 300)
    $pePath = Join-Path $fixture 'certificate-table.exe'
    [IO.File]::WriteAllBytes($pePath, $pe)
    Assert-Fails { Assert-UnsignedProduct $pePath '0.1.0' } 'unexpectedly has an Authenticode certificate table'
    Assert-Fails { Assert-UnsignedProduct $pePath '0.1.0' -RequireX64 } 'not Windows x64'

    $zip = Join-Path $fixture 'payload.zip'
    $zipFiles = @('expected.exe', 'licenses/notice.txt')
    Write-Zip $zip $zipFiles @('fixture', 'licence fixture')
    Assert-ZipPayload $zip $files $zipFiles
    $instructions = Join-Path $fixture 'install.txt'
    [IO.File]::WriteAllText($instructions, 'Installation instructions fixture')
    $withInstructions = Join-Path $fixture 'with-instructions.zip'
    Write-Zip $withInstructions ($zipFiles + @('install.txt')) @('fixture', 'licence fixture', 'Installation instructions fixture')
    Assert-ZipPayload $withInstructions $files $zipFiles -InstallInstructions $instructions
    Assert-Fails { Assert-ZipPayload $zip $files $zipFiles -InstallInstructions $instructions } 'missing or unexpected'
    Assert-Fails { Assert-ZipPayload $withInstructions $files $zipFiles } 'missing or unexpected'
    [IO.File]::WriteAllText($instructions, 'Changed installation instructions fixture')
    Assert-Fails { Assert-ZipPayload $withInstructions $files $zipFiles -InstallInstructions $instructions } 'ZIP payload differs: install.txt'
    $wrongZip = Join-Path $fixture 'wrong.zip'
    Write-Zip $wrongZip $zipFiles @('changed', 'licence fixture')
    Assert-Fails { Assert-ZipPayload $wrongZip $files $zipFiles } 'ZIP payload differs'
    Assert-Fails { Assert-ZipPayload $zip $files @('expected.exe') } 'missing or unexpected'
    $unsafeZip = Join-Path $fixture 'unsafe.zip'
    Write-Zip $unsafeZip @('../outside.exe') @('fixture')
    Assert-Fails { Assert-ZipPayload $unsafeZip $files $zipFiles } 'Unsafe ZIP entry'
    $duplicateZip = Join-Path $fixture 'duplicate.zip'
    Write-Zip $duplicateZip @('expected.exe', 'EXPECTED.exe') @('fixture', 'fixture')
    Assert-Fails { Assert-ZipPayload $duplicateZip $files $zipFiles } 'Duplicate ZIP entry'
    $backslashZip = Join-Path $fixture 'backslash.zip'
    Write-Zip $backslashZip @('licenses\notice.txt') @('licence fixture')
    Assert-Fails { Assert-ZipPayload $backslashZip $files $zipFiles } 'Unsafe ZIP entry'
    $directoryZip = Join-Path $fixture 'directory.zip'
    Write-Zip $directoryZip @('unwanted/') @('')
    Assert-Fails { Assert-ZipPayload $directoryZip $files $zipFiles } 'Unexpected ZIP directory'

    $candidate = Join-Path $fixture 'candidate'
    New-Item -ItemType Directory -Path $candidate | Out-Null
    $setupName = 'HyperGpuSupport-0.1.0-windows-x64-unsigned-setup.exe'
    $zipName = 'HyperGpuSupport-0.1.0-windows-x64-unsigned-binaries.zip'
    [IO.File]::WriteAllText((Join-Path $candidate $setupName), 'not an installer')
    Copy-Item -LiteralPath $zip -Destination (Join-Path $candidate $zipName)
    Write-Hashes $candidate @($setupName, $zipName)
    $payload = Join-Path $fixture 'payload'
    New-Item -ItemType Directory -Path (Join-Path $payload 'licenses') -Force | Out-Null
    foreach ($name in $script:ProductExecutables) { [IO.File]::WriteAllText((Join-Path $payload $name), 'not an executable') }
    [IO.File]::WriteAllText((Join-Path $payload 'LICENSE'), 'fixture licence')
    [IO.File]::WriteAllText((Join-Path $payload 'licenses/THIRD_PARTY_NOTICES.md'), 'fixture notices')
    Write-Hashes $payload $script:ProductExecutables
    $metadata = @{
        Version = '0.1.0'; Target = 'x86_64-pc-windows-msvc'; Toolchain = '1.90.0'
        PayloadDirectory = $payload; SetupPath = (Join-Path $candidate $setupName); BinaryZipPath = (Join-Path $candidate $zipName)
    } | ConvertTo-Json
    $metadataPath = Join-Path $fixture 'build-metadata.json'
    [IO.File]::WriteAllText($metadataPath, $metadata)
    $readback = Get-ReleaseMetadata $fixture
    if ($readback.Version -cne '0.1.0') { throw 'Metadata readback failed.' }
    [IO.File]::WriteAllText($metadataPath, $metadata.Replace('x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc'))
    Assert-Fails { Get-ReleaseMetadata $fixture } 'Windows x64'
    [IO.File]::WriteAllText($metadataPath, $metadata.Replace('0.1.0', 'bad-version'))
    Assert-Fails { Get-ReleaseMetadata $fixture } 'valid numeric Cargo version'
    [IO.File]::WriteAllText($metadataPath, '{"Version":"0.1.0"}')
    Assert-Fails { Get-ReleaseMetadata $fixture } 'missing Target'
    [IO.File]::WriteAllText($metadataPath, $metadata)
    $releaseOutput = Join-Path $fixture 'release-ready'
    Assert-Fails {
        & (Join-Path $PSScriptRoot 'finalize-release.ps1') -WorkDirectory $fixture -UnpackerPath 'must-not-run.exe' -OutputDirectory $releaseOutput
    } 'not a PE image'
    if (Test-Path -LiteralPath $releaseOutput) { throw 'Invalid payload produced release-ready output.' }
    [IO.File]::WriteAllText((Join-Path $candidate $setupName), 'changed installer')
    Assert-Fails {
        & (Join-Path $PSScriptRoot 'finalize-release.ps1') -WorkDirectory $fixture -UnpackerPath 'must-not-run.exe' -OutputDirectory $releaseOutput
    } 'Checksum mismatch'
    if (Test-Path -LiteralPath $releaseOutput) { throw 'Corrupt setup produced release-ready output.' }
    Write-Host 'Release regression checks passed; only isolated fixtures were used, and no setup executable was run.'
} finally {
    $resolved = [IO.Path]::GetFullPath($fixture)
    $temp = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\') + '\'
    if (-not $resolved.StartsWith($temp, [StringComparison]::OrdinalIgnoreCase)) { throw 'Refusing fixture cleanup outside temporary directory.' }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
