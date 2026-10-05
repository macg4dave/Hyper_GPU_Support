//! Complete driver environment staging contract, independent of guest transport.

use crate::{
    config::ProjectConfiguration,
    driver_environment::{
        DriverEnvironmentManifest, map_destination, physical_device_id, relative_windows_path,
        validate_source_ancestors,
    },
    guest::{GuestCredential, MAX_TRANSFER_BYTES},
    staging::{StagingError, hash_file},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, path::Path};

/// Guest-observed evidence for every managed destination, including verified reapply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentStageReceipt {
    /// Independent full-environment receipt schema.
    pub schema: u32,
    /// Enrolled VM identity.
    pub vm_id: String,
    /// Authenticated guest name.
    pub computer_name: String,
    /// Authenticated guest MachineGuid.
    pub machine_guid: String,
    /// SHA-256 of the full encoded environment, not the package subset.
    pub manifest_sha256: String,
    /// Guest-discovered Windows directory.
    pub windows_root: String,
    /// Number of independently rehashed destinations.
    pub files: usize,
    /// Total independently verified byte length.
    pub bytes: u64,
    /// Host qualification build.
    pub qualified_host_build: String,
    /// Measured host build.
    pub measured_host_build: String,
    /// Measured guest build.
    pub measured_guest_build: String,
    /// Exact reviewed residual delete-only records.
    pub pending_delete_sources: Vec<String>,
}

/// Complete success report; failure after possible writes requires child recreation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentStageResult {
    /// True only when an existing receipt and all destinations were reverified.
    pub already_applied: bool,
    /// Verified guest observations.
    pub receipt: EnvironmentStageReceipt,
}

/// Fixed selected-driver transport; implementations must independently authorize
/// native discovery, never accept arbitrary caller-selected files or commands.
pub trait GuestEnvironmentStager {
    /// Apply or reverify the complete manifest after validating native target state.
    fn stage_environment(
        &self,
        credential: &GuestCredential,
        manifest: &DriverEnvironmentManifest,
    ) -> Result<EnvironmentStageResult, StagingError>;
}

/// Validate all source mappings and bytes before opening a mutating transport.
///
/// # Errors
/// Rejects changed identities, unsafe sources/destinations, collisions and hashes.
pub fn validate_environment_sources(
    project: &ProjectConfiguration,
    windows_root: &Path,
    manifest: &DriverEnvironmentManifest,
) -> Result<u64, StagingError> {
    if !windows_root.is_absolute()
        || manifest.schema != 1
        || manifest.files.is_empty()
        || manifest.device_id != physical_device_id(&project.slot.gpu_interface)?
        || manifest.driver_version != project.driver_manifest.driver_version
        || manifest.inf_name.is_empty()
        || manifest.service.is_empty()
        || manifest.associated_file_count == 0
    {
        return Err(StagingError::ChangedSource);
    }
    // Validate the Windows root and its ancestors too, not only its descendants.
    let volume = windows_root
        .ancestors()
        .last()
        .ok_or(StagingError::UnsafeSource)?;
    validate_source_ancestors(volume, windows_root)?;
    let root = windows_root
        .to_str()
        .ok_or(StagingError::UnsafeSource)?
        .trim_end_matches('\\');
    let mut destinations = BTreeSet::new();
    let mut sources = BTreeSet::new();
    let mut bytes = 0_u64;
    let mut previous = None;
    for file in &manifest.files {
        let relative = relative_windows_path(root, &file.source)?;
        let mapped = map_destination(&relative)?;
        let key = file.windows_relative_destination.to_ascii_lowercase();
        if mapped != file.windows_relative_destination
            || file.driver_store == mapped.eq_ignore_ascii_case(&relative)
            || (!file.driver_store && !file.associated)
            || !destinations.insert(key.clone())
            || !sources.insert(file.source.to_string_lossy().to_ascii_lowercase())
            || previous.as_ref().is_some_and(|last| last >= &key)
            || file.sha256.len() != 64
            || !file
                .sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(StagingError::UnsafeSource);
        }
        // A file cannot also be another managed file's directory.
        for (index, _) in key.match_indices('\\') {
            if destinations.contains(&key[..index]) {
                return Err(StagingError::UnsafeSource);
            }
        }
        previous = Some(key);
        validate_source_ancestors(windows_root, &file.source)?;
        let metadata =
            fs::symlink_metadata(&file.source).map_err(|_| StagingError::UnsafeSource)?;
        if !metadata.is_file()
            || metadata.len() != file.bytes
            || file.bytes > MAX_TRANSFER_BYTES
            || hash_file(&file.source)? != file.sha256
        {
            return Err(StagingError::ChangedSource);
        }
        bytes = bytes
            .checked_add(file.bytes)
            .ok_or(StagingError::ChangedSource)?;
    }
    Ok(bytes)
}

