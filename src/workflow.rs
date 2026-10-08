//! One selected-VM workflow shared by CLI and GUI; laboratory tooling is not a dependency.
use crate::model::{Allocation, Gpu, Power, Settings, Target, VmState};
use serde::{Deserialize, Serialize};

/// Durable, per-VM ownership and interrupted-operation state.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Journal {
    /// Journal format.
    pub schema: u32,
    /// Exact selected VM.
    pub vm_id: String,
    /// Exact selected GPU.
    pub gpu_interface: String,
    /// Settings before the product first changed them.
    pub original: Settings,
    /// Last settings written by the product, for safe restoration.
    pub applied: Settings,
    /// Currently prepared driver payload digest.
    pub prepared: Option<String>,
    /// Mutation has started and has not completed independent verification.
    pub pending: bool,
    /// Power state to restore after this operation, retained across interruptions.
    pub restore_power: Power,
    /// Pending work only booted the guest for verification, without preparation changes.
    #[serde(default)]
    pub verification_only: bool,
    /// UTC Unix seconds of the last successful health/rendering check.
    #[serde(default)]
    pub last_verified: Option<u64>,
}
/// Product result; allocation is provider-defined, without enforcement claims.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationResult {
    /// Independently observed state after the operation.
    pub effective: VmState,
    /// Whether preparation was changed.
    pub prepared: bool,
    /// Graphics verification was executed successfully.
    pub verified: bool,
    /// Settings changed by another manager were preserved during disable.
    pub settings_preserved: bool,
}
/// The concrete management boundary, also used for meaningful workflow tests.
pub trait Backend {
    /// Read fresh selected VM state.
    fn inspect(&mut self, target: &Target) -> Result<VmState, String>;
    /// Read the selected host GPU and its current driver.
    fn gpu(&mut self, target: &Target) -> Result<Gpu, String>;
    /// Discover and validate a current payload; return its digest.
    fn payload(&mut self, target: &Target) -> Result<String, String>;
    /// Read product-owned durable state, if present.
    fn journal(&mut self, target: &Target) -> Result<Option<Journal>, String>;
    /// Publish protected durable state before effects.
    fn save(&mut self, target: &Target, journal: &Journal) -> Result<(), String>;
    /// Change guest power gracefully; never force power-off.
    fn power(&mut self, target: &Target, power: Power) -> Result<(), String>;
    /// Change only the selected GPU assignment and verify readback.
    fn assign(&mut self, target: &Target, enabled: bool) -> Result<(), String>;
    /// Change only GPU compatibility settings, preserving CPU/RAM/security devices.
    fn settings(&mut self, target: &Target, settings: &Settings) -> Result<(), String>;
    /// Apply supported provider VRAM values and verify readback.
    fn allocation(&mut self, target: &Target, value: &Allocation) -> Result<(), String>;
    /// Prepare the current payload in the running guest; return independently verified digest.
    fn prepare(&mut self, target: &Target) -> Result<String, String>;
    /// Require device health and checked hardware rendering.
    fn verify(&mut self, target: &Target, gpu: &Gpu) -> Result<(), String>;
}

