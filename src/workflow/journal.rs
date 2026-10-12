//! Durable per-VM state and independent check persistence.
use super::Backend;
use crate::model::{Power, Settings, Target};
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
    let observed = backend.journal(target)?;
    if serde_json::to_value(observed).map_err(|e| e.to_string())?
        != serde_json::to_value(Some(journal)).map_err(|e| e.to_string())?
    {
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
    use crate::workflow::test_support::{fake, target};
    use crate::workflow::{Journal, apply};

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
