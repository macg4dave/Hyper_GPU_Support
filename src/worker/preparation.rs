//! One authenticated preparation/approval exchange, with no transferable admission token.
use super::{
    client::submit_with_preparation,
    operations,
    progress::ObservedBackend,
    protocol::{Command, Outcome, Progress},
    runner,
};
use crate::{
    configuration_store::{self, Revision},
    credentials::Credential,
    model::{Power, Target},
    runner::NativeBackend,
    workflow::{self, Plan},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum Intent {
    Apply {
        target: Target,
        reinstall: bool,
        expected: Revision,
    },
    Verify {
        target: Target,
        expected: Revision,
    },
}
impl Intent {
    pub(super) fn target(&self) -> &Target {
        match self {
            Self::Apply { target, .. } | Self::Verify { target, .. } => target,
        }
    }
    fn expected(&self) -> &Revision {
        match self {
            Self::Apply { expected, .. } | Self::Verify { expected, .. } => expected,
        }
    }
    pub(super) fn operation_name(&self) -> &'static str {
        match self {
            Self::Apply { .. } => "ApplyPrepared",
            Self::Verify { .. } => "VerifyPrepared",
        }
    }
    pub(super) fn validate_review(&self, review: &PreparedReview) -> Result<(), String> {
        let valid = match (self, review) {
            (
                Self::Apply {
                    target,
                    reinstall,
                    expected,
                },
                PreparedReview::Apply {
                    plan,
                    expected: actual,
                },
            ) => {
                plan.desired == *target
                    && plan.reinstall == *reinstall
                    && actual == expected
                    && !plan.authentication_deferred
                    && plan.observed.vm_id == target.vm_id
                    && plan.gpu.interface == target.gpu_interface
            }
            (
                Self::Verify { target, expected },
                PreparedReview::Verify {
                    plan,
                    expected: actual,
                },
            ) => {
                plan.target == *target
                    && actual == expected
                    && plan.observed.vm_id == target.vm_id
                    && plan.gpu.interface == target.gpu_interface
            }
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err("Prepared operation differs from the requested target, revision or action".into())
        }
    }
}

/// Fully authenticated review from the still-running, locked worker session.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(
    clippy::large_enum_variant,
    reason = "One frame-bounded review per worker; retain the owned public plan contract"
)]
pub enum PreparedReview {
    /// Apply/reinstall scope, including current payload, lifecycle and exact saved revision.
    Apply {
        /// Fresh authenticated plan; presentation alone cannot execute it.
        plan: Plan,
        /// Required saved-intent revision.
        expected: Revision,
    },
    /// Verification scope; no installation or configuration publication is admitted.
    Verify {
        /// Fresh authenticated check plan.
        plan: workflow::VerificationPlan,
        /// Required saved-intent revision.
        expected: Revision,
    },
}
impl PreparedReview {
    pub(super) fn revision(&self) -> Result<Revision, String> {
        Ok(Revision::of(
            &serde_json::to_vec(self).map_err(|e| e.to_string())?,
        ))
    }
    pub(super) fn validate_power(&self, approved: bool) -> Result<(), String> {
        match self {
            Self::Apply { plan, .. } if plan.preview.guest_downtime && !approved => {
                Err("Graceful shutdown approval required".into())
            }
            Self::Verify { plan, .. } if plan.observed.power == Power::Off && !approved => {
                Err("Temporary start and shutdown approval required".into())
            }
            _ => Ok(()),
        }
    }
}

