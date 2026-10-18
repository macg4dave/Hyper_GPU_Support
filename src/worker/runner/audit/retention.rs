//! Conservative bounded retention of terminal read audits under the admission lock.
use super::{AuditOutcome, AuditRecord};
use crate::{configuration_store::Revision, runner::Operation};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

const KEEP: usize = 32;
const COUNT: usize = 512;
const BYTES: u64 = 4 * 1024 * 1024;
const GRACE: Duration = Duration::from_secs(600);
const AGE: Duration = Duration::from_secs(7 * 86400);
const SCAN: usize = 16384;
const INTERVAL: u64 = 60;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Maintenance {
    schema: u32,
    attempted: u64,
}

fn due(last: u64, now: u64) -> bool {
    now.checked_sub(last).is_some_and(|age| age >= INTERVAL)
}

fn claim_pass(data: &Path, now: SystemTime) -> Result<bool, String> {
    let now = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let path = data.join("audit-retention.json");
    if crate::security::entry_exists(&path)? {
        let previous: Maintenance =
            serde_json::from_slice(&read(&path)?).map_err(|_| "invalid retention checkpoint")?;
        if previous.schema != 1 {
            return Err("unknown retention checkpoint".into());
        }
        if !due(previous.attempted, now) {
            return Ok(false);
        }
    }
    crate::runner::atomic_json(
        &path,
        &Maintenance {
            schema: 1,
            attempted: now,
        },
    )?;
    Ok(true)
}

struct Candidate {
    path: PathBuf,
    revision: Revision,
    bytes: u64,
    modified: SystemTime,
}

fn eligible(record: &AuditRecord, filename: &str) -> bool {
    record.schema == 1
        && filename == format!("{}.json", record.nonce)
        && record.nonce.len() <= 100
        && !record.nonce.is_empty()
        && record
            .nonce
            .bytes()
            .all(|b| b.is_ascii_digit() || b == b'-')
        && matches!(record.outcome, AuditOutcome::Succeeded)
        && matches!(
            record.operation,
            Operation::Discover
                | Operation::Observe
                | Operation::Poll
                | Operation::Completion
                | Operation::Status
        )
}

fn selection(records: &[Candidate], now: SystemTime) -> Vec<usize> {
    // Caller sorts oldest first. Minimum history and grace override every bound.
    let mut count = records.len();
    let mut bytes = records.iter().map(|r| r.bytes).sum::<u64>();
    let mut selected = Vec::new();
    for (index, record) in records
        .iter()
        .enumerate()
        .take(records.len().saturating_sub(KEEP))
    {
        let Ok(age) = now.duration_since(record.modified) else {
            continue;
        };
        if age < GRACE {
            continue;
        }
        if age > AGE || count > COUNT || bytes > BYTES {
            selected.push(index);
            count -= 1;
            bytes -= record.bytes;
        }
    }
    selected
}

fn read(path: &Path) -> Result<Vec<u8>, String> {
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
        return Err("audit metadata exceeds bound".into());
    }
    Ok(bytes)
}

fn active_reference(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    name.contains("pending")
        || name.contains("recovery")
        || name.starts_with("provider-operation")
        || name.starts_with("configuration-import.")
}

fn plan(data: &Path, active: &Path, now: SystemTime) -> Result<Vec<Candidate>, String> {
    crate::security::verify(data)?;
    // These protected stores own all active-work references. Unknown/corrupt
    // markers remain present and suspend the whole pass rather than guessed scope.
    for (index, entry) in fs::read_dir(data).map_err(|e| e.to_string())?.enumerate() {
        crate::read_context::check()?;
        if index >= SCAN {
            return Err("state scan exceeds bound".into());
        }
        let entry = entry.map_err(|e| e.to_string())?;
        if active_reference(&entry.file_name().to_string_lossy()) {
            return Ok(Vec::new());
        }
    }
    let directory = data.join("audit");
    crate::security::verify(&directory)?;
    let mut records = Vec::new();
    for (index, entry) in fs::read_dir(&directory)
        .map_err(|e| e.to_string())?
        .enumerate()
    {
        crate::read_context::check()?;
        if index >= SCAN {
            return Err("audit scan exceeds bound".into());
        }
        let path = entry.map_err(|e| e.to_string())?.path();
        if path == active || path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("invalid audit filename")?;
        // Effect/worker evidence never enters the candidate namespace.
        if !filename
            .strip_suffix(".json")
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit() || b == b'-'))
        {
            continue;
        }
        if crate::security::entry_exists(&path.with_extension("json.retain"))? {
            continue;
        }
        let bytes = read(&path)?;
        let Ok(record) = serde_json::from_slice::<AuditRecord>(&bytes) else {
            continue;
        };
        if !eligible(&record, filename) {
            continue;
        }
        let modified = fs::metadata(&path)
            .and_then(|m| m.modified())
            .map_err(|e| e.to_string())?;
        records.push(Candidate {
            path,
            revision: Revision::of(&bytes),
            bytes: bytes.len() as u64,
            modified,
        });
    }
    records.sort_by(|a, b| (a.modified, &a.path).cmp(&(b.modified, &b.path)));
    let selected = selection(&records, now);
    Ok(records
        .into_iter()
        .enumerate()
        .filter_map(|(index, record)| selected.binary_search(&index).is_ok().then_some(record))
        .collect())
}

