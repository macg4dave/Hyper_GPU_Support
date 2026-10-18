//! Protected artifact installation, first-install admission and setup recovery.
use super::{
    backend::NativeBackend,
    policy::{Enrollment, enrollment, read_enrollment, validate_enrollment, verify_artifacts},
    protocol::FRAME_LIMIT,
    recovery::require_no_recovery,
    reinstall,
    scheduler::{TASK, register_product_task, scheduler},
    storage::{
        atomic_json, data_directory, install_directory, operation_lock, protect,
        remove_protected_leaf,
    },
};
use crate::{
    model::Configuration, payload, process, windows_hyperv, windows_pipe, workflow::Backend,
};
use std::fs;
use windows::core::BSTR;

/// Administrator-only native installation/enrollment of selected existing VMs.
#[allow(unsafe_code)]
pub fn install(configuration: &Configuration) -> Result<(), String> {
    install_inner(configuration, None)
}

/// Fixed first-install route only; an installed policy can never be replaced here.
pub(crate) fn bootstrap_install(
    configuration: &Configuration,
    artifacts: &std::collections::BTreeMap<String, String>,
    session: &str,
) -> Result<(), String> {
    install_inner(configuration, Some((artifacts, session)))
}

/// Distinguish genuinely absent enrollment from unreadable/interrupted installation.
pub fn installed() -> Result<bool, String> {
    let root = data_directory()?;
    match fs::symlink_metadata(&root) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.to_string()),
        Ok(_) => crate::security::verify(&root)?,
    }
    match fs::symlink_metadata(root.join("enrollment.json")) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.to_string()),
        Ok(_) => {
            enrollment()?;
            Ok(true)
        }
    }
}

#[allow(unsafe_code)]
fn install_inner(
    configuration: &Configuration,
    bootstrap: Option<(&std::collections::BTreeMap<String, String>, &str)>,
) -> Result<(), String> {
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
        // An existing ancestor may belong to a concurrently installed product.
        // Verify it without changing owner/operator ACLs before admission.
        match fs::symlink_metadata(p) {
            Ok(_) => crate::security::verify(p)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                crate::security::create_directory(p, &client)?
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    let source = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("missing executable directory")?
        .to_path_buf();
    // First-install setup must not disable/update an existing task before its
    // no-replacement check. Hold admission across the scheduler and publication.
    let install_lock = operation_lock()?;
    let prior = reinstall::admit(configuration, &client, bootstrap.is_some())?;
    if bootstrap.is_some() {
        match fs::symlink_metadata(data.join("enrollment.json")) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
            Ok(_) => {
                return Err(
                    "product already enrolled; Refresh and use incremental enrollment".into(),
                );
            }
        }
        require_no_recovery()?;
        crate::windows_wmi::require_no_pending_operation()?;
        // Revalidate mutable VM/GPU state after admission, not only before UAC.
        let mut view = crate::gui_model::View::new(Configuration {
            schema: 2,
            targets: vec![],
        });
        view.refresh(crate::gui_model::Inventory {
            discovery: windows_hyperv::discover()?,
            enrolled: vec![],
            managed: Default::default(),
        });
        let mut reader = NativeBackend::new(data.clone(), None);
        for target in &configuration.targets {
            view.proposal_eligibility(target)?;
            if reader.journal(target)?.is_some_and(|j| {
                j.schema != 1
                    || j.pending
                    || j.vm_id != target.vm_id
                    || j.gpu_interface != target.gpu_interface
            }) {
                return Err(
                    "managed state requires reconciliation before first-install setup".into(),
                );
            }
        }
    }
    if bootstrap.is_none() {
        reinstall::begin(prior.as_ref(), configuration, &client, &source)?;
    }
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
    let _lock = install_lock;
    if let Some((expected, session)) = bootstrap {
        // Repeat the absence check under common admission, before publication.
        // Even a concurrent CLI installation cannot be replaced from one GUI pair.
        if fs::symlink_metadata(data.join("enrollment.json")).is_ok() {
            return Err(
                "product enrollment already exists; Refresh and use incremental enrollment".into(),
            );
        }
        match fs::symlink_metadata(data.join("enrollment.json")) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
            Ok(_) => return Err("enrollment appeared during setup".into()),
        }
        require_no_recovery()?;
        crate::windows_wmi::require_no_pending_operation()?;
        if &crate::setup::source_artifacts()? != expected {
            return Err("setup artifacts changed after review".into());
        }
        let audit = data.join("audit");
        if !audit.exists() {
            crate::security::create_directory(&audit, &client)?;
        }
        crate::security::verify(&audit)?;
        let path = audit.join(format!("setup-{session}.json"));
        if path.exists() {
            return Err("replayed setup session".into());
        }
        atomic_json(
            &path,
            &serde_json::json!({"schema":1,"operation":"FirstInstall","session":session,"operator_sid":client,"targets":configuration.targets,"artifacts":expected,"outcome":"Started"}),
        )?;
    }
    if let Some((artifacts, session)) = bootstrap {
        let target = configuration
            .targets
            .first()
            .ok_or("setup requires one pair")?;
        let review = crate::enrollment::PairReview {
            vm_id: target.vm_id.clone(),
            gpu_interface: target.gpu_interface.clone(),
            expected: crate::configuration_store::Revision::Missing,
            operator_sid: client.clone(),
        };
        if configuration.targets.len() != 1 || &review.target()? != target {
            return Err("setup requires exactly one identity-only pair".into());
        }
        if let Some(pending) = pending_setup()?
            && (pending.review != review || &pending.artifacts != artifacts)
        {
            return Err("interrupted setup has a different original scope; retain recovery".into());
        }
        atomic_json(
            &data.join("installation-pending.json"),
            &crate::enrollment::PendingSetup {
                operation_id: session.into(),
                review,
                artifacts: artifacts.clone(),
            },
        )?;
    }
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
        if let Some((expected, _)) = bootstrap
            && expected.get(name) != Some(&hash)
        {
            return Err(
                "setup artifact changed during installation; installation remains held".into(),
            );
        }
        let to = install.join(name);
        if to.exists() {
            crate::security::verify(&to)?;
        }
        let staged = install.join(format!("{name}.partial"));
        remove_protected_leaf(&staged)?;
        let mut input = fs::File::open(&from).map_err(|e| e.to_string())?;
        let mut output = crate::security::create_file(&staged, &client)?;
        crate::diagnostics::run("install-file-copy", name, || {
            std::io::copy(&mut input, &mut output)
                .and_then(|_| output.sync_all())
                .map_err(|e| e.to_string())
        })?;
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
        targets: prior
            .as_ref()
            .map_or_else(|| configuration.targets.clone(), |p| p.targets.clone()),
        artifacts,
    };
    if bootstrap.is_none() {
        reinstall::publish(&enrollment)?;
    } else {
        atomic_json(&data.join("enrollment.json"), &enrollment)?;
    }
    protect(&data.join("enrollment.json"), &client)?;
    register_product_task(&service, &client)?;
    if let Some((expected, session)) = bootstrap {
        let readback = read_enrollment()?;
        if readback != enrollment {
            return Err("first-install authority readback mismatch; retain setup recovery".into());
        }
        verify_artifacts(&readback)?;
        atomic_json(
            &data.join("audit").join(format!("setup-{session}.json")),
            &serde_json::json!({"schema":1,"operation":"FirstInstall","session":session,"operator_sid":client,"targets":configuration.targets,"artifacts":expected,"outcome":"Succeeded"}),
        )?;
    } else {
        reinstall::finish(&client)?;
    }
    fs::remove_file(data.join("installation-pending.json")).map_err(|e| e.to_string())?;
    Ok(())
}

