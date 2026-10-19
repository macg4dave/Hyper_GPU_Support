//! Per-user selection preference. This identity grants no operation authority.
#[cfg(any(windows, test))]
use hyper_gpu_support::model::valid_vm_id;
#[cfg(any(windows, test))]
use std::{fs, io::Read, path::Path};

#[cfg(any(windows, test))]
fn read(path: &Path) -> Option<String> {
    let mut value = String::new();
    fs::File::open(path)
        .ok()?
        .take(37)
        .read_to_string(&mut value)
        .ok()?;
    // Discovery uses canonical lowercase GUIDs, including after a VM rename.
    valid_vm_id(&value).then(|| value.to_ascii_lowercase())
}

#[cfg(any(windows, test))]
fn write(path: &Path, id: &str) -> Result<(), String> {
    if !valid_vm_id(id) {
        return Err(hyper_gpu_support::ui_text::common::errors::selection_requires_guid().into());
    }
    fs::create_dir_all(
        path.parent()
            .ok_or(hyper_gpu_support::ui_text::common::errors::selection_directory_unavailable())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(path, id.to_ascii_lowercase()).map_err(|e| e.to_string())
}

#[cfg(windows)]
fn path() -> Result<std::path::PathBuf, String> {
    let directory = std::env::var_os("LOCALAPPDATA")
        .ok_or(hyper_gpu_support::ui_text::common::errors::preference_directory_unavailable())?;
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
    if let Some(id) = id {
        write(&path()?, id)?;
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
        assert!(valid_vm_id(id));
        for invalid in [
            "",
            "VM1",
            "../../other",
            "12345678-1234-1234-1234-123456789abz",
        ] {
            assert!(!valid_vm_id(invalid));
        }
        let directory =
            std::env::temp_dir().join(format!("hypergpu-selection-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("selection");
        assert!(read(&path).is_none());
        write(&path, id).unwrap();
        assert_eq!(read(&path).as_deref(), Some(id));
        // Both newly saved and older uppercase preferences restore canonically.
        write(&path, &id.to_ascii_uppercase()).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), id);
        fs::write(&path, id.to_ascii_uppercase()).unwrap();
        assert_eq!(read(&path).as_deref(), Some(id));
        assert!(write(&path, "VM1").is_err());
        assert_eq!(read(&path).as_deref(), Some(id));
        let other = "87654321-4321-4321-4321-cba987654321";
        write(&path, other).unwrap();
        assert_eq!(read(&path).as_deref(), Some(other));
        fs::write(&path, format!("{id}\nextra")).unwrap();
        assert!(read(&path).is_none());
        fs::write(&path, [0xff]).unwrap();
        assert!(read(&path).is_none());
        // A non-directory parent reports failure instead of claiming persistence.
        assert!(write(&path.join("selection"), id).is_err());
        fs::remove_file(&path).unwrap();
        fs::remove_dir(&directory).unwrap();
    }
}
