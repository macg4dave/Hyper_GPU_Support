//! Complete validated driver/runtime environment as a deterministic Rust manifest.
//!
//! Native discovery supplies the selected service and signed-driver associations;
//! Rust owns package expansion, destination mapping, source validation and hashes.
//! Inspection does not mutate the working guest. Historical recipe research: James
//! Stringer's Easy-GPU-PV, commit 2353d36325e18c759ca3888e6591e18e5f371011,
//! Add-VMGpuPartitionAdapterFiles.psm1; no upstream implementation text is reused.

use crate::{
    config::ProjectConfiguration,
    staging::{StagingError, collect_files, hash_file, inspect_driver_package},
};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

/// Native facts for the one explicitly selected physical GPU.
#[derive(Debug)]
pub struct DriverDiscovery {
    /// Verified partitionable interface path.
    pub gpu_interface: String,
    /// Physical PnP DeviceID correlated with that interface.
    pub device_id: String,
    /// Physical GPU name reported by its signed driver.
    pub name: String,
    /// Installed signed driver version.
    pub version: String,
    /// Published signed-driver INF name.
    pub inf_name: String,
    /// Physical GPU's kernel service name.
    pub service: String,
    /// Kernel service binary path, not a guest service installation instruction.
    pub service_binary: PathBuf,
    /// Actual CIMDataFile associations for that exact signed-driver DeviceID.
    pub associated_files: Vec<PathBuf>,
}

/// Full package trees plus every associated external destination.
#[derive(Debug, Clone, Serialize)]
pub struct DriverEnvironmentManifest {
    /// Version of this independent full-environment manifest contract.
    pub schema: u32,
    /// Selected physical PnP identity.
    pub device_id: String,
    /// Installed NVIDIA driver version.
    pub driver_version: String,
    /// Published host INF name.
    pub inf_name: String,
    /// Host kernel-driver service.
    pub service: String,
    /// Number of unique actual driver associations before package expansion.
    pub associated_file_count: usize,
    /// Whole package directories copied, including the service package.
    pub package_directories: Vec<PathBuf>,
    /// Exact normal-copy destinations in stable case-insensitive order.
    pub files: Vec<EnvironmentFile>,
}

/// One ordinary byte-preserving copy; there is no hard-link/rename instruction.
#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentFile {
    /// Native discovered host source path.
    pub source: PathBuf,
    /// Destination relative to the discovered guest Windows directory.
    pub windows_relative_destination: String,
    /// Whether package-tree expansion supplies this file.
    pub driver_store: bool,
    /// Whether the signed driver directly associates this source file.
    pub associated: bool,
    /// Exact source length.
    pub bytes: u64,
    /// Lowercase SHA-256 for copy and subsequent verification.
    pub sha256: String,
}

impl DriverEnvironmentManifest {
    /// Canonical UTF-8 JSON encoding with a terminal newline.
    ///
    /// # Errors
    /// Returns serialization failure without emitting a partial manifest.
    pub fn encode(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut encoded = serde_json::to_vec(self)?;
        encoded.push(b'\n');
        Ok(encoded)
    }

    /// SHA-256 of the complete encoded copy contract, including every destination.
    ///
    /// # Errors
    /// Returns the same serialization failure as [`Self::encode`].
    pub fn sha256(&self) -> Result<String, serde_json::Error> {
        Ok(crate::probe::sha256_hex(&self.encode()?))
    }
}

