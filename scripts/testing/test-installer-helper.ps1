<# Exercise the production Inno helper launcher in an isolated user-mode harness. #>
[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$IsccPath)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object System.Security.Principal.WindowsPrincipal($identity)
if ($principal.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Run installer helper tests from an ordinary, non-elevated terminal.'
}
$fixture = Join-Path ([IO.Path]::GetTempPath()) ('HyperGpuSupport-helper-tests-' + [guid]::NewGuid().ToString('N'))
$protected = Join-Path $fixture 'protected'
New-Item -ItemType Directory -Path $protected -Force | Out-Null
$originalAcl = Get-Acl -LiteralPath $protected
$acl = New-Object System.Security.AccessControl.DirectorySecurity
$acl.SetAccessRuleProtection($true, $false)
foreach ($sid in @('S-1-5-18', 'S-1-5-32-544')) {
    $account = New-Object System.Security.Principal.SecurityIdentifier($sid)
    $acl.AddAccessRule((New-Object System.Security.AccessControl.FileSystemAccessRule($account, 'FullControl', 'ContainerInherit,ObjectInherit', 'None', 'Allow')))
}
$acl.AddAccessRule((New-Object System.Security.AccessControl.FileSystemAccessRule($identity.User, 'ReadAndExecute', 'ContainerInherit,ObjectInherit', 'None', 'Allow')))
$helperSource = @'
using System;
using System.IO;
public static class InstallerHelperFixture {
    public static int Main(string[] args) {
        try {
            bool negative = Array.IndexOf(args, "--negative") >= 0;
            bool fail = negative || Array.IndexOf(args, "--fail") >= 0;
            int log = Array.IndexOf(args, "--log");
            if (log >= 0) File.WriteAllText(args[log + 1], fail ? "fixture failure" : "fixture success");
            return negative ? -1 : fail ? 17 : 0;
        } catch (Exception error) {
            Console.Error.WriteLine(error.Message);
            return 18;
        }
    }
}
'@
Add-Type -TypeDefinition $helperSource -OutputAssembly (Join-Path $fixture 'helper.exe') -OutputType ConsoleApplication
try {
    Set-Acl -LiteralPath $protected -AclObject $acl
    # Prove this fixture has Inno's elevated temporary-directory restriction.
    $denied = $false
    try { [IO.File]::WriteAllText((Join-Path $protected 'probe.log'), 'probe') }
    catch [System.UnauthorizedAccessException] { $denied = $true }
    if (-not $denied) { throw 'Fixture does not enforce the required write restriction.' }
    & $IsccPath "/DFixtureDirectory=$fixture" (Join-Path $PSScriptRoot 'installer-helper.iss') | Out-Host
    if ($LASTEXITCODE -ne 0) { throw 'Installer helper harness compilation failed.' }
    $test = Start-Process -FilePath (Join-Path $fixture 'compiled/helper-test.exe') -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART' -PassThru -WindowStyle Hidden
    try {
        if (-not $test.WaitForExit(30000)) { $test.Kill(); throw 'Installer helper harness timed out.' }
    } finally { $test.Dispose() }
    $resultPath = Join-Path $fixture 'result.txt'
    if (-not (Test-Path -LiteralPath $resultPath)) { throw "Installer helper harness produced no result: $fixture" }
    $result = Get-Content -LiteralPath $resultPath -Raw
    if ($result -cne 'PASS') { throw $result }
} finally {
    $restore = New-Object System.Security.AccessControl.DirectorySecurity
    $restore.SetSecurityDescriptorSddlForm($originalAcl.GetSecurityDescriptorSddlForm([System.Security.AccessControl.AccessControlSections]::Access), [System.Security.AccessControl.AccessControlSections]::Access)
    [IO.Directory]::SetAccessControl($protected, $restore)
    $identity.Dispose()
}
Write-Host "Installer helper execution checks passed. Isolated fixtures: $fixture"
