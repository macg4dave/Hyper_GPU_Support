$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$script:ProductExecutables = @('hyper-gpu-support.exe', 'hyper-gpu-runner.exe', 'hyper-gpu-guest.exe', 'd3d11-probe.exe', 'app-install.exe')

function Get-ReleaseMetadata {
    param([Parameter(Mandatory)][string]$WorkDirectory)
    $metadata = Get-Content -LiteralPath (Join-Path $WorkDirectory 'build-metadata.json') -Raw | ConvertFrom-Json
    foreach ($property in 'Version', 'Target', 'Toolchain', 'PayloadDirectory', 'SetupPath', 'BinaryZipPath') {
        if (-not $metadata.PSObject.Properties[$property] -or [string]::IsNullOrWhiteSpace([string]$metadata.$property)) {
            throw "Build metadata is missing $property."
        }
    }
    if ($metadata.Version -notmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$') {
        throw 'Build metadata has no valid numeric Cargo version.'
    }
    if ($metadata.Target -cne 'x86_64-pc-windows-msvc') { throw 'Official packages require the Windows x64 MSVC target.' }
    return $metadata
}

function Get-ArtifactFiles {
    param([Parameter(Mandatory)][string]$Directory)
    $rootItem = Get-Item -LiteralPath $Directory -Force
    if (-not $rootItem.PSIsContainer -or $rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Artifact root must be a regular directory.' }
    $root = $rootItem.FullName
    # Inspect each directory before descending so a link is never traversed.
    $pending = [Collections.Generic.Stack[string]]::new()
    $pending.Push($root)
    $actual = [Collections.Generic.List[string]]::new()
    while ($pending.Count -gt 0) {
        foreach ($item in Get-ChildItem -LiteralPath $pending.Pop() -Force) {
            if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Artifact files and directories must not be links.' }
            if ($item.PSIsContainer) { $pending.Push($item.FullName) } else { $actual.Add($item.FullName.Substring($root.Length + 1).Replace('\', '/')) }
        }
    }
    return @($actual | Sort-Object)
}

function Assert-ExactFiles {
    param([Parameter(Mandatory)][string]$Directory, [Parameter(Mandatory)][string[]]$RelativePaths)
    $actual = @(Get-ArtifactFiles $Directory)
    $expected = @($RelativePaths | ForEach-Object { $_.Replace('\', '/') } | Sort-Object)
    if ($actual.Count -ne $expected.Count -or @(Compare-Object $expected @($actual | Sort-Object) -CaseSensitive).Count -ne 0) {
        throw "Artifact has missing or unexpected files. Expected: $($expected -join ', '); found: $($actual -join ', ')"
    }
}

function Assert-FileHashes {
    param([Parameter(Mandatory)][string]$Directory, [Parameter(Mandatory)][string[]]$Names)
    $manifest = Join-Path $Directory 'SHA256SUMS.txt'
    $hashes = [Collections.Generic.Dictionary[string, string]]::new([StringComparer]::Ordinal)
    foreach ($line in [IO.File]::ReadAllLines($manifest)) {
        if ($line -cnotmatch '^([a-f0-9]{64})  ([A-Za-z0-9._-]+)$') { throw "Malformed checksum manifest: $manifest" }
        if ($hashes.ContainsKey($Matches[2])) { throw "Duplicate checksum entry: $($Matches[2])" }
        $hashes.Add($Matches[2], $Matches[1])
    }
    if ($hashes.Count -ne $Names.Count -or @(Compare-Object @($Names | Sort-Object) @($hashes.Keys | Sort-Object) -CaseSensitive).Count -ne 0) {
        throw 'Checksum manifest must name exactly the expected files.'
    }
    foreach ($name in $Names) {
        if ((Get-FileHash -LiteralPath (Join-Path $Directory $name) -Algorithm SHA256).Hash.ToLowerInvariant() -cne $hashes[$name]) {
            throw "Checksum mismatch: $name"
        }
    }
}

function Assert-UnsignedProduct {
    param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][string]$Version, [switch]$RequireX64)
    $item = Get-Item -LiteralPath $Path -Force
    if ($item.PSIsContainer -or $item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Release input must be a regular file: $Path" }
    $bytes = [IO.File]::ReadAllBytes($item.FullName)
    if ($bytes.Length -lt 256 -or $bytes[0] -ne 0x4d -or $bytes[1] -ne 0x5a) { throw "Release input is not a PE image: $Path" }
    $pe = [BitConverter]::ToInt32($bytes, 0x3c)
    if ($pe -lt 0 -or $pe + 168 -gt $bytes.Length -or [BitConverter]::ToUInt32($bytes, $pe) -ne 0x4550) { throw "Invalid PE header: $Path" }
    if ($RequireX64 -and [BitConverter]::ToUInt16($bytes, $pe + 4) -ne 0x8664) { throw "Product executable is not Windows x64: $Path" }
    $optional = $pe + 24
    $security = switch ([BitConverter]::ToUInt16($bytes, $optional)) {
        0x10b { $optional + 128 }
        0x20b { $optional + 144 }
        default { throw "Unsupported PE optional header: $Path" }
    }
    if ($security + 8 -gt $bytes.Length) { throw "Truncated PE header: $Path" }
    if ([BitConverter]::ToUInt32($bytes, $security) -ne 0 -or [BitConverter]::ToUInt32($bytes, $security + 4) -ne 0) {
        throw "Unsigned release input unexpectedly has an Authenticode certificate table: $Path"
    }
    $signature = Get-AuthenticodeSignature -LiteralPath $item.FullName
    if ($signature.Status -ne [Management.Automation.SignatureStatus]::NotSigned -or $signature.SignerCertificate) {
        throw "Expected an unsigned release input: $Path ($($signature.Status))"
    }
    $info = [Diagnostics.FileVersionInfo]::GetVersionInfo($item.FullName)
    if ($info.ProductName.Trim() -cne 'Hyper GPU Support' -or $info.ProductVersion.Trim() -cne $Version -or $info.FileVersion.Trim() -cne "$Version.0") {
        throw "Product name/version metadata mismatch: $Path"
    }
}

function Assert-ZipPayload {
    param([Parameter(Mandatory)][string]$ZipPath, [Parameter(Mandatory)][string]$PayloadDirectory, [Parameter(Mandatory)][string[]]$RelativePaths, [string]$InstallInstructions)
    $expectedPaths = @($RelativePaths)
    if ($InstallInstructions) { $expectedPaths += 'install.txt' }
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    Add-Type -AssemblyName System.IO.Compression
    $archive = [IO.Compression.ZipFile]::OpenRead($ZipPath)
    try {
        $entries = [Collections.Generic.Dictionary[string, object]]::new([StringComparer]::OrdinalIgnoreCase)
        foreach ($entry in $archive.Entries) {
            $name = $entry.FullName
            if ([string]::IsNullOrWhiteSpace($name) -or $name.Contains('\') -or $name.Contains(':') -or $name.StartsWith('/') -or
                @($name.TrimEnd('/').Split('/') | Where-Object { $_ -in @('', '.', '..') }).Count -gt 0) {
                throw "Unsafe ZIP entry: $name"
            }
            if ((($entry.ExternalAttributes -shr 16) -band 0xf000) -eq 0xa000) { throw "ZIP links are not permitted: $name" }
            if ($name.EndsWith('/') -and @($expectedPaths | Where-Object { $_.StartsWith($name, [StringComparison]::Ordinal) }).Count -eq 0) {
                throw "Unexpected ZIP directory: $name"
            }
            if ($entries.ContainsKey($name)) { throw "Duplicate ZIP entry: $name" }
            $entries.Add($name, $entry)
        }
        $files = @($entries.Keys | Where-Object { -not $_.EndsWith('/') } | Sort-Object)
        if ($files.Count -ne $expectedPaths.Count -or @(Compare-Object @($expectedPaths | Sort-Object) $files -CaseSensitive).Count -ne 0) {
            throw 'ZIP has missing or unexpected payload files.'
        }
        foreach ($name in $files) {
            $stream = $entries[$name].Open()
            $hasher = [Security.Cryptography.SHA256]::Create()
            try { $hash = [BitConverter]::ToString($hasher.ComputeHash($stream)).Replace('-', '') } finally { $hasher.Dispose(); $stream.Dispose() }
            $source = if ($name -ceq 'install.txt' -and $InstallInstructions) { $InstallInstructions } else { Join-Path $PayloadDirectory $name }
            if ($hash -cne (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash) { throw "ZIP payload differs: $name" }
        }
    } finally { $archive.Dispose() }
}
