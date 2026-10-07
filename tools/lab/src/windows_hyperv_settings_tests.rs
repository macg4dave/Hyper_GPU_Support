//! In-memory CIM fixtures exercise the production native settings decoder/writer.
use super::*;
use crate::windows_driver_environment::Apartment;
use crate::windows_hyperv_wmi::fixture;
use windows::Win32::System::Wmi::{CIM_BOOLEAN, CIM_STRING, CIM_UINT32, CIM_UINT64};

struct Fixture {
    settings: Object,
    memory: Object,
    cpu: Object,
    security: Object,
    adapter: Object,
}

#[test]
fn changed_native_target_refuses_missing_duplicate_or_running_adapters() {
    let _apartment = Apartment::initialize().unwrap();
    assert!(profile_adapter("Off", &[]).is_err());
    let adapters = [Fixture::new().adapter, Fixture::new().adapter];
    assert!(profile_adapter("Off", &adapters).is_err());
    assert!(profile_adapter("Running", &adapters[..1]).is_err());
    assert!(profile_adapter("Off", &adapters[..1]).is_ok());
}
impl Fixture {
    fn new() -> Self {
        let settings = fixture(&[
            ("UserSnapshotType", CIM_UINT32.0),
            ("AutomaticShutdownAction", CIM_UINT32.0),
            ("AutomaticSnapshotsEnabled", CIM_BOOLEAN.0),
            ("SecureBootTemplateId", CIM_STRING.0),
            ("SecureBootEnabled", CIM_BOOLEAN.0),
            ("LowMmioGapSize", CIM_UINT64.0),
            ("HighMmioGapSize", CIM_UINT64.0),
            ("GuestControlledCacheTypes", CIM_BOOLEAN.0),
        ]);
        for (name, value) in [("UserSnapshotType", 3), ("AutomaticShutdownAction", 3)] {
            settings.set(name, VARIANT::from(value)).unwrap();
        }
        for (name, value) in [
            ("AutomaticSnapshotsEnabled", true),
            ("SecureBootEnabled", true),
            ("GuestControlledCacheTypes", false),
        ] {
            settings.set(name, VARIANT::from(value)).unwrap();
        }
        settings
            .set(
                "SecureBootTemplateId",
                VARIANT::from(BSTR::from("1734c6e8-3154-4dda-ba5f-a874cc483422")),
            )
            .unwrap();
        settings
            .set("LowMmioGapSize", VARIANT::from(BSTR::from("1024")))
            .unwrap();
        settings
            .set("HighMmioGapSize", VARIANT::from(BSTR::from("2048")))
            .unwrap();
        let memory = fixture(&[
            ("VirtualQuantity", CIM_UINT64.0),
            ("DynamicMemoryEnabled", CIM_BOOLEAN.0),
        ]);
        memory
            .set("VirtualQuantity", VARIANT::from(BSTR::from("4096")))
            .unwrap();
        memory
            .set("DynamicMemoryEnabled", VARIANT::from(true))
            .unwrap();
        let cpu = fixture(&[
            ("VirtualQuantity", CIM_UINT64.0),
            ("ExposeVirtualizationExtensions", CIM_BOOLEAN.0),
        ]);
        cpu.set("VirtualQuantity", VARIANT::from(BSTR::from("2")))
            .unwrap();
        cpu.set("ExposeVirtualizationExtensions", VARIANT::from(false))
            .unwrap();
        let security = fixture(&[("TpmEnabled", CIM_BOOLEAN.0)]);
        security.set("TpmEnabled", VARIANT::from(true)).unwrap();
        // These are the provider schema's twelve independent uint64 properties.
        let names = [
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
        let definitions: Vec<_> = names.iter().map(|name| (*name, CIM_UINT64.0)).collect();
        let adapter = fixture(&definitions);
        for name in names {
            let value = if name.starts_with("Max") { u64::MAX } else { 0 };
            adapter
                .set(name, VARIANT::from(BSTR::from(value.to_string())))
                .unwrap();
        }
        Self {
            settings,
            memory,
            cpu,
            security,
            adapter,
        }
    }
    fn objects(&self) -> ProfileObjects<'_> {
        ProfileObjects {
            settings: &self.settings,
            memory: &self.memory,
            cpu: &self.cpu,
            security: &self.security,
            adapter: &self.adapter,
        }
    }
    fn limits() -> GpuResources {
        let range = Triple {
            minimum: 0,
            maximum: u64::MAX,
            optimal: 0,
        };
        GpuResources {
            vram: range,
            encode: range,
            decode: range,
            compute: range,
        }
    }
    fn read(&self, p: &ProjectConfiguration) -> Result<SettingsSnapshot> {
        decode_profile(p, "Off", &self.objects(), Self::limits())
    }
}

