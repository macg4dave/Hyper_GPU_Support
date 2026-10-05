//! Fixed full-manifest PowerShell Direct transport using the existing safeguards.

use crate::{
    config::ProjectConfiguration,
    driver_environment::{DriverEnvironmentManifest, inspect_driver_environment},
    environment_staging::{
        EnvironmentStageReceipt, EnvironmentStageResult, GuestEnvironmentStager,
        validate_environment_sources,
    },
    guest::GuestCredential,
    staging::StagingError,
    windows_driver_environment::discover_driver_environment,
    windows_guest::{
        STAGING_REQUEST_LIMIT, expand_staging_script, map_staging_error, parse_stage_receipt,
        run_script,
    },
    windows_paths::windows_directory,
};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::LazyLock};
use zeroize::Zeroizing;

/// Transport enrolled to one validated configuration; no caller-selected targets.
pub struct WindowsEnvironmentStager {
    project: ProjectConfiguration,
}
impl WindowsEnvironmentStager {
    /// Bind the full writer to the configured disposable slot and selected driver.
    #[must_use]
    pub fn new(project: ProjectConfiguration) -> Self {
        Self { project }
    }
}

#[derive(Serialize)]
struct Request<'a> {
    vm_id: &'a str,
    vm_name: &'a str,
    child_path: &'a std::path::Path,
    parent_path: &'a std::path::Path,
    parent_sha256: &'a str,
    computer_name: &'a str,
    machine_guid: &'a str,
    staging_root: &'a std::path::Path,
    source_root: &'a std::path::Path,
    windows_root: PathBuf,
    gpu_interface: &'a str,
    gpu_driver_version: &'a str,
    host_build: &'a str,
    signer_thumbprint: &'a str,
    signature_files: &'a [String],
    allowed_pending_delete_sources: &'a [String],
    manifest: &'a DriverEnvironmentManifest,
    manifest_sha256: String,
    byte_count: u64,
    username: &'a str,
    password: &'a str,
}

