//! Exact-pair enrollment, removal and interrupted authority reconciliation.
use super::{
    protocol::{Outcome, Progress, StageStatus},
    runner,
};
use crate::{configuration_store::Revision, credentials::Credential};

pub(super) fn forget(
    session: &str,
    approved: crate::enrollment::RemovalReview,
    credential: Option<Credential>,
    mut notify: impl FnMut(Progress) -> Result<(), String>,
) -> Result<Outcome, String> {
    if credential.is_some() {
        return Err("Pair removal does not accept guest credentials.".into());
    }
    notify(Progress {
        stage: "Remove inactive pairing and profile records".into(),
        status: StageStatus::Running,
    })?;
    runner::forget_pair(&approved, session)?;
    notify(Progress {
        stage: "Remove inactive pairing and profile records".into(),
        status: StageStatus::Done,
    })?;
    Ok(Outcome {
        operation: None,
        saved: false,
        finished: true,
        operation_id: session.into(),
        save_error: None,
    })
}

pub(super) fn enroll(
    session: &str,
    approved: crate::enrollment::PairReview,
    credential: Option<Credential>,
    mut notify: impl FnMut(Progress) -> Result<(), String>,
) -> Result<Outcome, String> {
    if credential.is_some() {
        return Err("enrollment does not accept guest credentials".into());
    }
    notify(Progress {
        stage: "Validate and enroll exact VM/GPU pair".into(),
        status: StageStatus::Running,
    })?;
    runner::enroll_pair(&approved, session)?;
    notify(Progress {
        stage: "Exact pair authority read back; GPU support has not been applied".into(),
        status: StageStatus::Done,
    })?;
    Ok(Outcome {
        operation: None,
        saved: false,
        finished: true,
        operation_id: session.into(),
        save_error: None,
    })
}

pub(super) fn reconcile(
    session: &str,
    operation_id: String,
    expected: Revision,
    credential: Option<Credential>,
) -> Result<Outcome, String> {
    if credential.is_some() {
        return Err("enrollment reconciliation does not accept credentials".into());
    }
    runner::reconcile_pair(&operation_id, &expected)?;
    Ok(Outcome {
        operation: None,
        saved: false,
        finished: true,
        operation_id: session.into(),
        save_error: None,
    })
}
