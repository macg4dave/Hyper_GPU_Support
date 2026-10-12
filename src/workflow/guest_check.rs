//! Shared guest-check evidence and graceful power restoration primitives.
use super::journal::persist_check;
use super::{Backend, GuestCheck, Journal};
use crate::model::{Gpu, Power, Target};

pub(super) fn check_guest(
    backend: &mut impl Backend,
    target: &Target,
    gpu: &Gpu,
    journal: &mut Journal,
) -> Result<(), String> {
    use crate::reporting::{Component, ComponentFailure, FailureCause};
    journal.latest_check = Some(GuestCheck::Interrupted);
    journal.check_restored = false;
    persist_check(backend, target, journal)?;
    let expected_journal = serde_json::to_value(&*journal).map_err(|e| e.to_string())?;
    let result = backend.verify(target, gpu);
    let unchanged = backend.journal(target).and_then(|current| {
        if serde_json::to_value(current).map_err(|e| e.to_string())? != expected_journal {
            return Err("check journal changed during guest verification; retain recovery".into());
        }
        Ok(())
    });
    if let Err(observation) = unchanged {
        return Err(match result {
            Err(primary) => format!("{observation}; original guest failure: {primary}"),
            Ok(()) => observation,
        });
    }
    journal.latest_check = Some(match &result {
        Ok(()) => GuestCheck::Passed,
        Err(error) => match ComponentFailure::decode(error) {
            Some(ComponentFailure {
                component: Component::GuestPayload,
                cause: FailureCause::FileMissing | FailureCause::FileChanged,
                ..
            })
            | Some(ComponentFailure {
                component: Component::GuestReceipt,
                cause: FailureCause::ReadbackMismatch,
                ..
            }) => {
                // The same host digest must not skip preparation after proven guest mismatch.
                journal.prepared = None;
                GuestCheck::PayloadMismatch
            }
            Some(ComponentFailure {
                cause:
                    FailureCause::AccessDenied
                    | FailureCause::TimedOut
                    | FailureCause::Unavailable
                    | FailureCause::InvalidReply,
                ..
            }) => GuestCheck::Unavailable,
            _ => GuestCheck::Failed,
        },
    });
    if let Err(persistence) = persist_check(backend, target, journal) {
        return Err(match result {
            Err(primary) => format!("{primary}; check outcome persistence failed: {persistence}"),
            Ok(()) => format!("check outcome persistence failed: {persistence}"),
        });
    }
    result
}

// Both initial Verify and explicit recovery must preserve the same durable
// negative outcome. This proof permits reviewed retirement, never clears a hold.
pub(super) fn record_restored_check_failure(
    backend: &mut impl Backend,
    target: &Target,
    journal: &mut Journal,
    checked: &Result<(), String>,
    restoration: &Result<(), String>,
) -> Result<(), String> {
    let Some(primary) = checked.as_ref().err() else {
        return Ok(());
    };
    let terminal_guest_failure =
        crate::reporting::ComponentFailure::decode(primary).is_some_and(|failure| {
            matches!(
                failure.component,
                crate::reporting::Component::GuestPayload
                    | crate::reporting::Component::GuestReceipt
                    | crate::reporting::Component::GuestGraphics
            )
        });
    if terminal_guest_failure
        && restoration.is_ok()
        && matches!(
            journal.latest_check,
            Some(GuestCheck::Failed | GuestCheck::Unavailable | GuestCheck::PayloadMismatch)
        )
    {
        journal.check_restored = true;
        if let Err(persistence) = persist_check(backend, target, journal) {
            // An unconfirmed proof must not hide the failure that led to it or
            // authorize retirement in this invocation. Keep the typed primary
            // envelope last so existing secret-free reporting can still decode it.
            journal.check_restored = false;
            return Err(format!(
                "restoration-proof persistence failed: {persistence}; original guest failure: {}",
                primary
            ));
        }
    }
    Ok(())
}
pub(super) fn restore_guest_power(
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
