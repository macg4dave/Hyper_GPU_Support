//! Fixed native Hyper-V cmdlet glue (DEC-016). Rust owns profile decisions.

use hyper_gpu_support::config::ProjectConfiguration;
use hyper_gpu_support::vm_settings::GpuResources;

// Executed after the fixed identity/disk guard. Fresh cmdlets resolve every object.
pub(super) const READ: &str = r#"
function Read-Profile {
    $v = Get-VM -Id $vmId -ErrorAction Stop
    $m = Get-VMMemory -VM $v -ErrorAction Stop
    $p = Get-VMProcessor -VM $v -ErrorAction Stop
    $f = Get-VMFirmware -VM $v -ErrorAction Stop
    $s = Get-VMSecurity -VM $v -ErrorAction Stop
    $a = @(Get-VMGpuPartitionAdapter -VM $v -ErrorAction Stop)
    if ($v.Name -ne $vmName -or $v.State -ne 'Off' -or $v.Generation -ne 2 -or
        $a.Count -ne 1 -or $a[0].InstancePath -ine $gpuPath) { throw 'settings target/state/adapter mismatch' }
    $h = Get-VMHostPartitionableGpu -Name $gpuPath -ErrorAction Stop
    if ($h.Name -ine $gpuPath) { throw 'settings provider identity mismatch' }
    $actual = [ordered]@{}
    $limits = [ordered]@{}
    foreach ($pair in @(@('vram','VRAM'),@('encode','Encode'),@('decode','Decode'),@('compute','Compute'))) {
        $n = $pair[1]
        $actual[$pair[0]] = [ordered]@{ minimum=[uint64]$a[0].("MinPartition$n"); maximum=[uint64]$a[0].("MaxPartition$n"); optimal=[uint64]$a[0].("OptimalPartition$n") }
        $limits[$pair[0]] = [ordered]@{ minimum=[uint64]$h.("MinPartition$n"); maximum=[uint64]$h.("MaxPartition$n"); optimal=[uint64]$h.("OptimalPartition$n") }
    }
    [ordered]@{
        vm_id=$v.Id.ToString().ToLowerInvariant(); state=[string]$v.State
        gpu_interface=[string]$a[0].InstancePath; gpu_adapters=$a.Count
        secure_boot=([string]$f.SecureBoot -eq 'On'); secure_boot_template=[string]$f.SecureBootTemplate
        tpm_enabled=[bool]$s.TpmEnabled; dynamic_memory=[bool]$m.DynamicMemoryEnabled
        checkpoint_type=[string]$v.CheckpointType; automatic_checkpoints=[bool]$v.AutomaticCheckpointsEnabled
        automatic_stop_action=[string]$v.AutomaticStopAction
        profile=[ordered]@{
            memory_bytes=[uint64]$m.Startup; processors=[uint32]$p.Count
            low_mmio_bytes=[uint64]$v.LowMemoryMappedIoSpace; high_mmio_bytes=[uint64]$v.HighMemoryMappedIoSpace
            guest_controlled_cache_types=[bool]$v.GuestControlledCacheTypes
            expose_virtualization_extensions=[bool]$p.ExposeVirtualizationExtensions
            checkpoints_disabled=([string]$v.CheckpointType -eq 'Disabled' -and -not $v.AutomaticCheckpointsEnabled)
            automatic_stop_guest_shutdown=([string]$v.AutomaticStopAction -eq 'ShutDown')
        }
        resources=$actual; limits=$limits
    } | ConvertTo-Json -Depth 5 -Compress
}
"#;