/// Derive the physical PnP ID from a validated GPU-PV interface, without selecting
/// another GPU or falling back to the first provider result.
///
/// # Errors
/// Rejects malformed interfaces outside the native PCI GPU-PV namespace.
pub fn physical_device_id(interface: &str) -> Result<String, StagingError> {
    let body = interface
        .strip_prefix(r"\\?\")
        .ok_or(StagingError::UnsafeSource)?;
    let (device, suffix) = body.split_once("#{").ok_or(StagingError::UnsafeSource)?;
    if !device.starts_with("PCI#")
        || !suffix.ends_with(r"}\GPUPARAV")
        || device.contains(['/', ':', '"', '\''])
    {
        return Err(StagingError::UnsafeSource);
    }
    Ok(device.replace('#', r"\"))
}

/// Expand the discovered driver environment without minimizing its payload.
///
/// # Errors
/// Rejects mismatched physical identity/version/service package, malformed or
/// reparse sources, conflicting destinations, and changed pinned package inputs.
/// Associations outside Windows fail explicitly rather than being silently omitted;
/// the measured baseline's entire association closure is within Windows.
pub fn inspect_driver_environment(
    project: &ProjectConfiguration,
    windows_directory: &Path,
    discovery: DriverDiscovery,
) -> Result<DriverEnvironmentManifest, StagingError> {
    if discovery.gpu_interface != project.slot.gpu_interface
        || !discovery
            .device_id
            .eq_ignore_ascii_case(&physical_device_id(&project.slot.gpu_interface)?)
        || discovery.name != project.slot.gpu_name
        || discovery.version != project.driver_manifest.driver_version
        || discovery.service.is_empty()
        || discovery.associated_files.is_empty()
    {
        return Err(StagingError::ChangedSource);
    }
    let pinned = inspect_driver_package(&project.driver_manifest)?;
    if pinned.sha256()? != project.driver_manifest.sha256 {
        return Err(StagingError::ChangedSource);
    }
    let windows_root = windows_directory
        .to_str()
        .ok_or(StagingError::UnsafeSource)?
        .trim_end_matches('\\');
    let mut associations = BTreeMap::new();
    let mut packages = BTreeMap::new();
    let service = relative_windows_path(windows_root, &discovery.service_binary)?;
    let service_package = package_relative_root(&service)?.ok_or(StagingError::UnsafeSource)?;
    let service_path = windows_directory.join(&service_package);
    if !service_path
        .to_string_lossy()
        .eq_ignore_ascii_case(&project.driver_manifest.source_path.to_string_lossy())
    {
        return Err(StagingError::ChangedSource);
    }
    packages.insert(service_package.to_ascii_lowercase(), service_path);
    for source in discovery.associated_files {
        let relative = relative_windows_path(windows_root, &source)?;
        if let Some(package) = package_relative_root(&relative)? {
            packages.insert(
                package.to_ascii_lowercase(),
                windows_directory.join(package),
            );
        }
        associations.insert(source.to_string_lossy().to_ascii_lowercase(), source);
    }
    let mut copies = BTreeMap::new();
    for package in packages.values() {
        let mut entries = Vec::new();
        validate_source_ancestors(windows_directory, package)?;
        collect_files(package, package, &mut entries)?;
        for entry in entries {
            let source = package.join(&entry.relative_path);
            let relative = relative_windows_path(windows_root, &source)?;
            insert_copy(
                &mut copies,
                EnvironmentFile {
                    associated: associations
                        .contains_key(&source.to_string_lossy().to_ascii_lowercase()),
                    source,
                    windows_relative_destination: map_destination(&relative)?,
                    driver_store: true,
                    bytes: entry.bytes,
                    sha256: entry.sha256,
                },
            )?;
        }
    }
    for source in associations.values() {
        let relative = relative_windows_path(windows_root, source)?;
        if package_relative_root(&relative)?.is_some() {
            continue;
        }
        validate_source_ancestors(windows_directory, source)?;
        let metadata = fs::symlink_metadata(source).map_err(|_| StagingError::UnsafeSource)?;
        if !metadata.is_file() {
            return Err(StagingError::UnsafeSource);
        }
        insert_copy(
            &mut copies,
            EnvironmentFile {
                source: source.clone(),
                windows_relative_destination: map_destination(&relative)?,
                driver_store: false,
                associated: true,
                bytes: metadata.len(),
                sha256: hash_file(source)?,
            },
        )?;
    }
    Ok(DriverEnvironmentManifest {
        schema: 1,
        device_id: discovery.device_id,
        driver_version: discovery.version,
        inf_name: discovery.inf_name,
        service: discovery.service,
        associated_file_count: associations.len(),
        package_directories: packages.into_values().collect(),
        files: copies.into_values().collect(),
    })
}

fn insert_copy(
    copies: &mut BTreeMap<String, EnvironmentFile>,
    copy: EnvironmentFile,
) -> Result<(), StagingError> {
    if copies
        .insert(copy.windows_relative_destination.to_ascii_lowercase(), copy)
        .is_some()
    {
        return Err(StagingError::UnsafeSource);
    }
    Ok(())
}

pub(crate) fn relative_windows_path(
    windows_root: &str,
    source: &Path,
) -> Result<String, StagingError> {
    let source = source.to_str().ok_or(StagingError::UnsafeSource)?;
    let prefix = format!("{windows_root}\\");
    if !source
        .get(..prefix.len())
        .is_some_and(|value| value.eq_ignore_ascii_case(&prefix))
    {
        return Err(StagingError::UnsafeSource);
    }
    let relative = &source[prefix.len()..];
    validate_relative(relative)?;
    Ok(relative.to_owned())
}

pub(crate) fn validate_relative(relative: &str) -> Result<(), StagingError> {
    if relative.is_empty()
        || relative.split('\\').any(|segment| {
            segment.is_empty()
                || segment == "."
                || segment == ".."
                || segment.ends_with(['.', ' '])
                || reserved_component(segment)
                || segment
                    .chars()
                    .any(|character| character.is_control() || ":/\"<>|?*".contains(character))
        })
    {
        return Err(StagingError::UnsafeSource);
    }
    Ok(())
}

fn reserved_component(segment: &str) -> bool {
    let stem = segment
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) || (stem.len() == 4
        && (stem.starts_with("COM") || stem.starts_with("LPT"))
        && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

fn package_relative_root(relative: &str) -> Result<Option<String>, StagingError> {
    validate_relative(relative)?;
    let parts = relative.split('\\').collect::<Vec<_>>();
    if parts.len() >= 3
        && parts[0].eq_ignore_ascii_case("System32")
        && parts[1].eq_ignore_ascii_case("DriverStore")
    {
        if parts.len() < 5 || !parts[2].eq_ignore_ascii_case("FileRepository") {
            return Err(StagingError::UnsafeSource);
        }
        return Ok(Some(parts[..4].join("\\")));
    }
    Ok(None)
}

pub(crate) fn map_destination(relative: &str) -> Result<String, StagingError> {
    if package_relative_root(relative)?.is_some() {
        let mut parts = relative.split('\\').collect::<Vec<_>>();
        parts[1] = "HostDriverStore";
        Ok(parts.join("\\"))
    } else {
        Ok(relative.to_owned())
    }
}

pub(crate) fn validate_source_ancestors(root: &Path, source: &Path) -> Result<(), StagingError> {
    use std::os::windows::fs::MetadataExt;
    let mut current = source;
    loop {
        let metadata = fs::symlink_metadata(current).map_err(|_| StagingError::UnsafeSource)?;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(StagingError::UnsafeSource);
        }
        if current
            .to_string_lossy()
            .eq_ignore_ascii_case(&root.to_string_lossy())
        {
            return Ok(());
        }
        current = current.parent().ok_or(StagingError::UnsafeSource)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encoded_copy_contract_hash_changes_when_a_destination_changes() {
        let mut manifest = DriverEnvironmentManifest {
            schema: 1,
            device_id: "PCI\\selected".into(),
            driver_version: "1.0".into(),
            inf_name: "oem.inf".into(),
            service: "selected".into(),
            associated_file_count: 1,
            package_directories: vec![],
            files: vec![EnvironmentFile {
                source: PathBuf::from(r"C:\Windows\nv.dll"),
                windows_relative_destination: r"System32\nv.dll".into(),
                driver_store: false,
                associated: true,
                bytes: 1,
                sha256: "0".repeat(64),
            }],
        };
        let encoded = manifest.encode().unwrap();
        assert_eq!(encoded.last(), Some(&b'\n'));
        let parsed: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(
            parsed["files"][0]["windows_relative_destination"],
            r"System32\nv.dll"
        );
        let original_hash = manifest.sha256().unwrap();
        assert_eq!(original_hash, crate::probe::sha256_hex(&encoded));
        manifest.files[0].windows_relative_destination = r"SysWOW64\nv.dll".into();
        assert_ne!(manifest.sha256().unwrap(), original_hash);
    }
    #[test]
    fn maps_package_and_every_external_path_without_alias_guessing() {
        assert_eq!(
            map_destination(r"system32\driverstore\filerepository\nv.inf_amd64_hash\x\nv.dll")
                .unwrap(),
            r"system32\HostDriverStore\filerepository\nv.inf_amd64_hash\x\nv.dll"
        );
        for path in [
            r"System32\nvml.dll",
            r"SysWOW64\nvcuda.dll",
            r"INF\oem59.inf",
            r"System32\lxss\lib\libcuda.so.1",
            r"System32\drivers\Nvidia Corporation\DRS\nvdrsdb.bin",
        ] {
            assert_eq!(map_destination(path).unwrap(), path);
        }
    }
    #[test]
    fn rejects_escape_and_malformed_package_paths() {
        for path in [
            r"..\nv.dll",
            r"System32\..\nv.dll",
            r"System32\nv.dll:stream",
            r"System32\\nv.dll",
            r"System32\nv.dll.",
            r"System32\DriverStore\nv.dll",
            r"System32\DriverStore\FileRepository\nv.inf",
            r"System32\CON.dll",
            r"System32\COM1",
            r"System32\LPT9.bin",
        ] {
            assert!(map_destination(path).is_err(), "{path}");
        }
        assert!(
            relative_windows_path(r"C:\Windows", Path::new(r"C:\WindowsOther\nv.dll")).is_err()
        );
    }
    #[test]
    fn maps_historical_nvidia_616_92_baseline_destinations() {
        let inventory = include_str!("../../../docs/evidence/GPU-PV-BASELINE-INVENTORY.tsv");
        let header = inventory
            .trim_start_matches('\u{feff}')
            .lines()
            .next()
            .unwrap()
            .split('\t')
            .collect::<Vec<_>>();
        let source_column = header.iter().position(|v| *v == "\"source\"").unwrap();
        let destination_column = header
            .iter()
            .position(|v| *v == "\"easy_destination\"")
            .unwrap();
        let mut destinations = std::collections::BTreeSet::new();
        for line in inventory.lines().skip(1) {
            let columns = line
                .split('\t')
                .map(|v| v.trim_matches('"'))
                .collect::<Vec<_>>();
            let relative =
                relative_windows_path(r"C:\Windows", Path::new(columns[source_column])).unwrap();
            let expected =
                relative_windows_path(r"C:\Windows", Path::new(columns[destination_column]))
                    .unwrap();
            let mapped = map_destination(&relative).unwrap();
            assert!(mapped.eq_ignore_ascii_case(&expected), "{relative}");
            assert!(destinations.insert(mapped.to_ascii_lowercase()));
        }
        // Historical RTX 5060 / NVIDIA 616.92 fixture extent, not a product limit.
        assert_eq!(destinations.len(), 271);
    }
    #[test]
    fn physical_interface_mapping_is_explicit_and_destination_collision_fails() {
        let project = ProjectConfiguration::embedded().unwrap();
        assert!(
            physical_device_id(&project.slot.gpu_interface)
                .unwrap()
                .starts_with(r"PCI\VEN_")
        );
        assert!(physical_device_id(r"\\?\ROOT#other#{guid}\GPUPARAV").is_err());
        let copy = || EnvironmentFile {
            source: PathBuf::from(r"C:\Windows\a.dll"),
            windows_relative_destination: "System32\\a.dll".into(),
            driver_store: false,
            associated: true,
            bytes: 0,
            sha256: "0".repeat(64),
        };
        let mut copies = BTreeMap::new();
        insert_copy(&mut copies, copy()).unwrap();
        assert!(insert_copy(&mut copies, copy()).is_err());
    }
}
