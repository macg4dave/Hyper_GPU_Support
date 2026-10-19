//! Review and confirmation of partial GPU setup removal using retained recovery.
use super::{
    SharedState, background::background_operation, presentation::present, progress::progress_to_ui,
    state_access::with_completion_state,
};
use crate::gui::{
    AppWindow, dialog,
    live_state::State,
    operation::{Kind, Operation},
};
use slint::ComponentHandle;
use std::sync::{Arc, atomic::AtomicBool};

#[cfg(windows)]
pub(super) fn partial_removal_candidate(
    state: &State,
) -> Result<
    (
        hyper_gpu_support::configuration_store::Revision,
        hyper_gpu_support::workflow::ResetReview,
    ),
    String,
> {
    let record = state
        .recovery_for_vm(state.view.selected.as_deref().unwrap_or_default())
        .ok_or(hyper_gpu_support::ui_text::operations::partial_removal::no_record())?;
    if state.observation_historical(&record.target.vm_id) {
        return Err(
            hyper_gpu_support::ui_text::operations::partial_removal::status_unavailable().into(),
        );
    }
    if matches!(
        record.phase,
        hyper_gpu_support::configuration_store::SavePhase::Admitted
    ) {
        return Err(
            hyper_gpu_support::ui_text::operations::partial_removal::stopped_before_changes()
                .into(),
        );
    }
    let inventory = state
        .view
        .inventory
        .as_ref()
        .ok_or(hyper_gpu_support::ui_text::common::errors::refresh_inventory())?;
    let observed = state
        .view
        .vm(&record.target.vm_id)
        .ok_or(hyper_gpu_support::ui_text::operations::partial_removal::vm_missing())?;
    let approved = if let Some(review) = &record.reset {
        review.clone()
    } else {
        hyper_gpu_support::workflow::ResetReview {
            observed: observed.clone(),
            gpu: inventory
                .discovery
                .gpus
                .iter()
                .find(|g| g.interface == record.target.gpu_interface)
                .ok_or(hyper_gpu_support::ui_text::operations::partial_removal::gpu_missing())?
                .clone(),
            journal: inventory
                .managed
                .get(&record.target.vm_id)
                .and_then(Option::as_ref)
                .ok_or(hyper_gpu_support::ui_text::operations::partial_removal::journal_missing())?
                .clone(),
        }
    };
    approved.validate(record, &state.committed.revision(&record.target.vm_id)?)?;
    let revision = hyper_gpu_support::configuration_store::Revision::of(
        &serde_json::to_vec(record).map_err(|e| e.to_string())?,
    );
    Ok((revision, approved))
}

