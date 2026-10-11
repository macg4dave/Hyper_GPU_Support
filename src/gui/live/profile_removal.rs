use super::*;

#[cfg(windows)]
pub(super) fn review_forget_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let target = {
        let mut state = state.lock().map_err(|_| "Presentation state unavailable")?;
        state.cancel_review();
        state.forget_target()?
    };
    let reviewed_state = state.clone();
    background(
        ui,
        busy,
        "Review inactive pairing removal",
        move || {
            Ok((
                hyper_gpu_support::runner::review_removal(&target)?,
                protected_worker_available(),
            ))
        },
        move |ui, result| match result {
            Ok((review, available)) => {
                let name = {
                    let mut state = match reviewed_state.lock() {
                        Ok(state) => state,
                        Err(_) => return,
                    };
                    if let Err(error) = state.accept_forget_review(review.clone()) {
                        dialog(ui, "error", "Review pairing removal again", &error);
                        return;
                    }
                    state
                        .view
                        .vm(&review.pair.vm_id)
                        .map_or(review.pair.vm_id.clone(), |vm| vm.name.clone())
                };
                ui.set_execution_ready(available);
                ui.set_needs_shutdown(false);
                dialog(
                    ui,
                    "forget-review",
                    "Review Forget pairing",
                    &format!(
                        "Virtual machine: {name}\nVM ID: {}\nGPU: {}\n\nRemove this VM's product authorization, saved GPU settings and managed setup/verification history. Other profiles, the Hyper-V VM, disks, checkpoints, guest driver files and shared package artifacts are retained. This requires GPU support already removed and the VM off. No guest credentials, driver uninstall or VM power changes are performed.\n\nForget pairing confirms this separate review; Windows requests approval before removing protected records. Cancel keeps all records. Afterwards this VM remains discoverable and requires a new reviewed enrollment/setup to enable support.",
                        review.pair.vm_id, review.pair.gpu_interface
                    ),
                );
            }
            Err(error) => dialog(
                ui,
                "error",
                "Pairing removal unavailable",
                &hyper_gpu_support::reporting::operator_error("Pairing removal review", &error),
            ),
        },
    )
}

#[cfg(windows)]
pub(super) fn forget_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
) -> Result<(), String> {
    let approved = state
        .lock()
        .map_err(|_| "Presentation state unavailable")?
        .reviewed_forget()?;
    let finished_state = state.clone();
    let refresh_busy = busy.clone();
    let source = source.clone();
    let progress = progress_to_ui(ui.as_weak());
    background(
        ui,
        busy,
        "Remove inactive pairing and profile",
        move || hyper_gpu_support::worker::forget_pair(approved, progress).map(|_| ()),
        move |ui, result| {
            if !with_state(ui, &finished_state, |state| {
                state.cancel_review();
                state.view.needs_readback = true;
                Ok(())
            }) {
                return;
            }
            ui.set_execution_ready(false);
            match result {
                Ok(()) => {
                    if let Err(error) = refresh(ui, &finished_state, &refresh_busy, &source, false)
                    {
                        dialog(ui, "error", "Removal completed; Refresh required", &error);
                    } else {
                        ui.set_notice("Pairing, saved settings and managed history removal verified. The VM and guest files remain. A new enrollment/setup requires review.".into());
                    }
                }
                Err(error) => dialog(
                    ui,
                    "error",
                    "Pairing removal did not finish",
                    &format!(
                        "{}\nSome profile records may have been removed. Refresh and review enrollment/profile recovery before continuing; do not repeat removal with a new request.",
                        hyper_gpu_support::reporting::operator_error("Pairing removal", &error)
                    ),
                ),
            }
        },
    )
}
