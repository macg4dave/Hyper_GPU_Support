//! Exact record-only profile removal through existing enrollment hold and protected store.
use super::{
    ipc::submit,
    pairing::{pair_pending_path, pending_pair},
    policy::{enrollment, enrollment_revision, verify_artifacts},
    protocol::FRAME_LIMIT,
    protocol::{Operation, request},
    storage::{atomic_json, data_directory},
};
use crate::model::Target;
use crate::{
    configuration_store::{self, Revision},
    enrollment::{self, PendingPair, RemovalBackend, RemovalReview, RemovalSnapshot, RemovalState},
};
use std::{fs, path::Path};

fn record_bytes(path: &Path) -> Result<Option<Vec<u8>>, String> {
    crate::security::read_optional(
        path,
        FRAME_LIMIT,
        "Profile history is not a regular file.",
        "Profile history exceeds its input limit.",
    )
}

fn file_revision(path: &Path) -> Result<Revision, String> {
    Ok(record_bytes(path)?.map_or(Revision::Missing, |bytes| Revision::of(&bytes)))
}

fn recovery_revisions(
    target: &Target,
) -> Result<std::collections::BTreeMap<String, Revision>, String> {
    let root = data_directory()?;
    let mut result = std::collections::BTreeMap::new();
    for name in enrollment::profile_recovery_names(&target.vm_id) {
        let bytes = record_bytes(&root.join(&name))?;
        let owned = bytes
            .as_deref()
            .is_some_and(|bytes| belongs_to_profile(&target.vm_id, &name, bytes));
        result.insert(
            name,
            if owned {
                bytes.map_or(Revision::Missing, |bytes| Revision::of(&bytes))
            } else {
                Revision::Missing
            },
        );
    }
    Ok(result)
}

// A legacy barrier, an unreadable identity or another VM's singleton is never
// selected by filename alone. Canonical per-VM leaf names need no JSON parsing.
fn legacy_vm(value: &serde_json::Value) -> Option<&str> {
    value
        .pointer("/target/vm_id")
        .or_else(|| value.pointer("/scope/target/vm_id"))?
        .as_str()
}

fn belongs_to_profile(id: &str, name: &str, bytes: &[u8]) -> bool {
    if !enrollment::profile_recovery_names(id).contains(&name.to_owned()) {
        return false;
    }
    if matches!(name, "operation-recovery.json" | "provider-operation.json") {
        return serde_json::from_slice::<serde_json::Value>(bytes)
            .is_ok_and(|value| legacy_vm(&value) == Some(id));
    }
    if name == "configuration-import.json" {
        return serde_json::from_slice::<configuration_store::ImportRecord>(bytes)
            .is_ok_and(|record| record.documents.contains_key(&format!("{id}.toml")));
    }
    true
}

fn discard_import_profile(
    record: &mut configuration_store::ImportRecord,
    id: &str,
) -> Result<bool, String> {
    record.validate()?;
    if record.documents.remove(&format!("{id}.toml")).is_none() {
        return Err("Profile is no longer part of this interrupted import.".into());
    }
    if record.documents.is_empty() {
        return Ok(true);
    }
    record.validate()?;
    Ok(false)
}

fn records(target: &Target) -> Result<RemovalSnapshot, String> {
    target.validate()?;
    let current = enrollment()?;
    verify_artifacts(&current)?;
    if current.targets.len() > 128 {
        return Err("Authority target limit exceeded; retain all profiles.".into());
    }
    let store = configuration_store::protected::read();
    if store
        .errors
        .keys()
        .any(|key| key != &format!("{}.toml", target.vm_id))
    {
        return Err("Saved configuration is unreadable; retain all profiles.".into());
    }
    let mut configurations: std::collections::BTreeMap<_, _> = store
        .documents
        .into_iter()
        .map(|(id, doc)| (id, doc.revision))
        .collect();
    configurations.insert(
        target.vm_id.clone(),
        configuration_store::protected::revision(&target.vm_id)?,
    );
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
        recovery: recovery_revisions(target)?,
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
    if pending_pair()?.is_some() {
        return Err("Finish the interrupted app-permission change before reviewing another profile deletion.".into());
    }
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
        recovery: snapshot.recovery,
        pair,
        configurations: snapshot.configurations,
        journals: snapshot.journals,
    })
}

