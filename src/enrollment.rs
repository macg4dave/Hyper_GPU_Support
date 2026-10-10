//! Reviewed pair authority, separate from desired configuration and GPU effects.
use crate::{configuration_store::Revision, model::Target};
use serde::{Deserialize, Serialize};

/// Exact installed-policy revision and operator reviewed before elevation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairReview {
    /// Existing Hyper-V VM identity.
    pub vm_id: String,
    /// Exact partitionable GPU interface.
    pub gpu_interface: String,
    /// Protected policy revision; a draft cannot create this authority.
    pub expected: Revision,
    /// Authenticated initiating operator, preserved through elevation.
    pub operator_sid: String,
}

impl PairReview {
    /// Enrollment carries identity only, never requested effects or allocation.
    pub fn target(&self) -> Result<Target, String> {
        let target = Target {
            vm_id: self.vm_id.clone(),
            gpu_interface: self.gpu_interface.clone(),
            enabled: false,
            vram: None,
        };
        target.validate()?;
        if target.vm_id != target.vm_id.to_ascii_lowercase() {
            return Err("enrollment requires a canonical VM GUID".into());
        }
        Ok(target)
    }

    /// Compute an additive policy change without replacing another VM's authority.
    /// GPU replacement has a separate transition/recovery contract.
    #[cfg(any(windows, test))]
    pub(crate) fn append(
        &self,
        targets: &[Target],
        current: &Revision,
        operator_sid: &str,
    ) -> Result<Vec<Target>, String> {
        if &self.expected != current || matches!(current, Revision::Missing) {
            return Err("Enrollment changed after review; Refresh and review again.".into());
        }
        if self.operator_sid != operator_sid {
            return Err("Enrollment operator changed after review.".into());
        }
        let target = self.target()?;
        if targets
            .iter()
            .any(|old| old.vm_id.eq_ignore_ascii_case(&target.vm_id))
        {
            return Err(
                "VM already enrolled; GPU replacement requires a separate reviewed transition."
                    .into(),
            );
        }
        if targets.len() >= 128 {
            return Err("Enrollment is limited to 128 VMs.".into());
        }
        let mut result = targets.to_vec();
        result.push(target);
        Ok(result)
    }
}

/// Write-ahead authority change. An interrupted change requires explicit readback.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingPair {
    /// Original authenticated worker session and audit identity.
    pub operation_id: String,
    /// Exact reviewed pair and original revision/operator.
    pub review: PairReview,
    /// Complete resulting policy revision, including other pairs/artifact pins.
    pub resulting: Revision,
}

impl PendingPair {
    /// Only complete old/new policies resolve uncertainty, never a third policy.
    #[cfg(any(windows, test))]
    pub(crate) fn check_readback(
        &self,
        operation_id: &str,
        reviewed: &Revision,
        actual: &Revision,
        operator: &str,
    ) -> Result<(), String> {
        if self.operation_id != operation_id
            || reviewed != actual
            || self.review.operator_sid != operator
            || (actual != &self.review.expected && actual != &self.resulting)
        {
            return Err("enrollment readback does not match reviewed recovery; retain hold".into());
        }
        Ok(())
    }
}

/// Publish with a durable hold before authority; leave release to terminal audit.
#[cfg(any(windows, test))]
pub(crate) fn publish_pair(
    expected: &Revision,
    hold: impl FnOnce() -> Result<(), String>,
    publish: impl FnOnce() -> Result<(), String>,
    readback: impl FnOnce() -> Result<Revision, String>,
) -> Result<(), String> {
    hold()?;
    publish()?;
    if &readback()? != expected {
        return Err("enrollment publication readback mismatch; retain recovery".into());
    }
    Ok(())
}

/// Original first-install scope retained until task/artifact/audit completion.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingSetup {
    /// Original setup audit session.
    pub operation_id: String,
    /// Exact initial identity; carries no desired GPU changes.
    pub review: PairReview,
    /// Fixed package artifacts reviewed before installation.
    pub artifacts: std::collections::BTreeMap<String, String>,
}

impl PendingSetup {
    /// An interrupted installer cannot acquire another pair, operator or package.
    #[cfg(any(windows, test))]
    pub(crate) fn check_scope(
        &self,
        review: &PairReview,
        artifacts: &std::collections::BTreeMap<String, String>,
        operator: &str,
    ) -> Result<(), String> {
        if &self.review != review
            || &self.artifacts != artifacts
            || self.review.operator_sid != operator
        {
            return Err(
                "interrupted setup scope/operator/artifacts changed; retain recovery".into(),
            );
        }
        Ok(())
    }

