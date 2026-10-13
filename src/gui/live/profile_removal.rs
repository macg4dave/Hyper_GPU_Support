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
        Operation::new(
            Kind::Read,
            hyper_gpu_support::ui_text::operations::profile_deletion::review::PROGRESS_TITLE,
        ),
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
                        dialog(ui, "error", hyper_gpu_support::ui_text::operations::profile_deletion::review::STALE_TITLE, &error);
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
                    hyper_gpu_support::ui_text::operations::profile_deletion::review::TITLE,
                    &hyper_gpu_support::ui_text::operations::profile_deletion::review::body(
                        &(name),
                        &(review.pair.vm_id),
                        &(review.pair.gpu_interface),
                    )
                    .into_string(),
                );
            }
            Err(error) => dialog(
                ui,
                "error",
                hyper_gpu_support::ui_text::operations::profile_deletion::review::UNAVAILABLE_TITLE,
                &hyper_gpu_support::reporting::operator_error(
                    hyper_gpu_support::ui_text::operations::profile_deletion::review::FAILURE_STAGE,
                    &error,
                ),
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
    let refresh_busy = busy.clone();
    background(
        ui,
        busy,
        Operation::new(
            Kind::Read,
            hyper_gpu_support::ui_text::operations::profile_deletion::PROGRESS_TITLE,
        ),
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
                    "profile-deleted",
                    hyper_gpu_support::ui_text::operations::profile_deletion::completed::TITLE,
                    &hyper_gpu_support::ui_text::operations::profile_deletion::completed::body(
                        &(vm_name),
                    )
                    .into_string(),
                ),
                Some(error) => dialog(
                    ui,
                    "error",
                    hyper_gpu_support::ui_text::operations::profile_deletion::incomplete::TITLE,
                    &format!(
                        "{vm_name}: {}\nSome app profile records may have been removed; which records remain is unknown until read back. Use Refresh and review profile recovery before continuing; do not repeat deletion with a new request. This operation does not change the VM, GPU assignment or guest files.",
                        hyper_gpu_support::reporting::operator_error(
                            hyper_gpu_support::ui_text::operations::profile_deletion::FAILURE_STAGE,
                            &error
                        )
                    ),
                ),
            }
            refresh_after_result(ui, &finished_state, &refresh_busy);
        },
    )
}
