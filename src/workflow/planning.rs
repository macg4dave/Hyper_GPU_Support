//! Read-only decisions and shared operation previews.
use super::{Backend, Journal};
use crate::model::{Allocation, Gpu, Power, Resource, Settings, Target, VmState};
use serde::{Deserialize, Serialize};

// Gather and validate read-only inputs once; preview and apply share these decisions.
pub(super) fn decision(backend: &mut impl Backend, target: &Target) -> Result<Decision, String> {
    decision_for_operation(backend, target, false)
}

pub(super) fn decision_for_operation(
    backend: &mut impl Backend,
    target: &Target,
    reinstall: bool,
) -> Result<Decision, String> {
    decision_inputs(backend, target, reinstall, false)
}

fn decision_inputs(
    backend: &mut impl Backend,
    target: &Target,
    reinstall: bool,
    preliminary: bool,
) -> Result<Decision, String> {
    target.validate()?;
    if reinstall && !target.enabled {
        return Err("driver reinstall requires enabled GPU support".into());
    }
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
    if target.enabled {
        target.validate_allocations(&gpu)?;
    }
    let old = backend.journal(target)?;
    if old.as_ref().is_some_and(|j| {
        j.schema != 1 || j.vm_id != target.vm_id || j.gpu_interface != target.gpu_interface
    }) {
        return Err("managed state belongs to a different target; reconcile enrollment".into());
    }
    let managed = old.clone();
    // Attachment alone and standalone graphics checks do not prove preparation.
    // Existing successful preparation + graphics receipts remain valid history.
    let initial_setup = target.enabled
        && !managed.as_ref().is_some_and(|journal| {
            journal.setup_eligible()
        })
        // An interrupted older operation retains its own recorded lifecycle.
        && managed.as_ref().is_none_or(|journal| {
            !journal.pending || journal.restore_power == Power::Off
        });
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
        latest_check: None,
        preparation_started: None,
        check_restored: false,
        settings_released: false,
    });
    if !journal.pending {
        journal.restore_power = Power::Off;
        if target.enabled && journal.settings_released {
            // A completed disable releases ownership of compatibility settings.
            // Capture the new baseline before changing them again.
            journal.original = initial.settings.clone();
        }
    }
    let restore_power = journal.restore_power.clone();
    if !matches!(restore_power, Power::Running | Power::Off) {
        return Err("managed restoration power is unsupported; reconcile managed state".into());
    }
    let (disabled_settings, preserved) =
        restore_owned_settings(&initial.settings, managed.as_ref());
    let settings_preserved = !target.enabled && preserved;
    let desired = initial.settings.for_gpu();
    // Recorded evidence predicts ordinary changes only. Admission always uses
    // the full current payload and rejects a changed digest or effect scope.
    let authentication_deferred =
        preliminary && target.enabled && !reinstall && !journal.pending && journal.setup_eligible();
    let digest = if authentication_deferred {
        journal.prepared.clone()
    } else if target.enabled {
        Some(backend.payload(target)?)
    } else {
        None
    };
    // The prepared digest is published only after a complete verified preparation.
    // A later Hyper-V failure must not turn that receipt back into partial staging.
    let stale = target.enabled && (reinstall || journal.prepared != digest);
    let setup_required = target.enabled && (stale || !journal.setup_eligible());
    let assignment = !initial.gpus.is_empty();
    let settings_change = target.enabled && initial.settings != desired;
    let allocation_change = target.enabled && !target.allocations_match(&initial);
    let needs_changes = assignment != target.enabled
        || stale
        || settings_change
        || allocation_change
        || (journal.pending && !journal.verification_only)
        || (!target.enabled && initial.settings != disabled_settings);
    // Any changed enabled path writes the requested allocation after attachment,
    // including refresh when the old adapter already had matching raw values.
    let allocation_change = needs_changes
        && target.enabled
        && Resource::ALL
            .into_iter()
            .any(|r| target.allocation(r).is_some());
    Ok(Decision {
        authentication_deferred,
        reinstall,
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
        initial_setup,
        setup_required,
    })
}

