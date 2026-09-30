[CmdletBinding()]
param(
    [string] $OutputPath,
    [string] $ProjectConfigurationPath = (Join-Path $PSScriptRoot '..\..\config\project.toml'),
    [switch] $SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
. (Join-Path $repositoryRoot 'scripts\common\project-config.ps1')
$projectConfiguration = Import-ProjectConfiguration -Path $ProjectConfigurationPath
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = [string](Get-ProjectConfigurationValue $projectConfiguration 'tests.host_probes.output_path')
}
if ($SelfTest) {
    $OutputPath = [string](Get-ProjectConfigurationValue $projectConfiguration 'tests.host_probes.self_test_output_path')
}
$localRoot = Join-Path $repositoryRoot 'local'
$resolvedOutput = [IO.Path]::GetFullPath((Join-Path $repositoryRoot $OutputPath))
if (-not $resolvedOutput.StartsWith("$localRoot\", [StringComparison]::OrdinalIgnoreCase)) {
    throw "OutputPath must remain beneath the repository local directory: $resolvedOutput"
}
$outputDirectory = Split-Path -Parent $resolvedOutput
$cudaDirectory = Join-Path $repositoryRoot 'probes\cuda\compiled'
$expectedHash = '00f88da6c22b46ab45bfc5fbc6659e601ebcefe324d52a3302e614d4a7fb3de4'
$outputLimit = 1MB
$processTimeoutSeconds = [int](Get-ProjectConfigurationValue $projectConfiguration 'tests.host_probes.process_timeout_seconds')
$cudaTimeoutSeconds = [int](Get-ProjectConfigurationValue $projectConfiguration 'tests.host_probes.cuda_timeout_seconds')
$suiteTimeout = [TimeSpan]::FromSeconds([uint64](Get-ProjectConfigurationValue $projectConfiguration 'tests.host_probes.suite_timeout_seconds'))
$repetitions = [int](Get-ProjectConfigurationValue $projectConfiguration 'tests.host_probes.repetitions')
$suiteStopwatch = [Diagnostics.Stopwatch]::new()
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

    $remainingSuiteMilliseconds = [Math]::Floor(($suiteTimeout - $suiteStopwatch.Elapsed).TotalMilliseconds)
    if ($remainingSuiteMilliseconds -le 0) {
        throw "The host control suite exceeded its $($suiteTimeout.TotalSeconds)-second deadline."
    }
    if ($Executable -match '["&|<>^%\r\n]' -or
        ($Arguments | Where-Object { $_ -match '[\s"&|<>^%\r\n]' })) {
        throw "$Name uses an unsupported shell metacharacter in its fixed command."
    }

    $effectiveTimeoutMilliseconds = [Math]::Min(
        $TimeoutSeconds * 1000,
        [int] $remainingSuiteMilliseconds
    )
    $captureRoot = Join-Path $outputDirectory ('.probe-capture-' + [guid]::NewGuid().ToString('N'))
    $stdoutPath = Join-Path $captureRoot 'stdout.txt'
    $stderrPath = Join-Path $captureRoot 'stderr.txt'
    New-Item -ItemType Directory -Force -Path $captureRoot | Out-Null
    $process = $null
    try {
        $command = '"' + $Executable + '"'
        if ($Arguments.Count -gt 0) {
            $command += ' ' + ($Arguments -join ' ')
        }
        $command += ' 1>"' + $stdoutPath + '" 2>"' + $stderrPath + '"'
        $startInfo = [Diagnostics.ProcessStartInfo]::new()
        $startInfo.FileName = Join-Path $env:SystemRoot 'System32\cmd.exe'
        $startInfo.Arguments = '/d /s /c "' + $command + '"'
        $startInfo.WorkingDirectory = $WorkingDirectory
        $startInfo.UseShellExecute = $false
        $startInfo.CreateNoWindow = $true
        $process = [Diagnostics.Process]::new()
        $process.StartInfo = $startInfo
        $stopwatch = [Diagnostics.Stopwatch]::StartNew()
        if (-not $process.Start()) {
            throw "Failed to start $Name."
        }
        while (-not $process.WaitForExit(25)) {
            $capturedBytes = 0
            foreach ($path in @($stdoutPath, $stderrPath)) {
                if (Test-Path -LiteralPath $path -PathType Leaf) {
                    $capturedBytes += (Get-Item -LiteralPath $path).Length
                }
            }
            if ($capturedBytes -gt $outputLimit) {
                $cleanupConfirmed = (Stop-CapturedProcess -Process $process) -and
                    (Wait-CaptureRelease -Paths @($stdoutPath, $stderrPath))
                $stopwatch.Stop()
                Add-FailedProcessResult -Name $Name -Phase $Phase -Executable $Executable `
                    -Arguments $Arguments -Process $process -DurationMs $stopwatch.ElapsedMilliseconds `
                    -Failure 'output-limit' -CleanupConfirmed $cleanupConfirmed `
                    -StdoutPath $stdoutPath -StderrPath $stderrPath -OmitCapturedOutput
                throw "$Name exceeded the 1 MiB output limit."
            }
            if ($stopwatch.ElapsedMilliseconds -ge $effectiveTimeoutMilliseconds) {
                $cleanupConfirmed = (Stop-CapturedProcess -Process $process) -and
                    (Wait-CaptureRelease -Paths @($stdoutPath, $stderrPath))
                $stopwatch.Stop()
                Add-FailedProcessResult -Name $Name -Phase $Phase -Executable $Executable `
                    -Arguments $Arguments -Process $process -DurationMs $stopwatch.ElapsedMilliseconds `
                    -Failure 'timeout' -CleanupConfirmed $cleanupConfirmed `
                    -StdoutPath $stdoutPath -StderrPath $stderrPath
                throw "$Name timed out after $effectiveTimeoutMilliseconds milliseconds."
            }
        }
        $stopwatch.Stop()
        $capturedBytes = (Get-Item -LiteralPath $stdoutPath).Length + (Get-Item -LiteralPath $stderrPath).Length
        if ($capturedBytes -gt $outputLimit) {
            Add-FailedProcessResult -Name $Name -Phase $Phase -Executable $Executable `
                -Arguments $Arguments -Process $process -DurationMs $stopwatch.ElapsedMilliseconds `
                -Failure 'output-limit' -CleanupConfirmed $true `
                -StdoutPath $stdoutPath -StderrPath $stderrPath -OmitCapturedOutput
            throw "$Name exceeded the 1 MiB output limit."
        }
        $stdout = [IO.File]::ReadAllText($stdoutPath).Trim()
        $stderr = [IO.File]::ReadAllText($stderrPath).Trim()
        if ($process.ExitCode -ne 0) {
            Add-FailedProcessResult -Name $Name -Phase $Phase -Executable $Executable `
                -Arguments $Arguments -Process $process -DurationMs $stopwatch.ElapsedMilliseconds `
                -Failure 'nonzero-exit' -CleanupConfirmed $true `
                -StdoutPath $stdoutPath -StderrPath $stderrPath
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
            Failure = ''
            CleanupConfirmed = $true
        }
    } finally {
        if ($null -ne $process -and -not $process.HasExited) {
            if (-not (Stop-CapturedProcess -Process $process)) {
                Write-Warning "Could not confirm termination of $Name process $($process.Id)."
            }
        }
        if ($null -ne $process) {
            $process.Dispose()
        }
        $captureReleased = Wait-CaptureRelease -Paths @($stdoutPath, $stderrPath)
        if ($captureReleased -and (Test-Path -LiteralPath $captureRoot)) {
            Remove-Item -LiteralPath $captureRoot -Recurse -Force
        } elseif (-not $captureReleased) {
            Write-Warning "Capture handles for $Name were not released within five seconds."
        }
    }
}

