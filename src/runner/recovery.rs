//! VM-bound protected receipts with guarded, resumable legacy migration.
use super::*;
use crate::configuration_store::SaveRecord;

const PREFIX: &str = "operation-recovery-";

// Older installed readers reject schema 2 instead of overlooking VM receipts.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceiptIndex {
    schema: u32,
    per_vm: bool,
}

fn index() -> ReceiptIndex {
    ReceiptIndex {
        schema: 2,
        per_vm: true,
    }
}

pub(crate) fn vm_receipt_name(prefix: &str, id: &str) -> Result<String, String> {
    let bytes = id.as_bytes();
    if bytes.len() != 36
        || bytes.iter().enumerate().any(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                *b != b'-'
            } else {
                !b.is_ascii_hexdigit()
            }
        })
    {
        return Err("recovery requires a VM GUID".into());
    }
    Ok(format!("{prefix}{}.json", id.to_ascii_lowercase()))
}

pub(crate) fn new_recovery_path(id: &str) -> Result<PathBuf, String> {
    Ok(data_directory()?.join(vm_receipt_name(PREFIX, id)?))
}

pub(crate) fn read_recovery_at(path: &Path) -> Result<Option<SaveRecord>, String> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
        Ok(metadata) if !metadata.is_file() => return Err("recovery record is not a file".into()),
        Ok(_) => {}
    }
    crate::security::verify(path)?;
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(FRAME_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > FRAME_LIMIT {
        return Err("recovery record exceeds input limit".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| format!("invalid protected recovery record: {e}"))?;
    if path == super::recovery_path()? && value.get("schema").and_then(|v| v.as_u64()) == Some(2) {
        let marker: ReceiptIndex = serde_json::from_value(value).map_err(|e| e.to_string())?;
        if !marker.per_vm {
            return Err("invalid VM recovery index".into());
        }
        return Ok(None);
    }
    let record: SaveRecord = serde_json::from_value(value).map_err(|e| e.to_string())?;
    validate(
        &record,
        path.file_name()
            .and_then(|n| n.to_str())
            .ok_or("invalid recovery filename")?,
    )?;
    Ok(Some(record))
}

fn validate(record: &SaveRecord, name: &str) -> Result<(), String> {
    if record.schema != 1 {
        return Err("unsupported protected recovery version".into());
    }
    record.target.validate()?;
    if record.initial.vm_id != record.target.vm_id
        || matches!(&record.phase, crate::configuration_store::SavePhase::Verified(state) | crate::configuration_store::SavePhase::Published(state) if state.vm_id != record.target.vm_id)
    {
        return Err("protected recovery VM bindings conflict".into());
    }
    if name != "operation-recovery.json" && name != vm_receipt_name(PREFIX, &record.target.vm_id)? {
        return Err("protected recovery filename and VM identity differ".into());
    }
    if record.operation_id.is_empty() {
        return Err("protected recovery operation identity is absent".into());
    }
    Ok(())
}

/// Read all retained VM holds. Unreadable or ambiguous scope fails closed.
pub fn recovery_records() -> Result<Vec<SaveRecord>, String> {
    let root = data_directory()?;
    match fs::symlink_metadata(&root) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.to_string()),
        Ok(_) => crate::security::verify(&root)?,
    }
    let mut records = Vec::new();
    let legacy_record = read_recovery_at(&super::recovery_path()?)?;
    if let Some(record) = &legacy_record {
        records.push(record.clone());
    }
    for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        if !name.to_string_lossy().starts_with(PREFIX)
            || name.to_string_lossy().ends_with(".partial")
        {
            continue;
        }
        // A disappearing entry is also a failed snapshot, never proof of absence.
        let record = read_recovery_at(&entry.path())?.ok_or("recovery changed during read")?;
        // Interrupted legacy migration may leave two identical copies. Only that
        // exact legacy/canonical pair is one hold; conflicting copies fail closed.
        if let Some(legacy) = &legacy_record
            && same_record(legacy, &record)?
        {
            continue;
        }
        if records.len() >= 128 {
            return Err("protected recovery count exceeds limit".into());
        }
        records.push(record);
    }
    validate_unique(&records)?;
    records.sort_by(|a, b| a.target.vm_id.cmp(&b.target.vm_id));
    Ok(records)
}

fn validate_unique(records: &[SaveRecord]) -> Result<(), String> {
    let mut vms = std::collections::BTreeSet::new();
    let mut operations = std::collections::BTreeSet::new();
    for record in records {
        if !vms.insert(record.target.vm_id.to_ascii_lowercase())
            || !operations.insert(&record.operation_id)
        {
            return Err("duplicate protected recovery identity; retain all receipts".into());
        }
    }
    Ok(())
}

