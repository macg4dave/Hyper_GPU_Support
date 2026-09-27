[CmdletBinding()]
param(
    [string] $OutputPath = 'local\evidence\CORE-020-host-controls.json'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$resolvedOutput = Join-Path $repositoryRoot $OutputPath
$outputDirectory = Split-Path -Parent $resolvedOutput
$cudaDirectory = Join-Path $repositoryRoot 'probes\cuda\compiled'
$expectedHash = '00f88da6c22b46ab45bfc5fbc6659e601ebcefe324d52a3302e614d4a7fb3de4'
$outputLimit = 1MB
$suiteTimeout = [TimeSpan]::FromSeconds(90)
$suiteStopwatch = [Diagnostics.Stopwatch]::StartNew()
$results = [Collections.Generic.List[object]]::new()

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

function Invoke-BoundedProcess {
    param(
        [Parameter(Mandatory)]
        [string] $Name,

        [Parameter(Mandatory)]
        [string] $Executable,

        [string[]] $Arguments = @(),

        [Parameter(Mandatory)]
        [string] $WorkingDirectory,

        [Parameter(Mandatory)]
        [int] $TimeoutSeconds,

        [Parameter(Mandatory)]
        [string] $Phase
    )

    if ($suiteStopwatch.Elapsed -ge $suiteTimeout) {
        throw "The host control suite exceeded its $($suiteTimeout.TotalSeconds)-second deadline."
    }
    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $Executable
    $startInfo.WorkingDirectory = $WorkingDirectory
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    if ($Arguments | Where-Object { $_ -match '[\s"]' }) {
        throw "$Name uses an unsupported whitespace or quote in its fixed argument list."
    }
    $startInfo.Arguments = $Arguments -join ' '

    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    $stopwatch = [Diagnostics.Stopwatch]::StartNew()
    if (-not $process.Start()) {
        throw "Failed to start $Name."
    }
    $stdoutTask = $process.StandardOutput.ReadToEndAsync()
    $stderrTask = $process.StandardError.ReadToEndAsync()
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        $process.Kill($true)
        $process.WaitForExit()
        throw "$Name timed out after $TimeoutSeconds seconds."
    }
    $stopwatch.Stop()
    $stdout = $stdoutTask.GetAwaiter().GetResult().Trim()
    $stderr = $stderrTask.GetAwaiter().GetResult().Trim()
    if ([Text.Encoding]::UTF8.GetByteCount($stdout) + [Text.Encoding]::UTF8.GetByteCount($stderr) -gt $outputLimit) {
        throw "$Name exceeded the 1 MiB output limit."
    }
    if ($process.ExitCode -ne 0) {
        throw "$Name exited $($process.ExitCode): $stderr"
    }

    [pscustomobject]@{
        Name = $Name
        Phase = $Phase
        Command = @($Executable) + $Arguments
        ExitCode = $process.ExitCode
        DurationMs = $stopwatch.ElapsedMilliseconds
        Stdout = $stdout
        Stderr = $stderr
    }
}

function Assert-D3DResult {
    param(
        [Parameter(Mandatory)]
        [object] $Result,

        [Parameter(Mandatory)]
        [string] $ExpectedProbe
    )

    $report = $Result.Stdout | ConvertFrom-Json
    if ($report.schema -ne 1 -or $report.probe -ne $ExpectedProbe -or $report.status -ne 'pass' -or
        $report.output_sha256 -ne $expectedHash -or $report.adapter.luid -eq '00000000:00000000' -or
        $report.adapter.software -or $report.adapter.indirect_display) {
        throw "$ExpectedProbe returned an invalid success report."
    }
    return $report
}

function Assert-CudaIdentity {
    param(
        [Parameter(Mandatory)]
        [object] $Result
    )

    $report = $Result.Stdout | ConvertFrom-Json
    if ($report.schema -ne 1 -or $report.probe -ne 'cuda-identity' -or $report.status -ne 'pass' -or
        $report.ordinal -ne 0 -or $report.compute_capability -ne '12.0' -or
        $report.luid -ne $report.dxgi_luid) {
        throw 'CUDA identity companion returned an invalid success report.'
    }
    return $report
}

