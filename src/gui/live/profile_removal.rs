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
        "Review app profile deletion",
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
                        dialog(ui, "error", "Review Delete profile again", &error);
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
                    "Delete this app profile?",
                    &format!(
                        "Virtual machine: {name}\nVM ID: {}\nGPU: {}\n\nDelete only this VM's app profile: its saved GPU settings, authorized VM/GPU pairing and managed setup/verification history. Keep all other app profiles, the Hyper-V VM, disks, checkpoints, guest driver files and shared application artifacts.\n\nThis action does not remove GPU support, change VM power, uninstall drivers or request guest credentials. The existing protected deletion path requires GPU support already inactive and the VM off.\n\nDelete profile confirms this review; Windows requests approval to remove the protected app records. Cancel keeps all records. The VM stays discoverable and a new reviewed setup is required to enable GPU support through this app again.",
                        review.pair.vm_id, review.pair.gpu_interface
                    ),
                );
            }
            Err(error) => dialog(
                ui,
                "error",
                "Profile deletion unavailable",
                &hyper_gpu_support::reporting::operator_error("Profile deletion review", &error),
            ),
        },
    )
}

#[cfg(windows)]
pub(super) fn forget_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let approved = state
        .lock()
        .map_err(|_| "Presentation state unavailable")?
        .reviewed_forget()?;
    let finished_state = state.clone();
    let vm_id = approved.pair.vm_id.clone();
    let vm_name = ui.get_vm_name().to_string();
    let progress = progress_to_ui(ui.as_weak());
    background(
        ui,
        busy,
        "Delete app profile",
        move || hyper_gpu_support::worker::forget_pair(approved, progress).map(|_| ()),
        move |ui, result| {
            let mut failure = None;
            if !with_state(ui, &finished_state, |state| {
                failure = state.finish_profile_deletion(&vm_id, result).err();
                present(ui, state, failure.is_none());
                Ok(())
            }) {
                return;
            }
            ui.set_execution_ready(false);
            match failure {
                None => dialog(
                    ui,
                    "info",
                    "App profile deleted",
                    &format!(
                        "{vm_name}: deletion of this app's saved settings, authorization and managed history was verified. The Hyper-V VM, disks, checkpoints, guest files and other app profiles remain. No GPU or power changes were performed. Use Refresh for current inventory before reviewing a new setup."
                    ),
                ),
                Some(error) => dialog(
                    ui,
                    "error",
                    "Profile deletion did not finish",
                    &format!(
                        "{vm_name}: {}\nSome app profile records may have been removed; which records remain is unknown until read back. Use Refresh and review profile recovery before continuing; do not repeat deletion with a new request. This operation does not change the VM, GPU assignment or guest files.",
                        hyper_gpu_support::reporting::operator_error("Profile deletion", &error)
                    ),
                ),
            }
        },
    )
}