    /// Recovery completes task/audit registration only for full original authority.
    #[cfg(any(windows, test))]
    pub(crate) fn check_authority(
        &self,
        targets: &[Target],
        artifacts: &std::collections::BTreeMap<String, String>,
        operator: &str,
    ) -> Result<(), String> {
        self.check_scope(&self.review, artifacts, operator)?;
        if targets != [self.review.target()?] {
            return Err(
                "installed authority does not match original setup; retain recovery".into(),
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn review() -> PairReview {
        PairReview {
            vm_id: "12345678-1234-1234-1234-123456789abc".into(),
            gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#fixture\GPUPARAV".into(),
            expected: Revision::of(b"policy"),
            operator_sid: "S-1-5-21-1".into(),
        }
    }

    #[test]
    fn addition_preserves_old_intent_and_authorizes_only_new_identity() {
        let review = review();
        let mut old = review.target().unwrap();
        old.vm_id = "12345678-1234-1234-1234-123456789abd".into();
        old.enabled = true;
        let result = review
            .append(&[old.clone()], &review.expected, &review.operator_sid)
            .unwrap();
        assert_eq!(result[0], old);
        assert_eq!(result[1], review.target().unwrap());
        assert!(!result[1].enabled);
        assert!(result[1].vram.is_none());
    }

    #[test]
    fn stale_identity_revision_capacity_and_replacement_are_rejected() {
        let review = review();
        assert!(
            review
                .append(&[], &Revision::of(b"changed"), &review.operator_sid)
                .is_err()
        );
        assert!(
            review
                .append(&[], &Revision::Missing, &review.operator_sid)
                .is_err()
        );
        assert!(
            review
                .append(&[], &review.expected, "another operator")
                .is_err()
        );
        let mut old = review.target().unwrap();
        old.gpu_interface = r"\\?\PCI#VEN_10DE&DEV_2D05#other\GPUPARAV".into();
        assert!(
            review
                .append(&[old.clone()], &review.expected, &review.operator_sid)
                .is_err()
        );
        old.vm_id = "12345678-1234-1234-1234-123456789abd".into();
        assert!(
            review
                .append(&vec![old; 128], &review.expected, &review.operator_sid)
                .is_err()
        );
    }

    #[test]
    fn publication_failures_cannot_skip_hold_or_claim_readback() {
        use std::cell::Cell;
        let held = Cell::new(false);
        let changed = Cell::new(false);
        let read = Cell::new(false);
        let revision = Revision::of(b"new policy");
        assert!(
            publish_pair(
                &revision,
                || Err("hold failed".into()),
                || {
                    changed.set(true);
                    Ok(())
                },
                || Ok(revision.clone())
            )
            .is_err()
        );
        assert!(!changed.get());
        assert!(
            publish_pair(
                &revision,
                || {
                    held.set(true);
                    Ok(())
                },
                || Err("publication failed".into()),
                || {
                    read.set(true);
                    Ok(revision.clone())
                }
            )
            .is_err()
        );
        assert!(held.get());
        assert!(!read.get());
        assert!(
            publish_pair(
                &revision,
                || Ok(()),
                || {
                    changed.set(true);
                    Ok(())
                },
                || Ok(Revision::of(b"third policy"))
            )
            .is_err()
        );
        assert!(changed.get());
        assert!(held.get());
        publish_pair(&revision, || Ok(()), || Ok(()), || Ok(revision.clone())).unwrap();
        // Even success cannot release authority recovery before terminal audit.
        assert!(held.get());
    }

    #[test]
    fn reconciliation_accepts_complete_old_or_new_without_replaying_authority() {
        let review = review();
        let pending = PendingPair {
            operation_id: "operation".into(),
            review: review.clone(),
            resulting: Revision::of(b"new"),
        };
        for actual in [&review.expected, &pending.resulting] {
            pending
                .check_readback("operation", actual, actual, &review.operator_sid)
                .unwrap();
        }
        let third = Revision::of(b"third");
        assert!(
            pending
                .check_readback("operation", &third, &third, &review.operator_sid)
                .is_err()
        );
        assert!(
            pending
                .check_readback(
                    "operation",
                    &review.expected,
                    &pending.resulting,
                    &review.operator_sid
                )
                .is_err()
        );
        assert!(
            pending
                .check_readback(
                    "another operation",
                    &review.expected,
                    &review.expected,
                    &review.operator_sid
                )
                .is_err()
        );
        assert!(
            pending
                .check_readback(
                    "operation",
                    &review.expected,
                    &review.expected,
                    "another operator"
                )
                .is_err()
        );
    }

    #[test]
    fn interrupted_setup_cannot_change_initial_pair_package_or_operator() {
        let mut review = review();
        review.expected = Revision::Missing;
        let artifacts =
            std::collections::BTreeMap::from([("fixed.exe".into(), "original hash".into())]);
        let pending = PendingSetup {
            operation_id: "setup".into(),
            review: review.clone(),
            artifacts: artifacts.clone(),
        };
        pending
            .check_scope(&review, &artifacts, &review.operator_sid)
            .unwrap();
        let mut changed_pair = review.clone();
        changed_pair.vm_id = "12345678-1234-1234-1234-123456789abd".into();
        assert!(
            pending
                .check_scope(&changed_pair, &artifacts, &review.operator_sid)
                .is_err()
        );
        let changed_artifacts =
            std::collections::BTreeMap::from([("fixed.exe".into(), "changed hash".into())]);
        assert!(
            pending
                .check_scope(&review, &changed_artifacts, &review.operator_sid)
                .is_err()
        );
        assert!(
            pending
                .check_scope(&review, &artifacts, "another operator")
                .is_err()
        );
        pending
            .check_authority(
                &[review.target().unwrap()],
                &artifacts,
                &review.operator_sid,
            )
            .unwrap();
        assert!(
            pending
                .check_authority(&[], &artifacts, &review.operator_sid)
                .is_err()
        );
        assert!(
            pending
                .check_authority(
                    &[review.target().unwrap(), changed_pair.target().unwrap()],
                    &artifacts,
                    &review.operator_sid
                )
                .is_err()
        );
        let mut changed_intent = review.target().unwrap();
        changed_intent.enabled = true;
        assert!(
            pending
                .check_authority(&[changed_intent], &artifacts, &review.operator_sid)
                .is_err()
        );
    }
}