fn same_record(a: &SaveRecord, b: &SaveRecord) -> Result<bool, String> {
    Ok(serde_json::to_value(a).map_err(|e| e.to_string())?
        == serde_json::to_value(b).map_err(|e| e.to_string())?)
}

/// Under the common lock and admission audit, preserve a legacy hold before
/// installing the barrier. No VM work starts until both writes/readback succeed.
pub(crate) fn prepare_vm_recovery() -> Result<(), String> {
    let legacy = super::recovery_path()?;
    let Some(record) = read_recovery_at(&legacy)? else {
        return Ok(());
    };
    recovery_records()?;
    let path = new_recovery_path(&record.target.vm_id)?;
    migrate_legacy(
        &record,
        read_recovery_at(&path)?.as_ref(),
        || atomic_json(&path, &record),
        || read_recovery_at(&path)?.ok_or("migrated receipt disappeared".into()),
        || atomic_json(&legacy, &index()),
    )
}

fn migrate_legacy(
    original: &SaveRecord,
    existing: Option<&SaveRecord>,
    write_copy: impl FnOnce() -> Result<(), String>,
    read_copy: impl FnOnce() -> Result<SaveRecord, String>,
    write_index: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    if let Some(existing) = existing {
        if !same_record(original, existing)? {
            return Err("legacy recovery migration conflicts; retain both receipts".into());
        }
    } else {
        write_copy()?;
    }
    if !same_record(original, &read_copy()?)? {
        return Err("legacy recovery migration readback changed; retain original".into());
    }
    write_index()
}

/// Select recovery by stable VM identity, including an existing legacy receipt.
pub fn recovery_for_vm(id: &str) -> Result<Option<SaveRecord>, String> {
    Ok(recovery_records()?
        .into_iter()
        .find(|r| r.target.vm_id.eq_ignore_ascii_case(id)))
}

/// Operation IDs select existing receipts; they are never interpreted as paths.
pub fn recovery_for_operation(id: &str) -> Result<Option<SaveRecord>, String> {
    Ok(recovery_records()?
        .into_iter()
        .find(|r| r.operation_id == id))
}

pub(crate) fn recovery_record_path(record: &SaveRecord) -> Result<PathBuf, String> {
    let legacy = super::recovery_path()?;
    if read_recovery_at(&legacy)?.is_some_and(|r| {
        r.operation_id == record.operation_id && r.target.vm_id == record.target.vm_id
    }) {
        Ok(legacy)
    } else {
        new_recovery_path(&record.target.vm_id)
    }
}

/// Persist the version barrier before effects; never replace a legacy VM's hold.
pub(crate) fn write_recovery_at(path: &Path, record: &SaveRecord) -> Result<(), String> {
    validate(
        record,
        path.file_name()
            .and_then(|n| n.to_str())
            .ok_or("invalid recovery filename")?,
    )?;
    let legacy = super::recovery_path()?;
    if path != legacy && read_recovery_at(&legacy)?.is_none() {
        atomic_json(&legacy, &index())?;
    }
    atomic_json(path, record)
}

/// Retire only the terminal command's bound receipt, after its durable audit.
pub(crate) fn retire_recovery_at(path: &Path, operation: &str) -> Result<(), String> {
    let record = read_recovery_at(path)?.ok_or("terminal recovery receipt disappeared")?;
    if recovery_record_path(&record)? != path {
        return Err("terminal recovery identity changed; retain all receipts".into());
    }
    let legacy = super::recovery_path()?;
    retire_bound(
        &record,
        path,
        &legacy,
        operation,
        &recovery_records()?,
        |path, replacement| {
            if replacement {
                atomic_json(path, &index())
            } else {
                crate::security::verify(path)?;
                fs::remove_file(path).map_err(|e| e.to_string())
            }
        },
    )
}

fn retire_bound(
    record: &SaveRecord,
    path: &Path,
    legacy: &Path,
    operation: &str,
    records: &[SaveRecord],
    mut change: impl FnMut(&Path, bool) -> Result<(), String>,
) -> Result<(), String> {
    validate_unique(records)?;
    if record.operation_id != operation {
        return Err("terminal recovery operation changed; retain all receipts".into());
    }
    let current = records
        .iter()
        .find(|r| r.operation_id == operation)
        .ok_or("terminal recovery snapshot is absent; retain all receipts")?;
    if serde_json::to_value(current).map_err(|e| e.to_string())?
        != serde_json::to_value(record).map_err(|e| e.to_string())?
    {
        return Err("terminal recovery snapshot changed; retain all receipts".into());
    }
    let others = records.iter().any(|r| r.operation_id != operation);
    if path == legacy && others {
        return change(legacy, true);
    }
    change(path, false)?;
    if path != legacy && !others {
        // The barrier contains no operation evidence. Every VM receipt is retired.
        change(legacy, false)?;
    }
    Ok(())
}

