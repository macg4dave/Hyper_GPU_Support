//! Shared in-memory fixtures for live-state regression tests.
use super::*;
use hyper_gpu_support::{
    gui_model::Inventory,
    model::{Allocation, Discovery, Gpu, Power, Settings, Target, VmState},
};

mod actions;
mod completion;
mod configuration;
mod draft;
mod host_configuration;
mod inventory;
mod profile;
mod recovery;
mod review;
mod review_text;

fn inventory() -> Inventory {
    let target = Target {
        compute: None,
        encode: None,
        decode: None,
        vm_id: "12345678-1234-1234-1234-123456789abc".into(),
        gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#fixture\GPUPARAV".into(),
        enabled: true,
        vram: None,
    };
    Inventory {
        discovery: Discovery {
            guest_os: Default::default(),
            issues: vec![],
            vms: vec![VmState {
                compute: None,
                encode: None,
                decode: None,
                vm_id: target.vm_id.clone(),
                name: "Renamable VM".into(),
                power: Power::Off,
                generation: 2,
                gpus: vec![],
                vram: None,
                settings: Settings {
                    low_mmio: 0,
                    high_mmio: 0,
                    cache_types: false,
                    automatic_checkpoints: true,
                },
            }],
            gpus: vec![Gpu {
                compute: Default::default(),
                encode: Default::default(),
                decode: Default::default(),
                interface: target.gpu_interface.clone(),
                name: "Identical GPU name".into(),
                vendor: 0x10de,
                device: 1,
                driver_version: "fixture".into(),
                physical_vram_bytes: None,
                total_vram: None,
                preparation_supported: true,
                vram: Allocation {
                    minimum: 10,
                    optimal: 50,
                    maximum: 100,
                }
                .into(),
            }],
        },
        enrolled: vec![target],
        managed: Default::default(),
    }
}

fn state() -> State {
    let mut s = State::default();
    s.refresh(inventory());
    s
}

fn action_state() -> State {
    let mut s = state();
    let vm_id = id(&s).to_owned();
    let gpu = s.target(&vm_id).unwrap().gpu_interface;
    let inventory = s.view.inventory.as_mut().unwrap();
    inventory.managed.insert(vm_id, None);
    inventory.discovery.vms[0].gpus = vec![gpu];
    s
}

fn id(s: &State) -> &str {
    s.view.selected.as_deref().unwrap()
}

fn values(a: &str, b: &str, c: &str) -> Vec<String> {
    let mut v = vec![String::new(); 12];
    v[0] = a.into();
    v[1] = b.into();
    v[2] = c.into();
    v
}

fn committed(
    targets: Vec<Target>,
    comment: &str,
) -> hyper_gpu_support::configuration_store::StoreSnapshot {
    let configuration = Configuration { schema: 2, targets };
    let mut store = hyper_gpu_support::configuration_store::StoreSnapshot::default();
    for (filename, source) in configuration.vm_documents().unwrap() {
        store.insert(&filename, Ok(format!("{source}\n# {comment}\n")));
    }
    store
}

fn save_record(s: &State) -> hyper_gpu_support::configuration_store::SaveRecord {
    use hyper_gpu_support::configuration_store::{Revision, SavePhase, SaveRecord};
    let observed = s.view.vm(id(s)).unwrap().clone();
    SaveRecord {
        schema: 1,
        retirement: None,
        reset: None,
        operation_id: "test-operation".into(),
        target: s.target(id(s)).unwrap(),
        expected: Revision::Missing,
        driver_version: "test-driver".into(),
        payload_digest: None,
        plan_revision: Revision::Missing,
        intended_source: String::new(),
        publication_required: true,
        initial: observed.clone(),
        expected_power: None,
        graphics_required: None,
        initial_journal: None,
        expected_settings: observed.settings.clone(),
        phase: SavePhase::Verified(observed),
    }
}

fn review_plan() -> Plan {
    use hyper_gpu_support::workflow::EffectPreview;
    let s = state();
    let target = s
        .draft(id(&s), true, 0, &values("10", "50", "100"))
        .unwrap();
    Plan {
        authentication_deferred: false,
        reinstall: false,
        initial_setup: false,
        desired: target.clone(),
        observed: s.view.vm(id(&s)).unwrap().clone(),
        managed: None,
        preparation: None,
        preview: EffectPreview {
            compute: None,
            encode: None,
            decode: None,
            actions: vec![],
            settings: None,
            settings_preserved: false,
            allocation: target.vram,
            credentials_required: true,
            guest_downtime: false,
            restore_power: Power::Off,
            pending_recovery: false,
            summary: vec!["Apply requested raw VRAM values".into()],
        },
        allocation_units: "provider-defined".into(),
        automatic_guest_restart: false,
    }
}
