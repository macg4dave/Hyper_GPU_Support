//! Validated, hardware-independent contract for one PowerShell Direct file transfer.

use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};
use zeroize::{Zeroize, Zeroizing};

use crate::config::GuestConfiguration;

/// Maximum source size accepted by the initial single-file transfer path.
pub const MAX_TRANSFER_BYTES: u64 = 4 * 1024 * 1024 * 1024;

/// Ephemeral guest credential whose password is zeroed when dropped.
pub struct GuestCredential {
    username: String,
    password: Zeroizing<String>,
}

impl GuestCredential {
    /// Construct a bounded credential without persisting or logging its password.
    ///
    /// # Errors
    /// Returns [`GuestError::InvalidCredential`] for empty, oversized or control-bearing input.
    pub fn new(username: String, mut password: String) -> Result<Self, GuestError> {
        if username.is_empty()
            || username.len() > 256
            || username.chars().any(char::is_control)
            || password.is_empty()
            || password.len() > 1024
        {
            password.zeroize();
            return Err(GuestError::InvalidCredential);
        }
        Ok(Self {
            username,
            password: Zeroizing::new(password),
        })
    }

    /// Guest-local account name.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Expose the password only to the bounded transport encoder.
    #[must_use]
    pub(crate) fn password(&self) -> &str {
        self.password.as_str()
    }
}

/// One immutable host file and its flat destination name below the configured guest root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferRequest {
    source: PathBuf,
    destination: PathBuf,
    sha256: String,
}

impl TransferRequest {
    /// Validate a transfer request before any guest session is opened.
    ///
    /// # Errors
    /// Rejects non-absolute sources, unsafe guest-relative paths and non-canonical hashes.
    pub fn new(source: PathBuf, destination: PathBuf, sha256: String) -> Result<Self, GuestError> {
        if !source.is_absolute() || !safe_relative_path(&destination) {
            return Err(GuestError::UnsafePath);
        }
        if !is_sha256(&sha256) {
            return Err(GuestError::InvalidHash);
        }
        Ok(Self {
            source,
            destination,
            sha256,
        })
    }

    /// Absolute host source path.
    #[must_use]
    pub fn source(&self) -> &Path {
        &self.source
    }

    /// Flat filename below the configured guest staging root.
    #[must_use]
    pub fn destination(&self) -> &Path {
        &self.destination
    }

    /// Expected lowercase SHA-256 for both source and destination.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

/// Verified observable result returned by a guest transport adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferReceipt {
    /// VM GUID used for the PowerShell Direct session.
    pub vm_id: String,
    /// Guest computer name observed in-session.
    pub computer_name: String,
    /// Guest MachineGuid observed in-session.
    pub machine_guid: String,
    /// Canonical destination reported by the guest.
    pub destination: PathBuf,
    /// SHA-256 measured in the guest after the atomic rename.
    pub sha256: String,
    /// Exact transferred byte count.
    pub bytes: u64,
}

/// Narrow adapter implemented by the Windows PowerShell Direct boundary.
pub trait GuestTransfer {
    /// Copy one already-validated request and return guest-observed evidence.
    fn transfer(
        &self,
        credential: &GuestCredential,
        request: &TransferRequest,
    ) -> Result<TransferReceipt, GuestError>;
}

/// Verify the host file, execute the adapter and bind its receipt to configured identities.
///
/// # Errors
/// Returns a categorized failure without credential or file content in its diagnostics.
pub fn transfer_verified(
    guest: &GuestConfiguration,
    vm_id: &str,
    credential: &GuestCredential,
    request: &TransferRequest,
    adapter: &impl GuestTransfer,
) -> Result<TransferReceipt, GuestError> {
    let metadata = request.source.symlink_metadata().map_err(source_error)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(GuestError::UnsafePath);
    }
    if metadata.len() > MAX_TRANSFER_BYTES {
        return Err(GuestError::SourceTooLarge);
    }
    let measured = hash_file(&request.source)?;
    if measured != request.sha256 {
        return Err(GuestError::SourceHashMismatch);
    }
    let receipt = adapter.transfer(credential, request)?;
    let expected_destination = guest.staging_root.join(&request.destination);
    if receipt.vm_id != vm_id
        || !receipt
            .computer_name
            .eq_ignore_ascii_case(&guest.computer_name)
        || receipt.machine_guid != guest.machine_guid
        || !paths_equal(&receipt.destination, &expected_destination)
        || receipt.sha256 != request.sha256
        || receipt.bytes != metadata.len()
    {
        return Err(GuestError::VerificationFailed);
    }
    Ok(receipt)
}

