[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Destination
)

# Copy notices for the enabled Windows runtime dependency graph, not the entire registry.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repositoryRoot
try {
    $metadataText = & cargo metadata --locked --format-version 1 --filter-platform x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'Cargo metadata failed while collecting dependency licences.' }
    $metadata = ($metadataText -join "`n") | ConvertFrom-Json
    # Metadata includes optional packages that Cargo does not actually build.
    # Cargo tree supplies the enabled normal edges without build/dev dependencies.
    $runtimeTree = & cargo tree --locked -p hyper-gpu-support -p app-install --target x86_64-pc-windows-msvc --edges normal --prefix none --format '{p}'
    if ($LASTEXITCODE -ne 0) { throw 'Cargo runtime dependency enumeration failed.' }
    $runtime = [System.Collections.Generic.HashSet[string]]::new()
    foreach ($line in $runtimeTree) {
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        if ($line -notmatch '^([A-Za-z0-9_-]+) v([^ ]+)(?: .*)?$') { throw "Unexpected cargo tree entry: $line" }
        [void]$runtime.Add("$($Matches[1])/$($Matches[2])")
    }
    $destinationPath = [System.IO.Path]::GetFullPath($Destination)
    New-Item -ItemType Directory -Path $destinationPath -Force | Out-Null
    $inventory = @()
    $rows = [System.Collections.Generic.List[string]]::new()
    foreach ($package in @($metadata.packages | Where-Object { $runtime.Contains("$($_.name)/$($_.version)") } | Sort-Object name, version)) {
        if ($package.id -in $metadata.workspace_members) { continue }
        if ($package.name -notmatch '^[A-Za-z0-9_-]+$' -or $package.version -notmatch '^[A-Za-z0-9.+-]+$') {
            throw "Unexpected Cargo package name/version: $($package.name) $($package.version)"
        }
        $source = Split-Path -Parent $package.manifest_path
        $slug = "$($package.name)-$($package.version)"
        $packageDestination = Join-Path $destinationPath $slug
        $files = @(Get-ChildItem -LiteralPath $source -File | Where-Object {
            $_.Name -match '^(LICENSE|LICENCE|COPYING|COPYRIGHT|NOTICE|UNLICENSE)([.-].*)?$'
        })
        foreach ($directory in @(Get-ChildItem -LiteralPath $source -Directory | Where-Object {
            $_.Name -match '^(LICENSES?|LICENCES?|COPYING)$'
        })) {
            $files += @(Get-ChildItem -LiteralPath $directory.FullName -File -Recurse)
        }
        if ($package.license_file) {
            $specifiedFile = Join-Path $source $package.license_file
            if (-not (Test-Path -LiteralPath $specifiedFile -PathType Leaf)) {
                throw "Missing declared licence file for $slug"
            }
            $files += Get-Item -LiteralPath $specifiedFile
        }
        $files = @($files | Sort-Object FullName -Unique)
        if ($files.Count -eq 0) {
            $override = Join-Path $repositoryRoot "licenses/crate-overrides/$slug"
            if (-not (Test-Path -LiteralPath $override -PathType Container)) {
                throw "No redistribution licence text found for $slug; review before packaging."
            }
            $source = $override
            $files = @(Get-ChildItem -LiteralPath $override -File)
            if ($files.Count -eq 0) { throw "Empty licence override for $slug" }
        }
        foreach ($file in $files) {
            $relative = $file.FullName.Substring($source.Length).TrimStart('\', '/')
            $target = Join-Path $packageDestination $relative
            New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force | Out-Null
            Copy-Item -LiteralPath $file.FullName -Destination $target
        }
        $expression = if ($package.license) { $package.license } else { 'See declared licence file' }
        $rows.Add("| $($package.name) | $($package.version) | $expression | [$slug]($slug/) |")
        $inventory += [ordered]@{ name = $package.name; version = $package.version; license = $expression; repository = $package.repository }
    }
    $header = @(
        '# Windows runtime dependency notices', '',
        'Generated from the locked, enabled Windows dependency graph. Alternative licence expressions describe upstream choices; they do not imply that every alternative applies simultaneously.', '',
        '| Package | Version | Upstream licence expression | Licence texts |',
        '| --- | --- | --- | --- |'
    )
    [System.IO.File]::WriteAllLines((Join-Path $destinationPath 'DEPENDENCIES.md'), @($header) + @($rows), [System.Text.UTF8Encoding]::new($false))
    [System.IO.File]::WriteAllText((Join-Path $destinationPath 'dependencies.json'), (ConvertTo-Json -InputObject @($inventory) -Depth 4), [System.Text.UTF8Encoding]::new($false))
    Write-Host "Collected licence texts for $($inventory.Count) Windows runtime packages."
}
finally { Pop-Location }
