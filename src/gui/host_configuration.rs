//! Read-only comparison of committed intent with the latest host inventory.
use super::live_state::State;
use hyper_gpu_support::model::Resource;

/// Never use enrollment or an editing draft as expected applied configuration.
pub(super) fn comparison(state: &State, id: &str) -> String {
    let unknown = |reason: &str| format!("Host configuration: Unknown; {reason}.");
    if state.committed.revision(id).is_err() {
        return unknown("saved intent is unreadable");
    }
    let Some(saved) = state.committed.documents.get(id).map(|d| &d.target) else {
        return unknown("no saved intent");
    };
    if state.view.needs_readback {
        return unknown("retained host observations are historical; Refresh to compare");
    }
    let Some(inventory) = state.view.inventory.as_ref() else {
        return unknown("host inventory is unavailable");
    };
    let Some(vm) = state.view.vm(id) else {
        return unknown("VM was not returned by the latest host read");
    };
    if inventory.discovery.issues.iter().any(|issue| {
        issue.stage != "Preparation record"
            && (issue.target.is_none() || issue.target.as_deref() == Some(id))
    }) {
        return unknown("latest host observations are incomplete; inspect diagnostics");
    }

    let mut differences = Vec::new();
    let mut unknowns = Vec::new();
    let expected = if saved.enabled {
        vec![saved.gpu_interface.clone()]
    } else {
        Vec::new()
    };
    if vm.gpus != expected {
        let attachments = |gpus: &[String]| {
            if gpus.is_empty() {
                "none".into()
            } else {
                gpus.join(", ")
            }
        };
        differences.push(format!(
            "GPU attachment: expected {}; observed {}",
            attachments(&expected),
            attachments(&vm.gpus)
        ));
    }
    // Disabled intent requests no resource or compatibility writes. A completed
    // disable releases settings ownership; old journal.applied is not its baseline.
    if saved.enabled {
        for resource in Resource::ALL {
            match (saved.allocation(resource), vm.allocation(resource)) {
                (None, _) => {
                    unknowns.push(format!("{} has no explicit saved request", resource.name()))
                }
                (Some(request), None) => unknowns.push(format!(
                    "{}: expected {}; observed Unknown",
                    resource.name(),
                    super::live_state::allocation_description(Some(request), "Unknown")
                )),
                (Some(request), Some(observed)) if request != observed => {
                    differences.push(format!(
                        "{}: expected {}; observed {}",
                        resource.name(),
                        super::live_state::allocation_description(Some(request), "Unknown"),
                        super::live_state::allocation_description(Some(observed), "Unknown")
                    ))
                }
                _ => {}
            }
        }
        let recorded = inventory.recorded_state(id);
        let journal = inventory
            .managed
            .get(id)
            .and_then(Option::as_ref)
            .filter(|journal| {
                !recorded.recovery_required
                    && journal.schema == 1
                    && journal.vm_id == id
                    && journal.gpu_interface == saved.gpu_interface
                    && journal.prepared.is_some()
                    && !journal.settings_released
            });
        if let Some(journal) = journal {
            let expected = &journal.applied;
            let observed = &vm.settings;
            for (name, expected, observed) in [
                (
                    "Low MMIO (MiB)",
                    expected.low_mmio.to_string(),
                    observed.low_mmio.to_string(),
                ),
                (
                    "High MMIO (MiB)",
                    expected.high_mmio.to_string(),
                    observed.high_mmio.to_string(),
                ),
                (
                    "Guest cache types",
                    expected.cache_types.to_string(),
                    observed.cache_types.to_string(),
                ),
                (
                    "Automatic checkpoints",
                    expected.automatic_checkpoints.to_string(),
                    observed.automatic_checkpoints.to_string(),
                ),
            ] {
                if expected != observed {
                    differences.push(format!(
                        "{name} (managed host baseline): expected {expected}; observed {observed}"
                    ));
                }
            }
        } else {
            unknowns.push("managed compatibility baseline is unavailable or not applicable".into());
        }
    }
    let mut text = if differences.is_empty() {
        "Host configuration: compared values match the saved intent.".to_owned()
    } else {
        format!(
            "Host configuration differences (saved intent / latest Hyper-V read): {}. Review changes before repair or GPU removal.",
            differences.join("; ")
        )
    };
    if !unknowns.is_empty() {
        text.push_str(&format!(" Unknown: {}.", unknowns.join("; ")));
    }
    text
}