struct ProtectedRemoval {
    target: Target,
    recovery_names: std::collections::BTreeSet<String>,
}
impl RemovalBackend for ProtectedRemoval {
    fn snapshot(&mut self) -> Result<RemovalSnapshot, String> {
        let mut snapshot = records(&self.target)?;
        if snapshot.recovery.iter().any(|(name, revision)| {
            !self.recovery_names.contains(name) && *revision != Revision::Missing
        }) {
            return Err("New recovery records were not included in this deletion review.".into());
        }
        // Older durable deletions approved no recovery-file removal. Resume that
        // original scope only while all unreviewed recovery paths remain absent.
        snapshot
            .recovery
            .retain(|name, _| self.recovery_names.contains(name));
        Ok(snapshot)
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
    fn remove_recovery(&mut self, name: &str, expected: &Revision) -> Result<(), String> {
        if !enrollment::profile_recovery_names(&self.target.vm_id).contains(&name.to_owned()) {
            return Err("Recovery deletion is outside this profile.".into());
        }
        let actual = recovery_revisions(&self.target)?[name].clone();
        if actual == Revision::Missing {
            return Ok(());
        }
        if &actual != expected {
            return Err("Profile recovery changed before deletion.".into());
        }
        let path = data_directory()?.join(name);
        crate::security::verify(&path)?;
        if name == "configuration-import.json" {
            let bytes =
                record_bytes(&path)?.ok_or("Interrupted import disappeared before deletion")?;
            if Revision::of(&bytes) != *expected {
                return Err("Interrupted import changed before profile deletion.".into());
            }
            let mut record: configuration_store::ImportRecord =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            if discard_import_profile(&mut record, &self.target.vm_id)? {
                fs::remove_file(&path).map_err(|e| e.to_string())?;
            } else {
                atomic_json(&path, &record)?;
            }
        } else {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        if recovery_revisions(&self.target)?[name] != Revision::Missing {
            return Err("Profile recovery removal was not established.".into());
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
            recovery_removed: Default::default(),
            approved: approved.clone(),
            configuration_removed: false,
            journal_removed: false,
        }),
    };
    enrollment::publish_removal(
        &mut ProtectedRemoval {
            target: approved.pair.target()?,
            recovery_names: approved.recovery.keys().cloned().collect(),
        },
        &mut pending,
    )
}

pub(super) fn resume_removal(mut pending: PendingPair) -> Result<(), String> {
    enrollment::publish_removal(
        &mut ProtectedRemoval {
            target: pending.review.target()?,
            recovery_names: pending
                .removal
                .as_ref()
                .ok_or("No bound profile deletion exists")?
                .approved
                .recovery
                .keys()
                .cloned()
                .collect(),
        },
        &mut pending,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deleting_a_profile_from_an_interrupted_import_cannot_recreate_it_or_drop_other_profiles() {
        let target = |last: char| Target {
            vm_id: format!("12345678-1234-1234-1234-123456789ab{last}"),
            gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#fixture\GPUPARAV".into(),
            enabled: false,
            vram: None,
            compute: None,
            encode: None,
            decode: None,
        };
        let deleted = target('c');
        let retained = target('d');
        let documents = crate::model::Configuration {
            schema: 3,
            targets: vec![deleted.clone(), retained.clone()],
        }
        .vm_documents()
        .unwrap()
        .into_iter()
        .map(|(name, source)| {
            (
                name,
                configuration_store::ImportItem {
                    source,
                    published: false,
                },
            )
        })
        .collect();
        let mut record = configuration_store::ImportRecord {
            schema: 1,
            operation_id: "original-import".into(),
            source_revision: Revision::of(b"original batch"),
            documents,
        };
        let before = record.clone();
        let bytes = serde_json::to_vec(&record).unwrap();
        assert!(belongs_to_profile(
            &deleted.vm_id,
            "configuration-import.json",
            &bytes
        ));
        assert!(!discard_import_profile(&mut record, &deleted.vm_id).unwrap());
        assert_eq!(record.operation_id, before.operation_id);
        assert_eq!(record.source_revision, before.source_revision);
        let retained_name = format!("{}.toml", retained.vm_id);
        assert_eq!(
            serde_json::to_value(&record.documents[&retained_name]).unwrap(),
            serde_json::to_value(&before.documents[&retained_name]).unwrap()
        );
        assert!(
            record
                .validate_destination(&format!("{}.toml", deleted.vm_id), &Revision::Missing)
                .is_err()
        );
        assert!(
            record
                .validate_destination(&retained_name, &Revision::Missing)
                .is_ok()
        );
        assert!(!belongs_to_profile(
            &deleted.vm_id,
            "configuration-import.json",
            &serde_json::to_vec(&record).unwrap()
        ));
        assert!(discard_import_profile(&mut record, &deleted.vm_id).is_err());
        assert!(discard_import_profile(&mut record, &retained.vm_id).unwrap());
    }

    #[test]
    fn profile_record_scope_rejects_other_vms_unknown_legacy_and_path_injection() {
        let id = "12345678-1234-1234-1234-123456789abc";
        let other = "12345678-1234-1234-1234-123456789abd";
        let workflow = serde_json::to_vec(&serde_json::json!({"target":{"vm_id":id}})).unwrap();
        let native =
            serde_json::to_vec(&serde_json::json!({"scope":{"target":{"vm_id":id}}})).unwrap();
        assert!(belongs_to_profile(id, "operation-recovery.json", &workflow));
        assert!(belongs_to_profile(id, "provider-operation.json", &native));
        assert!(!belongs_to_profile(
            other,
            "operation-recovery.json",
            &workflow
        ));
        assert!(!belongs_to_profile(
            other,
            "provider-operation.json",
            &native
        ));
        assert!(!belongs_to_profile(
            id,
            "operation-recovery.json",
            br#"{"schema":2,"per_vm":true}"#
        ));
        assert!(!belongs_to_profile(
            id,
            "provider-operation.json",
            b"invalid legacy identity"
        ));
        for name in [
            "../enrollment.json".to_owned(),
            format!("provider-operation-{other}.json"),
        ] {
            assert!(!belongs_to_profile(id, &name, &workflow));
        }
        // Canonical per-profile corrupt records can be explicitly discarded;
        // they cannot make another VM's filename part of this deletion.
        assert!(belongs_to_profile(
            id,
            &format!("operation-recovery-{id}.json"),
            b"corrupt"
        ));
    }
}
