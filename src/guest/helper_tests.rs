//! Production transport sections with local files and fixed remoting/ACL doubles.
use super::*;

#[test]
fn helper_reuse_replacement_and_failure_paths_do_not_enter_hyperv_or_launch_a_worker() {
    let script = transport_script("unused").replace("\r\n", "\n");
    let definitions = script.split_once("\ntry {").unwrap().0;
    let publication = script
        .split_once("    Write-Stage 'bootstrap-leaves' 'start' $root")
        .unwrap()
        .1
        .split_once("    $timer.Restart()\n    Write-Stage 'bundle-path'")
        .unwrap()
        .0;
    let fixture = r#"
function Invoke-Command { param($Session, $ArgumentList, $ScriptBlock) & $ScriptBlock @ArgumentList }
function Get-Acl {
    param($LiteralPath)
    $acl = [Security.AccessControl.DirectorySecurity]::new()
    $owner = 'S-1-5-32-544'
    if ($script:unsafeKind -eq 'owner' -and $script:unsafePath -eq $LiteralPath) { $owner = 'S-1-1-0' }
    $acl.SetOwner([Security.Principal.SecurityIdentifier]::new($owner))
    $acl.SetAccessRuleProtection($true, $false)
    $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'), 'FullControl', 'Allow'))
    if ($script:unsafeKind -eq 'writer' -and $script:unsafePath -eq $LiteralPath) {
        $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new('S-1-1-0'), 'Write', 'Allow'))
    }
    $acl
}
function Get-Item {
    param($LiteralPath, [switch]$Force)
    $item = Microsoft.PowerShell.Management\Get-Item -LiteralPath $LiteralPath -Force
    if ($script:unsafePath -eq $LiteralPath -and $script:unsafeKind -in @('link','reparse')) {
        $attributes = $item.Attributes; $linkType = 'HardLink'
        if ($script:unsafeKind -eq 'reparse') { $attributes = $attributes -bor [IO.FileAttributes]::ReparsePoint; $linkType = $null }
        return [pscustomobject]@{PSIsContainer=$item.PSIsContainer; Attributes=$attributes; LinkType=$linkType}
    }
    $item
}
function Set-Acl {
    param($LiteralPath, $AclObject)
    if ($script:failAcl -eq $LiteralPath) { throw 'Injected ACL failure' }
    $script:aclWrites++
}
$fixtureRoot = Join-Path ([IO.Path]::GetTempPath()) ('hyper-gpu-helpers-' + [guid]::NewGuid().ToString('N'))
$root = Join-Path $fixtureRoot 'guest'
$install = Join-Path $fixtureRoot 'install'
[void][IO.Directory]::CreateDirectory($root)
[void][IO.Directory]::CreateDirectory($install)
$timer = [Diagnostics.Stopwatch]::StartNew()
$r = @{ install=$install; artifacts=@() }
$script:unsafeKind = ''; $script:unsafePath = ''; $script:failAcl = ''
$sourceBytes = [byte[]]@(1,2,3,4,5)
foreach ($name in @('hyper-gpu-guest.exe', 'd3d11-probe.exe')) {
    [IO.File]::WriteAllBytes((Join-Path $install $name), $sourceBytes)
    [IO.File]::WriteAllBytes((Join-Path $root $name), $sourceBytes)
    $r.artifacts += @{name=$name; bytes=$sourceBytes.Length; hash=(Get-FileHash -LiteralPath (Join-Path $install $name)).Hash.ToLowerInvariant()}
}
$worker = Join-Path $root 'hyper-gpu-guest.exe'
$probe = Join-Path $root 'd3d11-probe.exe'
function Reset-Counters { $script:transferIndex=0; $script:transferTotal=0; $script:aclWrites=0 }
function Require-Rejected { param($work) $rejected=$false; try { & $work } catch { $rejected=$true }; if (!$rejected) { throw 'Expected refusal' } }
try {
    Reset-Counters
    & $selectionAndPublication
    if ($script:transferTotal -ne 0 -or $script:aclWrites -ne 0) { throw 'Matching helpers were written' }
    foreach ($path in @($worker,$probe)) { [IO.File]::Delete($path) }
    Reset-Counters
    & $selectionAndPublication
    if ($script:transferIndex -ne 2 -or $script:transferTotal -ne 10) { throw 'Missing helpers not transferred exactly' }
    $diagnosticPath = Join-Path $fixtureRoot 'diagnostic.jsonl'
    $diagnostic = [IO.FileStream]::new($diagnosticPath, 'CreateNew', 'Write', 'Read')
    [IO.File]::WriteAllBytes($probe, [byte[]]@(5,4,3,2,1))
    Reset-Counters
    & $selectionAndPublication
    $diagnostic.Dispose(); $diagnostic=$null
    if ($script:transferIndex -ne 1 -or $script:transferTotal -ne 5 -or $script:aclWrites -ne 2) { throw 'Mixed reuse/replacement mismatch' }
    foreach ($kind in @('owner','writer','link','reparse')) {
        $script:unsafeKind=$kind; $script:unsafePath=$worker; Reset-Counters
        Require-Rejected { & $selectionAndPublication }
        if ($script:transferTotal -ne 0 -or $script:aclWrites -ne 0) { throw 'Unsafe helper was written' }
    }
    $script:unsafeKind='writer'; $script:unsafePath=$fixtureRoot; Reset-Counters
    Require-Rejected { & $selectionAndPublication }
    if ($script:transferTotal -ne 0) { throw 'Unsafe parent allowed transfer' }
    $script:unsafeKind=''; $script:unsafePath=''
    [IO.File]::Delete($probe); [void][IO.Directory]::CreateDirectory($probe)
    Require-Rejected { & $selectionAndPublication }
    [IO.Directory]::Delete($probe)
    [IO.File]::WriteAllBytes($probe, [byte[]]@(5,4,3,2,1))
    $old = (Get-FileHash -LiteralPath $probe).Hash
    $hold = [IO.File]::Open($probe, 'Open', 'Read', 'Read')
    try { Require-Rejected { & $selectionAndPublication } } finally { $hold.Dispose() }
    if ((Get-FileHash -LiteralPath $probe).Hash -ne $old) { throw 'Failed publication lost original helper' }
    $script:failAcl=$probe
    Require-Rejected { & $selectionAndPublication }
    if (@(Microsoft.PowerShell.Management\Get-ChildItem -LiteralPath $root -Filter '*.previous').Count -eq 0) { throw 'Uncertain ACL publication discarded backup' }
    $script:failAcl=''
    [IO.File]::WriteAllBytes($probe, [byte[]]@(5,4,3,2,1))
    [IO.File]::WriteAllBytes((Join-Path $install 'd3d11-probe.exe'), [byte[]]@(9,9,9,9,9))
    $old = (Get-FileHash -LiteralPath $probe).Hash
    Require-Rejected { & $selectionAndPublication }
    if ((Get-FileHash -LiteralPath $probe).Hash -ne $old) { throw 'Staged hash failure changed the old helper' }
    # Fresh launch validation sees same-length drift; its read handle blocks writes.
    [IO.File]::WriteAllBytes($worker, [byte[]]@(5,4,3,2,1))
    if (& $validateBootstrap $worker 5 $r.artifacts[0].hash) { throw 'Launch drift was accepted' }
    [IO.File]::WriteAllBytes($worker, $sourceBytes)
    $hold = [IO.File]::Open($worker, 'Open', 'Read', 'Read')
    try { Require-Rejected { [IO.File]::WriteAllBytes($worker, [byte[]]@(5,4,3,2,1)) } } finally { $hold.Dispose() }
} finally {
    if ($global:hyperGpuTransfer) { $global:hyperGpuTransfer.Dispose() }
    if ($diagnostic) { $diagnostic.Dispose() }
    if ($diagnosticPath) { [IO.File]::Delete($diagnosticPath) }
    # Only non-recursive leaves under this freshly created isolated fixture.
    foreach ($directory in @($root,$install)) {
        foreach ($file in Microsoft.PowerShell.Management\Get-ChildItem -LiteralPath $directory -File) { [IO.File]::Delete($file.FullName) }
        [IO.Directory]::Delete($directory)
    }
    [IO.Directory]::Delete($fixtureRoot)
}
Write-Output 'Helper checks passed'
"#;
    let mut command = Command::new(windows_paths::windows_powershell_executable().unwrap());
    command.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        &format!("{definitions}\n$selectionAndPublication = {{\n{publication}\n}}\n{fixture}"),
    ]);
    let result =
        process::bounded_process_with_limit(command, Duration::from_secs(30), 1024 * 1024, &[])
            .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(result.exit_code, Some(0), "{}", result.stderr);
    assert_eq!(result.stdout.trim(), "Helper checks passed");
}
