//! Protected runtime enrollment and a bounded authenticated one-shot product runner.
use crate::{
    credentials::Credential,
    model::*,
    payload, process, windows_hyperv, windows_pipe,
    workflow::{Backend, Journal},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};
use windows::{
    Win32::{
        Foundation::{HLOCAL, LocalFree},
        Security::Authorization::{
            ConvertStringSecurityDescriptorToSecurityDescriptorW, ConvertStringSidToSidW,
            SDDL_REVISION_1, SE_FILE_OBJECT, SetNamedSecurityInfoW,
        },
        Security::{
            DACL_SECURITY_INFORMATION, GetSecurityDescriptorDacl, OWNER_SECURITY_INFORMATION,
            PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID,
        },
        System::{
            Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, CoTaskMemFree},
            TaskScheduler::{
                ITaskService, TASK_CREATE_OR_UPDATE, TASK_LOGON_SERVICE_ACCOUNT, TaskScheduler,
            },
            Variant::VARIANT,
        },
        UI::Shell::{
            FOLDERID_ProgramData, FOLDERID_ProgramFiles, KNOWN_FOLDER_FLAG, SHGetKnownFolderPath,
        },
    },
    core::{BSTR, PCWSTR},
};

/// Fixed maximum authenticated protocol frame.
pub const FRAME_LIMIT: usize = 1024 * 1024;
const PIPE: &str = r"\\.\pipe\HyperGpuSupport-Product-v2";
const TASK: &str = "HyperGpuSupport-Product-v2";
/// Bounded protocol error.
#[derive(Debug)]
pub struct FrameError;
impl std::error::Error for FrameError {}
impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid runner frame")
    }
}
/// Read a bounded length-prefixed frame.
pub fn read_frame(r: &mut impl Read) -> Result<Vec<u8>, FrameError> {
    let mut h = [0; 4];
    r.read_exact(&mut h).map_err(|_| FrameError)?;
    let n = u32::from_le_bytes(h) as usize;
    if n > FRAME_LIMIT {
        return Err(FrameError);
    }
    let mut b = vec![0; n];
    r.read_exact(&mut b).map_err(|_| FrameError)?;
    Ok(b)
}
/// Publish one bounded frame.
pub fn write_frame(w: &mut impl Write, b: &[u8]) -> Result<(), FrameError> {
    if b.len() > FRAME_LIMIT {
        return Err(FrameError);
    }
    w.write_all(&(b.len() as u32).to_le_bytes())
        .and_then(|()| w.write_all(b))
        .and_then(|()| w.flush())
        .map_err(|_| FrameError)
}
/// Fixed product operations. No arbitrary scripts, source paths or host lifecycle commands.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Operation {
    /// Enumerate host GPUs and existing VMs.
    Discover,
    /// Inspect an enrolled target.
    Status,
    /// Inspect desired changes and authenticate the current preparation payload without effects.
    Plan,
    /// Read-only preview of the guest verification lifecycle.
    VerifyPlan,
    /// Apply desired GPU state.
    Apply,
    /// Verify selected guest health and rendering.
    Verify,
}
/// Authenticated request. Credentials never enter journal/audit output.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    /// Protocol version.
    pub schema: u32,
    /// Unique request identity, consumed before mutation.
    pub nonce: String,
    /// Fixed operation.
    pub operation: Operation,
    /// Selected target, checked against protected enrollment.
    pub target: Option<Target>,
    /// Ephemeral guest credential, obtained by the unprivileged client.
    pub credential: Option<Credential>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Enrollment {
    schema: u32,
    pub(crate) client_sid: String,
    pub(crate) targets: Vec<Target>,
    pub(crate) artifacts: std::collections::BTreeMap<String, String>,
}
#[derive(Serialize, Deserialize)]
struct Reply {
    nonce: String,
    result: Option<serde_json::Value>,
    error: Option<String>,
}
#[derive(Serialize, Deserialize)]
enum AuditOutcome {
    Started,
    Succeeded,
    Failed,
}
// Deliberately excludes Request::credential, error text and response payloads.
// Failures retain their full error in the authenticated reply, not persistent logs.
#[derive(Serialize, Deserialize)]
struct AuditRecord {
    schema: u32,
    nonce: String,
    operation: Operation,
    target: Option<Target>,
    outcome: AuditOutcome,
}
impl AuditRecord {
    fn admission(request: &Request) -> Self {
        Self {
            schema: 1,
            nonce: request.nonce.clone(),
            operation: request.operation,
            target: request
                .target
                .as_ref()
                .filter(|t| t.validate().is_ok())
                .cloned(),
            outcome: AuditOutcome::Started,
        }
    }
}
fn run_audited<T>(
    mut record: AuditRecord,
    mut publish: impl FnMut(&AuditRecord) -> Result<(), String>,
    operation: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    publish(&record)?;
    let result = operation();
    record.outcome = if result.is_ok() {
        AuditOutcome::Succeeded
    } else {
        AuditOutcome::Failed
    };
    if let Err(audit_error) = publish(&record) {
        return Err(match result {
            Ok(_) => format!(
                "operation completed but terminal audit publication failed: {audit_error}; inspect state before retry"
            ),
            Err(error) => format!(
                "{error}; terminal audit publication failed: {audit_error}; inspect state before retry"
            ),
        });
    }
    result
}
#[allow(unsafe_code)]
pub(crate) fn folder(id: &windows::core::GUID) -> Result<PathBuf, String> {
    // SAFETY: known-folder GUID, current token, API-owned returned string.
    let p = unsafe { SHGetKnownFolderPath(id, KNOWN_FOLDER_FLAG(0), None) }
        .map_err(|e| e.to_string())?;
    // SAFETY: terminated result valid until CoTaskMemFree.
    let text = unsafe { p.to_string() }.map_err(|e| e.to_string());
    unsafe {
        CoTaskMemFree(Some(p.0.cast()));
    }
    text.map(PathBuf::from)
}
/// Discovered protected product installation path.
pub fn install_directory() -> Result<PathBuf, String> {
    Ok(folder(&FOLDERID_ProgramFiles)?.join("HyperGpuSupportProduct"))
}
/// Discovered administrator-owned product state path.
pub fn data_directory() -> Result<PathBuf, String> {
    Ok(folder(&FOLDERID_ProgramData)?.join("HyperGpuSupportProduct"))
}
#[allow(unsafe_code)]
pub(crate) fn protect(path: &Path, client: &str) -> Result<(), String> {
    payload::no_reparse(path)?;
    let sddl: Vec<_> = format!("D:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)(A;OICI;FRFX;;;{client})")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut descriptor = PSECURITY_DESCRIPTOR::default();
    // SAFETY: terminated SDDL and writable LocalAlloc descriptor output.
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            PCWSTR(sddl.as_ptr()),
            SDDL_REVISION_1,
            &mut descriptor,
            None,
        )
    }
    .map_err(|e| e.to_string())?;
    let mut owner = PSID::default();
    let admin: Vec<_> = "S-1-5-32-544".encode_utf16().chain(Some(0)).collect();
    // SAFETY: terminated trusted SID constant and LocalAlloc output.
    if let Err(e) = unsafe { ConvertStringSidToSidW(PCWSTR(admin.as_ptr()), &mut owner) } {
        unsafe {
            LocalFree(Some(HLOCAL(descriptor.0)));
        }
        return Err(e.to_string());
    }
    let result = (|| {
        let mut present = windows::core::BOOL(0);
        let mut defaulted = windows::core::BOOL(0);
        let mut acl = std::ptr::null_mut();
        // SAFETY: live security descriptor, writable ACL outputs.
        unsafe { GetSecurityDescriptorDacl(descriptor, &mut present, &mut acl, &mut defaulted) }
            .map_err(|e| e.to_string())?;
        let text: Vec<_> = path
            .as_os_str()
            .to_string_lossy()
            .encode_utf16()
            .chain(Some(0))
            .collect();
        // SAFETY: live ACL and terminated existing path; installs a protected DACL.
        unsafe {
            SetNamedSecurityInfoW(
                PCWSTR(text.as_ptr()),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION
                    | DACL_SECURITY_INFORMATION
                    | PROTECTED_DACL_SECURITY_INFORMATION,
                Some(owner),
                None,
                Some(acl),
                None,
            )
        }
        .ok()
        .map_err(|e| e.to_string())
    })();
    // SAFETY: release sole SDDL allocation after ACL is copied.
    unsafe {
        LocalFree(Some(HLOCAL(descriptor.0)));
        LocalFree(Some(HLOCAL(owner.0)));
    }
    result?;
    crate::security::verify(path)
}
#[allow(unsafe_code)]
fn scheduler() -> Result<(crate::windows_com::Apartment, ITaskService), String> {
    let apartment = crate::windows_com::Apartment::initialize().map_err(|e| e.to_string())?;
    // SAFETY: initialized COM apartment, no aggregation.
    let service: ITaskService =
        unsafe { CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER) }
            .map_err(|e| e.to_string())?;
    let empty = VARIANT::default();
    // SAFETY: empty variants connect to local scheduler using the current token.
    unsafe { service.Connect(&empty, &empty, &empty, &empty) }.map_err(|e| e.to_string())?;
    Ok((apartment, service))
}
fn xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
fn validate_enrollment(configuration: &Configuration, discovery: &Discovery) -> Result<(), String> {
    configuration.validate()?;
    for target in &configuration.targets {
        let vm = discovery
            .vms
            .iter()
            .find(|vm| vm.vm_id.eq_ignore_ascii_case(&target.vm_id))
            .ok_or_else(|| {
                format!(
                    "enrollment VM {} is absent from current native discovery",
                    target.vm_id
                )
            })?;
        if vm.generation != 2 {
            return Err(format!(
                "enrollment VM {} must be Hyper-V Generation 2",
                target.vm_id
            ));
        }
        if !discovery
            .gpus
            .iter()
            .any(|gpu| gpu.interface == target.gpu_interface)
        {
            return Err(format!(
                "enrollment GPU for VM {} is absent from current native discovery",
                target.vm_id
            ));
        }
    }
    Ok(())
}
/// Administrator-only native installation/enrollment of selected existing VMs.
#[allow(unsafe_code)]
pub fn install(configuration: &Configuration) -> Result<(), String> {
    if !process::is_elevated()? {
        return Err("runner installation requires an elevated administrator console".into());
    }
    configuration.validate()?;
    let client = windows_pipe::current_user_sid_string().map_err(|e| e.to_string())?;
    let discovery = windows_hyperv::discover()?;
    validate_enrollment(configuration, &discovery)?;
    let install = install_directory()?;
    let data = data_directory()?;
    for p in [&install, &data] {
        if p.exists() {
            crate::security::verify(p)?;
        } else {
            fs::create_dir(p).map_err(|e| e.to_string())?;
        }
        protect(p, &client)?;
    }
    let source = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("missing executable directory")?
        .to_path_buf();
    // Disable admission before publishing any installation changes. Existing work
    // is allowed to finish; installation never kills a guest operation.
    let (_apartment, service) = scheduler()?;
    let folder = unsafe { service.GetFolder(&BSTR::from(r"\")) }.map_err(|e| e.to_string())?;
    match unsafe { folder.GetTask(&BSTR::from(TASK)) } {
        Ok(task) => {
            unsafe { task.SetEnabled(windows::Win32::Foundation::VARIANT_BOOL(0)) }
                .map_err(|e| e.to_string())?;
            if unsafe { task.GetInstances(0).and_then(|instances| instances.Count()) }
                .map_err(|e| e.to_string())?
                != 0
            {
                return Err("runner admission disabled; an operation is still active. Retry installation after it finishes".into());
            }
        }
        Err(error) if error.code().0 == 0x80070002_u32 as i32 => {}
        Err(error) => return Err(error.to_string()),
    }
    let _lock = operation_lock()?;
    atomic_json(
        &data.join("installation-pending.json"),
        &serde_json::json!({"schema": 1}),
    )?;
    crate::configuration_store::protected::install(&client)?;
    let mut artifacts = std::collections::BTreeMap::new();
    for name in [
        "hyper-gpu-runner.exe",
        "hyper-gpu-guest.exe",
        "d3d11-probe.exe",
        "hyper-gpu-support.exe",
    ] {
        let from = source.join(name);
        payload::no_reparse(&from)?;
        let hash = payload::hash_file(&from)?;
        let to = install.join(name);
        if to.exists() {
            crate::security::verify(&to)?;
        }
        let staged = install.join(format!("{name}.partial"));
        remove_protected_leaf(&staged)?;
        let mut input = fs::File::open(&from).map_err(|e| e.to_string())?;
        let mut output = crate::security::create_file(&staged, &client)?;
        std::io::copy(&mut input, &mut output)
            .and_then(|_| output.sync_all())
            .map_err(|e| e.to_string())?;
        drop(output);
        protect(&staged, &client)?;
        if payload::hash_file(&staged)? != hash {
            return Err("installed artifact copy mismatch".into());
        }
        crate::guest::replace_file(&staged, &to)?;
        artifacts.insert(name.into(), hash);
    }
    let enrollment = Enrollment {
        schema: 2,
        client_sid: client.clone(),
        targets: configuration.targets.clone(),
        artifacts,
    };
    atomic_json(&data.join("enrollment.json"), &enrollment)?;
    protect(&data.join("enrollment.json"), &client)?;
    // SAFETY: local scheduler, fixed task path and finite administrator-generated XML.
    let folder = unsafe { service.GetFolder(&BSTR::from(r"\")) }.map_err(|e| e.to_string())?;
    let executable = xml(&install.join("hyper-gpu-runner.exe").to_string_lossy());
    let task = format!(
        r#"<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task"><Principals><Principal id="Runner"><UserId>S-1-5-18</UserId><RunLevel>HighestAvailable</RunLevel></Principal></Principals><Settings><MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy><ExecutionTimeLimit>PT65M</ExecutionTimeLimit><AllowStartOnDemand>true</AllowStartOnDemand><DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries><StopIfGoingOnBatteries>false</StopIfGoingOnBatteries></Settings><Actions Context="Runner"><Exec><Command>{executable}</Command></Exec></Actions></Task>"#
    );
    let sddl = VARIANT::from(BSTR::from(format!(
        "D:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;GRGX;;;{client})"
    )));
    unsafe {
        folder.RegisterTask(
            &BSTR::from(TASK),
            &BSTR::from(task),
            TASK_CREATE_OR_UPDATE.0,
            &VARIANT::from(BSTR::from("SYSTEM")),
            &VARIANT::default(),
            TASK_LOGON_SERVICE_ACCOUNT,
            &sddl,
        )
    }
    .map_err(|e| e.to_string())?;
    fs::remove_file(data.join("installation-pending.json")).map_err(|e| e.to_string())?;
    Ok(())
}
pub(crate) fn enrollment() -> Result<Enrollment, String> {
    let root = data_directory()?;
    crate::security::verify(&root)?;
    if root.join("installation-pending.json").exists() {
        return Err(
            "runner installation was interrupted; rerun administrator install before operations"
                .into(),
        );
    }
    let path = root.join("enrollment.json");
    crate::security::verify(&path)?;
    let b = fs::read(&path)
        .map_err(|_| "product runner is not installed; use install with administrator rights")?;
    if b.len() > FRAME_LIMIT {
        return Err("invalid enrollment size".into());
    }
    let value: Enrollment = serde_json::from_slice(&b).map_err(|e| e.to_string())?;
    if value.schema != 2 {
        return Err("unsupported enrollment version".into());
    }
    if !(value.artifacts.len() == 3
        || (value.artifacts.len() == 4 && value.artifacts.contains_key("hyper-gpu-support.exe")))
        || ![
            "hyper-gpu-runner.exe",
            "hyper-gpu-guest.exe",
            "d3d11-probe.exe",
        ]
        .iter()
        .all(|name| value.artifacts.contains_key(*name))
    {
        return Err("incomplete installed artifact policy".into());
    }
    Ok(value)
}
pub(crate) fn atomic_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    crate::security::verify(path.parent().ok_or("state directory missing")?)?;
    let temp = path.with_extension("partial");
    remove_protected_leaf(&temp)?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let client = enrollment()
        .map(|e| e.client_sid)
        .or_else(|_| windows_pipe::current_user_sid_string().map_err(|e| e.to_string()))?;
    let mut file = crate::security::create_file(&temp, &client)?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())?;
    drop(file);
    crate::security::verify(&temp)?;
    if path.exists() {
        crate::security::verify(path)?;
    }
    crate::guest::replace_file(&temp, path)
}
pub(crate) fn operation_lock() -> Result<fs::File, String> {
    use std::os::windows::fs::OpenOptionsExt;
    let data = data_directory()?;
    crate::security::verify(&data)?;
    let path = data.join("operation.lock");
    match fs::symlink_metadata(&path) {
        Ok(_) => {
            crate::security::verify(&path)?;
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .share_mode(0)
                .open(path)
                .map_err(|e| format!("another host operation is active: {e}"))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let client = enrollment()
                .map(|e| e.client_sid)
                .or_else(|_| windows_pipe::current_user_sid_string().map_err(|e| e.to_string()))?;
            crate::security::create_file(&path, &client)
        }
        Err(error) => Err(error.to_string()),
    }
}
pub(crate) fn verify_artifacts(e: &Enrollment) -> Result<(), String> {
    for (name, hash) in &e.artifacts {
        if ![
            "hyper-gpu-runner.exe",
            "hyper-gpu-guest.exe",
            "d3d11-probe.exe",
            "hyper-gpu-support.exe",
        ]
        .contains(&name.as_str())
        {
            return Err("unrecognized installed product artifact".into());
        }
        let path = install_directory()?.join(name);
        crate::security::verify(&path)?;
        if payload::hash_file(&path)? != *hash {
            return Err("installed product artifact changed".into());
        }
    }
    Ok(())
}
fn remove_protected_leaf(path: &Path) -> Result<(), String> {
    if path.exists() {
        crate::security::verify(path)?;
        if !fs::metadata(path).map_err(|e| e.to_string())?.is_file() {
            return Err("protected temporary path is not a file".into());
        }
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
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
/// Create a unique request with no credential persistence or logging.
pub fn request(
    operation: Operation,
    target: Option<Target>,
    credential: Option<Credential>,
) -> Request {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    Request {
        schema: 2,
        nonce: format!(
            "{time}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ),
        operation,
        target,
        credential,
    }
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
        require_no_recovery()?;
        crate::windows_wmi::require_no_pending_operation()?;
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
                discovery: crate::model::Discovery { vms: vec![observed.clone()], gpus: vec![], issues: issues.clone() },
                managed,
                enrolled: e.targets.clone(),
            };
            Ok(serde_json::json!({"desired": target, "observed": observed, "managed": inventory.managed.get(&target.vm_id), "recorded": inventory.recorded_state(&target.vm_id), "issues": issues, "provenance": "Fresh Hyper-V read; desired is supplied configuration, not committed intent. Preparation and last graphics verification are historical; current guest health unknown."}))
        },
        Operation::Plan => serde_json::to_value(crate::workflow::plan(&mut backend, &target)?),
        Operation::VerifyPlan => serde_json::to_value(crate::workflow::plan_verification(&mut backend, &target)?),
        Operation::Apply | Operation::Verify => return Err("mutation requires the reviewed per-operation worker; update the frontend and installation".into()),
    }
    .map_err(|e| e.to_string())?;
    Ok(output)
}
pub(crate) struct NativeBackend {
    pub(crate) credential: Option<Credential>,
    pub(crate) manifest: Option<payload::Manifest>,
    pub(crate) data: PathBuf,
}
pub(crate) fn recovery_path() -> Result<PathBuf, String> {
    Ok(data_directory()?.join("operation-recovery.json"))
}
pub(crate) fn import_path() -> Result<PathBuf, String> {
    Ok(data_directory()?.join("configuration-import.json"))
}
/// Read import recovery without accepting it as enrollment or changing any file.
pub fn import_record() -> Result<Option<crate::configuration_store::ImportRecord>, String> {
    let path = import_path()?;
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
        Ok(metadata) if !metadata.is_file() => {
            return Err("import receipt is not a regular file".into());
        }
        Ok(_) => {}
    }
    crate::security::verify(&path)?;
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(FRAME_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > FRAME_LIMIT {
        return Err("import recovery input limit exceeded".into());
    }
    let record: crate::configuration_store::ImportRecord =
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    record.validate()?;
    Ok(Some(record))
}
/// Read the host-wide protected operation hold without modifying or clearing it.
pub fn recovery_record() -> Result<Option<crate::configuration_store::SaveRecord>, String> {
    let path = recovery_path()?;
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
        Ok(metadata) if !metadata.is_file() => return Err("recovery record is not a file".into()),
        Ok(_) => {}
    }
    crate::security::verify(&path)?;
    let mut bytes = Vec::new();
    fs::File::open(&path)
        .map_err(|e| e.to_string())?
        .take(FRAME_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > FRAME_LIMIT {
        return Err("recovery record exceeds input limit".into());
    }
    let record: crate::configuration_store::SaveRecord = serde_json::from_slice(&bytes)
        .map_err(|e| format!("invalid protected recovery record: {e}"))?;
    if record.schema != 1 {
        return Err("unsupported protected recovery version".into());
    }
    record.target.validate()?;
    Ok(Some(record))
}
pub(crate) fn require_no_recovery() -> Result<(), String> {
    if recovery_record()?.is_some() {
        return Err("host-wide recovery hold is active; reconcile or retry saving only".into());
    }
    if import_record()?.is_some() {
        return Err(
            "interrupted configuration import requires explicit import reconciliation".into(),
        );
    }
    Ok(())
}
impl Backend for NativeBackend {
    fn inspect(&mut self, t: &Target) -> Result<VmState, String> {
        windows_hyperv::inspect(&t.vm_id)
    }
    fn gpu(&mut self, t: &Target) -> Result<Gpu, String> {
        windows_hyperv::selected_gpu(t)
    }
    fn payload(&mut self, t: &Target) -> Result<String, String> {
        let manifest = process::background_work(|| {
            let discovery =
                crate::windows_driver::discover_driver_environment(t, Duration::from_secs(300))
                    .map_err(|e| e.to_string())?;
            let manifest = payload::discover(
                &crate::windows_paths::windows_directory().map_err(|e| e.to_string())?,
                discovery,
            )?;
            crate::guest::validate_trust(&manifest)?;
            Ok(manifest)
        })?;
        let digest = manifest.digest()?;
        self.manifest = Some(manifest);
        Ok(digest)
    }
    fn journal(&mut self, t: &Target) -> Result<Option<Journal>, String> {
        let path = self
            .data
            .join(format!("{}.json", t.vm_id.to_ascii_lowercase()));
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.to_string()),
            Ok(metadata) if !metadata.is_file() => {
                return Err("journal is not a regular file".into());
            }
            Ok(_) => {}
        }
        crate::security::verify(&path)?;
        let mut b = Vec::new();
        fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(65537)
            .read_to_end(&mut b)
            .map_err(|e| e.to_string())?;
        if b.len() > 65536 {
            return Err("invalid journal size".into());
        }
        serde_json::from_slice(&b)
            .map(Some)
            .map_err(|e| e.to_string())
    }
    fn save(&mut self, t: &Target, j: &Journal) -> Result<(), String> {
        atomic_json(
            &self
                .data
                .join(format!("{}.json", t.vm_id.to_ascii_lowercase())),
            j,
        )
    }
    fn power(&mut self, t: &Target, p: Power) -> Result<(), String> {
        windows_hyperv::power(t, p)
    }
    fn assign(&mut self, t: &Target, e: bool) -> Result<(), String> {
        windows_hyperv::assign(t, e)
    }
    fn settings(&mut self, t: &Target, s: &Settings) -> Result<(), String> {
        windows_hyperv::configure(t, s)
    }
    fn allocation(&mut self, t: &Target, v: &Allocation) -> Result<(), String> {
        windows_hyperv::allocation(t, v)
    }
    fn prepare(&mut self, t: &Target) -> Result<String, String> {
        crate::guest::prepare(
            t,
            self.manifest
                .as_ref()
                .ok_or("payload must be discovered before preparation")?,
            self.credential
                .as_ref()
                .ok_or("guest credentials required")?,
        )
    }
    fn verify(&mut self, t: &Target, g: &Gpu) -> Result<(), String> {
        crate::guest::verify(
            t,
            g,
            self.credential
                .as_ref()
                .ok_or("guest credentials required")?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_admission_precedes_work_and_terminal_outcome_excludes_secrets() {
        for failed in [false, true] {
            let (configuration, _) = fixture();
            let request = request(
                Operation::Status,
                Some(configuration.targets[0].clone()),
                Some(Credential {
                    username: "private-user".into(),
                    password: "private-password".into(),
                }),
            );
            let published = std::cell::RefCell::new(Vec::new());
            let result = run_audited(
                AuditRecord::admission(&request),
                |record| {
                    published
                        .borrow_mut()
                        .push(serde_json::to_value(record).unwrap());
                    Ok(())
                },
                || {
                    assert_eq!(published.borrow()[0]["outcome"], "Started");
                    if failed {
                        Err("private-password provider error".into())
                    } else {
                        Ok("private-user response")
                    }
                },
            );
            assert_eq!(result.is_err(), failed);
            let values = published.into_inner();
            assert_eq!(values.len(), 2);
            assert_eq!(
                values[1]["outcome"],
                if failed { "Failed" } else { "Succeeded" }
            );
            let bytes = serde_json::to_string(&values).unwrap();
            assert!(!bytes.contains("private-"));
            assert!(!bytes.contains("credential"));
        }
    }

    #[test]
    fn audit_publication_failure_never_runs_unrecorded_work_or_reports_false_success() {
        let request = request(Operation::Discover, None, None);
        let result: Result<(), String> = run_audited(
            AuditRecord::admission(&request),
            |_| Err("admission unavailable".into()),
            || panic!("must not execute without durable admission"),
        );
        assert!(result.unwrap_err().contains("admission unavailable"));
        for failed in [false, true] {
            let published = std::cell::RefCell::new(Vec::new());
            let result: Result<(), String> = run_audited(
                AuditRecord::admission(&request),
                |record| {
                    if matches!(record.outcome, AuditOutcome::Started) {
                        published
                            .borrow_mut()
                            .push(serde_json::to_value(record).unwrap());
                        Ok(())
                    } else {
                        Err("disk failure".into())
                    }
                },
                || {
                    if failed {
                        Err("provider failed".into())
                    } else {
                        Ok(())
                    }
                },
            );
            let error = result.unwrap_err();
            assert!(error.contains("terminal audit publication failed"));
            assert!(!failed || error.contains("provider failed"));
            assert_eq!(published.borrow()[0]["outcome"], "Started");
        }
    }

    #[test]
    fn interrupted_operation_leaves_an_unfinished_admission() {
        let request = request(Operation::Discover, None, None);
        let published = std::cell::RefCell::new(Vec::new());
        let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _: Result<(), String> = run_audited(
                AuditRecord::admission(&request),
                |record| {
                    published
                        .borrow_mut()
                        .push(serde_json::to_value(record).unwrap());
                    Ok(())
                },
                || panic!("simulated interruption before an operation returns"),
            );
        }));
        assert!(interrupted.is_err());
        let values = published.into_inner();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0]["outcome"], "Started");
    }

    fn fixture() -> (Configuration, Discovery) {
        let target = Target {
            vm_id: "abcdef01-2345-6789-abcd-ef0123456789".into(),
            gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#test\GPUPARAV".into(),
            enabled: true,
            vram: None,
        };
        let vm = VmState {
            vm_id: target.vm_id.to_ascii_uppercase(),
            name: "An existing user VM".into(),
            power: Power::Off,
            generation: 2,
            gpus: vec![],
            settings: Settings {
                low_mmio: 0,
                high_mmio: 0,
                cache_types: false,
                automatic_checkpoints: true,
            },
            vram: None,
        };
        let gpu = Gpu {
            interface: target.gpu_interface.clone(),
            name: "Discovered GPU".into(),
            vendor: 0x10de,
            device: 0x2d05,
            driver_version: "current discovered version".into(),
            vram: Allocation {
                minimum: 0,
                maximum: 100,
                optimal: 50,
            },
            preparation_supported: true,
        };
        (
            Configuration {
                schema: 2,
                targets: vec![target],
            },
            Discovery {
                issues: vec![],
                vms: vec![vm],
                gpus: vec![gpu],
            },
        )
    }

    #[test]
    fn enrollment_uses_existing_vm_identity_without_name_or_driver_pins() {
        let (mut configuration, mut discovery) = fixture();
        validate_enrollment(&configuration, &discovery).unwrap();
        discovery.vms[0].name = "Renamed by the user".into();
        discovery.vms[0].power = Power::Running;
        discovery.gpus[0].driver_version = "updated discovered version".into();
        let mut second = configuration.targets[0].clone();
        second.vm_id = "22222222-2222-2222-2222-222222222222".into();
        second.enabled = false;
        let mut vm = discovery.vms[0].clone();
        vm.vm_id = second.vm_id.clone();
        configuration.targets.push(second);
        discovery.vms.push(vm);
        validate_enrollment(&configuration, &discovery).unwrap();
    }

    #[test]
    fn enrollment_refuses_unsupported_generation_and_missing_identities() {
        let (configuration, mut discovery) = fixture();
        discovery.vms[0].generation = 1;
        assert!(
            validate_enrollment(&configuration, &discovery)
                .unwrap_err()
                .contains("Generation 2")
        );
        discovery.vms[0].generation = 2;
        discovery.vms[0].vm_id = "22222222-2222-2222-2222-222222222222".into();
        assert!(
            validate_enrollment(&configuration, &discovery)
                .unwrap_err()
                .contains("enrollment VM")
        );
        discovery.vms[0].vm_id = configuration.targets[0].vm_id.clone();
        discovery.gpus[0].interface.push_str("-different");
        assert!(
            validate_enrollment(&configuration, &discovery)
                .unwrap_err()
                .contains("enrollment GPU")
        );
    }

    #[test]
    fn enrollment_validates_native_callers_before_admission() {
        let (mut configuration, discovery) = fixture();
        configuration.schema = 1;
        assert!(validate_enrollment(&configuration, &discovery).is_err());
        configuration.schema = 2;
        let mut duplicate = configuration.targets[0].clone();
        duplicate.vm_id.make_ascii_uppercase();
        configuration.targets.push(duplicate);
        assert!(
            validate_enrollment(&configuration, &discovery)
                .unwrap_err()
                .contains("each VM")
        );
        configuration.targets.clear();
        assert!(validate_enrollment(&configuration, &discovery).is_err());
    }

    #[test]
    fn enrollment_does_not_claim_preparation_support_for_other_vendors() {
        let (mut configuration, mut discovery) = fixture();
        discovery.gpus[0].interface = r"\\?\PCI#VEN_1002&DEV_0001#test\GPUPARAV".into();
        discovery.gpus[0].vendor = 0x1002;
        discovery.gpus[0].preparation_supported = false;
        configuration.targets[0].gpu_interface = discovery.gpus[0].interface.clone();
        validate_enrollment(&configuration, &discovery).unwrap();
    }
}
