//! Shared isolated backend and fixtures, compiled only for tests.
use super::{Backend, EffectPreview, Journal, PreviewAction};
use crate::model::{Allocation, Gpu, Power, Resource, Settings, Target, VmState};

pub(super) struct Fake {
    pub(super) os_preflights: usize,
    pub(super) os_preflight_failure: Option<String>,
    pub(super) os_checks: usize,
    pub(super) os_failure: Option<String>,
    pub(super) drift_at: Option<(&'static str, Drift)>,
    pub(super) inspect_unavailable: bool,
    pub(super) state: VmState,
    pub(super) journal: Option<Journal>,
    pub(super) events: Vec<&'static str>,
    pub(super) fail_prepare: bool,
    pub(super) fail_settings: bool,
    pub(super) settings_writes: usize,
    pub(super) fail_verify: bool,
    pub(super) stop_on_verify: bool,
    pub(super) change_settings_on_detach: bool,
    pub(super) payload_check_failure: Option<crate::reporting::FailureCause>,
    pub(super) ignore_shutdown: bool,
    pub(super) fail_shutdown: bool,
    pub(super) fail_final_shutdown: bool,
    pub(super) ignore_final_shutdown: bool,
    pub(super) fail_start_after_effect: bool,
    pub(super) change_settings_on_start_failure: bool,
    pub(super) driver: &'static str,
    pub(super) gpu_driver: &'static str,
    pub(super) payload_reads: usize,
    pub(super) journal_writes: usize,
    pub(super) fail_save_at: Option<usize>,
    pub(super) fail_read_after_save_at: Option<usize>,
    pub(super) mismatch_read_after_save_at: Option<usize>,
    pub(super) fail_payload: bool,
    pub(super) allocation_writes: usize,
    pub(super) resource_capability: crate::model::ResourceCapability,
    pub(super) allocation_calls: Vec<Resource>,
    pub(super) fail_allocation: Option<Resource>,
    pub(super) corrupt_resource: Option<Resource>,
}

#[derive(Clone, Copy)]
pub(super) enum Drift {
    Settings,
    Gpu,
    Assignment,
    Resource,
    Power,
    Journal,
    Unavailable,
}

impl Fake {
    fn drift(&mut self, stage: &str) {
        let Some((at, drift)) = self.drift_at else {
            return;
        };
        if at != stage {
            return;
        }
        self.drift_at = None;
        match drift {
            Drift::Settings => self.state.settings.high_mmio += 1,
            Drift::Gpu => self.gpu_driver = "externally-changed",
            Drift::Assignment => self.state.gpus = vec!["external-gpu".into()],
            Drift::Resource => {
                self.state.compute = Some(Allocation {
                    minimum: 1,
                    optimal: 2,
                    maximum: 3,
                })
            }
            Drift::Power => self.state.power = Power::Off,
            Drift::Journal => {
                if let Some(journal) = &mut self.journal {
                    journal.original.high_mmio += 1;
                }
            }
            Drift::Unavailable => self.inspect_unavailable = true,
        }
    }
}

pub(super) fn target() -> Target {
    Target {
        compute: None,
        encode: None,
        decode: None,
        vm_id: "11111111-1111-1111-1111-111111111111".into(),
        gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#test\GPUPARAV".into(),
        enabled: true,
        vram: None,
    }
}

pub(super) fn fake() -> Fake {
    Fake {
        os_preflights: 0,
        os_preflight_failure: None,
        os_checks: 0,
        os_failure: None,
        drift_at: None,
        inspect_unavailable: false,
        state: VmState {
            compute: None,
            encode: None,
            decode: None,
            vm_id: target().vm_id,
            name: "renamed VM".into(),
            power: Power::Running,
            generation: 2,
            gpus: vec![],
            vram: None,
            settings: Settings {
                low_mmio: 128,
                high_mmio: 512,
                cache_types: false,
                automatic_checkpoints: true,
            },
        },
        journal: None,
        events: vec![],
        fail_prepare: false,
        fail_settings: false,
        settings_writes: 0,
        fail_verify: false,
        stop_on_verify: false,
        change_settings_on_detach: false,
        payload_check_failure: None,
        ignore_shutdown: false,
        fail_shutdown: false,
        fail_final_shutdown: false,
        ignore_final_shutdown: false,
        fail_start_after_effect: false,
        change_settings_on_start_failure: false,
        driver: "current",
        gpu_driver: "new",
        payload_reads: 0,
        journal_writes: 0,
        fail_save_at: None,
        fail_read_after_save_at: None,
        mismatch_read_after_save_at: None,
        fail_payload: false,
        allocation_writes: 0,
        resource_capability: crate::model::ResourceCapability::Supported(Allocation {
            minimum: 0,
            optimal: 50,
            maximum: 100,
        }),
        allocation_calls: vec![],
        fail_allocation: None,
        corrupt_resource: None,
    }
}

pub(super) fn recovery_record(
    backend: &Fake,
    publication_required: bool,
) -> crate::configuration_store::SaveRecord {
    use crate::configuration_store::{Revision, SavePhase, SaveRecord};
    SaveRecord {
        schema: 1,
        retirement: None,
        reset: None,
        operation_id: "0123456789abcdef0123456789abcdef".into(),
        target: target(),
        expected: Revision::Missing,
        driver_version: "new".into(),
        payload_digest: Some(backend.driver.into()),
        plan_revision: Revision::of(b"approved"),
        intended_source: String::new(),
        publication_required,
        initial: backend.state.clone(),
        expected_power: Some(if publication_required {
            Power::Off
        } else {
            backend.state.power.clone()
        }),
        graphics_required: Some(true),
        initial_journal: backend.journal.clone(),
        expected_settings: backend.state.settings.clone(),
        phase: SavePhase::EffectsStarted,
    }
}

impl Backend for Fake {
    fn preflight_installation_os(&mut self, _: &Target) -> Result<(), String> {
        self.os_preflights += 1;
        crate::guest_compatibility::preflight(self.os_preflight_failure.clone().map_or(Ok(()), Err))
    }
    fn check_installation_os(&mut self, _: &Target) -> Result<(), String> {
        self.os_checks += 1;
        self.os_failure.clone().map_or(Ok(()), Err)
    }
    fn inspect(&mut self, _: &Target) -> Result<VmState, String> {
        if self.inspect_unavailable {
            return Err("host readback unavailable".into());
        }
        Ok(self.state.clone())
    }
    fn gpu(&mut self, t: &Target) -> Result<Gpu, String> {
        Ok(Gpu {
            compute: self.resource_capability.clone(),
            encode: self.resource_capability.clone(),
            decode: self.resource_capability.clone(),
            interface: t.gpu_interface.clone(),
            name: "GPU".into(),
            vendor: 0x10de,
            device: 1,
            driver_version: self.gpu_driver.into(),
            physical_vram_bytes: None,
            total_vram: None,
            preparation_supported: true,
            vram: Allocation {
                minimum: 0,
                maximum: 100,
                optimal: 50,
            }
            .into(),
        })
    }
    fn payload(&mut self, _: &Target) -> Result<String, String> {
        self.payload_reads += 1;
        if self.fail_payload {
            return Err("payload unavailable".into());
        }
        Ok(self.driver.into())
    }
    fn journal(&mut self, _: &Target) -> Result<Option<Journal>, String> {
        if self.fail_read_after_save_at == Some(self.journal_writes) {
            return Err("journal readback unavailable".into());
        }
        let mut journal = self.journal.clone();
        if self.mismatch_read_after_save_at == Some(self.journal_writes)
            && let Some(journal) = &mut journal
        {
            journal.check_restored = !journal.check_restored;
        }
        Ok(journal)
    }
    fn save(&mut self, _: &Target, j: &Journal) -> Result<(), String> {
        self.journal_writes += 1;
        if self.fail_save_at == Some(self.journal_writes) {
            return Err("journal publication denied".into());
        }
        self.journal = Some(j.clone());
        Ok(())
    }
    fn power(&mut self, _: &Target, p: Power) -> Result<(), String> {
        self.events.push(if p == Power::Running {
            "start"
        } else {
            "shutdown"
        });
        if p == Power::Off && self.fail_shutdown {
            return Err("shutdown refused".into());
        }
        if p == Power::Off && self.ignore_shutdown {
            return Ok(());
        }
        if p == Power::Off && self.events.contains(&"graphics") {
            if self.fail_final_shutdown {
                return Err("final shutdown refused".into());
            }
            if self.ignore_final_shutdown {
                return Ok(());
            }
        }
        self.state.power = p;
        self.drift(if self.state.power == Power::Running {
            "start"
        } else {
            "shutdown"
        });
        if self.state.power == Power::Running && self.fail_start_after_effect {
            if self.change_settings_on_start_failure {
                self.state.settings.high_mmio += 1;
            }
            return Err("start readback interrupted".into());
        }
        Ok(())
    }
    fn assign(&mut self, t: &Target, e: bool) -> Result<(), String> {
        self.events.push(if e { "attach" } else { "detach" });
        self.state.gpus = if e {
            vec![t.gpu_interface.clone()]
        } else {
            vec![]
        };
        if !e && self.change_settings_on_detach {
            self.state.settings.high_mmio += 1;
        }
        self.drift(if e { "attach" } else { "detach" });
        Ok(())
    }
    fn settings(&mut self, _: &Target, s: &Settings) -> Result<(), String> {
        self.settings_writes += 1;
        if self.fail_settings {
            return Err("settings interrupted".into());
        }
        self.state.settings = s.clone();
        self.drift("settings");
        Ok(())
    }
    fn allocation(&mut self, _: &Target, resource: Resource, v: &Allocation) -> Result<(), String> {
        self.allocation_calls.push(resource);
        if self.fail_allocation == Some(resource) {
            return Err("provider category write failed".into());
        }
        self.allocation_writes += 1;
        let readback = (self.corrupt_resource != Some(resource)).then(|| v.clone());
        match resource {
            Resource::Vram => self.state.vram = readback,
            Resource::Compute => self.state.compute = readback,
            Resource::Encode => self.state.encode = readback,
            Resource::Decode => self.state.decode = readback,
        }
        self.drift("allocation");
        Ok(())
    }
    fn prepare(&mut self, _: &Target) -> Result<String, String> {
        self.events.push("prepare");
        self.drift("prepare");
        if self.fail_prepare {
            Err("interrupted".into())
        } else {
            Ok(self.driver.into())
        }
    }
    fn verify(&mut self, _: &Target, _: &Gpu) -> Result<(), String> {
        self.drift("verify");
        if self.stop_on_verify {
            self.state.power = Power::Off;
        }
        if let Some(cause) = self.payload_check_failure {
            self.events.push("payload-check");
            return Err(crate::reporting::ComponentFailure {
                stage: None,
                component: if matches!(cause, crate::reporting::FailureCause::ReadbackMismatch) {
                    crate::reporting::Component::GuestReceipt
                } else {
                    crate::reporting::Component::GuestPayload
                },
                cause,
                code: None,
            }
            .into_error());
        }
        if let Some(resource) = self.corrupt_resource {
            match resource {
                Resource::Vram => self.state.vram = None,
                Resource::Compute => self.state.compute = None,
                Resource::Encode => self.state.encode = None,
                Resource::Decode => self.state.decode = None,
            }
        }
        self.events.push("graphics");
        if self.fail_verify {
            return Err(format!(
                "graphics failed: {}",
                crate::reporting::ComponentFailure {
                    stage: None,
                    component: crate::reporting::Component::GuestGraphics,
                    cause: crate::reporting::FailureCause::RenderingFailed,
                    code: None,
                }
                .into_error()
            ));
        }
        Ok(())
    }
}

pub(super) fn trace(preview: &EffectPreview) -> Vec<&'static str> {
    preview
        .actions
        .iter()
        .filter_map(|a| match a {
            PreviewAction::Shutdown => Some("shutdown"),
            PreviewAction::Start => Some("start"),
            PreviewAction::Detach => Some("detach"),
            PreviewAction::Prepare => Some("prepare"),
            PreviewAction::Attach => Some("attach"),
            PreviewAction::Verify => Some("graphics"),
            PreviewAction::Settings | PreviewAction::Allocation => None,
        })
        .collect()
}

pub(super) fn extended_target() -> Target {
    let mut target = target();
    for resource in [Resource::Compute, Resource::Encode, Resource::Decode] {
        target.set_allocation(
            resource,
            Some(Allocation {
                minimum: 10,
                optimal: 50,
                maximum: 90,
            }),
        );
    }
    target
}