function Stop-CapturedProcess {
    param(
        [Parameter(Mandatory)]
        [Diagnostics.Process] $Process
    )

    if ($Process.HasExited) {
        return $true
    }
    $stopInfo = [Diagnostics.ProcessStartInfo]::new()
    $stopInfo.FileName = Join-Path $env:SystemRoot 'System32\taskkill.exe'
    $stopInfo.Arguments = "/PID $($Process.Id) /T /F"
    $stopInfo.UseShellExecute = $false
    $stopInfo.CreateNoWindow = $true
    $stopper = [Diagnostics.Process]::new()
    $stopper.StartInfo = $stopInfo
    try {
        if (-not $stopper.Start()) {
            return $false
        }
        if (-not $stopper.WaitForExit(5000)) {
            $stopper.Kill()
            [void] $stopper.WaitForExit(1000)
            return $false
        }
        if ($stopper.ExitCode -ne 0) {
            return $false
        }
    } catch {
        return $Process.HasExited
    } finally {
        $stopper.Dispose()
    }
    return $Process.WaitForExit(5000)
}

function Wait-CaptureRelease {
    param(
        [Parameter(Mandatory)]
        [string[]] $Paths
    )

    $stopwatch = [Diagnostics.Stopwatch]::StartNew()
    do {
        $released = $true
        foreach ($path in $Paths) {
            if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
                continue
            }
            try {
                $stream = [IO.File]::Open($path, 'Open', 'Read', 'None')
                $stream.Dispose()
            } catch {
                $released = $false
                break
            }
        }
        if ($released) {
            return $true
        }
        Start-Sleep -Milliseconds 25
    } while ($stopwatch.ElapsedMilliseconds -lt 5000)
    return $false
}