/// Local acknowledgement of one exact prepared scope; never persist credentials.
pub struct Authorization {
    /// Approve the plan's graceful shutdown/temporary boot, where required.
    pub shutdown_approved: bool,
    /// Credentials acquired only after successful host preparation.
    pub credential: Option<Credential>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Approval {
    schema: u32,
    session: String,
    review: Revision,
    shutdown_approved: bool,
    credential: Option<Credential>,
}
impl Approval {
    pub(super) fn new(
        session: &str,
        review: &PreparedReview,
        authorization: Authorization,
    ) -> Result<Self, String> {
        Ok(Self {
            schema: 1,
            session: session.into(),
            review: review.revision()?,
            shutdown_approved: authorization.shutdown_approved,
            credential: authorization.credential,
        })
    }
    pub(super) fn validate(
        self,
        session: &str,
        review: &PreparedReview,
    ) -> Result<Authorization, String> {
        if self.schema != 1 || self.session != session || self.review != review.revision()? {
            return Err("Prepared approval identity or scope changed".into());
        }
        review.validate_power(self.shutdown_approved)?;
        let needs = match review {
            PreparedReview::Apply { plan, .. } => plan.preview.credentials_required,
            PreparedReview::Verify { .. } => true,
        };
        if needs != self.credential.is_some() {
            return Err("Credentials differ from prepared guest scope".into());
        }
        Ok(Authorization {
            shutdown_approved: self.shutdown_approved,
            credential: self.credential,
        })
    }
}

/// Prepare and execute in one locked worker. The callback approves only the
/// returned scope and acquires any credential locally, before effects start.
pub fn apply_prepared(
    target: Target,
    reinstall: bool,
    expected: Revision,
    authorize: impl FnMut(&PreparedReview) -> Result<Authorization, String>,
    progress: impl FnMut(Progress),
) -> Result<Outcome, String> {
    submit_with_preparation(
        Command::Prepare {
            intent: Intent::Apply {
                target,
                reinstall,
                expected,
            },
        },
        None,
        authorize,
        progress,
    )
}
/// Prepare Verify once and retain its payload until consumed by that same worker.
pub fn verify_prepared(
    target: Target,
    expected: Revision,
    authorize: impl FnMut(&PreparedReview) -> Result<Authorization, String>,
    progress: impl FnMut(Progress),
) -> Result<Outcome, String> {
    submit_with_preparation(
        Command::Prepare {
            intent: Intent::Verify { target, expected },
        },
        None,
        authorize,
        progress,
    )
}

pub(super) fn execute(
    context: &operations::Context<'_>,
    intent: Intent,
    mut notify: impl FnMut(Progress) -> Result<(), String>,
    mut exchange: impl FnMut(&PreparedReview) -> Result<Authorization, String>,
) -> Result<Outcome, String> {
    let target = intent.target();
    target.validate()?;
    if !context
        .enrollment
        .targets
        .iter()
        .any(|t| t.vm_id == target.vm_id && t.gpu_interface == target.gpu_interface)
    {
        return Err("Prepared target is outside enrollment".into());
    }
    runner::require_no_vm_recovery(&target.vm_id)?;
    crate::windows_wmi::require_no_pending_operation_for(&target.vm_id)?;
    if configuration_store::protected::revision(&target.vm_id)? != *intent.expected() {
        return Err("Committed configuration changed before preparation".into());
    }
    let mut backend = NativeBackend::new(context.data.into(), None);
    let (review, admitted) = ObservedBackend {
        backend: &mut backend,
        notify: &mut notify,
        before_effect: None,
    }
    .stage_observed("Checking reviewed VM and driver files", |backend| {
        crate::read_context::ReadControl::new(Duration::from_secs(360)).scope(|| match intent {
            Intent::Apply {
                target,
                reinstall,
                expected,
            } => {
                let (plan, token) = workflow::prepare_apply(backend, &target, reinstall)?;
                Ok((
                    PreparedReview::Apply { plan, expected },
                    operations::Admitted::Apply(Box::new(token)),
                ))
            }
            Intent::Verify { target, expected } => {
                let (plan, token) = workflow::prepare_verification(backend, &target)?;
                Ok((
                    PreparedReview::Verify { plan, expected },
                    operations::Admitted::Verify(Box::new(token)),
                ))
            }
        })
    })?;
    let authorization = exchange(&review)?;
    // Validate in-process exchanges too; the pipe additionally binds frame identity.
    let authorization = Approval::new(context.session, &review, authorization)?
        .validate(context.session, &review)?;
    backend.credential = authorization.credential;
    let target = match &review {
        PreparedReview::Apply { plan, .. } => &plan.desired,
        PreparedReview::Verify { plan, .. } => &plan.target,
    };
    backend.confirm_payload_inventory(target)?;
    operations::execute(context, review, backend, admitted, notify)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::test_support::{fake, target};

    fn review() -> PreparedReview {
        let (plan, _) = workflow::prepare_apply(&mut fake(), &target(), false).unwrap();
        PreparedReview::Apply {
            plan,
            expected: Revision::Missing,
        }
    }
    fn authorization() -> Authorization {
        Authorization {
            shutdown_approved: true,
            credential: Some(Credential {
                username: "fixture".into(),
                password: "fixture only".into(),
            }),
        }
    }

    #[test]
    fn approval_is_bound_to_session_exact_payload_scope_and_saved_revision() {
        let review = review();
        assert!(
            Approval::new("session", &review, authorization())
                .unwrap()
                .validate("session", &review)
                .is_ok()
        );
        assert!(
            Approval::new("session", &review, authorization())
                .unwrap()
                .validate("other", &review)
                .is_err()
        );
        for change in 0..4 {
            let approval = Approval::new("session", &review, authorization()).unwrap();
            let mut changed = review.clone();
            let PreparedReview::Apply { plan, expected } = &mut changed else {
                unreachable!()
            };
            match change {
                0 => plan
                    .preparation
                    .as_mut()
                    .unwrap()
                    .digest
                    .push_str("same-version-changed-membership-or-bytes"),
                1 => plan.preview.actions.push(workflow::PreviewAction::Detach),
                2 => *expected = Revision::of(b"changed configuration"),
                _ => plan.desired.enabled = false,
            }
            assert!(approval.validate("session", &changed).is_err());
        }
    }

    #[test]
    fn wrong_operation_target_reinstall_and_credentials_are_rejected() {
        let review = review();
        let intent = Intent::Apply {
            target: target(),
            reinstall: false,
            expected: Revision::Missing,
        };
        intent.validate_review(&review).unwrap();
        assert!(
            Intent::Apply {
                target: target(),
                reinstall: true,
                expected: Revision::Missing
            }
            .validate_review(&review)
            .is_err()
        );
        assert!(
            Intent::Verify {
                target: target(),
                expected: Revision::Missing
            }
            .validate_review(&review)
            .is_err()
        );
        let mut changed = target();
        changed.gpu_interface.push_str("changed");
        assert!(
            Intent::Apply {
                target: changed,
                reinstall: false,
                expected: Revision::Missing
            }
            .validate_review(&review)
            .is_err()
        );
        assert!(
            Approval::new(
                "session",
                &review,
                Authorization {
                    shutdown_approved: true,
                    credential: None
                }
            )
            .unwrap()
            .validate("session", &review)
            .is_err()
        );
        assert!(
            Approval::new(
                "session",
                &review,
                Authorization {
                    shutdown_approved: false,
                    credential: authorization().credential
                }
            )
            .unwrap()
            .validate("session", &review)
            .is_err()
        );
        let mut approval = Approval::new("session", &review, authorization()).unwrap();
        approval.schema = 9;
        assert!(approval.validate("session", &review).is_err());
    }

    #[test]
    fn verify_approval_requires_exact_revision_credentials_and_approved_temporary_power() {
        for running in [false, true] {
            let mut backend = fake();
            let target = target();
            workflow::apply(&mut backend, &target).unwrap();
            backend.state.power = if running { Power::Running } else { Power::Off };
            let expected = Revision::of(b"saved revision");
            let (plan, _) = workflow::prepare_verification(&mut backend, &target).unwrap();
            let review = PreparedReview::Verify {
                plan,
                expected: expected.clone(),
            };
            Intent::Verify {
                target: target.clone(),
                expected,
            }
            .validate_review(&review)
            .unwrap();
            assert!(
                Intent::Verify {
                    target,
                    expected: Revision::Missing
                }
                .validate_review(&review)
                .is_err()
            );
            for power_approved in [false, true] {
                for credential_present in [false, true] {
                    let mut authorization = authorization();
                    authorization.shutdown_approved = power_approved;
                    if !credential_present {
                        authorization.credential = None;
                    }
                    let approval = Approval::new("session", &review, authorization).unwrap();
                    assert_eq!(
                        approval.validate("session", &review).is_ok(),
                        credential_present && (running || power_approved)
                    );
                }
            }
        }
    }

    #[test]
    fn ordinary_prepared_scope_requires_no_credentials_but_keeps_power_and_session_approval() {
        let mut backend = fake();
        let target = target();
        workflow::apply(&mut backend, &target).unwrap();
        backend.state.power = crate::model::Power::Running;
        let (plan, _) = workflow::prepare_apply(&mut backend, &target, false).unwrap();
        assert!(!plan.preview.credentials_required);
        let intent = Intent::Apply {
            target,
            reinstall: false,
            expected: Revision::Missing,
        };
        let review = PreparedReview::Apply {
            plan,
            expected: Revision::Missing,
        };
        intent.validate_review(&review).unwrap();
        assert!(
            review.validate_power(false).is_err(),
            "CLI power approval is still required"
        );
        review.validate_power(true).unwrap();
        let approval = Approval::new(
            "ordinary",
            &review,
            Authorization {
                shutdown_approved: true,
                credential: None,
            },
        )
        .unwrap();
        approval.validate("ordinary", &review).unwrap();
        assert!(
            Approval::new("ordinary", &review, authorization())
                .unwrap()
                .validate("ordinary", &review)
                .is_err(),
            "unexpected credentials are refused"
        );
    }
}
