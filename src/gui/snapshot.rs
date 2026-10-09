//! Read-only rehearsal input. Snapshot contents never authorize backend operations.
use hyper_gpu_support::{
    gui_model::Inventory,
    model::{Allocation, Gpu, Power, Settings, Target, VmState},
    workflow::{self, Backend, Journal, Plan},
};
use std::{io::Read, path::Path};

const MAX_SNAPSHOT_BYTES: u64 = 8 * 1024 * 1024;

pub(super) fn capture(path: &Path) -> Result<(Inventory, Vec<u8>), String> {
    let bytes = read_bytes(path)?;
    Ok((parse(&bytes)?, bytes))
}

fn read_bytes(path: &Path) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path)
        .map_err(|error| format!("Cannot open inventory snapshot: {error}"))?;
    let mut bytes = Vec::new();
    file.take(MAX_SNAPSHOT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Cannot read inventory snapshot: {error}"))?;
    if bytes.len() as u64 > MAX_SNAPSHOT_BYTES {
        return Err("Inventory snapshot exceeds the 8 MiB input limit".into());
    }
    Ok(bytes)
}

/// Recompute the shared plan exclusively from historical inputs. A recorded plan
/// supplies a recorded payload digest; never infer driver parity
/// from a journal alone or discover/hash host payloads in this route.
pub(super) fn plan(path: &Path, target: &Target, expected: Option<&[u8]>) -> Result<Plan, String> {
    let bytes = read_bytes(path)?;
    plan_bytes(&bytes, target, expected)
}

fn plan_bytes(bytes: &[u8], target: &Target, expected: Option<&[u8]>) -> Result<Plan, String> {
    if expected.is_some_and(|expected| expected != bytes) {
        return Err("Snapshot changed externally. Draft preserved; discard and Refresh.".into());
    }
    let inventory = parse(bytes)?;
    let mut view =
        hyper_gpu_support::gui_model::View::new(hyper_gpu_support::model::Configuration {
            schema: 2,
            targets: vec![],
        });
    view.refresh(inventory.clone());
    view.eligibility(target)?;
    if inventory.recorded_state(&target.vm_id).recovery_required {
        return Err("Historical recovery is unresolved; rehearsal cannot clear it.".into());
    }
    let digest = if target.enabled {
        let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        let plans: Vec<Plan> = serde_json::from_value(value.get("plans").cloned().ok_or("Enabled rehearsal requires a recorded shared plan in snapshot plans; no current payload is discovered.")?)
            .map_err(|e| format!("Invalid recorded plans: {e}"))?;
        let gpu = inventory
            .discovery
            .gpus
            .iter()
            .find(|g| g.interface == target.gpu_interface)
            .ok_or("Recorded GPU unavailable")?;
        let matching = plans
            .iter()
            .filter(|p| {
                p.desired.vm_id == target.vm_id
                    && p.desired.gpu_interface == target.gpu_interface
                    && p.observed.vm_id == target.vm_id
            })
            .filter_map(|p| p.preparation.as_ref())
            .filter(|p| p.current_driver == gpu.driver_version && !p.digest.is_empty())
            .map(|p| p.digest.clone())
            .collect::<std::collections::BTreeSet<_>>();
        if matching.len() != 1 {
            return Err("A unique recorded payload digest for this VM/GPU/driver is required; Refresh the capture inputs.".into());
        }
        matching.into_iter().next()
    } else {
        None
    };
    workflow::plan(&mut RecordedBackend { inventory, digest }, target)
}