/// Shared authority/import uncertainty remains global; VM effects block their own VM.
pub(crate) fn require_no_vm_recovery(id: &str) -> Result<(), String> {
    if pending_pair()?.is_some() || import_record()?.is_some() {
        return Err("resolve shared enrollment/import recovery before GPU effects".into());
    }
    let records = recovery_records()?;
    if records
        .iter()
        .any(|r| r.target.vm_id.eq_ignore_ascii_case(id))
    {
        return Err("selected VM requires recovery; other VM receipts are retained".into());
    }
    if records.len() >= 128 {
        return Err("VM recovery capacity is exhausted; retain all receipts".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration_store::{Revision, SavePhase};
    use std::collections::BTreeMap;

    fn receipt(last: char) -> SaveRecord {
        let target = Target {
            vm_id: format!("12345678-1234-1234-1234-123456789ab{last}"),
            gpu_interface: r"\\?\PCI#fixture\GPUPARAV".into(),
            enabled: false,
            vram: None,
            compute: None,
            encode: None,
            decode: None,
        };
        let initial = VmState {
            vm_id: target.vm_id.clone(),
            name: "Same name".into(),
            generation: 2,
            power: Power::Off,
            gpus: vec![],
            vram: None,
            compute: None,
            encode: None,
            decode: None,
            settings: Settings {
                low_mmio: 0,
                high_mmio: 0,
                cache_types: false,
                automatic_checkpoints: true,
            },
        };
        SaveRecord {
            schema: 1,
            retirement: None,
            reset: None,
            operation_id: format!("operation-{last}"),
            target,
            expected: Revision::Missing,
            driver_version: "fixture".into(),
            payload_digest: None,
            plan_revision: Revision::Missing,
            intended_source: String::new(),
            publication_required: true,
            expected_power: None,
            graphics_required: None,
            initial_journal: None,
            expected_settings: initial.settings.clone(),
            initial,
            phase: SavePhase::EffectsStarted,
        }
    }

    #[test]
    fn legacy_migration_cannot_release_the_original_before_durable_identical_readback() {
        use std::cell::RefCell;
        let original = receipt('a');
        let before = serde_json::to_value(&original).unwrap();
        for fail in [None, Some("copy"), Some("read"), Some("index")] {
            let trace = RefCell::new(Vec::new());
            let result = migrate_legacy(
                &original,
                None,
                || {
                    trace.borrow_mut().push("copy");
                    if fail == Some("copy") {
                        Err("copy failed".into())
                    } else {
                        Ok(())
                    }
                },
                || {
                    trace.borrow_mut().push("read");
                    if fail == Some("read") {
                        Err("read denied".into())
                    } else {
                        Ok(original.clone())
                    }
                },
                || {
                    trace.borrow_mut().push("index");
                    if fail == Some("index") {
                        Err("index failed".into())
                    } else {
                        Ok(())
                    }
                },
            );
            assert_eq!(result.is_ok(), fail.is_none());
            assert_eq!(
                &*trace.borrow(),
                match fail {
                    Some("copy") => &["copy"][..],
                    Some("read") => &["copy", "read"][..],
                    _ => &["copy", "read", "index"][..],
                }
            );
            assert_eq!(serde_json::to_value(&original).unwrap(), before);
        }
        let trace = RefCell::new(Vec::new());
        migrate_legacy(
            &original,
            Some(&original),
            || panic!("an already durable identical copy must not be rewritten"),
            || {
                trace.borrow_mut().push("read");
                Ok(original.clone())
            },
            || {
                trace.borrow_mut().push("index");
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(&*trace.borrow(), &["read", "index"]);
        let conflicting = receipt('b');
        assert!(
            migrate_legacy(
                &original,
                Some(&conflicting),
                || panic!("conflicting copy must not be replaced"),
                || panic!("conflict stops readback"),
                || panic!("conflict retains the original legacy receipt")
            )
            .is_err()
        );
        assert!(
            migrate_legacy(
                &original,
                None,
                || Ok(()),
                || Ok(conflicting),
                || panic!("changed readback must retain the legacy receipt")
            )
            .is_err()
        );
    }

    #[test]
    fn vm_receipt_binding_rejects_aliases_corruption_and_duplicate_legacy_scope() {
        let a = receipt('a');
        let b = receipt('b');
        let name = vm_receipt_name(PREFIX, &a.target.vm_id).unwrap();
        validate(&a, &name).unwrap();
        validate(&a, "operation-recovery.json").unwrap();
        assert!(validate(&b, &name).is_err());
        for bad in ["../outside", "", "12345678-1234-1234-1234-123456789abg"] {
            assert!(vm_receipt_name(PREFIX, bad).is_err());
        }
        assert_eq!(
            vm_receipt_name(PREFIX, &a.target.vm_id.to_ascii_uppercase()).unwrap(),
            name
        );
        validate_unique(&[a.clone(), b.clone()]).unwrap();
        assert!(validate_unique(&[a.clone(), a.clone()]).is_err());
        let mut duplicate_operation = b.clone();
        duplicate_operation.operation_id = a.operation_id.clone();
        assert!(validate_unique(&[a.clone(), duplicate_operation]).is_err());
        for corrupt in 0..5 {
            let mut record = a.clone();
            match corrupt {
                0 => record.schema = 99,
                1 => record.target.vm_id = "../outside".into(),
                2 => record.operation_id.clear(),
                3 => record.initial.vm_id = b.target.vm_id.clone(),
                _ => record.phase = SavePhase::Verified(b.initial.clone()),
            }
            assert!(validate(&record, &name).is_err());
        }
    }

    #[test]
    fn completing_one_vm_preserves_other_receipts_and_the_older_reader_barrier() {
        for legacy_a in [false, true] {
            let a = receipt('a');
            let b = receipt('b');
            let legacy = PathBuf::from("operation-recovery.json");
            let a_path = if legacy_a {
                legacy.clone()
            } else {
                PathBuf::from(vm_receipt_name(PREFIX, &a.target.vm_id).unwrap())
            };
            let b_path = PathBuf::from(vm_receipt_name(PREFIX, &b.target.vm_id).unwrap());
            let a_bytes = serde_json::to_vec(&a).unwrap();
            let b_bytes = serde_json::to_vec(&b).unwrap();
            let mut files = BTreeMap::from([
                (legacy.clone(), serde_json::to_vec(&index()).unwrap()),
                (a_path.clone(), a_bytes.clone()),
                (b_path.clone(), b_bytes),
            ]);
            retire_bound(
                &b,
                &b_path,
                &legacy,
                &b.operation_id,
                &[a.clone(), b.clone()],
                |path, replace| {
                    assert!(!replace);
                    files.remove(path).ok_or("missing receipt")?;
                    Ok(())
                },
            )
            .unwrap();
            assert_eq!(files.get(&a_path).unwrap(), &a_bytes);
            assert!(!files.contains_key(&b_path));
            assert!(files.contains_key(&legacy));
            retire_bound(
                &a,
                &a_path,
                &legacy,
                &a.operation_id,
                std::slice::from_ref(&a),
                |path, replace| {
                    assert!(!replace);
                    files.remove(path).ok_or("missing receipt")?;
                    Ok(())
                },
            )
            .unwrap();
            assert!(files.is_empty());
        }
    }

    #[test]
    fn legacy_retirement_keeps_the_barrier_and_failed_retirement_does_not_touch_other_vms() {
        let a = receipt('a');
        let b = receipt('b');
        let legacy = Path::new("operation-recovery.json");
        let records = vec![a.clone(), b];
        let mut changed = Vec::new();
        assert!(
            retire_bound(&a, legacy, legacy, "wrong-operation", &records, |_, _| {
                panic!("mismatched completion must not write or remove anything")
            })
            .is_err()
        );
        assert!(
            retire_bound(
                &a,
                legacy,
                legacy,
                &a.operation_id,
                &records,
                |path, replace| {
                    changed.push((path.to_owned(), replace));
                    Err("injected durable replacement failure".into())
                }
            )
            .is_err()
        );
        assert_eq!(changed, vec![(legacy.to_owned(), true)]);
        changed.clear();
        retire_bound(
            &a,
            legacy,
            legacy,
            &a.operation_id,
            &records,
            |path, replace| {
                changed.push((path.to_owned(), replace));
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(changed, vec![(legacy.to_owned(), true)]);
        // Index schema is deliberately rejected by the old singleton SaveRecord reader.
        assert!(
            serde_json::from_value::<SaveRecord>(serde_json::to_value(index()).unwrap()).is_err()
        );
    }
}
