//! Public submissions and the process-bound client exchange, including release observation.
use super::{
    preparation::{self, Authorization, PreparedReview},
    process::{OwnedProcess, launch, pipe_name, random_session},
    progress::preparation_stage,
    protocol::{Command, Message, Outcome, Progress, Request},
    runner,
};
use crate::{
    configuration_store::Revision,
    credentials::Credential,
    windows_pipe::{self, session::Pipe},
    workflow::{self, Plan},
};
use std::time::{Duration, Instant};
use windows::Win32::{
    Foundation::WAIT_OBJECT_0,
    System::Threading::{GetProcessId, WaitForSingleObject},
};

struct ReleaseGuard<'a> {
    pub pipe: Option<Pipe>,
    process: &'a OwnedProcess,
    release: Option<crate::read_context::Release>,
}
impl<'a> ReleaseGuard<'a> {
    fn new(
        process: &'a OwnedProcess,
        pipe: Pipe,
        release: Option<crate::read_context::Release>,
    ) -> Self {
        Self {
            process,
            pipe: Some(pipe),
            release,
        }
    }
}
impl Drop for ReleaseGuard<'_> {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        drop(self.pipe.take());
        if let Some(release) = self.release.take() {
            // SAFETY: retained exact launch handle; observing exit never stops a
            // process or equates pipe cancellation with backend release.
            if unsafe { WaitForSingleObject(self.process.0, 3_905_000) } == WAIT_OBJECT_0 {
                release.confirm();
            }
        }
    }
}

/// Authorize only the separately reviewed pair; no GPU/guest/configuration effects.
pub fn enroll_pair(
    approved: crate::enrollment::PairReview,
    progress: impl FnMut(Progress),
) -> Result<Outcome, String> {
    submit(Command::EnrollPair { approved }, None, progress)
}

/// Explicitly resolve interrupted authority publication from independent readback.
pub fn reconcile_pair(
    operation_id: String,
    expected: Revision,
    progress: impl FnMut(Progress),
) -> Result<Outcome, String> {
    submit(
        Command::ReconcilePair {
            operation_id,
            expected,
        },
        None,
        progress,
    )
}

/// Remove only an explicitly reviewed inactive profile; no guest credential/effects.
pub fn forget_pair(
    approved: crate::enrollment::RemovalReview,
    progress: impl FnMut(Progress),
) -> Result<Outcome, String> {
    submit(Command::ForgetPair { approved }, None, progress)
}
/// Launch elevation before effects; private request and progress are process-bound.
pub fn apply(
    approved: Plan,
    expected: Revision,
    shutdown_approved: bool,
    credential: Option<Credential>,
    progress: impl FnMut(Progress),
) -> Result<Outcome, String> {
    if approved.preview.guest_downtime && !shutdown_approved {
        return Err("separate graceful guest shutdown approval is required".into());
    }
    submit(
        Command::Apply {
            approved,
            expected,
            shutdown_approved,
        },
        credential,
        progress,
    )
}
/// Retry protected publication only; this path cannot call GPU mutation adapters.
pub fn save_only(operation_id: String, progress: impl FnMut(Progress)) -> Result<Outcome, String> {
    submit(Command::SaveOnly { operation_id }, None, progress)
}
/// Execute only the reviewed guest graphics check and disclosed power restoration.
pub fn verify(
    approved: workflow::VerificationPlan,
    shutdown_approved: bool,
    credential: Credential,
    progress: impl FnMut(Progress),
) -> Result<Outcome, String> {
    submit(
        Command::Verify {
            approved,
            shutdown_approved,
        },
        Some(credential),
        progress,
    )
}
/// Explicit reconciliation checks achieved state, graphics and disclosed power;
/// it cannot repeat GPU assignment, allocation, compatibility or preparation.
pub fn reconcile(
    operation_id: String,
    shutdown_approved: bool,
    credential: Option<Credential>,
    progress: impl FnMut(Progress),
) -> Result<Outcome, String> {
    submit(
        Command::Reconcile {
            operation_id,
            shutdown_approved,
        },
        credential,
        progress,
    )
}
/// Remove a reviewed partial attachment and restore its recorded starting host
/// settings, keeping guest files and archived failure evidence.
pub fn reset_partial(
    operation_id: String,
    record_revision: Revision,
    approved: workflow::ResetReview,
    shutdown_approved: bool,
    progress: impl FnMut(Progress),
) -> Result<Outcome, String> {
    submit(
        Command::ResetPartial {
            operation_id,
            record_revision,
            approved,
            shutdown_approved,
        },
        None,
        progress,
    )
}
/// Explicitly import a bundle into approved absent canonical destinations. Source
/// is transferred as bounded data, never interpreted as a worker filesystem path.
pub fn import(
    source: String,
    expected: std::collections::BTreeMap<String, Revision>,
    progress: impl FnMut(Progress),
) -> Result<Outcome, String> {
    submit(Command::Import { source, expected }, None, progress)
}
/// Explicitly reconcile interrupted per-file import, recognizing completed bytes.
pub fn resume_import(
    operation_id: String,
    progress: impl FnMut(Progress),
) -> Result<Outcome, String> {
    submit(Command::ImportResume { operation_id }, None, progress)
}
#[allow(unsafe_code)]
fn submit(
    command: Command,
    credential: Option<Credential>,
    mut progress: impl FnMut(Progress),
) -> Result<Outcome, String> {
    submit_with_preparation(
        command,
        credential,
        |_: &PreparedReview| Err("Unexpected worker preparation".into()),
        &mut progress,
    )
}