struct RecordedBackend {
    inventory: Inventory,
    digest: Option<String>,
}
impl Backend for RecordedBackend {
    fn inspect(&mut self, t: &Target) -> Result<VmState, String> {
        self.inventory
            .discovery
            .vms
            .iter()
            .find(|v| v.vm_id == t.vm_id)
            .cloned()
            .ok_or("Recorded VM unavailable".into())
    }
    fn gpu(&mut self, t: &Target) -> Result<Gpu, String> {
        self.inventory
            .discovery
            .gpus
            .iter()
            .find(|g| g.interface == t.gpu_interface)
            .cloned()
            .ok_or("Recorded GPU unavailable".into())
    }
    fn journal(&mut self, t: &Target) -> Result<Option<Journal>, String> {
        self.inventory
            .managed
            .get(&t.vm_id)
            .cloned()
            .ok_or("Managed state was not captured; absence cannot be inferred".into())
    }
    fn payload(&mut self, _: &Target) -> Result<String, String> {
        self.digest
            .clone()
            .ok_or("Historical payload digest unavailable".into())
    }
    fn save(&mut self, _: &Target, _: &Journal) -> Result<(), String> {
        Err("Rehearsal writes are blocked".into())
    }
    fn power(&mut self, _: &Target, _: Power) -> Result<(), String> {
        Err("Rehearsal effects are blocked".into())
    }
    fn assign(&mut self, _: &Target, _: bool) -> Result<(), String> {
        Err("Rehearsal effects are blocked".into())
    }
    fn settings(&mut self, _: &Target, _: &Settings) -> Result<(), String> {
        Err("Rehearsal effects are blocked".into())
    }
    fn allocation(&mut self, _: &Target, _: &Allocation) -> Result<(), String> {
        Err("Rehearsal effects are blocked".into())
    }
    fn prepare(&mut self, _: &Target) -> Result<String, String> {
        Err("Rehearsal effects are blocked".into())
    }
    fn verify(&mut self, _: &Target, _: &Gpu) -> Result<(), String> {
        Err("Rehearsal effects are blocked".into())
    }
}

