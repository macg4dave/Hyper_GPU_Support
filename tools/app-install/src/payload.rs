//! The exact executable manifest is checked before resetting or accepting a package.

use crate::{
    PAYLOAD,
    storage::{hash_file, require_plain_path},
};
use std::{collections::BTreeMap, fs, path::Path};

pub(super) fn verify(directory: &Path) -> Result<(), String> {
    require_plain_path(directory)?;
    let manifest = directory.join("SHA256SUMS.txt");
    require_plain_path(&manifest)?;
    let text = fs::read_to_string(&manifest)
        .map_err(|error| format!("manifest {}: {error}", manifest.display()))?;
    let hashes = parse_manifest(&text)?;
    for name in PAYLOAD {
        let file = directory.join(name);
        require_plain_path(&file)?;
        if !file.is_file() {
            return Err(format!(
                "{} is missing; build the complete installer payload first",
                file.display()
            ));
        }
        if hash_file(&file)? != hashes[name] {
            return Err(format!("{name} does not match the package SHA256SUMS.txt"));
        }
    }
    Ok(())
}

fn parse_manifest(text: &str) -> Result<BTreeMap<&str, String>, String> {
    let mut hashes = BTreeMap::new();
    for line in text.lines() {
        let (hash, name) = line.split_once("  ").ok_or(
            "invalid SHA256SUMS.txt line: expected SHA256, two spaces and executable name",
        )?;
        if hash.len() != 64
            || !hash.bytes().all(|value| value.is_ascii_hexdigit())
            || !PAYLOAD.contains(&name)
        {
            return Err(format!("invalid package manifest entry {name}"));
        }
        if hashes.insert(name, hash.to_ascii_lowercase()).is_some() {
            return Err(format!("duplicate package manifest entry {name}"));
        }
    }
    if hashes.len() != PAYLOAD.len() {
        return Err("package manifest must contain exactly the five product executables".into());
    }
    Ok(hashes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::temp;

    fn write_manifest(dir: &Path) {
        let text: String = PAYLOAD
            .iter()
            .map(|name| format!("{}  {name}\n", hash_file(&dir.join(name)).unwrap()))
            .collect();
        fs::write(dir.join("SHA256SUMS.txt"), text).unwrap();
    }

    #[test]
    fn missing_artifact_fails_before_any_change() {
        let dir = temp("missing");
        let manifest: String = PAYLOAD
            .iter()
            .map(|name| format!("{}  {name}\n", "0".repeat(64)))
            .collect();
        fs::write(dir.join("SHA256SUMS.txt"), manifest).unwrap();
        assert!(verify(&dir).unwrap_err().contains(PAYLOAD[0]));
        for name in PAYLOAD {
            fs::write(dir.join(name), name).unwrap();
        }
        write_manifest(&dir);
        verify(&dir).unwrap();
        fs::write(dir.join("app-install.exe"), "changed helper").unwrap();
        assert!(verify(&dir).unwrap_err().contains("app-install.exe"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn manifest_rejects_duplicates_missing_entries_and_path_traversal() {
        let good: String = PAYLOAD
            .iter()
            .map(|name| format!("{}  {name}\r\n", "A".repeat(64)))
            .collect();
        assert!(parse_manifest(&good).is_ok());
        assert!(
            parse_manifest(&(good.clone() + &format!("{}  app-install.exe\n", "a".repeat(64))))
                .is_err()
        );
        assert!(parse_manifest(&good.replace("app-install.exe", "../app-install.exe")).is_err());
        assert!(
            parse_manifest(&good.replace(&format!("{}  app-install.exe\r\n", "A".repeat(64)), ""))
                .is_err()
        );
    }
}
