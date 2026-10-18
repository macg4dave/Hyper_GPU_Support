//! Additive pair authority and revision-bound recovery of interrupted pair changes.
use super::{
    backend::NativeBackend,
    policy::{enrollment, enrollment_revision, validate_enrollment, verify_artifacts},
    profile_removal,
    protocol::FRAME_LIMIT,
    recovery::{import_record, recovery_for_vm, require_no_vm_recovery},
    storage::{atomic_json, data_directory},
};
use crate::{
    model::{Configuration, Power, Target},
    windows_hyperv, windows_pipe,
    workflow::Backend,
};
use std::{fs, io::Read, path::PathBuf};

/// Review additive authority against the current protected policy, without writes.
pub fn review_pair(target: &Target) -> Result<crate::enrollment::PairReview, String> {
    target.validate()?;
    let current = enrollment()?;
    verify_artifacts(&current)?;
    let operator = windows_pipe::current_user_sid_string().map_err(|e| e.to_string())?;
    if operator != current.client_sid {
        return Err("current user is not the enrolled operator".into());
    }
    let review = crate::enrollment::PairReview {
        vm_id: target.vm_id.to_ascii_lowercase(),
        gpu_interface: target.gpu_interface.clone(),
        expected: enrollment_revision(&current)?,
        operator_sid: operator,
    };
    review.append(&current.targets, &review.expected, &current.client_sid)?;
    require_no_vm_recovery(&target.vm_id)?;
    crate::windows_wmi::require_no_pending_operation_for(&target.vm_id)?;
    Ok(review)
}

pub(crate) fn pair_pending_path() -> Result<PathBuf, String> {
    Ok(data_directory()?.join("enrollment-pending.json"))
}

/// Inspect an interrupted authority change; absence is accepted only on NotFound.
pub fn pending_pair() -> Result<Option<crate::enrollment::PendingPair>, String> {
    use crate::configuration_store::Revision;
    let path = pair_pending_path()?;
    match fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
        Ok(m) if !m.is_file() => return Err("enrollment recovery is not a file".into()),
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
        return Err("enrollment recovery exceeds input limit".into());
    }
    let record: crate::enrollment::PendingPair =
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    record.review.target()?;
    if record
        .removal
        .as_ref()
        .is_some_and(|state| state.approved.pair != record.review)
    {
        return Err("Profile removal recovery scope is inconsistent.".into());
    }
    if record.operation_id.len() != 32
        || !record
            .operation_id
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        || matches!(record.review.expected, Revision::Missing)
        || matches!(record.resulting, Revision::Missing)
    {
        return Err("invalid enrollment recovery identity/revisions".into());
    }
    Ok(Some(record))
}

/// Called under the worker's common lock and mandatory admission audit only.
pub(crate) fn enroll_pair(
    review: &crate::enrollment::PairReview,
    session: &str,
) -> Result<(), String> {
    require_no_vm_recovery(&review.vm_id)?;
    crate::windows_wmi::require_no_pending_operation_for(&review.vm_id)?;
    let mut current = enrollment()?;
    let revision = enrollment_revision(&current)?;
    let targets = review.append(&current.targets, &revision, &current.client_sid)?;
    let target = review.target()?;
    let discovery = windows_hyperv::discover()?;
    validate_enrollment(
        &Configuration {
            schema: 2,
            targets: vec![target.clone()],
        },
        &discovery,
    )?;
    if discovery
        .issues
        .iter()
        .any(|i| i.target.as_deref() == Some(&target.vm_id))
    {
        return Err("selected VM observations are incomplete; Refresh".into());
    }
    let vm = discovery
        .vms
        .iter()
        .find(|v| v.vm_id == target.vm_id)
        .ok_or("VM disappeared")?;
    if !matches!(vm.power, Power::Off | Power::Running)
        || vm.gpus.len() > 1
        || vm.gpus.iter().any(|g| g != &target.gpu_interface)
    {
        return Err("enrollment requires a stable VM with no unrelated GPU assignment".into());
    }
    let mut reader = NativeBackend::new(data_directory()?, None);
    // Enrollment cannot sidestep any old attributable unfinished operation.
    for old in current.targets.iter().chain(std::iter::once(&target)) {
        if reader.journal(old)?.is_some_and(|j| {
            j.schema != 1
                || j.pending
                || j.vm_id != old.vm_id
                || j.gpu_interface != old.gpu_interface
        }) {
            return Err("managed operation requires reconciliation before enrollment".into());
        }
    }
    current.targets = targets;
    let resulting = enrollment_revision(&current)?;
    crate::enrollment::publish_pair(
        &resulting,
        || {
            atomic_json(
                &pair_pending_path()?,
                &crate::enrollment::PendingPair {
                    operation_id: session.into(),
                    removal: None,
                    review: review.clone(),
                    resulting: resulting.clone(),
                },
            )
        },
        || atomic_json(&data_directory()?.join("enrollment.json"), &current),
        || enrollment_revision(&enrollment()?),
    )
}

/// Read back either complete old or complete new authority; never repeat a write.
pub(crate) fn reconcile_pair(
    operation_id: &str,
    expected: &crate::configuration_store::Revision,
) -> Result<(), String> {
    let pending = pending_pair()?.ok_or("no interrupted enrollment exists")?;
    let current = enrollment()?;
    let actual = enrollment_revision(&current)?;
    pending.check_readback(operation_id, expected, &actual, &current.client_sid)?;
    if pending.removal.is_some() {
        return profile_removal::resume_removal(pending);
    }
    if recovery_for_vm(&pending.review.vm_id)?.is_some() || import_record()?.is_some() {
        return Err("resolve the separate GPU/configuration recovery first".into());
    }
    crate::windows_wmi::require_no_pending_operation_for(&pending.review.vm_id)?;
    Ok(())
}

/// Exact current policy token for explicit recovery review.
pub fn pair_revision() -> Result<crate::configuration_store::Revision, String> {
    enrollment_revision(&enrollment()?)
}
