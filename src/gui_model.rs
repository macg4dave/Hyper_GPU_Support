//! Native GUI presentation state; drafts never grant enrollment or mutate Hyper-V.
use crate::{
    model::{Configuration, Discovery, Power, Target, VmState},
    workflow::Journal,
};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};

/// Read-only runner discovery, including protected enrollment rather than inferred authorization.
#[derive(Clone, Debug, Deserialize)]
pub struct Inventory {
    /// Current provider data; guest health is not freshly checked.
    #[serde(flatten)]
    pub discovery: Discovery,
    /// Existing per-VM durable state.
    pub managed: BTreeMap<String, Option<Journal>>,
    /// Exact administrator-enrolled pairs. Older replies fail closed for editing.
    #[serde(default)]
    pub enrolled: Vec<Target>,
}

/// Recorded preparation and graphics facts, never a fresh guest health result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedState {
    /// Previous preparation; current driver parity requires a fresh plan.
    pub preparation: String,
    /// Last successful check for the exact enrolled pair, when available.
    pub graphics: String,
    /// Pending or inconsistent durable state requires operator inspection.
    pub recovery_required: bool,
}

impl Inventory {
    /// Present durable state only when its schema and identities match enrollment.
    /// Missing map entries mean no record was read; an explicit `None` means absent.
    pub fn recorded_state(&self, id: &str) -> RecordedState {
        let unknown = |preparation: &str, recovery_required| RecordedState {
            preparation: preparation.into(),
            graphics: "Unknown; no matching graphics verification record was read".into(),
            recovery_required,
        };
        match self.managed.get(id) {
            None => unknown("Preparation record unavailable", false),
            Some(None) => RecordedState {
                preparation: "No preparation record".into(),
                graphics: "No recorded graphics verification; attachment does not prove guest health".into(),
                recovery_required: false,
            },
            Some(Some(journal)) => {
                if journal.schema != 1
                    || journal.vm_id != id
                    || !self.enrolled.iter().any(|target| {
                        target.vm_id == id && target.gpu_interface == journal.gpu_interface
                    })
                {
                    return unknown("Preparation record does not match protected enrollment; inspect recovery", true);
                }
                RecordedState {
                    preparation: if journal.pending {
                        "Recovery pending; preparation outcome requires inspection"
                    } else if journal.prepared.is_some() {
                        "Previously prepared; current driver parity has not been checked"
                    } else {
                        "No completed preparation recorded"
                    }
                    .into(),
                    graphics: journal.last_verified.map_or_else(
                        || "No recorded graphics verification; attachment does not prove guest health".into(),
                        |time| format!("Last successful checked rendering: Unix UTC {time} (historical; current guest health unknown)"),
                    ),
                    recovery_required: journal.pending,
                }
            }
        }
    }
}

