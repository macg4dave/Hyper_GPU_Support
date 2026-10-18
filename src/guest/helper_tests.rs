//! Production transport sections with local files and fixed remoting/ACL doubles.
use super::*;

#[test]
fn complete_transport_streams_cold_preparation_and_warm_verification_in_order() {
    // Run the complete fixed bridge against local remoting/ACL doubles. Only
    // the worker invocation and Program Files root are replaced; no VM, guest,
    // installed binaries, credentials or real protected ACLs are accessed.
    let script = transport_script("unused")
        .replace(
            "[Environment]::GetFolderPath('ProgramFiles')",
            "$fixtureProgramFiles",
        )
        .replace(
            "$r = [Console]::In.ReadToEnd() | ConvertFrom-Json",
            "$r = $testRequest",
        )
        .replace(
            "& (Join-Path $root 'hyper-gpu-guest.exe')",
            "Invoke-TestWorker",
        );
    let fixture = r#"
function Import-Module {}
function New-PSSession { 'isolated-session' }
function Remove-PSSession {}
function Invoke-Command {
    param($Session, [object[]]$ArgumentList, [scriptblock]$ScriptBlock)
    & $ScriptBlock @ArgumentList
}
function Get-Acl {
    param($LiteralPath)
    $acl = [Security.AccessControl.DirectorySecurity]::new()
    $admin = [Security.Principal.SecurityIdentifier]::new('S-1-5-32-544')
    $acl.SetOwner($admin)
    $acl.SetAccessRuleProtection($true, $false)
    $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($admin, 'FullControl', 'Allow'))
    $acl
}
function Set-Acl { param($LiteralPath, $AclObject) }
function Invoke-TestWorker {
    param($mode, $bundle, $vendor, $device)
    $phases = @('checking-copied','installing','checking-installed')
    if ($mode -eq 'verify') { $phases = @('checking-guest','graphics') }
    foreach ($phase in $phases) { 'HGPREPARATION ' + $phase }
    '{"digest":"test-payload","machine_guid":"test-machine","files":1}'
    $global:LASTEXITCODE = 0
}
$fixtureBase = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
$fixtureRoot = Join-Path $fixtureBase ('hyper-gpu-bridge-' + [guid]::NewGuid().ToString('N'))
$fixtureProgramFiles = Join-Path $fixtureRoot 'program-files'
$install = Join-Path $fixtureRoot 'install'
[void][IO.Directory]::CreateDirectory($fixtureProgramFiles)
[void][IO.Directory]::CreateDirectory($install)
$testRequest = [pscustomobject]@{
    vm_id='11111111-1111-1111-1111-111111111111'; username='placeholder'; password='placeholder'
    install=$install; mode='prepare'; digest='bundle-prepare'; diagnostics=$false
    identity='{}'; acknowledgements='[]'; verification='{}'; vendor=4318; device=1
    artifacts=@(); manifest=@{files=@()}
}
try {
    foreach ($entry in @(@('hyper-gpu-guest.exe',3), @('d3d11-probe.exe',5))) {
        $path = Join-Path $install $entry[0]
        [IO.File]::WriteAllBytes($path, [byte[]]::new($entry[1]))
        $testRequest.artifacts += @{name=$entry[0]; bytes=$entry[1]; hash=(Get-FileHash -LiteralPath $path).Hash.ToLowerInvariant()}
    }
    $source = Join-Path $fixtureRoot 'driver.bin'
    [IO.File]::WriteAllBytes($source, [byte[]]::new(7))
    $testRequest.manifest.files = @(@{source=$source; bytes=7})
    . $fixedBridge
    $testRequest.mode='verify'; $testRequest.digest='bundle-verify'; $testRequest.password='placeholder'
    . $fixedBridge
} finally {
    $resolved = [IO.Path]::GetFullPath($fixtureRoot)
    if (!$resolved.StartsWith($fixtureBase, [StringComparison]::OrdinalIgnoreCase) -or
        (Split-Path $resolved -Leaf) -notmatch '^hyper-gpu-bridge-[a-f0-9]{32}$') { throw 'Unsafe fixture cleanup path' }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
"#;
    let mut command = Command::new(windows_paths::windows_powershell_executable().unwrap());
    command.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        &format!("$fixedBridge = {{\n{script}\n}}\n{fixture}"),
    ]);
    let result = process::bounded_process_with_limit(command, Duration::from_secs(30), 65536, &[])
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        result.exit_code,
        Some(0),
        "{}\n{}",
        result.stdout,
        result.stderr
    );
    let replies: Vec<BridgeResult> = result
        .stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|error| panic!("{error}: {line}")))
        .collect();
    assert_eq!(replies.len(), 2, "{}", result.stdout);
    let now = Instant::now();
    let cold = crate::guest_transfer::Monitor::for_operation(vec![3, 5, 7], now, false)
        .unwrap()
        .select_helpers(2)
        .unwrap();
    let warm = crate::guest_transfer::Monitor::for_operation(vec![3, 5], now, true)
        .unwrap()
        .select_helpers(2)
        .unwrap();
    use crate::process::StderrMonitor;
    let mut operation = 0;
    for line in result.stderr.split_inclusive('\n') {
        let monitor = if operation == 0 { &cold } else { &warm };
        assert!(monitor.line(line.as_bytes()).unwrap(), "{line}");
        if line.trim_end() == "HGPREPARATION checking-installed" {
            operation += 1;
        }
    }
    for (monitor, reply) in [(&cold, &replies[0]), (&warm, &replies[1])] {
        monitor.require_worker_phases().unwrap();
        assert_eq!(reply.worker_exit, Some(0));
        let receipt: Receipt = serde_json::from_str(&reply.output).unwrap();
        assert_eq!(receipt.digest, "test-payload");
    }
    let cold = cold.take_progress(now).unwrap().unwrap();
    assert_eq!((cold.bytes, cold.total, cold.files), (15, 15, 3));
    let warm = warm.take_progress(now).unwrap().unwrap();
    assert_eq!((warm.bytes, warm.total, warm.files), (0, 0, 0));
}

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
function Invoke-Command { param($Session, [object[]]$ArgumentList, [scriptblock]$ScriptBlock) & $ScriptBlock @ArgumentList }
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
