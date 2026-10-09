//! Read-only rehearsal input. Snapshot contents never authorize backend operations.
use hyper_gpu_support::gui_model::Inventory;
use std::{io::Read, path::Path};

const MAX_SNAPSHOT_BYTES: u64 = 8 * 1024 * 1024;

pub(super) fn read(path: &Path) -> Result<Inventory, String> {
    let file = std::fs::File::open(path)
        .map_err(|error| format!("Cannot open inventory snapshot: {error}"))?;
    let mut bytes = Vec::new();
    file.take(MAX_SNAPSHOT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Cannot read inventory snapshot: {error}"))?;
    parse(&bytes)
}

fn parse(bytes: &[u8]) -> Result<Inventory, String> {
    if bytes.len() as u64 > MAX_SNAPSHOT_BYTES {
        return Err("Inventory snapshot exceeds the 8 MiB input limit".into());
    }
    let mut value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| format!("Invalid inventory snapshot JSON: {error}"))?;
    let object = value.as_object_mut().ok_or("Inventory snapshot must be a JSON object")?;
    // Elevated CLI inventory returns Discovery without protected managed records.
    // Absence remains unread, never proof of no journal or enrollment.
    object.entry("managed").or_insert_with(|| serde_json::json!({}));
    serde_json::from_value(value)
        .map_err(|error| format!("Invalid inventory snapshot data: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_snapshot_does_not_invent_enrollment_or_managed_records() {
        let inventory = parse(br#"{"vms":[],"gpus":[]}"#).unwrap();
        assert!(inventory.enrolled.is_empty());
        assert!(inventory.managed.is_empty());
        assert!(inventory.recorded_state("vm").preparation.contains("unavailable"));
    }

    #[test]
    fn protected_snapshot_preserves_explicit_absent_records() {
        let inventory = parse(br#"{"vms":[],"gpus":[],"enrolled":[],"managed":{"vm":null}}"#).unwrap();
        assert_eq!(inventory.recorded_state("vm").preparation, "No preparation record");
    }

    #[test]
    fn malformed_snapshot_fails_instead_of_substituting_fixtures() {
        for bytes in [b"null".as_slice(), b"[]", b"{}", b"{", br#"{"vms":[],"gpus":[],"managed":false}"#] {
            assert!(parse(bytes).is_err());
        }
    }

    #[test]
    fn oversized_snapshot_is_rejected_before_json_parsing() {
        assert!(parse(&vec![b' '; MAX_SNAPSHOT_BYTES as usize + 1])
            .unwrap_err().contains("input limit"));
    }
}