/// One staged target; selection and effective observation remain independent.
#[derive(Clone, Debug)]
pub struct View {
    /// Runtime intent, separately constrained by protected enrollment.
    pub configuration: Configuration,
    /// Last successful discovery; failure does not become an empty successful list.
    pub inventory: Option<Inventory>,
    /// Stable selected VM identity, retained across refresh.
    pub selected: Option<String>,
    /// One VM's proposed enable/disable operation.
    pub draft: Option<Target>,
    /// Successful apply whose configuration has not yet been saved.
    pub unsaved: bool,
    /// Failed/lost operation requires a successful readback before further execution.
    pub needs_readback: bool,
}
impl View {
    /// Start without any observed state or staged effects.
    pub fn new(configuration: Configuration) -> Self {
        Self {
            configuration,
            inventory: None,
            selected: None,
            draft: None,
            unsaved: false,
            needs_readback: false,
        }
    }
    /// Accept fresh discovery while preserving a stable selection and staged intent.
    pub fn refresh(&mut self, inventory: Inventory) {
        if self
            .selected
            .as_ref()
            .is_none_or(|id| !inventory.discovery.vms.iter().any(|v| &v.vm_id == id))
        {
            self.selected = inventory.discovery.vms.first().map(|v| v.vm_id.clone());
        }
        self.inventory = Some(inventory);
        self.needs_readback = false;
    }
    /// Lookup observed state by identity, never by name or a stale row index.
    pub fn vm(&self, id: &str) -> Option<&VmState> {
        self.inventory
            .as_ref()?
            .discovery
            .vms
            .iter()
            .find(|v| v.vm_id == id)
    }
    /// Explain why a target is ineligible; config cannot broaden protected enrollment.
    pub fn eligibility(&self, target: &Target) -> Result<(), String> {
        if self.needs_readback {
            return Err("Inventory is historical; refresh before further actions.".into());
        }
        let inventory = self.inventory.as_ref().ok_or("Refresh inventory first.")?;
        let vm = self
            .vm(&target.vm_id)
            .ok_or("The selected VM is no longer present; refresh or discard.")?;
        if !inventory
            .enrolled
            .iter()
            .any(|t| t.vm_id == target.vm_id && t.gpu_interface == target.gpu_interface)
        {
            return Err("This VM/GPU pair is not enrolled. Run administrator install after updating configuration.".into());
        }
        if vm.generation != 2 || !matches!(vm.power, Power::Off | Power::Running) {
            return Err("A stable Generation 2 VM is required.".into());
        }
        if vm.gpus.len() > 1 || vm.gpus.iter().any(|g| g != &target.gpu_interface) {
            return Err("Another GPU assignment is present; reconcile it before applying.".into());
        }
        let gpu = inventory
            .discovery
            .gpus
            .iter()
            .find(|g| g.interface == target.gpu_interface)
            .ok_or("The enrolled GPU is unavailable.")?;
        if inventory
            .managed
            .get(&target.vm_id)
            .and_then(Option::as_ref)
            .is_some_and(|j| {
                j.schema != 1 || j.vm_id != target.vm_id || j.gpu_interface != target.gpu_interface
            })
        {
            return Err(
                "Managed state belongs to a different pair; reconcile enrollment first.".into(),
            );
        }
        if target.enabled && !gpu.preparation_supported {
            return Err("Guest preparation is not supported for this GPU vendor.".into());
        }
        target.validate()?;
        if let Some(vram) = &target.vram {
            vram.validate(&gpu.vram)?;
        }
        Ok(())
    }
    /// Stage a toggle without touching configuration, journals, credentials or providers.
    pub fn stage(&mut self, id: &str, enabled: bool) -> Result<(), String> {
        if self.needs_readback {
            return Err("Refresh after the uncertain operation before editing.".into());
        }
        if self.unsaved {
            return Err(
                "Save the completed configuration before staging another operation.".into(),
            );
        }
        if self.draft.as_ref().is_some_and(|t| t.vm_id != id) {
            return Err("Apply or discard the other VM's pending change first.".into());
        }
        let mut target = self
            .configuration
            .targets
            .iter()
            .find(|t| t.vm_id == id)
            .cloned()
            .ok_or("Add this VM to configuration and enroll it first.")?;
        target.enabled = enabled;
        self.eligibility(&target)?;
        let observed = self.vm(id).ok_or("The VM is unavailable.")?;
        let effective = observed.gpus == vec![target.gpu_interface.clone()];
        let pending = self
            .inventory
            .as_ref()
            .and_then(|i| i.managed.get(id))
            .and_then(Option::as_ref)
            .is_some_and(|j| j.pending);
        self.draft = (effective != enabled || pending).then_some(target);
        Ok(())
    }
    /// Discard UI intent only.
    pub fn discard(&mut self) {
        self.draft = None;
    }
    /// Deliberately reapply enabled support, including current driver preparation.
    pub fn reapply(&mut self, id: &str) -> Result<(), String> {
        self.stage(id, true)?;
        self.draft = Some(
            self.configuration
                .targets
                .iter()
                .find(|t| t.vm_id == id)
                .ok_or("Unconfigured VM")?
                .clone(),
        );
        self.draft.as_mut().expect("just staged").enabled = true;
        Ok(())
    }
    /// Shared core validation for a truthful verification lifecycle confirmation.
    pub fn verification(&self, target: &Target) -> Result<Power, String> {
        let mut pair = target.clone();
        pair.enabled = false;
        self.eligibility(&pair)?;
        let inventory = self.inventory.as_ref().ok_or("Refresh first")?;
        crate::workflow::verification_power(
            target,
            self.vm(&target.vm_id).ok_or("VM unavailable")?,
            inventory
                .managed
                .get(&target.vm_id)
                .and_then(Option::as_ref),
        )
    }
    /// Revalidate the staged pair against fresh readback before requesting a plan.
    pub fn apply_target(&self) -> Result<Target, String> {
        if self.needs_readback {
            return Err("Refresh/read back the uncertain operation before retrying.".into());
        }
        if self.unsaved {
            return Err(
                "The operation succeeded; save its configuration instead of replaying apply."
                    .into(),
            );
        }
        let target = self.draft.as_ref().ok_or("Stage a change first.")?;
        self.eligibility(target)?;
        Ok(target.clone())
    }
    /// Separate successful provider execution from persistence; never replay for save failure.
    pub fn applied(&mut self, target: &Target) -> Result<(), String> {
        let old = self
            .configuration
            .targets
            .iter_mut()
            .find(|t| t.vm_id == target.vm_id)
            .ok_or("Applied target is absent from runtime configuration.")?;
        *old = target.clone();
        self.draft = None;
        self.unsaved = true;
        self.needs_readback = true;
        Ok(())
    }
    /// Row text reflects observed assignment; pending intent is a separate suffix.
    pub fn support_text(&self, vm: &VmState) -> String {
        let configured = self
            .configuration
            .targets
            .iter()
            .find(|t| t.vm_id == vm.vm_id);
        let effective = if vm.gpus.is_empty() {
            "Disabled"
        } else if configured.is_some_and(|t| vm.gpus == vec![t.gpu_interface.clone()]) {
            "Enabled"
        } else {
            "Other assignment"
        };
        match self.draft.as_ref().filter(|t| t.vm_id == vm.vm_id) {
            Some(t) => format!(
                "{effective} — Pending {}",
                if t.enabled { "enable" } else { "disable" }
            ),
            None => effective.into(),
        }
    }
}
/// Truthful provider allocation label, without GiB or enforcement interpretation.
pub fn allocation_text(vm: &VmState) -> &'static str {
    if vm.gpus.is_empty() {
        "Not assigned"
    } else if vm.vram.is_some() {
        "Provider values"
    } else {
        "Provider default"
    }
}

