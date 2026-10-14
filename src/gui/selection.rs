//! Per-user selection preference. This identity grants no operation authority.
#[cfg(any(windows, test))]
use std::{fs, io::Read, path::Path};

#[cfg(any(windows, test))]
fn valid_id(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(index, byte)| {
            if [8, 13, 18, 23].contains(&index) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

#[cfg(any(windows, test))]
fn read(path: &Path) -> Option<String> {
    let mut value = String::new();
    fs::File::open(path)
        .ok()?
        .take(37)
        .read_to_string(&mut value)
        .ok()?;
    valid_id(&value).then_some(value)
}

#[cfg(windows)]
fn path() -> Result<std::path::PathBuf, String> {
    let directory =
        std::env::var_os("LOCALAPPDATA").ok_or("Local user preference directory unavailable")?;
    Ok(std::path::PathBuf::from(directory).join("HyperGpuSupport/selected-vm.txt"))
}

pub(super) fn load() -> Option<String> {
    #[cfg(windows)]
    {
        read(&path().ok()?)
    }
    #[cfg(not(windows))]
    {
        None
    }
}

pub(super) fn save(id: Option<&str>) -> Result<(), String> {
    #[cfg(windows)]
    if let Some(id) = id.filter(|id| valid_id(id)) {
        let path = path()?;
        fs::create_dir_all(
            path.parent()
                .ok_or("Selection preference directory unavailable")?,
        )
        .map_err(|e| e.to_string())?;
        fs::write(path, id).map_err(|e| e.to_string())?;
    }
    #[cfg(not(windows))]
    let _ = id;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preference_accepts_only_a_bounded_vm_identity() {
        let id = "12345678-1234-1234-1234-123456789abc";
        assert!(valid_id(id));
        for invalid in [
            "",
            "VM1",
            "../../other",
            "12345678-1234-1234-1234-123456789abz",
        ] {
            assert!(!valid_id(invalid));
        }
        let directory =
            std::env::temp_dir().join(format!("hypergpu-selection-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("selection");
        fs::write(&path, id).unwrap();
        assert_eq!(read(&path).as_deref(), Some(id));
        fs::write(&path, format!("{id}\nextra")).unwrap();
        assert!(read(&path).is_none());
        fs::remove_file(&path).unwrap();
        fs::remove_dir(&directory).unwrap();
    }
}
