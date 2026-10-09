//! One restricted elevated instance of the product executable for an approved plan.
//! Private process-bound IPC; fixed paths, exact enrollment and durable admission.
use crate::{
    configuration_store::{self, Revision, SavePhase, SaveRecord},
    credentials::Credential,
    model::{Allocation, Configuration, Gpu, Power, Settings, Target, VmState},
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
#[derive(Clone, Debug, Serialize, Deserialize)]
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
        result: Result<Outcome, String>,
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
struct OwnedProcess(HANDLE);
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
fn valid_session(session: &str) -> bool {
    session.len() == 32
        && session
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn pipe_name(session: &str) -> Result<String, String> {
    if !valid_session(session) {
        return Err("invalid worker session identity".into());
    }
    Ok(format!(r"\\.\pipe\HyperGpuSupport.Operation.{session}"))
}
#[allow(unsafe_code)]
fn random_session() -> Result<String, String> {
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
#[allow(unsafe_code)]
fn launch(session: &str) -> Result<OwnedProcess, String> {
    let executable = trusted_executable()?;
    let verb = wide("runas");
    let path: Vec<u16> = executable
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let parameters = wide(&format!(
        "--internal-worker {session} {}",
        std::process::id()
    ));
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(path.as_ptr()),
        lpParameters: PCWSTR(parameters.as_ptr()),
        nShow: SW_HIDE.0,
        ..Default::default()
    };
    // SAFETY: only the protected installed executable, fixed internal mode and bounded numeric/session arguments.
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
            } if actual == session => return result,
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
            result,
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
        Command::Apply { .. } => "ApplyApproved",
        Command::Verify { .. } => "VerifyApproved",
        Command::Reconcile { .. } => "ReconcileObserved",
        Command::SaveOnly { .. } => "SaveOnly",
        Command::Import { .. } => "ImportConfiguration",
        Command::ImportResume { .. } => "ReconcileImport",
    };
    let binding = match &request.command {
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
            let gpu = backend.gpu(&approved.desired)?;
            let current = workflow::plan(&mut backend, &approved.desired)?;
            if serde_json::to_value(&approved).map_err(|e| e.to_string())?
                != serde_json::to_value(&current).map_err(|e| e.to_string())?
                || current.preview.pending_recovery
            {
                return Err("approved plan changed; Refresh and review before effects".into());
            }
            let mut record = SaveRecord {
                schema: 1,
                operation_id: session.into(),
                target: approved.desired.clone(),
                expected,
                driver_version: gpu.driver_version.clone(),
                payload_digest: current.preparation.as_ref().map(|p| p.digest.clone()),
                plan_revision: Revision::of(
                    &serde_json::to_vec(&current).map_err(|e| e.to_string())?,
                ),
                intended_source: Configuration {
                    schema: 2,
                    targets: vec![approved.desired.clone()],
                }
                .vm_documents()?
                .into_values()
                .next()
                .ok_or("missing output document")?,
                publication_required: true,
                initial: current.observed.clone(),
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
            let result = workflow::apply_approved(
                &mut ObservedBackend {
                    backend: &mut backend,
                    notify: &mut notify,
                    before_effect: Some(&mut mark_effects),
                },
                &approved,
            )?;
            if serde_json::to_value(backend.inspect(&record.target)?).map_err(|e| e.to_string())?
                != serde_json::to_value(&result.effective).map_err(|e| e.to_string())?
            {
                return Err(
                    "final independent readback changed; manual reconciliation required".into(),
                );
            }
            if backend.gpu(&record.target)?.driver_version != gpu.driver_version {
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
            let saved = configuration_store::protected::publish(&record, &enrollment.client_sid);
            if saved.is_ok() {
                record.phase = SavePhase::Published(result.effective.clone());
                runner::atomic_json(&runner::recovery_path()?, &record)?;
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
            let current = workflow::plan_verification(&mut backend, &approved.target)?;
            if serde_json::to_value(&current).map_err(|e| e.to_string())?
                != serde_json::to_value(&approved).map_err(|e| e.to_string())?
            {
                return Err("verification scope changed; Refresh and review".into());
            }
            let mut record = SaveRecord {
                schema: 1,
                operation_id: session.into(),
                target: approved.target.clone(),
                expected: configuration_store::protected::revision(&approved.target.vm_id)?,
                driver_version: approved.gpu.driver_version.clone(),
                payload_digest: None,
                plan_revision: Revision::of(
                    &serde_json::to_vec(&approved).map_err(|e| e.to_string())?,
                ),
                intended_source: Configuration {
                    schema: 2,
                    targets: vec![approved.target.clone()],
                }
                .vm_documents()?
                .into_values()
                .next()
                .ok_or("missing receipt intent")?,
                publication_required: false,
                initial: approved.observed.clone(),
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
            let result = workflow::verify_approved(
                &mut ObservedBackend {
                    backend: &mut backend,
                    notify: &mut notify,
                    before_effect: Some(&mut mark_effects),
                },
                &approved,
            )?;
            if serde_json::to_value(backend.inspect(&record.target)?).map_err(|e| e.to_string())?
                != serde_json::to_value(&record.initial).map_err(|e| e.to_string())?
                || serde_json::to_value(backend.gpu(&record.target)?).map_err(|e| e.to_string())?
                    != serde_json::to_value(&approved.gpu).map_err(|e| e.to_string())?
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
            if let Some(digest) = &record.payload_digest
                && backend.payload(&record.target)? != *digest
            {
                return Err("current payload differs from the recorded operation".into());
            }
            let check_graphics = record.target.enabled || !record.publication_required;
            if check_graphics
                && (before.power == Power::Off || record.initial.power == Power::Off)
                && !shutdown_approved
            {
                return Err("explicit graceful guest shutdown/restoration approval is required for reconciliation".into());
            }
            let mut checked_target = record.target.clone();
            if !record.publication_required {
                checked_target.enabled = true;
                checked_target.vram = None;
            }
            let result = workflow::reconcile_observed(
                &mut ObservedBackend {
                    backend: &mut backend,
                    notify: &mut notify,
                    before_effect: None,
                },
                &checked_target,
                &before,
                record.initial.power.clone(),
                record.payload_digest.as_deref(),
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
            let saved = configuration_store::protected::publish(&record, &enrollment.client_sid);
            if saved.is_ok() {
                record.phase = SavePhase::Published(result.effective.clone());
                runner::atomic_json(&runner::recovery_path()?, &record)?;
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
            configuration_store::protected::publish(&record, &enrollment.client_sid)?;
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
        let path = if operation_name == "ImportConfiguration" || operation_name == "ReconcileImport"
        {
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
        let result = action(self.backend);
        (self.notify)(Progress {
            stage: name.into(),
            status: if result.is_ok() {
                StageStatus::Done
            } else {
                StageStatus::Failed
            },
        })?;
        result
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
    fn allocation(&mut self, t: &Target, a: &Allocation) -> Result<(), String> {
        self.effect("Provider allocation", |b| b.allocation(t, a))
    }
    fn prepare(&mut self, t: &Target) -> Result<String, String> {
        self.effect("Prepare current signed payload", |b| b.prepare(t))
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
