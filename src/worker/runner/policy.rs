//! Protected enrollment policy, exact identity validation and installed artifact checks.
use super::{
    protocol::FRAME_LIMIT,
    storage::{data_directory, install_directory},
};
use crate::{
    model::{Configuration, Discovery, Target},
    payload,
};
use serde::{Deserialize, Serialize};
use std::{fs, io::Read};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Enrollment {
    pub(super) schema: u32,
    pub(crate) client_sid: String,
    pub(crate) targets: Vec<Target>,
    pub(crate) artifacts: std::collections::BTreeMap<String, String>,
}

pub(super) fn enrollment_revision(
    value: &Enrollment,
) -> Result<crate::configuration_store::Revision, String> {
    Ok(crate::configuration_store::Revision::of(
        &serde_json::to_vec(value).map_err(|e| e.to_string())?,
    ))
}
pub(super) fn validate_enrollment(
    configuration: &Configuration,
    discovery: &Discovery,
) -> Result<(), String> {
    configuration.validate()?;
    for target in &configuration.targets {
        let vm = discovery
            .vms
            .iter()
            .find(|vm| vm.vm_id.eq_ignore_ascii_case(&target.vm_id))
            .ok_or_else(|| {
                format!(
                    "enrollment VM {} is absent from current native discovery",
                    target.vm_id
                )
            })?;
        if vm.generation != 2 {
            return Err(format!(
                "enrollment VM {} must be Hyper-V Generation 2",
                target.vm_id
            ));
        }
        if !discovery
            .gpus
            .iter()
            .any(|gpu| gpu.interface == target.gpu_interface)
        {
            return Err(format!(
                "enrollment GPU for VM {} is absent from current native discovery",
                target.vm_id
            ));
        }
    }
    Ok(())
}
pub(crate) fn enrollment() -> Result<Enrollment, String> {
    let root = data_directory()?;
    crate::security::verify(&root)?;
    if root.join("installation-pending.json").exists() {
        return Err(
            "runner installation was interrupted; rerun administrator install before operations"
                .into(),
        );
    }
    read_enrollment()
}

pub(super) fn read_enrollment() -> Result<Enrollment, String> {
    let root = data_directory()?;
    crate::security::verify(&root)?;
    let path = root.join("enrollment.json");
    crate::security::verify(&path)?;
    parse_enrollment(
        fs::File::open(&path).map_err(
            |_| "product runner is not installed; use install with administrator rights",
        )?,
    )
}
fn parse_enrollment(input: impl Read) -> Result<Enrollment, String> {
    let mut b = Vec::new();
    input
        .take(FRAME_LIMIT as u64 + 1)
        .read_to_end(&mut b)
        .map_err(|e| e.to_string())?;
    if b.len() > FRAME_LIMIT {
        return Err("invalid enrollment size".into());
    }
    let value: Enrollment = serde_json::from_slice(&b).map_err(|e| e.to_string())?;
    if value.schema != 2 {
        return Err("unsupported enrollment version".into());
    }
    if !(value.artifacts.len() == 3
        || (value.artifacts.len() == 4 && value.artifacts.contains_key("hyper-gpu-support.exe")))
        || ![
            "hyper-gpu-runner.exe",
            "hyper-gpu-guest.exe",
            "d3d11-probe.exe",
        ]
        .iter()
        .all(|name| value.artifacts.contains_key(*name))
    {
        return Err("incomplete installed artifact policy".into());
    }
    Ok(value)
}
pub(crate) fn verify_artifacts(e: &Enrollment) -> Result<(), String> {
    crate::diagnostics::run("artifact-authentication", "enrolled-product", || {
        verify_artifacts_inner(e)
    })
}

/// Invoked once under the operation lock, before any observation/reply. There is
/// no earlier unlocked attestation to reuse or persistent digest cache.
pub(super) fn authenticate_read(
    expected: &Enrollment,
    current: &Enrollment,
    authenticate: impl FnOnce(&Enrollment) -> Result<(), String>,
) -> Result<(), String> {
    if serde_json::to_value(expected).map_err(|e| e.to_string())?
        != serde_json::to_value(current).map_err(|e| e.to_string())?
    {
        return Err("enrollment changed before admission; retry discovery".into());
    }
    authenticate(current)
}