fn remove(
    records: &[Candidate],
    mut deletion: impl FnMut(&Candidate) -> Result<(), String>,
) -> Result<(), String> {
    for record in records {
        deletion(record)?;
    }
    Ok(())
}

/// Best effort only after terminal publication. Errors leave evidence for later passes.
pub(crate) fn cleanup(data: &Path, active: &Path) -> Result<(), String> {
    crate::read_context::check()?;
    let now = SystemTime::now();
    if !claim_pass(data, now)? {
        return Ok(());
    }
    let records = plan(data, active, now)?;
    remove(&records, |record| {
        crate::read_context::check()?;
        if crate::security::entry_exists(&record.path.with_extension("json.retain"))? {
            return Ok(());
        }
        if Revision::of(&read(&record.path)?) != record.revision {
            return Err("audit changed during retention".into());
        }
        fs::remove_file(&record.path).map_err(|e| e.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maintenance_is_rate_limited_and_future_checkpoints_preserve_records() {
        assert!(!due(100, 100));
        assert!(!due(100, 159));
        assert!(due(100, 160));
        assert!(!due(160, 100));
    }
    #[test]
    fn active_or_ambiguous_references_suspend_retention_without_parsing_them_as_completed() {
        for name in [
            "operation-recovery.json",
            "operation-recovery-12345678-1234-1234-1234-123456789abc.json",
            "installation-pending.json",
            "pair-pending.json",
            "configuration-import.json",
            "configuration-import.partial",
            "import-pending.partial",
            "provider-operation-unknown.json",
            "OPERATION-RECOVERY.JSON",
        ] {
            assert!(active_reference(name), "{name}");
        }
        for name in [
            "audit",
            "staging",
            "operation.lock",
            "enrollment.json",
            "nonces",
        ] {
            assert!(!active_reference(name));
        }
    }
    fn candidate(index: usize, bytes: u64, modified: SystemTime) -> Candidate {
        Candidate {
            path: format!("{index}.json").into(),
            revision: Revision::Missing,
            bytes,
            modified,
        }
    }
    #[test]
    fn retention_bounds_preserve_minimum_and_recent_history() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 86400);
        let old = now - AGE - Duration::from_secs(1);
        let records: Vec<_> = (0..600)
            .map(|i| candidate(i, 100, now - Duration::from_secs(601)))
            .collect();
        assert_eq!(selection(&records, now), (0..88).collect::<Vec<_>>());
        let records: Vec<_> = (0..100).map(|i| candidate(i, BYTES / 40, old)).collect();
        assert_eq!(selection(&records, now).len(), 68);
        let recent: Vec<_> = (0..600)
            .map(|i| candidate(i, BYTES, now - Duration::from_secs(599)))
            .collect();
        assert!(selection(&recent, now).is_empty());
        let future: Vec<_> = (0..600).map(|i| candidate(i, BYTES, now + GRACE)).collect();
        assert!(selection(&future, now).is_empty());
        let large: Vec<_> = (0..80)
            .map(|i| candidate(i, BYTES / 40, now - GRACE))
            .collect();
        assert_eq!(selection(&large, now).len(), 40);
    }
    #[test]
    fn only_exact_terminal_read_records_are_eligible() {
        let original = serde_json::json!({"schema":1,"nonce":"123-4-5","operation":"Status","target":null,"outcome":"Succeeded"});
        for operation in [
            "Status",
            "Discover",
            "Observe",
            "Poll",
            "Completion",
            "Apply",
            "Verify",
            "Plan",
            "RemovalPlan",
        ] {
            for outcome in ["Started", "Failed", "Succeeded"] {
                let mut value = original.clone();
                value["operation"] = operation.into();
                value["outcome"] = outcome.into();
                let record: AuditRecord = serde_json::from_value(value).unwrap();
                assert_eq!(
                    eligible(&record, "123-4-5.json"),
                    outcome == "Succeeded"
                        && matches!(
                            operation,
                            "Status" | "Discover" | "Observe" | "Poll" | "Completion"
                        )
                );
                assert!(!eligible(&record, "different.json"));
            }
        }
        // Retention skips undecodable records, preserving retired operation evidence.
        for operation in ["PreliminaryPlan", "ReinstallPlan", "VerifyPlan"] {
            for outcome in ["Started", "Failed", "Succeeded"] {
                let mut value = original.clone();
                value["operation"] = operation.into();
                value["outcome"] = outcome.into();
                assert!(serde_json::from_value::<AuditRecord>(value).is_err());
            }
        }
        let mut malformed = original;
        malformed["retained"] = true.into();
        assert!(serde_json::from_value::<AuditRecord>(malformed).is_err());
    }
    #[test]
    fn interrupted_deletion_stops_and_a_new_pass_can_resume_without_replaying_work() {
        let records: Vec<_> = (0..4)
            .map(|i| candidate(i, 1, SystemTime::UNIX_EPOCH))
            .collect();
        let mut removed = Vec::new();
        assert!(
            remove(&records, |r| {
                if removed.len() == 2 {
                    return Err("delete denied".into());
                }
                removed.push(r.path.clone());
                Ok(())
            })
            .is_err()
        );
        assert_eq!(removed.len(), 2);
        remove(&records[2..], |r| {
            removed.push(r.path.clone());
            Ok(())
        })
        .unwrap();
        assert_eq!(removed.len(), 4);
    }
}
