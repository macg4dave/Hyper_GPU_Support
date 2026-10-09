//! GUID-keyed committed intent and exact-source conflict tokens.
//! Reading or editing these documents grants no enrollment or effect authority.
use crate::model::{Configuration, Target, VmState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Exact committed bytes, or a deliberately observed absent destination.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Revision {
    /// The destination was independently observed absent.
    Missing,
    /// SHA-256 of exact source bytes, including comments and whitespace.
    Content(String),
}
impl Revision {
    /// Bind a read to its exact original bytes, not a reserialized value.
    pub fn of(source: &[u8]) -> Self {
        Self::Content(
            Sha256::digest(source)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        )
    }
}
/// One structurally valid committed document, not fresh provider state.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedDocument {
    /// Canonical typed intent.
    pub target: Target,
    /// Exact original source for provenance and conflict detection.
    pub source: String,
    /// Byte revision independently read from the file.
    pub revision: Revision,
}
/// Directory reads isolate per-file errors; invalid intent is never a default.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoreSnapshot {
    /// Valid canonical GUID-keyed documents.
    pub documents: BTreeMap<String, SavedDocument>,
    /// File or directory admission failures. Keys are filenames, or `store`.
    pub errors: BTreeMap<String, String>,
}
impl StoreSnapshot {
    /// Add one file read using the production schema and canonical filename rule.
    pub fn insert(&mut self, filename: &str, source: Result<String, String>) {
        let parsed = source.and_then(|source| {
            let configuration = Configuration::parse_vm_file(&source, filename)?;
            let target = configuration
                .targets
                .into_iter()
                .next()
                .ok_or("missing target")?;
            Ok(SavedDocument {
                target,
                revision: Revision::of(source.as_bytes()),
                source,
            })
        });
        let normalized = filename.to_ascii_lowercase();
        let filename = normalized.as_str();
        match parsed {
            Ok(document) => {
                let id = document.target.vm_id.clone();
                if self.documents.contains_key(&id) {
                    self.documents.remove(&id);
                    self.errors
                        .insert(filename.into(), "duplicate configuration identity".into());
                } else if !self.errors.contains_key(filename) {
                    self.documents.insert(id, document);
                }
            }
            Err(error) => {
                self.errors.insert(filename.into(), error);
            }
        }
    }
    /// Loaded intent for presentation. Errors remain separate and must gate edits.
    pub fn configuration(&self) -> Configuration {
        Configuration {
            schema: 2,
            targets: self.documents.values().map(|d| d.target.clone()).collect(),
        }
    }
    /// Missing is a revision only when the directory and target file were readable.
    pub fn revision(&self, id: &str) -> Result<Revision, String> {
        if let Some(error) = self
            .errors
            .get("store")
            .or_else(|| self.errors.get(&format!("{id}.toml")))
        {
            return Err(format!("committed configuration unavailable: {error}"));
        }
        Ok(self
            .documents
            .get(id)
            .map_or(Revision::Missing, |d| d.revision.clone()))
    }
}

