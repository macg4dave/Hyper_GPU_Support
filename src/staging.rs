//! Read-only inspection and encoding of the pinned NVIDIA guest-staging manifest.
//!
//! This module does not copy driver files or mutate a guest. It turns the exact
//! GPU-correlated DriverStore tree into a deterministic, hashable manifest after
//! validating the GPU-002 package identity.

use std::collections::BTreeSet;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::config::DriverManifestConfiguration;

/// Version of the deterministic driver-package manifest contract.
pub const DRIVER_MANIFEST_SCHEMA: u32 = 1;

/// Complete immutable NVIDIA package manifest used by later guest staging.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DriverPackageManifest {
    /// Manifest contract version.
    pub schema: u32,
    /// Human-auditable identity from project configuration.
    pub id: String,
    /// DriverStore package directory name, without a host-specific prefix.
    pub package_directory: String,
    /// Exact display INF version observed in the package.
    pub inf_version: String,
    /// GPU-002 digest of the entire package tree.
    pub package_tree_sha256: String,
    /// Aggregate byte count of all regular package files.
    pub byte_count: u64,
    /// Every package file in deterministic case-insensitive path order.
    pub files: Vec<DriverPackageFile>,
}

/// One byte-preserving copy operation within the NVIDIA package tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DriverPackageFile {
    /// Backslash-separated path relative to the package root.
    pub relative_path: String,
    /// Exact file length.
    pub bytes: u64,
    /// Lowercase SHA-256 of the file contents.
    pub sha256: String,
}

impl DriverPackageManifest {
    /// Encode the manifest as deterministic compact UTF-8 JSON.
    ///
    /// # Errors
    /// Returns [`StagingError::Encoding`] if serialization unexpectedly fails.
    pub fn canonical_json(&self) -> Result<Vec<u8>, StagingError> {
        serde_json::to_vec(self).map_err(|_| StagingError::Encoding)
    }

    /// SHA-256 of [`Self::canonical_json`].
    ///
    /// # Errors
    /// Returns the same error as [`Self::canonical_json`].
    pub fn sha256(&self) -> Result<String, StagingError> {
        Ok(hex(&Sha256::digest(self.canonical_json()?)))
    }
}

/// Inspect the configured package and produce the exact staging manifest.
///
/// File hashes are collected before the configured aggregate identity, INF version
/// and catalog hash are checked. The later apply path must repeat validation
/// immediately before copying because this read-only inspection holds no package lock.
///
/// # Errors
/// Rejects absent, reparse-backed, malformed, incomplete or changed packages.
pub fn inspect_driver_package(
    configuration: &DriverManifestConfiguration,
) -> Result<DriverPackageManifest, StagingError> {
    let root_metadata = fs::symlink_metadata(&configuration.source_path).map_err(io_error)?;
    if !root_metadata.is_dir() || is_reparse_point(&root_metadata) {
        return Err(StagingError::UnsafeSource);
    }
    let package_directory = configuration
        .source_path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| safe_component(value))
        .ok_or(StagingError::UnsafeSource)?
        .to_owned();

    let mut files = Vec::new();
    collect_files(
        &configuration.source_path,
        &configuration.source_path,
        &mut files,
    )?;
    files.sort_by(|left, right| {
        left.relative_path
            .to_ascii_uppercase()
            .cmp(&right.relative_path.to_ascii_uppercase())
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });

    let mut case_folded = BTreeSet::new();
    if files
        .iter()
        .any(|file| !case_folded.insert(file.relative_path.to_ascii_lowercase()))
    {
        return Err(StagingError::UnsafeSource);
    }
    let byte_count = files.iter().try_fold(0_u64, |total, file| {
        total
            .checked_add(file.bytes)
            .ok_or(StagingError::ChangedSource)
    })?;
    let package_tree_sha256 = tree_digest(&files);

    if files.len() != configuration.file_count as usize
        || byte_count != configuration.byte_count
        || package_tree_sha256 != configuration.package_tree_sha256
    {
        return Err(StagingError::PackageIdentityMismatch {
            files: files.len(),
            bytes: byte_count,
            tree_sha256: package_tree_sha256,
        });
    }
    let inf_version = read_inf_version(&configuration.source_path.join(&configuration.inf_name))?;
    if inf_version != configuration.inf_version {
        return Err(StagingError::ChangedSource);
    }
    let catalog =
        find_file(&files, &configuration.catalog_name).ok_or(StagingError::MissingInput)?;
    if catalog.sha256 != configuration.catalog_sha256 {
        return Err(StagingError::ChangedSource);
    }

    Ok(DriverPackageManifest {
        schema: DRIVER_MANIFEST_SCHEMA,
        id: configuration.id.clone(),
        package_directory,
        inf_version,
        package_tree_sha256,
        byte_count,
        files,
    })
}

