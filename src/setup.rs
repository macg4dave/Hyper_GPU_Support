//! Fixed first-install discovery/setup, using the UAC-approved package executable.
//! No caller-selected executable, source directory, command or destination path.
use crate::{
    enrollment::PairReview,
    gui_model::Inventory,
    model::Configuration,
    runner,
    windows_pipe::{self, session::Pipe},
    worker,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf, time::Duration};
use windows::{
    Win32::{
        Foundation::WAIT_OBJECT_0,
        System::{
            RemoteDesktop::ProcessIdToSessionId,
            Threading::{
                GetCurrentProcessId, GetProcessId, OpenProcess, PROCESS_NAME_WIN32,
                PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, QueryFullProcessImageNameW,
                WaitForSingleObject,
            },
        },
    },
    core::PWSTR,
};

const ARTIFACTS: [&str; 4] = [
    "hyper-gpu-support.exe",
    "hyper-gpu-runner.exe",
    "hyper-gpu-guest.exe",
    "d3d11-probe.exe",
];

/// Initiating operator identity from the local token, never from configuration.
pub fn operator_sid() -> Result<String, String> {
    windows_pipe::current_user_sid_string().map_err(|e| e.to_string())
}

/// Package executable selected by Windows UAC; paths are inferred from itself.
pub(crate) fn source_executable() -> Result<PathBuf, String> {
    let path = std::env::current_exe().map_err(|e| e.to_string())?;
    if path.file_name().and_then(|n| n.to_str()) != Some("hyper-gpu-support.exe") {
        return Err("setup must run from the hyper-gpu-support.exe package".into());
    }
    crate::payload::no_reparse(&path)?;
    Ok(path)
}