/// Save beside the source and replace only after flush; refuse external edits rather than clobbering them.
pub fn save_configuration(
    path: &Path,
    expected: &str,
    configuration: &Configuration,
) -> Result<String, String> {
    let text = toml::to_string_pretty(configuration).map_err(|e| e.to_string())?;
    if std::fs::read_to_string(path)
        .map_err(|e| format!("Read configuration before saving: {e}"))?
        != expected
    {
        return Err("Configuration changed outside the GUI. Resolve the file before saving; do not replay the successful VM operation.".into());
    }
    use std::io::Write;
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let temp = path.with_file_name(format!(
        ".hyper-gpu-save-{}-{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let mut created = false;
    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| e.to_string())?;
        created = true;
        file.write_all(text.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|e| e.to_string())?;
        drop(file);
        if std::fs::read_to_string(path).map_err(|e| e.to_string())? != expected {
            return Err(
                "Configuration changed during save; resolve the file before retrying save.".into(),
            );
        }
        std::fs::rename(&temp, path).map_err(|e| e.to_string())?;
        Ok(text)
    })();
    if created
        && temp.exists()
        && let Err(cleanup) = std::fs::remove_file(&temp)
    {
        return Err(format!(
            "{}; temporary save cleanup failed: {cleanup}",
            result.err().unwrap_or_else(|| "Save completed".into())
        ));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Allocation, Gpu, Settings};
    fn target(n: u8) -> Target {
        Target {
            vm_id: format!("00000000-0000-0000-0000-{n:012}"),
            gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#fixture\GPUPARAV".into(),
            enabled: true,
            vram: None,
        }
    }
    fn vm(n: u8) -> VmState {
        VmState {
            vm_id: target(n).vm_id,
            name: format!("VM {n}"),
            generation: 2,
            power: Power::Off,
            gpus: vec![],
            vram: None,
            settings: Settings {
                low_mmio: 128,
                high_mmio: 512,
                cache_types: false,
                automatic_checkpoints: true,
            },
        }
    }
    fn inventory() -> Inventory {
        Inventory {
            discovery: Discovery {
                vms: vec![vm(1), vm(2)],
                gpus: vec![Gpu {
                    interface: target(1).gpu_interface,
                    name: "GPU".into(),
                    vendor: 0x10de,
                    device: 1,
                    driver_version: "fixture".into(),
                    vram: Allocation {
                        minimum: 0,
                        optimal: 50,
                        maximum: 100,
                    },
                    preparation_supported: true,
                }],
            },
            managed: BTreeMap::new(),
            enrolled: vec![target(1), target(2)],
        }
    }
    fn view() -> View {
        let mut v = View::new(Configuration {
            schema: 2,
            targets: vec![target(1), target(2)],
        });
        v.refresh(inventory());
        v
    }
    fn journal() -> Journal {
        Journal {
            schema: 1,
            vm_id: target(1).vm_id,
            gpu_interface: target(1).gpu_interface,
            original: vm(1).settings,
            applied: vm(1).settings,
            prepared: Some("fixture-digest".into()),
            pending: false,
            restore_power: Power::Off,
            verification_only: false,
            last_verified: Some(123),
        }
    }
    #[test]
    fn unavailable_absent_and_unprepared_records_are_distinct() {
        let mut inventory = inventory();
        let id = target(1).vm_id;
        let unavailable = inventory.recorded_state(&id);
        assert!(unavailable.preparation.contains("unavailable"));
        assert!(unavailable.graphics.contains("Unknown"));
        inventory.managed.insert(id.clone(), None);
        let absent = inventory.recorded_state(&id);
        assert_eq!(absent.preparation, "No preparation record");
        assert!(!absent.recovery_required);
        let mut record = journal();
        record.prepared = None;
        record.last_verified = None;
        inventory.managed.insert(id.clone(), Some(record));
        let unprepared = inventory.recorded_state(&id);
        assert_eq!(unprepared.preparation, "No completed preparation recorded");
        assert!(unprepared.graphics.contains("No recorded"));
        assert_ne!(unavailable, absent);
        assert_ne!(absent, unprepared);
    }
    #[test]
    fn previous_preparation_and_rendering_never_claim_current_health_or_driver_parity() {
        let mut inventory = inventory();
        let id = target(1).vm_id;
        inventory.managed.insert(id.clone(), Some(journal()));
        inventory.discovery.vms[0].gpus = vec![target(1).gpu_interface];
        let recorded = inventory.recorded_state(&id);
        assert!(recorded.preparation.contains("current driver parity has not been checked"));
        assert!(recorded.graphics.contains("123"));
        assert!(recorded.graphics.contains("historical"));
        assert!(recorded.graphics.contains("current guest health unknown"));
        assert!(!recorded.recovery_required);
        // A detach does not erase history or turn it into a current check.
        inventory.discovery.vms[0].gpus.clear();
        assert_eq!(recorded, inventory.recorded_state(&id));
        let record = inventory.managed.get_mut(&id).unwrap().as_mut().unwrap();
        record.pending = true;
        let pending = inventory.recorded_state(&id);
        assert!(pending.recovery_required);
        assert!(pending.preparation.contains("outcome requires inspection"));
        assert_eq!(pending.graphics, recorded.graphics);
    }
    #[test]
    fn mismatched_records_cannot_supply_a_graphics_pass_for_the_selected_pair() {
        for mismatch in 0..4 {
            let mut inventory = inventory();
            let id = target(1).vm_id;
            let mut record = journal();
            match mismatch {
                0 => record.schema = 2,
                1 => record.vm_id = target(2).vm_id,
                2 => record.gpu_interface = "different-interface".into(),
                _ => inventory.enrolled.clear(),
            }
            inventory.managed.insert(id.clone(), Some(record));
            let recorded = inventory.recorded_state(&id);
            assert!(recorded.recovery_required);
            assert!(recorded.preparation.contains("does not match"));
            assert!(recorded.graphics.starts_with("Unknown"));
            assert!(!recorded.graphics.contains("123"));
        }
    }
    #[test]
    fn historical_inventory_blocks_actions_and_reapply_is_deliberate() {
        let mut v = view();
        v.inventory.as_mut().unwrap().discovery.vms[0].gpus = vec![target(1).gpu_interface];
        v.stage(&target(1).vm_id, true).unwrap();
        assert!(v.draft.is_none());
        v.reapply(&target(1).vm_id).unwrap();
        assert!(v.apply_target().unwrap().enabled);
        v.needs_readback = true;
        assert!(v.apply_target().is_err());
        assert!(v.verification(&target(1)).is_err());
        assert!(v.reapply(&target(1).vm_id).is_err());
        v.refresh(inventory());
        assert!(v.apply_target().is_ok());
    }
    #[test]
    fn verification_confirmation_uses_pending_off_recovery_and_refuses_management() {
        let mut v = view();
        let mut vm = vm(1);
        vm.gpus = vec![target(1).gpu_interface];
        vm.power = Power::Running;
        let journal = Journal {
            schema: 1,
            vm_id: vm.vm_id.clone(),
            gpu_interface: target(1).gpu_interface,
            original: vm.settings.clone(),
            applied: vm.settings.clone(),
            prepared: None,
            pending: true,
            restore_power: Power::Off,
            verification_only: true,
            last_verified: None,
        };
        let i = v.inventory.as_mut().unwrap();
        i.discovery.vms[0] = vm;
        i.managed.insert(target(1).vm_id.clone(), Some(journal));
        assert_eq!(v.verification(&target(1)).unwrap(), Power::Off);
        v.inventory
            .as_mut()
            .unwrap()
            .managed
            .get_mut(&target(1).vm_id)
            .unwrap()
            .as_mut()
            .unwrap()
            .verification_only = false;
        assert!(v.verification(&target(1)).is_err());
    }
    #[test]
    fn staged_toggle_is_separate_from_observed_and_config_and_survives_refresh() {
        let mut v = view();
        v.stage(&target(1).vm_id, true).unwrap();
        assert!(v.vm(&target(1).vm_id).unwrap().gpus.is_empty());
        assert!(v.configuration.targets[0].enabled);
        v.selected = Some(target(2).vm_id);
        v.refresh(inventory());
        assert_eq!(v.selected, Some(target(2).vm_id));
        assert_eq!(v.draft.as_ref().unwrap().vm_id, target(1).vm_id);
        assert!(
            v.stage(&target(2).vm_id, true)
                .unwrap_err()
                .contains("other VM")
        );
        v.discard();
        assert!(v.draft.is_none());
        assert!(v.configuration.targets[0].enabled);
    }
    #[test]
    fn enrollment_is_not_inferred_from_config_or_old_discovery() {
        let mut v = view();
        v.inventory.as_mut().unwrap().enrolled.clear();
        assert!(
            v.stage(&target(1).vm_id, true)
                .unwrap_err()
                .contains("not enrolled")
        );
        let mut data = serde_json::to_value(&v.inventory.as_ref().unwrap().discovery).unwrap();
        data["managed"] = serde_json::json!({});
        let old: Inventory = serde_json::from_value(data).unwrap();
        assert!(old.enrolled.is_empty());
    }
    #[test]
    fn refresh_invalidates_execution_when_draft_target_disappears_or_pair_changes() {
        let mut v = view();
        v.stage(&target(1).vm_id, true).unwrap();
        let mut next = inventory();
        next.discovery.vms.remove(0);
        v.refresh(next);
        assert!(v.draft.is_some());
        assert!(v.apply_target().is_err());
        let mut next = inventory();
        next.enrolled[0].gpu_interface = r"\\?\PCI#VEN_10DE&DEV_2D05#other\GPUPARAV".into();
        v.refresh(next);
        assert!(v.apply_target().is_err());
    }
    #[test]
    fn successful_apply_save_failure_and_uncertain_reply_cannot_trigger_blind_replay() {
        let mut v = view();
        v.stage(&target(1).vm_id, true).unwrap();
        v.applied(&target(1)).unwrap();
        assert!(v.draft.is_none());
        assert!(v.unsaved && v.needs_readback);
        assert!(v.apply_target().is_err());
        v.refresh(inventory());
        assert!(v.unsaved);
        assert!(v.stage(&target(1).vm_id, true).is_err());
        v.unsaved = false;
        v.needs_readback = true;
        assert!(v.stage(&target(1).vm_id, true).is_err());
        v.refresh(inventory());
        assert!(v.stage(&target(1).vm_id, true).is_ok());
    }
    #[test]
    fn eligibility_refuses_wrong_assignment_generation_transitions_and_vendor_enable() {
        for n in 0..4 {
            let mut v = view();
            let i = v.inventory.as_mut().unwrap();
            match n {
                0 => i.discovery.vms[0].generation = 1,
                1 => i.discovery.vms[0].power = Power::Other(4),
                2 => i.discovery.vms[0].gpus.push("other".into()),
                _ => i.discovery.gpus[0].preparation_supported = false,
            }
            assert!(v.stage(&target(1).vm_id, true).is_err());
            assert!(v.draft.is_none());
        }
    }
    #[test]
    fn observed_attachment_and_raw_allocation_do_not_claim_graphics_or_gib() {
        let mut v = view();
        let vm = &mut v.inventory.as_mut().unwrap().discovery.vms[0];
        assert_eq!(allocation_text(vm), "Not assigned");
        vm.gpus.push(target(1).gpu_interface);
        assert_eq!(allocation_text(vm), "Provider default");
        vm.vram = Some(Allocation {
            minimum: 1,
            optimal: 2,
            maximum: 3,
        });
        assert_eq!(allocation_text(vm), "Provider values");
        assert_eq!(v.support_text(v.vm(&target(1).vm_id).unwrap()), "Enabled");
        v.stage(&target(1).vm_id, false).unwrap();
        assert!(
            v.support_text(v.vm(&target(1).vm_id).unwrap())
                .contains("Enabled — Pending disable")
        );
    }
    #[test]
    fn save_replaces_complete_configuration_and_preserves_external_edits() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("local/gui-save-test-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("target.toml");
        let cfg = view().configuration;
        let old = toml::to_string(&cfg).unwrap();
        std::fs::write(&path, &old).unwrap();
        let mut changed = cfg.clone();
        changed.targets[0].enabled = false;
        let saved = save_configuration(&path, &old, &changed).unwrap();
        assert!(!Configuration::parse(&saved).unwrap().targets[0].enabled);
        std::fs::write(&path, &old).unwrap();
        assert!(
            save_configuration(&path, &saved, &changed)
                .unwrap_err()
                .contains("outside")
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), old);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
