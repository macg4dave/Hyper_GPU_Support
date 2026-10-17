//! Independent outcome reconciliation without replaying Apply effects.
use super::guest_check::{check_guest, record_restored_check_failure, restore_guest_power};
use super::journal::{persist_check, require_journal, verified_at};
use super::{Backend, GuestCheck, OperationResult};
use crate::model::{Power, Target, VmState};

/// Explicitly reconcile an achieved outcome without GPU, settings or payload
/// mutation. Caller has independently validated the durable approved receipt.
pub fn reconcile_observed(
    backend: &mut impl Backend,
    target: &Target,
    observed: &VmState,
    restore: Power,
    payload: Option<&str>,
) -> Result<OperationResult, String> {
    reconcile_with_graphics(backend, target, observed, restore, payload, target.enabled)
}

/// Reconcile the durable reviewed check requirement without replaying GPU work.
pub(crate) fn reconcile_with_graphics(
    backend: &mut impl Backend,
    target: &Target,
    observed: &VmState,
    restore: Power,
    payload: Option<&str>,
    check_graphics: bool,
) -> Result<OperationResult, String> {
    target.validate()?;
    let mut journal = backend
        .journal(target)?
        .ok_or("no attributable operation journal; retain recovery for administrator inspection")?;
    if journal.vm_id != target.vm_id
        || journal.gpu_interface != target.gpu_interface
        || journal.schema != 1
        || journal.restore_power != restore
        || !matches!(restore, Power::Off | Power::Running)
        // Verify-only recovery proves current guest facts independently; it
        // neither requires nor upgrades historical host preparation authority.
        || (target.enabled
            && !journal.verification_only
            && payload.is_some_and(|p| journal.prepared.as_deref() != Some(p)))
    {
        return Err(
            "operation journal does not prove the reviewed preparation and power intent".into(),
        );
    }
    if backend.inspect(target)?.operational() != observed.operational() {
        return Err("state changed during reconciliation".into());
    }
    if check_graphics {
        let gpu = backend.gpu(target)?;
        journal.latest_check = Some(GuestCheck::Interrupted);
        journal.check_restored = false;
        persist_check(backend, target, &journal)?;
        // A failed start can follow a real power change. Keep it inside the
        // checked stage so fresh fact checks and safe restoration still run.
        let checked = (|| {
            if observed.power == Power::Off {
                backend.power(target, Power::Running)?;
            }
            check_guest(backend, target, &gpu, &mut journal)
        })();
        let restoration = (|| {
            require_journal(backend, target, &journal)?;
            let current = backend.inspect(target)?;
            let mut expected = observed.clone();
            expected.power = current.power.clone();
            if !matches!(current.power, Power::Off | Power::Running)
                || current.operational() != expected.operational()
                || backend.gpu(target)?.operational() != gpu.operational()
                || payload.is_some_and(|digest| backend.payload(target).as_deref() != Ok(digest))
            {
                return Err(
                    "reconciliation facts changed; unsafe power restoration blocked".into(),
                );
            }
            if restore == Power::Running && current.power != Power::Running {
                return Err("verification cannot authorize restarting the stopped guest".into());
            }
            restore_guest_power(backend, target, &restore)
        })();
        // Record restoration only after independent full host readback, including
        // any final power transition; a partial restoration cannot enable repair.
        let restoration = restoration.and_then(|()| {
            require_journal(backend, target, &journal)?;
            let mut expected = observed.clone();
            expected.power = restore.clone();
            if backend.inspect(target)?.operational() != expected.operational()
                || backend.gpu(target)?.operational() != gpu.operational()
            {
                return Err("reconciliation restoration facts changed; retain recovery".into());
            }
            Ok(())
        });
        if journal.verification_only {
            record_restored_check_failure(backend, target, &mut journal, &checked, &restoration)?;
        }
        match (checked, restoration) {
            (Err(primary), Err(restoration)) => {
                return Err(format!(
                    "verification failed: {primary}; safe power restoration blocked or failed: {restoration}"
                ));
            }
            (Err(primary), Ok(())) => return Err(primary),
            (Ok(()), Err(restoration)) => return Err(restoration),
            (Ok(()), Ok(())) => {}
        }
    } else {
        // Ordinary reconciliation requires the achieved final power; it never boots.
        if backend.inspect(target)?.power != restore {
            return Err("outcome has unresolved guest power; retain recovery".into());
        }
    }
    let effective = backend.inspect(target)?;
    let mut expected = observed.clone();
    expected.power = restore;
    if effective.operational() != expected.operational() {
        return Err("reconciliation independent readback changed".into());
    }
    require_journal(backend, target, &journal)?;
    journal.pending = false;
    if check_graphics {
        journal.last_verified = Some(verified_at());
    }
    if !target.enabled {
        journal.settings_released = true;
    } else if !journal.verification_only {
        // Management reconciliation proves the bound host write; Verify cannot acquire ownership.
        journal.applied = effective.settings.clone();
        journal.settings_released = false;
    }
    persist_check(backend, target, &journal)?;
    Ok(OperationResult {
        effective,
        prepared: false,
        verified: check_graphics,
        settings_preserved: false,
    })
}

