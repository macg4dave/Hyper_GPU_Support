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
    if bootstrap.is_none() {
        validate_enrollment(configuration, &windows_hyperv::discover()?)?;
    }
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
    let _install_lock = operation_lock()?;
    let prior = reinstall::admit(configuration, &client, bootstrap.is_some())?;
    let setup = if let Some((expected, session)) = bootstrap {
        if prior.is_some() {
            return Err("product already enrolled; Refresh and use incremental enrollment".into());
        }
        let pending = pending_setup()?;
        let fresh = pending.is_none();
        let record = prepare_setup(configuration, expected, &client, session, pending)?;
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
        if &crate::setup::source_artifacts()? != expected {
            return Err("setup artifacts changed after review".into());
        }
        Some((record, fresh))
    } else {
        None
    };
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
    if let (Some((expected, session)), Some((record, fresh))) = (bootstrap, &setup) {
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
            &serde_json::json!({"schema":1,"operation":"FirstInstall","session":session,"original_session":record.operation_id,"operator_sid":client,"targets":configuration.targets,"artifacts":expected,"outcome":"Started"}),
        )?;
        if *fresh {
            atomic_json(&data.join("installation-pending.json"), record)?;
        }
    }
    crate::configuration_store::protected::install(&client)?;
    let in_place = package_is_installed(&source, &install)?;
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
        // First setup can run from the protected installation directory. Its
        // frontend retains read-only package locks across UAC; replacing these
        // same files would fail sharing checks, including the running frontend.
        // Keep the existing ACL and current reviewed hash checks above.
        if in_place {
            artifacts.insert(name.into(), hash);
            continue;
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
    if let (Some((expected, session)), Some((record, _))) = (bootstrap, &setup) {
        let readback = read_enrollment()?;
        if readback != enrollment {
            return Err("first-install authority readback mismatch; retain setup recovery".into());
        }
        verify_artifacts(&readback)?;
        atomic_json(
            &data.join("audit").join(format!("setup-{session}.json")),
            &serde_json::json!({"schema":1,"operation":"FirstInstall","session":session,"original_session":record.operation_id,"operator_sid":client,"targets":configuration.targets,"artifacts":expected,"outcome":"Succeeded"}),
        )?;
    } else {
        reinstall::finish(&client)?;
    }
    fs::remove_file(data.join("installation-pending.json")).map_err(|e| e.to_string())?;
    Ok(())
}

// Admission is read-only and runs under the common lock, before task changes.
// A retry retains the original receipt rather than rebinding its audit identity.
fn prepare_setup(
    configuration: &Configuration,
    artifacts: &std::collections::BTreeMap<String, String>,
    client: &str,
    session: &str,
    pending: Option<crate::enrollment::PendingSetup>,
) -> Result<crate::enrollment::PendingSetup, String> {
    if !crate::worker::valid_session(session) {
        return Err("invalid setup session".into());
    }
    let target = configuration
        .targets
        .first()
        .ok_or("setup requires one pair")?;
    let review = crate::enrollment::PairReview {
        vm_id: target.vm_id.clone(),
        gpu_interface: target.gpu_interface.clone(),
        expected: crate::configuration_store::Revision::Missing,
        operator_sid: client.into(),
    };
    if configuration.targets.len() != 1 || &review.target()? != target {
        return Err("setup requires exactly one identity-only pair".into());
    }
    if let Some(pending) = pending {
        pending.check_scope(&review, artifacts, client)?;
        return Ok(pending);
    }
    Ok(crate::enrollment::PendingSetup {
        operation_id: session.into(),
        review,
        artifacts: artifacts.clone(),
    })
}

fn package_is_installed(
    source: &std::path::Path,
    install: &std::path::Path,
) -> Result<bool, String> {
    payload::no_reparse(source)?;
    payload::no_reparse(install)?;
    Ok(fs::canonicalize(source).map_err(|e| e.to_string())?
        == fs::canonicalize(install).map_err(|e| e.to_string())?)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::windows::fs::OpenOptionsExt;

    #[test]
    fn setup_retry_retains_original_receipt_and_rejects_changed_scope_before_publication() {
        let review = crate::enrollment::PairReview {
            vm_id: "12345678-1234-1234-1234-123456789abc".into(),
            gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#fixture\GPUPARAV".into(),
            expected: crate::configuration_store::Revision::Missing,
            operator_sid: "operator".into(),
        };
        let configuration = Configuration {
            schema: 2,
            targets: vec![review.target().unwrap()],
        };
        let artifacts =
            std::collections::BTreeMap::from([("fixed.exe".into(), "reviewed hash".into())]);
        let original = prepare_setup(
            &configuration,
            &artifacts,
            "operator",
            "0123456789abcdef0123456789abcdef",
            None,
        )
        .unwrap();
        let original_bytes = serde_json::to_vec(&original).unwrap();
        let retry = prepare_setup(
            &configuration,
            &artifacts,
            "operator",
            "1123456789abcdef0123456789abcdef",
            Some(original.clone()),
        )
        .unwrap();
        assert_eq!(serde_json::to_vec(&retry).unwrap(), original_bytes);
        for scenario in 0..7 {
            let mut changed = configuration.clone();
            let mut package = artifacts.clone();
            let mut operator = "operator";
            let mut session = "1123456789abcdef0123456789abcdef";
            match scenario {
                0 => changed.targets[0].enabled = true,
                1 => changed.targets.push(changed.targets[0].clone()),
                2 => changed.targets[0].vm_id = "12345678-1234-1234-1234-123456789abd".into(),
                3 => {
                    package.insert("fixed.exe".into(), "changed hash".into());
                }
                4 => operator = "another operator",
                5 => session = "invalid session",
                _ => changed.targets.clear(),
            }
            assert!(
                prepare_setup(
                    &changed,
                    &package,
                    operator,
                    session,
                    Some(original.clone())
                )
                .is_err(),
                "scenario {scenario}"
            );
        }
        assert_eq!(serde_json::to_vec(&original).unwrap(), original_bytes);
    }

    #[test]
    fn first_setup_reuses_the_locked_installed_package_but_not_an_external_package() {
        let root = std::env::temp_dir().join(format!(
            "hyper-gpu-in-place-{}-{}",
            std::process::id(),
            crate::worker::random_session().unwrap()
        ));
        let installed = root.join("installed");
        let external = root.join("external");
        fs::create_dir_all(&installed).unwrap();
        fs::create_dir_all(&external).unwrap();
        let target = installed.join("runner.exe");
        let staged = installed.join("runner.exe.partial");
        fs::write(&target, b"reviewed package").unwrap();
        fs::write(&staged, b"reviewed package").unwrap();
        fs::write(external.join("runner.exe"), b"reviewed package").unwrap();
        let lock = fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&target)
            .unwrap();
        assert!(crate::guest::replace_file(&staged, &target).is_err());
        assert!(package_is_installed(&installed, &installed.join(".")).unwrap());
        assert!(!package_is_installed(&external, &installed).unwrap());
        assert!(package_is_installed(&root.join("missing"), &installed).is_err());
        assert_eq!(fs::read(&target).unwrap(), b"reviewed package");
        drop(lock);
        fs::remove_dir_all(root).unwrap();
    }
}