/// Durable admission, verified-unsaved state and completed publication are distinct.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SavePhase {
    /// Written before effects. A crash requires manual reconciliation, never replay.
    Admitted,
    /// Persisted before entering any guest/Hyper-V effects; uncertain work is never replayed.
    EffectsStarted,
    /// Independently verified GPU effects; only saving may be retried.
    Verified(VmState),
    /// Configuration publication completed; the operation can be retired safely.
    Published(VmState),
}
/// Protected immutable operation/configuration binding. Contains no credentials.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveRecord {
    /// Versioned recovery format.
    pub schema: u32,
    /// Unique admitted operation identity.
    pub operation_id: String,
    /// Exact desired target independently validated by the worker.
    pub target: Target,
    /// Destination revision approved before any effect.
    pub expected: Revision,
    /// Captured current GPU driver identity.
    pub driver_version: String,
    /// Payload identity when enabling; never an operator driver pin.
    pub payload_digest: Option<String>,
    /// Hash of the complete independently recomputed approved plan.
    pub plan_revision: Revision,
    /// Exact canonical output prepared before effects; publication is idempotent.
    pub intended_source: String,
    /// False for standalone graphics checks; recovery must never import their intent.
    pub publication_required: bool,
    /// Initial independently observed state for explicit reconciliation.
    pub initial: VmState,
    /// Original journal for explicitly retiring a proven pre-effect rejection.
    pub initial_journal: Option<crate::workflow::Journal>,
    /// Reviewed compatibility values expected after the operation.
    pub expected_settings: crate::model::Settings,
    /// Durable operation outcome, separate from a GUI reply or timer.
    pub phase: SavePhase,
}
/// One explicit import destination; importing intent grants no effect authority.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportItem {
    /// Canonical validated single-target TOML output.
    pub source: String,
    /// Successful publication and readback; a crash may precede this receipt.
    pub published: bool,
}
/// Durable per-file import attribution; several files are not one transaction.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportRecord {
    /// Import recovery protocol version.
    pub schema: u32,
    /// Original approved importing operation.
    pub operation_id: String,
    /// Exact source bytes supplied by the operator; source itself is retained.
    pub source_revision: Revision,
    /// Complete canonical destination set approved before any publication.
    pub documents: BTreeMap<String, ImportItem>,
}
impl ImportRecord {
    /// Structural receipt validation independent of privileged publication.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != 1
            || self.operation_id.is_empty()
            || self.documents.is_empty()
            || self.documents.len() > 128
        {
            return Err("invalid protected import receipt".into());
        }
        for (filename, item) in &self.documents {
            Configuration::parse_vm_file(&item.source, filename)?;
        }
        Ok(())
    }
    /// Explicit resume recognizes an unreceipted completed replacement, but cannot
    /// overwrite any external edit or recreate an already published missing file.
    pub fn validate_destination(&self, filename: &str, revision: &Revision) -> Result<(), String> {
        let item = self
            .documents
            .get(filename)
            .ok_or("destination outside approved import")?;
        if *revision == Revision::of(item.source.as_bytes())
            || (!item.published && *revision == Revision::Missing)
        {
            Ok(())
        } else {
            Err(
                "import destination changed externally; stop and reconcile without overwriting"
                    .into(),
            )
        }
    }
}
impl SaveRecord {
    /// Manual reconciliation accepts only the already achieved reviewed outcome;
    /// partial assignment/settings are blocked rather than repaired or replayed.
    pub fn validate_reconciliation(&self, state: &VmState, driver: &str) -> Result<(), String> {
        self.target.validate()?;
        if self.schema != 1
            || self.operation_id.is_empty()
            || self.driver_version != driver
            || state.vm_id != self.target.vm_id
            || state.generation != 2
            || !matches!(
                state.power,
                crate::model::Power::Off | crate::model::Power::Running
            )
            || serde_json::to_value(&state.settings).map_err(|e| e.to_string())?
                != serde_json::to_value(&self.expected_settings).map_err(|e| e.to_string())?
            || (self.publication_required
                && self.target.enabled
                && state.gpus != vec![self.target.gpu_interface.clone()])
            || (self.publication_required && !self.target.enabled && !state.gpus.is_empty())
            || (!self.publication_required && state.gpus != self.initial.gpus)
            || (self.publication_required
                && self.target.enabled
                && self
                    .target
                    .vram
                    .as_ref()
                    .is_some_and(|a| state.vram.as_ref() != Some(a)))
            || (!self.publication_required && state.vram != self.initial.vram)
        {
            return Err("recorded outcome is not independently established; partial state requires administrator reconciliation, never Apply replay".into());
        }
        Ok(())
    }
    /// A save-only retry needs the original verified result and unchanged facts.
    pub fn validate_save_only(
        &self,
        effective: &VmState,
        driver: &str,
        revision: &Revision,
    ) -> Result<(), String> {
        if !self.publication_required {
            return Err("verification-only recovery cannot publish configuration".into());
        }
        self.target.validate()?;
        if self.schema != 1 || self.operation_id.is_empty() {
            return Err("invalid protected save record".into());
        }
        let verified = match &self.phase {
            SavePhase::Verified(verified) | SavePhase::Published(verified) => verified,
            SavePhase::Admitted | SavePhase::EffectsStarted => {
                return Err(
                    "operation is not verified-unsaved; manual reconciliation required".into(),
                );
            }
        };
        let intended = Configuration::parse_vm_file(
            &self.intended_source,
            &format!("{}.toml", self.target.vm_id),
        )?;
        if intended.targets != vec![self.target.clone()] {
            return Err("protected save receipt output does not match its target".into());
        }
        if (self.expected != *revision
            && Revision::of(self.intended_source.as_bytes()) != *revision)
            || self.driver_version != driver
        {
            return Err(
                "configuration or driver changed since verification; refresh and reconcile".into(),
            );
        }
        if matches!(self.phase, SavePhase::Published(_))
            && Revision::of(self.intended_source.as_bytes()) != *revision
        {
            return Err(
                "previously committed output changed; manual reconciliation required".into(),
            );
        }
        if verified.vm_id != self.target.vm_id
            || effective.vm_id != self.target.vm_id
            || serde_json::to_value(verified).map_err(|e| e.to_string())?
                != serde_json::to_value(effective).map_err(|e| e.to_string())?
            || effective.generation != 2
            || (self.target.enabled && effective.gpus != vec![self.target.gpu_interface.clone()])
            || (!self.target.enabled && !effective.gpus.is_empty())
            || (self.target.enabled
                && self
                    .target
                    .vram
                    .as_ref()
                    .is_some_and(|v| effective.vram.as_ref() != Some(v)))
        {
            return Err("verified readback changed; save-only retry cannot modify the VM".into());
        }
        Ok(())
    }
}

