//! Per-user, last-observed OS metadata for icons only; never operation authority.
#[cfg(any(windows, test))]
use std::{fs, io::Read, path::Path};

#[cfg(any(windows, test))]
fn valid_name(name: &str) -> bool {
    !name.trim().is_empty() && name.len() <= 1024 && !name.chars().any(char::is_control)
}

#[cfg(any(windows, test))]
fn read(path: &Path) -> Option<String> {
    let mut name = String::new();
    fs::File::open(path)
        .ok()?
        .take(1025)
        .read_to_string(&mut name)
        .ok()?;
    valid_name(&name).then_some(name)
}

/// Missing observations retain the last value. A fresh unfamiliar OS replaces it
/// too, so an OS change cannot leave an old Windows/Linux icon behind.
#[cfg(any(windows, test))]
fn resolve(directory: &Path, id: &str, fresh: Option<&str>) -> Result<Option<String>, String> {
    if !super::selection::valid_id(id) {
        return Ok(None);
    }
    let path = directory.join(format!("{}.txt", id.to_ascii_lowercase()));
    let saved = read(&path);
    match fresh {
        Some(name) if valid_name(name) => {
            if saved.as_deref() != Some(name) {
                fs::create_dir_all(directory).map_err(|e| e.to_string())?;
                fs::write(path, name).map_err(|e| e.to_string())?;
            }
            Ok(Some(name.into()))
        }
        Some(_) => Ok(None),
        None => Ok(saved),
    }
}

/// Called only in GUI background discovery, including setup and status refresh.
/// The protected runner's inventory and all GPU intent remain unchanged.
#[cfg(windows)]
pub(super) fn remember(
    inventory: &mut hyper_gpu_support::gui_model::Inventory,
) -> Result<(), String> {
    let directory =
        std::env::var_os("LOCALAPPDATA").ok_or("Local user preference directory unavailable")?;
    let directory = std::path::PathBuf::from(directory).join("HyperGpuSupport/guest-os");
    remember_in(&directory, &mut inventory.discovery)
}

#[cfg(any(windows, test))]
pub(super) fn remember_in(
    directory: &Path,
    discovery: &mut hyper_gpu_support::model::Discovery,
) -> Result<(), String> {
    let mut failure = None;
    for vm in &discovery.vms {
        let fresh = discovery.guest_os.get(&vm.vm_id);
        match resolve(directory, &vm.vm_id, fresh.map(String::as_str)) {
            Ok(Some(name)) => {
                discovery.guest_os.insert(vm.vm_id.clone(), name);
            }
            Ok(None) => {}
            // Keep fresh discovery usable even when optional persistence fails.
            Err(error) => failure = Some(error),
        }
    }
    failure.map_or(Ok(()), Err)
}

#[cfg(test)]
mod tests {
    use super::*;
    const ID: &str = "12345678-1234-1234-1234-123456789abc";
    const OTHER: &str = "87654321-4321-4321-4321-cba987654321";

    #[test]
    fn saved_os_survives_missing_metadata_and_fresh_os_changes_replace_it() {
        let directory = std::env::temp_dir().join(format!("hypergpu-os-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        assert_eq!(resolve(&directory, ID, None).unwrap(), None);
        resolve(&directory, ID, Some("Microsoft Windows 11 Pro")).unwrap();
        // No in-memory cache: a stopped VM/reopened app reads the saved file.
        assert_eq!(
            resolve(&directory, &ID.to_ascii_uppercase(), None)
                .unwrap()
                .as_deref(),
            Some("Microsoft Windows 11 Pro")
        );
        assert_eq!(resolve(&directory, OTHER, None).unwrap(), None);
        for name in ["Ubuntu 24.04", "FreeBSD"] {
            resolve(&directory, ID, Some(name)).unwrap();
            assert_eq!(
                resolve(&directory, ID, None).unwrap().as_deref(),
                Some(name)
            );
        }
        let path = directory.join(format!("{ID}.txt"));
        for corrupt in [vec![0xff], vec![b'x'; 1025], b"\n".to_vec()] {
            fs::write(&path, corrupt).unwrap();
            assert_eq!(resolve(&directory, ID, None).unwrap(), None);
        }
        // A valid observation repairs a malformed optional cache.
        resolve(&directory, ID, Some("Windows")).unwrap();
        assert_eq!(read(&path).as_deref(), Some("Windows"));
        assert_eq!(
            resolve(&directory, "../escape", Some("Linux")).unwrap(),
            None
        );
        assert_eq!(resolve(&directory, ID, Some(" ")).unwrap(), None);
        assert_eq!(read(&path).as_deref(), Some("Windows"));
        // A blocked destination reports failure rather than claiming it was saved.
        assert!(resolve(&path, ID, Some("Linux")).is_err());
        fs::remove_file(path).unwrap();
        fs::remove_dir(directory).unwrap();
    }
}