/// Only GUI first-install records are accepted here; legacy CLI markers stay explicit.
pub fn pending_setup() -> Result<Option<crate::enrollment::PendingSetup>, String> {
    let path = data_directory()?.join("installation-pending.json");
    let Some(bytes) = crate::security::read_optional(
        &path,
        FRAME_LIMIT,
        "setup recovery is not a file",
        "setup recovery exceeds input limit",
    )?
    else {
        return Ok(None);
    };
    let record: crate::enrollment::PendingSetup = serde_json::from_slice(&bytes)
        .map_err(|_| "interrupted CLI installation requires administrator install; no GUI setup binding exists")?;
    record.review.target()?;
    if record.review.expected != crate::configuration_store::Revision::Missing
        || !crate::worker::valid_session(&record.operation_id)
        || record.artifacts.len() != 4
        || ![
            "hyper-gpu-support.exe",
            "hyper-gpu-runner.exe",
            "hyper-gpu-guest.exe",
            "d3d11-probe.exe",
        ]
        .iter()
        .all(|n| record.artifacts.contains_key(*n))
    {
        return Err("invalid first-install recovery scope".into());
    }
    Ok(Some(record))
}

/// Complete only the original fully published install. No artifact/enrollment overwrite.
pub(crate) fn reconcile_setup(
    approved: &crate::enrollment::PairReview,
    artifacts: &std::collections::BTreeMap<String, String>,
    session: &str,
) -> Result<(), String> {
    if !process::is_elevated()? {
        return Err("setup recovery requires elevation".into());
    }
    let _lock = operation_lock()?;
    let record = pending_setup()?.ok_or("no interrupted setup exists")?;
    let client = windows_pipe::current_user_sid_string().map_err(|e| e.to_string())?;
    record.check_scope(approved, artifacts, &client)?;
    require_no_recovery()?;
    crate::windows_wmi::require_no_pending_operation()?;
    let current = read_enrollment()?;
    record.check_authority(&current.targets, &current.artifacts, &current.client_sid)?;
    verify_artifacts(&current)?;
    let data = data_directory()?;
    let audit = data.join("audit").join(format!("setup-{session}.json"));
    if audit.exists() {
        return Err("replayed setup recovery session".into());
    }
    atomic_json(
        &audit,
        &serde_json::json!({"schema":1,"operation":"ReconcileSetup","session":session,"original":record,"outcome":"Started"}),
    )?;
    let (_apartment, service) = scheduler()?;
    register_product_task(&service, &client)?;
    atomic_json(
        &audit,
        &serde_json::json!({"schema":1,"operation":"ReconcileSetup","session":session,"original":record,"outcome":"Succeeded"}),
    )?;
    crate::security::verify(&data.join("installation-pending.json"))?;
    fs::remove_file(data.join("installation-pending.json")).map_err(|e| e.to_string())
}