fn hash_file(path: &Path) -> Result<String, GuestError> {
    let mut file = File::open(path).map_err(source_error)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 128 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(source_error)?;
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

fn safe_relative_path(path: &Path) -> bool {
    let mut components = path.components();
    let Some(Component::Normal(value)) = components.next() else {
        return false;
    };
    let value = value.to_string_lossy();
    components.next().is_none()
        && !value.is_empty()
        && !value.ends_with([' ', '.'])
        && !value.contains([':', '\0'])
        && !value.chars().any(char::is_control)
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    left.to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn source_error(error: io::Error) -> GuestError {
    GuestError::SourceIo {
        kind: error.kind(),
        code: error.raw_os_error(),
    }
}

/// Guest session and transfer failure categories safe to show to an operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuestError {
    /// Runtime credentials were malformed before use.
    InvalidCredential,
    /// A source or destination path violated the transfer boundary.
    UnsafePath,
    /// The expected digest was not canonical SHA-256.
    InvalidHash,
    /// The host source could not be inspected or read.
    SourceIo {
        /// Stable standard-library error category.
        kind: io::ErrorKind,
        /// Native OS error code when available.
        code: Option<i32>,
    },
    /// The initial single-file lane rejects sources above its fixed bound.
    SourceTooLarge,
    /// The host source no longer matches its reviewed digest.
    SourceHashMismatch,
    /// Guest credentials were denied.
    CredentialDenied,
    /// PowerShell Direct or its integration service was unavailable.
    IntegrationUnavailable,
    /// A bounded operation was interrupted; guest staging state is uncertain.
    Interrupted,
    /// The adapter failed without establishing that guest state remained unchanged.
    AdapterFailed,
    /// The returned identity, path, length or hash did not match the request.
    VerificationFailed,
    /// Adapter output was malformed or exceeded its bound.
    InvalidProtocol,
}

impl fmt::Display for GuestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidCredential => "invalid guest credential",
            Self::UnsafePath => "unsafe transfer path",
            Self::InvalidHash => "invalid transfer hash",
            Self::SourceIo { .. } => "cannot read transfer source",
            Self::SourceTooLarge => "transfer source exceeds the fixed size limit",
            Self::SourceHashMismatch => "transfer source hash mismatch",
            Self::CredentialDenied => "guest credentials were denied",
            Self::IntegrationUnavailable => "PowerShell Direct is unavailable",
            Self::Interrupted => "guest transfer was interrupted; guest staging state is uncertain",
            Self::AdapterFailed => {
                "guest transfer adapter failed; guest staging state is uncertain"
            }
            Self::VerificationFailed => "guest transfer verification failed",
            Self::InvalidProtocol => "invalid guest transfer adapter response",
        })
    }
}

impl std::error::Error for GuestError {}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    struct Fake(Result<TransferReceipt, GuestError>);

    impl GuestTransfer for Fake {
        fn transfer(
            &self,
            _credential: &GuestCredential,
            _request: &TransferRequest,
        ) -> Result<TransferReceipt, GuestError> {
            self.0.clone()
        }
    }

    fn guest() -> GuestConfiguration {
        GuestConfiguration {
            powershell_path: PathBuf::from(
                r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
            ),
            computer_name: "TESTVM".into(),
            machine_guid: "046edc35-4c8f-4910-9c53-574681e623af".into(),
            staging_root: PathBuf::from(r"C:\Program Files\HyperGpuSupport\Staging"),
            session_timeout: std::time::Duration::from_secs(60),
            transfer_timeout: std::time::Duration::from_secs(900),
            staging_timeout: std::time::Duration::from_secs(3600),
        }
    }

    #[test]
    fn rejects_credentials_unsafe_paths_and_hashes() {
        assert!(matches!(
            GuestCredential::new("user".into(), String::new()),
            Err(GuestError::InvalidCredential)
        ));
        let hash = "0".repeat(64);
        for destination in [
            r"..\escape",
            r"C:\absolute",
            r"safe\..\escape",
            r"nested\file.bin",
            "bad:name",
        ] {
            assert_eq!(
                TransferRequest::new(
                    PathBuf::from(r"C:\source.bin"),
                    PathBuf::from(destination),
                    hash.clone()
                ),
                Err(GuestError::UnsafePath)
            );
        }
        assert_eq!(
            TransferRequest::new(
                PathBuf::from(r"C:\source.bin"),
                PathBuf::from("safe.bin"),
                "ABC".into()
            ),
            Err(GuestError::InvalidHash)
        );
    }

    #[test]
    fn verifies_source_and_every_receipt_field() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let source = std::env::temp_dir().join(format!("hyper-gpu-guest-{unique}.bin"));
        fs::write(&source, b"verified transfer").unwrap();
        let hash = hex(&Sha256::digest(b"verified transfer"));
        let request =
            TransferRequest::new(source.clone(), "input.bin".into(), hash.clone()).unwrap();
        let receipt = TransferReceipt {
            vm_id: "2627e735-5b33-4104-b739-622727dd3a40".into(),
            computer_name: "testvm".into(),
            machine_guid: guest().machine_guid,
            destination: guest().staging_root.join("input.bin"),
            sha256: hash,
            bytes: 17,
        };
        let credential = GuestCredential::new("user".into(), "secret".into()).unwrap();
        assert_eq!(
            transfer_verified(
                &guest(),
                "2627e735-5b33-4104-b739-622727dd3a40",
                &credential,
                &request,
                &Fake(Ok(receipt.clone()))
            ),
            Ok(receipt.clone())
        );
        let mut wrong = receipt;
        wrong.machine_guid = "00000000-0000-0000-0000-000000000000".into();
        assert_eq!(
            transfer_verified(
                &guest(),
                "2627e735-5b33-4104-b739-622727dd3a40",
                &credential,
                &request,
                &Fake(Ok(wrong))
            ),
            Err(GuestError::VerificationFailed)
        );
        fs::remove_file(source).unwrap();
    }

    #[test]
    fn preserves_denied_unavailable_and_interrupted_categories() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let source = std::env::temp_dir().join(format!("hyper-gpu-guest-errors-{unique}.bin"));
        fs::write(&source, b"error paths").unwrap();
        let request = TransferRequest::new(
            source.clone(),
            PathBuf::from("input.bin"),
            hex(&Sha256::digest(b"error paths")),
        )
        .unwrap();
        let credential = GuestCredential::new("user".into(), "secret".into()).unwrap();
        for error in [
            GuestError::CredentialDenied,
            GuestError::IntegrationUnavailable,
            GuestError::Interrupted,
        ] {
            assert_eq!(
                transfer_verified(
                    &guest(),
                    "2627e735-5b33-4104-b739-622727dd3a40",
                    &credential,
                    &request,
                    &Fake(Err(error.clone()))
                ),
                Err(error)
            );
        }
        fs::remove_file(source).unwrap();
    }
}