#[cfg(test)]
mod tests {
    use crate::model::Power;
    use crate::workflow::test_support::{fake, target};
    use crate::workflow::{apply, reconcile_observed, reconcile_with_graphics, verify};

    #[test]
    fn successful_reconciliation_check_cannot_restart_an_externally_stopped_guest() {
        let mut backend = fake();
        apply(&mut backend, &target()).unwrap();
        backend.state.power = Power::Running;
        let journal = backend.journal.as_mut().unwrap();
        journal.pending = true;
        journal.verification_only = true;
        journal.restore_power = Power::Running;
        backend.stop_on_verify = true;
        backend.events.clear();
        let observed = backend.state.clone();
        assert!(
            reconcile_with_graphics(
                &mut backend,
                &target(),
                &observed,
                Power::Running,
                Some("current"),
                true
            )
            .unwrap_err()
            .contains("restarting")
        );
        assert_eq!(backend.state.power, Power::Off);
        assert_eq!(backend.events, ["graphics"]);
        assert!(backend.journal.as_ref().unwrap().pending);
        assert!(!backend.journal.as_ref().unwrap().check_restored);
    }

    #[test]
    fn verify_only_reconciliation_checks_current_payload_without_certifying_old_preparation() {
        let mut backend = fake();
        apply(&mut backend, &target()).unwrap();
        backend.journal.as_mut().unwrap().prepared = Some("historical-payload".into());
        backend.fail_verify = true;
        assert!(verify(&mut backend, &target()).is_err());
        assert!(backend.journal.as_ref().unwrap().verification_only);
        backend.fail_verify = false;
        backend.events.clear();
        let observed = backend.state.clone();
        let result = reconcile_with_graphics(
            &mut backend,
            &target(),
            &observed,
            Power::Off,
            Some("current"),
            true,
        )
        .unwrap();
        assert!(result.verified);
        assert_eq!(backend.events, ["start", "graphics", "shutdown"]);
        assert_eq!(
            backend.journal.as_ref().unwrap().prepared.as_deref(),
            Some("historical-payload")
        );
        // Management/setup recovery still needs a matching preparation receipt.
        backend.journal.as_mut().unwrap().verification_only = false;
        backend.events.clear();
        assert!(
            reconcile_with_graphics(
                &mut backend,
                &target(),
                &observed,
                Power::Off,
                Some("current"),
                true
            )
            .is_err()
        );
        assert!(backend.events.is_empty());
    }