#[cfg(windows)]
pub(crate) mod protected;

/// Read committed machine-wide intent without performing backend effects or writes.
#[cfg(windows)]
pub fn read_committed() -> StoreSnapshot {
    protected::read()
}

#[cfg(test)]
mod tests {
    use super::*;
    const ID: &str = "a5801e91-1083-4e79-a803-000000000001";
    const SOURCE: &str =
        include_str!("../config/samples/vms/a5801e91-1083-4e79-a803-000000000001.toml");
    #[test]
    fn bad_file_is_isolated_and_missing_is_not_denied() {
        let mut store = StoreSnapshot::default();
        store.insert(&format!("{ID}.toml"), Ok(SOURCE.into()));
        store.insert("broken.toml", Err("access denied".into()));
        assert_eq!(store.configuration().targets.len(), 1);
        assert!(store.errors.contains_key("broken.toml"));
        assert!(matches!(store.revision(ID).unwrap(), Revision::Content(_)));
        assert_eq!(
            store
                .revision("a5801e91-1083-4e79-a803-000000000003")
                .unwrap(),
            Revision::Missing
        );
        store.errors.insert("store".into(), "access denied".into());
        assert!(store.revision(ID).is_err());
    }
    #[test]
    fn exact_bytes_not_typed_equivalence_are_conflict_tokens() {
        assert_ne!(
            Revision::of(SOURCE.as_bytes()),
            Revision::of(format!("{SOURCE}\n# edit\n").as_bytes())
        );
        let mut store = StoreSnapshot::default();
        store.insert(&format!("{ID}.toml"), Ok(SOURCE.into()));
        store.insert(&format!("{ID}.toml"), Ok(SOURCE.into()));
        assert!(store.documents.is_empty());
        assert!(store.revision(ID).is_err());
        let mut store = StoreSnapshot::default();
        store.insert(&format!("{ID}.TOML"), Err("access denied".into()));
        assert!(store.revision(ID).is_err());
    }
    #[test]
    fn interrupted_import_recognizes_committed_bytes_and_blocks_external_edits() {
        let filename = format!("{ID}.toml");
        let mut record = ImportRecord {
            schema: 1,
            operation_id: "import-one".into(),
            source_revision: Revision::of(SOURCE.as_bytes()),
            documents: [(
                filename.clone(),
                ImportItem {
                    source: SOURCE.into(),
                    published: false,
                },
            )]
            .into(),
        };
        record.validate().unwrap();
        assert!(
            record
                .validate_destination(&filename, &Revision::Missing)
                .is_ok()
        );
        assert!(
            record
                .validate_destination(&filename, &Revision::of(SOURCE.as_bytes()))
                .is_ok()
        );
        assert!(
            record
                .validate_destination(&filename, &Revision::of(b"external change"))
                .is_err()
        );
        record.documents.get_mut(&filename).unwrap().published = true;
        assert!(
            record
                .validate_destination(&filename, &Revision::Missing)
                .is_err()
        );
        assert!(
            record
                .validate_destination("another.toml", &Revision::Missing)
                .is_err()
        );
        let item = record.documents.remove(&filename).unwrap();
        record
            .documents
            .insert("a5801e91-1083-4e79-a803-000000000003.toml".into(), item);
        assert!(record.validate().is_err());
    }
    #[test]
    fn save_only_requires_verified_exact_readback_and_original_revision() {
        let target = Configuration::parse(SOURCE).unwrap().targets.remove(0);
        let effective = VmState {
            vm_id: ID.into(),
            name: "same name".into(),
            power: crate::model::Power::Off,
            generation: 2,
            gpus: vec![target.gpu_interface.clone()],
            settings: crate::model::Settings {
                low_mmio: 3072,
                high_mmio: 32768,
                cache_types: true,
                automatic_checkpoints: false,
            },
            vram: None,
        };
        let mut record = SaveRecord {
            schema: 1,
            operation_id: "one".into(),
            target,
            expected: Revision::Missing,
            driver_version: "driver".into(),
            payload_digest: Some("digest".into()),
            plan_revision: Revision::of(b"plan"),
            intended_source: SOURCE.into(),
            publication_required: true,
            initial: effective.clone(),
            initial_journal: None,
            expected_settings: effective.settings.clone(),
            phase: SavePhase::Admitted,
        };
        assert!(
            record
                .validate_save_only(&effective, "driver", &Revision::Missing)
                .is_err()
        );
        record.phase = SavePhase::Verified(effective.clone());
        assert!(record.validate_reconciliation(&effective, "driver").is_ok());
        let mut partial = effective.clone();
        partial.gpus.clear();
        assert!(record.validate_reconciliation(&partial, "driver").is_err());
        partial = effective.clone();
        partial.settings.high_mmio += 1;
        assert!(record.validate_reconciliation(&partial, "driver").is_err());
        assert!(
            record
                .validate_reconciliation(&effective, "changed driver")
                .is_err()
        );
        assert!(
            record
                .validate_save_only(&effective, "driver", &Revision::Missing)
                .is_ok()
        );
        assert!(
            record
                .validate_save_only(&effective, "new driver", &Revision::Missing)
                .is_err()
        );
        // Crash after replacement but before receipt update is recognized without GPU replay.
        assert!(
            record
                .validate_save_only(&effective, "driver", &Revision::of(SOURCE.as_bytes()))
                .is_ok()
        );
        assert!(
            record
                .validate_save_only(&effective, "driver", &Revision::of(b"external edit"))
                .is_err()
        );
        let mut changed = effective.clone();
        changed.power = crate::model::Power::Running;
        assert!(
            record
                .validate_save_only(&changed, "driver", &Revision::Missing)
                .is_err()
        );
        changed = effective;
        changed.vm_id = "a5801e91-1083-4e79-a803-000000000003".into();
        assert!(
            record
                .validate_save_only(&changed, "driver", &Revision::Missing)
                .is_err()
        );
        record.phase = SavePhase::Published(changed.clone());
        assert!(
            record
                .validate_save_only(&changed, "driver", &Revision::Missing)
                .is_err()
        );
    }
}