fn verify_artifacts_inner(e: &Enrollment) -> Result<(), String> {
    for (name, hash) in &e.artifacts {
        crate::read_context::check()?;
        if ![
            "hyper-gpu-runner.exe",
            "hyper-gpu-guest.exe",
            "d3d11-probe.exe",
            "hyper-gpu-support.exe",
        ]
        .contains(&name.as_str())
        {
            return Err("unrecognized installed product artifact".into());
        }
        let path = install_directory()?.join(name);
        crate::security::verify(&path)?;
        if payload::hash_file(&path)? != *hash {
            return Err("installed product artifact changed".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Allocation, Gpu, Power, Settings, VmState};

    #[test]
    fn read_admission_authenticates_once_and_never_reuses_a_changed_policy() {
        let policy = Enrollment {
            schema: 2,
            client_sid: "fixture".into(),
            targets: vec![],
            artifacts: [
                "hyper-gpu-runner.exe",
                "hyper-gpu-guest.exe",
                "d3d11-probe.exe",
                "hyper-gpu-support.exe",
            ]
            .into_iter()
            .map(|name| (name.into(), "hash".into()))
            .collect(),
        };
        let hashes = std::cell::Cell::new(0);
        authenticate_read(&policy, &policy, |current| {
            hashes.set(hashes.get() + current.artifacts.len());
            Ok(())
        })
        .unwrap();
        assert_eq!(
            hashes.get(),
            4,
            "one isolated authentication pass, without the previous preliminary pass or GUI availability rehash"
        );
        let mut changed = policy.clone();
        changed.artifacts.insert(
            "hyper-gpu-support.exe".into(),
            "same-version different bytes".into(),
        );
        assert!(
            authenticate_read(&policy, &changed, |_| {
                hashes.set(99);
                Ok(())
            })
            .is_err()
        );
        assert_eq!(hashes.get(), 4);
        assert!(
            authenticate_read(&policy, &policy, |_| Err("fixture hash mismatch".into())).is_err()
        );
    }

    #[test]
    fn enrollment_read_stops_at_its_limit_before_parsing() {
        let mut input = std::io::repeat(b' ').take(FRAME_LIMIT as u64 + 4096);
        assert_eq!(
            parse_enrollment(&mut input).err().unwrap(),
            "invalid enrollment size"
        );
        assert_eq!(input.limit(), 4095, "do not read the whole oversized file");
        assert!(parse_enrollment(b"invalid JSON".as_slice()).is_err());
    }
    fn fixture() -> (Configuration, Discovery) {
        let target = Target {
            compute: None,
            encode: None,
            decode: None,
            vm_id: "abcdef01-2345-6789-abcd-ef0123456789".into(),
            gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#test\GPUPARAV".into(),
            enabled: true,
            vram: None,
        };
        let vm = VmState {
            compute: None,
            encode: None,
            decode: None,
            vm_id: target.vm_id.to_ascii_uppercase(),
            name: "An existing user VM".into(),
            power: Power::Off,
            generation: 2,
            gpus: vec![],
            settings: Settings {
                low_mmio: 0,
                high_mmio: 0,
                cache_types: false,
                automatic_checkpoints: true,
            },
            vram: None,
        };
        let gpu = Gpu {
            compute: Default::default(),
            encode: Default::default(),
            decode: Default::default(),
            interface: target.gpu_interface.clone(),
            name: "Discovered GPU".into(),
            vendor: 0x10de,
            device: 0x2d05,
            driver_version: "current discovered version".into(),
            physical_vram_bytes: None,
            total_vram: None,
            vram: Allocation {
                minimum: 0,
                maximum: 100,
                optimal: 50,
            }
            .into(),
            preparation_supported: true,
        };
        (
            Configuration {
                schema: 2,
                targets: vec![target],
            },
            Discovery {
                guest_os: Default::default(),
                issues: vec![],
                vms: vec![vm],
                gpus: vec![gpu],
            },
        )
    }

    #[test]
    fn enrollment_uses_existing_vm_identity_without_name_or_driver_pins() {
        let (mut configuration, mut discovery) = fixture();
        validate_enrollment(&configuration, &discovery).unwrap();
        discovery.vms[0].name = "Renamed by the user".into();
        discovery.vms[0].power = Power::Running;
        discovery.gpus[0].driver_version = "updated discovered version".into();
        let mut second = configuration.targets[0].clone();
        second.vm_id = "22222222-2222-2222-2222-222222222222".into();
        second.enabled = false;
        let mut vm = discovery.vms[0].clone();
        vm.vm_id = second.vm_id.clone();
        configuration.targets.push(second);
        discovery.vms.push(vm);
        validate_enrollment(&configuration, &discovery).unwrap();
    }

    #[test]
    fn enrollment_refuses_unsupported_generation_and_missing_identities() {
        let (configuration, mut discovery) = fixture();
        discovery.vms[0].generation = 1;
        assert!(
            validate_enrollment(&configuration, &discovery)
                .unwrap_err()
                .contains("Generation 2")
        );
        discovery.vms[0].generation = 2;
        discovery.vms[0].vm_id = "22222222-2222-2222-2222-222222222222".into();
        assert!(
            validate_enrollment(&configuration, &discovery)
                .unwrap_err()
                .contains("enrollment VM")
        );
        discovery.vms[0].vm_id = configuration.targets[0].vm_id.clone();
        discovery.gpus[0].interface.push_str("-different");
        assert!(
            validate_enrollment(&configuration, &discovery)
                .unwrap_err()
                .contains("enrollment GPU")
        );
    }

    #[test]
    fn enrollment_validates_native_callers_before_admission() {
        let (mut configuration, discovery) = fixture();
        configuration.schema = 1;
        assert!(validate_enrollment(&configuration, &discovery).is_err());
        configuration.schema = 2;
        let mut duplicate = configuration.targets[0].clone();
        duplicate.vm_id.make_ascii_uppercase();
        configuration.targets.push(duplicate);
        assert!(
            validate_enrollment(&configuration, &discovery)
                .unwrap_err()
                .contains("each VM")
        );
        configuration.targets.clear();
        assert!(validate_enrollment(&configuration, &discovery).is_err());
    }

    #[test]
    fn enrollment_does_not_claim_preparation_support_for_other_vendors() {
        let (mut configuration, mut discovery) = fixture();
        discovery.gpus[0].interface = r"\\?\PCI#VEN_1002&DEV_0001#test\GPUPARAV".into();
        discovery.gpus[0].vendor = 0x1002;
        discovery.gpus[0].preparation_supported = false;
        configuration.targets[0].gpu_interface = discovery.gpus[0].interface.clone();
        validate_enrollment(&configuration, &discovery).unwrap();
    }
}