#[allow(unsafe_code)]
pub(super) fn submit_with_preparation(
    command: Command,
    credential: Option<Credential>,
    mut authorize: impl FnMut(&PreparedReview) -> Result<Authorization, String>,
    mut progress: impl FnMut(Progress),
) -> Result<Outcome, String> {
    let session = random_session()?;
    let (pipe, process) = preparation_stage(
        "Prepare protected worker connection",
        &mut |update| {
            progress(update);
            Ok(())
        },
        || {
            let enrollment = runner::enrollment()?;
            if windows_pipe::current_user_sid_string().map_err(|e| e.to_string())?
                != enrollment.client_sid
            {
                return Err("current user is not the enrolled operator".into());
            }
            let process = launch(&session)?;
            // Retained handle prevents PID reuse until the exchange completes.
            let pid = unsafe { GetProcessId(process.0) };
            if pid == 0 {
                return Err("worker process identity unavailable".into());
            }
            let pipe = Pipe::connect(&pipe_name(&session)?, "S-1-5-32-544", pid).map_err(|e| {
                format!("worker connection failed; inspect recovery before retry: {e}")
            })?;
            Ok((pipe, process))
        },
    )?;
    let preparing = matches!(command, Command::Prepare { .. });
    let read = crate::read_context::request();
    let connection = ReleaseGuard::new(&process, pipe, preparing.then(|| read.submitted()));
    let pipe = connection
        .pipe
        .as_ref()
        .ok_or("Missing worker connection")?;
    let request = Request {
        schema: 1,
        session: session.clone(),
        command,
        credential,
    };
    let bytes = zeroize::Zeroizing::new(serde_json::to_vec(&request).map_err(|e| e.to_string())?);
    pipe.send(&bytes)
        .map_err(|e| format!("worker request outcome uncertain: {e}"))?;
    let deadline = Instant::now() + Duration::from_secs(3900);
    let mut prepared_command = None;
    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or("worker timed out; inspect durable recovery before retry")?;
        let bytes = (if preparing && prepared_command.is_none() {
            read.scope(|| {
                pipe.receive(remaining.min(read.remaining()))
                    .map_err(|e| e.to_string())
            })
        } else {
            pipe.receive(remaining).map_err(|e| e.to_string())
        })
        .map_err(|e| format!("worker disconnected; outcome uncertain; inspect recovery: {e}"))?;
        match serde_json::from_slice::<Message>(&bytes).map_err(|e| e.to_string())? {
            Message::Prepared {
                session: actual,
                review,
            } if actual == session => {
                if prepared_command.is_some() {
                    return Err("Repeated worker preparation".into());
                }
                let Command::Prepare { intent } = &request.command else {
                    return Err("Unrequested worker preparation".into());
                };
                intent.validate_review(&review)?;
                read.check()?;
                let authorization = authorize(&review)?;
                read.check()?;
                prepared_command = Some(review.command(authorization.shutdown_approved)?);
                let approval = preparation::Approval::new(&session, &review, authorization)?;
                let bytes = zeroize::Zeroizing::new(
                    serde_json::to_vec(&approval).map_err(|e| e.to_string())?,
                );
                pipe.send(&bytes)
                    .map_err(|e| format!("Prepared worker approval outcome uncertain: {e}"))?;
            }
            Message::Progress {
                session: actual,
                progress: update,
            } if actual == session => progress(update),
            Message::Complete {
                session: actual,
                result,
            } if actual == session => {
                return (*result).and_then(|mut outcome| {
                    prepared_command
                        .as_ref()
                        .unwrap_or(&request.command)
                        .validate_outcome(&outcome, &session)?;
                    outcome.save_error = outcome.save_error.map(|error| {
                        crate::reporting::operator_error("Configuration publication", &error)
                    });
                    Ok(outcome)
                });
            }
            _ => return Err("worker session identity mismatch; inspect recovery".into()),
        }
    }
}
