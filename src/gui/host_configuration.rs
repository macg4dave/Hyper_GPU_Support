//! Read-only comparison of committed intent with the latest host inventory.
use super::live_state::State;
use hyper_gpu_support::model::Resource;

/// Never use enrollment or an editing draft as expected applied configuration.
pub(super) fn comparison(state: &State, id: &str) -> String {
    assessment(state, id).0
}

/// Only fresh observed differences are problems; unknowns and history are not failures.
pub(super) fn has_current_problem(state: &State, id: &str) -> bool {
    assessment(state, id).1
}

fn assessment(state: &State, id: &str) -> (String, bool) {
    let unknown = |reason: &str| (format!("Host configuration: Unknown; {reason}."), false);
    if state.committed.revision(id).is_err() {
        return unknown(hyper_gpu_support::ui_text::diagnostics::host_comparison::SAVED_UNREADABLE);
    }
    let Some(saved) = state.committed.documents.get(id).map(|d| &d.target) else {
        return unknown(hyper_gpu_support::ui_text::diagnostics::host_comparison::SAVED_ABSENT);
    };
    if state.view.needs_readback {
        return unknown(hyper_gpu_support::ui_text::diagnostics::host_comparison::HISTORICAL);
    }
    let Some(inventory) = state.view.inventory.as_ref() else {
        return unknown(
            hyper_gpu_support::ui_text::diagnostics::host_comparison::INVENTORY_UNAVAILABLE,
        );
    };
    let Some(vm) = state.view.vm(id) else {
        return unknown(hyper_gpu_support::ui_text::diagnostics::host_comparison::VM_MISSING);
    };
    if inventory.discovery.issues.iter().any(|issue| {
        issue.stage != "Preparation record"
            && (issue.target.is_none() || issue.target.as_deref() == Some(id))
    }) {
        return unknown(
            hyper_gpu_support::ui_text::diagnostics::host_comparison::OBSERVATIONS_INCOMPLETE,
        );
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
                hyper_gpu_support::ui_text::diagnostics::host_comparison::attachments::NONE.into()
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
                    super::live_state::allocation_description(
                        Some(request),
                        hyper_gpu_support::ui_text::common::state::UNKNOWN
                    )
                )),
                (Some(request), Some(observed)) if request != observed => {
                    differences.push(format!(
                        "{}: expected {}; observed {}",
                        resource.name(),
                        super::live_state::allocation_description(
                            Some(request),
                            hyper_gpu_support::ui_text::common::state::UNKNOWN
                        ),
                        super::live_state::allocation_description(
                            Some(observed),
                            hyper_gpu_support::ui_text::common::state::UNKNOWN
                        )
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
                    && !journal.settings_released
            });
        if let Some(journal) = journal {
            let expected = &journal.applied;
            let observed = &vm.settings;
            for (name, expected, observed) in [
                (
                    hyper_gpu_support::ui_text::common::compatibility::LOW_MMIO,
                    expected.low_mmio.to_string(),
                    observed.low_mmio.to_string(),
                ),
                (
                    hyper_gpu_support::ui_text::common::compatibility::HIGH_MMIO,
                    expected.high_mmio.to_string(),
                    observed.high_mmio.to_string(),
                ),
                (
                    hyper_gpu_support::ui_text::common::compatibility::CACHE_TYPES,
                    expected.cache_types.to_string(),
                    observed.cache_types.to_string(),
                ),
                (
                    hyper_gpu_support::ui_text::common::compatibility::CHECKPOINTS,
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
            unknowns.push(
                hyper_gpu_support::ui_text::diagnostics::host_comparison::BASELINE_UNAVAILABLE
                    .into(),
            );
        }
    }
    let mut text = if differences.is_empty() {
        hyper_gpu_support::ui_text::diagnostics::host_comparison::MATCHES.to_owned()
    } else {
        format!(
            "Host configuration differences (saved intent / latest Hyper-V read): {}. Review changes before repair or GPU removal.",
            differences.join("; ")
        )
    };
    if !unknowns.is_empty() {
        text.push_str(&format!(" Unknown: {}.", unknowns.join("; ")));
    }
    (text, !differences.is_empty())
}