/// Generate numeric/boolean-only calls from validated, pinned Rust values.
pub(super) fn apply(project: &ProjectConfiguration) -> Result<String, Box<dyn std::error::Error>> {
    project.vm_profile.validate()?;
    let p = &project.vm_profile;
    let r = GpuResources::desired(project)?;
    let mut script = format!(
        "Set-VM -VM $vm -LowMemoryMappedIoSpace {} -HighMemoryMappedIoSpace {} -GuestControlledCacheTypes ${} -CheckpointType Disabled -AutomaticCheckpointsEnabled $false -AutomaticStopAction ShutDown -ErrorAction Stop\nSet-VMMemory -VM $vm -DynamicMemoryEnabled $false -StartupBytes {} -ErrorAction Stop\nSet-VMProcessor -VM $vm -Count {} -ExposeVirtualizationExtensions ${} -ErrorAction Stop\n",
        p.low_mmio_bytes,
        p.high_mmio_bytes,
        p.guest_controlled_cache_types,
        p.memory_bytes,
        p.processors,
        p.expose_virtualization_extensions,
    );
    script.push_str("Set-VMGpuPartitionAdapter -VM $vm");
    for (name, triple) in [
        ("VRAM", r.vram),
        ("Encode", r.encode),
        ("Decode", r.decode),
        ("Compute", r.compute),
    ] {
        // String casts avoid PowerShell rounding a numeric literal beyond i64.
        script.push_str(&format!(
            " -MinPartition{name} ([uint64]'{}') -MaxPartition{name} ([uint64]'{}') -OptimalPartition{name} ([uint64]'{}')",
            triple.minimum, triple.maximum, triple.optimal,
        ));
    }
    script.push_str(" -ErrorAction Stop\n[Console]::Out.WriteLine((Read-Profile))\n");
    Ok(script)
}

#[cfg(test)]
mod tests {
    use super::{READ, apply};
    use crate::{fixed_script, powershell_literal, run_bounded, settings_guard};
    use hyper_gpu_support::config::ProjectConfiguration;
    use hyper_gpu_support::vm_settings::SettingsSnapshot;
    use std::time::Duration;

    fn run(body: &str) -> Result<String, String> {
        // Only test fakes define these cmdlets; no native mutation is invoked.
        let script =
            fixed_script(&format!("{}\n{}", include_str!("settings_fakes.ps1"), body)).unwrap();
        run_bounded(&script, Duration::from_secs(10))
    }

    #[test]
    fn native_mapping_applies_every_field_and_preserves_unsigned_encode() {
        let project = ProjectConfiguration::embedded().unwrap();
        let output = run(&format!(
            "{}\n{READ}\n[Console]::Out.WriteLine((Read-Profile))\n{}",
            settings_guard().unwrap(),
            apply(&project).unwrap()
        ))
        .unwrap();
        let lines: Vec<_> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        let before: SettingsSnapshot = serde_json::from_str(lines[0]).unwrap();
        let after: SettingsSnapshot = serde_json::from_str(lines[1]).unwrap();
        assert!(!before.matches(&project).unwrap());
        after.verify(&before, &project).unwrap();
        assert_eq!(after.resources.encode.optimal, 1 << 63);
    }

    #[test]
    fn native_partial_update_and_stale_preimage_never_succeed() {
        let project = ProjectConfiguration::embedded().unwrap();
        let partial = run(&format!(
            "{}\n{READ}\n$script:failGpu=$true\n{}",
            settings_guard().unwrap(),
            apply(&project).unwrap()
        ))
        .unwrap_err();
        assert!(
            partial.contains("injected partial GPU update failure"),
            "{partial}"
        );
        let before = run(&format!("{READ}\n[Console]::Out.WriteLine((Read-Profile))")).unwrap();
        let stale = run(&format!("{}\n{READ}\n$script:cpu.Count=3\nif ((Read-Profile) -cne {}) {{ throw 'stale settings preimage rejected' }}\n{}", settings_guard().unwrap(), powershell_literal(before.trim()), apply(&project).unwrap())).unwrap_err();
        assert!(
            stale.contains("stale settings preimage rejected"),
            "{stale}"
        );
        // Independent fake processes return a stable snapshot for the CAS guard.
        let same = run(&format!("{READ}\n[Console]::Out.WriteLine((Read-Profile))")).unwrap();
        assert_eq!(same, before);
    }
}
