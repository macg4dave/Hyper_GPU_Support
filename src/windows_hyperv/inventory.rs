//! Batched observational relationships; effects still re-fetch exact mutable objects.
use super::*;
use windows::Win32::System::Wmi::IWbemServices;

type Rows<T> = Result<Vec<T>, String>;
struct Batch<T> {
    vms: Vec<T>,
    settings: Rows<T>,
    adapters: Rows<T>,
    links: Rows<T>,
    providers: Rows<T>,
}

fn read_batch<T>(mut query: impl FnMut(&str) -> Rows<T>) -> Result<Batch<T>, String> {
    let vms = query("SELECT * FROM Msvm_ComputerSystem WHERE Caption='Virtual Machine'")?;
    let (settings, adapters, links) = if vms.is_empty() {
        (Ok(vec![]), Ok(vec![]), Ok(vec![]))
    } else {
        (
            query(
                "SELECT * FROM Msvm_VirtualSystemSettingData WHERE VirtualSystemType='Microsoft:Hyper-V:System:Realized'",
            ),
            query("SELECT * FROM Msvm_GpuPartitionSettingData"),
            query(
                "SELECT GroupComponent,PartComponent FROM Msvm_VirtualSystemSettingDataComponent",
            ),
        )
    };
    let providers = query("SELECT * FROM Msvm_PartitionableGpu");
    Ok(Batch {
        vms,
        settings,
        adapters,
        links,
        providers,
    })
}

fn aliases(object: &Object) -> Result<[String; 2], String> {
    Ok([
        object.path()?.to_ascii_lowercase(),
        object.string("__PATH")?.to_ascii_lowercase(),
    ])
}

struct Relationships<T> {
    adapters: std::collections::BTreeMap<String, (String, T)>,
    links: std::collections::BTreeMap<String, Vec<String>>,
}
impl<T: Clone> Relationships<T> {
    fn new(adapters: Vec<([String; 2], T)>, links: Vec<(String, String)>) -> Result<Self, String> {
        let mut index = std::collections::BTreeMap::new();
        for (paths, adapter) in adapters {
            let canonical = paths[0].to_ascii_lowercase();
            for path in paths {
                if index
                    .insert(
                        path.to_ascii_lowercase(),
                        (canonical.clone(), adapter.clone()),
                    )
                    .is_some()
                {
                    return Err("Ambiguous GPU adapter object identity".into());
                }
            }
        }
        let mut grouped = std::collections::BTreeMap::<String, Vec<String>>::new();
        for (group, part) in links {
            grouped
                .entry(group.to_ascii_lowercase())
                .or_default()
                .push(part.to_ascii_lowercase());
        }
        Ok(Self {
            adapters: index,
            links: grouped,
        })
    }
    fn selected(&self, config: &[String; 2]) -> Result<Vec<T>, String> {
        let mut seen = std::collections::BTreeSet::new();
        let mut selected = Vec::new();
        for path in config {
            for part in self
                .links
                .get(&path.to_ascii_lowercase())
                .into_iter()
                .flatten()
            {
                crate::read_context::check()?;
                if let Some((canonical, adapter)) = self.adapters.get(part) {
                    if !seen.insert(canonical) {
                        return Err("Duplicate GPU adapter relationship".into());
                    }
                    selected.push(adapter.clone());
                } else if part.contains("msvm_gpupartitionsettingdata.") {
                    return Err(
                        "GPU relationship references an unavailable or nonlocal adapter".into(),
                    );
                }
            }
        }
        Ok(selected)
    }
}

fn relationships(batch: &Batch<Object>) -> Result<Relationships<Object>, String> {
    let adapters = batch
        .adapters
        .as_ref()
        .map_err(Clone::clone)?
        .iter()
        .map(|adapter| Ok((aliases(adapter)?, adapter.clone())))
        .collect::<Result<_, String>>()?;
    let links = batch
        .links
        .as_ref()
        .map_err(Clone::clone)?
        .iter()
        .map(|link| {
            Ok((
                link.string("GroupComponent")?,
                link.string("PartComponent")?,
            ))
        })
        .collect::<Result<_, String>>()?;
    Relationships::new(adapters, links)
}

