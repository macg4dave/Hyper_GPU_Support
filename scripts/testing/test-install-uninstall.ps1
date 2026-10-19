<#
.SYNOPSIS
Test installation and full uninstallation, then leave the app installed.
.DESCRIPTION
Run only when installation and app-data reset are intended. This runs the real
installer and uninstaller, requests Windows administrator permission and resets
app-owned data. It does not launch the app or change VMs or guest files.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$SetupPath,
    [Parameter(Mandatory = $true)][string]$PayloadDirectory
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$SetupPath = (Get-Item -LiteralPath $SetupPath).FullName
$PayloadDirectory = (Get-Item -LiteralPath $PayloadDirectory).FullName
$work = Join-Path ([IO.Path]::GetTempPath()) ('HyperGpuSupport-install-test-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $work | Out-Null
$app = Join-Path ([Environment]::GetFolderPath('ProgramFiles')) 'HyperGpuSupport'
$shortcut = Join-Path ([Environment]::GetFolderPath('CommonPrograms')) 'Hyper GPU Support/Hyper GPU Support.lnk'
$registration = 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\{582BBF4B-C798-48B5-9F32-56F8064CA698}_is1'
$names = @('hyper-gpu-support.exe', 'hyper-gpu-runner.exe', 'hyper-gpu-guest.exe', 'd3d11-probe.exe', 'app-install.exe')
$version = (Get-Item -LiteralPath (Join-Path $PayloadDirectory 'hyper-gpu-support.exe')).VersionInfo.ProductVersion

function Run-Setup {
    param([string]$Executable, [string]$Name)
    $log = Join-Path $work ($Name + '.log')
    $process = Start-Process -FilePath $Executable -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', ('/LOG="' + $log + '"') -WindowStyle Hidden -PassThru
    try {
        # Keep the handle valid while Windows waits through the UAC bootstrapper.
        $handle = $process.Handle
        $process.WaitForExit()
        $code = $process.get_ExitCode()
    } finally { $process.Dispose() }
    if ($code -ne 0) { throw "$Name failed with exit $code. See $log" }
    Write-Host "$Name passed (exit 0). Log: $log"
}

function Assert-Installed {
    foreach ($name in $names) {
        $expected = (Get-FileHash -LiteralPath (Join-Path $PayloadDirectory $name) -Algorithm SHA256).Hash
        $actual = (Get-FileHash -LiteralPath (Join-Path $app $name) -Algorithm SHA256).Hash
        if ($actual -cne $expected) { throw "Installed executable differs: $name" }
    }
    if (-not (Test-Path -LiteralPath $shortcut)) { throw 'Start menu shortcut is missing.' }
    $shell = New-Object -ComObject WScript.Shell
    $target = $shell.CreateShortcut($shortcut).TargetPath
    if ($target -ine (Join-Path $app 'hyper-gpu-support.exe')) { throw 'Start menu shortcut has the wrong target.' }
    $entry = Get-ItemProperty -LiteralPath $registration
    if ($entry.DisplayName -cne 'Hyper GPU Support' -or $entry.DisplayVersion -cne $version) {
        throw 'Windows Installed apps entry has the wrong product or version.'
    }
    $uninstaller = $entry.UninstallString.Trim('"')
    if ([IO.Path]::GetDirectoryName($uninstaller) -ine $app -or -not (Test-Path -LiteralPath $uninstaller)) {
        throw 'Windows Installed apps entry does not point to the installed uninstaller.'
    }
    Write-Host 'Installed executable hashes, shortcut and uninstall entry match.'
    return $uninstaller
}

function Assert-Uninstalled {
    # Inno's final cleanup process can outlive the uninstaller we started.
    $deadline = [DateTime]::UtcNow.AddSeconds(15)
    while ((Test-Path -LiteralPath $app) -and [DateTime]::UtcNow -lt $deadline) {
        Start-Sleep -Milliseconds 100
    }
    $programData = [Environment]::GetFolderPath('CommonApplicationData')
    $programFiles = [Environment]::GetFolderPath('ProgramFiles')
    foreach ($path in @($app, $shortcut, $registration,
        (Join-Path $programFiles 'HyperGpuSupportProduct'),
        (Join-Path $programData 'HyperGpuSupportProduct'),
        (Join-Path $programData 'HyperGpuSupport'))) {
        if (Test-Path -LiteralPath $path) { throw "Uninstall left an application path or registration: $path" }
    }
    $tasks = @(Get-ScheduledTask -ErrorAction Stop | Where-Object {
        ($_.TaskPath -eq '\' -and $_.TaskName -eq 'HyperGpuSupport-Product-v2') -or
        ($_.TaskPath -eq '\HyperGpuSupport\' -and $_.TaskName -in @('Runner-v1', 'ResetSlot-v1'))
    })
    if ($tasks.Count -ne 0) { throw 'Uninstall left a product background task.' }
    Write-Host 'Uninstall removed app files, shortcut, uninstall entry, protected state and product tasks.'
}

Run-Setup $SetupPath 'install'
$uninstaller = Assert-Installed
Run-Setup $uninstaller 'uninstall'
Assert-Uninstalled
Run-Setup $SetupPath 'reinstall'
$uninstaller = Assert-Installed
Write-Host "Install/uninstall/reinstall checks passed. The app is installed. Logs: $work"
