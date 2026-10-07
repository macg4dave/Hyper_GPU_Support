//! Per-operation NVIDIA preparation manifests. Paths and hashes are discovered data.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

/// Associated installed-driver facts returned by Windows.
pub struct DriverDiscovery {
    /// Selected partitionable interface.
    pub gpu_interface: String,
    /// Physical PnP identity.
    pub device_id: String,
    /// Current display name.
    pub name: String,
    /// Current driver version.
    pub version: String,
    /// Installed INF identity.
    pub inf_name: String,
    /// Kernel service name.
    pub service: String,
    /// Kernel service binary.
    pub service_binary: PathBuf,
    /// Complete associated host file list.
    pub associated_files: Vec<PathBuf>,
}
/// One validated source and logical Windows-relative guest destination.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyFile {
    /// Current host source, never supplied by an unelevated caller.
    pub source: PathBuf,
    /// Relative destination under the guest's discovered Windows directory.
    pub destination: String,
    /// Expected byte length.
    pub bytes: u64,
    /// SHA-256 for this operation.
    pub sha256: String,
}
/// Complete discovered payload for one installed driver.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Format version.
    pub schema: u32,
    /// Selected physical device identity.
    pub device_id: String,
    /// Discovered driver version.
    pub driver_version: String,
    /// Signed package catalogs requiring native trust verification.
    pub catalogs: Vec<PathBuf>,
    /// Dynamically sized payload.
    pub files: Vec<CopyFile>,
}
impl Manifest {
    /// Deterministic digest of current preparation inputs.
    pub fn digest(&self) -> Result<String, String> {
        Ok(hex(&Sha256::digest(
            serde_json::to_vec(self).map_err(|e| e.to_string())?,
        )))
    }
}
/// Convert the provider's partition interface into its physical PCI PnP key.
pub fn physical_device_id(interface: &str) -> Result<String, String> {
    let body = interface
        .strip_prefix(r"\\?\")
        .ok_or("invalid partition interface")?;
    let (device, suffix) = body.split_once("#{").ok_or("invalid partition interface")?;
    if !device.starts_with("PCI#")
        || !suffix.ends_with(r"}\GPUPARAV")
        || device.contains(['/', ':', '"', '\''])
    {
        return Err("invalid partition interface".into());
    }
    Ok(device.replace('#', r"\"))
}
/// Validate a Windows-relative destination, including device names and alternate streams.
pub fn validate_relative(relative: &str) -> Result<(), String> {
    if relative.is_empty()
        || relative.len() > 240
        || relative.split('\\').any(|s| {
            let stem = s.split('.').next().unwrap_or_default().to_ascii_uppercase();
            s.is_empty()
                || s == "."
                || s == ".."
                || s.ends_with(['.', ' '])
                || s.chars().any(|c| c.is_control() || ":/\"<>|?*".contains(c))
                || matches!(
                    stem.as_str(),
                    "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
                )
                || (stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        })
    {
        return Err("unsafe Windows-relative path".into());
    }
    Ok(())
}
fn package_root(relative: &str) -> Result<Option<String>, String> {
    validate_relative(relative)?;
    let parts: Vec<_> = relative.split('\\').collect();
    if parts.len() >= 3
        && parts[0].eq_ignore_ascii_case("System32")
        && parts[1].eq_ignore_ascii_case("DriverStore")
    {
        if parts.len() < 5 || !parts[2].eq_ignore_ascii_case("FileRepository") {
            return Err("invalid DriverStore path".into());
        }
        return Ok(Some(parts[..4].join("\\")));
    }
    Ok(None)
}
/// Map discovered driver-package paths into GPU-PV's HostDriverStore layout.
pub fn destination(relative: &str) -> Result<String, String> {
    validate_relative(relative)?;
    if package_root(relative)?.is_some() {
        let mut parts: Vec<_> = relative.split('\\').collect();
        parts[1] = "HostDriverStore";
        Ok(parts.join("\\"))
    } else {
        Ok(relative.into())
    }
}
/// Hash a file without loading the complete payload into memory.
pub fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("read preparation file: {e}"))?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        digest.update(&buffer[..n]);
    }
    Ok(hex(&digest.finalize()))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
/// Reject reparse points in every existing ancestor of a protected file path.
pub fn no_reparse(path: &Path) -> Result<(), String> {
    for part in path.ancestors() {
        if part.as_os_str().is_empty() {
            continue;
        }
        let metadata =
            fs::symlink_metadata(part).map_err(|e| format!("inspect file ancestor: {e}"))?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x400 != 0 {
                return Err("reparse point refused".into());
            }
        }
        if metadata.file_type().is_symlink() {
            return Err("symbolic link refused".into());
        }
    }
    Ok(())
}
/// Expand the complete associated payload without historical package pins or file counts.
pub fn discover(root: &Path, discovery: DriverDiscovery) -> Result<Manifest, String> {
    let prefix = format!("{}\\", root.to_string_lossy().trim_end_matches('\\'));
    let relative = |path: &Path| -> Result<String, String> {
        let text = path.to_string_lossy();
        if !text
            .get(..prefix.len())
            .is_some_and(|p| p.eq_ignore_ascii_case(&prefix))
        {
            return Err("associated source is outside Windows".into());
        }
        let value = text[prefix.len()..].to_owned();
        validate_relative(&value)?;
        Ok(value)
    };
    if discovery.associated_files.is_empty() || discovery.service.is_empty() {
        return Err("driver association inventory is incomplete".into());
    }
    let mut packages = std::collections::BTreeSet::new();
    packages.insert(
        package_root(&relative(&discovery.service_binary)?)?
            .ok_or("service is outside DriverStore")?,
    );
    let mut sources = discovery.associated_files;
    for source in &sources {
        if let Some(package) = package_root(&relative(source)?)? {
            packages.insert(package);
        }
    }
    fn walk(path: &Path, sources: &mut Vec<PathBuf>) -> Result<(), String> {
        no_reparse(path)?;
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            no_reparse(&path)?;
            if path.is_dir() {
                walk(&path, sources)?;
            } else if path.is_file() {
                sources.push(path);
            } else {
                return Err("non-file driver input".into());
            }
            if sources.len() > 65536 {
                return Err("driver inventory exceeds safety limit".into());
            }
        }
        Ok(())
    }
    for package in packages {
        walk(&root.join(package), &mut sources)?;
    }
    // CIM associations and directory enumeration use different path casing for
    // the same installed Windows file. Fold identities before hashing/mapping.
    sources.sort_by_key(|p| p.to_string_lossy().to_ascii_lowercase());
    sources.dedup_by(|a, b| {
        a.to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy())
    });
    let mut files = BTreeMap::new();
    let mut catalogs = Vec::new();
    for source in sources {
        no_reparse(&source)?;
        let rel = relative(&source)?;
        let mapped = destination(&rel)?;
        let entry = CopyFile {
            bytes: fs::metadata(&source).map_err(|e| e.to_string())?.len(),
            sha256: hash_file(&source)?,
            source: source.clone(),
            destination: mapped.clone(),
        };
        if let Some(previous) = files.insert(mapped.to_lowercase(), entry)
            && !previous
                .source
                .to_string_lossy()
                .eq_ignore_ascii_case(&source.to_string_lossy())
        {
            return Err("conflicting guest destinations".into());
        }
        if source
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("cat"))
        {
            catalogs.push(source);
        }
    }
    catalogs.sort();
    catalogs.dedup();
    if catalogs.is_empty() {
        return Err("installed driver has no discovered signed catalog".into());
    }
    Ok(Manifest {
        schema: 1,
        device_id: discovery.device_id,
        driver_version: discovery.version,
        catalogs,
        files: files.into_values().collect(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maps_packages_and_preserves_associated_runtime_paths() {
        assert_eq!(
            destination(r"System32\DriverStore\FileRepository\new.inf_amd64\driver.dll").unwrap(),
            r"System32\HostDriverStore\FileRepository\new.inf_amd64\driver.dll"
        );
        assert_eq!(
            destination(r"SysWOW64\runtime.dll").unwrap(),
            r"SysWOW64\runtime.dll"
        );
        for p in [
            r"..\x",
            r"System32\x:stream",
            r"System32\CON.txt",
            r"System32\x.",
        ] {
            assert!(destination(p).is_err());
        }
    }
    #[test]
    fn hashes_describe_the_current_manifest() {
        let mut m = Manifest {
            schema: 1,
            device_id: "gpu".into(),
            driver_version: "new".into(),
            catalogs: vec![],
            files: vec![],
        };
        let old = m.digest().unwrap();
        m.driver_version = "newer".into();
        assert_ne!(old, m.digest().unwrap());
    }
    #[test]
    fn association_case_aliases_do_not_conflict_with_expanded_variable_packages() {
        let root = std::env::temp_dir().join(format!("hyper-gpu-payload-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let package = root.join(r"System32\DriverStore\FileRepository\fresh.inf_amd64_dynamic");
        fs::create_dir_all(&package).unwrap();
        fs::write(package.join("driver.sys"), b"driver").unwrap();
        fs::write(package.join("signed.cat"), b"catalog fixture").unwrap();
        let discovery = || DriverDiscovery {
            gpu_interface: "selected".into(),
            device_id: "physical".into(),
            name: "runtime GPU".into(),
            version: "current".into(),
            inf_name: "fresh.inf".into(),
            service: "driver".into(),
            service_binary: package.join("driver.sys"),
            associated_files: vec![PathBuf::from(
                package
                    .join("driver.sys")
                    .to_string_lossy()
                    .to_ascii_lowercase(),
            )],
        };
        let first = discover(&root, discovery()).unwrap();
        assert_eq!(first.files.len(), 2);
        fs::write(package.join("new-runtime.dll"), b"new associated payload").unwrap();
        let second = discover(&root, discovery()).unwrap();
        assert_eq!(second.files.len(), 3);
        assert_ne!(first.digest().unwrap(), second.digest().unwrap());
        fs::remove_dir_all(root).unwrap();
    }
}
