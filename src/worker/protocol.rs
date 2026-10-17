//! Worker wire contracts and validation of authenticated terminal outcomes.
use super::preparation::{self, PreparedReview};
use crate::{
    configuration_store::Revision,
    credentials::Credential,
    workflow::{self, OperationResult, Plan},
};
use serde::{Deserialize, Serialize};

/// Concrete execution state, not a percentage or demonstration timer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageStatus {
    /// Work is about to enter an adapter.
    Running,
    /// Adapter completed successfully.
    Done,
    /// Adapter failed; durable recovery remains.
    Failed,
}
/// Typed progress frame. No request/credential is logged or returned in progress.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    /// Fixed stage label.
    pub stage: String,
    /// Actual adapter outcome.
    pub status: StageStatus,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum Command {
    Prepare {
        intent: preparation::Intent,
    },
    ResetPartial {
        operation_id: String,
        record_revision: Revision,
        approved: workflow::ResetReview,
        shutdown_approved: bool,
    },
    ForgetPair {
        approved: crate::enrollment::RemovalReview,
    },
    EnrollPair {
        approved: crate::enrollment::PairReview,
    },
    ReconcilePair {
        operation_id: String,
        expected: Revision,
    },
    Apply {
        approved: Plan,
        expected: Revision,
        shutdown_approved: bool,
    },
    Verify {
        approved: workflow::VerificationPlan,
        shutdown_approved: bool,
    },
    Reconcile {
        operation_id: String,
        shutdown_approved: bool,
    },
    SaveOnly {
        operation_id: String,
    },
    Import {
        source: String,
        expected: std::collections::BTreeMap<String, Revision>,
    },
    ImportResume {
        operation_id: String,
    },
}