impl GuestEnvironmentStager for WindowsEnvironmentStager {
    fn stage_environment(
        &self,
        credential: &GuestCredential,
        manifest: &DriverEnvironmentManifest,
    ) -> Result<EnvironmentStageResult, StagingError> {
        let project = &self.project;
        // A public manifest struct is not authorization. Repeat native discovery
        // and expansion here, before encoding any credential-bearing request.
        let root = windows_directory().map_err(|_| StagingError::UnsafeSource)?;
        let discovery = discover_driver_environment(project).map_err(|_| {
            StagingError::GuestPreflightFailed {
                phase: "native-discovery".into(),
            }
        })?;
        let fresh = inspect_driver_environment(project, &root, discovery)?;
        let digest = manifest.sha256().map_err(|_| StagingError::Encoding)?;
        if fresh.sha256().map_err(|_| StagingError::Encoding)? != digest {
            return Err(StagingError::ChangedSource);
        }
        let byte_count = validate_environment_sources(project, &root, manifest)?;
        let request = Request {
            vm_id: &project.slot.vm_id,
            vm_name: &project.slot.vm_name,
            child_path: &project.slot.child_path,
            parent_path: &project.slot.parent_path,
            parent_sha256: &project.slot.parent_sha256,
            computer_name: &project.guest.computer_name,
            machine_guid: &project.guest.machine_guid,
            staging_root: &project.guest.staging_root,
            source_root: &project.driver_manifest.source_path,
            windows_root: root,
            gpu_interface: &project.slot.gpu_interface,
            gpu_driver_version: &project.driver_manifest.driver_version,
            host_build: &project.driver_manifest.host_build,
            signer_thumbprint: &project.driver_manifest.signer_thumbprint,
            signature_files: &project.driver_manifest.signature_files,
            allowed_pending_delete_sources: &project.driver_manifest.allowed_pending_delete_sources,
            manifest,
            manifest_sha256: digest,
            byte_count,
            username: credential.username(),
            password: credential.password(),
        };
        let payload =
            Zeroizing::new(serde_json::to_vec(&request).map_err(|_| StagingError::Encoding)?);
        if payload.len() > STAGING_REQUEST_LIMIT {
            return Err(StagingError::InvalidProtocol);
        }
        let output = run_script(
            &project.guest.powershell_path,
            &SCRIPT,
            payload,
            project.guest.staging_timeout,
        )
        .map_err(map_staging_error)?;
        parse_environment_result(&output)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Response {
    status: String,
    category: String,
    receipt: EnvironmentStageReceipt,
}
fn parse_environment_result(output: &str) -> Result<EnvironmentStageResult, StagingError> {
    let value: serde_json::Value =
        serde_json::from_str(output.trim()).map_err(|_| StagingError::GuestStateUncertain)?;
    if value["status"] == "error" {
        return match parse_stage_receipt(output) {
            Err(error) => Err(error),
            Ok(_) => Err(StagingError::GuestStateUncertain),
        };
    }
    let response: Response =
        serde_json::from_value(value).map_err(|_| StagingError::GuestStateUncertain)?;
    if response.status != "ok"
        || !matches!(response.category.as_str(), "applied" | "already-applied")
    {
        return Err(StagingError::GuestStateUncertain);
    }
    Ok(EnvironmentStageResult {
        already_applied: response.category == "already-applied",
        receipt: response.receipt,
    })
}

static SCRIPT: LazyLock<String> = LazyLock::new(|| expand_staging_script(SCRIPT_TEMPLATE));

const SCRIPT_TEMPLATE: &str = r#"
__STAGING_PRELUDE__
function Invoke-Environment($Session, $Request, [string]$Mode) {
    Invoke-Command -Session $Session -ArgumentList @($Request.manifest.files, $Request.staging_root, $Request.manifest_sha256, $Request.byte_count, $Mode, $Request.vm_id, $Request.computer_name, $Request.machine_guid) -ScriptBlock {
        param($Files, $StagingRoot, $Digest, [uint64]$ExpectedBytes, $Mode, $VmId, $ComputerName, $MachineGuid)
        $ErrorActionPreference = 'Stop'
        function Target([string]$Relative) {
            $root = [IO.Path]::GetFullPath($env:SystemRoot).TrimEnd('\')
            $path = [IO.Path]::GetFullPath([IO.Path]::Combine($root, $Relative))
            if (-not $path.StartsWith($root + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'unsafe path' }
            return $path
        }
        function Check-Parent([string]$Path) {
            $parent = Split-Path -Parent $Path
            while (-not (Test-Path -LiteralPath $parent)) {
                $parent = Split-Path -Parent $parent
                if (-not $parent) { throw 'unsafe path' }
            }
            Assert-ProtectedTree $parent $env:SystemRoot
        }
        function Check-File([string]$Path, $Entry) {
            Check-Parent $Path
            Assert-ProtectedFile $Path
            $item = Get-Item -LiteralPath $Path -Force
            if ([uint64]$item.Length -ne [uint64]$Entry.bytes -or [string]$item.LinkType -ceq 'HardLink' -or
                (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() -cne [string]$Entry.sha256) { throw 'verification failed' }
        }
        $receiptPath = [IO.Path]::Combine([IO.Path]::GetFullPath($StagingRoot), 'applied-environment-v1.json')
        $lockPath = [IO.Path]::Combine([IO.Path]::GetFullPath($StagingRoot), 'stage-runtime-v1.lock')
        if ($Mode -eq 'preflight') {
            $existing = [IO.Path]::GetFullPath($StagingRoot)
            while (-not (Test-Path -LiteralPath $existing)) { $existing = Split-Path -Parent $existing; if (-not $existing) { throw 'unsafe path' } }
            Assert-ProtectedTree $existing
            if (Test-Path -LiteralPath $lockPath) { throw 'uncertain state' }
            if (Test-Path -LiteralPath ($receiptPath + '.partial')) { throw 'uncertain state' }
            $present = Test-Path -LiteralPath $receiptPath
            if ($present) {
                Assert-ProtectedFile $receiptPath
                Assert-ProtectedTree (Split-Path -Parent $receiptPath)
                $receipt = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json
                if ([int]$receipt.schema -ne 1 -or [string]$receipt.manifest_sha256 -cne $Digest -or
                    [string]$receipt.vm_id -cne $VmId -or [string]$receipt.computer_name -ine $ComputerName -or
                    [string]$receipt.machine_guid -cne $MachineGuid -or [string]$receipt.windows_root -ine $env:SystemRoot -or
                    [int]$receipt.files -ne $Files.Count -or [uint64]$receipt.bytes -ne $ExpectedBytes) { throw 'uncertain state' }
            }
            foreach ($entry in $Files) {
                $path = Target ([string]$entry.windows_relative_destination)
                Check-Parent $path
                if (Test-Path -LiteralPath ($path + '.hyper-gpu-partial')) { throw 'uncertain state' }
                if (Test-Path -LiteralPath $path) { Assert-ProtectedFile $path }
            }
            return [pscustomobject]@{ present=$present; windows_root=$env:SystemRoot }
        }
        if ($Mode -eq 'lock') {
            [IO.Directory]::CreateDirectory($StagingRoot) | Out-Null
            Assert-ProtectedTree $StagingRoot
            $stream = [IO.File]::Open($lockPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
            try { $bytes = [Text.Encoding]::ASCII.GetBytes($Digest); $stream.Write($bytes, 0, $bytes.Length); $stream.Flush() }
            finally { $stream.Dispose() }
            Assert-ProtectedFile $lockPath
            return
        }
        if ($Mode -eq 'verify') {
            [uint64]$total = 0
            foreach ($entry in $Files) {
                Check-File (Target ([string]$entry.windows_relative_destination)) $entry
                $total += [uint64]$entry.bytes
            }
            if ($total -ne $ExpectedBytes) { throw 'verification failed' }
            return [pscustomobject]@{ files=$Files.Count; bytes=$total }
        }
        throw 'invalid mode'
    }
}
__STAGING_PREFLIGHT__
        $phase = 'guest-paths'
        if ($vm.Generation -ne 2 -or
            (Get-FileHash -LiteralPath $request.parent_path -Algorithm SHA256).Hash.ToLowerInvariant() -cne [string]$request.parent_sha256) { throw 'target identity mismatch' }
        foreach ($disk in @($request.parent_path, $request.child_path)) {
            Assert-SafeSourceFile ([IO.Path]::GetPathRoot([string]$disk)) ([string]$disk)
        }
        $gpuCommand = Get-Command -Name 'Get-VMGpuPartitionAdapter' -Module 'Hyper-V' -CommandType Cmdlet -ErrorAction Stop
        if (-not [IO.Path]::GetFullPath($gpuCommand.Module.Path).StartsWith($trustedModuleRoot + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'target identity mismatch' }
        # Validate the complete source closure before any guest effects.
        foreach ($entry in $request.manifest.files) {
            Assert-SafeRelative ([string]$entry.windows_relative_destination)
            Assert-SafeSourceFile ([string]$request.windows_root) ([string]$entry.source)
            $item = Get-Item -LiteralPath $entry.source -Force
            if ($item.PSIsContainer -or [uint64]$item.Length -ne [uint64]$entry.bytes -or
                (Get-FileHash -LiteralPath $entry.source -Algorithm SHA256).Hash.ToLowerInvariant() -cne [string]$entry.sha256) { throw 'source changed' }
        }
        $paths = Invoke-Environment $session $request 'preflight'
        $adapters = @(& $gpuCommand -VM $vm -ErrorAction Stop)
        if (-not $paths.present -and $adapters.Count -ne 0) { throw 'target identity mismatch' }
        $mutationStarted = $true
        Invoke-Environment $session $request 'lock'
        if (-not $paths.present) {
            foreach ($entry in $request.manifest.files) {
                # Recheck native identity and attachment immediately before every copy.
                $currentVm = & $getVmCommand -Id ([guid]$request.vm_id) -ErrorAction Stop
                $currentDisks = @(& $getVmHardDiskDriveCommand -VM $currentVm -ErrorAction Stop)
                $currentVhd = & $getVhdCommand -Path $request.child_path -ErrorAction Stop
                if ($currentVm.Name -cne [string]$request.vm_name -or $currentVm.State -cne 'Running' -or
                    $currentDisks.Count -ne 1 -or $currentDisks[0].Path -ine [string]$request.child_path -or
                    $currentVhd.ParentPath -ine [string]$request.parent_path -or
                    @(& $getVmSnapshotCommand -VM $currentVm -ErrorAction Stop).Count -ne 0 -or
                    @(& $gpuCommand -VM $currentVm -ErrorAction Stop).Count -ne 0) { throw 'target identity mismatch' }
                Assert-SafeSourceFile ([string]$request.windows_root) ([string]$entry.source)
                if ((Get-FileHash -LiteralPath $entry.source -Algorithm SHA256).Hash.ToLowerInvariant() -cne [string]$entry.sha256) { throw 'source changed' }
                $temporary = Invoke-Command -Session $session -ArgumentList @($entry.windows_relative_destination) -ScriptBlock {
                    param($Relative)
                    $ErrorActionPreference = 'Stop'
                    $root = [IO.Path]::GetFullPath($env:SystemRoot).TrimEnd('\')
                    $target = [IO.Path]::GetFullPath([IO.Path]::Combine($root, [string]$Relative))
                    if (-not $target.StartsWith($root + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'unsafe path' }
                    $parent = Split-Path -Parent $target
                    $existing = $parent
                    while (-not (Test-Path -LiteralPath $existing)) { $existing = Split-Path -Parent $existing }
                    Assert-ProtectedTree $existing $env:SystemRoot
                    [IO.Directory]::CreateDirectory($parent) | Out-Null
                    Assert-ProtectedTree $parent $env:SystemRoot
                    $partial = $target + '.hyper-gpu-partial'
                    if (Test-Path -LiteralPath $partial) { throw 'uncertain state' }
                    $partial
                }
                Copy-Item -LiteralPath $entry.source -Destination $temporary -ToSession $session -ErrorAction Stop
                Invoke-Command -Session $session -ArgumentList @($temporary, $entry.bytes, $entry.sha256) -ScriptBlock {
                    param($Partial, [uint64]$Bytes, $Hash)
                    $ErrorActionPreference = 'Stop'
                    Assert-ProtectedTree (Split-Path -Parent $Partial) $env:SystemRoot
                    Assert-ProtectedFile $Partial
                    $item = Get-Item -LiteralPath $Partial -Force
                    if ([uint64]$item.Length -ne $Bytes -or (Get-FileHash -LiteralPath $Partial -Algorithm SHA256).Hash.ToLowerInvariant() -cne [string]$Hash) { throw 'verification failed' }
                    $target = $Partial.Substring(0, $Partial.Length - '.hyper-gpu-partial'.Length)
                    if (Test-Path -LiteralPath $target) {
                        Assert-ProtectedFile $target
                        [IO.File]::Replace($Partial, $target, [NullString]::Value)
                    } else { [IO.File]::Move($Partial, $target) }
                }
            }
        }
        $verified = Invoke-Environment $session $request 'verify'
        $receipt = [ordered]@{
            schema=1; vm_id=[string]$request.vm_id; computer_name=[string]$identity.computer_name
            machine_guid=[string]$identity.machine_guid; manifest_sha256=[string]$request.manifest_sha256
            windows_root=[string]$paths.windows_root; files=[int]$verified.files; bytes=[uint64]$verified.bytes
            qualified_host_build=[string]$request.host_build; measured_host_build=$measuredBuild
            measured_guest_build=[string]$identity.build; pending_delete_sources=@($pendingDeletes.sources)
        }
        Invoke-Command -Session $session -ArgumentList @($request.staging_root, ($receipt | ConvertTo-Json -Compress), $paths.present) -ScriptBlock {
            param($Root, $Receipt, $AlreadyApplied)
            $ErrorActionPreference = 'Stop'
            Assert-ProtectedTree $Root
            $path = [IO.Path]::Combine($Root, 'applied-environment-v1.json')
            if (-not $AlreadyApplied) {
                $partial = $path + '.partial'
                if ((Test-Path -LiteralPath $path) -or (Test-Path -LiteralPath $partial)) { throw 'uncertain state' }
                [IO.File]::WriteAllText($partial, $Receipt, [Text.UTF8Encoding]::new($false))
                Assert-ProtectedFile $partial
                [IO.File]::Move($partial, $path)
            }
            Assert-ProtectedFile $path
            Remove-Item -LiteralPath ([IO.Path]::Combine($Root, 'stage-runtime-v1.lock')) -Force
        }
        $category = if ($paths.present) { 'already-applied' } else { 'applied' }
        Emit 'ok' $category ([ordered]@{ receipt=$receipt })
    } finally { if ($session) { Remove-PSSession -Session $session -ErrorAction SilentlyContinue } }
} catch {
    if ($mutationStarted -or [string]$_.Exception.Message -match 'uncertain state') { Emit 'error' 'uncertain-state'; exit 15 }
    Emit 'error' 'preflight-failed' ([ordered]@{ phase=$phase }); exit 14
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::windows_paths::windows_powershell_executable;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
        time::Duration,
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture {
        root: PathBuf,
        request: serde_json::Value,
    }
    impl Fixture {
        fn new() -> Self {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("local/tests")
                .join(format!(
                    "core022-transport-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
            fs::create_dir_all(root.join("guest/System32")).unwrap();
            fs::create_dir_all(root.join("host")).unwrap();
            let mut files = vec![];
            for (name, destination) in [
                (
                    "driver",
                    r"System32\HostDriverStore\FileRepository\nv\driver.sys",
                ),
                ("runtime", r"System32\nv.dll"),
                ("runtime32", r"SysWOW64\nv.dll"),
            ] {
                let source = root.join("host").join(name);
                fs::write(&source, name.as_bytes()).unwrap();
                files.push(
                    serde_json::json!({"source": source, "windows_relative_destination":destination,
                    "bytes":name.len(), "sha256":crate::probe::sha256_hex(name.as_bytes())}),
                );
            }
            // Exercise replacement of a pre-existing Windows file as well as creation.
            fs::write(root.join("guest/System32/nv.dll"), b"old runtime").unwrap();
            let parent = root.join("parent");
            fs::write(&parent, b"parent").unwrap();
            let request = serde_json::json!({"guest_root": root.join("guest"), "staging_root":root.join("staging"),
                "vm_id":"11111111-1111-1111-1111-111111111111", "vm_name":"vm", "child_path":root.join("child"), "parent_path":parent,
                "parent_sha256":crate::probe::sha256_hex(b"parent"), "windows_root":root.join("host"),
                "computer_name":"guest", "machine_guid":"guid", "host_build":"1.0",
                "manifest_sha256":"a".repeat(64), "byte_count":22,
                "manifest":{"files":files}, "fail_copy":0});
            Self { root, request }
        }
        fn run(&self) -> Result<EnvironmentStageResult, StagingError> {
            // Native calls/remoting and ACL trust are faked here. The unchanged
            // production body performs real local copies, publication, hashes,
            // receipts and retained locks. Existing tests cover the real ACL policy.
            let script = SCRIPT_TEMPLATE
                .replace("__STAGING_PRELUDE__", FIXTURE_PRELUDE)
                .replace("__STAGING_PREFLIGHT__", FIXTURE_PREFLIGHT)
                .replace("$env:SystemRoot", "$script:guestRoot");
            let output = run_script(
                &windows_powershell_executable().unwrap(),
                &script,
                Zeroizing::new(serde_json::to_vec(&self.request).unwrap()),
                Duration::from_secs(15),
            )
            .unwrap();
            parse_environment_result(&output)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }

    #[test]
    fn full_script_is_bounded_and_parses() {
        assert!(SCRIPT.encode_utf16().count() < 30_000);
        assert!(!SCRIPT.contains("__"));
        assert_eq!(
            run_script(
                &windows_powershell_executable().unwrap(),
                "[scriptblock]::Create([Console]::In.ReadToEnd()) | Out-Null",
                Zeroizing::new(SCRIPT.as_bytes().to_vec()),
                Duration::from_secs(5)
            ),
            Ok(String::new())
        );
    }
    #[test]
    fn writes_all_destinations_replaces_existing_bytes_and_reverifies_noop() {
        let f = Fixture::new();
        let applied = f.run().unwrap();
        assert!(!applied.already_applied);
        assert_eq!(applied.receipt.files, 3);
        assert_eq!(
            fs::read(f.root.join("guest/System32/nv.dll")).unwrap(),
            b"runtime"
        );
        assert_eq!(
            fs::read(f.root.join("guest/SysWOW64/nv.dll")).unwrap(),
            b"runtime32"
        );
        assert!(!f.root.join("staging/stage-runtime-v1.lock").exists());
        assert!(f.run().unwrap().already_applied);
        fs::write(f.root.join("guest/SysWOW64/nv.dll"), b"tampered").unwrap();
        assert_eq!(f.run(), Err(StagingError::GuestStateUncertain));
        assert!(f.root.join("staging/stage-runtime-v1.lock").exists());
    }
    #[test]
    fn partial_write_retains_lock_and_rejects_retry() {
        let mut f = Fixture::new();
        f.request["fail_copy"] = 2.into();
        assert_eq!(f.run(), Err(StagingError::GuestStateUncertain));
        assert!(f.root.join("staging/stage-runtime-v1.lock").exists());
        assert!(!f.root.join("staging/applied-environment-v1.json").exists());
        f.request["fail_copy"] = 0.into();
        assert_eq!(f.run(), Err(StagingError::GuestStateUncertain));
    }
    #[test]
    fn changed_source_is_rejected_before_guest_writes() {
        let f = Fixture::new();
        fs::write(f.root.join("host/runtime32"), b"changed").unwrap();
        assert_eq!(
            f.run(),
            Err(StagingError::GuestPreflightFailed {
                phase: "guest-paths".into()
            })
        );
        assert!(!f.root.join("staging").exists());
        assert_eq!(
            fs::read(f.root.join("guest/System32/nv.dll")).unwrap(),
            b"old runtime"
        );
    }
    #[test]
    fn saved_receipt_identity_drift_requires_recreation() {
        let f = Fixture::new();
        f.run().unwrap();
        let path = f.root.join("staging/applied-environment-v1.json");
        let mut receipt: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        receipt["machine_guid"] = "other-guest".into();
        fs::write(path, serde_json::to_vec(&receipt).unwrap()).unwrap();
        assert_eq!(f.run(), Err(StagingError::GuestStateUncertain));
        assert!(!f.root.join("staging/stage-runtime-v1.lock").exists());
    }
    #[test]
    fn fresh_staging_rejects_an_attached_guest_before_writes() {
        let mut f = Fixture::new();
        f.request["attached"] = true.into();
        assert_eq!(
            f.run(),
            Err(StagingError::GuestPreflightFailed {
                phase: "guest-paths".into()
            })
        );
        assert!(!f.root.join("staging").exists());
        assert_eq!(
            fs::read(f.root.join("guest/System32/nv.dll")).unwrap(),
            b"old runtime"
        );
    }
    #[test]
    fn full_receipt_parser_rejects_malformed_or_missing_evidence() {
        for output in [
            "not json",
            r#"{"status":"ok","category":"applied"}"#,
            r#"{"status":"ok","category":"other","receipt":{}}"#,
        ] {
            assert_eq!(
                parse_environment_result(output),
                Err(StagingError::GuestStateUncertain)
            );
        }
    }

    const FIXTURE_PRELUDE: &str = r#"
$ErrorActionPreference = 'Stop'
function Emit($Status, $Category, $Data=@{}) {
    $result=[ordered]@{status=$Status;category=$Category}
    foreach($entry in $Data.GetEnumerator()){$result[$entry.Key]=$entry.Value}
    [Console]::Out.Write(($result|ConvertTo-Json -Depth 8 -Compress))
}
function Invoke-Command { param($Session,$ArgumentList,$ScriptBlock) & $ScriptBlock @ArgumentList }
function Remove-PSSession { param($Session,$ErrorAction) }
function Assert-ProtectedTree { param($Path,$Anchor)
    $current=$Path
    while($current){$item=Get-Item -LiteralPath $current -Force;if(($item.Attributes -band [IO.FileAttributes]::ReparsePoint)-ne 0 -or -not $item.PSIsContainer){throw 'unsafe path'};if($current -ieq $request.guest_root -or $current -ieq $request.staging_root -or $current -ieq (Split-Path -Parent $request.guest_root)){break};$current=Split-Path -Parent $current}
}
function Assert-ProtectedFile { param($Path)
    $item=Get-Item -LiteralPath $Path -Force
    if($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)-ne 0){throw 'unsafe path'}
}
function Assert-SafeRelative { param($Relative) if($Relative -match '\.\.|:|^\\'){throw 'unsafe path'} }
function Assert-SafeSourceFile { param($Root,$Path) }
function Copy-Item { param($LiteralPath,$Destination,$ToSession,$ErrorAction)
    $script:copies++
    if($request.fail_copy -eq $script:copies){throw 'simulated interruption'}
    [IO.File]::Copy($LiteralPath,$Destination,$false)
}
$script:copies=0
"#;
    const FIXTURE_PREFLIGHT: &str = r#"
$request=[Console]::In.ReadToEnd()|ConvertFrom-Json
$script:guestRoot=[string]$request.guest_root
$trustedModuleRoot='C:\trusted'
function Fixed-Gpu { param($VM,$ErrorAction) if($request.attached){[pscustomobject]@{InstancePath='fixture-gpu'}}else{@()} }
$script:gpuCommand=Microsoft.PowerShell.Core\Get-Command -Name Fixed-Gpu
Add-Member -InputObject $script:gpuCommand -MemberType NoteProperty -Name Module -Value ([pscustomobject]@{Path='C:\trusted\Hyper-V'}) -Force
function Get-Command { param($Name,$Module,$CommandType,$ErrorAction) return $script:gpuCommand }
$getVmCommand={param($Id) [pscustomobject]@{Name=$request.vm_name;State='Running';Generation=2}}
$getVmHardDiskDriveCommand={param($VM) [pscustomobject]@{Path=$request.child_path}}
$getVhdCommand={param($Path) [pscustomobject]@{ParentPath=$request.parent_path}}
$getVmSnapshotCommand={param($VM) @()}
$vm=& $getVmCommand
$identity=[pscustomobject]@{computer_name=$request.computer_name;machine_guid=$request.machine_guid;build='1.1'}
$measuredBuild='1.2'
$pendingDeletes=[pscustomobject]@{sources=@()}
$session=1
$mutationStarted=$false
$phase='request'
try { try {
"#;
}
