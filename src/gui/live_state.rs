//! Live presentation decisions reuse protected enrollment and shared validation.
use hyper_gpu_support::{
    gui_model::{Inventory, View},
    model::{Allocation, Configuration, Target},
    workflow::Plan,
};

pub(super) struct State {
    pub view: View,
    pub pending_selection: Option<String>,
    pub draft_gpu: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            view: View::new(Configuration {
                schema: 2,
                targets: vec![],
            }),
            pending_selection: None,
            draft_gpu: None,
        }
    }
}

impl State {
    pub fn refresh(&mut self, inventory: Inventory) {
        // Enrollment supplies a bounded starting draft, not committed configuration.
        self.view.configuration.targets = inventory.enrolled.clone();
        let selected = self.view.selected.clone();
        self.view.refresh(inventory);
        if self.view.draft.is_some() {
            // A disappeared VM must not silently transfer its draft to another VM.
            self.view.selected = selected;
        }
    }

    pub fn target(&self, id: &str) -> Result<Target, String> {
        self.view.configuration.targets.iter().find(|t| t.vm_id == id).cloned()
            .ok_or_else(|| "This VM has no protected enrollment. Use CLI installation until GUI enrollment is connected.".into())
    }

    pub fn editor_eligibility(&self, id: &str) -> Result<(), String> {
        let mut pair = self.target(id)?;
        // Pair eligibility must allow correcting a now-invalid old allocation or
        // disabling an unsupported preparation adapter. Draft validation is separate.
        pair.enabled = false;
        pair.vram = None;
        self.view.eligibility(&pair)
    }

    pub fn draft(
        &self,
        id: &str,
        enabled: bool,
        gpu_index: i32,
        values: &[String],
    ) -> Result<Target, String> {
        let mut target = self.target(id)?;
        let inventory = self
            .view
            .inventory
            .as_ref()
            .ok_or("Refresh inventory first.")?;
        let gpu = usize::try_from(gpu_index)
            .ok()
            .and_then(|i| inventory.discovery.gpus.get(i))
            .ok_or("Select an available physical GPU.")?;
        target.gpu_interface = gpu.interface.clone();
        target.enabled = enabled;
        if values.len() != 12 || values[3..].iter().any(|v| !v.trim().is_empty()) {
            return Err(
                "Compute, encode and decode allocation are not supported by the current backend."
                    .into(),
            );
        }
        let vram = &values[..3];
        target.vram = if vram.iter().all(|v| v.trim().is_empty()) {
            None
        } else {
            let parsed = vram.iter().map(|v| v.trim().parse::<u64>()
                .map_err(|_| "Enter all three VRAM values as unsigned provider integers, or leave all three empty.".to_owned()))
                .collect::<Result<Vec<_>, _>>()?;
            Some(Allocation {
                minimum: parsed[0],
                optimal: parsed[1],
                maximum: parsed[2],
            })
        };
        self.view.eligibility(&target)?;
        Ok(target)
    }
}

pub(super) fn allocation_values(target: &Target) -> Vec<String> {
    let mut values = vec![String::new(); 12];
    if let Some(vram) = &target.vram {
        values[0] = vram.minimum.to_string();
        values[1] = vram.optimal.to_string();
        values[2] = vram.maximum.to_string();
    }
    values
}

pub(super) fn review_text(plan: &Plan) -> String {
    let requested = allocation_description(
        plan.desired.vram.as_ref(),
        "Unspecified; no explicit VRAM request",
    );
    let planned = allocation_description(
        plan.preview.allocation.as_ref(),
        "No allocation write planned; existing/provider values are not explicitly reset",
    );
    format!(
        "VM: {}\nVM ID: {}\nGPU interface: {}\nDesired GPU support: {}\nVRAM draft (Minimum / Optimal / Maximum): {requested}\nPlanned allocation write: {planned}\n\n{}\n\nAllocation units: {}\n\nRead-only preview from the shared CLI planner. Execution is unavailable until per-operation elevation and protected configuration saving are connected. No VM or guest changes were performed.",
        plan.observed.name,
        plan.desired.vm_id,
        plan.desired.gpu_interface,
        if plan.desired.enabled {
            "Enabled"
        } else {
            "Disabled"
        },
        plan.preview.summary.join("\n"),
        plan.allocation_units,
    )
}