impl Command {
    /// An authenticated terminal reply must still match the submitted operation.
    /// Progress completion is never sufficient to establish a terminal outcome.
    pub(super) fn validate_outcome(&self, outcome: &Outcome, session: &str) -> Result<(), String> {
        let expected_id = match self {
            Self::SaveOnly { operation_id }
            | Self::ResetPartial { operation_id, .. }
            | Self::Reconcile { operation_id, .. }
            | Self::ImportResume { operation_id } => operation_id.as_str(),
            _ => session,
        };
        let invalid = || {
            "Worker terminal reply does not match the reviewed operation; outcome uncertain. Refresh and inspect protected recovery.".to_owned()
        };
        if outcome.operation_id != expected_id
            || expected_id.is_empty()
            || ((outcome.saved || outcome.finished) && outcome.save_error.is_some())
        {
            return Err(invalid());
        }
        match self {
            Self::Prepare { .. } => return Err(invalid()),
            Self::ResetPartial { .. } => {
                if outcome.operation.is_some() || outcome.saved || !outcome.finished {
                    return Err(invalid());
                }
            }
            Self::ForgetPair { .. } => {
                if outcome.operation.is_some() || outcome.saved || !outcome.finished {
                    return Err(invalid());
                }
            }
            Self::Apply { approved, .. } => {
                let operation = outcome.operation.as_ref().ok_or_else(invalid)?;
                let effective = &operation.effective;
                let target = &approved.desired;
                let expected_gpus = if target.enabled {
                    vec![target.gpu_interface.clone()]
                } else {
                    vec![]
                };
                let expected_settings = approved
                    .preview
                    .settings
                    .as_ref()
                    .map_or(&approved.observed.settings, |settings| &settings.after);
                if effective.vm_id != target.vm_id
                    || effective.generation != 2
                    || effective.gpus != expected_gpus
                    || effective.power != approved.preview.restore_power
                    || effective.settings != *expected_settings
                    || (target.enabled && !target.allocations_match(effective))
                    || operation.verified
                        != approved
                            .preview
                            .actions
                            .contains(&workflow::PreviewAction::Verify)
                    || operation.prepared
                        != approved
                            .preview
                            .actions
                            .contains(&workflow::PreviewAction::Prepare)
                    || operation.settings_preserved != approved.preview.settings_preserved
                    || outcome.saved != outcome.finished
                    || outcome.saved != outcome.save_error.is_none()
                {
                    return Err(invalid());
                }
            }
            Self::SaveOnly { .. } => {
                if outcome.operation.is_some() || !outcome.saved || !outcome.finished {
                    return Err(invalid());
                }
            }
            Self::Verify { approved, .. } => {
                let operation = outcome.operation.as_ref().ok_or_else(invalid)?;
                if operation.effective.vm_id != approved.target.vm_id
                    || operation.effective.gpus != approved.observed.gpus
                    || operation.effective.power
                        != workflow::verification_power(
                            &approved.target,
                            &approved.observed,
                            approved.managed.as_ref(),
                        )?
                    || !operation.verified
                    || operation.prepared
                    || outcome.saved
                    || !outcome.finished
                {
                    return Err(invalid());
                }
            }
            _ => {}
        }
        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Request {
    pub(super) schema: u32,
    pub(super) session: String,
    pub(super) command: Command,
    pub(super) credential: Option<Credential>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum Message {
    Prepared {
        session: String,
        review: Box<PreparedReview>,
    },
    Progress {
        session: String,
        progress: Progress,
    },
    Complete {
        session: String,
        result: Box<Result<Outcome, String>>,
    },
}
/// Verified GPU outcome and configuration outcome remain explicitly separate.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    /// Independent effective state and graphics result.
    pub operation: Option<OperationResult>,
    /// True only after protected publication readback.
    pub saved: bool,
    /// True only when all effects and required publication completed.
    pub finished: bool,
    /// Protected identity for save-only retry if publication failed.
    pub operation_id: String,
    /// Publication failure; GPU effects must never be replayed for this.
    pub save_error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::super::test_support::ordinary_disabled_plan;
    use super::*;
    use crate::model::Power;

    #[test]
    fn forget_terminal_reply_requires_complete_bound_removal_without_gpu_effect_result() {
        let plan = ordinary_disabled_plan();
        let approved = crate::enrollment::RemovalReview {
            recovery: Default::default(),
            pair: crate::enrollment::PairReview {
                vm_id: plan.desired.vm_id.clone(),
                gpu_interface: plan.desired.gpu_interface.clone(),
                expected: Revision::of(b"policy"),
                operator_sid: "S-1-5-21-1".into(),
            },
            configurations: std::collections::BTreeMap::from([(
                plan.desired.vm_id.clone(),
                Revision::Missing,
            )]),
            journals: std::collections::BTreeMap::from([(
                plan.desired.vm_id.clone(),
                Revision::Missing,
            )]),
        };
        let command = Command::ForgetPair { approved };
        let complete = Outcome {
            operation: None,
            saved: false,
            finished: true,
            operation_id: "session".into(),
            save_error: None,
        };
        command.validate_outcome(&complete, "session").unwrap();
        for change in 0..4 {
            let mut reply = complete.clone();
            match change {
                0 => reply.finished = false,
                1 => reply.saved = true,
                2 => reply.operation_id = "another-session".into(),
                _ => {
                    reply.operation = Some(OperationResult {
                        effective: plan.observed.clone(),
                        prepared: false,
                        verified: false,
                        settings_preserved: false,
                    })
                }
            }
            assert!(command.validate_outcome(&reply, "session").is_err());
        }
    }

    #[test]
    fn terminal_apply_reply_must_match_the_reviewed_pair_lifecycle_and_save_result() {
        let approved = ordinary_disabled_plan();
        let outcome = Outcome {
            operation: Some(OperationResult {
                effective: approved.observed.clone(),
                prepared: false,
                verified: false,
                settings_preserved: false,
            }),
            saved: true,
            finished: true,
            operation_id: "session".into(),
            save_error: None,
        };
        let command = Command::Apply {
            approved,
            expected: Revision::Missing,
            shutdown_approved: false,
        };
        command.validate_outcome(&outcome, "session").unwrap();
        let mut unsaved = outcome.clone();
        unsaved.saved = false;
        unsaved.finished = false;
        unsaved.save_error = Some("publication refused".into());
        command.validate_outcome(&unsaved, "session").unwrap();
        for changed in 0..8 {
            let mut reply = outcome.clone();
            match changed {
                0 => reply.operation_id = "other-operation".into(),
                1 => reply.operation.as_mut().unwrap().effective.vm_id = "other-vm".into(),
                2 => reply.operation.as_mut().unwrap().effective.power = Power::Running,
                3 => reply
                    .operation
                    .as_mut()
                    .unwrap()
                    .effective
                    .gpus
                    .push("other-gpu".into()),
                4 => reply.operation.as_mut().unwrap().verified = true,
                5 => reply.operation.as_mut().unwrap().prepared = true,
                6 => reply.finished = false,
                _ => reply.operation = None,
            }
            assert!(
                command
                    .validate_outcome(&reply, "session")
                    .unwrap_err()
                    .contains("outcome uncertain")
            );
        }
    }

    #[test]
    fn save_only_terminal_reply_retains_the_original_operation_binding() {
        let command = Command::SaveOnly {
            operation_id: "original".into(),
        };
        let mut outcome = Outcome {
            operation: None,
            saved: true,
            finished: true,
            operation_id: "original".into(),
            save_error: None,
        };
        command.validate_outcome(&outcome, "new-session").unwrap();
        outcome.operation_id = "new-session".into();
        assert!(command.validate_outcome(&outcome, "new-session").is_err());
        outcome.operation_id = "original".into();
        outcome.finished = false;
        assert!(command.validate_outcome(&outcome, "new-session").is_err());
    }

    #[test]
    fn completion_storage_keeps_the_existing_wire_format() {
        let wire = serde_json::json!({"Complete":{"session":"test-session","result":{"Err":"safe failure"}}});
        let message: Message = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(message).unwrap(), wire);
    }
}