fn parse(bytes: &[u8]) -> Result<Inventory, String> {
    if bytes.len() as u64 > MAX_SNAPSHOT_BYTES {
        return Err("Inventory snapshot exceeds the 8 MiB input limit".into());
    }
    let mut value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| format!("Invalid inventory snapshot JSON: {error}"))?;
    let object = value
        .as_object_mut()
        .ok_or("Inventory snapshot must be a JSON object")?;
    // Elevated CLI inventory returns Discovery without protected managed records.
    // Absence remains unread, never proof of no journal or enrollment.
    object
        .entry("managed")
        .or_insert_with(|| serde_json::json!({}));
    serde_json::from_value(value)
        .map_err(|error| format!("Invalid inventory snapshot data: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> (serde_json::Value, Target) {
        let target = Target {
            vm_id: "12345678-1234-1234-1234-123456789abc".into(),
            gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#fixture\GPUPARAV".into(),
            enabled: true,
            vram: None,
        };
        let settings = Settings {
            low_mmio: 0,
            high_mmio: 0,
            cache_types: false,
            automatic_checkpoints: true,
        };
        let vm = VmState {
            vm_id: target.vm_id.clone(),
            name: "Synthetic VM".into(),
            generation: 2,
            power: Power::Running,
            gpus: vec![],
            settings,
            vram: None,
        };
        let gpu = Gpu {
            interface: target.gpu_interface.clone(),
            name: "Synthetic GPU".into(),
            vendor: 0x10de,
            device: 1,
            driver_version: "synthetic-driver".into(),
            preparation_supported: true,
            vram: Allocation {
                minimum: 10,
                optimal: 50,
                maximum: 100,
            },
        };
        let value = serde_json::json!({"vms": [vm], "gpus": [gpu], "enrolled": [target], "managed": {(target.vm_id.clone()): null}});
        (value, target)
    }

    #[test]
    fn recorded_inputs_recompute_identical_shared_plan_and_allow_new_vram_draft() {
        let (mut value, mut target) = inputs();
        let inventory = parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        let mut backend = RecordedBackend {
            inventory,
            digest: Some("synthetic-payload-digest".into()),
        };
        let captured = workflow::plan(&mut backend, &target).unwrap();
        value["plans"] = serde_json::json!([captured]);
        let bytes = serde_json::to_vec(&value).unwrap();
        let replay = plan_bytes(&bytes, &target, Some(&bytes)).unwrap();
        assert_eq!(
            serde_json::to_value(&captured).unwrap(),
            serde_json::to_value(replay).unwrap()
        );
        target.vram = Some(Allocation {
            minimum: 10,
            optimal: 40,
            maximum: 80,
        });
        let expected = workflow::plan(&mut backend, &target).unwrap();
        let replay = plan_bytes(&bytes, &target, Some(&bytes)).unwrap();
        assert_eq!(
            serde_json::to_value(expected).unwrap(),
            serde_json::to_value(replay).unwrap()
        );
        assert!(
            plan_bytes(&bytes, &target, Some(b"changed"))
                .unwrap_err()
                .contains("changed externally")
        );
        let mut conflicting = captured.clone();
        conflicting.preparation.as_mut().unwrap().digest = "conflicting-recorded-digest".into();
        value["plans"] = serde_json::json!([captured, conflicting]);
        assert!(
            plan_bytes(&serde_json::to_vec(&value).unwrap(), &target, None)
                .unwrap_err()
                .contains("unique recorded payload digest")
        );
        value["gpus"][0]["driver_version"] = "different-driver".into();
        assert!(plan_bytes(&serde_json::to_vec(&value).unwrap(), &target, None).is_err());
    }

    #[test]
    fn missing_inputs_do_not_invent_payload_journal_or_enrollment() {
        let (mut value, mut target) = inputs();
        assert!(
            plan_bytes(&serde_json::to_vec(&value).unwrap(), &target, None)
                .unwrap_err()
                .contains("recorded shared plan")
        );
        target.enabled = false;
        assert!(plan_bytes(&serde_json::to_vec(&value).unwrap(), &target, None).is_ok());
        value["managed"] = serde_json::json!({});
        assert!(
            plan_bytes(&serde_json::to_vec(&value).unwrap(), &target, None)
                .unwrap_err()
                .contains("not captured")
        );
        value["enrolled"] = serde_json::json!([]);
        assert!(
            plan_bytes(&serde_json::to_vec(&value).unwrap(), &target, None)
                .unwrap_err()
                .contains("not enrolled")
        );
    }

    #[test]
    fn mutation_methods_are_unconditionally_refused() {
        let (value, target) = inputs();
        let inventory = parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        let mut backend = RecordedBackend {
            inventory,
            digest: None,
        };
        let vm = backend.inspect(&target).unwrap();
        let gpu = backend.gpu(&target).unwrap();
        assert!(backend.power(&target, Power::Off).is_err());
        assert!(backend.assign(&target, true).is_err());
        assert!(backend.settings(&target, &vm.settings).is_err());
        assert!(backend.allocation(&target, &gpu.vram).is_err());
        assert!(backend.prepare(&target).is_err());
        assert!(backend.verify(&target, &gpu).is_err());
        let journal = Journal {
            schema: 1,
            vm_id: target.vm_id.clone(),
            gpu_interface: target.gpu_interface.clone(),
            original: vm.settings.clone(),
            applied: vm.settings.clone(),
            prepared: None,
            pending: true,
            restore_power: Power::Running,
            verification_only: false,
            last_verified: None,
        };
        assert!(backend.save(&target, &journal).is_err());
        let mut recovery = value;
        recovery["managed"][&target.vm_id] = serde_json::to_value(journal).unwrap();
        let mut disabled = target;
        disabled.enabled = false;
        assert!(
            plan_bytes(&serde_json::to_vec(&recovery).unwrap(), &disabled, None)
                .unwrap_err()
                .contains("recovery is unresolved")
        );
    }

    #[test]
    fn native_snapshot_does_not_invent_enrollment_or_managed_records() {
        let inventory = parse(br#"{"vms":[],"gpus":[]}"#).unwrap();
        assert!(inventory.enrolled.is_empty());
        assert!(inventory.managed.is_empty());
        assert!(
            inventory
                .recorded_state("vm")
                .preparation
                .contains("unavailable")
        );
    }

    #[test]
    fn protected_snapshot_preserves_explicit_absent_records() {
        let inventory =
            parse(br#"{"vms":[],"gpus":[],"enrolled":[],"managed":{"vm":null}}"#).unwrap();
        assert_eq!(
            inventory.recorded_state("vm").preparation,
            "No preparation record"
        );
    }

    #[test]
    fn malformed_snapshot_fails_instead_of_substituting_fixtures() {
        for bytes in [
            b"null".as_slice(),
            b"[]",
            b"{}",
            b"{",
            br#"{"vms":[],"gpus":[],"managed":false}"#,
        ] {
            assert!(parse(bytes).is_err());
        }
    }

    #[test]
    fn oversized_snapshot_is_rejected_before_json_parsing() {
        assert!(
            parse(&vec![b' '; MAX_SNAPSHOT_BYTES as usize + 1])
                .unwrap_err()
                .contains("input limit")
        );
    }
}