#[derive(Clone)]
pub(super) struct Decision {
    authentication_deferred: bool,
    reinstall: bool,
    pub(super) initial: VmState,
    pub(super) gpu: Gpu,
    pub(super) managed: Option<Journal>,
    pub(super) journal: Journal,
    pub(super) restore_power: Power,
    pub(super) settings_preserved: bool,
    pub(super) disabled_settings: Settings,
    pub(super) desired: Settings,
    pub(super) digest: Option<String>,
    pub(super) stale: bool,
    pub(super) assignment: bool,
    allocation_change: bool,
    pub(super) needs_changes: bool,
    initial_setup: bool,
    pub(super) setup_required: bool,
}

// Ownership is per setting. Guest preparation history is not host-setting ownership.
fn restore_owned_settings(current: &Settings, journal: Option<&Journal>) -> (Settings, bool) {
    let Some(journal) = journal else {
        return (current.clone(), true);
    };
    if journal.settings_released {
        return (current.clone(), true);
    }
    let mut after = current.clone();
    if current.low_mmio == journal.applied.low_mmio {
        after.low_mmio = journal.original.low_mmio;
    }
    if current.high_mmio == journal.applied.high_mmio {
        after.high_mmio = journal.original.high_mmio;
    }
    if current.cache_types == journal.applied.cache_types {
        after.cache_types = journal.original.cache_types;
    }
    if current.automatic_checkpoints == journal.applied.automatic_checkpoints {
        after.automatic_checkpoints = journal.original.automatic_checkpoints;
    }
    let preserved = after != journal.original;
    (after, preserved)
}

/// Read-only preparation comparison against the authenticated current host payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreparationPreview {
    /// Discovered installed driver version.
    pub current_driver: String,
    /// Preparation must run: missing/different receipt or explicitly reviewed reinstall.
    pub stale: bool,
    /// Payload identity; historical when the enclosing plan defers authentication.
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
    /// Explicit compute write, absent when omitted or no change path is needed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compute: Option<Allocation>,
    /// Explicit encode write.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encode: Option<Allocation>,
    /// Explicit decode write.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decode: Option<Allocation>,
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

