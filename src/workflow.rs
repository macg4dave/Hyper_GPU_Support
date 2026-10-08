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

// Gather and validate read-only inputs once; preview and apply share these decisions.
fn decision(backend: &mut impl Backend, target: &Target) -> Result<Decision, String> {
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
    let managed = old.clone();
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
    if !matches!(restore_power, Power::Running | Power::Off) {
        return Err("managed restoration power is unsupported; reconcile managed state".into());
    }
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
    // Any changed enabled path writes the requested allocation after attachment,
    // including refresh when the old adapter already had matching raw values.
    let allocation_change = needs_changes && target.enabled && target.vram.is_some();
    Ok(Decision {
        initial,
        gpu,
        managed,
        journal,
        restore_power,
        settings_preserved,
        disabled_settings,
        desired,
        digest,
        stale,
        assignment,
        allocation_change,
        needs_changes,
    })
}

struct Decision {
    initial: VmState,
    gpu: Gpu,
    managed: Option<Journal>,
    journal: Journal,
    restore_power: Power,
    settings_preserved: bool,
    disabled_settings: Settings,
    desired: Settings,
    digest: Option<String>,
    stale: bool,
    assignment: bool,
    allocation_change: bool,
    needs_changes: bool,
}

/// Read-only preparation comparison against the authenticated current host payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreparationPreview {
    /// Discovered installed driver version.
    pub current_driver: String,
    /// Preparation must run because its recorded digest is absent or different.
    pub stale: bool,
    /// Authenticated current payload identity, not an operator integrity pin.
    pub digest: String,
}

/// Ordered effects expected from apply if observations remain unchanged.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreviewAction {
    /// Graceful guest shutdown; refusal stops subsequent mutation.
    Shutdown,
    /// Start the selected guest.
    Start,
    /// Remove only the selected GPU.
    Detach,
    /// Prepare the complete authenticated current payload.
    Prepare,
    /// Write compatibility settings.
    Settings,
    /// Attach the enrolled GPU.
    Attach,
    /// Write requested raw provider allocation values.
    Allocation,
    /// Require PnP health and checked hardware rendering.
    Verify,
}

/// Full before/after settings when at least one compatibility value changes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingsPreview {
    /// Current observed settings.
    pub before: Settings,
    /// Expected settings after apply or attributable restoration.
    pub after: Settings,
}

/// Shared CLI/GUI effect summary. This is a preview, never execution authorization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EffectPreview {
    /// Ordered semantic effects, including temporary boots and final restoration.
    pub actions: Vec<PreviewAction>,
    /// Compatibility changes, if any.
    pub settings: Option<SettingsPreview>,
    /// Preserve settings changed externally during disable.
    pub settings_preserved: bool,
    /// Raw allocation written by apply, including reapplication after attachment refresh.
    pub allocation: Option<Allocation>,
    /// Guest administrator credentials are required, from a prompt or stored vault entry.
    pub credentials_required: bool,
    /// The currently running guest must be stopped; no duration estimate is promised.
    pub guest_downtime: bool,
    /// Expected final power, including interrupted operation's recorded intent.
    pub restore_power: Power,
    /// Apply must reconcile unfinished per-VM work.
    pub pending_recovery: bool,
    /// Shared operator text used by both frontends.
    pub summary: Vec<String>,
}

/// Full plan; enabled targets discover/hash/authenticate the payload, so this is not a dashboard poll.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    /// Runtime intent, still constrained by protected administrator enrollment.
    pub desired: Target,
    /// Fresh Hyper-V observation, not fresh guest receipt or graphics health.
    pub observed: VmState,
    /// Existing durable state before the read-only preview.
    pub managed: Option<Journal>,
    /// Current payload comparison; disable does not discover a payload.
    pub preparation: Option<PreparationPreview>,
    /// Shared semantic effects and text.
    pub preview: EffectPreview,
    /// Allocation interpretation, retained for existing plan consumers.
    pub allocation_units: String,
    /// Whether the currently running guest needs a shutdown/start cycle.
    pub automatic_guest_restart: bool,
}

