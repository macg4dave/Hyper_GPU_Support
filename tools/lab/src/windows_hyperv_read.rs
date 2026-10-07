//! Fixed native Hyper-V VM and host GPU reads for the enrolled runner.
//! Synchronous COM calls must execute in the bounded runner worker process.

use crate::{
    config::ProjectConfiguration,
    vm_settings::{GpuResources, Triple},
    windows_driver_environment::{Apartment, connect, query, quoted},
    windows_native_inventory::require_management_access,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Instant};

type Row = BTreeMap<String, String>;

/// Launch this runner's fixed read mode suspended into a bounded kill-on-close job.
///
/// # Errors
/// Reports launch, timeout, output overflow, failed worker or malformed output.
/// The caller must be the runner executable, which implements `read-hyperv`.
pub fn collect_bounded(timeout: std::time::Duration) -> Result<HypervRead, String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut command = std::process::Command::new(executable);
    command.arg("read-hyperv");
    let output = crate::windows_validation::bounded_process_with_limit(command, timeout, 64 * 1024)
        .map_err(|e| e.to_string())?;
    if output.exit_code != Some(0) || !output.stderr.is_empty() {
        return Err(format!(
            "native Hyper-V read failed ({:?}): {}",
            output.exit_code, output.stderr
        ));
    }
    let result: HypervRead = serde_json::from_str(&output.stdout)
        .map_err(|e| format!("invalid native Hyper-V read output: {e}"))?;
    result.validate(&ProjectConfiguration::embedded().map_err(|e| e.to_string())?)?;
    Ok(result)
}

/// Exact-target observation; opaque GPU resource units retain their full u64 range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HypervRead {
    /// Binds the fixed worker to the same compiled configuration as its parent.
    pub configuration: String,
    /// Enrolled VM GUID.
    pub vm_id: String,
    /// Enrolled VM display name.
    pub vm_name: String,
    /// Fresh VM state (only Off or Running is accepted).
    pub state: String,
    /// Generation of the realized VM configuration.
    pub generation: u32,
    /// Observed Hyper-V configuration version.
    pub version: String,
    /// Exact selected host partitionable GPU interface.
    pub gpu_interface: String,
    /// Fresh GPU capability triples, without percentage conversion.
    pub limits: GpuResources,
}

impl HypervRead {
    /// Validate a worker report before using its native observations.
    ///
    /// # Errors
    /// Rejects stale workers, wrong identities, unsafe states and invalid ranges.
    pub fn validate(&self, project: &ProjectConfiguration) -> Result<(), String> {
        if self.configuration != crate::probe::sha256_hex(include_bytes!("../../../config/project.toml"))
            || self.vm_id != project.slot.vm_id
            || self.vm_name != project.slot.vm_name
            || self.generation != 2
            || !matches!(self.state.as_str(), "Off" | "Running")
            || !self
                .gpu_interface
                .eq_ignore_ascii_case(&project.slot.gpu_interface)
            || !self
                .version
                .split_once('.')
                .is_some_and(|(a, b)| a.parse::<u32>().is_ok() && b.parse::<u32>().is_ok())
        {
            return Err("native Hyper-V worker configuration/target/state mismatch".into());
        }
        for t in [
            self.limits.vram,
            self.limits.encode,
            self.limits.decode,
            self.limits.compute,
        ] {
            if t.minimum > t.optimal || t.optimal > t.maximum {
                return Err("malformed native GPU capability range".into());
            }
        }
        Ok(())
    }
}

fn one(rows: Vec<Row>, context: &str) -> Result<Row, String> {
    if rows.len() != 1 {
        return Err(format!(
            "{context}: expected exactly one provider object, got {}",
            rows.len()
        ));
    }
    rows.into_iter()
        .next()
        .ok_or_else(|| format!("{context}: missing provider object"))
}

fn field<'a>(row: &'a Row, name: &str) -> Result<&'a str, String> {
    row.get(name)
        .filter(|v| !v.is_empty())
        .map(String::as_str)
        .ok_or_else(|| format!("missing/empty native Hyper-V property {name}"))
}

fn triple(row: &Row, resource: &str) -> Result<Triple, String> {
    let number = |prefix: &str| {
        let name = format!("{prefix}Partition{resource}");
        let value = field(row, &name)?;
        if !value.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!("malformed unsigned native GPU property {name}"));
        }
        value
            .parse::<u64>()
            .map_err(|_| format!("native GPU property {name} exceeds u64"))
    };
    Ok(Triple {
        minimum: number("Min")?,
        maximum: number("Max")?,
        optimal: number("Optimal")?,
    })
}

fn decode(
    project: &ProjectConfiguration,
    vm: Row,
    settings: Row,
    gpu: Row,
) -> Result<HypervRead, String> {
    if !field(&settings, "VirtualSystemIdentifier")?.eq_ignore_ascii_case(&project.slot.vm_id) {
        return Err("realized settings belong to another VM".into());
    }
    let result = HypervRead {
        configuration: crate::probe::sha256_hex(include_bytes!("../../../config/project.toml")),
        vm_id: field(&vm, "Name")?.to_ascii_lowercase(),
        vm_name: field(&vm, "ElementName")?.into(),
        state: match field(&vm, "EnabledState")? {
            "2" => "Running",
            "3" => "Off",
            _ => return Err("unsafe or unknown native VM state".into()),
        }
        .into(),
        generation: match field(&settings, "VirtualSystemSubType")? {
            "Microsoft:Hyper-V:SubType:2" => 2,
            _ => return Err("enrolled VM must be Generation 2".into()),
        },
        version: field(&settings, "Version")?.into(),
        gpu_interface: field(&gpu, "Name")?.into(),
        limits: GpuResources {
            vram: triple(&gpu, "VRAM")?,
            encode: triple(&gpu, "Encode")?,
            decode: triple(&gpu, "Decode")?,
            compute: triple(&gpu, "Compute")?,
        },
    };
    result.validate(project)?;
    Ok(result)
}