/// Reviewed effect scope. Preliminary reviews defer payload authentication to admission.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    /// Payload identity is historical; Apply must authenticate and compare it before effects.
    #[serde(default)]
    pub authentication_deferred: bool,
    /// Explicitly reviewed complete driver preparation, even with matching history.
    #[serde(default)]
    pub reinstall: bool,
    /// No completed preparation/graphics receipts exist; success must leave the VM off.
    #[serde(default)]
    pub initial_setup: bool,
    /// Runtime intent, still constrained by protected administrator enrollment.
    pub desired: Target,
    /// Fresh Hyper-V observation, not fresh guest receipt or graphics health.
    pub observed: VmState,
    /// Existing durable state before the read-only preview.
    pub managed: Option<Journal>,
    /// Payload comparison; historical when authentication is deferred, absent on disable.
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
    Ok(plan_from_decision(d, target))
}
/// Predict ordinary changes from fresh host facts and completed historical receipts.
/// Missing, adverse or pending history falls back to full payload authentication.
/// This preview never substitutes for authenticated admission or guest health.
pub fn plan_preliminary(backend: &mut impl Backend, target: &Target) -> Result<Plan, String> {
    let d = decision_inputs(backend, target, false, true)?;
    Ok(plan_from_decision(d, target))
}
/// Preview an explicit full reinstall through the ordinary protected operation path.
pub fn plan_reinstall(backend: &mut impl Backend, target: &Target) -> Result<Plan, String> {
    let d = decision_for_operation(backend, target, true)?;
    Ok(plan_from_decision(d, target))
}
pub(super) fn plan_from_decision(d: Decision, target: &Target) -> Plan {
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
    if d.setup_required {
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
    if d.reinstall {
        summary.push("Repair / Reinstall explicitly runs complete signed guest driver preparation, even when historical host receipts match. Files independently confirmed to match may be retained. Complete preparation and selected-hardware graphics verification are required; successful completion leaves the VM off. This is a one-operation request, not an automatic repair policy.".into());
    }
    if d.initial_setup && !d.reinstall {
        summary.push("Initial GPU setup includes preparation as needed and checked hardware rendering. Successful completion requires confirmed graceful shutdown; leave the VM off for the user to start in Hyper-V.".into());
    }
    if d.setup_required && !d.initial_setup && !d.reinstall {
        summary.push("GPU setup/update required: completed preparation/graphics evidence is missing or invalid, or differs from the current signed driver. Review the preparation, graphics check and final shutdown before confirming.".into());
    } else if !d.setup_required {
        summary.push("Ordinary GPU change: no guest access, preparation or graphics check. Successful completion leaves the VM off; start it through Hyper-V when ready.".into());
    }
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
    if d.reinstall {
        summary.push("Reinstall the authenticated payload named in this review; historical receipts cannot skip driver transfer.".into());
    } else if d.stale {
        summary.push(if d.journal.prepared.is_some() {
            "Refresh guest preparation: the recorded payload differs from the current signed driver."
        } else { "Prepare the current signed driver: no matching completed preparation is recorded." }.into());
    } else if d.setup_required {
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
        summary.push("Preserve unowned compatibility settings and values changed by another manager; restore only values still matching the product's last write.".into());
    }
    if !target.enabled {
        summary.push("Remove host GPU support for this exact managed pair, even if guest drivers or the adapter are absent. Keep guest driver files, product authorization and historical records. No guest credentials, preparation, boot or graphics check is required. Publish disabled intent only after independent host and final-off readback.".into());
    }
    if d.allocation_change {
        for resource in Resource::ALL {
            if let Some(value) = target.allocation(resource) {
                summary.push(format!(
                    "{} write (Minimum / Optimal / Maximum): {} / {} / {}; raw provider units, no enforcement claim.",
                    resource.name(), value.minimum, value.optimal, value.maximum
                ));
            }
        }
    }
    if target.enabled
        && Resource::ALL
            .into_iter()
            .any(|r| target.allocation(r).is_none())
    {
        summary.push("Omitted categories request no allocation write. A new or recreated attachment uses provider defaults; an attachment retained in place keeps its existing values.".into());
    }
    if d.setup_required {
        summary.push("Guest administrator credentials are required (stored vault entry or local prompt). Check PnP health and hardware D3D11 rendering.".into());
    } else {
        summary.push(
            "Keep prepared driver files; no guest credentials or graphics check required.".into(),
        );
    }
    summary.push(format!("Final guest power: {:?}. Apply rechecks identity and state; a preview does not reserve them.", d.restore_power));
    Plan {
        authentication_deferred: d.authentication_deferred,
        reinstall: d.reinstall,
        initial_setup: d.initial_setup,
        desired: target.clone(),
        observed: d.initial,
        managed: d.managed,
        preparation: d.digest.map(|digest| PreparationPreview {
            current_driver: d.gpu.driver_version,
            stale: d.stale,
            digest,
        }),
        preview: EffectPreview {
            compute: d
                .allocation_change
                .then(|| target.compute.clone())
                .flatten(),
            encode: d.allocation_change.then(|| target.encode.clone()).flatten(),
            decode: d.allocation_change.then(|| target.decode.clone()).flatten(),
            actions,
            settings,
            settings_preserved: d.settings_preserved,
            allocation: d.allocation_change.then(|| target.vram.clone()).flatten(),
            credentials_required: d.setup_required,
            guest_downtime: downtime,
            restore_power: d.restore_power,
            pending_recovery: pending,
            summary,
        },
        allocation_units: "VRAM bytes; Compute/Encode/Decode provider-defined".into(),
        automatic_guest_restart: restart,
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Allocation, Power};
    use crate::workflow::test_support::{extended_target, fake, target, trace};
    use crate::workflow::{Plan, PreviewAction, admit_approved, apply, plan};

    #[test]
    fn preliminary_ordinary_review_defers_payload_but_admission_authenticates_before_effects() {
        let mut f = fake();
        let mut t = target();
        apply(&mut f, &t).unwrap();
        t.vram = Some(Allocation {
            minimum: 0,
            optimal: 25,
            maximum: 100,
        });
        f.events.clear();
        let reads = f.payload_reads;
        let writes = f.journal_writes;
        let approved = super::plan_preliminary(&mut f, &t).unwrap();
        assert!(approved.authentication_deferred);
        assert!(!approved.preview.credentials_required);
        assert_eq!(f.payload_reads, reads);
        assert_eq!(f.journal_writes, writes);
        let (current, admitted) = admit_approved(&mut f, &approved).unwrap();
        assert!(!current.authentication_deferred);
        assert_eq!(f.payload_reads, reads + 1);
        assert_eq!(f.journal_writes, writes);
        assert!(f.events.is_empty());
        f.fail_prepare = true;
        f.fail_verify = true;
        let result = super::super::apply_admitted(&mut f, admitted).unwrap();
        assert_eq!(result.effective.vram, t.vram);
        assert_eq!(result.effective.power, Power::Off);
        assert!(!result.prepared && !result.verified);
    }

    #[test]
    fn preliminary_review_drift_and_authentication_failure_never_expand_effects() {
        for scenario in 0..5 {
            let mut f = fake();
            let t = target();
            apply(&mut f, &t).unwrap();
            let approved = super::plan_preliminary(&mut f, &t).unwrap();
            match scenario {
                0 => f.driver = "changed-payload",
                1 => f.gpu_driver = "changed-driver",
                2 => f.state.settings.high_mmio += 1,
                3 => {
                    f.journal.as_mut().unwrap().latest_check =
                        Some(super::super::GuestCheck::Failed)
                }
                _ => f.fail_payload = true,
            }
            let writes = f.journal_writes;
            f.events.clear();
            assert!(admit_approved(&mut f, &approved).is_err());
            assert_eq!(f.journal_writes, writes);
            assert!(f.events.is_empty());
        }
    }

    #[test]
    fn preliminary_review_authenticates_when_setup_history_is_missing_or_adverse() {
        for adverse in [false, true] {
            let mut f = fake();
            let t = target();
            if adverse {
                apply(&mut f, &t).unwrap();
                f.journal.as_mut().unwrap().latest_check = Some(super::super::GuestCheck::Failed);
            }
            let reads = f.payload_reads;
            let review = super::plan_preliminary(&mut f, &t).unwrap();
            assert!(!review.authentication_deferred);
            assert!(review.preview.credentials_required);
            assert_eq!(f.payload_reads, reads + 1);
        }
    }

    #[test]
    fn unknown_capability_and_capability_drift_block_before_effects() {
        let target = extended_target();
        let mut f = fake();
        let approved = plan(&mut f, &target).unwrap();
        f.resource_capability = crate::model::ResourceCapability::Unknown;
        assert!(admit_approved(&mut f, &approved).is_err());
        assert!(plan(&mut f, &target).is_err());
        assert!(f.events.is_empty());
        assert_eq!(f.journal_writes, 0);
        assert_eq!(f.allocation_writes, 0);
        let mut omitted = target.clone();
        omitted.clear_allocations();
        assert!(plan(&mut f, &omitted).is_ok());
        let mut disabled = target;
        disabled.enabled = false;
        assert!(plan(&mut f, &disabled).is_ok());
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
    fn preview_explains_raw_allocation_and_disable_without_payload_discovery() {
        let mut f = fake();
        let mut t = target();
        apply(&mut f, &t).unwrap();
        f.state.power = Power::Running;
        t.vram = Some(Allocation {
            minimum: 10,
            maximum: 90,
            optimal: 50,
        });
        let p = plan(&mut f, &t).unwrap();
        assert!(p.preview.actions.contains(&PreviewAction::Allocation));
        assert_eq!(p.preview.allocation, t.vram);
        assert!(!p.automatic_guest_restart);
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
                        // Ordering remains invalid independently of provider metadata.
                        maximum: 49,
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
}