function Add-FailedProcessResult {
    param(
        [string] $Name,
        [string] $Phase,
        [string] $Executable,
        [string[]] $Arguments,
        [Diagnostics.Process] $Process,
        [long] $DurationMs,
        [string] $Failure,
        [bool] $CleanupConfirmed,
        [string] $StdoutPath,
        [string] $StderrPath,
        [switch] $OmitCapturedOutput
    )

    if ($OmitCapturedOutput) {
        $stdout = '[omitted: combined output exceeded 1 MiB]'
        $stderr = '[omitted: combined output exceeded 1 MiB]'
    } else {
        $stdout = if (Test-Path -LiteralPath $StdoutPath) {
            [IO.File]::ReadAllText($StdoutPath).Trim()
        } else { '' }
        $stderr = if (Test-Path -LiteralPath $StderrPath) {
            [IO.File]::ReadAllText($StderrPath).Trim()
        } else { '' }
    }
    $exitCode = if ($Process.HasExited) { $Process.ExitCode } else { $null }
    $script:results.Add([pscustomobject]@{
        Name = $Name
        Phase = $Phase
        Command = @($Executable) + $Arguments
        ExitCode = $exitCode
        DurationMs = $DurationMs
        Stdout = $stdout
        Stderr = $stderr
        Failure = $Failure
        CleanupConfirmed = $CleanupConfirmed
    })
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

$operatingSystem = Get-CimInstance -ClassName Win32_OperatingSystem
New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
if (Test-Path -LiteralPath $resolvedOutput -PathType Leaf) {
    Remove-Item -LiteralPath $resolvedOutput -Force
}
if ($SelfTest) {
    $suiteStopwatch.Restart()
    $timeoutCommand = [Convert]::ToBase64String(
        [Text.Encoding]::Unicode.GetBytes('Start-Sleep -Seconds 10')
    )
    $overflowCommand = [Convert]::ToBase64String(
        [Text.Encoding]::Unicode.GetBytes(
            "[Console]::Out.Write(('x' * 2097152)); Start-Sleep -Seconds 10"
        )
    )
    foreach ($case in @(
        @{ Name = 'timeout-self-test'; Command = $timeoutCommand; Timeout = 1; Expected = 'timed out' },
        @{ Name = 'overflow-self-test'; Command = $overflowCommand; Timeout = 10; Expected = 'output limit' }
    )) {
        $failedAsExpected = $false
        try {
            Invoke-BoundedProcess -Name $case.Name -Executable 'powershell.exe' `
                -Arguments @('-NoLogo', '-NoProfile', '-NonInteractive', '-EncodedCommand', $case.Command) `
                -WorkingDirectory $repositoryRoot -TimeoutSeconds $case.Timeout -Phase 'self-test' | Out-Null
        } catch {
            if ($_.Exception.Message -match $case.Expected) {
                $failedAsExpected = $true
            } else {
                throw
            }
        }
        if (-not $failedAsExpected) {
            throw "$($case.Name) unexpectedly succeeded."
        }
    }
    $suiteStopwatch.Stop()
    if ($results.Count -ne 2 -or @($results | Where-Object { -not $_.CleanupConfirmed }).Count -ne 0) {
        throw 'Bounded-process self-test did not retain two cleaned-up failure results.'
    }
    $selfTestSummary = [ordered]@{
        Schema = 1
        Status = 'pass'
        Diagnostic = 'Expected timeout and output-limit failures were captured and reaped.'
        Utc = [DateTime]::UtcNow.ToString('o')
        SuiteDurationMs = $suiteStopwatch.ElapsedMilliseconds
        Results = $results
    }
    $selfTestSummary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $resolvedOutput -Encoding UTF8
    Write-Host "Bounded-process timeout/output-limit self-test passed: $resolvedOutput"
    return
}

$caught = $null
$status = 'failed'
$diagnostic = ''
try {
Invoke-Native -Executable 'cargo.exe' -Arguments @(
    'build', '--release', '--locked', '--target', 'x86_64-pc-windows-msvc',
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

$suiteStopwatch.Restart()
$phases = @('warm-up') + @(1..$repetitions | ForEach-Object { "measured-$_" })
foreach ($phase in $phases) {
    $d3d11Result = Invoke-BoundedProcess -Name 'd3d11-offscreen' -Executable $d3d11 `
        -WorkingDirectory $repositoryRoot -TimeoutSeconds $processTimeoutSeconds -Phase $phase
    $results.Add($d3d11Result)
    $d3d11Report = Assert-D3DResult -Result $d3d11Result -ExpectedProbe 'd3d11-offscreen'

    $d3d12Result = Invoke-BoundedProcess -Name 'd3d12-offscreen' -Executable $d3d12 `
        -WorkingDirectory $repositoryRoot -TimeoutSeconds $processTimeoutSeconds -Phase $phase
    $results.Add($d3d12Result)
    $d3d12Report = Assert-D3DResult -Result $d3d12Result -ExpectedProbe 'd3d12-offscreen'

    $cudaIdentityResult = Invoke-BoundedProcess -Name 'cuda-identity' -Executable $cudaIdentity `
        -WorkingDirectory $repositoryRoot -TimeoutSeconds $processTimeoutSeconds -Phase $phase
    $results.Add($cudaIdentityResult)
    $cudaReport = Assert-CudaIdentity -Result $cudaIdentityResult

    if ($d3d11Report.adapter.luid -ne $d3d12Report.adapter.luid -or
        $d3d11Report.adapter.luid -ne $cudaReport.luid) {
        throw "Cross-API LUID mismatch during ${phase}."
    }

    $cudaResult = Invoke-BoundedProcess -Name 'cuda-vectorAddDrv' -Executable $vectorAdd `
        -Arguments @('--device=0') -WorkingDirectory $cudaDirectory -TimeoutSeconds $cudaTimeoutSeconds -Phase $phase
    $results.Add($cudaResult)
    if ($cudaResult.Stdout -notmatch '(?m)^Result = PASS\s*$') {
        throw "CUDA vectorAddDrv did not report PASS during ${phase}."
    }
}

$suiteStopwatch.Stop()
if ($suiteStopwatch.Elapsed -gt $suiteTimeout) {
    throw "The host control suite exceeded its $($suiteTimeout.TotalSeconds)-second deadline."
}
$status = 'pass'
} catch {
    $caught = $_
    $diagnostic = $_.Exception.Message
} finally {
$suiteStopwatch.Stop()
$summary = [ordered]@{
    Schema = 1
    Status = $status
    Diagnostic = $diagnostic
    Utc = [DateTime]::UtcNow.ToString('o')
    Machine = $env:COMPUTERNAME
    Windows = "$($operatingSystem.Caption) $($operatingSystem.Version) build $($operatingSystem.BuildNumber)"
    Architecture = $env:PROCESSOR_ARCHITECTURE
    SessionId = [Diagnostics.Process]::GetCurrentProcess().SessionId
    SuiteDurationMs = $suiteStopwatch.ElapsedMilliseconds
    ExpectedImageSha256 = $expectedHash
    Results = $results
}
$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $resolvedOutput -Encoding UTF8
}
if ($null -ne $caught) {
    throw $caught.Exception
}
Write-Host "Host controls passed: one warm-up plus $repetitions measured repetitions for D3D11, D3D12 and CUDA."
Write-Host "Evidence: $resolvedOutput"
