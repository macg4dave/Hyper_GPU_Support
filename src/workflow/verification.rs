//! Explicit guest verification with reviewed admission and lifecycle restoration.
use super::guest_check::{check_guest, record_restored_check_failure, restore_guest_power};
use super::journal::{persist_check, verified_at};
use super::{Backend, GuestCheck, Journal, OperationResult};
use crate::model::{Gpu, Power, Target, VmState};
use serde::{Deserialize, Serialize};

/// Verify an enrolled assignment with durable lifecycle intent and graceful restoration.
/// Validate verification recovery and report the final power intent used by all clients.
pub fn verification_power(
    target: &Target,
    before: &VmState,
    old: Option<&Journal>,
) -> Result<Power, String> {
    target.validate()?;
    if before.vm_id == target.vm_id && before.gpus.is_empty() {
        return Err("Host configuration differs: the selected GPU is not attached. Review GPU support changes or removal before Verify; no guest verification ran.".into());
    }
    if before.vm_id != target.vm_id
        || before.generation != 2
        || before.gpus != vec![target.gpu_interface.clone()]
        || !matches!(before.power, Power::Running | Power::Off)
    {
        return Err("verification requires the selected, stable GPU assignment".into());
    }
    if old.is_some_and(|j| {
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
    let power = old
        .filter(|j| j.pending)
        .map_or(&before.power, |j| &j.restore_power);
    if !matches!(power, Power::Off | Power::Running) {
        return Err("verification recovery has an invalid power intent".into());
    }
    Ok(power.clone())
}
/// Check current payload and graphics, restoring the validated recovery power intent.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationPlan {
    /// Exact selected enrolled pair.
    pub target: Target,
    /// Fresh state; an off guest is temporarily started and then stopped.
    pub observed: VmState,
    /// Current device and driver identity.
    pub gpu: Gpu,
    /// Exact management state; pending operations prohibit fresh verification.
    pub managed: Option<Journal>,
    /// Current authenticated host manifest, bound into the explicit review.
    pub payload_digest: String,
}
/// Preview verification without guest, journal or power effects.
pub fn plan_verification(
    backend: &mut impl Backend,
    target: &Target,
) -> Result<VerificationPlan, String> {
    verification_plan(backend, target, false)
}

fn verification_plan(
    backend: &mut impl Backend,
    target: &Target,
    resume: bool,
) -> Result<VerificationPlan, String> {
    let observed = backend.inspect(target)?;
    let managed = backend.journal(target)?;
    if !resume && managed.as_ref().is_some_and(|j| j.pending) {
        return Err("unfinished operation requires manual reconciliation".into());
    }
    verification_power(target, &observed, managed.as_ref())?;
    let gpu = backend.gpu(target)?;
    if gpu.interface != target.gpu_interface {
        return Err("selected GPU identity changed".into());
    }
    let payload_digest = backend.payload(target)?;
    Ok(VerificationPlan {
        target: target.clone(),
        observed,
        gpu,
        managed,
        payload_digest,
    })
}
/// Independently check reviewed verification scope immediately before effects.
pub fn verify_approved(
    backend: &mut impl Backend,
    approved: &VerificationPlan,
) -> Result<OperationResult, String> {
    let admitted = admit_verification(backend, approved)?;
    verify_admitted(backend, admitted)
}

/// In-process verification admission, retained only in the locked backend that
/// authenticated its payload. Frontends cannot construct or deserialize it.
pub(crate) struct AdmittedVerification {
    plan: VerificationPlan,
}

/// Authenticate the reviewed manifest once before recording the operation hold.
pub(crate) fn admit_verification(
    backend: &mut impl Backend,
    approved: &VerificationPlan,
) -> Result<super::AdmittedVerification, String> {
    let current = plan_verification(backend, &approved.target)?;
    if serde_json::to_value(&current).map_err(|e| e.to_string())?
        != serde_json::to_value(approved).map_err(|e| e.to_string())?
    {
        return Err("verification observations changed; Refresh and review".into());
    }
    Ok(AdmittedVerification { plan: current })
}

/// Consume admission in the same backend; mutable host facts are checked again
/// before writes, without repeating manifest discovery and authentication.
pub(crate) fn verify_admitted(
    backend: &mut impl Backend,
    admitted: super::AdmittedVerification,
) -> Result<OperationResult, String> {
    verify_current_payload(backend, &admitted.plan)
}
/// Check current payload and graphics, restoring the validated recovery power intent.
pub fn verify(backend: &mut impl Backend, target: &Target) -> Result<OperationResult, String> {
    let current = verification_plan(backend, target, true)?;
    verify_current_payload(backend, &current)
}

fn verify_current_payload(
    backend: &mut impl Backend,
    approved: &VerificationPlan,
) -> Result<OperationResult, String> {
    let target = &approved.target;
    target.validate()?;
    let before = backend.inspect(target)?;
    let old = backend.journal(target)?;
    let restore_power = verification_power(target, &before, old.as_ref())?;
    let gpu = backend.gpu(target)?;
    if gpu.interface != target.gpu_interface {
        return Err("selected GPU identity changed".into());
    }
    if serde_json::to_value(&before).map_err(|e| e.to_string())?
        != serde_json::to_value(&approved.observed).map_err(|e| e.to_string())?
        || serde_json::to_value(&gpu).map_err(|e| e.to_string())?
            != serde_json::to_value(&approved.gpu).map_err(|e| e.to_string())?
        || serde_json::to_value(&old).map_err(|e| e.to_string())?
            != serde_json::to_value(&approved.managed).map_err(|e| e.to_string())?
    {
        return Err("verification observations changed; Refresh and review".into());
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
        latest_check: None,
        preparation_started: None,
        check_restored: false,
        // A read-only guest check cannot acquire compatibility-setting ownership.
        settings_released: true,
    });
    journal.restore_power = restore_power;
    journal.pending = true;
    journal.verification_only = true;
    journal.latest_check = Some(GuestCheck::Interrupted);
    journal.check_restored = false;
    journal.preparation_started = None;
    persist_check(backend, target, &journal)?;
    let checked = (|| {
        if before.power == Power::Off {
            backend.power(target, Power::Running)?;
        }
        check_guest(backend, target, &gpu, &mut journal)
    })();
    // A start error may follow a real state change. Inspect and restore even then,
    // retaining both failures and the durable intent if recovery cannot complete.
    let restoration = (|| {
        let current = backend.inspect(target)?;
        let mut expected = before.clone();
        expected.power = current.power.clone();
        if !matches!(current.power, Power::Off | Power::Running)
            || serde_json::to_value(&current).map_err(|e| e.to_string())?
                != serde_json::to_value(&expected).map_err(|e| e.to_string())?
            || serde_json::to_value(backend.gpu(target)?).map_err(|e| e.to_string())?
                != serde_json::to_value(&gpu).map_err(|e| e.to_string())?
        {
            return Err("verification facts changed; unsafe power restoration blocked".into());
        }
        // A running guest was never reviewed for a new start. Honor an external stop.
        if journal.restore_power == Power::Running && current.power != Power::Running {
            return Err("verification cannot authorize restarting the stopped guest".into());
        }
        restore_guest_power(backend, target, &journal.restore_power)?;
        expected.power = journal.restore_power.clone();
        if serde_json::to_value(backend.inspect(target)?).map_err(|e| e.to_string())?
            != serde_json::to_value(&expected).map_err(|e| e.to_string())?
            || serde_json::to_value(backend.gpu(target)?).map_err(|e| e.to_string())?
                != serde_json::to_value(&gpu).map_err(|e| e.to_string())?
        {
            return Err("verification restoration facts changed; retain recovery".into());
        }
        Ok(())
    })();
    record_restored_check_failure(backend, target, &mut journal, &checked, &restoration)?;
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

#[cfg(test)]
mod tests {
    use super::verify_current_payload;
    use crate::model::Power;
    use crate::workflow::test_support::{fake, target};
    use crate::workflow::{
        GuestCheck, Journal, admit_verification, apply, plan_verification, verify, verify_admitted,
        verify_approved,
    };

    #[test]
    fn reviewed_verification_rejects_stale_state_before_power_or_journal_effects() {
        let mut backend = fake();
        apply(&mut backend, &target()).unwrap();
        backend.events.clear();
        backend.journal_writes = 0;
        let approved = plan_verification(&mut backend, &target()).unwrap();
        backend.state.power = Power::Running;
        assert!(verify_approved(&mut backend, &approved).is_err());
        assert!(backend.events.is_empty());
        assert_eq!(backend.journal_writes, 0);
        backend.journal.as_mut().unwrap().pending = true;
        assert!(plan_verification(&mut backend, &target()).is_err());
    }

    #[test]
    fn verification_binds_current_payload_and_rejects_same_version_payload_change() {
        let mut backend = fake();
        apply(&mut backend, &target()).unwrap();
        backend.events.clear();
        backend.journal_writes = 0;
        let approved = plan_verification(&mut backend, &target()).unwrap();
        assert_eq!(approved.payload_digest, backend.driver);
        // Driver identity reported by gpu() is unchanged; the manifest digest changed.
        backend.driver = "different-content";
        assert!(
            verify_approved(&mut backend, &approved)
                .unwrap_err()
                .contains("observations changed")
        );
        assert!(backend.events.is_empty());
        assert_eq!(backend.journal_writes, 0);
    }

    #[test]
    fn verify_without_host_attachment_reports_discrepancy_before_guest_or_journal_work() {
        let mut backend = fake();
        backend.state.gpus.clear();
        let error = plan_verification(&mut backend, &target()).unwrap_err();
        assert!(error.contains("no guest verification ran"));
        assert!(verify(&mut backend, &target()).is_err());
        assert_eq!(backend.payload_reads, 0);
        assert_eq!(backend.journal_writes, 0);
        assert!(backend.events.is_empty());
    }

    #[test]
    fn explicit_verify_discovers_current_payload_even_when_historical_receipt_matches() {
        let mut backend = fake();
        apply(&mut backend, &target()).unwrap();
        backend.events.clear();
        backend.payload_reads = 0;
        verify(&mut backend, &target()).unwrap();
        assert_eq!(backend.payload_reads, 1);
        assert_eq!(backend.events, ["start", "graphics", "shutdown"]);
        assert!(!backend.events.contains(&"prepare"));
    }

    #[test]
    fn verification_rechecks_admitted_host_gpu_and_journal_before_writes() {
        for changed in 0..3 {
            let mut backend = fake();
            apply(&mut backend, &target()).unwrap();
            let approved = plan_verification(&mut backend, &target()).unwrap();
            backend.events.clear();
            backend.journal_writes = 0;
            match changed {
                0 => backend.state.power = Power::Running,
                1 => backend.gpu_driver = "changed-driver",
                _ => backend.journal.as_mut().unwrap().prepared = None,
            }
            assert!(
                verify_current_payload(&mut backend, &approved)
                    .unwrap_err()
                    .contains("observations changed")
            );
            assert_eq!(backend.journal_writes, 0);
            assert!(backend.events.is_empty());
        }
    }

    #[test]
    fn admitted_verification_authenticates_payload_once_before_execution() {
        let mut backend = fake();
        apply(&mut backend, &target()).unwrap();
        let approved = plan_verification(&mut backend, &target()).unwrap();
        backend.payload_reads = 0;
        let admitted = admit_verification(&mut backend, &approved).unwrap();
        assert_eq!(backend.payload_reads, 1);
        verify_admitted(&mut backend, admitted).unwrap();
        assert_eq!(backend.payload_reads, 1);
    }

    #[test]
    fn payload_mismatch_or_unavailable_verify_preserves_recovery_and_restores_only_reviewed_power()
    {
        use crate::reporting::{ComponentFailure, FailureCause};
        for power in [Power::Off, Power::Running] {
            for cause in [
                FailureCause::FileMissing,
                FailureCause::FileChanged,
                FailureCause::ReadbackMismatch,
                FailureCause::AccessDenied,
                FailureCause::TimedOut,
                FailureCause::Unavailable,
            ] {
                let mut backend = fake();
                apply(&mut backend, &target()).unwrap();
                backend.state.power = power.clone();
                let history = backend.journal.as_ref().unwrap().last_verified;
                backend.events.clear();
                backend.payload_check_failure = Some(cause);
                let error = verify(&mut backend, &target()).unwrap_err();
                let failure = ComponentFailure::decode(&error).unwrap();
                assert_eq!(format!("{:?}", failure.cause), format!("{cause:?}"));
                assert_eq!(backend.state.power, power);
                assert!(backend.journal.as_ref().unwrap().pending);
                assert!(backend.journal.as_ref().unwrap().verification_only);
                assert_eq!(backend.journal.as_ref().unwrap().last_verified, history);
                let journal = backend.journal.as_ref().unwrap();
                let mismatch = matches!(
                    cause,
                    FailureCause::FileMissing
                        | FailureCause::FileChanged
                        | FailureCause::ReadbackMismatch
                );
                assert_eq!(
                    journal.latest_check,
                    Some(if mismatch {
                        GuestCheck::PayloadMismatch
                    } else {
                        GuestCheck::Unavailable
                    })
                );
                assert!(!journal.setup_eligible());
                assert_eq!(journal.prepared.is_none(), mismatch);
                let reopened: Journal =
                    serde_json::from_str(&serde_json::to_string(journal).unwrap()).unwrap();
                assert!(!reopened.setup_eligible());
                if mismatch {
                    let repair = decision(&mut backend, &target()).unwrap();
                    assert!(repair.stale && repair.setup_required);
                }
                assert!(
                    !backend
                        .events
                        .iter()
                        .any(|e| matches!(*e, "graphics" | "prepare" | "attach" | "allocation"))
                );
                assert_eq!(
                    backend.events,
                    if power == Power::Off {
                        vec!["start", "payload-check", "shutdown"]
                    } else {
                        vec!["payload-check"]
                    }
                );
            }
        }
    }

    #[test]
    fn failed_verify_does_not_restart_an_externally_stopped_guest() {
        let mut backend = fake();
        apply(&mut backend, &target()).unwrap();
        backend.state.power = Power::Running;
        backend.events.clear();
        backend.stop_on_verify = true;
        backend.fail_verify = true;
        assert!(
            verify(&mut backend, &target())
                .unwrap_err()
                .contains("cannot authorize restarting")
        );
        assert_eq!(backend.state.power, Power::Off);
        assert!(!backend.events.contains(&"start"));
        let journal = backend.journal.as_ref().unwrap();
        assert!(journal.pending && !journal.check_restored);
    }

    #[test]
    fn failed_off_noop_verification_restores_power_without_repreparation() {
        let mut f = fake();
        f.state.power = Power::Off;
        apply(&mut f, &target()).unwrap();
        f.events.clear();
        f.fail_verify = true;
        assert!(verify(&mut f, &target()).is_err());
        assert_eq!(f.state.power, Power::Off);
        assert_eq!(f.journal.as_ref().unwrap().restore_power, Power::Off);
        assert!(f.journal.as_ref().unwrap().pending);
        f.fail_verify = false;
        f.events.clear();
        verify(&mut f, &target()).unwrap();
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
    fn failed_running_standalone_verify_is_pending_and_retry_does_not_restart_or_prepare() {
        let mut f = fake();
        apply(&mut f, &target()).unwrap();
        f.state.power = Power::Running;
        let previous_verified = f.journal.as_ref().unwrap().last_verified;
        f.events.clear();
        f.fail_verify = true;
        assert!(
            verify(&mut f, &target())
                .unwrap_err()
                .contains("graphics failed")
        );
        let j = f.journal.as_ref().unwrap();
        assert!(j.pending && j.verification_only);
        assert_eq!(j.last_verified, previous_verified);
        assert_eq!(f.events, ["graphics"]);
        f.fail_verify = false;
        f.events.clear();
        verify(&mut f, &target()).unwrap();
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
}
