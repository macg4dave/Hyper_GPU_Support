//! Exact record-only profile removal through existing enrollment hold and protected store.
use super::*;
use crate::{
    configuration_store::{self, Revision},
    enrollment::{self, PendingPair, RemovalBackend, RemovalReview, RemovalSnapshot, RemovalState},
};

fn file_revision(path: &Path) -> Result<Revision, String> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Revision::Missing),
        Err(e) => return Err(e.to_string()),
        Ok(m) if !m.is_file() => return Err("Profile history is not a regular file.".into()),
        Ok(_) => {}
    }
    crate::security::verify(path)?;
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65536 {
        return Err("Profile history exceeds its input limit.".into());
    }
    Ok(Revision::of(&bytes))
}

fn records(target: &Target) -> Result<RemovalSnapshot, String> {
    if recovery_for_vm(&target.vm_id)?.is_some() || import_record()?.is_some() {
        return Err("Resolve operation/publication/import recovery before profile removal.".into());
    }
    crate::windows_wmi::require_no_pending_operation_for(&target.vm_id)?;
    let current = enrollment()?;
    verify_artifacts(&current)?;
    if current.targets.len() > 128 {
        return Err("Authority target limit exceeded; retain all profiles.".into());
    }
    let mut backend = NativeBackend {
        credential: None,
        manifest: None,
        data: data_directory()?,
    };
    let vm = backend.inspect(target)?;
    for pair in &current.targets {
        pair.validate()?;
        if backend.journal(pair)?.is_some_and(|j| {
            j.schema != 1
                || j.pending
                || j.vm_id != pair.vm_id
                || j.gpu_interface != pair.gpu_interface
        }) {
            return Err("Managed recovery requires reconciliation before profile removal.".into());
        }
    }
    let store = configuration_store::protected::read();
    if !store.errors.is_empty() {
        return Err("Saved configuration is unreadable; retain all profiles.".into());
    }
    enrollment::validate_inactive_profile(
        target,
        &vm,
        backend.journal(target)?.as_ref(),
        store.documents.get(&target.vm_id).map(|d| &d.target),
    )?;
    let mut configurations: std::collections::BTreeMap<_, _> = store
        .documents
        .into_iter()
        .map(|(id, doc)| (id, doc.revision))
        .collect();
    configurations
        .entry(target.vm_id.clone())
        .or_insert(Revision::Missing);
    let root = data_directory()?;
    let mut journals = std::collections::BTreeMap::new();
    for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if !extension.eq_ignore_ascii_case("json") || id.len() != 36 {
            continue;
        }
        let mut candidate = target.clone();
        candidate.vm_id = id.into();
        if candidate.validate().is_err() {
            continue;
        }
        if id != id.to_ascii_lowercase() || extension != "json" || journals.len() >= 128 {
            return Err("Managed history identities are noncanonical or exceed limits.".into());
        }
        journals.insert(id.into(), file_revision(&path)?);
    }
    journals
        .entry(target.vm_id.clone())
        .or_insert(Revision::Missing);
    Ok(RemovalSnapshot {
        policy: enrollment_revision(&current)?,
        configurations,
        journals,
    })
}

/// Preview exact pairing/profile records without elevation or writes.
pub fn review_removal(target: &Target) -> Result<RemovalReview, String> {
    let value = submit(request(Operation::RemovalPlan, Some(target.clone()), None))?;
    serde_json::from_value(value).map_err(|e| format!("Invalid removal review reply: {e}"))
}

pub(super) fn plan(target: &Target, operator: &str) -> Result<RemovalReview, String> {
    review_for_operator(target, operator)
}

fn review_for_operator(target: &Target, operator: &str) -> Result<RemovalReview, String> {
    require_no_vm_recovery(&target.vm_id)?;
    let current = enrollment()?;
    let pair = crate::enrollment::PairReview {
        vm_id: target.vm_id.clone(),
        gpu_interface: target.gpu_interface.clone(),
        expected: enrollment_revision(&current)?,
        operator_sid: operator.into(),
    };
    pair.remove(&current.targets, &pair.expected, &current.client_sid)?;
    let snapshot = records(&pair.target()?)?;
    if snapshot.policy != pair.expected {
        return Err("Authority changed during review; Refresh.".into());
    }
    Ok(RemovalReview {
        pair,
        configurations: snapshot.configurations,
        journals: snapshot.journals,
    })
}

struct ProtectedRemoval {
    target: Target,
}
impl RemovalBackend for ProtectedRemoval {
    fn snapshot(&mut self) -> Result<RemovalSnapshot, String> {
        records(&self.target)
    }
    fn hold(&mut self, pending: &PendingPair) -> Result<(), String> {
        atomic_json(&pair_pending_path()?, pending)
    }
    fn remove_configuration(&mut self, expected: &Revision) -> Result<(), String> {
        configuration_store::protected::remove(&self.target.vm_id, expected)
    }
    fn remove_journal(&mut self, expected: &Revision) -> Result<(), String> {
        let path = data_directory()?.join(format!("{}.json", self.target.vm_id));
        let actual = file_revision(&path)?;
        if actual == Revision::Missing {
            return Ok(());
        }
        if &actual != expected {
            return Err("Managed history changed; retain recovery.".into());
        }
        crate::security::verify(&path)?;
        fs::remove_file(&path).map_err(|e| e.to_string())?;
        if file_revision(&path)? != Revision::Missing {
            return Err("Managed history deletion not established.".into());
        }
        Ok(())
    }
    fn revoke(
        &mut self,
        review: &crate::enrollment::PairReview,
        resulting: &Revision,
    ) -> Result<(), String> {
        let mut current = enrollment()?;
        let actual = enrollment_revision(&current)?;
        if &actual == resulting {
            return Ok(());
        }
        current.targets = review.remove(&current.targets, &actual, &current.client_sid)?;
        if &enrollment_revision(&current)? != resulting {
            return Err("Resulting authority changed; retain recovery.".into());
        }
        atomic_json(&data_directory()?.join("enrollment.json"), &current)
    }
}

pub(crate) fn forget_pair(approved: &RemovalReview, session: &str) -> Result<(), String> {
    let operator = enrollment()?.client_sid;
    let current_review = review_for_operator(&approved.pair.target()?, &operator)?;
    if &current_review != approved {
        return Err("Profile or authority changed after review; no removal performed.".into());
    }
    let mut current = enrollment()?;
    current.targets = approved.pair.remove(
        &current.targets,
        &enrollment_revision(&current)?,
        &current.client_sid,
    )?;
    let mut pending = PendingPair {
        operation_id: session.into(),
        review: approved.pair.clone(),
        resulting: enrollment_revision(&current)?,
        removal: Some(RemovalState {
            approved: approved.clone(),
            configuration_removed: false,
            journal_removed: false,
        }),
    };
    enrollment::publish_removal(
        &mut ProtectedRemoval {
            target: approved.pair.target()?,
        },
        &mut pending,
    )
}

pub(super) fn resume_removal(mut pending: PendingPair) -> Result<(), String> {
    enrollment::publish_removal(
        &mut ProtectedRemoval {
            target: pending.review.target()?,
        },
        &mut pending,
    )
}
