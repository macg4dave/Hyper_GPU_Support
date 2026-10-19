$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$script:ProductExecutables = @('hyper-gpu-support.exe', 'hyper-gpu-runner.exe', 'hyper-gpu-guest.exe', 'd3d11-probe.exe', 'app-install.exe')

function Get-ReleaseMetadata {
    param([Parameter(Mandatory)][string]$WorkDirectory)
    $metadata = Get-Content -LiteralPath (Join-Path $WorkDirectory 'build-metadata.json') -Raw | ConvertFrom-Json
    if ($metadata.Version -notmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$') {
        throw 'Build metadata has no valid Cargo version.'
    }
    if ($metadata.Target -cne 'x86_64-pc-windows-msvc') { throw 'Official installers require the Windows x64 MSVC target.' }
    return $metadata
}

function Assert-ExactFiles {
    param([Parameter(Mandatory)][string]$Directory, [Parameter(Mandatory)][string[]]$RelativePaths)
    $root = (Get-Item -LiteralPath $Directory -Force).FullName
    $actual = @(Get-ChildItem -LiteralPath $root -Force -Recurse -File | ForEach-Object {
        if ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Artifact files must not be links.' }
        $_.FullName.Substring($root.Length + 1).Replace('\', '/')
    } | Sort-Object)
    foreach ($childDirectory in Get-ChildItem -LiteralPath $root -Force -Recurse -Directory) {
        if ($childDirectory.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Artifact directories must not be links.' }
    }
    $expected = @($RelativePaths | ForEach-Object { $_.Replace('\', '/') } | Sort-Object)
    if ($actual.Count -ne $expected.Count -or @(Compare-Object $expected $actual -CaseSensitive).Count -ne 0) {
        throw "Artifact has missing or unexpected files. Expected: $($expected -join ', '); found: $($actual -join ', ')"
    }
}

function Assert-SigningOnlyChange {
    param([Parameter(Mandatory)][string]$UnsignedPath, [Parameter(Mandatory)][string]$SignedPath)
    # Authenticode changes only the PE checksum, certificate-directory entry,
    # alignment padding and appended certificate table. Keep every other byte.
    $before = [IO.File]::ReadAllBytes($UnsignedPath)
    $after = [IO.File]::ReadAllBytes($SignedPath)
    if ($before.Length -lt 256 -or $after.Length -lt 256 -or $before[0] -ne 0x4d -or $before[1] -ne 0x5a) {
        throw 'Signing input is not a PE image.'
    }
    $pe = [BitConverter]::ToInt32($before, 0x3c)
    if ($pe -lt 0 -or $pe + 168 -gt $before.Length -or [BitConverter]::ToUInt32($before, $pe) -ne 0x4550) {
        throw 'Invalid PE header.'
    }
    $optional = $pe + 24
    $magic = [BitConverter]::ToUInt16($before, $optional)
    $security = switch ($magic) { 0x10b { $optional + 128 } 0x20b { $optional + 144 } default { throw 'Unsupported PE optional header.' } }
    $checksum = $optional + 64
    if ($security + 8 -gt $before.Length -or $security + 8 -gt $after.Length) { throw 'Truncated PE header.' }
    if ([BitConverter]::ToUInt32($before, $security) -ne 0 -or [BitConverter]::ToUInt32($before, $security + 4) -ne 0) {
        throw 'Signing request unexpectedly already has a certificate table.'
    }
    $certificateOffset = [BitConverter]::ToUInt32($after, $security)
    $certificateSize = [BitConverter]::ToUInt32($after, $security + 4)
    $alignedLength = [long]([Math]::Ceiling($before.Length / 8.0) * 8)
    if ($certificateOffset -ne $alignedLength -or $certificateSize -lt 8 -or [long]$certificateOffset + $certificateSize -ne $after.LongLength) {
        throw 'Signing changed the PE layout or omitted its certificate.'
    }
    $signedContent = [byte[]]::new($before.Length)
    [Array]::Copy($after, $signedContent, $before.Length)
    [Array]::Clear($before, $checksum, 4)
    [Array]::Clear($before, $security, 8)
    [Array]::Clear($signedContent, $checksum, 4)
    [Array]::Clear($signedContent, $security, 8)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        $expected = [BitConverter]::ToString($hasher.ComputeHash($before))
        $actual = [BitConverter]::ToString($hasher.ComputeHash($signedContent))
    } finally { $hasher.Dispose() }
    if ($expected -cne $actual) { throw 'Signing changed executable content.' }
    for ($i = $before.Length; $i -lt $certificateOffset; $i++) {
        if ($after[$i] -ne 0) { throw 'Unexpected nonzero signing alignment padding.' }
    }
}