/// Read exact fixed sibling artifact pins for review; performs no installation.
pub fn source_artifacts() -> Result<BTreeMap<String, String>, String> {
    let path = source_executable()?;
    let root = path.parent().ok_or("package directory unavailable")?;
    ARTIFACTS
        .iter()
        .map(|name| {
            let path = root.join(name);
            crate::payload::no_reparse(&path)?;
            Ok(((*name).into(), crate::payload::hash_file(&path)?))
        })
        .collect()
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Command {
    Discover,
    Install {
        approved: PairReview,
        artifacts: BTreeMap<String, String>,
    },
    Resume {
        approved: PairReview,
        artifacts: BTreeMap<String, String>,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    session: String,
    operator_sid: String,
    command: Command,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Reply {
    Inventory(Inventory),
    Installed,
}

/// Read eligible native inventory before installation, with explicit UAC.
pub fn discover() -> Result<Inventory, String> {
    match submit(Command::Discover)? {
        Reply::Inventory(value) => Ok(value),
        _ => Err("invalid setup inventory reply".into()),
    }
}

/// Install only the exact reviewed initial pair from the fixed reviewed package.
pub fn install(approved: PairReview, artifacts: BTreeMap<String, String>) -> Result<(), String> {
    let command = if runner::pending_setup()?.is_some()
        && std::fs::symlink_metadata(runner::data_directory()?.join("enrollment.json")).is_ok()
    {
        Command::Resume {
            approved,
            artifacts,
        }
    } else {
        Command::Install {
            approved,
            artifacts,
        }
    };
    match submit(command)? {
        Reply::Installed => Ok(()),
        _ => Err("invalid setup installation reply".into()),
    }
}

#[allow(unsafe_code)]
fn submit(command: Command) -> Result<Reply, String> {
    use std::os::windows::fs::OpenOptionsExt;
    if runner::pending_setup()?.is_none() && runner::installed()? {
        return Err("product is installed; Refresh and use its protected worker".into());
    }
    // Hold fixed package files read-only across UAC and installation to prevent
    // unprivileged replacement after the review. Discovery needs only the main app.
    let path = source_executable()?;
    let root = path.parent().ok_or("package directory unavailable")?;
    let names = if matches!(command, Command::Install { .. } | Command::Resume { .. }) {
        &ARTIFACTS[..]
    } else {
        &ARTIFACTS[..1]
    };
    let _sources = names
        .iter()
        .map(|name| {
            std::fs::OpenOptions::new()
                .read(true)
                .share_mode(1)
                .open(root.join(name))
                .map_err(|e| format!("lock setup source {name}: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if let Command::Install { artifacts, .. } | Command::Resume { artifacts, .. } = &command
        && &source_artifacts()? != artifacts
    {
        return Err("package changed after setup review; review again".into());
    }
    let session = worker::random_session()?;
    let process = worker::launch_setup(&session)?;
    // SAFETY: retained process handle owns the launch identity through completion.
    let pid = unsafe { GetProcessId(process.0) };
    if pid == 0 {
        return Err("setup process identity unavailable".into());
    }
    let pipe = Pipe::connect(&worker::pipe_name(&session)?, "S-1-5-32-544", pid)
        .map_err(|e| e.to_string())?;
    let request = Request {
        session,
        operator_sid: windows_pipe::current_user_sid_string().map_err(|e| e.to_string())?,
        command,
    };
    pipe.send(&serde_json::to_vec(&request).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let bytes = pipe
        .receive(Duration::from_secs(600))
        .map_err(|e| format!("setup outcome uncertain; Refresh before retry: {e}"))?;
    serde_json::from_slice::<Result<Reply, String>>(&bytes).map_err(|e| e.to_string())?
}

/// Restricted first-install mode. Authenticate the same operator/session/process
/// and the actual running package; different-administrator UAC cannot rebind SID.
#[allow(unsafe_code)]
pub fn serve(session: &str, frontend: u32) -> Result<(), String> {
    if !worker::valid_session(session) || frontend == 0 || !crate::process::is_elevated()? {
        return Err("invalid fixed setup invocation".into());
    }
    let path = source_executable()?;
    let operator = windows_pipe::current_user_sid_string().map_err(|e| e.to_string())?;
    // SAFETY: query-only retained frontend handle; IPC separately authenticates PID/SID.
    let parent = worker::OwnedProcess(
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
    let mut name = vec![0_u16; 32768];
    let mut length = name.len() as u32;
    // SAFETY: retained handles, initialized output buffers and live writable sizes.
    unsafe {
        ProcessIdToSessionId(frontend, &mut their_session).map_err(|e| e.to_string())?;
        ProcessIdToSessionId(GetCurrentProcessId(), &mut our_session).map_err(|e| e.to_string())?;
        QueryFullProcessImageNameW(
            parent.0,
            PROCESS_NAME_WIN32,
            PWSTR(name.as_mut_ptr()),
            &mut length,
        )
        .map_err(|e| e.to_string())?;
    }
    let frontend_path =
        PathBuf::from(String::from_utf16(&name[..length as usize]).map_err(|e| e.to_string())?);
    if their_session != our_session
        || unsafe { WaitForSingleObject(parent.0, 0) } == WAIT_OBJECT_0
        || !frontend_path
            .as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(&path.as_os_str().to_string_lossy())
    {
        return Err("setup frontend must be the same running package/session".into());
    }
    let _deadline = crate::process::WorkerDeadline::start(Duration::from_secs(600));
    let pipe = Pipe::accept(&worker::pipe_name(session)?, &operator, frontend)
        .map_err(|e| e.to_string())?;
    let bytes = pipe
        .receive(Duration::from_secs(30))
        .map_err(|e| e.to_string())?;
    pipe.authenticate_client(&operator)
        .map_err(|e| e.to_string())?;
    let request: Request =
        serde_json::from_slice(&bytes).map_err(|_| "invalid bounded setup request")?;
    let result: Result<Reply, String> = (|| {
        if request.session != session || request.operator_sid != operator {
            return Err(
                "setup operator/session mismatch; use the initiating user's UAC token".into(),
            );
        }
        let pending = runner::pending_setup()?;
        if pending.is_none() && runner::installed()? {
            return Err("installation appeared; Refresh protected authority".into());
        }
        match request.command {
            Command::Discover => Ok(Reply::Inventory(Inventory {
                discovery: crate::windows_hyperv::discover()?,
                enrolled: vec![],
                managed: Default::default(),
            })),
            Command::Install {
                approved,
                artifacts,
            } => {
                if let Some(record) = &pending {
                    record.check_scope(&approved, &artifacts, &operator)?;
                }
                if approved.expected != crate::configuration_store::Revision::Missing
                    || approved.operator_sid != operator
                    || source_artifacts()? != artifacts
                {
                    return Err("setup review identities/artifacts changed".into());
                }
                let target = approved.target()?;
                let discovery = crate::windows_hyperv::discover()?;
                let view = {
                    let mut view = crate::gui_model::View::new(Configuration {
                        schema: 2,
                        targets: vec![],
                    });
                    view.refresh(Inventory {
                        discovery,
                        enrolled: vec![],
                        managed: Default::default(),
                    });
                    view
                };
                view.proposal_eligibility(&target)?;
                runner::bootstrap_install(
                    &Configuration {
                        schema: 2,
                        targets: vec![target],
                    },
                    &artifacts,
                    session,
                )?;
                Ok(Reply::Installed)
            }
            Command::Resume {
                approved,
                artifacts,
            } => {
                runner::reconcile_setup(&approved, &artifacts, session)?;
                Ok(Reply::Installed)
            }
        }
    })();
    pipe.send(&serde_json::to_vec(&result).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
