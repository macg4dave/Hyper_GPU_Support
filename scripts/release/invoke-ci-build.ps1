<# Run the unchanged source build as an ordinary user on a disposable hosted runner. #>
[CmdletBinding(DefaultParameterSetName = 'Launch')]
param(
    [Parameter(Mandatory, ParameterSetName = 'Launch')][string]$SourceDirectory,
    [Parameter(Mandatory, ParameterSetName = 'Launch')][string]$WorkDirectory,
    [Parameter(Mandatory, ParameterSetName = 'Launch')][string]$IsccPath,
    [Parameter(Mandatory, ParameterSetName = 'Worker')][string]$ConfigPath
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ($env:GITHUB_ACTIONS -cne 'true' -or $env:RUNNER_ENVIRONMENT -cne 'github-hosted') {
    throw 'This account-provisioning launcher is only for GitHub-hosted Actions runners.'
}
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$administrator = ([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if ($PSCmdlet.ParameterSetName -eq 'Worker') {
    if ($administrator) { throw 'The source build and non-hardware tests must run without administrator rights.' }
    $config = Get-Content -LiteralPath $ConfigPath -Raw | ConvertFrom-Json
    # Start-Process inherits the caller's environment even with another credential.
    # Restore this user's profile and Windows PowerShell's inbox module search path.
    $env:USERPROFILE = [Environment]::GetFolderPath('UserProfile')
    $env:APPDATA = [Environment]::GetFolderPath('ApplicationData')
    $env:LOCALAPPDATA = [Environment]::GetFolderPath('LocalApplicationData')
    $env:USERNAME = $identity.Name.Split('\')[-1]
    $env:PSModulePath = (Join-Path $PSHOME 'Modules') + ';' + (Join-Path $env:ProgramFiles 'WindowsPowerShell/Modules')
    $env:CARGO_HOME = $config.CargoHome
    $env:RUSTUP_HOME = $config.RustupHome
    $env:PATH = (Join-Path $config.CargoHome 'bin') + ';' + $env:PATH
    $env:TEMP = Join-Path $config.WorkDirectory 'user-temp'
    $env:TMP = $env:TEMP
    New-Item -ItemType Directory -Path $env:TEMP -Force | Out-Null
    Get-Command Get-FileHash -ErrorAction Stop | Out-Null
    Write-Host "Running source checks and packaging as $($identity.Name) without administrator rights."
    Set-Location -LiteralPath $config.SourceDirectory
    & (Join-Path $config.SourceDirectory 'scripts/build-installer.ps1') -WorkDirectory $config.WorkDirectory -OutputDirectory (Join-Path $config.WorkDirectory 'candidate') -IsccPath $config.IsccPath
    if (-not $?) { throw 'Standard-user source build failed.' }
    exit 0
}

if (-not $administrator) { throw 'Only the disposable runner administrator can provision the build account.' }
$SourceDirectory = (Get-Item -LiteralPath $SourceDirectory).FullName
$WorkDirectory = [IO.Path]::GetFullPath($WorkDirectory)
$workspacePrefix = [IO.Path]::GetFullPath($env:GITHUB_WORKSPACE).TrimEnd('\') + '\'
$temporaryPrefix = [IO.Path]::GetFullPath($env:RUNNER_TEMP).TrimEnd('\') + '\'
if (-not $SourceDirectory.StartsWith($workspacePrefix, [StringComparison]::OrdinalIgnoreCase) -or
    -not $WorkDirectory.StartsWith($temporaryPrefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'CI source and staging must remain inside the hosted workspace and runner temporary directory.'
}
if (Test-Path -LiteralPath $WorkDirectory) { throw 'Use a fresh CI staging directory.' }
New-Item -ItemType Directory -Path $WorkDirectory | Out-Null
$cargoHome = $env:CARGO_HOME
if (-not $cargoHome) { $cargoHome = Join-Path $env:USERPROFILE '.cargo' }
$rustupHome = $env:RUSTUP_HOME
if (-not $rustupHome) { $rustupHome = Join-Path $env:USERPROFILE '.rustup' }
$cargoHome = (Get-Item -LiteralPath $cargoHome).FullName
$rustupHome = (Get-Item -LiteralPath $rustupHome).FullName
$compiler = (Get-Item -LiteralPath $IsccPath).FullName
$worker = Join-Path $WorkDirectory 'invoke-ci-build.ps1'
Copy-Item -LiteralPath $PSCommandPath -Destination $worker
$configuration = Join-Path $WorkDirectory 'ci-build.json'
@{
    SourceDirectory = $SourceDirectory; WorkDirectory = $WorkDirectory; IsccPath = $compiler
    CargoHome = $cargoHome; RustupHome = $rustupHome
} | ConvertTo-Json | Set-Content -LiteralPath $configuration -Encoding UTF8

$accountName = 'HyperGpuBuild'
$passwordText = [guid]::NewGuid().ToString('N') + 'aA1!'
Write-Host "::add-mask::$passwordText"
$password = ConvertTo-SecureString $passwordText -AsPlainText -Force
$account = New-LocalUser -Name $accountName -Password $password -Description 'Disposable standard account for release builds'
try {
    $users = Get-LocalGroup -SID 'S-1-5-32-545'
    Add-LocalGroupMember -Group $users -Member $account
    $credential = [Management.Automation.PSCredential]::new(($env:COMPUTERNAME + '\' + $accountName), $password)
    $icacls = Join-Path ([Environment]::SystemDirectory) 'icacls.exe'
    foreach ($directory in @($SourceDirectory, $WorkDirectory, $cargoHome, $rustupHome)) {
        & $icacls $directory '/grant' ('*{0}:(OI)(CI)M' -f $account.SID.Value) '/T' '/Q'
        if ($LASTEXITCODE -ne 0) { throw "Could not grant the build account access to $directory." }
    }
    & $icacls (Split-Path -Parent $compiler) '/grant' ('*{0}:(OI)(CI)RX' -f $account.SID.Value) '/T' '/Q'
    if ($LASTEXITCODE -ne 0) { throw 'Could not grant read access to the pinned compiler.' }
    $stdout = Join-Path $WorkDirectory 'build-stdout.log'
    $stderr = Join-Path $WorkDirectory 'build-stderr.log'
    $powershell = Join-Path ([Environment]::SystemDirectory) 'WindowsPowerShell/v1.0/powershell.exe'
    Write-Host 'Starting the full source build under the standard build account.'
    $process = Start-Process -FilePath $powershell -ArgumentList @('-NoLogo', '-NoProfile', '-NonInteractive', '-File', ('"{0}"' -f $worker), '-ConfigPath', ('"{0}"' -f $configuration)) -Credential $credential -LoadUserProfile -WorkingDirectory $SourceDirectory -WindowStyle Hidden -RedirectStandardOutput $stdout -RedirectStandardError $stderr -PassThru -Wait
    Get-Content -LiteralPath $stdout | ForEach-Object { Write-Host $_ }
    Get-Content -LiteralPath $stderr | ForEach-Object { Write-Host $_ }
    if ($process.ExitCode -ne 0) { throw "Standard-user source build failed with exit $($process.ExitCode)." }
} finally {
    Remove-LocalUser -SID $account.SID
}
