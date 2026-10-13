# Non-elevated, non-hardware gate. Catalogue tests use local files and headless windows.
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Set-Location -LiteralPath $root
if ([Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Run this gate from a non-elevated shell.'
}
foreach ($tool in @('git', 'python', 'cargo')) { Get-Command $tool -ErrorAction Stop | Out-Null }
$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffZ')
$evidence = Join-Path $root "local/evidence/m1-gate-$stamp"
New-Item -ItemType Directory -Path $evidence -Force | Out-Null
$stages = @(
    @{ Name = 'Catalogue validator tests'; Tool = 'cargo'; Arguments = @('test', '--locked', '--manifest-path', 'tools/ui-text/standalone/Cargo.toml') },
    @{ Name = 'Active catalogue'; Tool = 'cargo'; Arguments = @('run', '--locked', '--manifest-path', 'tools/ui-text/standalone/Cargo.toml', '--', 'resources/i18n/en-GB') },
    @{ Name = 'Copy scan tests'; Tool = 'python'; Arguments = @('-B', 'scripts/testing/test_check_ui_copy.py') },
    @{ Name = 'Copy scan'; Tool = 'python'; Arguments = @('-B', 'scripts/testing/check-ui-copy.py') },
    @{ Name = 'Documentation'; Tool = 'python'; Arguments = @('scripts/testing/check-docs.py') },
    @{ Name = 'Formatting'; Tool = 'cargo'; Arguments = @('fmt', '--all', '--', '--check') },
    @{ Name = 'Non-hardware tests'; Tool = 'cargo'; Arguments = @('test', '--locked', '--all-targets') },
    @{ Name = 'Lint'; Tool = 'cargo'; Arguments = @('clippy', '--locked', '--all-targets', '--', '-D', 'warnings') },
    @{ Name = 'Product binaries'; Tool = 'cargo'; Arguments = @('build', '--locked', '--bins') },
    @{ Name = 'API documentation'; Tool = 'cargo'; Arguments = @('doc', '--locked', '--no-deps') }
)
$report = [ordered]@{
    revision = (& git rev-parse HEAD)
    working_tree = @(& git status --short)
    diff_stat = @(& git -c core.safecrlf=false diff --stat)
    os = [Environment]::OSVersion.VersionString
    architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
    elevated = $false
    cargo = (& cargo --version)
    rustc = (& rustc --version)
    input_hashes = @(& git ls-files --cached --others --exclude-standard | Where-Object {
        $_ -match '^(src/|scripts/testing/|resources/i18n/|tools/ui-text/|tests/|Cargo\.(toml|lock)$|build\.rs$|rust-toolchain\.toml$|\.cargo/)'
    } | ForEach-Object {
        if (Test-Path -LiteralPath $_ -PathType Leaf) {
            [ordered]@{ path = $_; sha256 = (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash.ToLowerInvariant() }
        }
    })
    started_utc = [DateTime]::UtcNow.ToString('o')
    result = 'Running'
    exclusions = @('GUI launch/render/interaction', 'Credentials/vault access', 'Installed runner/worker integration', 'Protected writes/setup/enrollment', 'Guest/VM/GPU access and workloads', 'Package trials')
    stages = @($stages | ForEach-Object { [ordered]@{ name = $_.Name; command = "$($_.Tool) $($_.Arguments -join ' ')"; result = 'Not run' } })
}
$reportPath = Join-Path $evidence 'report.json'
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding UTF8
Write-Host "Evidence: $evidence"
for ($index = 0; $index -lt $stages.Count; $index++) {
    $stage = $stages[$index]
    $entry = $report.stages[$index]
    $entry.started_utc = [DateTime]::UtcNow.ToString('o')
    $entry.result = 'Running'
    $report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding UTF8
    Write-Host "Running $($entry.command)"
    $arguments = $stage.Arguments
    # Windows PowerShell wraps native stderr as ErrorRecords, including Cargo's
    # ordinary progress output. Stage success is the native exit code.
    $ErrorActionPreference = 'Continue'
    try {
        & $stage.Tool @arguments 2>&1 | ForEach-Object { $_.ToString() } | Tee-Object -FilePath (Join-Path $evidence "stage-$index.txt")
        $code = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = 'Stop'
    }
    $entry.exit_code = $code
    $entry.finished_utc = [DateTime]::UtcNow.ToString('o')
    $entry.result = if ($code -eq 0) { 'Passed' } else { 'Failed' }
    $report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding UTF8
    if ($code -ne 0) {
        $report.result = 'Failed'
        $report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding UTF8
        throw "Gate failed: $($entry.name). Remaining stages are Not run. Evidence: $evidence"
    }
}
$report.result = 'Passed'
$report.finished_utc = [DateTime]::UtcNow.ToString('o')
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding UTF8
Write-Host "M1 non-hardware gate passed. Evidence: $evidence"
exit 0