/// Read the exact configured VM and GPU using the existing native WMI bindings.
///
/// # Errors
/// Access denial, provider/HRESULT errors, ambiguity and malformed data fail closed.
/// This synchronous entry point is only for a deadline-supervised fixed worker.
pub fn collect(project: &ProjectConfiguration) -> Result<HypervRead, String> {
    require_management_access()?;
    let _apartment = Apartment::initialize().map_err(|e| e.to_string())?;
    let services = connect(r"ROOT\virtualization\v2").map_err(|e| e.to_string())?;
    let deadline = Instant::now() + project.runner.inspect_timeout;
    let read = |wql: &str, properties: &[&str]| {
        one(
            query(&services, wql, properties, deadline).map_err(|e| e.to_string())?,
            wql,
        )
    };
    let vm = read(
        &format!(
            "SELECT Name,ElementName,EnabledState FROM Msvm_ComputerSystem WHERE Caption='Virtual Machine' AND Name='{}'",
            quoted(&project.slot.vm_id)
        ),
        &["Name", "ElementName", "EnabledState"],
    )?;
    let settings = read(
        &format!(
            "SELECT VirtualSystemIdentifier,VirtualSystemSubType,Version FROM Msvm_VirtualSystemSettingData WHERE VirtualSystemType='Microsoft:Hyper-V:System:Realized' AND VirtualSystemIdentifier='{}'",
            quoted(&project.slot.vm_id)
        ),
        &["VirtualSystemIdentifier", "VirtualSystemSubType", "Version"],
    )?;
    let names = [
        "Name",
        "MinPartitionVRAM",
        "MaxPartitionVRAM",
        "OptimalPartitionVRAM",
        "MinPartitionEncode",
        "MaxPartitionEncode",
        "OptimalPartitionEncode",
        "MinPartitionDecode",
        "MaxPartitionDecode",
        "OptimalPartitionDecode",
        "MinPartitionCompute",
        "MaxPartitionCompute",
        "OptimalPartitionCompute",
    ];
    let gpu = read(
        &format!(
            "SELECT {} FROM Msvm_PartitionableGpu WHERE Name='{}'",
            names.join(","),
            quoted(&project.slot.gpu_interface)
        ),
        &names,
    )?;
    decode(project, vm, settings, gpu)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (ProjectConfiguration, Row, Row, Row) {
        let p = ProjectConfiguration::embedded().unwrap();
        let vm = [
            ("Name", p.slot.vm_id.as_str()),
            ("ElementName", p.slot.vm_name.as_str()),
            ("EnabledState", "3"),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();
        let settings = [
            ("VirtualSystemIdentifier", p.slot.vm_id.as_str()),
            ("VirtualSystemSubType", "Microsoft:Hyper-V:SubType:2"),
            ("Version", "12.0"),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();
        let mut gpu = Row::from([("Name".into(), p.slot.gpu_interface.clone())]);
        for resource in ["VRAM", "Encode", "Decode", "Compute"] {
            for (prefix, value) in [
                ("Min", "0"),
                ("Max", "18446744073709551615"),
                ("Optimal", "9223372036854775808"),
            ] {
                gpu.insert(format!("{prefix}Partition{resource}"), value.into());
            }
        }
        (p, vm, settings, gpu)
    }
    #[test]
    fn exact_native_observation_preserves_unsigned_limits() {
        let (p, vm, settings, gpu) = fixture();
        let result = decode(&p, vm, settings, gpu).unwrap();
        assert_eq!(result.limits.encode.maximum, u64::MAX);
        assert_eq!(result.limits.encode.optimal, 1 << 63);
        assert_eq!(result.state, "Off");
        let mut stale = result.clone();
        stale.configuration = "0".repeat(64);
        assert!(stale.validate(&p).is_err());
        assert!(
            serde_json::from_str::<HypervRead>(
                &serde_json::to_string(&result)
                    .unwrap()
                    .replace("\"generation\":2", "\"generation\":2,\"extra\":1")
            )
            .is_err()
        );
    }
    #[test]
    fn wrong_missing_ambiguous_and_unsafe_provider_values_fail() {
        let (p, vm, settings, gpu) = fixture();
        assert!(one(vec![], "VM").is_err());
        assert!(one(vec![vm.clone(), vm.clone()], "VM").is_err());
        for (which, name, value) in [
            (0, "Name", "wrong"),
            (0, "ElementName", "wrong"),
            (0, "EnabledState", "32769"),
            (1, "VirtualSystemIdentifier", "wrong"),
            (1, "VirtualSystemSubType", "Microsoft:Hyper-V:SubType:1"),
            (1, "Version", "unknown"),
            (2, "Name", "wrong"),
            (2, "MinPartitionEncode", "18446744073709551616"),
            (2, "OptimalPartitionEncode", "-1"),
            (2, "MaxPartitionEncode", "1"),
        ] {
            let mut rows = [vm.clone(), settings.clone(), gpu.clone()];
            rows[which].insert(name.into(), value.into());
            assert!(
                decode(&p, rows[0].clone(), rows[1].clone(), rows[2].clone()).is_err(),
                "{name}={value}"
            );
        }
        let mut absent = gpu.clone();
        absent.remove("MinPartitionVRAM");
        assert!(decode(&p, vm, settings, absent).is_err());
    }
}
