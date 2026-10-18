//! Current guest-file parity; historical receipts alone never certify preparation.
use crate::{
    guest::Receipt,
    payload::{self, CopyFile, Manifest},
    reporting::{Component, ComponentFailure, FailureCause},
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::Path,
    time::{Duration, Instant},
};

fn failure(component: Component, cause: FailureCause) -> String {
    ComponentFailure {
        stage: None,
        component,
        cause,
        code: None,
    }
    .into_error()
}

fn io_failure(error: std::io::Error, component: Component) -> String {
    failure(
        component,
        if error.kind() == std::io::ErrorKind::PermissionDenied {
            FailureCause::AccessDenied
        } else {
            FailureCause::Unavailable
        },
    )
}

fn security_failure(error: String, component: Component) -> String {
    let observation = crate::reporting::Diagnostic::observation("Guest payload read", None, &error);
    failure(
        component,
        if matches!(
            observation.availability,
            crate::reporting::Availability::Denied
        ) {
            FailureCause::AccessDenied
        } else {
            FailureCause::Unavailable
        },
    )
}

/// Authenticate request membership before resolving any Windows-relative path.
pub(super) fn validate(manifest: &Manifest, gpu: &str) -> Result<(), String> {
    let invalid = || failure(Component::GuestPayload, FailureCause::InvalidReply);
    if manifest.schema != 1
        || manifest.files.is_empty()
        || manifest.files.len() > 65536
        || manifest.device_id != payload::physical_device_id(gpu).map_err(|_| invalid())?
    {
        return Err(invalid());
    }
    let mut destinations = std::collections::BTreeSet::new();
    let mut total = 0_u64;
    for file in &manifest.files {
        payload::validate_relative(&file.destination).map_err(|_| invalid())?;
        if !destinations.insert(file.destination.to_ascii_lowercase())
            || file.sha256.len() != 64
            || !file.sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(invalid());
        }
        total = total
            .checked_add(file.bytes)
            .filter(|n| *n <= 16 * 1024 * 1024 * 1024)
            .ok_or_else(invalid)?;
    }
    Ok(())
}

fn compare(
    manifest: &Manifest,
    machine: &str,
    receipt: Result<Option<Receipt>, String>,
    observe: impl FnMut(&CopyFile) -> Result<Option<(u64, String)>, String>,
) -> Result<Receipt, String> {
    compare_files(manifest, observe)?;
    let digest = manifest.digest()?;
    let receipt =
        receipt?.ok_or_else(|| failure(Component::GuestReceipt, FailureCause::ReadbackMismatch))?;
    if machine.is_empty()
        || receipt.machine_guid != machine
        || receipt.digest != digest
        || receipt.files != manifest.files.len()
    {
        return Err(failure(
            Component::GuestReceipt,
            FailureCause::ReadbackMismatch,
        ));
    }
    Ok(receipt)
}

fn compare_files(
    manifest: &Manifest,
    mut observe: impl FnMut(&CopyFile) -> Result<Option<(u64, String)>, String>,
) -> Result<(), String> {
    for file in &manifest.files {
        match observe(file)? {
            None => return Err(failure(Component::GuestPayload, FailureCause::FileMissing)),
            Some((bytes, hash)) if bytes != file.bytes || hash != file.sha256 => {
                return Err(failure(Component::GuestPayload, FailureCause::FileChanged));
            }
            _ => {}
        }
    }
    Ok(())
}

fn deadline_check(deadline: Instant) -> Result<(), String> {
    if Instant::now() >= deadline {
        Err(failure(Component::GuestPayload, FailureCause::TimedOut))
    } else {
        Ok(())
    }
}

/// Read only current destinations and the receipt; never copy, install or publish.
pub(super) fn check(
    manifest: &Manifest,
    windows: &Path,
    receipt_path: &Path,
    machine: &str,
) -> Result<Receipt, String> {
    let deadline = Instant::now() + Duration::from_secs(300);
    let receipt = read_receipt(receipt_path);
    compare(manifest, machine, receipt, |file| observe_file(file, windows, deadline))
}

