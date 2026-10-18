//! Exact, durable completion acknowledgements; no lifecycle work is initiated here.
use crate::{
    configuration_store::{SavePhase, SaveRecord},
    model::Target,
    payload::Manifest,
    runner,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

const SCAN_LIMIT: usize = 16384;
// Bound cleanup memory/work independently of the host queue size.
const ACK_LIMIT: usize = 2;
thread_local! { static OPERATION: RefCell<Option<String>> = const { RefCell::new(None) }; }

pub(crate) fn scope<T>(
    operation: &str,
    work: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    if !crate::worker::valid_session(operation) {
        return Err("invalid staging operation".into());
    }
    struct Reset(Option<String>);
    impl Drop for Reset {
        fn drop(&mut self) {
            OPERATION.with(|slot| *slot.borrow_mut() = self.0.take());
        }
    }
    let _reset = Reset(OPERATION.with(|slot| slot.replace(Some(operation.into()))));
    work()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Identity {
    schema: u32,
    operation: String,
    vm_id: String,
    manifest: String,
    mode: String,
    files: usize,
    pub(crate) bundle: String,
}
impl Identity {
    pub(crate) fn require_files(&self, files: usize) -> Result<(), String> {
        if files != self.files {
            return Err("bundle file count mismatch".into());
        }
        Ok(())
    }
    fn validate(&self) -> Result<(), String> {
        let prefix = format!("{}-{}-", self.mode, self.manifest);
        if self.schema != 1
            || !crate::worker::valid_session(&self.operation)
            || !crate::model::valid_vm_id(&self.vm_id)
            || self.vm_id.bytes().any(|b| b.is_ascii_uppercase())
            || self.manifest.len() != 64
            || !self
                .manifest
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            || !matches!(self.mode.as_str(), "prepare" | "verify")
            || self.files == 0
            || self.files > 65536
            || !self
                .bundle
                .strip_prefix(&prefix)
                .is_some_and(crate::worker::valid_session)
        {
            return Err("invalid staging identity".into());
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    identity: Identity,
    completed: bool,
}

fn bounded<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    use std::os::windows::fs::OpenOptionsExt;
    crate::security::verify_single_file(path)?;
    let mut bytes = Vec::new();
    fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(path)
        .map_err(|e| e.to_string())?
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65536 {
        return Err("staging metadata exceeds bound".into());
    }
    serde_json::from_slice(&bytes).map_err(|_| "invalid staging metadata".into())
}

fn directory(data: &Path) -> Result<PathBuf, String> {
    let path = data.join("staging");
    if !crate::security::entry_exists(&path)? {
        crate::security::create_directory(&path, &runner::enrollment()?.client_sid)?;
    }
    crate::security::verify(&path)?;
    Ok(path)
}

fn records(data: &Path) -> Result<Vec<(PathBuf, Record)>, String> {
    let root = data.join("staging");
    if !crate::security::entry_exists(&root)? {
        return Ok(Vec::new());
    }
    crate::security::verify(&root)?;
    let mut records = Vec::new();
    for (count, entry) in fs::read_dir(&root).map_err(|e| e.to_string())?.enumerate() {
        if count >= SCAN_LIMIT {
            return Err("staging scan exceeds bound".into());
        }
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let record: Record = bounded(&path)?;
        record.identity.validate()?;
        if path != root.join(format!("{}.json", record.identity.bundle)) {
            return Err("staging filename mismatch".into());
        }
        records.push((path, record));
    }
    records.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(records)
}

pub(crate) fn register(
    target: &Target,
    manifest: &Manifest,
    mode: &str,
) -> Result<Identity, String> {
    let identity = Identity {
        schema: 1,
        operation: OPERATION
            .with(|slot| slot.borrow().clone())
            .map(Ok)
            .unwrap_or_else(crate::worker::random_session)?,
        vm_id: target.vm_id.to_ascii_lowercase(),
        manifest: manifest.digest()?,
        mode: mode.into(),
        files: manifest.files.len(),
        bundle: format!(
            "{mode}-{}-{}",
            manifest.digest()?,
            crate::worker::random_session()?
        ),
    };
    identity.validate()?;
    let data = runner::data_directory()?;
    if records(&data)?.len() >= SCAN_LIMIT {
        return Err("staging capacity exhausted; retain evidence".into());
    }
    runner::atomic_json(
        &directory(&data)?.join(format!("{}.json", identity.bundle)),
        &Record {
            identity: identity.clone(),
            completed: false,
        },
    )?;
    Ok(identity)
}

fn permits(record: &SaveRecord, identity: &Identity) -> bool {
    identity.operation == record.operation_id
        && identity.vm_id == record.target.vm_id
        && Some(identity.manifest.as_str()) == record.payload_digest.as_deref()
        && record.retirement.is_none()
        && matches!(
            record.phase,
            SavePhase::Verified(_) | SavePhase::Published(_)
        )
}

/// Called only after durable terminal audit and exact recovery retirement, under the common lock.
pub(crate) fn acknowledge(data: &Path, terminal: &SaveRecord) -> Result<(), String> {
    if runner::recovery_records()?
        .iter()
        .any(|r| r.operation_id == terminal.operation_id)
    {
        return Err("operation still has recovery".into());
    }
    for (path, mut record) in records(data)? {
        if permits(terminal, &record.identity)
            && !crate::security::entry_exists(&path.with_extension("json.retain"))?
        {
            record.completed = true;
            runner::atomic_json(&path, &record)?;
        }
    }
    Ok(())
}

pub(crate) fn pending(vm: &str) -> Result<Vec<Identity>, String> {
    let active = runner::recovery_records()?;
    Ok(records(&runner::data_directory()?)?
        .into_iter()
        .filter(|(path, r)| {
            r.completed
                && r.identity.vm_id == vm
                && !active
                    .iter()
                    .any(|a| a.operation_id == r.identity.operation)
                && matches!(
                    crate::security::entry_exists(&path.with_extension("json.retain")),
                    Ok(false)
                )
        })
        .take(ACK_LIMIT)
        .map(|(_, r)| r.identity)
        .collect())
}

pub(crate) fn received(sent: &[Identity], cleaned: &[String]) -> Result<(), String> {
    if cleaned.is_empty() {
        return Ok(());
    }
    let mut seen = BTreeSet::new();
    // Validate the complete reply before removing any host acknowledgement.
    for bundle in cleaned {
        if !seen.insert(bundle.clone()) || !sent.iter().any(|i| &i.bundle == bundle) {
            return Err("invalid staging cleanup reply".into());
        }
    }
    let root = directory(&runner::data_directory()?)?;
    for identity in sent.iter().filter(|i| seen.contains(&i.bundle)) {
        let path = root.join(format!("{}.json", identity.bundle));
        let record: Record = bounded(&path)?;
        if record.identity != *identity || !record.completed {
            return Err("staging acknowledgement changed".into());
        }
        if crate::security::entry_exists(&path.with_extension("json.retain"))? {
            continue;
        }
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub(crate) fn guest_identity(bundle: &Path, mode: &str, digest: &str) -> Result<Identity, String> {
    let identity: Identity = bounded(&bundle.join("identity.json"))?;
    identity.validate()?;
    if identity.bundle
        != bundle
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("invalid bundle leaf")?
        || identity.mode != mode
        || identity.manifest != digest
    {
        return Err("bundle identity mismatch".into());
    }
    Ok(identity)
}

fn cleanup_plan(
    current: &Identity,
    acknowledgements: &[Identity],
    mut inspect: impl FnMut(&Identity) -> Result<Option<Vec<PathBuf>>, String>,
) -> Result<Vec<(Identity, Vec<PathBuf>)>, String> {
    if acknowledgements.len() > ACK_LIMIT {
        return Err("too many cleanup acknowledgements".into());
    }
    let mut seen = BTreeSet::new();
    let mut plan = Vec::new();
    for ack in acknowledgements {
        ack.validate()?;
        if ack.vm_id != current.vm_id
            || ack.operation == current.operation
            || !seen.insert(&ack.bundle)
        {
            return Err("duplicate or mismatched cleanup acknowledgement".into());
        }
        if let Some(leaves) = inspect(ack)? {
            plan.push((ack.clone(), leaves));
        }
    }
    Ok(plan)
}

fn inspect_bundle(root: &Path, ack: &Identity) -> Result<Option<Vec<PathBuf>>, String> {
    let directory = root.join(&ack.bundle);
    if !crate::security::entry_exists(&directory)? {
        return Ok(Some(Vec::new()));
    }
    crate::security::verify(&directory)?;
    if crate::security::entry_exists(&directory.join(".retain"))? {
        return Ok(None);
    }
    let mut leaves = Vec::new();
    for (count, entry) in fs::read_dir(&directory)
        .map_err(|e| e.to_string())?
        .enumerate()
    {
        if count > ack.files + 4 {
            return Err("staging leaf count exceeds bound".into());
        }
        let path = entry.map_err(|e| e.to_string())?.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("invalid staging leaf")?;
        if !owned_leaf(ack, name) {
            return Err("unknown staging leaf; retain bundle".into());
        }
        crate::security::verify_single_file(&path)?;
        leaves.push(path);
    }
    if !leaves.is_empty() && bounded::<Identity>(&directory.join("identity.json"))? != *ack {
        return Err("cleanup identity mismatch".into());
    }
    leaves.sort_by_key(|path| path.file_name().is_some_and(|n| n == "identity.json"));
    Ok(Some(leaves))
}

fn owned_leaf(identity: &Identity, name: &str) -> bool {
    matches!(
        name,
        "identity.json" | "acknowledgements.json" | "cleanup-result.json"
    ) || (identity.mode == "prepare"
        && (name == "manifest.json"
            || name
                .parse::<usize>()
                .is_ok_and(|i| i < identity.files && i.to_string() == name)))
        || (identity.mode == "verify" && name == "verification.json")
}

fn clean_with(
    current: &Identity,
    acknowledgements: &[Identity],
    inspect: impl FnMut(&Identity) -> Result<Option<Vec<PathBuf>>, String>,
    mut remove: impl FnMut(&Identity, Vec<PathBuf>) -> Result<(), String>,
) -> Result<Vec<String>, String> {
    let plan = cleanup_plan(current, acknowledgements, inspect)?;
    let mut cleaned = Vec::new();
    for (identity, leaves) in plan {
        remove(&identity, leaves)?;
        cleaned.push(identity.bundle);
    }
    Ok(cleaned)
}

/// Called with the guest preparation lock held. Failures never affect the main operation.
pub(crate) fn clean_guest(root: &Path, current: &Identity) -> Result<Vec<String>, String> {
    let acknowledgements: Vec<Identity> =
        bounded(&root.join(&current.bundle).join("acknowledgements.json"))?;
    clean_with(
        current,
        &acknowledgements,
        |ack| inspect_bundle(root, ack),
        |identity, leaves| {
            for path in leaves {
                crate::security::verify_single_file(&path)?;
                fs::remove_file(path).map_err(|e| e.to_string())?;
            }
            let path = root.join(&identity.bundle);
            if crate::security::entry_exists(&path)? {
                fs::remove_dir(path).map_err(|e| e.to_string())?;
            }
            Ok(())
        },
    )
}

pub(crate) fn cleanup_in_session(root: &Path, current: &Identity) {
    // A failed cleanup is diagnostic evidence, never a failed GPU operation.
    let _ = crate::diagnostics::run(
        "guest-staging-cleanup",
        "completed acknowledged bundles",
        || {
            let cleaned = clean_guest(root, current)?;
            runner::atomic_json(
                &root.join(&current.bundle).join("cleanup-result.json"),
                &cleaned,
            )
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operation_scope_restores_original_identity_after_failure_or_nested_reconciliation() {
        assert!(OPERATION.with(|s| s.borrow().is_none()));
        let original = "1".repeat(32);
        let reconciliation = "2".repeat(32);
        scope(&original, || {
            assert_eq!(
                OPERATION.with(|s| s.borrow().clone()),
                Some(original.clone())
            );
            let failed: Result<(), String> = scope(&reconciliation, || {
                assert_eq!(
                    OPERATION.with(|s| s.borrow().clone()),
                    Some(reconciliation.clone())
                );
                Err("interrupted readback".into())
            });
            assert!(failed.is_err());
            assert_eq!(
                OPERATION.with(|s| s.borrow().clone()),
                Some(original.clone())
            );
            Ok(())
        })
        .unwrap();
        assert!(OPERATION.with(|s| s.borrow().is_none()));
        assert!(scope("../invalid", || Ok(())).is_err());
    }
    fn identity(operation: char, nonce: char) -> Identity {
        Identity {
            schema: 1,
            operation: operation.to_string().repeat(32),
            vm_id: "12345678-1234-1234-1234-123456789abc".into(),
            manifest: "a".repeat(64),
            mode: "prepare".into(),
            files: 2,
            bundle: format!(
                "prepare-{}-{}",
                "a".repeat(64),
                nonce.to_string().repeat(32)
            ),
        }
    }
    #[test]
    fn cleanup_never_owns_installed_payload_receipts_helpers_or_ambiguous_leaves() {
        let mut ack = identity('2', '2');
        for name in [
            "0",
            "1",
            "manifest.json",
            "identity.json",
            "acknowledgements.json",
            "cleanup-result.json",
        ] {
            assert!(owned_leaf(&ack, name));
        }
        for name in [
            "2",
            "00",
            "-1",
            "0.",
            "0:stream",
            "../0",
            "prepared.json",
            "verification.json",
            "hyper-gpu-guest.exe",
            "d3d11-probe.exe",
            "diagnostics",
            "Windows",
            "bundle",
            ".retain",
        ] {
            assert!(!owned_leaf(&ack, name), "{name}");
        }
        ack.mode = "verify".into();
        assert!(owned_leaf(&ack, "verification.json"));
        assert!(!owned_leaf(&ack, "0"));
        assert!(!owned_leaf(&ack, "manifest.json"));
    }
    #[test]
    fn duplicate_or_unknown_cleanup_replies_never_touch_protected_host_storage() {
        let ack = identity('2', '2');
        assert!(
            received(
                std::slice::from_ref(&ack),
                &[ack.bundle.clone(), ack.bundle.clone()]
            )
            .is_err()
        );
        assert!(received(&[ack], &["unrequested-bundle".into()]).is_err());
        received(&[], &[]).unwrap();
    }
    #[test]
    fn acknowledgement_scope_and_namespace_are_exact_and_duplicates_never_delete() {
        let current = identity('1', '1');
        let valid = identity('2', '2');
        let mut cases = vec![vec![valid.clone(), valid.clone()], vec![current.clone()]];
        for field in 0..8 {
            let mut invalid = valid.clone();
            match field {
                0 => invalid.vm_id = "abcdefab-1234-1234-1234-123456789abc".into(),
                1 => invalid.bundle.push_str("/../prepared.json"),
                2 => invalid.manifest = "b".repeat(64),
                3 => invalid.mode = "install".into(),
                4 => invalid.files = 65537,
                5 => invalid.operation = "../operation".into(),
                6 => invalid.schema = 2,
                _ => invalid.vm_id.make_ascii_uppercase(),
            }
            cases.push(vec![invalid]);
        }
        for acknowledgements in cases {
            assert!(
                clean_with(
                    &current,
                    &acknowledgements,
                    |_| Ok(Some(vec![])),
                    |_, _| panic!("invalid acknowledgement must not delete")
                )
                .is_err()
            );
        }
        assert!(
            cleanup_plan(&current, &vec![valid; ACK_LIMIT + 1], |_| panic!(
                "bounded before inspection"
            ))
            .is_err()
        );
    }
    #[test]
    fn preflight_failure_preserves_every_bundle_and_retained_bundles_are_not_acknowledged() {
        let current = identity('1', '1');
        let a = identity('2', '2');
        let b = identity('3', '3');
        assert!(
            clean_with(
                &current,
                &[a.clone(), b.clone()],
                |i| {
                    if i == &b {
                        Err("unsafe path, ACL, unknown leaf or mismatched stored identity".into())
                    } else {
                        Ok(Some(vec![]))
                    }
                },
                |_, _| panic!("preflight must finish before any deletion")
            )
            .is_err()
        );
        let cleaned = clean_with(
            &current,
            &[a.clone(), b.clone()],
            |i| Ok((i != &a).then(Vec::new)),
            |i, _| {
                assert_eq!(i, &b);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(cleaned, vec![b.bundle]);
    }
    #[test]
    fn interrupted_leaf_deletion_retains_identity_and_resumes_without_replaying_guest_work() {
        let current = identity('1', '1');
        let ack = identity('2', '2');
        let mut remaining = vec![
            PathBuf::from("0"),
            PathBuf::from("1"),
            PathBuf::from("manifest.json"),
            PathBuf::from("identity.json"),
        ];
        let initial = remaining.clone();
        assert!(
            clean_with(
                &current,
                std::slice::from_ref(&ack),
                |_| Ok(Some(initial.clone())),
                |_, leaves| {
                    for (index, leaf) in leaves.iter().enumerate() {
                        if index == 2 {
                            return Err("interrupted deletion".into());
                        }
                        remaining.retain(|p| p != leaf);
                    }
                    Ok(())
                }
            )
            .is_err()
        );
        assert_eq!(
            remaining,
            vec![
                PathBuf::from("manifest.json"),
                PathBuf::from("identity.json")
            ]
        );
        let next = remaining.clone();
        let cleaned = clean_with(
            &current,
            std::slice::from_ref(&ack),
            |_| Ok(Some(next.clone())),
            |_, leaves| {
                for leaf in leaves {
                    remaining.retain(|p| p != &leaf);
                }
                Ok(())
            },
        )
        .unwrap();
        assert!(remaining.is_empty());
        assert_eq!(cleaned, vec![ack.bundle.clone()]);
        // Lost reply and interruption after identity deletion are idempotent.
        assert_eq!(
            clean_with(
                &current,
                std::slice::from_ref(&ack),
                |_| Ok(Some(Vec::new())),
                |_, _| Ok(())
            )
            .unwrap(),
            vec![ack.bundle]
        );
    }
    #[test]
    fn only_verified_terminal_receipts_bind_completion_to_the_exact_operation_vm_and_manifest() {
        use crate::{
            configuration_store::Revision,
            model::{Power, Settings, VmState},
        };
        let identity = identity('2', '2');
        let settings = Settings {
            low_mmio: 0,
            high_mmio: 0,
            cache_types: false,
            automatic_checkpoints: true,
        };
        let target = Target {
            vm_id: identity.vm_id.clone(),
            gpu_interface: "fixture".into(),
            enabled: true,
            vram: None,
            compute: None,
            encode: None,
            decode: None,
        };
        let initial = VmState {
            vm_id: target.vm_id.clone(),
            name: "fixture".into(),
            generation: 2,
            power: Power::Off,
            gpus: vec![],
            settings: settings.clone(),
            vram: None,
            compute: None,
            encode: None,
            decode: None,
        };
        let mut terminal = SaveRecord {
            schema: 1,
            retirement: None,
            reset: None,
            operation_id: identity.operation.clone(),
            target,
            expected: Revision::Missing,
            driver_version: "fixture".into(),
            payload_digest: Some(identity.manifest.clone()),
            plan_revision: Revision::Missing,
            intended_source: String::new(),
            publication_required: true,
            initial: initial.clone(),
            expected_power: None,
            graphics_required: None,
            initial_journal: None,
            expected_settings: settings,
            phase: SavePhase::Admitted,
        };
        assert!(!permits(&terminal, &identity));
        terminal.phase = SavePhase::EffectsStarted;
        assert!(!permits(&terminal, &identity));
        terminal.phase = SavePhase::Verified(initial.clone());
        assert!(permits(&terminal, &identity));
        terminal.phase = SavePhase::Published(initial);
        assert!(permits(&terminal, &identity));
        for field in 0..3 {
            let mut changed = identity.clone();
            match field {
                0 => changed.operation = "3".repeat(32),
                1 => changed.vm_id = "another VM".into(),
                _ => changed.manifest = "b".repeat(64),
            }
            assert!(!permits(&terminal, &changed));
        }
        terminal.payload_digest = None;
        assert!(!permits(&terminal, &identity));
    }
}
