//! Authenticate the elevated worker and its frontend before dispatching a bounded request.
use super::{
    dispatch::execute,
    preparation::{self, PreparedReview},
    process::{OwnedProcess, pipe_name, trusted_executable, valid_session},
    protocol::{Message, Progress, Request},
    runner,
};
use crate::windows_pipe::{self, session::Pipe};
use std::time::Duration;
use windows::Win32::{
    Foundation::WAIT_OBJECT_0,
    System::{
        RemoteDesktop::ProcessIdToSessionId,
        Threading::{
            GetCurrentProcessId, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
            PROCESS_SYNCHRONIZE, WaitForSingleObject,
        },
    },
};

/// Restricted dispatch; rejects ordinary tokens, arbitrary paths and other processes.
#[allow(unsafe_code)]
pub fn serve(session: &str, frontend: u32) -> Result<(), String> {
    if !valid_session(session) || frontend == 0 || !crate::process::is_elevated()? {
        return Err("invalid restricted worker invocation".into());
    }
    if std::env::current_exe().map_err(|e| e.to_string())? != trusted_executable()? {
        return Err("restricted worker must execute from its protected installation".into());
    }
    let enrollment = runner::enrollment()?;
    if windows_pipe::current_user_sid_string().map_err(|e| e.to_string())? != enrollment.client_sid
    {
        return Err("worker operator identity mismatch".into());
    }
    // SAFETY: query-only retained handle to the supplied frontend, validated against the pipe peer.
    let parent = OwnedProcess(
        unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                false,
                frontend,
            )
        }
        .map_err(|e| e.to_string())?,
    );
    let mut their_session = 0;
    let mut our_session = 0;
    unsafe { ProcessIdToSessionId(frontend, &mut their_session) }.map_err(|e| e.to_string())?;
    unsafe { ProcessIdToSessionId(GetCurrentProcessId(), &mut our_session) }
        .map_err(|e| e.to_string())?;
    if their_session != our_session || unsafe { WaitForSingleObject(parent.0, 0) } == WAIT_OBJECT_0
    {
        return Err("frontend session ended or differs from the worker".into());
    }
    let _deadline = crate::process::WorkerDeadline::start(Duration::from_secs(3900));
    let pipe = Pipe::accept(&pipe_name(session)?, &enrollment.client_sid, frontend)
        .map_err(|e| e.to_string())?;
    let bytes = zeroize::Zeroizing::new(
        pipe.receive(Duration::from_secs(30))
            .map_err(|e| e.to_string())?,
    );
    pipe.authenticate_client(&enrollment.client_sid)
        .map_err(|e| e.to_string())?;
    let request: Request =
        serde_json::from_slice(&bytes).map_err(|_| "invalid bounded worker request")?;
    if request.schema != 1 || request.session != session {
        return Err("worker request does not match launch identity".into());
    }
    let notify = |update: Progress| {
        pipe.send(
            &serde_json::to_vec(&Message::Progress {
                session: session.into(),
                progress: update,
            })
            .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())
    };
    let exchange = |review: &PreparedReview| {
        pipe.send(
            &serde_json::to_vec(&Message::Prepared {
                session: session.into(),
                review: Box::new(review.clone()),
            })
            .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let bytes = zeroize::Zeroizing::new(
            pipe.receive(Duration::from_secs(300))
                .map_err(|e| e.to_string())?,
        );
        let approval: preparation::Approval =
            serde_json::from_slice(&bytes).map_err(|_| "Invalid preparation approval")?;
        approval.validate(session, review)
    };
    let result = execute(&enrollment, session, request, notify, exchange);
    pipe.send(
        &serde_json::to_vec(&Message::Complete {
            session: session.into(),
            result: Box::new(result),
        })
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