/// Independently authenticate installed destinations before publishing a setup
/// receipt. Share Verify's locked reads, bounded hashing and typed failures.
pub(super) fn check_installed(manifest: &Manifest, windows: &Path) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(300);
    compare_files(manifest, |file| observe_file(file, windows, deadline))
}

fn observe_file(
    file: &CopyFile,
    windows: &Path,
    deadline: Instant,
) -> Result<Option<(u64, String)>, String> {
        use std::os::windows::fs::OpenOptionsExt;
        deadline_check(deadline)?;
        let path = windows.join(&file.destination);
        match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(io_failure(e, Component::GuestPayload)),
            Ok(_) => {}
        }
        crate::security::verify_system(&path)
            .map_err(|error| security_failure(error, Component::GuestPayload))?;
        // Deny writes/deletion while hashing; one bounded buffer, no whole-file allocation.
        let mut input = fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&path)
            .map_err(|e| io_failure(e, Component::GuestPayload))?;
        let bytes = input
            .metadata()
            .map_err(|e| io_failure(e, Component::GuestPayload))?
            .len();
        if bytes != file.bytes {
            return Ok(Some((bytes, String::new())));
        }
        let mut hash = Sha256::new();
        crate::work_counts::add(crate::work_counts::Counter::FileHashes, 1);
        let mut buffer = [0_u8; 65536];
        let mut read = 0_u64;
        loop {
            deadline_check(deadline)?;
            let count = input
                .read(&mut buffer)
                .map_err(|e| io_failure(e, Component::GuestPayload))?;
            if count == 0 {
                break;
            }
            read = read
                .checked_add(count as u64)
                .filter(|n| *n <= bytes)
                .ok_or_else(|| failure(Component::GuestPayload, FailureCause::FileChanged))?;
            hash.update(&buffer[..count]);
            crate::work_counts::add(crate::work_counts::Counter::HashedBytes, count as u64);
        }
        deadline_check(deadline)?;
        if read != bytes {
            return Err(failure(Component::GuestPayload, FailureCause::FileChanged));
        }
        Ok(Some((
            bytes,
            hash.finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        )))
}