/// Stage all discovered files and bind success to the exact full manifest/guest.
///
/// # Errors
/// Preflight failures retain their category. Invalid success evidence and uncertain
/// transport failures require disposable-child recreation.
pub fn stage_driver_environment(
    project: &ProjectConfiguration,
    windows_root: &Path,
    manifest: &DriverEnvironmentManifest,
    credential: &GuestCredential,
    adapter: &impl GuestEnvironmentStager,
) -> Result<EnvironmentStageResult, StagingError> {
    let bytes = validate_environment_sources(project, windows_root, manifest)?;
    let digest = manifest.sha256().map_err(|_| StagingError::Encoding)?;
    let result = adapter.stage_environment(credential, manifest)?;
    let receipt = &result.receipt;
    if receipt.schema != 1
        || receipt.vm_id != project.slot.vm_id
        || !receipt
            .computer_name
            .eq_ignore_ascii_case(&project.guest.computer_name)
        || receipt.machine_guid != project.guest.machine_guid
        || receipt.manifest_sha256 != digest
        || receipt.files != manifest.files.len()
        || receipt.bytes != bytes
        || !Path::new(&receipt.windows_root).is_absolute()
        || receipt.windows_root.contains(['/', '"', '\''])
        || receipt.qualified_host_build != project.driver_manifest.host_build
        || !valid_build(&receipt.measured_host_build)
        || !valid_build(&receipt.measured_guest_build)
        || receipt.pending_delete_sources.iter().any(|value| {
            !project
                .driver_manifest
                .allowed_pending_delete_sources
                .contains(value)
        })
        || receipt
            .pending_delete_sources
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != receipt.pending_delete_sources.len()
    {
        return Err(StagingError::GuestStateUncertain);
    }
    Ok(result)
}

