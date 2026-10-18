//! Shared worker test fixture; never used by production admission.
use crate::{
    model::{Power, Settings, Target, VmState},
    workflow::{self, Plan},
};

pub(super) fn ordinary_disabled_plan() -> Plan {
    use crate::workflow::Backend;
    let settings = Settings {
        low_mmio: 0,
        high_mmio: 0,
        cache_types: false,
        automatic_checkpoints: true,
    };
    let target = Target {
        vm_id: "12345678-1234-1234-1234-000000000001".into(),
        gpu_interface: "fixture-gpu".into(),
        enabled: false,
        vram: None,
        compute: None,
        encode: None,
        decode: None,
    };
    Plan {
        gpu: crate::workflow::test_support::fake().gpu(&target).unwrap(),
        reinstall: false,
        initial_setup: false,
        desired: target.clone(),
        observed: VmState {
            vm_id: target.vm_id,
            name: "Same friendly name".into(),
            generation: 2,
            power: Power::Off,
            gpus: vec![],
            settings,
            vram: None,
            compute: None,
            encode: None,
            decode: None,
        },
        managed: None,
        preparation: None,
        preview: workflow::EffectPreview {
            actions: vec![],
            settings: None,
            settings_preserved: false,
            allocation: None,
            compute: None,
            encode: None,
            decode: None,
            credentials_required: false,
            guest_downtime: false,
            restore_power: Power::Off,
            pending_recovery: false,
            summary: vec![],
        },
        allocation_units: "provider-defined".into(),
        automatic_guest_restart: false,
    }
}