Invoke-Native -Executable 'cargo.exe' -Arguments @(
    'build', '--release', '--target', 'x86_64-pc-windows-msvc',
    '--bin', 'd3d11-probe', '--bin', 'd3d12-probe', '--bin', 'cuda-identity'
)
$metadata = cargo.exe metadata --no-deps --format-version 1 | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) {
    throw 'cargo metadata failed.'
}
$binaryRoot = Join-Path $metadata.target_directory 'x86_64-pc-windows-msvc\release'
$d3d11 = Join-Path $binaryRoot 'd3d11-probe.exe'
$d3d12 = Join-Path $binaryRoot 'd3d12-probe.exe'
$cudaIdentity = Join-Path $binaryRoot 'cuda-identity.exe'
$vectorAdd = Join-Path $cudaDirectory 'vectorAddDrv.exe'
foreach ($path in @($d3d11, $d3d12, $cudaIdentity, $vectorAdd, (Join-Path $cudaDirectory 'vectorAdd_kernel64.fatbin'))) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Required probe artifact is missing: $path"
    }
}

foreach ($phase in @('warm-up', 'measured-1', 'measured-2', 'measured-3')) {
    $d3d11Result = Invoke-BoundedProcess -Name 'd3d11-offscreen' -Executable $d3d11 `
        -WorkingDirectory $repositoryRoot -TimeoutSeconds 15 -Phase $phase
    $d3d11Report = Assert-D3DResult -Result $d3d11Result -ExpectedProbe 'd3d11-offscreen'
    $results.Add($d3d11Result)

    $d3d12Result = Invoke-BoundedProcess -Name 'd3d12-offscreen' -Executable $d3d12 `
        -WorkingDirectory $repositoryRoot -TimeoutSeconds 15 -Phase $phase
    $d3d12Report = Assert-D3DResult -Result $d3d12Result -ExpectedProbe 'd3d12-offscreen'
    $results.Add($d3d12Result)

    $cudaIdentityResult = Invoke-BoundedProcess -Name 'cuda-identity' -Executable $cudaIdentity `
        -WorkingDirectory $repositoryRoot -TimeoutSeconds 15 -Phase $phase
    $cudaReport = Assert-CudaIdentity -Result $cudaIdentityResult
    $results.Add($cudaIdentityResult)

    if ($d3d11Report.adapter.luid -ne $d3d12Report.adapter.luid -or
        $d3d11Report.adapter.luid -ne $cudaReport.luid) {
        throw "Cross-API LUID mismatch during ${phase}."
    }

    $cudaResult = Invoke-BoundedProcess -Name 'cuda-vectorAddDrv' -Executable $vectorAdd `
        -Arguments @('--device=0') -WorkingDirectory $cudaDirectory -TimeoutSeconds 30 -Phase $phase
    if ($cudaResult.Stdout -notmatch '(?m)^Result = PASS\s*$') {
        throw "CUDA vectorAddDrv did not report PASS during ${phase}."
    }
    $results.Add($cudaResult)
}

$suiteStopwatch.Stop()
if ($suiteStopwatch.Elapsed -gt $suiteTimeout) {
    throw "The host control suite exceeded its $($suiteTimeout.TotalSeconds)-second deadline."
}

$operatingSystem = Get-CimInstance -ClassName Win32_OperatingSystem
$summary = [ordered]@{
    Schema = 1
    Status = 'pass'
    Utc = [DateTime]::UtcNow.ToString('o')
    Machine = $env:COMPUTERNAME
    Windows = "$($operatingSystem.Caption) $($operatingSystem.Version) build $($operatingSystem.BuildNumber)"
    Architecture = $env:PROCESSOR_ARCHITECTURE
    SessionId = [Diagnostics.Process]::GetCurrentProcess().SessionId
    SuiteDurationMs = $suiteStopwatch.ElapsedMilliseconds
    ExpectedImageSha256 = $expectedHash
    Results = $results
}
New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $resolvedOutput -Encoding utf8NoBOM
Write-Host "Host controls passed: one warm-up plus three measured repetitions for D3D11, D3D12 and CUDA."
Write-Host "Evidence: $resolvedOutput"