fn collect_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<DriverPackageFile>,
) -> Result<(), StagingError> {
    for entry in fs::read_dir(directory).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(io_error)?;
        if is_reparse_point(&metadata) {
            return Err(StagingError::UnsafeSource);
        }
        if metadata.is_dir() {
            collect_files(root, &entry.path(), files)?;
        } else if metadata.is_file() {
            let relative = entry
                .path()
                .strip_prefix(root)
                .map_err(|_| StagingError::UnsafeSource)?
                .to_path_buf();
            let relative_path = manifest_path(&relative)?;
            files.push(DriverPackageFile {
                relative_path,
                bytes: metadata.len(),
                sha256: hash_file(&entry.path())?,
            });
        } else {
            return Err(StagingError::UnsafeSource);
        }
    }
    Ok(())
}

fn manifest_path(path: &Path) -> Result<String, StagingError> {
    let mut values = Vec::new();
    for component in path.components() {
        let Component::Normal(value) = component else {
            return Err(StagingError::UnsafeSource);
        };
        let value = value
            .to_str()
            .filter(|value| safe_component(value))
            .ok_or(StagingError::UnsafeSource)?;
        values.push(value);
    }
    if values.is_empty() {
        return Err(StagingError::UnsafeSource);
    }
    Ok(values.join("\\"))
}

fn safe_component(value: &str) -> bool {
    !value.is_empty()
        && value.is_ascii()
        && !value.ends_with([' ', '.'])
        && !value.contains([':', '\0', '/', '\\'])
        && !value.chars().any(char::is_control)
}

fn tree_digest(files: &[DriverPackageFile]) -> String {
    let mut digest = Sha256::new();
    for file in files {
        digest.update(file.relative_path.as_bytes());
        digest.update([0]);
        digest.update(file.sha256.as_bytes());
        digest.update(b"\n");
    }
    hex(&digest.finalize())
}

fn read_inf_version(path: &Path) -> Result<String, StagingError> {
    let bytes = fs::read(path).map_err(io_error)?;
    let text = String::from_utf8(bytes).map_err(|_| StagingError::InvalidInf)?;
    let mut in_version = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_version = line.eq_ignore_ascii_case("[Version]");
            continue;
        }
        if in_version {
            let Some((name, value)) = line.split_once('=') else {
                continue;
            };
            if name.trim().eq_ignore_ascii_case("DriverVer") {
                let value = value.trim();
                if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
                    return Err(StagingError::InvalidInf);
                }
                return Ok(value.to_owned());
            }
        }
    }
    Err(StagingError::InvalidInf)
}

fn find_file<'a>(files: &'a [DriverPackageFile], name: &str) -> Option<&'a DriverPackageFile> {
    files
        .iter()
        .find(|file| file.relative_path.eq_ignore_ascii_case(name))
}

fn hash_file(path: &Path) -> Result<String, StagingError> {
    let mut file = File::open(path).map_err(io_error)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 128 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(io_error)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(hex(&digest.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(windows)]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn io_error(error: io::Error) -> StagingError {
    if error.kind() == io::ErrorKind::NotFound {
        StagingError::MissingInput
    } else {
        StagingError::SourceIo {
            kind: error.kind(),
            code: error.raw_os_error(),
        }
    }
}

/// Read-only package-inspection failure without proprietary contents in diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StagingError {
    /// A required package path or file is absent.
    MissingInput,
    /// The source tree contains a reparse point, unsafe name or unsupported entry.
    UnsafeSource,
    /// A source read failed.
    SourceIo {
        /// Stable standard-library error category.
        kind: io::ErrorKind,
        /// Native OS error code when available.
        code: Option<i32>,
    },
    /// The display INF is not valid UTF-8 or lacks one usable `DriverVer` value.
    InvalidInf,
    /// File count, length, content, version or pinned digest changed.
    ChangedSource,
    /// Aggregate package identity did not match the configured GPU-002 pin.
    PackageIdentityMismatch {
        /// Observed regular-file count.
        files: usize,
        /// Observed aggregate file length.
        bytes: u64,
        /// Observed canonical package-tree digest.
        tree_sha256: String,
    },
    /// Deterministic manifest encoding failed.
    Encoding,
}

impl fmt::Display for StagingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingInput => "required driver package input is missing",
            Self::UnsafeSource => "driver package source contains an unsafe entry",
            Self::SourceIo { .. } => "cannot read driver package source",
            Self::InvalidInf => "driver package INF identity is invalid",
            Self::ChangedSource => "driver package no longer matches the pinned manifest identity",
            Self::PackageIdentityMismatch {
                files,
                bytes,
                tree_sha256,
            } => {
                return write!(
                    formatter,
                    "driver package identity mismatch (files={files}, bytes={bytes}, tree_sha256={tree_sha256})"
                );
            }
            Self::Encoding => "cannot encode the driver package manifest",
        })
    }
}

