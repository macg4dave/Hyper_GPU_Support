//! Fixed Windows known-folder store. Caller-selected publication paths are absent.
use super::{Revision, SavePhase, SaveRecord, StoreSnapshot};
use crate::{model::Configuration, payload, runner, security};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use windows::Win32::UI::Shell::FOLDERID_ProgramData;

fn directories() -> Result<[PathBuf; 3], String> {
    let root = runner::folder(&FOLDERID_ProgramData)?.join("HyperGpuSupport");
    Ok([root.clone(), root.join("config"), root.join("config/vms")])
}
fn verify_directories() -> Result<Option<PathBuf>, String> {
    let paths = directories()?;
    for path in &paths {
        match fs::symlink_metadata(path) {
            Ok(metadata) => {
                if !metadata.is_dir() {
                    return Err("configuration ancestor is not a directory".into());
                }
                security::verify(path)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("inspect configuration directory: {error}")),
        }
    }
    Ok(Some(paths[2].clone()))
}
/// Installer alone creates directories. Never fix an untrusted existing directory.
pub(crate) fn install(client: &str) -> Result<(), String> {
    if !crate::process::is_elevated()? {
        return Err("configuration store installation requires elevation".into());
    }
    for path in directories()? {
        match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if !metadata.is_dir() {
                    return Err("configuration ancestor is not a directory".into());
                }
                security::verify(&path)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                security::create_directory(&path, client)?
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}
/// Read bounded committed intent without audit, enrollment, credentials or writes.
pub(crate) fn read() -> StoreSnapshot {
    let mut snapshot = StoreSnapshot::default();
    let root = match verify_directories() {
        Ok(Some(root)) => root,
        Ok(None) => return snapshot,
        Err(error) => {
            snapshot.errors.insert("store".into(), error);
            return snapshot;
        }
    };
    let entries = match fs::read_dir(&root) {
        Ok(entries) => entries,
        Err(error) => {
            snapshot.errors.insert("store".into(), error.to_string());
            return snapshot;
        }
    };
    let mut count = 0;
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                snapshot.errors.insert("store".into(), error.to_string());
                continue;
            }
        };
        let path = entry.path();
        let Some(filename) = path.file_name().and_then(|f| f.to_str()) else {
            snapshot
                .errors
                .insert("store".into(), "configuration filename is not UTF-8".into());
            continue;
        };
        if !filename.to_ascii_lowercase().ends_with(".toml") {
            continue;
        }
        count += 1;
        if count > 128 {
            snapshot
                .errors
                .insert("store".into(), "configuration target limit exceeded".into());
            break;
        }
        snapshot.insert(
            filename,
            read_source(&path)
                .and_then(|source| source.ok_or("configuration disappeared during read".into())),
        );
    }
    snapshot
}
fn read_source(path: &Path) -> Result<Option<String>, String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.is_file() {
                return Err("configuration destination is not a regular file".into());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    }
    security::verify(path)?;
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65536 {
        return Err("per-VM configuration exceeds the 64 KiB input limit".into());
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| "configuration is not UTF-8".into())
}
/// Exact independent revision read while caller holds the common operation lock.
pub(crate) fn revision(id: &str) -> Result<Revision, String> {
    if id.len() != 36
        || id.bytes().enumerate().any(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b != b'-'
            } else {
                !b.is_ascii_hexdigit() || b.is_ascii_uppercase()
            }
        })
    {
        return Err("configuration requires a canonical VM GUID".into());
    }
    let root = verify_directories()?.ok_or("configuration store is not installed")?;
    Ok(read_source(&root.join(format!("{id}.toml")))?
        .map_or(Revision::Missing, |source| Revision::of(source.as_bytes())))
}
/// Admission enforces the same aggregate bound as the directory reader.
pub(crate) fn check_capacity(ids: &[String]) -> Result<(), String> {
    let store = read();
    if let Some(error) = store.errors.get("store") {
        return Err(format!("configuration store unavailable: {error}"));
    }
    let existing = store.documents.len()
        + store
            .errors
            .keys()
            .filter(|name| name.ends_with(".toml"))
            .count();
    let mut new = 0;
    for id in ids.iter().collect::<std::collections::BTreeSet<_>>() {
        if revision(id)? == Revision::Missing {
            new += 1;
        }
    }
    if existing + new > 128 {
        return Err("resulting configuration store would exceed 128 targets".into());
    }
    Ok(())
}
/// Flush/protect/recheck/replace. Caller is the fixed elevated worker under lock.
pub(crate) fn publish(record: &SaveRecord, client: &str) -> Result<(), String> {
    if !crate::process::is_elevated()? {
        return Err("configuration publication requires the elevated worker".into());
    }
    if !matches!(
        record.phase,
        SavePhase::Verified(_) | SavePhase::Published(_)
    ) {
        return Err("only independently verified operations may publish intent".into());
    }
    let filename = format!("{}.toml", record.target.vm_id);
    let intended = Configuration::parse_vm_file(&record.intended_source, &filename)?;
    if intended.targets != vec![record.target.clone()] {
        return Err("configuration receipt output mismatch".into());
    }
    let current = revision(&record.target.vm_id)?;
    if current == Revision::of(record.intended_source.as_bytes()) {
        return Ok(());
    }
    if matches!(record.phase, SavePhase::Published(_)) {
        return Err("previously committed output changed; reconciliation required".into());
    }
    if current != record.expected {
        return Err("committed configuration changed externally; save blocked".into());
    }
    publish_document(&filename, &record.intended_source, &record.expected, client)
}
/// Explicit import is distinct from effects publication; never invent a verified
/// GPU result to authorize importing a desired-state document.
pub(crate) fn publish_import(filename: &str, source: &str, client: &str) -> Result<(), String> {
    publish_document(filename, source, &Revision::Missing, client)
}
fn publish_document(
    filename: &str,
    source: &str,
    expected: &Revision,
    client: &str,
) -> Result<(), String> {
    if !crate::process::is_elevated()? {
        return Err("publication requires the restricted elevated worker".into());
    }
    let target = Configuration::parse_vm_file(source, filename)?
        .targets
        .remove(0);
    let root = verify_directories()?.ok_or("configuration store is not installed")?;
    if revision(&target.vm_id)? != *expected {
        return Err("committed configuration changed before publication".into());
    }
    let path = root.join(filename);
    let temp = path.with_extension("toml.partial");
    if fs::symlink_metadata(&temp).is_ok() {
        security::verify(&temp)?;
        fs::remove_file(&temp).map_err(|e| e.to_string())?;
    }
    payload::no_reparse(&root)?;
    let mut file = security::create_file(&temp, client)?;
    file.write_all(source.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())?;
    drop(file);
    runner::protect(&temp, client)?;
    if revision(&target.vm_id)? != *expected {
        return Err("committed configuration changed before publication".into());
    }
    if *expected == Revision::Missing {
        publish_absent(&temp, &path)?;
    } else {
        crate::guest::replace_file(&temp, &path)?;
    }
    security::verify(&path)?;
    if revision(&target.vm_id)? != Revision::of(source.as_bytes()) {
        return Err("configuration publication readback mismatch".into());
    }
    Ok(())
}
#[allow(unsafe_code)]
fn publish_absent(from: &Path, to: &Path) -> Result<(), String> {
    use windows::{
        Win32::Storage::FileSystem::{MOVEFILE_WRITE_THROUGH, MoveFileExW},
        core::PCWSTR,
    };
    let from: Vec<u16> = from
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let to: Vec<u16> = to
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    // SAFETY: fixed-store terminated paths. Omitting REPLACE_EXISTING makes the
    // publication itself fail atomically if an external destination appeared.
    unsafe {
        MoveFileExW(
            PCWSTR(from.as_ptr()),
            PCWSTR(to.as_ptr()),
            MOVEFILE_WRITE_THROUGH,
        )
    }
    .map_err(|e| e.to_string())
}
