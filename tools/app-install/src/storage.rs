//! Verified archives and bounded filesystem cleanup shared by package helpers.

use crate::Log;
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::path::PathBuf;
use std::process::Command;
use std::{
    fs,
    io::{self, Read},
    path::Path,
};

pub(super) fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 1 << 16];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

pub(super) fn system_command(name: &str) -> Result<Command, String> {
    #[cfg(windows)]
    return Ok(Command::new(crate::native::system_executable(name)?));
    #[cfg(not(windows))]
    Err(format!("{name} requires Windows"))
}

pub(super) fn exists(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("inspect {}: {error}", path.display())),
    }
}

fn is_link(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    metadata.file_type().is_symlink()
}

/// Refuse junctions and symlinks at any existing ancestor of an owned path.
pub(super) fn require_plain_path(path: &Path) -> Result<(), String> {
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if is_link(&metadata) => {
                return Err(format!(
                    "refusing linked installation path {}",
                    ancestor.display()
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("inspect {}: {error}", ancestor.display())),
        }
    }
    Ok(())
}

/// Copy every file and read its hash back before any source can be reset.
/// Reparse points fail the archive instead of silently dropping recovery evidence.
pub(super) fn copy_tree(from: &Path, to: &Path, log: &mut Log) -> Result<(), String> {
    require_plain_path(from)?;
    require_plain_path(to)?;
    fs::create_dir_all(to)
        .map_err(|error| format!("archive directory {}: {error}", to.display()))?;
    for entry in fs::read_dir(from).map_err(|error| format!("read {}: {error}", from.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        let source = entry.path();
        let metadata = fs::symlink_metadata(&source).map_err(|error| error.to_string())?;
        let target = to.join(entry.file_name());
        if is_link(&metadata) {
            return Err(format!("cannot archive linked state {}", source.display()));
        }
        if metadata.is_dir() {
            copy_tree(&source, &target, log)?;
        } else if metadata.is_file() {
            fs::copy(&source, &target)
                .map_err(|error| format!("archive {}: {error}", source.display()))?;
            if hash_file(&source)? != hash_file(&target)? {
                return Err(format!(
                    "archive readback mismatch for {}",
                    source.display()
                ));
            }
        } else {
            return Err(format!(
                "cannot archive non-file state {}",
                source.display()
            ));
        }
    }
    log.line(&format!("Archived {}", from.display()))
}

/// Remove only caller-selected, fixed product paths; never follow linked roots.
pub(super) fn remove_tree_if_present(path: &Path) -> Result<bool, String> {
    if !exists(path)? {
        return Ok(false);
    }
    require_plain_path(path)?;
    match fs::remove_dir_all(path) {
        Ok(()) => return Ok(true),
        Err(error) if error.kind() != io::ErrorKind::PermissionDenied => {
            return Err(format!("remove {}: {error}", path.display()));
        }
        Err(_) => {}
    }
    let readonly_result = clear_readonly(path);
    if readonly_result.is_ok() && fs::remove_dir_all(path).is_ok() {
        return Ok(true);
    }
    #[cfg(windows)]
    {
        if crate::native::is_elevated()? {
            let text = path.to_str().ok_or("cleanup path is not valid Unicode")?;
            run_cleanup("takeown.exe", &["/F", text, "/R", "/A", "/D", "Y"])?;
            run_cleanup(
                "icacls.exe",
                &[text, "/grant", "*S-1-5-32-544:F", "/T", "/C", "/Q"],
            )?;
        } else {
            readonly_result?;
        }
    }
    clear_readonly(path)?;
    fs::remove_dir_all(path).map_err(|error| format!("remove {}: {error}", path.display()))?;
    Ok(true)
}

#[cfg(windows)]
fn run_cleanup(command: &str, args: &[&str]) -> Result<(), String> {
    let output = system_command(command)?
        .args(args)
        .output()
        .map_err(|error| format!("{command}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{command} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
}

pub(super) fn clear_readonly(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if is_link(&metadata) {
        return Err(format!("refusing linked cleanup path {}", path.display()));
    }
    if metadata.permissions().readonly() {
        #[cfg(windows)]
        crate::native::clear_readonly(path)?;
        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(metadata.permissions().mode() | 0o200);
            fs::set_permissions(path, permissions).map_err(|error| error.to_string())?;
        }
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
            clear_readonly(&entry.map_err(|error| error.to_string())?.path())?;
        }
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn temp(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "app-install-test-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&dir).unwrap();
    dir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_lowercase_sha256() {
        let dir = temp("hash");
        fs::write(dir.join("f"), "abc").unwrap();
        assert_eq!(
            hash_file(&dir.join("f")).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn archive_copies_nested_state_and_removal_handles_readonly() {
        let dir = temp("archive");
        let source = dir.join("state");
        fs::create_dir_all(source.join("audit")).unwrap();
        fs::write(source.join("enrollment.json"), "{}").unwrap();
        fs::write(source.join("audit/events.jsonl"), "e").unwrap();
        let mut permissions = fs::metadata(source.join("enrollment.json"))
            .unwrap()
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(source.join("enrollment.json"), permissions).unwrap();
        let mut log = Log::create(&dir.join("log")).unwrap();
        copy_tree(&source, &dir.join("copy"), &mut log).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("copy/audit/events.jsonl")).unwrap(),
            "e"
        );
        assert!(remove_tree_if_present(&source).unwrap());
        assert!(!source.exists());
        assert!(!remove_tree_if_present(&source).unwrap());
        drop(log);
        clear_readonly(&dir).unwrap();
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn archive_failure_leaves_original_state_untouched() {
        let dir = temp("archive-failure");
        let source = dir.join("state");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("recovery.json"), "evidence").unwrap();
        let target = dir.join("not-a-directory");
        fs::write(&target, "occupied").unwrap();
        let mut log = Log::create(&dir.join("log")).unwrap();
        assert!(copy_tree(&source, &target.join("archive"), &mut log).is_err());
        assert_eq!(
            fs::read_to_string(source.join("recovery.json")).unwrap(),
            "evidence"
        );
        drop(log);
        fs::remove_dir_all(dir).unwrap();
    }
}
