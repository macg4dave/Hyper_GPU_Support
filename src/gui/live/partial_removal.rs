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
    if state.view.needs_readback {
        return Err(
            "Current VM status is unavailable. Retry the status update before reviewing removal."
                .into(),
        );
    }
    let record = state.recovery.as_ref().ok_or("Refresh recovery first")?;
    if matches!(
        record.phase,
        hyper_gpu_support::configuration_store::SavePhase::Admitted
    ) {
        return Err("This attempt stopped before VM/GPU changes. Use Check recorded outcome to confirm its unchanged starting state and end recovery.".into());
    }
    let inventory = state
        .view
        .inventory
        .as_ref()
        .ok_or("Refresh inventory first")?;
    let observed = state
        .view
        .vm(&record.target.vm_id)
        .ok_or("Recovery VM missing; Refresh")?;
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
                .ok_or("Recovery GPU missing; Refresh")?
                .clone(),
            journal: inventory
                .managed
                .get(&record.target.vm_id)
                .and_then(Option::as_ref)
                .ok_or("Recovery journal missing; Refresh")?
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
pub(super) fn review_reset(ui: &AppWindow, state: &SharedState) -> Result<(), String> {
    let mut state = state.lock().map_err(|_| "Presentation state unavailable")?;
    state.reset_review = None;
    let (revision, approved) = partial_removal_candidate(&state)?;
    let record = state.recovery.as_ref().ok_or("Refresh recovery first")?;
    let host_action = if approved.observed.gpus.is_empty() {
        "Keep this VM unattached and confirm its recorded starting host settings. No attachment removal is required."
    } else {
        "Remove the partial attachment and its allocations. Restore the compatibility settings recorded before the interrupted change."
    };
    let body = format!(
        "VM GUID: {}\nGPU: {}\nGPU interface: {}\n\n{host_action} Selecting Remove partial GPU setup also approves graceful shutdown of a running VM and leaving it off. No guest sign-in or driver installation is performed.\n\nKeep the saved settings, guest files and archived failure evidence. Invalidate old setup eligibility so a later setup performs fresh checks. After removal, review the next step here. A fresh enrollment may require deleting the inactive app profile first; deletion has its own review and confirmation.\n\nStarting compatibility settings to restore: low MMIO {} MB, high MMIO {} MB, cache types {}, automatic checkpoints {}. This does not complete the interrupted allocation or save its requested settings.",
        record.target.vm_id,
        approved.gpu.name,
        record.target.gpu_interface,
        record.initial.settings.low_mmio,
        record.initial.settings.high_mmio,
        record.initial.settings.cache_types,
        record.initial.settings.automatic_checkpoints
    );
    let heading =
        hyper_gpu_support::ui_text::operations::action::review_prompt(&approved.observed.name);
    ui.set_needs_shutdown(approved.observed.power == hyper_gpu_support::model::Power::Running);
    ui.set_power_consent(
        "Selecting Remove partial GPU setup approves graceful shutdown of this VM and leaving it off.".into(),
    );
    state.reset_review = Some((revision, approved));
    dialog(
        ui,
        "reset-review",
        hyper_gpu_support::ui_text::operations::partial_removal::review::TITLE,
        &body,
    );
    ui.set_recovery_heading(heading.into_string().into());
    Ok(())
}

#[cfg(windows)]
pub(super) fn reset_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let (operation, revision, approved) = {
        let state = state.lock().map_err(|_| "Presentation state unavailable")?;
        let record = state.recovery.as_ref().ok_or("Refresh recovery first")?;
        let (revision, approved) = state
            .reset_review
            .clone()
            .ok_or("Review partial removal first")?;
        if revision
            != hyper_gpu_support::configuration_store::Revision::of(
                &serde_json::to_vec(record).map_err(|e| e.to_string())?,
            )
        {
            return Err("Recovery changed; review partial removal again".into());
        }
        (record.operation_id.clone(), revision, approved)
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
            hyper_gpu_support::ui_text::operations::partial_removal::TITLE,
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
                    hyper_gpu_support::ui_text::operations::partial_removal::completed::TITLE,
                    hyper_gpu_support::ui_text::operations::partial_removal::completed::BODY,
                ),
                Err(error) => dialog(ui, "error", "Partial GPU removal did not complete", &error),
            }
        },
    )
}