/// Apply one runtime target. Caller owns authorization and a cross-VM mutation lock.
///
/// Failures retain the journal. No disk recreation or forced shutdown is attempted.
pub fn apply(backend: &mut impl Backend, target: &Target) -> Result<OperationResult, String> {
    target.validate()?;
    let initial = backend.inspect(target)?;
    if initial.vm_id != target.vm_id
        || initial.generation != 2
        || !matches!(initial.power, Power::Running | Power::Off)
        || initial.gpus.len() > 1
        || initial.gpus.iter().any(|g| g != &target.gpu_interface)
    {
        return Err(
            "selected VM is unsupported, transitional, or has another GPU assignment".into(),
        );
    }
    let gpu = backend.gpu(target)?;
    if gpu.interface != target.gpu_interface {
        return Err("selected GPU identity changed".into());
    }
    if target.enabled && !gpu.preparation_supported {
        return Err("automatic guest preparation is not implemented for this GPU vendor".into());
    }
    if let Some(vram) = &target.vram {
        vram.validate(&gpu.vram)?;
    }
    let old = backend.journal(target)?;
    if old.as_ref().is_some_and(|j| {
        j.schema != 1 || j.vm_id != target.vm_id || j.gpu_interface != target.gpu_interface
    }) {
        return Err("managed state belongs to a different target; reconcile enrollment".into());
    }
    let mut journal = old.unwrap_or(Journal {
        schema: 1,
        vm_id: target.vm_id.clone(),
        gpu_interface: target.gpu_interface.clone(),
        original: initial.settings.clone(),
        applied: initial.settings.for_gpu(),
        prepared: None,
        pending: false,
        restore_power: initial.power.clone(),
        verification_only: false,
        last_verified: None,
    });
    if !journal.pending {
        journal.restore_power = initial.power.clone();
        if initial.gpus.is_empty() && target.enabled {
            // A completed disable releases ownership of compatibility settings.
            // Capture the new baseline before changing them again.
            journal.original = initial.settings.clone();
        }
    }
    let restore_power = journal.restore_power.clone();
    let settings_preserved = !target.enabled
        && initial.settings != journal.applied
        && initial.settings != journal.original;
    let disabled_settings = if settings_preserved {
        initial.settings.clone()
    } else {
        journal.original.clone()
    };
    let desired = initial.settings.for_gpu();
    let digest = if target.enabled {
        Some(backend.payload(target)?)
    } else {
        None
    };
    // The prepared digest is published only after a complete verified preparation.
    // A later Hyper-V failure must not turn that receipt back into partial staging.
    let stale = target.enabled && journal.prepared != digest;
    let assignment = !initial.gpus.is_empty();
    let settings_change = target.enabled && initial.settings != desired;
    let allocation_change = target.enabled
        && target
            .vram
            .as_ref()
            .is_some_and(|v| initial.vram.as_ref() != Some(v));
    let needs_changes = assignment != target.enabled
        || stale
        || settings_change
        || allocation_change
        || (journal.pending && !journal.verification_only)
        || (!target.enabled
            && initial.settings == journal.applied
            && initial.settings != journal.original);
    // Verification is an operation even on a running no-op target. Persist its
    // intent before the check so failure cannot leave a completed-looking journal.
    if needs_changes || target.enabled {
        journal.pending = true;
        journal.verification_only = !needs_changes;
        backend.save(target, &journal)?;
    }
    let operation: Result<OperationResult, String> = (|| {
        if needs_changes {
            if initial.power == Power::Running {
                backend.power(target, Power::Off)?;
            }
            if stale {
                if assignment {
                    backend.assign(target, false)?;
                }
                backend.power(target, Power::Running)?;
                let prepared = backend.prepare(target)?;
                if Some(&prepared) != digest.as_ref() {
                    return Err(
                        "driver changed during preparation; reconcile and rediscover".into(),
                    );
                }
                backend.power(target, Power::Off)?;
                journal.prepared = Some(prepared);
                backend.save(target, &journal)?;
            }
            if target.enabled {
                backend
                    .settings(target, &desired)
                    .map_err(|e| format!("apply compatibility settings: {e}"))?;
                journal.applied = desired.clone();
                backend.save(target, &journal)?;
                if backend.inspect(target)?.gpus.is_empty() {
                    backend
                        .assign(target, true)
                        .map_err(|e| format!("attach selected GPU: {e}"))?;
                }
                if let Some(vram) = &target.vram {
                    backend.allocation(target, vram)?;
                }
            } else {
                if assignment {
                    backend.assign(target, false)?;
                }
                if initial.settings == journal.applied {
                    backend.settings(target, &journal.original)?;
                }
            }
        }
        let mut verified = false;
        if target.enabled {
            if backend.inspect(target)?.power != Power::Running {
                backend.power(target, Power::Running)?;
            }
            backend.verify(target, &gpu)?;
            verified = true;
        }
        if backend.inspect(target)?.power != restore_power {
            backend.power(target, restore_power.clone())?;
        }
        let effective = backend.inspect(target)?;
        let expected_gpus = if target.enabled {
            vec![target.gpu_interface.clone()]
        } else {
            vec![]
        };
        if effective.gpus != expected_gpus
            || effective.power != restore_power
            || (target.enabled && effective.settings != desired)
            || (!target.enabled && effective.settings != disabled_settings)
            || (target.enabled
                && target
                    .vram
                    .as_ref()
                    .is_some_and(|v| effective.vram.as_ref() != Some(v)))
        {
            return Err("independent final state verification failed".into());
        }
        journal.pending = false;
        if verified {
            journal.last_verified = Some(verified_at());
        }
        backend.save(target, &journal)?;
        Ok(OperationResult {
            effective,
            prepared: stale,
            verified,
            settings_preserved,
        })
    })();
    // Only restore power after success. A failed preparation must not trigger an
    // attached boot using partial guest files. Pending state explains the next apply.
    operation.map_err(|error| {
        if journal.verification_only && restore_power == Power::Off {
            let restoration = restore_guest_power(backend, target, &Power::Off);
            return format!(
                "GPU-PV verification failed: {error}; power restoration: {}",
                restoration.err().unwrap_or_else(|| "completed".into())
            );
        }
        format!("GPU-PV operation failed: {error}")
    })
}

