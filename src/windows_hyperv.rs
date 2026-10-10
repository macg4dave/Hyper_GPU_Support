//! Existing-VM native Hyper-V operations. No golden disk, VM name or host memory pins.
use crate::{
    model::*,
    payload::physical_device_id,
    windows_com::{Apartment, connect, query, quoted},
    windows_wmi::{Object, Session, one, string_array},
};
use std::time::{Duration, Instant};
use windows::{Win32::System::Variant::VARIANT, core::BSTR};
const TIMEOUT: Duration = Duration::from_secs(120);
fn settings(s: &Session, id: &str) -> Result<Object, String> {
    s.one(&format!("SELECT * FROM Msvm_VirtualSystemSettingData WHERE VirtualSystemType='Microsoft:Hyper-V:System:Realized' AND VirtualSystemIdentifier='{}'",quoted(id)))
}
/// Read one VM by stable GUID with fresh provider objects.
pub fn inspect(id: &str) -> Result<VmState, String> {
    let s = Session::new(TIMEOUT)?;
    let vm = s.one(&format!(
        "SELECT * FROM Msvm_ComputerSystem WHERE Caption='Virtual Machine' AND Name='{}'",
        quoted(id)
    ))?;
    let cfg = settings(&s, id)?;
    let gpus = s.related(&cfg, "Msvm_GpuPartitionSettingData")?;
    let providers = if gpus.is_empty() {
        Vec::new()
    } else {
        s.query("SELECT * FROM Msvm_PartitionableGpu")?
            .into_iter()
            .map(|gpu| {
                let interface = gpu.string("Name")?;
                Ok(vec![
                    (gpu.path()?, interface.clone()),
                    (gpu.string("__PATH")?, interface),
                ])
            })
            .collect::<Result<Vec<_>, String>>()?
            .into_iter()
            .flatten()
            .collect()
    };
    let vram = gpus.first().map(read_optional_vram).transpose()?.flatten();
    Ok(VmState {
        vm_id: vm.string("Name")?.to_ascii_lowercase(),
        name: vm.string("ElementName")?,
        power: match vm.number("EnabledState")? {
            2 => Power::Running,
            3 => Power::Off,
            n => Power::Other(n),
        },
        generation: if cfg.string("VirtualSystemSubType")? == "Microsoft:Hyper-V:SubType:2" {
            2
        } else {
            1
        },
        gpus: gpus
            .iter()
            .map(|g| selected_gpu_interface(&g.strings("HostResource")?, &providers))
            .collect::<Result<Vec<_>, _>>()?,
        vram,
        settings: Settings {
            low_mmio: cfg.number("LowMmioGapSize")?,
            high_mmio: cfg.number("HighMmioGapSize")?,
            cache_types: cfg.boolean("GuestControlledCacheTypes")?,
            automatic_checkpoints: cfg.boolean("AutomaticSnapshotsEnabled")?,
        },
    })
}

