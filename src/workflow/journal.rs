//! Durable per-VM state and independent check persistence.
use super::Backend;
use crate::model::{Power, Settings, Target};
use serde::{Deserialize, Serialize};

/// Durable, per-VM ownership and interrupted-operation state.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
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
    /// Latest admitted guest check, separate from historical successful rendering.
    #[serde(default)]
    pub latest_check: Option<GuestCheck>,
    /// Fresh admission proof: false until preparation-related effects may start.
    #[serde(default)]
    pub preparation_started: Option<bool>,
    /// A terminal failed Verify independently read back its original host lifecycle.
    #[serde(default)]
    pub check_restored: bool,
    /// A completed removal releases compatibility ownership without deleting history.
    #[serde(default)]
    pub settings_released: bool,
}

/// Durable check facts; no adapter messages or credentials are persisted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GuestCheck {
    /// Admitted check has no durable terminal result; interruption is possible.
    Interrupted,
    /// Current payload and graphics check passed; pending lifecycle still takes precedence.
    Passed,
    /// Current files or receipt differ from the authenticated expected payload.
    PayloadMismatch,
    /// Check failed without establishing a payload mismatch.
    Failed,
    /// Current guest facts could not be obtained.
    Unavailable,
}

impl Journal {
    /// Historical receipts are usable for ordinary changes only without a newer adverse check.
    pub fn setup_eligible(&self) -> bool {
        self.prepared.is_some()
            && self.last_verified.is_some()
            && matches!(self.latest_check, None | Some(GuestCheck::Passed))
    }
}

pub(super) fn persist_check(
    backend: &mut impl Backend,
    target: &Target,
    journal: &Journal,
) -> Result<(), String> {
    backend.save(target, journal)?;
    require_journal(backend, target, journal)
}

// A local check result grants no authority to overwrite changed durable history.
// Use this before delayed power changes and before advancing completion evidence.
pub(super) fn require_journal(
    backend: &mut impl Backend,
    target: &Target,
    journal: &Journal,
) -> Result<(), String> {
    let observed = backend.journal(target)?;
    if observed.as_ref() != Some(journal) {
        return Err("check outcome journal readback mismatch; retain recovery".into());
    }
    Ok(())
}

pub(super) fn verified_at() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::test_support::{fake, target};
    use crate::workflow::{
        apply, apply_admitted, prepare_apply, prepare_verification, verify_admitted,
    };

    #[test]
    fn every_journal_field_remains_bound_before_prepared_apply_or_verify_effects() {
        let changes: [fn(&mut Journal); 14] = [
            |j| j.schema += 1,
            |j| j.vm_id = "12345678-1234-1234-1234-000000000002".into(),
            |j| j.gpu_interface.push('x'),
            |j| j.original.high_mmio += 1,
            |j| j.applied.low_mmio += 1,
            |j| j.prepared = Some("changed digest".into()),
            |j| j.pending = !j.pending,
            |j| j.restore_power = Power::Running,
            |j| j.verification_only = !j.verification_only,
            |j| j.last_verified = Some(j.last_verified.unwrap_or_default() + 1),
            |j| j.latest_check = Some(GuestCheck::Failed),
            |j| j.preparation_started = Some(j.preparation_started != Some(true)),
            |j| j.check_restored = !j.check_restored,
            |j| j.settings_released = !j.settings_released,
        ];
        for (field, change) in changes.into_iter().enumerate() {
            for verify in [false, true] {
                let mut backend = fake();
                let target = target();
                apply(&mut backend, &target).unwrap();
                let (apply_token, verify_token) = if verify {
                    (
                        None,
                        Some(prepare_verification(&mut backend, &target).unwrap().1),
                    )
                } else {
                    (
                        Some(prepare_apply(&mut backend, &target, false).unwrap().1),
                        None,
                    )
                };
                backend.events.clear();
                let writes = backend.journal_writes;
                change(backend.journal.as_mut().unwrap());
                let changed = backend.journal.clone();
                let result = if let Some(token) = apply_token {
                    apply_admitted(&mut backend, token)
                } else {
                    verify_admitted(&mut backend, verify_token.unwrap())
                };
                assert!(result.is_err(), "journal field {field}, verify {verify}");
                assert!(
                    backend.events.is_empty(),
                    "must reject before host/guest effects"
                );
                assert_eq!(backend.journal_writes, writes);
                assert_eq!(
                    backend.journal, changed,
                    "must not overwrite changed history"
                );
            }
        }
    }

    #[test]
    fn journal_check_field_is_backward_readable_and_rejects_unknown_outcomes() {
        let mut backend = fake();
        apply(&mut backend, &target()).unwrap();
        let mut value = serde_json::to_value(backend.journal.as_ref().unwrap()).unwrap();
        value.as_object_mut().unwrap().remove("latest_check");
        let old: Journal = serde_json::from_value(value.clone()).unwrap();
        assert!(old.latest_check.is_none() && old.setup_eligible());
        value["latest_check"] = serde_json::json!("invented_success");
        assert!(serde_json::from_value::<Journal>(value).is_err());
    }
}
