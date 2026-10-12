//! One selected-VM workflow shared by CLI and GUI; laboratory tooling is not a dependency.
use crate::model::{Allocation, Gpu, Power, Resource, Settings, Target, VmState};
use serde::{Deserialize, Serialize};

mod apply;
mod guest_check;
mod journal;
mod planning;
mod reconciliation;
mod verification;

pub use apply::{apply, apply_approved};
pub use journal::{GuestCheck, Journal};
pub use planning::{
    EffectPreview, Plan, PreparationPreview, PreviewAction, SettingsPreview, plan, plan_reinstall,
};
pub use reconciliation::reconcile_observed;
pub use verification::{
    VerificationPlan, plan_verification, verification_power, verify, verify_approved,
};

pub(crate) use apply::{AdmittedApply, admit_approved, apply_admitted};
pub(crate) use reconciliation::reconcile_with_graphics;
pub(crate) use verification::{AdmittedVerification, admit_verification, verify_admitted};

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

/// Product result; allocation is provider-defined, without enforcement claims.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationResult {
    /// Independently observed state after the operation.
    pub effective: VmState,
    /// Whether preparation was changed.
    pub prepared: bool,
    /// Graphics verification was executed successfully.
    pub verified: bool,
    /// Settings changed by another manager were preserved during disable.
    pub settings_preserved: bool,
}
/// The concrete management boundary, also used for meaningful workflow tests.
pub trait Backend {
    /// Read fresh selected VM state.
    fn inspect(&mut self, target: &Target) -> Result<VmState, String>;
    /// Read the selected host GPU and its current driver.
    fn gpu(&mut self, target: &Target) -> Result<Gpu, String>;
    /// Discover and validate a current payload; return its digest.
    fn payload(&mut self, target: &Target) -> Result<String, String>;
    /// Read product-owned durable state, if present.
    fn journal(&mut self, target: &Target) -> Result<Option<Journal>, String>;
    /// Publish protected durable state before effects.
    fn save(&mut self, target: &Target, journal: &Journal) -> Result<(), String>;
    /// Change guest power gracefully; never force power-off.
    fn power(&mut self, target: &Target, power: Power) -> Result<(), String>;
    /// Change only the selected GPU assignment and verify readback.
    fn assign(&mut self, target: &Target, enabled: bool) -> Result<(), String>;
    /// Change only GPU compatibility settings, preserving CPU/RAM/security devices.
    fn settings(&mut self, target: &Target, settings: &Settings) -> Result<(), String>;
    /// Apply one supported raw provider category and verify independent readback.
    fn allocation(
        &mut self,
        target: &Target,
        resource: Resource,
        value: &Allocation,
    ) -> Result<(), String>;
    /// Prepare the current payload in the running guest; return independently verified digest.
    fn prepare(&mut self, target: &Target) -> Result<String, String>;
    /// Prepare with (acknowledged bytes, total bytes, completed files, total files).
    /// Adapters without transport retain stage progress only.
    fn prepare_with_progress(
        &mut self,
        target: &Target,
        progress: &mut dyn FnMut(u64, u64, usize, usize) -> Result<(), String>,
    ) -> Result<String, String> {
        let _ = progress;
        self.prepare(target)
    }
    /// Compare current guest payload/receipt, then require device health and hardware rendering.
    fn verify(&mut self, target: &Target, gpu: &Gpu) -> Result<(), String>;
}
