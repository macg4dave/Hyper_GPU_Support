//! Authenticated one-shot client/server transport, admission and fixed request dispatch.
use super::{
    audit::{AuditRecord, run_audited},
    backend::NativeBackend,
    policy::{Enrollment, enrollment, verify_artifacts},
    profile_removal,
    protocol::{Operation, Reply, Request},
    recovery::require_no_vm_recovery,
    scheduler::{TASK, scheduler},
    storage::{atomic_json, data_directory, install_directory, operation_lock},
};
use crate::{process, windows_hyperv, windows_pipe, workflow::Backend};
use std::{fs, path::PathBuf, time::Duration};
use windows::{Win32::System::Variant::VARIANT, core::BSTR};

const PIPE: &str = r"\\.\pipe\HyperGpuSupport-Product-v2";
/// Submit a request through the installed authenticated runner.
#[allow(unsafe_code)]
pub fn submit(request: Request) -> Result<serde_json::Value, String> {
    let e = enrollment()?;
    if !(e.artifacts.len() == 3
        || (e.artifacts.len() == 4 && e.artifacts.contains_key("hyper-gpu-support.exe")))
        || ![
            "hyper-gpu-runner.exe",
            "hyper-gpu-guest.exe",
            "d3d11-probe.exe",
        ]
        .iter()
        .all(|name| e.artifacts.contains_key(*name))
    {
        return Err("incomplete installed artifact policy".into());
    }
    crate::security::verify(&install_directory()?)?;
    if windows_pipe::current_user_sid_string().map_err(|e| e.to_string())? != e.client_sid {
        return Err("current user is not the enrolled product operator".into());
    }
    windows_pipe::wait_for_task_ready(r"\", TASK, Duration::from_secs(30))
        .map_err(|e| e.to_string())?;
    let (_a, service) = scheduler()?;
    // SAFETY: fixed registered task; no caller-controlled executable or arguments.
    (|| unsafe {
        service
            .GetFolder(&BSTR::from(r"\"))?
            .GetTask(&BSTR::from(TASK))?
            .Run(&VARIANT::default())
    })()
    .map_err(|e: windows::core::Error| e.to_string())?;
    let bytes = zeroize::Zeroizing::new(serde_json::to_vec(&request).map_err(|e| e.to_string())?);
    let reply = windows_pipe::transact(PIPE, "S-1-5-18", &bytes, Duration::from_secs(3900))
        .map_err(|e| e.to_string())?;
    let reply: Reply = serde_json::from_slice(&reply).map_err(|e| e.to_string())?;
    if reply.nonce != request.nonce {
        return Err("runner response identity mismatch".into());
    }
    reply.result.ok_or_else(|| {
        reply
            .error
            .unwrap_or_else(|| "runner returned no result".into())
    })
}
/// Run exactly one authenticated request; parent task and watchdog bound its lifetime.
pub fn serve() -> Result<(), String> {
    let _deadline = process::WorkerDeadline::start(Duration::from_secs(3900));
    if windows_pipe::current_user_sid_string().map_err(|e| e.to_string())? != "S-1-5-18"
        || std::env::current_exe().map_err(|e| e.to_string())?
            != install_directory()?.join("hyper-gpu-runner.exe")
    {
        return Err("runner must execute from its protected installation as SYSTEM".into());
    }
    let e = enrollment()?;
    verify_artifacts(&e)?;
    // Specific client rights exclude FILE_CREATE_PIPE_INSTANCE, which generic
    // write would grant. Clients cannot create a competing server instance.
    let sddl = format!("O:SYG:SYD:P(A;;GA;;;SY)(A;;0x0012008b;;;{})", e.client_sid);
    windows_pipe::serve_one(PIPE, &e.client_sid, &sddl, |bytes| {
        let result = (|| {
            let request: Request =
                serde_json::from_slice(bytes).map_err(|_| "invalid product request")?;
            let nonce = request.nonce.clone();
            let result = execute(&e, request);
            Ok::<_, String>(Reply {
                nonce,
                result: result.as_ref().ok().cloned(),
                error: result.err(),
            })
        })();
        let reply = result.unwrap_or_else(|error| Reply {
            nonce: String::new(),
            result: None,
            error: Some(error),
        });
        serde_json::to_vec(&reply).map_err(std::io::Error::other)
    })
    .map_err(|e| e.to_string())
}
fn execute(e: &Enrollment, request: Request) -> Result<serde_json::Value, String> {
    if request.schema != 2
        || request.nonce.is_empty()
        || request.nonce.len() > 96
        || !request
            .nonce
            .bytes()
            .all(|b| b.is_ascii_digit() || b == b'-')
    {
        return Err("invalid protocol identity".into());
    }
    let issued = request
        .nonce
        .split('-')
        .next()
        .and_then(|s| s.parse::<u128>().ok())
        .ok_or("invalid request timestamp")?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    if issued > now || now - issued > 600_000_000_000 {
        return Err("expired request identity".into());
    }
    let data = data_directory()?;
    let _lock = operation_lock()?;
    let current = enrollment()?;
    if serde_json::to_value(e).map_err(|e| e.to_string())?
        != serde_json::to_value(&current).map_err(|e| e.to_string())?
    {
        return Err("enrollment changed before admission; retry discovery".into());
    }
    verify_artifacts(&current)?;
    let nonces = data.join("nonces");
    fs::create_dir_all(&nonces).map_err(|e| e.to_string())?;
    crate::security::verify(&nonces)?;
    for entry in fs::read_dir(&nonces).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        crate::security::verify(&path)?;
        if fs::metadata(&path)
            .and_then(|m| m.modified())
            .map_err(|e| e.to_string())?
            .elapsed()
            .is_ok_and(|age| age > Duration::from_secs(600))
        {
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(nonces.join(&request.nonce))
        .map_err(|_| "replayed product request")?;
    let audit = data.join("audit");
    fs::create_dir_all(&audit).map_err(|e| e.to_string())?;
    crate::security::verify(&audit)?;
    let path = audit.join(format!("{}.json", request.nonce));
    run_audited(
        AuditRecord::admission(&request),
        |record| atomic_json(&path, record),
        || execute_operation(e, request, data),
    )
}
fn execute_operation(
    e: &Enrollment,
    request: Request,
    data: PathBuf,
) -> Result<serde_json::Value, String> {
    if matches!(request.operation, Operation::Discover) {
        let mut inventory =
            serde_json::to_value(windows_hyperv::discover()?).map_err(|e| e.to_string())?;
        let mut reader = NativeBackend {
            credential: None,
            manifest: None,
            data,
        };
        let mut managed = std::collections::BTreeMap::new();
        for target in &e.targets {
            target.validate()?;
            match reader.journal(target) {
                Ok(journal) => {
                    managed.insert(target.vm_id.clone(), journal);
                }
                Err(error) => {
                    let issue = crate::reporting::Diagnostic::observation(
                        "Preparation record",
                        Some(target.vm_id.clone()),
                        &error,
                    );
                    inventory["issues"]
                        .as_array_mut()
                        .ok_or("invalid discovery issues")?
                        .push(serde_json::to_value(issue).map_err(|e| e.to_string())?);
                }
            }
        }
        inventory["managed"] = serde_json::to_value(managed).map_err(|e| e.to_string())?;
        inventory["enrolled"] = serde_json::to_value(&e.targets).map_err(|e| e.to_string())?;
        return Ok(inventory);
    }
    let target = request
        .target
        .ok_or("operation requires an enrolled target")?;
    target.validate()?;
    if !e.targets.iter().any(|t| {
        t.vm_id.eq_ignore_ascii_case(&target.vm_id) && t.gpu_interface == target.gpu_interface
    }) {
        return Err("target is outside administrator enrollment".into());
    }
    let mut backend = NativeBackend {
        credential: request.credential,
        manifest: None,
        data,
    };
    if matches!(request.operation, Operation::Apply | Operation::Verify) {
        require_no_vm_recovery(&target.vm_id)?;
        crate::windows_wmi::require_no_pending_operation_for(&target.vm_id)?;
        if backend
            .journal(&target)?
            .is_some_and(|journal| journal.pending)
        {
            return Err(
                "unfinished managed operation requires explicit manual reconciliation".into(),
            );
        }
    }
    let output = match request.operation {
        Operation::Discover => return Err("invalid operation".into()),
        Operation::Status => {
            let observed = backend.inspect(&target)?;
            let mut issues = Vec::new();
            let mut managed = std::collections::BTreeMap::new();
            match backend.journal(&target) {
                Ok(journal) => { managed.insert(target.vm_id.clone(), journal); }
                Err(error) => issues.push(crate::reporting::Diagnostic::observation("Preparation record", Some(target.vm_id.clone()), &error)),
            }
            let inventory = crate::gui_model::Inventory {
                discovery: crate::model::Discovery { guest_os: Default::default(), vms: vec![observed.clone()], gpus: vec![], issues: issues.clone() },
                managed,
                enrolled: e.targets.clone(),
            };
            Ok(serde_json::json!({"desired": target, "observed": observed, "managed": inventory.managed.get(&target.vm_id), "recorded": inventory.recorded_state(&target.vm_id), "issues": issues, "provenance": "Fresh Hyper-V read; desired is supplied configuration, not committed intent. Preparation and last graphics verification are historical; current guest health unknown."}))
        },
        Operation::Plan => serde_json::to_value(crate::workflow::plan(&mut backend, &target)?),
        Operation::PreliminaryPlan => serde_json::to_value(crate::workflow::plan_preliminary(&mut backend, &target)?),
        Operation::ReinstallPlan => serde_json::to_value(crate::workflow::plan_reinstall(&mut backend, &target)?),
        Operation::InstallationOs => {
            if !target.enabled || backend.credential.is_some() {
                return Err("OS preflight requires installation intent and no credentials".into());
            }
            backend.preflight_installation_os(&target)?;
            Ok(serde_json::Value::Null)
        },
        Operation::RemovalPlan => serde_json::to_value(profile_removal::plan(&target, &e.client_sid)?),
        Operation::VerifyPlan => serde_json::to_value(crate::workflow::plan_verification(&mut backend, &target)?),
        Operation::Apply | Operation::Verify => return Err("mutation requires the reviewed per-operation worker; update the frontend and installation".into()),
    }
    .map_err(|e| e.to_string())?;
    Ok(output)
}