fn allocation_description(value: Option<&Allocation>, absent: &str) -> String {
    value.map_or_else(
        || absent.to_owned(),
        |v| format!("{} / {} / {}", v.minimum, v.optimal, v.maximum),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper_gpu_support::model::{Discovery, Gpu, Power, Settings, VmState};

    fn inventory() -> Inventory {
        let target = Target {
            vm_id: "12345678-1234-1234-1234-123456789abc".into(),
            gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#fixture\GPUPARAV".into(),
            enabled: true,
            vram: None,
        };
        Inventory {
            discovery: Discovery {
                vms: vec![VmState {
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
                    interface: target.gpu_interface.clone(),
                    name: "Identical GPU name".into(),
                    vendor: 0x10de,
                    device: 1,
                    driver_version: "fixture".into(),
                    preparation_supported: true,
                    vram: Allocation {
                        minimum: 10,
                        optimal: 50,
                        maximum: 100,
                    },
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

    #[test]
    fn drafts_validate_raw_vram_and_never_mutate_observation_or_enrollment() {
        let s = state();
        let target = s
            .draft(id(&s), true, 0, &values("10", "50", "100"))
            .unwrap();
        assert_eq!(target.vram.unwrap().optimal, 50);
        assert!(s.view.vm(id(&s)).unwrap().gpus.is_empty());
        assert!(s.view.configuration.targets[0].vram.is_none());
        assert!(
            s.draft(id(&s), false, 0, &values("", "", ""))
                .unwrap()
                .vram
                .is_none()
        );
        assert_eq!(
            allocation_values(&s.target(id(&s)).unwrap()),
            values("", "", "")
        );
    }
    #[test]
    fn incomplete_overflow_out_of_bounds_and_unsupported_fields_are_blocked() {
        let s = state();
        for v in [
            values("10", "", "100"),
            values("-1", "50", "100"),
            values("10", "50", "18446744073709551616"),
            values("9", "50", "100"),
            values("10", "101", "100"),
            values("10", "50", "101"),
        ] {
            assert!(s.draft(id(&s), true, 0, &v).is_err(), "{v:?}");
        }
        let mut v = values("10", "50", "100");
        v[4] = "50".into();
        assert!(
            s.draft(id(&s), true, 0, &v)
                .unwrap_err()
                .contains("Compute")
        );
        assert!(s.draft(id(&s), true, 0, &v[..3]).is_err());
    }
    #[test]
    fn duplicate_gpu_names_do_not_grant_a_different_pair() {
        let mut s = state();
        let i = s.view.inventory.as_mut().unwrap();
        let mut other = i.discovery.gpus[0].clone();
        other.interface = r"\\?\PCI#VEN_10DE&DEV_2D05#other\GPUPARAV".into();
        i.discovery.gpus.push(other);
        assert!(
            s.draft(id(&s), true, 1, &values("", "", ""))
                .unwrap_err()
                .contains("not enrolled")
        );
        assert!(s.draft(id(&s), true, -1, &values("", "", "")).is_err());
    }
    #[test]
    fn failed_reads_and_removed_enrollment_block_preview_without_losing_drafts() {
        let mut s = state();
        let original = s
            .draft(id(&s), true, 0, &values("10", "50", "100"))
            .unwrap();
        s.view.draft = Some(original.clone());
        s.view.needs_readback = true;
        assert!(
            s.draft(id(&s), true, 0, &values("10", "50", "100"))
                .is_err()
        );
        let mut next = inventory();
        next.enrolled.clear();
        s.refresh(next);
        assert_eq!(s.view.draft, Some(original));
        assert!(
            s.draft(id(&s), true, 0, &values("10", "50", "100"))
                .is_err()
        );
    }
    #[test]
    fn disappearing_vm_keeps_its_draft_identity_until_discard() {
        let mut s = state();
        s.view.draft = Some(s.target(id(&s)).unwrap());
        let selected = s.view.selected.clone();
        let mut next = inventory();
        next.discovery.vms[0].vm_id = "12345678-1234-1234-1234-000000000002".into();
        s.refresh(next);
        assert_eq!(s.view.selected, selected);
        assert!(s.view.vm(id(&s)).is_none());
        assert!(s.draft(id(&s), true, 0, &values("", "", "")).is_err());
        s.view.discard();
        s.refresh(inventory());
        assert!(s.view.vm(id(&s)).is_some());
    }

    #[test]
    fn changed_provider_bounds_leave_editor_available_to_correct_old_vram() {
        let mut s = state();
        s.view.configuration.targets[0].vram = Some(Allocation {
            minimum: 10,
            optimal: 150,
            maximum: 200,
        });
        assert!(s.editor_eligibility(id(&s)).is_ok());
        let old = allocation_values(&s.target(id(&s)).unwrap());
        assert!(s.draft(id(&s), true, 0, &old).is_err());
        assert!(s.draft(id(&s), true, 0, &values("10", "50", "100")).is_ok());
        assert!(s.draft(id(&s), false, 0, &values("", "", "")).is_ok());
    }

    #[test]
    fn review_distinguishes_requested_triples_and_no_write_semantics() {
        use hyper_gpu_support::workflow::EffectPreview;
        let s = state();
        let target = s
            .draft(id(&s), true, 0, &values("10", "50", "100"))
            .unwrap();
        let mut plan = Plan {
            desired: target.clone(),
            observed: s.view.vm(id(&s)).unwrap().clone(),
            managed: None,
            preparation: None,
            preview: EffectPreview {
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
        };
        let first = review_text(&plan);
        assert!(first.contains("VRAM draft (Minimum / Optimal / Maximum): 10 / 50 / 100"));
        assert!(first.contains("Planned allocation write: 10 / 50 / 100"));
        plan.desired.vram = Some(Allocation {
            minimum: 20,
            optimal: 60,
            maximum: 90,
        });
        plan.preview.allocation = plan.desired.vram.clone();
        assert_ne!(first, review_text(&plan));
        plan.preview.allocation = None;
        assert!(review_text(&plan).contains("No allocation write planned"));
        plan.desired.vram = None;
        assert!(review_text(&plan).contains("Unspecified; no explicit VRAM request"));
    }
}