fn valid_build(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 2
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver_environment::EnvironmentFile;
    use std::{
        cell::Cell,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture {
        root: PathBuf,
        project: ProjectConfiguration,
        manifest: DriverEnvironmentManifest,
    }
    impl Fixture {
        fn new() -> Self {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("local/tests")
                .join(format!(
                    "core022-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
            fs::create_dir_all(&root).unwrap();
            // The checkout itself is reached through a junction on this host.
            // Use its resolved test directory rather than weakening source guards.
            let root = resolved_test_directory(&root);
            let project = ProjectConfiguration::embedded().unwrap();
            let mut files = Vec::new();
            for relative in [
                r"System32\DriverStore\FileRepository\nv.inf_amd64_hash\driver.sys",
                r"System32\nv.dll",
                r"SysWOW64\nv.dll",
            ] {
                let source = root.join(relative);
                fs::create_dir_all(source.parent().unwrap()).unwrap();
                fs::write(&source, relative.as_bytes()).unwrap();
                let destination = map_destination(relative).unwrap();
                files.push(EnvironmentFile {
                    source,
                    driver_store: destination != relative,
                    associated: true,
                    windows_relative_destination: destination,
                    bytes: relative.len() as u64,
                    sha256: crate::probe::sha256_hex(relative.as_bytes()),
                });
            }
            files.sort_by_key(|f| f.windows_relative_destination.to_ascii_lowercase());
            let manifest = DriverEnvironmentManifest {
                schema: 1,
                device_id: physical_device_id(&project.slot.gpu_interface).unwrap(),
                driver_version: project.driver_manifest.driver_version.clone(),
                inf_name: "oem.inf".into(),
                service: "nv".into(),
                associated_file_count: 3,
                package_directories: vec![],
                files,
            };
            Self {
                root,
                project,
                manifest,
            }
        }
        fn receipt(&self) -> EnvironmentStageReceipt {
            EnvironmentStageReceipt {
                schema: 1,
                vm_id: self.project.slot.vm_id.clone(),
                computer_name: self.project.guest.computer_name.clone(),
                machine_guid: self.project.guest.machine_guid.clone(),
                manifest_sha256: self.manifest.sha256().unwrap(),
                windows_root: r"C:\Windows".into(),
                files: self.manifest.files.len(),
                bytes: self.manifest.files.iter().map(|f| f.bytes).sum(),
                qualified_host_build: self.project.driver_manifest.host_build.clone(),
                measured_host_build: "26300.9457".into(),
                measured_guest_build: "26200.9457".into(),
                pending_delete_sources: vec![],
            }
        }
        fn stage(&self, adapter: &Fake) -> Result<EnvironmentStageResult, StagingError> {
            stage_driver_environment(
                &self.project,
                &self.root,
                &self.manifest,
                &GuestCredential::new("user".into(), "ephemeral".into()).unwrap(),
                adapter,
            )
        }
    }
    #[allow(unsafe_code)]
    fn resolved_test_directory(path: &Path) -> PathBuf {
        use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
        use windows::Win32::{
            Foundation::HANDLE,
            Storage::FileSystem::{
                FILE_FLAG_BACKUP_SEMANTICS, GetFinalPathNameByHandleW, VOLUME_NAME_GUID,
            },
        };
        let directory = fs::OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS.0)
            .open(path)
            .unwrap();
        let mut buffer = vec![0_u16; 32768];
        // SAFETY: the owned directory handle and output buffer live through the
        // call. Resolve a volume GUID so mount-point checkout ancestors disappear.
        let length = unsafe {
            GetFinalPathNameByHandleW(
                HANDLE(directory.as_raw_handle()),
                &mut buffer,
                VOLUME_NAME_GUID,
            )
        } as usize;
        assert!(length > 0 && length < buffer.len());
        PathBuf::from(String::from_utf16(&buffer[..length]).unwrap())
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
    struct Fake {
        result: Result<EnvironmentStageResult, StagingError>,
        calls: Cell<usize>,
    }
    impl GuestEnvironmentStager for Fake {
        fn stage_environment(
            &self,
            _: &GuestCredential,
            _: &DriverEnvironmentManifest,
        ) -> Result<EnvironmentStageResult, StagingError> {
            self.calls.set(self.calls.get() + 1);
            self.result.clone()
        }
    }
    fn fake(result: Result<EnvironmentStageResult, StagingError>) -> Fake {
        Fake {
            result,
            calls: Cell::new(0),
        }
    }

    #[test]
    fn binds_full_manifest_apply_and_verified_noop_not_a_fixed_file_count() {
        let f = Fixture::new();
        assert_eq!(
            validate_source_ancestors(f.root.ancestors().last().unwrap(), &f.root),
            Ok(()),
            "root {:?}",
            f.root
        );
        for file in &f.manifest.files {
            assert_eq!(
                validate_source_ancestors(&f.root, &file.source),
                Ok(()),
                "source {:?}",
                file.source
            );
            assert_eq!(
                map_destination(
                    &relative_windows_path(f.root.to_str().unwrap(), &file.source).unwrap()
                )
                .unwrap(),
                file.windows_relative_destination
            );
        }
        for already_applied in [false, true] {
            let result = EnvironmentStageResult {
                already_applied,
                receipt: f.receipt(),
            };
            let adapter = fake(Ok(result.clone()));
            assert_eq!(f.stage(&adapter), Ok(result));
            assert_eq!(adapter.calls.get(), 1);
        }
    }
    #[test]
    fn rejects_changed_external_bytes_before_transport() {
        let f = Fixture::new();
        fs::write(&f.manifest.files[1].source, b"changed").unwrap();
        let adapter = fake(Ok(EnvironmentStageResult {
            already_applied: false,
            receipt: f.receipt(),
        }));
        assert_eq!(f.stage(&adapter), Err(StagingError::ChangedSource));
        assert_eq!(adapter.calls.get(), 0);
    }
    #[test]
    fn rejects_mapping_collisions_unsorted_paths_and_source_escape_before_effects() {
        for case in 0..5 {
            let mut f = Fixture::new();
            match case {
                0 => {
                    f.manifest.files[0].windows_relative_destination = r"System32\..\escape".into()
                }
                1 => f.manifest.files.push(f.manifest.files[0].clone()),
                2 => f.manifest.files.reverse(),
                3 => f.manifest.files[0].source = f.root.parent().unwrap().join("escape"),
                _ => f.manifest.files[1].associated = false,
            }
            let adapter = fake(Ok(EnvironmentStageResult {
                already_applied: false,
                receipt: f.receipt(),
            }));
            assert!(f.stage(&adapter).is_err(), "case {case}");
            assert_eq!(adapter.calls.get(), 0);
        }
    }
    #[test]
    fn mismatched_success_evidence_requires_recreation() {
        let f = Fixture::new();
        for case in 0..9 {
            let mut receipt = f.receipt();
            match case {
                0 => receipt.vm_id.clear(),
                1 => receipt.machine_guid.clear(),
                2 => receipt.manifest_sha256 = "0".repeat(64),
                3 => receipt.files -= 1,
                4 => receipt.bytes -= 1,
                5 => receipt.measured_guest_build.clear(),
                6 => receipt.pending_delete_sources.push("unreviewed".into()),
                7 => receipt.windows_root = "relative".into(),
                _ => receipt.schema = 2,
            }
            assert_eq!(
                f.stage(&fake(Ok(EnvironmentStageResult {
                    already_applied: true,
                    receipt
                }))),
                Err(StagingError::GuestStateUncertain),
                "case {case}"
            );
        }
    }
    #[test]
    fn uncertain_write_and_preflight_errors_remain_distinct() {
        let f = Fixture::new();
        for error in [
            StagingError::GuestStateUncertain,
            StagingError::GuestUnavailable,
            StagingError::InvalidSignature,
        ] {
            assert_eq!(f.stage(&fake(Err(error.clone()))), Err(error));
        }
    }
}