#[cfg(windows)]
pub(super) fn review_reset(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let (operation, revision, approved) = {
        let state = state
            .lock()
            .map_err(|_| hyper_gpu_support::ui_text::common::state::presentation_unavailable())?;
        let (revision, approved) = partial_removal_candidate(&state)?;
        let operation = state
            .recovery_for_vm(state.view.selected.as_deref().unwrap_or_default())
            .ok_or(hyper_gpu_support::ui_text::common::errors::refresh_recovery())?
            .operation_id
            .clone();
        (operation, revision, approved)
    };
    let shutdown_approved = approved.observed.power == hyper_gpu_support::model::Power::Running;
    let name = approved.observed.name.clone();
    let progress = progress_to_ui(ui.as_weak());
    let finished = state.clone();
    background_operation(
        ui,
        state,
        busy,
        Operation::new(
            Kind::Remove,
            hyper_gpu_support::ui_text::operations::partial_removal::title(),
        ),
        Some(&name),
        false,
        move || {
            hyper_gpu_support::worker::reset_partial(
                operation,
                revision,
                approved,
                shutdown_approved,
                progress,
            )
        },
        move |ui, result, readback| {
            if !with_completion_state(ui, &finished, &result, |state| {
                let status = state.finish_readback(readback);
                present(ui, state, !ui.get_dirty());
                status
            }) {
                return;
            }
            match result {
                Ok(_) => dialog(
                    ui,
                    "reset-complete",
                    hyper_gpu_support::ui_text::operations::partial_removal::completed::title(),
                    hyper_gpu_support::ui_text::operations::partial_removal::completed::body(),
                ),
                Err(error) => dialog(
                    ui,
                    "error",
                    hyper_gpu_support::ui_text::operations::partial_removal::failed_title(),
                    &error,
                ),
            }
        },
    )
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use hyper_gpu_support::{
        configuration_store::{Revision, SavePhase, SaveRecord},
        model::{Configuration, Power, Settings, VmState},
        observation::{Mode, Provenance},
    };

    #[test]
    fn completion_for_another_vm_cannot_offer_removal_from_a_historical_recovery_row() {
        let target = Configuration::parse(include_str!(
            "../../../config/samples/vms/a5801e91-1083-4e79-a803-000000000001.toml"
        ))
        .unwrap()
        .targets
        .remove(0);
        let initial = VmState {
            vm_id: target.vm_id.clone(),
            name: "Recovery VM".into(),
            power: Power::Off,
            generation: 2,
            gpus: vec![],
            settings: Settings {
                low_mmio: 0,
                high_mmio: 0,
                cache_types: false,
                automatic_checkpoints: true,
            },
            vram: None,
            compute: None,
            encode: None,
            decode: None,
        };
        let mut state = State::default();
        state.view.selected = Some(target.vm_id.clone());
        state.recoveries = vec![SaveRecord {
            reset: None,
            retirement: None,
            schema: 1,
            operation_id: "0123456789abcdef0123456789abcdef".into(),
            target: target.clone(),
            expected: Revision::Missing,
            driver_version: "fixture".into(),
            payload_digest: Some("fixture payload".into()),
            plan_revision: Revision::of(b"fixture plan"),
            intended_source: Configuration {
                schema: 2,
                targets: vec![target.clone()],
            }
            .vm_documents()
            .unwrap()
            .into_values()
            .next()
            .unwrap(),
            publication_required: true,
            expected_power: Some(Power::Off),
            graphics_required: Some(true),
            initial_journal: None,
            expected_settings: initial.settings.for_gpu(),
            initial,
            phase: SavePhase::EffectsStarted,
        }];
        let mut other = target.clone();
        other.vm_id = "a5801e91-1083-4e79-a803-000000000002".into();
        let unavailable = "Current VM status isn't available. Try updating the status again before reviewing removal.";
        for (mode, selected, historical) in [
            (Mode::Completion, Some(other), true),
            (Mode::Completion, None, true),
            (Mode::Completion, Some(target), false),
            (Mode::Full, None, false),
            (Mode::Poll, None, false),
        ] {
            state.observation = Some(Provenance {
                schema: 1,
                authority: Revision::of(b"fixture authority"),
                configuration: Revision::of(b"fixture configuration"),
                mode,
                selected,
            });
            assert!(!state.view.needs_readback);
            let before = state.recoveries.clone();
            // No inventory is needed to reject an older VM's row. Fresh scope
            // continues to the existing inventory requirement instead.
            let error = partial_removal_candidate(&state).unwrap_err();
            assert_eq!(
                error,
                if historical {
                    unavailable
                } else {
                    "Select Refresh first."
                }
            );
            assert_eq!(state.recoveries, before);
        }
        state.view.needs_readback = true;
        assert_eq!(partial_removal_candidate(&state).unwrap_err(), unavailable);
        let retained = state.recoveries.clone();
        for selected in [None, Some("a5801e91-1083-4e79-a803-000000000002".into())] {
            state.view.selected = selected;
            assert_eq!(
                partial_removal_candidate(&state).unwrap_err(),
                "No interrupted GPU setup is recorded for this VM."
            );
            assert_eq!(state.recoveries, retained);
        }
        state.recoveries.clear();
        assert_eq!(
            partial_removal_candidate(&state).unwrap_err(),
            "No interrupted GPU setup is recorded for this VM."
        );
    }
}
