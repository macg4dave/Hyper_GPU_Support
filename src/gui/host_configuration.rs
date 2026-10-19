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
    let unknown = |reason: &str| {
        (
            hyper_gpu_support::ui_text::diagnostics::host_comparison::unknown_reason(reason)
                .into_string(),
            false,
        )
    };
    if state.committed.revision(id).is_err() {
        return unknown(
            hyper_gpu_support::ui_text::diagnostics::host_comparison::saved_unreadable(),
        );
    }
    let Some(saved) = state.committed.documents.get(id).map(|d| &d.target) else {
        return unknown(hyper_gpu_support::ui_text::diagnostics::host_comparison::saved_absent());
    };
    if state.view.needs_readback {
        return unknown(hyper_gpu_support::ui_text::diagnostics::host_comparison::historical());
    }
    let Some(inventory) = state.view.inventory.as_ref() else {
        return unknown(
            hyper_gpu_support::ui_text::diagnostics::host_comparison::inventory_unavailable(),
        );
    };
    let Some(vm) = state.view.vm(id) else {
        return unknown(hyper_gpu_support::ui_text::diagnostics::host_comparison::vm_missing());
    };
    if inventory.discovery.issues.iter().any(|issue| {
        issue.stage != "Preparation record"
            && (issue.target.is_none() || issue.target.as_deref() == Some(id))
    }) {
        return unknown(
            hyper_gpu_support::ui_text::diagnostics::host_comparison::observations_incomplete(),
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
                hyper_gpu_support::ui_text::diagnostics::host_comparison::attachments::none().into()
            } else {
                gpus.join(", ")
            }
        };
        differences.push(
            hyper_gpu_support::ui_text::diagnostics::host_comparison::attachment_difference(
                &attachments(&expected),
                &attachments(&vm.gpus),
            )
            .into_string(),
        );
    }
    // Disabled intent requests no resource or compatibility writes. A completed
    // disable releases settings ownership; old journal.applied is not its baseline.
    if saved.enabled {
        for resource in Resource::ALL {
            match (saved.allocation(resource), vm.allocation(resource)) {
                (None, _) => unknowns.push(
                    hyper_gpu_support::ui_text::diagnostics::host_comparison::no_saved_request(
                        resource.name(),
                    )
                    .into_string(),
                ),
                (Some(request), None) => unknowns.push(
                    hyper_gpu_support::ui_text::diagnostics::host_comparison::observed_unknown(
                        resource.name(),
                        &super::live_state::allocation_description(
                            Some(request),
                            hyper_gpu_support::ui_text::common::state::unknown(),
                        ),
                    )
                    .into_string(),
                ),
                (Some(request), Some(observed)) if request != observed => differences.push(
                    hyper_gpu_support::ui_text::diagnostics::host_comparison::resource_difference(
                        resource.name(),
                        &super::live_state::allocation_description(
                            Some(request),
                            hyper_gpu_support::ui_text::common::state::unknown(),
                        ),
                        &super::live_state::allocation_description(
                            Some(observed),
                            hyper_gpu_support::ui_text::common::state::unknown(),
                        ),
                    )
                    .into_string(),
                ),
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
                    hyper_gpu_support::ui_text::common::compatibility::low_mmio(),
                    expected.low_mmio.to_string(),
                    observed.low_mmio.to_string(),
                ),
                (
                    hyper_gpu_support::ui_text::common::compatibility::high_mmio(),
                    expected.high_mmio.to_string(),
                    observed.high_mmio.to_string(),
                ),
                (
                    hyper_gpu_support::ui_text::common::compatibility::cache_types(),
                    expected.cache_types.to_string(),
                    observed.cache_types.to_string(),
                ),
                (
                    hyper_gpu_support::ui_text::common::compatibility::checkpoints(),
                    expected.automatic_checkpoints.to_string(),
                    observed.automatic_checkpoints.to_string(),
                ),
            ] {
                if expected != observed {
                    differences.push(
                        hyper_gpu_support::ui_text::diagnostics::host_comparison::baseline_difference(name, &expected, &observed).into_string(),
                    );
                }
            }
        } else {
            unknowns.push(
                hyper_gpu_support::ui_text::diagnostics::host_comparison::baseline_unavailable()
                    .into(),
            );
        }
    }
    let mut text = if differences.is_empty() {
        hyper_gpu_support::ui_text::diagnostics::host_comparison::matches().to_owned()
    } else {
        hyper_gpu_support::ui_text::diagnostics::host_comparison::differences(
            &differences.join(hyper_gpu_support::ui_text::common::list::separator()),
        )
        .into_string()
    };
    if !unknowns.is_empty() {
        text.push_str(
            &hyper_gpu_support::ui_text::diagnostics::host_comparison::unknowns(
                &unknowns.join(hyper_gpu_support::ui_text::common::list::separator()),
            ),
        );
    }
    (text, !differences.is_empty())
}
