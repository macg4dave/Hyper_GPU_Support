//! Stage-specific host facts. Only our completed effect advances expectations.
use crate::{
    model::{Allocation, Gpu, Power, Resource, Settings, Target, VmState},
    workflow::{Backend, Journal},
};

pub(super) struct ApplyFacts {
    pub(super) state: VmState,
    gpu: Gpu,
    pub(super) journal: Option<Journal>,
}

fn equal<T: serde::Serialize>(left: &T, right: &T) -> Result<bool, String> {
    Ok(serde_json::to_value(left).map_err(|e| e.to_string())?
        == serde_json::to_value(right).map_err(|e| e.to_string())?)
}

impl ApplyFacts {
    pub(super) fn new(state: VmState, gpu: Gpu, journal: Option<Journal>) -> Self {
        Self {
            state,
            gpu,
            journal,
        }
    }

    pub(super) fn check(&self, backend: &mut impl Backend, target: &Target) -> Result<(), String> {
        if backend.inspect(target)?.operational() != self.state.operational()
            || backend.gpu(target)?.operational() != self.gpu.operational()
            || !equal(&backend.journal(target)?, &self.journal)?
        {
            return Err("Apply facts changed or independent final state verification failed; retain recovery before further effects".into());
        }
        Ok(())
    }

    pub(super) fn save(
        &mut self,
        backend: &mut impl Backend,
        target: &Target,
        journal: &Journal,
    ) -> Result<(), String> {
        self.check(backend, target)?;
        backend.save(target, journal)?;
        self.journal = Some(journal.clone());
        self.check(backend, target)
    }

    pub(super) fn power(
        &mut self,
        backend: &mut impl Backend,
        target: &Target,
        power: Power,
    ) -> Result<(), String> {
        self.check(backend, target)?;
        backend.power(target, power.clone())?;
        self.state.power = power;
        self.check(backend, target)
    }

    pub(super) fn settings(
        &mut self,
        backend: &mut impl Backend,
        target: &Target,
        settings: &Settings,
    ) -> Result<(), String> {
        self.check(backend, target)?;
        backend.settings(target, settings)?;
        self.state.settings = settings.clone();
        self.check(backend, target)
    }

    pub(super) fn allocation(
        &mut self,
        backend: &mut impl Backend,
        target: &Target,
        resource: Resource,
        value: &Allocation,
    ) -> Result<(), String> {
        self.check(backend, target)?;
        backend.allocation(target, resource, value)?;
        match resource {
            Resource::Vram => self.state.vram = Some(value.clone()),
            Resource::Compute => self.state.compute = Some(value.clone()),
            Resource::Encode => self.state.encode = Some(value.clone()),
            Resource::Decode => self.state.decode = Some(value.clone()),
        }
        self.check(backend, target)
    }

    pub(super) fn assign(
        &mut self,
        backend: &mut impl Backend,
        target: &Target,
        enabled: bool,
    ) -> Result<(), String> {
        self.check(backend, target)?;
        backend.assign(target, enabled)?;
        self.state.gpus = if enabled {
            vec![target.gpu_interface.clone()]
        } else {
            vec![]
        };
        let observed = backend.inspect(target)?;
        // Adapter creation/removal changes resource observations, including native
        // defaults. Capture only these fields; all unrelated facts must still match.
        self.state.vram = observed.vram;
        self.state.compute = observed.compute;
        self.state.encode = observed.encode;
        self.state.decode = observed.decode;
        self.check(backend, target)
    }

    pub(super) fn restore(
        &mut self,
        backend: &mut impl Backend,
        target: &Target,
        power: &Power,
    ) -> Result<(), String> {
        let observed = backend.inspect(target)?;
        if !matches!(observed.power, Power::Off | Power::Running) {
            return Err(
                "Apply power restoration is transitional or unavailable; retain recovery".into(),
            );
        }
        // A failed start can have taken effect. Power alone may differ, but changed
        // assignments/settings/resources/GPU/journal must block any restoration.
        let expected_power = self.state.power.clone();
        self.state.power = observed.power;
        self.check(backend, target)?;
        if *power == Power::Running
            && self.state.power != Power::Running
            && expected_power != Power::Off
        {
            return Err("Apply cannot authorize restarting the externally stopped guest".into());
        }
        if self.state.power != *power {
            self.power(backend, target, power.clone())?;
        }
        self.check(backend, target)
    }
}