#[test]
fn actual_native_writes_round_trip_every_profile_and_gpu_field() {
    let _apartment = Apartment::initialize().unwrap();
    let p = ProjectConfiguration::embedded().unwrap();
    let f = Fixture::new();
    let before = f.read(&p).unwrap();
    assert_eq!(before.profile.memory_bytes, 4 * 1024 * 1024 * 1024);
    assert_eq!(before.profile.low_mmio_bytes, 1024 * 1024 * 1024);
    assert_eq!(before.profile.high_mmio_bytes, 2 * 1024 * 1024 * 1024);
    assert_eq!(before.profile.processors, 2);
    assert_eq!(before.checkpoint_type, "Production");
    assert_eq!(before.automatic_stop_action, "Save");
    assert!(!before.matches(&p).unwrap());
    let mut methods = Vec::new();
    apply_profile(&p, &before, &before, &f.objects(), |_, system| {
        methods.push(system);
        Ok(())
    })
    .unwrap();
    assert_eq!(methods, [true, false, false, false]);
    let after = f.read(&p).unwrap();
    after.verify(&before, &p).unwrap();
    assert_eq!(after.resources.encode.optimal, 1 << 63);
    assert_eq!(after.resources, GpuResources::desired(&p).unwrap());
    assert_eq!(after.profile, p.vm_profile);
    assert!(after.secure_boot && after.tpm_enabled);
    assert_eq!(after.secure_boot_template, "MicrosoftWindows");
}

#[test]
fn stale_native_preimage_and_security_refusal_publish_nothing() {
    let _apartment = Apartment::initialize().unwrap();
    let p = ProjectConfiguration::embedded().unwrap();
    let f = Fixture::new();
    let before = f.read(&p).unwrap();
    let mut stale = before.clone();
    stale.profile.processors += 1;
    let mut calls = 0;
    let error = apply_profile(&p, &stale, &before, &f.objects(), |_, _| {
        calls += 1;
        Ok(())
    })
    .unwrap_err();
    assert_eq!(error, "stale settings preimage rejected");
    let mut insecure = before.clone();
    insecure.tpm_enabled = false;
    assert!(
        apply_profile(&p, &insecure, &insecure, &f.objects(), |_, _| {
            calls += 1;
            Ok(())
        })
        .is_err()
    );
    assert_eq!(calls, 0);
    assert_eq!(f.read(&p).unwrap(), before);
}

#[test]
fn partial_native_method_failure_stops_before_gpu_publication() {
    let _apartment = Apartment::initialize().unwrap();
    let p = ProjectConfiguration::embedded().unwrap();
    let f = Fixture::new();
    let before = f.read(&p).unwrap();
    let mut calls = 0;
    let error = apply_profile(&p, &before, &before, &f.objects(), |_, _| {
        calls += 1;
        if calls == 3 {
            Err("injected CPU method failure".into())
        } else {
            Ok(())
        }
    })
    .unwrap_err();
    assert_eq!(error, "injected CPU method failure");
    assert_eq!(calls, 3);
    assert_eq!(resources(&f.adapter).unwrap(), before.resources);
    assert!(f.read(&p).unwrap().verify(&before, &p).is_err());
}

#[test]
fn native_decoder_rejects_overflow_unknown_enums_and_disabled_security() {
    let _apartment = Apartment::initialize().unwrap();
    let p = ProjectConfiguration::embedded().unwrap();
    for (object, name, value, diagnostic) in [
        (
            0,
            "LowMmioGapSize",
            VARIANT::from(BSTR::from(u64::MAX.to_string())),
            "byte conversion overflow",
        ),
        (
            1,
            "VirtualQuantity",
            VARIANT::from(BSTR::from(u64::MAX.to_string())),
            "CPU count overflow",
        ),
        (
            0,
            "UserSnapshotType",
            VARIANT::from(99i32),
            "unknown checkpoint policy",
        ),
        (
            0,
            "AutomaticShutdownAction",
            VARIANT::from(99i32),
            "unknown automatic stop policy",
        ),
        (
            0,
            "SecureBootEnabled",
            VARIANT::from(false),
            "Secure Boot and vTPM required",
        ),
        (
            2,
            "TpmEnabled",
            VARIANT::from(false),
            "Secure Boot and vTPM required",
        ),
    ] {
        let f = Fixture::new();
        let target = match object {
            0 => &f.settings,
            1 => &f.cpu,
            _ => &f.security,
        };
        target.set(name, value).unwrap();
        assert!(f.read(&p).unwrap_err().contains(diagnostic));
    }
}