    #[test]
    fn explicit_reconciliation_never_reassigns_reprepares_or_reallocates() {
        let mut backend = fake();
        apply(&mut backend, &target()).unwrap();
        backend.state.power = Power::Running;
        backend.journal.as_mut().unwrap().restore_power = Power::Running;
        backend.journal.as_mut().unwrap().pending = true;
        backend.events.clear();
        backend.allocation_writes = 0;
        let observed = backend.state.clone();
        let result = reconcile_observed(
            &mut backend,
            &target(),
            &observed,
            Power::Running,
            Some("current"),
        )
        .unwrap();
        assert!(result.verified);
        assert_eq!(backend.events, ["graphics"]);
        assert_eq!(backend.allocation_writes, 0);
        assert!(!backend.journal.as_ref().unwrap().pending);
        backend.journal.as_mut().unwrap().pending = true;
        backend.fail_verify = true;
        assert!(
            reconcile_observed(
                &mut backend,
                &target(),
                &observed,
                Power::Running,
                Some("current")
            )
            .is_err()
        );
        assert!(backend.journal.as_ref().unwrap().pending);
        backend.events.clear();
        backend.state.settings.high_mmio += 1;
        assert!(
            reconcile_observed(
                &mut backend,
                &target(),
                &observed,
                Power::Running,
                Some("current")
            )
            .is_err()
        );
        assert!(backend.events.is_empty());
    }

    #[test]
    fn ordinary_reconciliation_checks_readback_without_boot_or_graphics() {
        let mut f = fake();
        apply(&mut f, &target()).unwrap();
        let previous = f.journal.as_ref().unwrap().last_verified;
        f.journal.as_mut().unwrap().pending = true;
        f.events.clear();
        f.fail_verify = true;
        let observed = f.state.clone();
        let result = reconcile_with_graphics(
            &mut f,
            &target(),
            &observed,
            Power::Off,
            Some("current"),
            false,
        )
        .unwrap();
        assert!(!result.verified);
        assert!(f.events.is_empty());
        assert_eq!(f.journal.as_ref().unwrap().last_verified, previous);
        f.journal.as_mut().unwrap().pending = true;
        f.state.power = Power::Running;
        let observed = f.state.clone();
        assert!(
            reconcile_with_graphics(
                &mut f,
                &target(),
                &observed,
                Power::Off,
                Some("current"),
                false,
            )
            .is_err()
        );
        assert!(f.events.is_empty());
        assert!(f.journal.as_ref().unwrap().pending);
    }

    #[test]
    fn reconciliation_start_failure_restores_off_and_retains_pending_without_replay() {
        for refuse_shutdown in [false, true] {
            let mut backend = fake();
            apply(&mut backend, &target()).unwrap();
            backend.journal.as_mut().unwrap().pending = true;
            backend.events.clear();
            backend.allocation_writes = 0;
            backend.fail_start_after_effect = true;
            backend.fail_shutdown = refuse_shutdown;
            let observed = backend.state.clone();
            let error = reconcile_observed(
                &mut backend,
                &target(),
                &observed,
                Power::Off,
                Some("current"),
            )
            .unwrap_err();
            assert!(error.contains("start readback interrupted"));
            assert_eq!(backend.events, ["start", "shutdown"]);
            assert_eq!(backend.allocation_writes, 0);
            assert_eq!(
                backend.state.power,
                if refuse_shutdown {
                    Power::Running
                } else {
                    Power::Off
                }
            );
            assert!(backend.journal.as_ref().unwrap().pending);
            if refuse_shutdown {
                assert!(error.contains("shutdown refused"));
            }
        }
    }

    #[test]
    fn reconciliation_start_failure_with_changed_facts_blocks_restoration() {
        let mut backend = fake();
        apply(&mut backend, &target()).unwrap();
        backend.journal.as_mut().unwrap().pending = true;
        backend.events.clear();
        backend.fail_start_after_effect = true;
        backend.change_settings_on_start_failure = true;
        let observed = backend.state.clone();
        let error = reconcile_observed(
            &mut backend,
            &target(),
            &observed,
            Power::Off,
            Some("current"),
        )
        .unwrap_err();
        assert!(error.contains("start readback interrupted"));
        assert!(error.contains("unsafe power restoration blocked"));
        assert_eq!(backend.events, ["start"]);
        assert_eq!(backend.state.power, Power::Running);
        assert!(backend.journal.as_ref().unwrap().pending);
    }
}
