//! Existing-VM native Hyper-V operations. No golden disk, VM name or host memory pins.
use crate::{
    model::*,
    payload::physical_device_id,
    windows_com::quoted,
    windows_wmi::{
        Object, Session, one,
        recovery::{Effect, Scope},
        string_array,
    },
};
use std::time::{Duration, Instant};
use windows::{Win32::System::Variant::VARIANT, core::BSTR};
mod guest_os;
mod inventory;
mod reads;
pub(crate) use reads::ReadSession;

const TIMEOUT: Duration = Duration::from_secs(120);
fn settings(s: &Session, id: &str) -> Result<Object, String> {
    s.one(&format!("SELECT * FROM Msvm_VirtualSystemSettingData WHERE VirtualSystemType='Microsoft:Hyper-V:System:Realized' AND VirtualSystemIdentifier='{}'",quoted(id)))
}
/// Read one VM by stable GUID with fresh provider objects.
pub fn inspect(id: &str) -> Result<VmState, String> {
    ReadSession::new()?.inspect(id)
}

fn inspect_in(s: &Session, id: &str) -> Result<VmState, String> {
    let vm = s.one(&format!(
        "SELECT * FROM Msvm_ComputerSystem WHERE Caption='Virtual Machine' AND Name='{}'",
        quoted(id)
    ))?;
    let cfg = settings(s, id)?;
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
    let vram = observed_resource(gpus.first(), Resource::Vram);
    Ok(VmState {
        compute: observed_resource(gpus.first(), Resource::Compute),
        encode: observed_resource(gpus.first(), Resource::Encode),
        decode: observed_resource(gpus.first(), Resource::Decode),
        vm_id: vm.string("Name")?.to_ascii_lowercase(),
        name: vm.string("ElementName").unwrap_or_else(|_| id.to_owned()),
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
fn require_selected_adapter(
    resources: &[String],
    providers: &[(String, String)],
    expected: &str,
) -> Result<(), String> {
    if selected_gpu_interface(resources, providers)? != expected {
        return Err("GPU adapter changed before the GPU operation; review again".into());
    }
    Ok(())
}

fn read_optional_resource(o: &Object, resource: Resource) -> Result<Option<Allocation>, String> {
    let name = resource.name();
    match (
        o.optional_number(&format!("MinPartition{name}"))?,
        o.optional_number(&format!("MaxPartition{name}"))?,
        o.optional_number(&format!("OptimalPartition{name}"))?,
    ) {
        (None, None, None) => Ok(None),
        (Some(minimum), Some(maximum), Some(optimal)) => {
            let value = Allocation {
                minimum,
                maximum,
                optimal,
            };
            value.validate(&value)?;
            Ok(Some(value))
        }
        _ => Err(format!("{name} allocation readback is partially specified")),
    }
}
fn observed_resource(o: Option<&Object>, resource: Resource) -> Option<Allocation> {
    // Missing/malformed observations stay unknown. Requested writes require a
    // successful independent read below; this never manufactures zero/defaults.
    o.and_then(|o| read_optional_resource(o, resource).ok().flatten())
}
fn resource_capability(o: &Object, resource: Resource) -> ResourceCapability {
    match read_optional_resource(o, resource) {
        Ok(Some(limits)) => ResourceCapability::Supported(limits),
        Ok(None) => {
            ResourceCapability::Unavailable("Provider did not report a complete range".into())
        }
        Err(_) => {
            ResourceCapability::Unavailable("Provider range could not be read or validated".into())
        }
    }
}
/// Enumerate all partitionable GPUs, without claiming preparation support for unimplemented vendors.
pub fn discover() -> Result<Discovery, String> {
    ReadSession::new()?.discover(true)
}
fn read_gpus(
    objects: Vec<Object>,
    cim: &windows::Win32::System::Wmi::IWbemServices,
    display: bool,
    identities: Option<&[crate::probe::AdapterIdentity]>,
) -> Result<Vec<Gpu>, String> {
    let mut gpus = Vec::new();
    for o in objects {
        let interface = o.string("Name")?;
        let physical = physical_device_id(&interface)?;
        let rows = crate::windows_com::query_with_optional(
            cim,
            &format!(
                "SELECT DriverVersion FROM Win32_PNPSignedDriver WHERE DeviceID='{}'",
                quoted(&physical)
            ),
            &["DriverVersion"],
            &[],
            Instant::now() + TIMEOUT,
        )
        .map_err(|e| e.to_string())?;
        if rows.len() != 1 {
            return Err("partitionable GPU signed-driver identity is ambiguous".into());
        }
        let row = &rows[0];
        let pci = |tag: &str, digits: usize| -> Result<u32, String> {
            let (_, rest) = physical.split_once(tag).ok_or("PCI identity missing")?;
            u32::from_str_radix(rest.get(..digits).ok_or("PCI identity truncated")?, 16)
                .map_err(|e| e.to_string())
        };
        let vendor = pci("VEN_", 4)?;
        let device = pci("DEV_", 4)?;
        let physical_vram_bytes = identities.and_then(|identities| {
            crate::windows_probe::physical_memory_for_pci(
                identities,
                vendor,
                device,
                pci("SUBSYS_", 8).ok()?,
                pci("REV_", 2).ok()?,
            )
        });
        let display_name = if display {
            crate::windows_com::query_with_optional(
                cim,
                &format!(
                    "SELECT DeviceName FROM Win32_PNPSignedDriver WHERE DeviceID='{}'",
                    quoted(&physical)
                ),
                &[],
                &["DeviceName"],
                Instant::now() + TIMEOUT,
            )
            .ok()
            .filter(|rows| rows.len() == 1)
            .and_then(|rows| rows[0].get("DeviceName").cloned())
        } else {
            None
        };
        gpus.push(Gpu {
            compute: resource_capability(&o, Resource::Compute),
            encode: resource_capability(&o, Resource::Encode),
            decode: resource_capability(&o, Resource::Decode),
            interface: interface.clone(),
            name: display_name.unwrap_or_else(|| interface.clone()),
            driver_version: row
                .get("DriverVersion")
                .filter(|version| !version.trim().is_empty())
                .ok_or("missing driver version")?
                .clone(),
            vendor,
            device,
            physical_vram_bytes,
            total_vram: if display {
                o.optional_number("TotalVRAM").ok().flatten()
            } else {
                None
            },
            vram: resource_capability(&o, Resource::Vram),
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
            Scope::new(t, Effect::Assignment(true))?,
        )?;
    } else if !enabled && !adapters.is_empty() {
        let adapter = one(adapters)?;
        let gpu = s.one(&format!(
            "SELECT * FROM Msvm_PartitionableGpu WHERE Name='{}'",
            quoted(&t.gpu_interface)
        ))?;
        require_selected_adapter(
            &adapter.strings("HostResource")?,
            &[
                (gpu.path()?, t.gpu_interface.clone()),
                (gpu.string("__PATH")?, t.gpu_interface.clone()),
            ],
            &t.gpu_interface,
        )?;
        s.invoke(
            &service,
            "RemoveResourceSettings",
            vec![("ResourceSettings", string_array(&[adapter.path()?])?)],
            Scope::new(t, Effect::Assignment(false))?,
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
    s.modify(
        &service,
        &cfg,
        true,
        Scope::new(t, Effect::Settings(value.clone()))?,
    )?;
    if inspect(&t.vm_id)?.settings != *value {
        return Err("VM settings readback failed".into());
    }
    Ok(())
}
/// Apply one resource with fresh category-specific admission and exact readback.
pub fn allocation(t: &Target, resource: Resource, value: &Allocation) -> Result<(), String> {
    require_off(t)?;
    let s = Session::new(TIMEOUT)?;
    let gpu = s.one(&format!(
        "SELECT * FROM Msvm_PartitionableGpu WHERE Name='{}'",
        quoted(&t.gpu_interface)
    ))?;
    let limits = read_optional_resource(&gpu, resource)?
        .ok_or_else(|| format!("{} capability unavailable", resource.name()))?;
    value.validate_request(resource, &limits)?;
    let cfg = settings(&s, &t.vm_id)?;
    let adapter = one(s.related(&cfg, "Msvm_GpuPartitionSettingData")?)?;
    // Bind the actual adapter being modified, not only the earlier VM inspection.
    require_selected_adapter(
        &adapter.strings("HostResource")?,
        &[
            (gpu.path()?, t.gpu_interface.clone()),
            (gpu.string("__PATH")?, t.gpu_interface.clone()),
        ],
        &t.gpu_interface,
    )?;
    let name = resource.name();
    for (property, n) in [
        (format!("MinPartition{name}"), value.minimum),
        (format!("MaxPartition{name}"), value.maximum),
        (format!("OptimalPartition{name}"), value.optimal),
    ] {
        adapter.set_number(&property, n)?;
    }
    s.modify(
        &s.one("SELECT * FROM Msvm_VirtualSystemManagementService")?,
        &adapter,
        false,
        Scope::new(t, Effect::Allocation(resource, value.clone()))?,
    )?;
    if inspect(&t.vm_id)?.allocation(resource) != Some(value) {
        return Err(format!("{name} allocation readback failed"));
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
            Scope::new(t, Effect::Power(desired.clone()))?,
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
            Scope::new(t, Effect::Power(desired.clone()))?,
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
    use super::{require_selected_adapter, selected_gpu_interface};

    #[test]
    fn allocation_and_detach_adapter_binding_rejects_replacement_and_missing_resources() {
        let providers = vec![
            ("relative-path".into(), "selected-interface".into()),
            ("absolute-path".into(), "selected-interface".into()),
            ("replacement-path".into(), "other-interface".into()),
        ];
        for path in ["relative-path", "absolute-path"] {
            assert!(
                require_selected_adapter(&[path.into()], &providers, "selected-interface").is_ok()
            );
        }
        assert!(
            require_selected_adapter(
                &["replacement-path".into()],
                &providers,
                "selected-interface"
            )
            .unwrap_err()
            .contains("changed before the GPU operation")
        );
        assert!(require_selected_adapter(&[], &providers, "selected-interface").is_err());
        assert!(
            require_selected_adapter(
                &["relative-path".into(), "replacement-path".into()],
                &providers,
                "selected-interface"
            )
            .is_err()
        );
        assert!(
            require_selected_adapter(&["unknown-path".into()], &providers, "selected-interface")
                .is_err()
        );
    }

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
