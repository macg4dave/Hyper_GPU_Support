//! One restricted elevated instance of the product executable for an approved plan.
//! Private process-bound IPC; fixed paths, exact enrollment and durable admission.
use crate::{
    configuration_store::{self, Revision, SavePhase, SaveRecord},
    credentials::Credential,
    model::{Allocation, Configuration, Gpu, Power, Resource, Settings, Target, VmState},
    runner::{self, NativeBackend},
    windows_pipe::{self, session::Pipe},
    workflow::{self, Backend, Journal, OperationResult, Plan},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0},
        Security::Cryptography::{BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom},
        System::{
            RemoteDesktop::ProcessIdToSessionId,
            Threading::{
                GetCurrentProcessId, GetProcessId, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
                PROCESS_SYNCHRONIZE, WaitForSingleObject,
            },
        },
        UI::{
            Shell::{
                SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW,
            },
            WindowsAndMessaging::SW_HIDE,
        },
    },
    core::PCWSTR,
};

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
enum Command {
    ResetPartial {
        operation_id: String,
        record_revision: Revision,
        approved: workflow::ResetReview,
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
    fn validate_outcome(&self, outcome: &Outcome, session: &str) -> Result<(), String> {
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
struct Request {
    schema: u32,
    session: String,
    command: Command,
    credential: Option<Credential>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Message {
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
pub(crate) struct OwnedProcess(pub(crate) HANDLE);
impl Drop for OwnedProcess {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: owns only its retained process handle; never terminates the process.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
pub(crate) fn valid_session(session: &str) -> bool {
    session.len() == 32
        && session
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
pub(crate) fn pipe_name(session: &str) -> Result<String, String> {
    if !valid_session(session) {
        return Err("invalid worker session identity".into());
    }
    Ok(format!(r"\\.\pipe\HyperGpuSupport.Operation.{session}"))
}
#[allow(unsafe_code)]
pub(crate) fn random_session() -> Result<String, String> {
    let mut bytes = [0_u8; 16];
    // SAFETY: fixed initialized output; uses Windows system-preferred cryptographic RNG.
    unsafe { BCryptGenRandom(None, &mut bytes, BCRYPT_USE_SYSTEM_PREFERRED_RNG) }
        .ok()
        .map_err(|e| e.to_string())?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
fn trusted_executable() -> Result<std::path::PathBuf, String> {
    let enrollment = runner::enrollment()?;
    runner::verify_artifacts(&enrollment)?;
    let path = runner::install_directory()?.join("hyper-gpu-support.exe");
    crate::security::verify(&path)?;
    if crate::payload::hash_file(&path)?
        != *enrollment
            .artifacts
            .get("hyper-gpu-support.exe")
            .ok_or("restricted worker is not installed; update the product installation")?
    {
        return Err("installed worker artifact changed".into());
    }
    Ok(path)
}
/// Report whether the protected, enrolled worker artifact is currently usable.
pub fn available() -> Result<(), String> {
    trusted_executable().map(|_| ())
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
#[allow(unsafe_code)]
fn launch(session: &str) -> Result<OwnedProcess, String> {
    let executable = trusted_executable()?;
    launch_fixed(&executable, session, "--internal-worker")
}

pub(crate) fn launch_setup(session: &str) -> Result<OwnedProcess, String> {
    let executable = crate::setup::source_executable()?;
    launch_fixed(&executable, session, "--internal-setup")
}

#[allow(unsafe_code)]
fn launch_fixed(
    executable: &std::path::Path,
    session: &str,
    mode: &str,
) -> Result<OwnedProcess, String> {
    if !valid_session(session) || !matches!(mode, "--internal-worker" | "--internal-setup") {
        return Err("invalid fixed worker launch".into());
    }
    let verb = wide("runas");
    let path: Vec<u16> = executable
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let parameters = wide(&format!("{mode} {session} {}", std::process::id()));
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(path.as_ptr()),
        lpParameters: PCWSTR(parameters.as_ptr()),
        nShow: SW_HIDE.0,
        ..Default::default()
    };
    // SAFETY: fixed installed-worker or same-package setup executable; callers
    // select one fixed internal mode and bounded numeric/session arguments.
    unsafe { ShellExecuteExW(&mut info) }
        .map_err(|e| format!("worker elevation refused or unavailable: {e}"))?;
    if info.hProcess.is_invalid() {
        return Err("worker launch returned no process identity".into());
    }
    Ok(OwnedProcess(info.hProcess))
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
    progress: impl FnMut(Progress),
) -> Result<Outcome, String> {
    submit(
        Command::ResetPartial {
            operation_id,
            record_revision,
            approved,
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
    let enrollment = runner::enrollment()?;
    if windows_pipe::current_user_sid_string().map_err(|e| e.to_string())? != enrollment.client_sid
    {
        return Err("current user is not the enrolled operator".into());
    }
    let session = random_session()?;
    let process = launch(&session)?;
    // Retained handle prevents PID reuse until the exchange completes.
    let pid = unsafe { GetProcessId(process.0) };
    if pid == 0 {
        return Err("worker process identity unavailable".into());
    }
    let pipe = Pipe::connect(&pipe_name(&session)?, "S-1-5-32-544", pid)
        .map_err(|e| format!("worker connection failed; inspect recovery before retry: {e}"))?;
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
    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or("worker timed out; inspect durable recovery before retry")?;
        let bytes = pipe.receive(remaining).map_err(|e| {
            format!("worker disconnected; outcome uncertain; inspect recovery: {e}")
        })?;
        match serde_json::from_slice::<Message>(&bytes).map_err(|e| e.to_string())? {
            Message::Progress {
                session: actual,
                progress: update,
            } if actual == session => progress(update),
            Message::Complete {
                session: actual,
                result,
            } if actual == session => {
                return (*result).and_then(|mut outcome| {
                    request.command.validate_outcome(&outcome, &session)?;
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
    let result = execute(&enrollment, session, request, notify);
    pipe.send(
        &serde_json::to_vec(&Message::Complete {
            session: session.into(),
            result: Box::new(result),
        })
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn execute(
    enrollment: &runner::Enrollment,
    session: &str,
    request: Request,
    mut notify: impl FnMut(Progress) -> Result<(), String>,
) -> Result<Outcome, String> {
    let data = runner::data_directory()?;
    crate::security::verify(&data)?;
    let _lock = runner::operation_lock()?;
    let current = runner::enrollment()?;
    if serde_json::to_value(enrollment).map_err(|e| e.to_string())?
        != serde_json::to_value(&current).map_err(|e| e.to_string())?
    {
        return Err("enrollment changed before admission; Refresh".into());
    }
    runner::verify_artifacts(&current)?;
    if windows_pipe::current_user_sid_string().map_err(|e| e.to_string())? != current.client_sid {
        return Err("operator enrollment changed before admission".into());
    }
    let audit_directory = data.join("audit");
    crate::security::verify(&audit_directory)?;
    let audit = audit_directory.join(format!("worker-{session}.json"));
    if fs::symlink_metadata(&audit).is_ok() {
        return Err("replayed worker session".into());
    }
    let operation_name = match &request.command {
        Command::ResetPartial { .. } => "ResetPartial",
        Command::ForgetPair { .. } => "ForgetPair",
        Command::EnrollPair { .. } => "EnrollPair",
        Command::ReconcilePair { .. } => "ReconcilePair",
        Command::Apply { .. } => "ApplyApproved",
        Command::Verify { .. } => "VerifyApproved",
        Command::Reconcile { .. } => "ReconcileObserved",
        Command::SaveOnly { .. } => "SaveOnly",
        Command::Import { .. } => "ImportConfiguration",
        Command::ImportResume { .. } => "ReconcileImport",
    };
    let binding = match &request.command {
        Command::ResetPartial {
            operation_id,
            record_revision,
            approved,
        } => {
            serde_json::json!({"original_operation_id":operation_id,"record_revision":record_revision,"review":approved})
        }
        Command::ForgetPair { approved } => {
            serde_json::to_value(approved).map_err(|e| e.to_string())?
        }
        Command::EnrollPair { approved } => {
            serde_json::to_value(approved).map_err(|e| e.to_string())?
        }
        Command::ReconcilePair {
            operation_id,
            expected,
        } => serde_json::json!({"original_operation_id":operation_id,"expected":expected}),
        Command::Apply {
            approved,
            expected,
            shutdown_approved,
        } => {
            serde_json::json!({"vm_id":approved.desired.vm_id,"gpu_interface":approved.desired.gpu_interface,"plan_revision":Revision::of(&serde_json::to_vec(approved).map_err(|e| e.to_string())?),"expected":expected,"shutdown_approved":shutdown_approved})
        }
        Command::SaveOnly { operation_id } => {
            serde_json::json!({"original_operation_id":operation_id})
        }
        Command::Reconcile {
            operation_id,
            shutdown_approved,
        } => {
            serde_json::json!({"original_operation_id":operation_id,"shutdown_approved":shutdown_approved})
        }
        Command::Verify {
            approved,
            shutdown_approved,
        } => {
            serde_json::json!({"vm_id":approved.target.vm_id,"gpu_interface":approved.target.gpu_interface,"plan_revision":Revision::of(&serde_json::to_vec(approved).map_err(|e| e.to_string())?),"shutdown_approved":shutdown_approved})
        }
        Command::Import { source, expected } => {
            serde_json::json!({"source_revision":Revision::of(source.as_bytes()),"expected_destinations":expected})
        }
        Command::ImportResume { operation_id } => {
            serde_json::json!({"original_operation_id":operation_id,"receipt":runner::import_record()?})
        }
    };
    runner::atomic_json(
        &audit,
        &serde_json::json!({"schema":1,"session":session,"operation":operation_name,"binding":binding,"outcome":"Started"}),
    )?;
    let result = (|| match request.command {
        Command::ResetPartial {
            operation_id,
            record_revision,
            approved,
        } => {
            if request.credential.is_some() {
                return Err("Partial GPU removal does not accept credentials".into());
            }
            let mut record =
                runner::recovery_record()?.ok_or("No interrupted operation to remove")?;
            if record.operation_id != operation_id
                || Revision::of(&serde_json::to_vec(&record).map_err(|e| e.to_string())?)
                    != record_revision
                || !enrollment.targets.iter().any(|t| {
                    t.vm_id == record.target.vm_id && t.gpu_interface == record.target.gpu_interface
                })
                || runner::pending_pair()?.is_some()
                || runner::import_record()?.is_some()
            {
                return Err("Partial GPU removal review changed; Refresh and review again".into());
            }
            crate::windows_wmi::reconcile_pending_operation()?;
            let revision = configuration_store::protected::revision(&record.target.vm_id)?;
            approved.validate(&record, &revision)?;
            if let Some(existing) = &record.reset {
                if serde_json::to_value(existing).map_err(|e| e.to_string())?
                    != serde_json::to_value(&approved).map_err(|e| e.to_string())?
                {
                    return Err("Partial GPU removal is bound to its original review".into());
                }
            }
            record.reset = Some(approved);
            runner::atomic_json(&runner::recovery_path()?, &record)?;
            runner::atomic_json(
                &audit_directory.join(format!("partial-removal-{}.json", operation_id)),
                &record,
            )?;
            let bound = serde_json::to_value(&record).map_err(|e| e.to_string())?;
            let mut check_binding = || {
                if serde_json::to_value(runner::recovery_record()?).map_err(|e| e.to_string())?
                    != serde_json::json!(bound)
                    || configuration_store::protected::revision(&record.target.vm_id)? != revision
                {
                    return Err("Partial GPU removal receipt changed; retain recovery".into());
                }
                crate::windows_wmi::require_no_pending_operation()
            };
            check_binding()?;
            let mut backend = NativeBackend {
                credential: None,
                manifest: None,
                data: data.clone(),
            };
            workflow::reset_partial(
                &mut ObservedBackend {
                    backend: &mut backend,
                    notify: &mut notify,
                    before_effect: Some(&mut check_binding),
                },
                &record,
                &revision,
            )?;
            if configuration_store::protected::revision(&record.target.vm_id)? != revision
                || serde_json::to_value(runner::recovery_record()?).map_err(|e| e.to_string())?
                    != serde_json::json!(bound)
            {
                return Err("Partial GPU removal final receipt changed; retain recovery".into());
            }
            crate::windows_wmi::require_no_pending_operation()?;
            Ok(Outcome {
                operation: None,
                saved: false,
                finished: true,
                operation_id,
                save_error: None,
            })
        }
        Command::ForgetPair { approved } => {
            if request.credential.is_some() {
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
        Command::EnrollPair { approved } => {
            if request.credential.is_some() {
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
        Command::ReconcilePair {
            operation_id,
            expected,
        } => {
            if request.credential.is_some() {
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
        Command::Import { source, expected } => {
            runner::require_no_recovery()?;
            crate::windows_wmi::require_no_pending_operation()?;
            let documents = Configuration::parse(&source)?.vm_documents()?;
            let ids = documents
                .iter()
                .map(|(filename, text)| {
                    Configuration::parse_vm_file(text, filename)
                        .map(|mut c| c.targets.remove(0).vm_id)
                })
                .collect::<Result<Vec<_>, _>>()?;
            configuration_store::protected::check_capacity(&ids)?;
            if documents.len() != expected.len() {
                return Err("import destination scope mismatch".into());
            }
            for (filename, text) in &documents {
                let target = Configuration::parse_vm_file(text, filename)?
                    .targets
                    .remove(0);
                if expected.get(filename) != Some(&Revision::Missing)
                    || configuration_store::protected::revision(&target.vm_id)? != Revision::Missing
                {
                    return Err(format!(
                        "import destination {filename} already exists or changed; explicit conflict resolution required"
                    ));
                }
            }
            let mut record = configuration_store::ImportRecord {
                schema: 1,
                operation_id: session.into(),
                source_revision: Revision::of(source.as_bytes()),
                documents: documents
                    .into_iter()
                    .map(|(filename, source)| {
                        (
                            filename,
                            configuration_store::ImportItem {
                                source,
                                published: false,
                            },
                        )
                    })
                    .collect(),
            };
            runner::atomic_json(&runner::import_path()?, &record)?;
            publish_import_record(&mut record, &enrollment.client_sid, false, &mut notify)?;
            Ok(Outcome {
                operation: None,
                saved: true,
                finished: true,
                operation_id: session.into(),
                save_error: None,
            })
        }
        Command::ImportResume { operation_id } => {
            if runner::recovery_record()?.is_some() {
                return Err("GPU recovery must remain distinct from import reconciliation".into());
            }
            let mut record = runner::import_record()?.ok_or("no interrupted import exists")?;
            if record.operation_id != operation_id {
                return Err("import operation identity mismatch".into());
            }
            publish_import_record(&mut record, &enrollment.client_sid, true, &mut notify)?;
            Ok(Outcome {
                operation: None,
                saved: true,
                finished: true,
                operation_id,
                save_error: None,
            })
        }
        Command::Apply {
            approved,
            expected,
            shutdown_approved,
        } => {
            runner::require_no_recovery()?;
            crate::windows_wmi::require_no_pending_operation()?;
            approved.desired.validate()?;
            if !enrollment.targets.iter().any(|target| {
                target.vm_id == approved.desired.vm_id
                    && target.gpu_interface == approved.desired.gpu_interface
            }) {
                return Err("selected pair requires administrator enrollment before Apply".into());
            }
            if approved.preview.guest_downtime && !shutdown_approved {
                return Err("graceful guest shutdown was not separately approved".into());
            }
            if configuration_store::protected::revision(&approved.desired.vm_id)? != expected {
                return Err("committed configuration changed before admission; Refresh".into());
            }
            if expected == Revision::Missing {
                configuration_store::protected::check_capacity(std::slice::from_ref(
                    &approved.desired.vm_id,
                ))?;
            }
            let mut backend = NativeBackend {
                credential: request.credential,
                manifest: None,
                data: data.clone(),
            };
            let (current, admitted) = workflow::admit_approved(&mut backend, &approved)?;
            let driver_version = admitted.driver_version().to_owned();
            let mut record = SaveRecord {
                schema: 1,
                retirement: None,
                reset: None,
                operation_id: session.into(),
                target: approved.desired.clone(),
                expected,
                driver_version: driver_version.clone(),
                payload_digest: current.preparation.as_ref().map(|p| p.digest.clone()),
                plan_revision: Revision::of(
                    &serde_json::to_vec(&current).map_err(|e| e.to_string())?,
                ),
                intended_source: Configuration {
                    schema: 3,
                    targets: vec![approved.desired.clone()],
                }
                .vm_documents()?
                .into_values()
                .next()
                .ok_or("missing output document")?,
                publication_required: true,
                initial: current.observed.clone(),
                expected_power: Some(current.preview.restore_power.clone()),
                graphics_required: Some(
                    current
                        .preview
                        .actions
                        .contains(&workflow::PreviewAction::Verify),
                ),
                initial_journal: current.managed.clone(),
                expected_settings: current
                    .preview
                    .settings
                    .as_ref()
                    .map_or_else(|| current.observed.settings.clone(), |s| s.after.clone()),
                phase: SavePhase::Admitted,
            };
            runner::atomic_json(&runner::recovery_path()?, &record)?;
            let mut mark_effects = || {
                if matches!(record.phase, SavePhase::Admitted) {
                    record.phase = SavePhase::EffectsStarted;
                    runner::atomic_json(&runner::recovery_path()?, &record)?;
                }
                Ok(())
            };
            let result = workflow::apply_admitted(
                &mut ObservedBackend {
                    backend: &mut backend,
                    notify: &mut notify,
                    before_effect: Some(&mut mark_effects),
                },
                admitted,
            )?;
            if serde_json::to_value(backend.inspect(&record.target)?).map_err(|e| e.to_string())?
                != serde_json::to_value(&result.effective).map_err(|e| e.to_string())?
            {
                return Err(
                    "final independent readback changed; manual reconciliation required".into(),
                );
            }
            if backend.gpu(&record.target)?.driver_version != driver_version {
                return Err("driver changed during effects; manual reconciliation required".into());
            }
            if let Some(digest) = &record.payload_digest
                && backend.payload(&record.target)? != *digest
            {
                return Err(
                    "signed payload changed during effects; manual reconciliation required".into(),
                );
            }
            record.phase = SavePhase::Verified(result.effective.clone());
            runner::atomic_json(&runner::recovery_path()?, &record)?;
            notify(Progress {
                stage: "Publish checked VM configuration".into(),
                status: StageStatus::Running,
            })?;
            let saved = configuration_store::protected::publish(&record, &enrollment.client_sid);
            if saved.is_ok() {
                record.phase = SavePhase::Published(result.effective.clone());
                runner::atomic_json(&runner::recovery_path()?, &record)?;
            }
            let delivery = notify(Progress {
                stage: "Publish checked VM configuration".into(),
                status: if saved.is_ok() {
                    StageStatus::Done
                } else {
                    StageStatus::Failed
                },
            });
            if let Err(delivery) = delivery {
                return Err(saved.err().map_or_else(
                    || format!("Publication completed but progress delivery failed; outcome uncertain: {delivery}"),
                    |primary| format!("Progress delivery failed: {delivery}; {primary}"),
                ));
            }
            Ok(Outcome {
                operation: Some(result),
                saved: saved.is_ok(),
                finished: saved.is_ok(),
                operation_id: record.operation_id,
                save_error: saved.err(),
            })
        }
        Command::Verify {
            approved,
            shutdown_approved,
        } => {
            runner::require_no_recovery()?;
            crate::windows_wmi::require_no_pending_operation()?;
            approved.target.validate()?;
            if approved.observed.power == Power::Off && !shutdown_approved {
                return Err("temporary guest start and graceful shutdown were not approved".into());
            }
            if !enrollment.targets.iter().any(|target| {
                target.vm_id == approved.target.vm_id
                    && target.gpu_interface == approved.target.gpu_interface
            }) {
                return Err("verification target is outside enrollment".into());
            }
            let mut backend = NativeBackend {
                credential: request.credential,
                manifest: None,
                data: data.clone(),
            };
            let admitted = workflow::admit_verification(&mut backend, &approved)?;
            let mut record = SaveRecord {
                schema: 1,
                retirement: None,
                reset: None,
                operation_id: session.into(),
                target: approved.target.clone(),
                expected: configuration_store::protected::revision(&approved.target.vm_id)?,
                driver_version: approved.gpu.driver_version.clone(),
                payload_digest: Some(approved.payload_digest.clone()),
                plan_revision: Revision::of(
                    &serde_json::to_vec(&approved).map_err(|e| e.to_string())?,
                ),
                intended_source: Configuration {
                    schema: 3,
                    targets: vec![approved.target.clone()],
                }
                .vm_documents()?
                .into_values()
                .next()
                .ok_or("missing receipt intent")?,
                publication_required: false,
                initial: approved.observed.clone(),
                expected_power: Some(workflow::verification_power(
                    &approved.target,
                    &approved.observed,
                    approved.managed.as_ref(),
                )?),
                graphics_required: Some(true),
                initial_journal: approved.managed.clone(),
                expected_settings: approved.observed.settings.clone(),
                phase: SavePhase::Admitted,
            };
            runner::atomic_json(&runner::recovery_path()?, &record)?;
            let mut mark_effects = || {
                if matches!(record.phase, SavePhase::Admitted) {
                    record.phase = SavePhase::EffectsStarted;
                    runner::atomic_json(&runner::recovery_path()?, &record)?;
                }
                Ok(())
            };
            let result = workflow::verify_admitted(
                &mut ObservedBackend {
                    backend: &mut backend,
                    notify: &mut notify,
                    before_effect: Some(&mut mark_effects),
                },
                admitted,
            )?;
            if serde_json::to_value(backend.inspect(&record.target)?).map_err(|e| e.to_string())?
                != serde_json::to_value(&record.initial).map_err(|e| e.to_string())?
                || serde_json::to_value(backend.gpu(&record.target)?).map_err(|e| e.to_string())?
                    != serde_json::to_value(&approved.gpu).map_err(|e| e.to_string())?
                || backend.payload(&record.target)? != approved.payload_digest
            {
                return Err(
                    "verification final facts changed; manual reconciliation required".into(),
                );
            }
            record.phase = SavePhase::Verified(result.effective.clone());
            runner::atomic_json(&runner::recovery_path()?, &record)?;
            Ok(Outcome {
                operation: Some(result),
                saved: false,
                finished: true,
                operation_id: record.operation_id,
                save_error: None,
            })
        }
        Command::Reconcile {
            operation_id,
            shutdown_approved,
        } => {
            let mut record =
                runner::recovery_record()?.ok_or("no durable operation requires reconciliation")?;
            if record.operation_id != operation_id {
                return Err("reconciliation operation identity mismatch".into());
            }
            if !enrollment.targets.iter().any(|t| {
                t.vm_id == record.target.vm_id && t.gpu_interface == record.target.gpu_interface
            }) {
                return Err("recorded pair is no longer enrolled".into());
            }
            if record.publication_required
                && matches!(
                    record.phase,
                    SavePhase::Verified(_) | SavePhase::Published(_)
                )
            {
                return Err(
                    "already verified: use save-only without repeating graphics or guest power"
                        .into(),
                );
            }
            let mut backend = NativeBackend {
                credential: request.credential,
                manifest: None,
                data: data.clone(),
            };
            let before = backend.inspect(&record.target)?;
            let negative = record.negative_completion(
                &before,
                &backend.gpu(&record.target)?.driver_version,
                &configuration_store::protected::revision(&record.target.vm_id)?,
                backend.journal(&record.target)?.as_ref(),
            );
            let retirement = if record.retirement.is_some() {
                negative?
            } else {
                // A noncandidate follows ordinary reconciliation, including WMI recovery.
                negative.ok().flatten()
            };
            if let Some(retirement) = retirement {
                crate::windows_wmi::require_no_pending_operation()?;
                // Persist the exact transition before its journal write. A lost reply or
                // interruption can resume only the bound before/after states.
                record.retirement = Some(retirement.clone());
                runner::atomic_json(&runner::recovery_path()?, &record)?;
                runner::atomic_json(
                    &audit_directory.join(format!("ended-{}.json", record.operation_id)),
                    &record,
                )?;
                record
                    .negative_completion(
                        &backend.inspect(&record.target)?,
                        &backend.gpu(&record.target)?.driver_version,
                        &configuration_store::protected::revision(&record.target.vm_id)?,
                        backend.journal(&record.target)?.as_ref(),
                    )?
                    .ok_or("negative completion proof unavailable; retain recovery")?;
                backend.save(&record.target, &retirement.after)?;
                record
                    .negative_completion(
                        &backend.inspect(&record.target)?,
                        &backend.gpu(&record.target)?.driver_version,
                        &configuration_store::protected::revision(&record.target.vm_id)?,
                        backend.journal(&record.target)?.as_ref(),
                    )?
                    .ok_or("negative completion readback unavailable; retain recovery")?;
                if serde_json::to_value(backend.journal(&record.target)?)
                    .map_err(|e| e.to_string())?
                    != serde_json::to_value(Some(&retirement.after)).map_err(|e| e.to_string())?
                {
                    return Err(
                        "negative completion journal publication not established; retain recovery"
                            .into(),
                    );
                }
                crate::windows_wmi::require_no_pending_operation()?;
                return Ok(Outcome {
                    operation: None,
                    saved: false,
                    finished: true,
                    operation_id,
                    save_error: None,
                });
            }
            if matches!(record.phase, SavePhase::EffectsStarted) && shutdown_approved {
                crate::windows_wmi::require_no_pending_operation()?;
                let journal = backend.journal(&record.target)?;
                if record
                    .validate_restored_baseline(
                        &before,
                        &backend.gpu(&record.target)?.driver_version,
                        &configuration_store::protected::revision(&record.target.vm_id)?,
                        journal.as_ref(),
                    )
                    .is_ok()
                {
                    // Archive the original uncertain attempt before retiring its journal.
                    // Partial guest files remain untrusted: there is no preparation receipt.
                    let archive =
                        audit_directory.join(format!("ended-{}.json", record.operation_id));
                    runner::atomic_json(
                        &archive,
                        &serde_json::json!({"schema":1,"disposition":"OriginalHostStateEstablished","record":record,"guest_files":"Unknown; retained without a preparation receipt"}),
                    )?;
                    record.validate_restored_baseline(
                        &backend.inspect(&record.target)?,
                        &backend.gpu(&record.target)?.driver_version,
                        &configuration_store::protected::revision(&record.target.vm_id)?,
                        backend.journal(&record.target)?.as_ref(),
                    )?;
                    if journal.is_some() {
                        let path = data.join(format!("{}.json", record.target.vm_id));
                        crate::security::verify(&path)?;
                        fs::remove_file(path).map_err(|e| e.to_string())?;
                    }
                    record.validate_restored_baseline(
                        &backend.inspect(&record.target)?,
                        &backend.gpu(&record.target)?.driver_version,
                        &configuration_store::protected::revision(&record.target.vm_id)?,
                        backend.journal(&record.target)?.as_ref(),
                    )?;
                    return Ok(Outcome {
                        operation: None,
                        saved: false,
                        finished: true,
                        operation_id,
                        save_error: None,
                    });
                }
            }
            if matches!(record.phase, SavePhase::Admitted) {
                // The wrapper marks EffectsStarted before every guest/Hyper-V
                // adapter, so this phase proves no such effect was dispatched.
                crate::windows_wmi::require_no_pending_operation()?;
                if serde_json::to_value(&before).map_err(|e| e.to_string())?
                    != serde_json::to_value(&record.initial).map_err(|e| e.to_string())?
                    || configuration_store::protected::revision(&record.target.vm_id)?
                        != record.expected
                {
                    return Err("pre-effect rejection facts changed; retain recovery".into());
                }
                let current_journal = backend.journal(&record.target)?;
                if current_journal.as_ref().is_some_and(|j| {
                    j.schema != 1
                        || j.vm_id != record.target.vm_id
                        || j.gpu_interface != record.target.gpu_interface
                }) {
                    return Err("journal identity changed during pre-effect rejection".into());
                }
                if let Some(original) = &record.initial_journal {
                    backend.save(&record.target, original)?;
                } else if current_journal.is_some() {
                    let path = data.join(format!("{}.json", record.target.vm_id));
                    crate::security::verify(&path)?;
                    fs::remove_file(path).map_err(|e| e.to_string())?;
                }
                return Ok(Outcome {
                    operation: None,
                    saved: false,
                    finished: true,
                    operation_id,
                    save_error: None,
                });
            }
            crate::windows_wmi::reconcile_pending_operation()?;
            if !record.publication_required
                && let SavePhase::Verified(verified) | SavePhase::Published(verified) =
                    &record.phase
            {
                if serde_json::to_value(&before).map_err(|e| e.to_string())?
                    != serde_json::to_value(verified).map_err(|e| e.to_string())?
                    || backend.gpu(&record.target)?.driver_version != record.driver_version
                    || backend.journal(&record.target)?.is_some_and(|j| j.pending)
                {
                    return Err("durably verified graphics receipt changed; retain recovery".into());
                }
                return Ok(Outcome {
                    operation: None,
                    saved: false,
                    finished: true,
                    operation_id,
                    save_error: None,
                });
            }
            record
                .validate_reconciliation(&before, &backend.gpu(&record.target)?.driver_version)?;
            let final_power = record.final_power()?;
            if let Some(digest) = &record.payload_digest
                && backend.payload(&record.target)? != *digest
            {
                return Err("current payload differs from the recorded operation".into());
            }
            let check_graphics = record.requires_graphics();
            if check_graphics
                && (before.power == Power::Off || final_power == Power::Off)
                && !shutdown_approved
            {
                return Err("explicit graceful guest shutdown/restoration approval is required for reconciliation".into());
            }
            let mut checked_target = record.target.clone();
            if !record.publication_required {
                checked_target.enabled = true;
                checked_target.clear_allocations();
            }
            let result = workflow::reconcile_with_graphics(
                &mut ObservedBackend {
                    backend: &mut backend,
                    notify: &mut notify,
                    before_effect: None,
                },
                &checked_target,
                &before,
                final_power,
                record.payload_digest.as_deref(),
                check_graphics,
            )?;
            record.validate_reconciliation(
                &backend.inspect(&record.target)?,
                &backend.gpu(&record.target)?.driver_version,
            )?;
            record.phase = SavePhase::Verified(result.effective.clone());
            runner::atomic_json(&runner::recovery_path()?, &record)?;
            if !record.publication_required {
                return Ok(Outcome {
                    operation: Some(result),
                    saved: false,
                    finished: true,
                    operation_id,
                    save_error: None,
                });
            }
            record.validate_save_only(
                &result.effective,
                &backend.gpu(&record.target)?.driver_version,
                &configuration_store::protected::revision(&record.target.vm_id)?,
            )?;
            notify(Progress {
                stage: "Publish checked VM configuration".into(),
                status: StageStatus::Running,
            })?;
            let saved = configuration_store::protected::publish(&record, &enrollment.client_sid);
            if saved.is_ok() {
                record.phase = SavePhase::Published(result.effective.clone());
                runner::atomic_json(&runner::recovery_path()?, &record)?;
            }
            let delivery = notify(Progress {
                stage: "Publish checked VM configuration".into(),
                status: if saved.is_ok() {
                    StageStatus::Done
                } else {
                    StageStatus::Failed
                },
            });
            if let Err(delivery) = delivery {
                return Err(saved.err().map_or_else(
                    || format!("Publication completed but progress delivery failed; outcome uncertain: {delivery}"),
                    |primary| format!("Progress delivery failed: {delivery}; {primary}"),
                ));
            }
            Ok(Outcome {
                operation: Some(result),
                saved: saved.is_ok(),
                finished: saved.is_ok(),
                operation_id,
                save_error: saved.err(),
            })
        }
        Command::SaveOnly { operation_id } => {
            crate::windows_wmi::require_no_pending_operation()?;
            let mut record =
                runner::recovery_record()?.ok_or("no verified-unsaved operation exists")?;
            if record.operation_id != operation_id {
                return Err("save-only operation identity mismatch".into());
            }
            if !enrollment.targets.iter().any(|target| {
                target.vm_id == record.target.vm_id
                    && target.gpu_interface == record.target.gpu_interface
            }) {
                return Err("save-only pair no longer enrolled".into());
            }
            let mut backend = NativeBackend {
                credential: None,
                manifest: None,
                data: data.clone(),
            };
            if backend
                .journal(&record.target)?
                .is_some_and(|journal| journal.pending)
            {
                return Err(
                    "unfinished managed operation requires manual reconciliation before saving"
                        .into(),
                );
            }
            if let Some(digest) = &record.payload_digest
                && backend.payload(&record.target)? != *digest
            {
                return Err(
                    "signed payload changed since verification; reconcile before saving".into(),
                );
            }
            record.validate_save_only(
                &backend.inspect(&record.target)?,
                &backend.gpu(&record.target)?.driver_version,
                &configuration_store::protected::revision(&record.target.vm_id)?,
            )?;
            notify(Progress {
                stage: "Save verified configuration only".into(),
                status: StageStatus::Running,
            })?;
            let publication =
                configuration_store::protected::publish(&record, &enrollment.client_sid);
            if publication.is_err() {
                let delivery = notify(Progress {
                    stage: "Save verified configuration only".into(),
                    status: StageStatus::Failed,
                });
                finish_progress(publication, delivery)?;
            }
            let verified = match &record.phase {
                SavePhase::Verified(state) | SavePhase::Published(state) => state.clone(),
                SavePhase::Admitted | SavePhase::EffectsStarted => {
                    return Err("manual reconciliation required".into());
                }
            };
            record.phase = SavePhase::Published(verified);
            runner::atomic_json(&runner::recovery_path()?, &record)?;
            notify(Progress {
                stage: "Save verified configuration only".into(),
                status: StageStatus::Done,
            })?;
            Ok(Outcome {
                operation: None,
                saved: true,
                finished: true,
                operation_id,
                save_error: None,
            })
        }
    })();
    runner::atomic_json(
        &audit,
        &serde_json::json!({"schema":1,"session":session,"operation":operation_name,"binding":binding,"outcome":if result.as_ref().is_ok_and(|o| o.finished) { "Succeeded" } else { "Failed" }}),
    )?;
    // Release hold last, after both publication and terminal audit are durable.
    if result.as_ref().is_ok_and(|o| o.finished) {
        let path = if matches!(
            operation_name,
            "EnrollPair" | "ReconcilePair" | "ForgetPair"
        ) {
            runner::pair_pending_path()?
        } else if operation_name == "ImportConfiguration" || operation_name == "ReconcileImport" {
            runner::import_path()?
        } else {
            runner::recovery_path()?
        };
        crate::security::verify(&path)?;
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    result
}
fn publish_import_record(
    record: &mut configuration_store::ImportRecord,
    client: &str,
    resume: bool,
    notify: &mut impl FnMut(Progress) -> Result<(), String>,
) -> Result<(), String> {
    record.validate()?;
    // Recheck the complete destination set before writing any remaining file.
    for (filename, item) in &record.documents {
        let target = Configuration::parse_vm_file(&item.source, filename)?
            .targets
            .remove(0);
        let revision = configuration_store::protected::revision(&target.vm_id)?;
        if resume {
            record.validate_destination(filename, &revision)?;
        } else if revision != Revision::Missing {
            return Err(
                "import destination appeared after admission; explicit reconciliation required"
                    .into(),
            );
        }
    }
    for filename in record.documents.keys().cloned().collect::<Vec<_>>() {
        let source = record.documents[&filename].source.clone();
        let target = Configuration::parse_vm_file(&source, &filename)?
            .targets
            .remove(0);
        let current = configuration_store::protected::revision(&target.vm_id)?;
        record.validate_destination(&filename, &current)?;
        notify(Progress {
            stage: format!("Import {filename}"),
            status: StageStatus::Running,
        })?;
        if current != Revision::of(source.as_bytes()) {
            configuration_store::protected::publish_import(&filename, &source, client)?;
        }
        if configuration_store::protected::revision(&target.vm_id)?
            != Revision::of(source.as_bytes())
        {
            return Err("import publication readback changed".into());
        }
        record
            .documents
            .get_mut(&filename)
            .ok_or("import destination disappeared")?
            .published = true;
        runner::atomic_json(&runner::import_path()?, record)?;
        notify(Progress {
            stage: format!("Import {filename}"),
            status: StageStatus::Done,
        })?;
    }
    Ok(())
}

struct ObservedBackend<'a, B, F> {
    backend: &'a mut B,
    notify: &'a mut F,
    before_effect: Option<&'a mut dyn FnMut() -> Result<(), String>>,
}

fn finish_progress<T>(
    result: Result<T, String>,
    delivery: Result<(), String>,
) -> Result<T, String> {
    match (result, delivery) {
        (Err(primary), Err(delivery)) => {
            Err(format!("Progress delivery failed: {delivery}; {primary}"))
        }
        (Err(primary), Ok(())) => Err(primary),
        (Ok(_), Err(delivery)) => Err(format!(
            "Stage completed but progress delivery failed; outcome uncertain: {delivery}"
        )),
        (Ok(value), Ok(())) => Ok(value),
    }
}
impl<B: Backend, F: FnMut(Progress) -> Result<(), String>> ObservedBackend<'_, B, F> {
    fn effect<T>(
        &mut self,
        name: &str,
        action: impl FnOnce(&mut B) -> Result<T, String>,
    ) -> Result<T, String> {
        if let Some(before) = &mut self.before_effect {
            before()?;
        }
        self.stage(name, action)
    }
    fn stage<T>(
        &mut self,
        name: &str,
        action: impl FnOnce(&mut B) -> Result<T, String>,
    ) -> Result<T, String> {
        (self.notify)(Progress {
            stage: name.into(),
            status: StageStatus::Running,
        })?;
        let result = crate::diagnostics::run(name, "approved-worker", || action(self.backend));
        let delivery = (self.notify)(Progress {
            stage: name.into(),
            status: if result.is_ok() {
                StageStatus::Done
            } else {
                StageStatus::Failed
            },
        });
        finish_progress(
            result.map_err(|error| crate::reporting::failure_at_stage(error, name)),
            delivery,
        )
    }
}
impl<B: Backend, F: FnMut(Progress) -> Result<(), String>> Backend for ObservedBackend<'_, B, F> {
    fn inspect(&mut self, t: &Target) -> Result<VmState, String> {
        self.backend.inspect(t)
    }
    fn gpu(&mut self, t: &Target) -> Result<Gpu, String> {
        self.backend.gpu(t)
    }
    fn payload(&mut self, t: &Target) -> Result<String, String> {
        self.backend.payload(t)
    }
    fn journal(&mut self, t: &Target) -> Result<Option<Journal>, String> {
        self.backend.journal(t)
    }
    fn save(&mut self, t: &Target, j: &Journal) -> Result<(), String> {
        self.stage("Persist operation recovery", |b| b.save(t, j))
    }
    fn power(&mut self, t: &Target, p: Power) -> Result<(), String> {
        self.effect(
            if p == Power::Off {
                "Graceful guest shutdown"
            } else {
                "Start guest"
            },
            |b| b.power(t, p),
        )
    }
    fn assign(&mut self, t: &Target, e: bool) -> Result<(), String> {
        self.effect(
            if e {
                "Attach selected GPU"
            } else {
                "Detach selected GPU"
            },
            |b| b.assign(t, e),
        )
    }
    fn settings(&mut self, t: &Target, s: &Settings) -> Result<(), String> {
        self.effect("Compatibility settings", |b| b.settings(t, s))
    }
    fn allocation(&mut self, t: &Target, resource: Resource, a: &Allocation) -> Result<(), String> {
        self.effect("Provider allocation", |b| b.allocation(t, resource, a))
    }
    fn prepare(&mut self, t: &Target) -> Result<String, String> {
        if let Some(before) = &mut self.before_effect {
            before()?;
        }
        (self.notify)(Progress {
            stage: "Prepare current signed payload".into(),
            status: StageStatus::Running,
        })?;
        let result =
            crate::diagnostics::run("Prepare current signed payload", "approved-worker", || {
                self.backend
                    .prepare_with_progress(t, &mut |bytes, total, files, count| {
                        (self.notify)(Progress {
                            stage: crate::reporting::transfer_progress(bytes, total, files, count),
                            status: StageStatus::Running,
                        })
                    })
            });
        let delivery = (self.notify)(Progress {
            stage: "Prepare current signed payload".into(),
            status: if result.is_ok() {
                StageStatus::Done
            } else {
                StageStatus::Failed
            },
        });
        finish_progress(
            result.map_err(|error| {
                crate::reporting::failure_at_stage(error, "Prepare current signed payload")
            }),
            delivery,
        )
    }
    fn verify(&mut self, t: &Target, g: &Gpu) -> Result<(), String> {
        self.effect("Check guest health and hardware rendering", |b| {
            b.verify(t, g)
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forget_terminal_reply_requires_complete_bound_removal_without_gpu_effect_result() {
        let plan = ordinary_disabled_plan();
        let approved = crate::enrollment::RemovalReview {
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
    fn failed_progress_delivery_preserves_the_adapter_failure_and_uncertainty() {
        let both = finish_progress::<()>(
            Err("adapter failed".into()),
            Err("pipe disconnected".into()),
        )
        .unwrap_err();
        assert!(both.contains("adapter failed") && both.contains("pipe disconnected"));
        let lost = finish_progress(Ok(()), Err("pipe disconnected".into())).unwrap_err();
        assert!(lost.contains("outcome uncertain"));
        assert!(finish_progress(Ok(()), Ok(())).is_ok());
    }

    fn ordinary_disabled_plan() -> Plan {
        let settings = Settings {
            low_mmio: 0,
            high_mmio: 0,
            cache_types: false,
            automatic_checkpoints: true,
        };
        let target = Target {
            vm_id: "12345678-1234-1234-1234-000000000001".into(),
            gpu_interface: "fixture-gpu".into(),
            enabled: false,
            vram: None,
            compute: None,
            encode: None,
            decode: None,
        };
        Plan {
            reinstall: false,
            initial_setup: false,
            desired: target.clone(),
            observed: VmState {
                vm_id: target.vm_id,
                name: "Same friendly name".into(),
                generation: 2,
                power: Power::Off,
                gpus: vec![],
                settings,
                vram: None,
                compute: None,
                encode: None,
                decode: None,
            },
            managed: None,
            preparation: None,
            preview: workflow::EffectPreview {
                actions: vec![],
                settings: None,
                settings_preserved: false,
                allocation: None,
                compute: None,
                encode: None,
                decode: None,
                credentials_required: false,
                guest_downtime: false,
                restore_power: Power::Off,
                pending_recovery: false,
                summary: vec![],
            },
            allocation_units: "provider-defined".into(),
            automatic_guest_restart: false,
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
    #[test]
    fn restricted_session_refuses_paths_and_arbitrary_modes() {
        for value in ["", "../escape", "a".repeat(33).as_str(), "123"] {
            assert!(pipe_name(value).is_err());
        }
        assert!(pipe_name("0123456789abcdef0123456789abcdef").is_ok());
        assert!(
            serde_json::from_str::<Request>(
                r#"{"schema":1,"session":"x","command":{"Shell":"whoami"},"credential":null}"#
            )
            .is_err()
        );
    }
}