/// Verify an enrolled assignment with durable lifecycle intent and graceful restoration.
pub fn verify(backend: &mut impl Backend, target: &Target) -> Result<OperationResult, String> {
    target.validate()?;
    let before = backend.inspect(target)?;
    if before.vm_id != target.vm_id
        || before.generation != 2
        || before.gpus != vec![target.gpu_interface.clone()]
        || !matches!(before.power, Power::Running | Power::Off)
    {
        return Err("verification requires the selected, stable GPU assignment".into());
    }
    let old = backend.journal(target)?;
    if old.as_ref().is_some_and(|j| {
        j.schema != 1
            || j.vm_id != target.vm_id
            || j.gpu_interface != target.gpu_interface
            || (j.pending && !j.verification_only)
    }) {
        return Err(
            "unfinished management operation requires apply reconciliation before verification"
                .into(),
        );
    }
    let gpu = backend.gpu(target)?;
    if gpu.interface != target.gpu_interface {
        return Err("selected GPU identity changed".into());
    }
    let mut journal = old.unwrap_or(Journal {
        schema: 1,
        vm_id: target.vm_id.clone(),
        gpu_interface: target.gpu_interface.clone(),
        original: before.settings.clone(),
        applied: before.settings.clone(),
        prepared: None,
        pending: false,
        restore_power: before.power.clone(),
        verification_only: true,
        last_verified: None,
    });
    if !journal.pending {
        journal.restore_power = before.power.clone();
    }
    journal.pending = true;
    journal.verification_only = true;
    backend.save(target, &journal)?;
    let checked = (|| {
        if before.power == Power::Off {
            backend.power(target, Power::Running)?;
        }
        backend.verify(target, &gpu)
    })();
    // A start error may follow a real state change. Inspect and restore even then,
    // retaining both failures and the durable intent if recovery cannot complete.
    let restoration = restore_guest_power(backend, target, &journal.restore_power);
    match (checked, restoration) {
        (Err(primary), Err(recovery)) => {
            return Err(format!(
                "GPU-PV verification failed: {primary}; power restoration failed: {recovery}"
            ));
        }
        (Err(primary), Ok(())) => return Err(primary),
        (Ok(()), Err(recovery)) => {
            return Err(format!("power restoration failed: {recovery}"));
        }
        (Ok(()), Ok(())) => {}
    }
    let effective = backend.inspect(target)?;
    if effective.power != journal.restore_power || effective.gpus != before.gpus {
        return Err("verification lifecycle readback failed".into());
    }
    journal.pending = false;
    journal.last_verified = Some(verified_at());
    backend.save(target, &journal)?;
    Ok(OperationResult {
        effective,
        prepared: false,
        verified: true,
        settings_preserved: false,
    })
}
fn restore_guest_power(
    backend: &mut impl Backend,
    target: &Target,
    desired: &Power,
) -> Result<(), String> {
    if backend.inspect(target)?.power != *desired {
        backend.power(target, desired.clone())?;
    }
    if backend.inspect(target)?.power != *desired {
        return Err("power restoration readback failed".into());
    }
    Ok(())
}
fn verified_at() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fake {
        state: VmState,
        journal: Option<Journal>,
        events: Vec<&'static str>,
        fail_prepare: bool,
        fail_settings: bool,
        fail_verify: bool,
        fail_shutdown: bool,
        fail_start_after_effect: bool,
    }
    fn target() -> Target {
        Target {
            vm_id: "11111111-1111-1111-1111-111111111111".into(),
            gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#test\GPUPARAV".into(),
            enabled: true,
            vram: None,
        }
    }
    fn fake() -> Fake {
        Fake {
            state: VmState {
                vm_id: target().vm_id,
                name: "renamed VM".into(),
                power: Power::Running,
                generation: 2,
                gpus: vec![],
                vram: None,
                settings: Settings {
                    low_mmio: 128,
                    high_mmio: 512,
                    cache_types: false,
                    automatic_checkpoints: true,
                },
            },
            journal: None,
            events: vec![],
            fail_prepare: false,
            fail_settings: false,
            fail_verify: false,
            fail_shutdown: false,
            fail_start_after_effect: false,
        }
    }
    impl Backend for Fake {
        fn inspect(&mut self, _: &Target) -> Result<VmState, String> {
            Ok(self.state.clone())
        }
        fn gpu(&mut self, t: &Target) -> Result<Gpu, String> {
            Ok(Gpu {
                interface: t.gpu_interface.clone(),
                name: "GPU".into(),
                vendor: 0x10de,
                device: 1,
                driver_version: "new".into(),
                preparation_supported: true,
                vram: Allocation {
                    minimum: 0,
                    maximum: 100,
                    optimal: 50,
                },
            })
        }
        fn payload(&mut self, _: &Target) -> Result<String, String> {
            Ok("current".into())
        }
        fn journal(&mut self, _: &Target) -> Result<Option<Journal>, String> {
            Ok(self.journal.clone())
        }
        fn save(&mut self, _: &Target, j: &Journal) -> Result<(), String> {
            self.journal = Some(j.clone());
            Ok(())
        }
        fn power(&mut self, _: &Target, p: Power) -> Result<(), String> {
            self.events.push(if p == Power::Running {
                "start"
            } else {
                "shutdown"
            });
            if p == Power::Off && self.fail_shutdown {
                return Err("shutdown refused".into());
            }
            self.state.power = p;
            if self.state.power == Power::Running && self.fail_start_after_effect {
                return Err("start readback interrupted".into());
            }
            Ok(())
        }
        fn assign(&mut self, t: &Target, e: bool) -> Result<(), String> {
            self.events.push(if e { "attach" } else { "detach" });
            self.state.gpus = if e {
                vec![t.gpu_interface.clone()]
            } else {
                vec![]
            };
            Ok(())
        }
        fn settings(&mut self, _: &Target, s: &Settings) -> Result<(), String> {
            if self.fail_settings {
                return Err("settings interrupted".into());
            }
            self.state.settings = s.clone();
            Ok(())
        }
        fn allocation(&mut self, _: &Target, v: &Allocation) -> Result<(), String> {
            self.state.vram = Some(v.clone());
            Ok(())
        }
        fn prepare(&mut self, _: &Target) -> Result<String, String> {
            self.events.push("prepare");
            if self.fail_prepare {
                Err("interrupted".into())
            } else {
                Ok("current".into())
            }
        }
        fn verify(&mut self, _: &Target, _: &Gpu) -> Result<(), String> {
            self.events.push("graphics");
            if self.fail_verify {
                return Err("graphics failed".into());
            }
            Ok(())
        }
    }
    #[test]
    fn prepare_precedes_attachment_noop_has_no_restart_disable_preserves_files() {
        let mut f = fake();
        let mut t = target();
        assert!(apply(&mut f, &t).unwrap().verified);
        assert_eq!(
            f.events,
            [
                "shutdown", "start", "prepare", "shutdown", "attach", "start", "graphics"
            ]
        );
        f.events.clear();
        assert!(!apply(&mut f, &t).unwrap().prepared);
        assert_eq!(f.events, ["graphics"]);
        t.enabled = false;
        f.events.clear();
        apply(&mut f, &t).unwrap();
        assert_eq!(f.events, ["shutdown", "detach", "start"]);
        assert_eq!(
            f.journal.as_ref().unwrap().prepared.as_deref(),
            Some("current")
        );
        assert!(!f.state.settings.cache_types);
    }
    #[test]
    fn partial_preparation_is_durable_and_never_attached() {
        let mut f = fake();
        f.fail_prepare = true;
        assert!(
            apply(&mut f, &target())
                .unwrap_err()
                .contains("interrupted")
        );
        assert!(f.journal.as_ref().unwrap().pending);
        assert!(!f.events.contains(&"attach"));
        f.fail_prepare = false;
        assert!(apply(&mut f, &target()).is_ok());
    }
    #[test]
    fn unrelated_gpu_is_rejected_before_mutation() {
        let mut f = fake();
        f.state.gpus.push("another".into());
        assert!(apply(&mut f, &target()).is_err());
        assert!(f.events.is_empty());
        assert!(f.journal.is_none());
    }
    #[test]
    fn interrupted_disable_restores_settings_and_original_running_state_on_retry() {
        let mut f = fake();
        let mut t = target();
        apply(&mut f, &t).unwrap();
        t.enabled = false;
        f.fail_settings = true;
        assert!(apply(&mut f, &t).is_err());
        assert!(f.state.gpus.is_empty());
        assert_eq!(f.state.power, Power::Off);
        assert!(f.journal.as_ref().unwrap().pending);
        f.fail_settings = false;
        apply(&mut f, &t).unwrap();
        assert_eq!(f.state.power, Power::Running);
        assert!(!f.state.settings.cache_types);
    }
    #[test]
    fn disable_preserves_external_settings_and_completes_power_restoration() {
        let mut f = fake();
        let mut t = target();
        apply(&mut f, &t).unwrap();
        f.state.settings.high_mmio += 1024;
        let external = f.state.settings.clone();
        t.enabled = false;
        assert!(apply(&mut f, &t).unwrap().settings_preserved);
        assert_eq!(f.state.power, Power::Running);
        assert_eq!(f.state.settings, external);
        assert!(!f.journal.as_ref().unwrap().pending);
    }
    #[test]
    fn failed_off_noop_verification_restores_power_without_repreparation() {
        let mut f = fake();
        f.state.power = Power::Off;
        apply(&mut f, &target()).unwrap();
        f.events.clear();
        f.fail_verify = true;
        assert!(apply(&mut f, &target()).is_err());
        assert_eq!(f.state.power, Power::Off);
        assert_eq!(f.journal.as_ref().unwrap().restore_power, Power::Off);
        assert!(f.journal.as_ref().unwrap().pending);
        f.fail_verify = false;
        f.events.clear();
        apply(&mut f, &target()).unwrap();
        assert_eq!(f.events, ["start", "graphics", "shutdown"]);
    }
    #[test]
    fn standalone_verify_retains_interrupted_off_intent_and_refuses_pending_preparation() {
        let mut f = fake();
        f.state.power = Power::Off;
        apply(&mut f, &target()).unwrap();
        let j = f.journal.as_mut().unwrap();
        j.pending = true;
        j.verification_only = true;
        f.state.power = Power::Running;
        verify(&mut f, &target()).unwrap();
        assert_eq!(f.state.power, Power::Off);
        let j = f.journal.as_mut().unwrap();
        j.pending = true;
        j.verification_only = false;
        f.events.clear();
        assert!(verify(&mut f, &target()).is_err());
        assert!(f.events.is_empty());
    }
    #[test]
    fn failed_running_reapply_is_pending_and_retry_does_not_restart_or_prepare() {
        let mut f = fake();
        apply(&mut f, &target()).unwrap();
        let previous_verified = f.journal.as_ref().unwrap().last_verified;
        f.events.clear();
        f.fail_verify = true;
        assert!(
            apply(&mut f, &target())
                .unwrap_err()
                .contains("graphics failed")
        );
        let j = f.journal.as_ref().unwrap();
        assert!(j.pending && j.verification_only);
        assert_eq!(j.last_verified, previous_verified);
        assert_eq!(f.events, ["graphics"]);
        f.fail_verify = false;
        f.events.clear();
        apply(&mut f, &target()).unwrap();
        assert_eq!(f.events, ["graphics"]);
        assert!(!f.journal.as_ref().unwrap().pending);
    }
    #[test]
    fn standalone_verify_preserves_check_and_restoration_failures_for_retry() {
        let mut f = fake();
        f.state.power = Power::Off;
        apply(&mut f, &target()).unwrap();
        f.events.clear();
        f.fail_verify = true;
        f.fail_shutdown = true;
        let error = verify(&mut f, &target()).unwrap_err();
        assert!(error.contains("graphics failed"));
        assert!(error.contains("shutdown refused"));
        assert_eq!(f.events, ["start", "graphics", "shutdown"]);
        assert_eq!(f.state.power, Power::Running);
        let j = f.journal.as_ref().unwrap();
        assert!(j.pending && j.verification_only);
        assert_eq!(j.restore_power, Power::Off);
        f.fail_verify = false;
        f.fail_shutdown = false;
        f.events.clear();
        verify(&mut f, &target()).unwrap();
        assert_eq!(f.events, ["graphics", "shutdown"]);
        assert_eq!(f.state.power, Power::Off);
        assert!(!f.journal.as_ref().unwrap().pending);
    }
    #[test]
    fn uncertain_start_restores_off_and_retains_verification_intent() {
        let mut f = fake();
        f.state.power = Power::Off;
        apply(&mut f, &target()).unwrap();
        f.events.clear();
        f.fail_start_after_effect = true;
        assert!(
            verify(&mut f, &target())
                .unwrap_err()
                .contains("start readback interrupted")
        );
        assert_eq!(f.events, ["start", "shutdown"]);
        assert_eq!(f.state.power, Power::Off);
        assert!(f.journal.as_ref().unwrap().pending);
    }
    #[test]
    fn shutdown_failure_never_prepares_or_detaches() {
        let mut f = fake();
        f.fail_shutdown = true;
        assert!(
            apply(&mut f, &target())
                .unwrap_err()
                .contains("shutdown refused")
        );
        assert_eq!(f.events, ["shutdown"]);
        assert_eq!(f.state.power, Power::Running);
        assert!(f.journal.as_ref().unwrap().pending);
    }
    #[test]
    fn completed_preparation_survives_settings_failure_without_retransfer() {
        let mut f = fake();
        f.fail_settings = true;
        assert!(
            apply(&mut f, &target())
                .unwrap_err()
                .contains("apply compatibility settings")
        );
        assert_eq!(
            f.journal.as_ref().unwrap().prepared.as_deref(),
            Some("current")
        );
        assert!(f.journal.as_ref().unwrap().pending);
        f.fail_settings = false;
        f.events.clear();
        assert!(!apply(&mut f, &target()).unwrap().prepared);
        assert_eq!(f.events, ["attach", "start", "graphics"]);
        assert!(!f.journal.as_ref().unwrap().pending);
    }
}