fn read_receipt(path: &Path) -> Result<Option<Receipt>, String> {
    use std::os::windows::fs::OpenOptionsExt;
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(io_failure(e, Component::GuestReceipt)),
        Ok(_) => {}
    }
    crate::security::verify(path)
        .map_err(|error| security_failure(error, Component::GuestReceipt))?;
    let input = fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(path)
        .map_err(|e| io_failure(e, Component::GuestReceipt))?;
    let mut bytes = Vec::new();
    input
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(|e| io_failure(e, Component::GuestReceipt))?;
    if bytes.len() > 65536 {
        return Err(failure(Component::GuestReceipt, FailureCause::InvalidReply));
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|_| failure(Component::GuestReceipt, FailureCause::InvalidReply))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> Manifest {
        Manifest {
            schema: 1,
            device_id: r"PCI\VEN_10DE&DEV_2D05\fixture".into(),
            driver_version: "unchanged-driver".into(),
            catalogs: vec![],
            files: vec![
                CopyFile {
                    source: "host-a".into(),
                    destination: r"System32\HostDriverStore\a.dll".into(),
                    bytes: 10,
                    sha256: "a".repeat(64),
                },
                CopyFile {
                    source: "host-b".into(),
                    destination: r"System32\b.dll".into(),
                    bytes: 20,
                    sha256: "b".repeat(64),
                },
            ],
        }
    }

    fn receipt(manifest: &Manifest) -> Receipt {
        Receipt {
            digest: manifest.digest().unwrap(),
            machine_guid: "current-guest".into(),
            files: manifest.files.len(),
        }
    }

    fn cause(error: &str, component: &str, expected: &str) {
        let decoded = ComponentFailure::decode(error).unwrap();
        assert_eq!(format!("{:?}", decoded.component), component);
        assert_eq!(format!("{:?}", decoded.cause), expected);
    }

    #[test]
    fn current_files_are_all_checked_even_with_unchanged_host_and_guest_receipts() {
        let manifest = manifest();
        let mut seen = Vec::new();
        let result = compare(
            &manifest,
            "current-guest",
            Ok(Some(receipt(&manifest))),
            |file| {
                seen.push(file.destination.clone());
                Ok(Some((file.bytes, file.sha256.clone())))
            },
        )
        .unwrap();
        assert_eq!(seen.len(), 2);
        assert_eq!(result.digest, manifest.digest().unwrap());
        assert_eq!(result.files, 2);
        for missing in 0..2 {
            let error = compare(
                &manifest,
                "current-guest",
                Ok(Some(receipt(&manifest))),
                |file| {
                    Ok((file.destination != manifest.files[missing].destination)
                        .then(|| (file.bytes, file.sha256.clone())))
                },
            )
            .err()
            .unwrap();
            cause(&error, "GuestPayload", "FileMissing");
        }
    }

    #[test]
    fn changed_lengths_and_corrupted_hashes_do_not_certify_current_payload() {
        let manifest = manifest();
        for length_changed in [false, true] {
            let error = compare(
                &manifest,
                "current-guest",
                Ok(Some(receipt(&manifest))),
                |file| {
                    Ok(Some(if length_changed {
                        (file.bytes + 1, file.sha256.clone())
                    } else {
                        (file.bytes, "c".repeat(64))
                    }))
                },
            )
            .err()
            .unwrap();
            cause(&error, "GuestPayload", "FileChanged");
        }
    }

    #[test]
    fn absent_wrong_guest_wrong_payload_and_partial_receipts_fail_independently() {
        let manifest = manifest();
        for variant in 0..4 {
            let mut receipt = receipt(&manifest);
            match variant {
                1 => receipt.machine_guid = "other-guest".into(),
                2 => receipt.digest = "other-gpu-payload".into(),
                3 => receipt.files -= 1,
                _ => {}
            }
            let error = compare(
                &manifest,
                "current-guest",
                Ok((variant != 0).then_some(receipt)),
                |file| Ok(Some((file.bytes, file.sha256.clone()))),
            )
            .err()
            .unwrap();
            cause(&error, "GuestReceipt", "ReadbackMismatch");
        }
    }

    #[test]
    fn denied_unavailable_and_timed_out_observations_remain_unknown_not_mismatch() {
        let manifest = manifest();
        for reason in [
            FailureCause::AccessDenied,
            FailureCause::Unavailable,
            FailureCause::TimedOut,
        ] {
            let error = compare(
                &manifest,
                "current-guest",
                Ok(Some(receipt(&manifest))),
                |_| Err(failure(Component::GuestPayload, reason)),
            )
            .err()
            .unwrap();
            cause(&error, "GuestPayload", &format!("{reason:?}"));
        }
        let error = compare(
            &manifest,
            "current-guest",
            Err(failure(Component::GuestReceipt, FailureCause::AccessDenied)),
            |file| Ok(Some((file.bytes, file.sha256.clone()))),
        )
        .err()
        .unwrap();
        cause(&error, "GuestReceipt", "AccessDenied");
        cause(
            &deadline_check(Instant::now()).unwrap_err(),
            "GuestPayload",
            "TimedOut",
        );
    }

    #[test]
    fn manifest_refuses_wrong_gpu_duplicate_destinations_and_unsafe_paths() {
        let mut manifest = manifest();
        let gpu =
            r"\\?\PCI#VEN_10DE&DEV_2D05#fixture#{12345678-1234-1234-1234-123456789abc}\GPUPARAV";
        validate(&manifest, gpu).unwrap();
        assert!(
            validate(
                &manifest,
                r"\\?\PCI#VEN_10DE&DEV_0001#other#{12345678-1234-1234-1234-123456789abc}\GPUPARAV"
            )
            .is_err()
        );
        manifest.files[1].destination = manifest.files[0].destination.to_ascii_uppercase();
        assert!(validate(&manifest, gpu).is_err());
        manifest.files[1].destination = r"..\outside.dll".into();
        assert!(validate(&manifest, gpu).is_err());
    }
}