/// Preview apply using the same validation and decisions, without saving or mutating guest/VM state.
pub fn plan(backend: &mut impl Backend, target: &Target) -> Result<Plan, String> {
    let d = decision(backend, target)?;
    let mut actions = Vec::new();
    let mut power = d.initial.power.clone();
    if d.needs_changes {
        if power == Power::Running {
            actions.push(PreviewAction::Shutdown);
            power = Power::Off;
        }
        if d.stale {
            if d.assignment {
                actions.push(PreviewAction::Detach);
            }
            actions.extend([
                PreviewAction::Start,
                PreviewAction::Prepare,
                PreviewAction::Shutdown,
            ]);
            power = Power::Off;
        }
        if target.enabled {
            if d.initial.settings != d.desired {
                actions.push(PreviewAction::Settings);
            }
            if !d.assignment || d.stale {
                actions.push(PreviewAction::Attach);
            }
            if d.allocation_change {
                actions.push(PreviewAction::Allocation);
            }
        } else {
            if d.assignment {
                actions.push(PreviewAction::Detach);
            }
            if d.initial.settings != d.disabled_settings {
                actions.push(PreviewAction::Settings);
            }
        }
    }
    if target.enabled {
        if power != Power::Running {
            actions.push(PreviewAction::Start);
            power = Power::Running;
        }
        actions.push(PreviewAction::Verify);
    }
    if power != d.restore_power {
        actions.push(if d.restore_power == Power::Off {
            PreviewAction::Shutdown
        } else {
            PreviewAction::Start
        });
    }
    let after = if target.enabled {
        &d.desired
    } else {
        &d.disabled_settings
    };
    let settings = (d.initial.settings != *after).then(|| SettingsPreview {
        before: d.initial.settings.clone(),
        after: after.clone(),
    });
    let downtime = d.initial.power == Power::Running && actions.contains(&PreviewAction::Shutdown);
    let mut stopped = false;
    let restart = d.initial.power == Power::Running
        && actions.iter().any(|action| {
            if *action == PreviewAction::Shutdown {
                stopped = true;
            }
            stopped && *action == PreviewAction::Start
        });
    let pending = d.journal.pending;
    let mut summary = Vec::new();
    if pending {
        summary.push("Reconcile the pending operation before completing this request.".into());
    }
    if downtime {
        summary.push("Gracefully stop the running guest; shutdown refusal stops changes. Downtime is required.".into());
    } else if d.initial.power == Power::Running {
        summary.push("Keep the guest running; no restart is required.".into());
    } else if actions.contains(&PreviewAction::Start) {
        summary.push("Temporarily start the guest for preparation or verification.".into());
    }
    if d.stale {
        summary.push(if d.journal.prepared.is_some() {
            "Refresh guest preparation: the recorded payload differs from the current signed driver."
        } else { "Prepare the current signed driver: no matching completed preparation is recorded." }.into());
    } else if target.enabled {
        summary
            .push("Reuse the recorded current preparation; verify guest graphics afresh.".into());
    }
    if actions.contains(&PreviewAction::Detach) {
        summary.push("Detach the selected GPU.".into());
    }
    if actions.contains(&PreviewAction::Attach) {
        summary.push("Attach the selected GPU.".into());
    }
    if let Some(change) = &settings {
        summary.push(format!("Compatibility settings: low MMIO {} -> {} MiB; high MMIO {} -> {} MiB; cache types {} -> {}; automatic checkpoints {} -> {}.", change.before.low_mmio, change.after.low_mmio, change.before.high_mmio, change.after.high_mmio, change.before.cache_types, change.after.cache_types, change.before.automatic_checkpoints, change.after.automatic_checkpoints));
    }
    if d.settings_preserved {
        summary.push("Preserve compatibility settings changed by another manager.".into());
    }
    if d.allocation_change {
        summary.push("Apply requested raw VRAM values in provider-defined units; no GiB or hard-limit claim.".into());
    }
    if target.enabled && target.vram.is_none() {
        summary.push("Use provider defaults for a new attachment; retain the existing attachment's allocation otherwise.".into());
    }
    if target.enabled {
        summary.push("Guest administrator credentials are required (stored vault entry or local prompt). Check PnP health and hardware D3D11 rendering.".into());
    } else {
        summary.push(
            "Keep prepared driver files; no guest credentials or graphics check required.".into(),
        );
    }
    summary.push(format!("Final guest power: {:?}. Apply rechecks identity and state; a preview does not reserve them.", d.restore_power));
    Ok(Plan {
        desired: target.clone(),
        observed: d.initial,
        managed: d.managed,
        preparation: d.digest.map(|digest| PreparationPreview {
            current_driver: d.gpu.driver_version,
            stale: d.stale,
            digest,
        }),
        preview: EffectPreview {
            actions,
            settings,
            settings_preserved: d.settings_preserved,
            allocation: d.allocation_change.then(|| target.vram.clone()).flatten(),
            credentials_required: target.enabled,
            guest_downtime: downtime,
            restore_power: d.restore_power,
            pending_recovery: pending,
            summary,
        },
        allocation_units: "provider-defined".into(),
        automatic_guest_restart: restart,
    })
}