impl std::error::Error for StagingError {}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    struct Fixture {
        root: PathBuf,
        configuration: DriverManifestConfiguration,
    }

    impl Fixture {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!("hyper-gpu-manifest-{unique}"));
            fs::create_dir_all(root.join("sub")).unwrap();
            fs::write(
                root.join("driver.inf"),
                b"[Version]\r\nDriverVer = 01/02/2026, 1.2.3.4\r\n",
            )
            .unwrap();
            fs::write(root.join("CAT.CAT"), b"catalog").unwrap();
            fs::write(root.join("sub").join("runtime.dll"), b"runtime").unwrap();

            let mut files = Vec::new();
            collect_files(&root, &root, &mut files).unwrap();
            files.sort_by_key(|file| file.relative_path.to_ascii_uppercase());
            let byte_count = files.iter().map(|file| file.bytes).sum();
            let package_tree_sha256 = tree_digest(&files);
            let catalog_sha256 = find_file(&files, "CAT.CAT").unwrap().sha256.clone();
            Self {
                configuration: DriverManifestConfiguration {
                    id: "test-manifest-v1".into(),
                    sha256: "0".repeat(64),
                    source_path: root.clone(),
                    inf_name: "driver.inf".into(),
                    inf_version: "01/02/2026, 1.2.3.4".into(),
                    catalog_name: "CAT.CAT".into(),
                    file_count: files.len() as u32,
                    byte_count,
                    package_tree_sha256,
                    catalog_sha256,
                },
                root,
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn inspects_complete_package_and_encodes_deterministically() {
        let fixture = Fixture::new();
        let first = inspect_driver_package(&fixture.configuration).unwrap();
        let second = inspect_driver_package(&fixture.configuration).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.files.len(), 3);
        assert_eq!(first.files[2].relative_path, r"sub\runtime.dll");
        assert_eq!(
            first.canonical_json().unwrap(),
            second.canonical_json().unwrap()
        );
        assert_eq!(first.sha256().unwrap().len(), 64);
    }

    #[test]
    fn rejects_missing_changed_and_partial_inputs() {
        let fixture = Fixture::new();
        fs::remove_file(fixture.root.join("sub").join("runtime.dll")).unwrap();
        assert!(matches!(
            inspect_driver_package(&fixture.configuration),
            Err(StagingError::PackageIdentityMismatch { .. })
        ));

        let fixture = Fixture::new();
        fs::write(fixture.root.join("sub").join("runtime.dll"), b"changed").unwrap();
        assert!(matches!(
            inspect_driver_package(&fixture.configuration),
            Err(StagingError::PackageIdentityMismatch { .. })
        ));

        let fixture = Fixture::new();
        fs::write(fixture.root.join("runtime.dll.partial-copy"), b"partial").unwrap();
        assert!(matches!(
            inspect_driver_package(&fixture.configuration),
            Err(StagingError::PackageIdentityMismatch { .. })
        ));

        let fixture = Fixture::new();
        fs::remove_file(fixture.root.join("CAT.CAT")).unwrap();
        assert!(matches!(
            inspect_driver_package(&fixture.configuration),
            Err(StagingError::PackageIdentityMismatch { .. })
        ));
    }

    #[test]
    fn rejects_inf_version_drift_after_tree_identity_matches() {
        let mut fixture = Fixture::new();
        fixture.configuration.inf_version = "01/02/2026, 9.9.9.9".into();
        assert_eq!(
            inspect_driver_package(&fixture.configuration),
            Err(StagingError::ChangedSource)
        );
    }

    #[test]
    fn rejects_non_ascii_package_paths() {
        let fixture = Fixture::new();
        fs::write(fixture.root.join("runtimé.dll"), b"unsafe-name").unwrap();
        assert_eq!(
            inspect_driver_package(&fixture.configuration),
            Err(StagingError::UnsafeSource)
        );
    }
}