fn vm_from_batch(
    vm: &Object,
    id: &str,
    batch: &Batch<Object>,
    relationships: &Relationships<Object>,
) -> Result<VmState, String> {
    let configs = batch.settings.as_ref().map_err(Clone::clone)?;
    let cfg = one(configs
        .iter()
        .filter(|c| {
            c.string("VirtualSystemIdentifier")
                .is_ok_and(|value| value.eq_ignore_ascii_case(id))
        })
        .cloned()
        .collect())?;
    let attached = relationships.selected(&aliases(&cfg)?)?;
    let providers = if attached.is_empty() {
        vec![]
    } else {
        batch
            .providers
            .as_ref()
            .map_err(Clone::clone)?
            .iter()
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
    if attached.len() > 1 {
        return Err("VM has multiple or ambiguous GPU adapter relationships".into());
    }
    Ok(VmState {
        vm_id: id.to_owned(),
        name: vm.string("ElementName").unwrap_or_else(|_| id.to_owned()),
        power: match vm.number("EnabledState")? {
            2 => Power::Running,
            3 => Power::Off,
            other => Power::Other(other),
        },
        generation: if cfg.string("VirtualSystemSubType")? == "Microsoft:Hyper-V:SubType:2" {
            2
        } else {
            1
        },
        gpus: attached
            .iter()
            .map(|a| selected_gpu_interface(&a.strings("HostResource")?, &providers))
            .collect::<Result<_, _>>()?,
        settings: Settings {
            low_mmio: cfg.number("LowMmioGapSize")?,
            high_mmio: cfg.number("HighMmioGapSize")?,
            cache_types: cfg.boolean("GuestControlledCacheTypes")?,
            automatic_checkpoints: cfg.boolean("AutomaticSnapshotsEnabled")?,
        },
        vram: observed_resource(attached.first(), Resource::Vram),
        compute: observed_resource(attached.first(), Resource::Compute),
        encode: observed_resource(attached.first(), Resource::Encode),
        decode: observed_resource(attached.first(), Resource::Decode),
    })
}

pub(super) fn discover(
    s: &Session,
    cim: &IWbemServices,
    display: bool,
) -> Result<Discovery, String> {
    let batch = read_batch(|query| s.query(query))?;
    let graph = relationships(&batch);
    let entries = batch
        .vms
        .iter()
        .map(|vm| {
            s.check_deadline()?;
            let id = vm.string("Name")?.to_ascii_lowercase();
            // A failed class read must not erase independently readable VMs. Reuse
            // the same connection for the existing per-VM fallback, with attribution.
            let result = match &graph {
                Ok(graph) if batch.settings.is_ok() => vm_from_batch(vm, &id, &batch, graph),
                _ => inspect_in(s, &id),
            };
            Ok((id, result))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let (vms, mut issues) = crate::reporting::collect_observations("VM inspection", entries);
    if let Err(error) = &graph {
        issues.push(crate::reporting::Diagnostic::observation(
            "Inventory relationship binding",
            None,
            error,
        ));
    }
    for error in [&batch.settings, &batch.adapters, &batch.links]
        .into_iter()
        .filter_map(|r| r.as_ref().err())
    {
        issues.push(crate::reporting::Diagnostic::observation(
            "Inventory relationships",
            None,
            error,
        ));
    }
    let identities = if display {
        crate::windows_probe::enumerate_adapter_identities().ok()
    } else {
        None
    };
    let gpus = match batch.providers {
        Ok(objects) => {
            let entries: Vec<_> = objects
                .into_iter()
                .map(|object| {
                    let id = object
                        .string("Name")
                        .unwrap_or_else(|_| "Unidentified GPU".into());
                    let result = read_gpus(vec![object], cim, display, identities.as_deref())
                        .and_then(|mut values| {
                            values.pop().ok_or_else(|| "GPU observation missing".into())
                        });
                    (id, result)
                })
                .collect();
            let (gpus, errors) = crate::reporting::collect_observations("GPU provider", entries);
            issues.extend(errors);
            gpus
        }
        Err(error) => {
            issues.push(crate::reporting::Diagnostic::observation(
                "GPU provider",
                None,
                &error,
            ));
            vec![]
        }
    };
    let mut guest_os = std::collections::BTreeMap::new();
    if display
        && !vms.is_empty()
        && let Ok(components) =
            s.query("SELECT SystemName,GuestIntrinsicExchangeItems FROM Msvm_KvpExchangeComponent")
    {
        let known: std::collections::BTreeSet<_> = vms.iter().map(|vm| vm.vm_id.as_str()).collect();
        let mut observed = std::collections::BTreeMap::<String, Option<String>>::new();
        for component in components {
            s.check_deadline()?;
            let Ok(id) = component.string("SystemName") else {
                continue;
            };
            let id = id.to_ascii_lowercase();
            if !known.contains(id.as_str()) {
                continue;
            }
            let name = component
                .strings("GuestIntrinsicExchangeItems")
                .ok()
                .and_then(|items| guest_os::name(&items));
            observed
                .entry(id)
                .and_modify(|value| *value = None)
                .or_insert(name);
        }
        guest_os.extend(
            observed
                .into_iter()
                .filter_map(|(id, name)| name.map(|name| (id, name))),
        );
    }
    Ok(Discovery {
        vms,
        gpus,
        guest_os,
        issues,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relationships_bind_exact_aliases_and_isolate_missing_or_duplicate_gpu_edges() {
        let paths = [
            "Msvm_GpuPartitionSettingData.InstanceID=1".to_owned(),
            "local:Msvm_GpuPartitionSettingData.InstanceID=1".to_owned(),
        ];
        let graph = Relationships::new(
            vec![(paths.clone(), 7)],
            vec![
                ("cfg-a".into(), paths[1].to_ascii_uppercase()),
                ("cfg-b".into(), "OtherResource.InstanceID=2".into()),
                (
                    "cfg-c".into(),
                    "remote:Msvm_GpuPartitionSettingData.InstanceID=1".into(),
                ),
            ],
        )
        .unwrap();
        assert_eq!(
            graph.selected(&["CFG-A".into(), "unused".into()]).unwrap(),
            vec![7]
        );
        assert!(
            graph
                .selected(&["cfg-b".into(), "unused".into()])
                .unwrap()
                .is_empty()
        );
        assert!(graph.selected(&["cfg-c".into(), "unused".into()]).is_err());
        let duplicate = Relationships::new(
            vec![(paths.clone(), 7)],
            vec![
                ("cfg-a".into(), paths[0].clone()),
                ("cfg-a".into(), paths[1].clone()),
            ],
        )
        .unwrap();
        assert!(
            duplicate
                .selected(&["cfg-a".into(), "unused".into()])
                .is_err()
        );
        assert!(Relationships::new(vec![(paths.clone(), 7), (paths, 8)], vec![]).is_err());
    }
    #[test]
    fn batch_query_count_is_independent_of_vm_count_and_retains_class_errors() {
        for vms in [0, 1, 10] {
            for fail in [None, Some(1), Some(2), Some(3), Some(4)] {
                let mut queries = 0;
                let batch = read_batch(|_| {
                    let index = queries;
                    queries += 1;
                    if fail == Some(index) {
                        Err("fixture class denied".into())
                    } else {
                        Ok(if index == 0 { vec![(); vms] } else { vec![] })
                    }
                })
                .unwrap();
                assert_eq!(queries, if vms == 0 { 2 } else { 5 });
                assert_eq!(batch.vms.len(), vms);
                if vms > 0 {
                    assert_eq!(batch.settings.is_err(), fail == Some(1));
                    assert_eq!(batch.adapters.is_err(), fail == Some(2));
                    assert_eq!(batch.links.is_err(), fail == Some(3));
                    assert_eq!(batch.providers.is_err(), fail == Some(4));
                }
            }
        }
    }
}