/// Apply one selected target after independently reading and validating current state.
/// Failures retain recovery intent; no disk recreation or forced shutdown is attempted.
pub fn apply(backend: &mut impl Backend, target: &Target) -> Result<OperationResult, String> {
    let Decision {
        initial,
        gpu,
        mut journal,
        restore_power,
        settings_preserved,
        disabled_settings,
        desired,
        digest,
        stale,
        assignment,
        needs_changes,
        ..
    } = decision(backend, target)?;
    // Verification is an operation even on a running no-op target. Persist its
    // intent before the check so failure cannot leave a completed-looking journal.
    if needs_changes || target.enabled {
        journal.pending = true;
        journal.verification_only = !needs_changes;
        if stale {
            // A failed refresh may leave a mixture of the old and new payload.
            // Invalidate the old receipt before effects, including if the host
            // later rolls back to that old driver's digest.
            journal.prepared = None;
        }
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
        driver: &'static str,
        payload_reads: usize,
        journal_writes: usize,
        fail_payload: bool,
        allocation_writes: usize,
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
            driver: "current",
            payload_reads: 0,
            journal_writes: 0,
            fail_payload: false,
            allocation_writes: 0,
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
            self.payload_reads += 1;
            if self.fail_payload {
                return Err("payload unavailable".into());
            }
            Ok(self.driver.into())
        }
        fn journal(&mut self, _: &Target) -> Result<Option<Journal>, String> {
            Ok(self.journal.clone())
        }
        fn save(&mut self, _: &Target, j: &Journal) -> Result<(), String> {
            self.journal_writes += 1;
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
            self.allocation_writes += 1;
            self.state.vram = Some(v.clone());
            Ok(())
        }
        fn prepare(&mut self, _: &Target) -> Result<String, String> {
            self.events.push("prepare");
            if self.fail_prepare {
                Err("interrupted".into())
            } else {
                Ok(self.driver.into())
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
    fn trace(preview: &EffectPreview) -> Vec<&'static str> {
        preview
            .actions
            .iter()
            .filter_map(|a| match a {
                PreviewAction::Shutdown => Some("shutdown"),
                PreviewAction::Start => Some("start"),
                PreviewAction::Detach => Some("detach"),
                PreviewAction::Prepare => Some("prepare"),
                PreviewAction::Attach => Some("attach"),
                PreviewAction::Verify => Some("graphics"),
                PreviewAction::Settings | PreviewAction::Allocation => None,
            })
            .collect()
    }

    #[test]
    fn preview_matches_enable_refresh_disable_and_recovery_without_effects() {
        for scenario in 0..8 {
            let mut f = fake();
            let mut t = target();
            if scenario >= 2 {
                apply(&mut f, &t).unwrap();
            }
            match scenario {
                1 => f.state.power = Power::Off,
                3 => f.state.power = Power::Off,
                4 => f.driver = "updated",
                5 => t.enabled = false,
                6 => {
                    t.enabled = false;
                    f.state.settings.low_mmio += 1024;
                }
                7 => {
                    f.journal.as_mut().unwrap().pending = true;
                    f.journal.as_mut().unwrap().restore_power = Power::Running;
                    f.state.power = Power::Off;
                }
                _ => {}
            }
            f.events.clear();
            let before = serde_json::to_value(&f.state).unwrap();
            let journal = serde_json::to_value(&f.journal).unwrap();
            let writes = f.journal_writes;
            let plan = plan(&mut f, &t).unwrap();
            assert_eq!(f.journal_writes, writes);
            assert!(
                f.events.is_empty(),
                "preview had effects in scenario {scenario}"
            );
            assert_eq!(serde_json::to_value(&f.state).unwrap(), before);
            assert_eq!(serde_json::to_value(&f.journal).unwrap(), journal);
            let expected_settings = plan
                .preview
                .settings
                .as_ref()
                .map_or_else(|| plan.observed.settings.clone(), |s| s.after.clone());
            let applied = apply(&mut f, &t).unwrap();
            assert_eq!(trace(&plan.preview), f.events, "scenario {scenario}");
            assert_eq!(
                applied.prepared,
                plan.preparation.as_ref().is_some_and(|p| p.stale)
            );
            assert_eq!(applied.effective.power, plan.preview.restore_power);
            assert_eq!(applied.effective.settings, expected_settings);
            assert_eq!(applied.settings_preserved, plan.preview.settings_preserved);
        }
    }

    #[test]
    fn running_verification_retry_reuses_preparation_without_restart() {
        let mut f = fake();
        apply(&mut f, &target()).unwrap();
        f.fail_verify = true;
        assert!(apply(&mut f, &target()).is_err());
        f.events.clear();
        let p = plan(&mut f, &target()).unwrap();
        assert!(p.preview.pending_recovery);
        assert!(!p.preparation.unwrap().stale);
        assert!(p.preview.credentials_required);
        assert!(!p.automatic_guest_restart);
        assert!(!p.preview.guest_downtime);
        assert_eq!(p.preview.actions, [PreviewAction::Verify]);
        assert!(f.events.is_empty());
    }

    #[test]
    fn preview_explains_raw_allocation_and_disable_without_payload_discovery() {
        let mut f = fake();
        let mut t = target();
        apply(&mut f, &t).unwrap();
        t.vram = Some(Allocation {
            minimum: 10,
            maximum: 90,
            optimal: 50,
        });
        let p = plan(&mut f, &t).unwrap();
        assert!(p.preview.actions.contains(&PreviewAction::Allocation));
        assert_eq!(p.preview.allocation, t.vram);
        assert!(p.automatic_guest_restart);
        let applied = apply(&mut f, &t).unwrap();
        assert_eq!(applied.effective.vram, t.vram);
        t.enabled = false;
        f.payload_reads = 0;
        f.fail_payload = true;
        let p = plan(&mut f, &t).unwrap();
        assert_eq!(f.payload_reads, 0);
        assert!(p.preparation.is_none());
        assert!(!p.preview.credentials_required);
        assert!(!p.preview.actions.contains(&PreviewAction::Prepare));
        assert!(!p.preview.actions.contains(&PreviewAction::Verify));
    }

    #[test]
    fn preview_rejects_wrong_identity_generation_allocation_and_journal() {
        for scenario in 0..5 {
            let mut f = fake();
            let mut t = target();
            match scenario {
                0 => f.state.vm_id = "another".into(),
                1 => f.state.generation = 1,
                2 => f.state.gpus.push("another".into()),
                3 => {
                    t.vram = Some(Allocation {
                        minimum: 0,
                        maximum: 101,
                        optimal: 50,
                    })
                }
                _ => {
                    apply(&mut f, &t).unwrap();
                    f.journal.as_mut().unwrap().gpu_interface = "another".into();
                }
            }
            f.events.clear();
            assert!(plan(&mut f, &t).is_err());
            assert!(f.events.is_empty());
        }
    }

    #[test]
    fn invalid_restore_power_is_refused_before_preview_or_apply_effects() {
        let mut f = fake();
        apply(&mut f, &target()).unwrap();
        let j = f.journal.as_mut().unwrap();
        j.pending = true;
        j.restore_power = Power::Other(4);
        f.events.clear();
        assert!(
            plan(&mut f, &target())
                .unwrap_err()
                .contains("restoration power")
        );
        assert!(
            apply(&mut f, &target())
                .unwrap_err()
                .contains("restoration power")
        );
        assert!(f.events.is_empty());
    }

    #[test]
    fn preview_reports_payload_failure_without_journal_or_guest_effects() {
        let mut f = fake();
        f.fail_payload = true;
        assert!(
            plan(&mut f, &target())
                .unwrap_err()
                .contains("payload unavailable")
        );
        assert_eq!(f.journal_writes, 0);
        assert!(f.events.is_empty());
    }

    #[test]
    fn plan_round_trip_preserves_shared_frontend_summary_and_observations() {
        let mut f = fake();
        let p = plan(&mut f, &target()).unwrap();
        let json = serde_json::to_value(&p).unwrap();
        let frontend: Plan = serde_json::from_value(json).unwrap();
        assert_eq!(frontend.preview.summary, p.preview.summary);
        assert_eq!(frontend.preview.actions, p.preview.actions);
        assert_eq!(frontend.desired, target());
        assert_eq!(frontend.observed.power, Power::Running);
        assert!(frontend.managed.is_none());
        assert_eq!(frontend.preparation.unwrap().digest, "current");
    }

    #[test]
    fn refresh_reapplies_matching_raw_allocation_after_recreating_adapter() {
        let mut f = fake();
        let mut t = target();
        t.vram = Some(Allocation {
            minimum: 10,
            maximum: 90,
            optimal: 50,
        });
        apply(&mut f, &t).unwrap();
        assert_eq!(f.state.vram, t.vram);
        f.driver = "updated";
        f.events.clear();
        f.allocation_writes = 0;
        let p = plan(&mut f, &t).unwrap();
        assert_eq!(p.preview.allocation, t.vram);
        assert!(p.preview.actions.contains(&PreviewAction::Detach));
        assert!(p.preview.actions.contains(&PreviewAction::Attach));
        assert!(p.preview.actions.contains(&PreviewAction::Allocation));
        apply(&mut f, &t).unwrap();
        assert_eq!(f.allocation_writes, 1);
        assert_eq!(trace(&p.preview), f.events);
        assert_eq!(f.state.vram, t.vram);
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
    #[test]
    fn interrupted_refresh_requires_preparation_after_host_driver_rollback() {
        let mut f = fake();
        apply(&mut f, &target()).unwrap();
        f.driver = "new-driver";
        f.fail_prepare = true;
        assert!(
            apply(&mut f, &target())
                .unwrap_err()
                .contains("interrupted")
        );
        assert!(f.state.gpus.is_empty());
        f.driver = "current";
        f.fail_prepare = false;
        f.events.clear();
        assert!(apply(&mut f, &target()).unwrap().prepared);
        let preparation = f.events.iter().position(|e| *e == "prepare").unwrap();
        let attachment = f.events.iter().position(|e| *e == "attach").unwrap();
        assert!(preparation < attachment);
        assert!(!f.journal.as_ref().unwrap().pending);
    }
}
