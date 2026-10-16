//! Fixed local Task Scheduler connection and product task registration.
use super::storage::install_directory;
use windows::{
    Win32::System::{
        Com::{CLSCTX_INPROC_SERVER, CoCreateInstance},
        TaskScheduler::{
            ITaskService, TASK_CREATE_OR_UPDATE, TASK_LOGON_SERVICE_ACCOUNT, TaskScheduler,
        },
        Variant::VARIANT,
    },
    core::BSTR,
};

pub(super) const TASK: &str = "HyperGpuSupport-Product-v2";
#[allow(unsafe_code)]
pub(super) fn scheduler() -> Result<(crate::windows_com::Apartment, ITaskService), String> {
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
#[allow(unsafe_code)]
pub(super) fn register_product_task(service: &ITaskService, client: &str) -> Result<(), String> {
    let install = install_directory()?;
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
    Ok(())
}