fn selected_gpu_interface(
    resources: &[String],
    providers: &[(String, String)],
) -> Result<String, String> {
    let [resource] = resources else {
        return Err("GPU adapter must reference exactly one host resource".into());
    };
    let mut matches = providers
        .iter()
        .filter(|(path, _)| path.eq_ignore_ascii_case(resource));
    let (_, interface) = matches
        .next()
        .ok_or("GPU host resource is not a discovered local GPU")?;
    if matches.next().is_some() {
        return Err("GPU host resource identity is ambiguous".into());
    }
    Ok(interface.clone())
}
fn read_vram(o: &Object) -> Result<Allocation, String> {
    Ok(Allocation {
        minimum: o.number("MinPartitionVRAM")?,
        maximum: o.number("MaxPartitionVRAM")?,
        optimal: o.number("OptimalPartitionVRAM")?,
    })
}
fn read_optional_vram(o: &Object) -> Result<Option<Allocation>, String> {
    match (
        o.optional_number("MinPartitionVRAM")?,
        o.optional_number("MaxPartitionVRAM")?,
        o.optional_number("OptimalPartitionVRAM")?,
    ) {
        (None, None, None) => Ok(None),
        (Some(minimum), Some(maximum), Some(optimal)) => Ok(Some(Allocation {
            minimum,
            maximum,
            optimal,
        })),
        _ => Err("GPU allocation readback is partially specified".into()),
    }
}
/// Enumerate all partitionable GPUs, without claiming preparation support for unimplemented vendors.
pub fn discover() -> Result<Discovery, String> {
    let s = Session::new(TIMEOUT)?;
    let entries = s
        .query("SELECT * FROM Msvm_ComputerSystem WHERE Caption='Virtual Machine'")?
        .into_iter()
        .map(|v| {
            let id = v.string("Name")?.to_ascii_lowercase();
            let result = inspect(&id);
            Ok((id, result))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let (vms, mut issues) = crate::reporting::collect_observations("VM inspection", entries);
    let gpus = match discover_gpus(&s, "SELECT * FROM Msvm_PartitionableGpu") {
        Ok(gpus) => gpus,
        Err(error) => {
            issues.push(crate::reporting::Diagnostic::observation(
                "GPU provider",
                None,
                &error,
            ));
            Vec::new()
        }
    };
    Ok(Discovery { issues, vms, gpus })
}
pub(crate) fn selected_gpu(t: &Target) -> Result<Gpu, String> {
    t.validate()?;
    let s = Session::new(TIMEOUT)?;
    let mut gpus = discover_gpus(
        &s,
        &format!(
            "SELECT * FROM Msvm_PartitionableGpu WHERE Name='{}'",
            quoted(&t.gpu_interface)
        ),
    )?;
    if gpus.len() != 1 {
        return Err("selected GPU is absent or ambiguous".into());
    }
    gpus.pop().ok_or("selected GPU disappeared".into())
}
fn discover_gpus(s: &Session, gpu_query: &str) -> Result<Vec<Gpu>, String> {
    let _apartment = Apartment::initialize().map_err(|e| e.to_string())?;
    let cim = connect(r"ROOT\cimv2").map_err(|e| e.to_string())?;
    let mut gpus = Vec::new();
    for o in s.query(gpu_query)? {
        let interface = o.string("Name")?;
        let physical = physical_device_id(&interface)?;
        let rows = query(
            &cim,
            &format!(
                "SELECT DeviceName,DriverVersion FROM Win32_PNPSignedDriver WHERE DeviceID='{}'",
                quoted(&physical)
            ),
            &["DeviceName", "DriverVersion"],
            Instant::now() + TIMEOUT,
        )
        .map_err(|e| e.to_string())?;
        if rows.len() != 1 {
            return Err("partitionable GPU signed-driver identity is ambiguous".into());
        }
        let row = &rows[0];
        let pci = |tag: &str| -> Result<u32, String> {
            let (_, rest) = physical.split_once(tag).ok_or("PCI identity missing")?;
            u32::from_str_radix(rest.get(..4).ok_or("PCI identity truncated")?, 16)
                .map_err(|e| e.to_string())
        };
        let vendor = pci("VEN_")?;
        gpus.push(Gpu {
            interface,
            name: row.get("DeviceName").ok_or("missing GPU name")?.clone(),
            driver_version: row
                .get("DriverVersion")
                .ok_or("missing driver version")?
                .clone(),
            vendor,
            device: pci("DEV_")?,
            vram: read_vram(&o)?,
            preparation_supported: vendor == 0x10de,
        });
    }
    Ok(gpus)
}
fn require_off(t: &Target) -> Result<(), String> {
    t.validate()?;
    let v = inspect(&t.vm_id)?;
    if v.power != Power::Off
        || v.generation != 2
        || v.gpus.len() > 1
        || v.gpus.iter().any(|g| g != &t.gpu_interface)
    {
        return Err("VM must be Off with no conflicting GPU assignment".into());
    }
    Ok(())
}
/// Apply/remove only the exact selected GPU and independently verify the result.
pub fn assign(t: &Target, enabled: bool) -> Result<(), String> {
    require_off(t)?;
    let s = Session::new(TIMEOUT)?;
    let cfg = settings(&s, &t.vm_id)?;
    let service = s.one("SELECT * FROM Msvm_VirtualSystemManagementService")?;
    let adapters = s.related(&cfg, "Msvm_GpuPartitionSettingData")?;
    if enabled && adapters.is_empty() {
        let template = one(s
            .query("SELECT * FROM Msvm_GpuPartitionSettingData")?
            .into_iter()
            .filter_map(|o| match o.string("InstanceID") {
                Ok(id) if id.starts_with("Microsoft:Definition\\") && id.ends_with("\\Default") => {
                    Some(Ok(o))
                }
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<Vec<_>, String>>()?)?;
        // Keep the provider's default definition identity and allocation values.
        // AddResourceSettings creates the VM-specific resource identity.
        let gpu = s.one(&format!(
            "SELECT * FROM Msvm_PartitionableGpu WHERE Name='{}'",
            quoted(&t.gpu_interface)
        ))?;
        template.set_strings("HostResource", &[gpu.string("__PATH")?])?;
        s.invoke(
            &service,
            "AddResourceSettings",
            vec![
                (
                    "AffectedConfiguration",
                    VARIANT::from(BSTR::from(cfg.path()?)).into(),
                ),
                ("ResourceSettings", string_array(&[template.xml()?])?),
            ],
        )?;
    } else if !enabled && !adapters.is_empty() {
        s.invoke(
            &service,
            "RemoveResourceSettings",
            vec![("ResourceSettings", string_array(&[adapters[0].path()?])?)],
        )?;
    }
    let expected = if enabled {
        vec![t.gpu_interface.clone()]
    } else {
        vec![]
    };
    if inspect(&t.vm_id)?.gpus != expected {
        return Err("GPU assignment readback failed".into());
    }
    Ok(())
}
/// Apply compatibility settings without changing CPU/RAM amounts or security devices.
pub fn configure(t: &Target, value: &Settings) -> Result<(), String> {
    require_off(t)?;
    if inspect(&t.vm_id)?.settings == *value {
        return Ok(());
    }
    let s = Session::new(TIMEOUT)?;
    let cfg = settings(&s, &t.vm_id)?;
    let service = s.one("SELECT * FROM Msvm_VirtualSystemManagementService")?;
    cfg.set_number("LowMmioGapSize", value.low_mmio)?;
    cfg.set_number("HighMmioGapSize", value.high_mmio)?;
    cfg.set(
        "GuestControlledCacheTypes",
        VARIANT::from(value.cache_types),
    )?;
    cfg.set(
        "AutomaticSnapshotsEnabled",
        VARIANT::from(value.automatic_checkpoints),
    )?;
    s.modify(&service, &cfg, true)?;
    if inspect(&t.vm_id)?.settings != *value {
        return Err("VM settings readback failed".into());
    }
    Ok(())
}
/// Apply provider-defined VRAM values, with actual range validation and readback.
pub fn allocation(t: &Target, value: &Allocation) -> Result<(), String> {
    require_off(t)?;
    let s = Session::new(TIMEOUT)?;
    let gpu = s.one(&format!(
        "SELECT * FROM Msvm_PartitionableGpu WHERE Name='{}'",
        quoted(&t.gpu_interface)
    ))?;
    value.validate(&read_vram(&gpu)?)?;
    let cfg = settings(&s, &t.vm_id)?;
    let adapter = one(s.related(&cfg, "Msvm_GpuPartitionSettingData")?)?;
    for (name, n) in [
        ("MinPartitionVRAM", value.minimum),
        ("MaxPartitionVRAM", value.maximum),
        ("OptimalPartitionVRAM", value.optimal),
    ] {
        adapter.set_number(name, n)?;
    }
    s.modify(
        &s.one("SELECT * FROM Msvm_VirtualSystemManagementService")?,
        &adapter,
        false,
    )?;
    if inspect(&t.vm_id)?.vram.as_ref() != Some(value) {
        return Err("VRAM allocation readback failed".into());
    }
    Ok(())
}
/// Gracefully start/stop a guest; never force stop or perform host lifecycle actions.
pub fn power(t: &Target, desired: Power) -> Result<(), String> {
    t.validate()?;
    let current = inspect(&t.vm_id)?.power;
    if current == desired {
        return Ok(());
    }
    if !matches!(current, Power::Off | Power::Running)
        || !matches!(desired, Power::Off | Power::Running)
    {
        return Err("transitional/saved VM state refused".into());
    }
    let s = Session::new(TIMEOUT)?;
    let vm = s.one(&format!(
        "SELECT * FROM Msvm_ComputerSystem WHERE Caption='Virtual Machine' AND Name='{}'",
        quoted(&t.vm_id)
    ))?;
    if desired == Power::Running {
        s.invoke(
            &vm,
            "RequestStateChange",
            vec![("RequestedState", VARIANT::from(2i32).into())],
        )?;
    } else {
        let shutdown = one(s.related(&vm, "Msvm_ShutdownComponent")?)?;
        s.invoke(
            &shutdown,
            "InitiateShutdown",
            vec![
                ("Force", VARIANT::from(false).into()),
                (
                    "Reason",
                    VARIANT::from(BSTR::from("Hyper GPU Support configuration")).into(),
                ),
            ],
        )?;
    }
    let deadline = Instant::now() + TIMEOUT;
    while read_power(&s, &t.vm_id)? != desired {
        if Instant::now() >= deadline {
            return Err("guest lifecycle timeout; reconcile before retry".into());
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    // Preserve independent full readback after the lightweight transition poll.
    if inspect(&t.vm_id)?.power != desired {
        return Err("guest power state changed during final verification".into());
    }
    Ok(())
}

fn read_power(s: &Session, id: &str) -> Result<Power, String> {
    let vm = s.one(&format!(
        "SELECT EnabledState FROM Msvm_ComputerSystem WHERE Caption='Virtual Machine' AND Name='{}'",
        quoted(id)
    ))?;
    Ok(match vm.number("EnabledState")? {
        2 => Power::Running,
        3 => Power::Off,
        n => Power::Other(n),
    })
}

#[cfg(test)]
mod tests {
    use super::selected_gpu_interface;

    #[test]
    fn host_resources_resolve_only_to_discovered_local_gpu_identities() {
        let path = r#"\\host\root\virtualization\v2:Msvm_PartitionableGpu.Name="gpu""#;
        let providers = vec![(path.into(), "selected-interface".into())];
        assert_eq!(
            selected_gpu_interface(&[path.to_ascii_uppercase()], &providers).unwrap(),
            "selected-interface"
        );
        for resources in [
            vec![],
            vec!["selected-interface".into()],
            vec![path.replace("host", "remote")],
            vec![path.into(), path.into()],
        ] {
            assert!(selected_gpu_interface(&resources, &providers).is_err());
        }
        let ambiguous = vec![providers[0].clone(), providers[0].clone()];
        assert!(selected_gpu_interface(&[path.into()], &ambiguous).is_err());
        let relative = r#"Msvm_PartitionableGpu.Name="gpu""#;
        let local_forms = vec![
            providers[0].clone(),
            (relative.into(), "selected-interface".into()),
        ];
        for reference in [path, relative] {
            assert_eq!(
                selected_gpu_interface(&[reference.into()], &local_forms).unwrap(),
                "selected-interface"
            );
        }
    }
}
