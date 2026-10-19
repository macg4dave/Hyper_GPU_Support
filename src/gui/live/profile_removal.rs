//! Exact-profile deletion reviews and protected record-removal completion.
use super::{
    SharedState, background::background, presentation::present, progress::progress_to_ui,
    refresh::refresh_after_result, source::protected_worker_available, state_access::with_state,
};
use crate::gui::{
    AppWindow, dialog,
    operation::{Kind, Operation},
};
use slint::ComponentHandle;
use std::sync::{Arc, atomic::AtomicBool};

#[cfg(windows)]
pub(super) fn review_forget_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let target = {
        let mut state = state
            .lock()
            .map_err(|_| hyper_gpu_support::ui_text::common::state::presentation_unavailable())?;
        state.result_notice = None;
        state.forget_target()?
    };
    let reviewed_state = state.clone();
    let delete_busy = busy.clone();
    background(
        ui,
        state,
        busy,
        Operation::new(
            Kind::Read,
            hyper_gpu_support::ui_text::operations::profile_deletion::review::progress_title(),
        ),
        move || {
            Ok((
                hyper_gpu_support::runner::review_removal(&target)?,
                protected_worker_available(),
            ))
        },
        move |ui, result| match result {
            Ok((review, available)) => {
                ui.set_execution_ready(available);
                if !with_state(ui, &reviewed_state, |state| {
                    state.validate_forget_review(&review)
                }) {
                    return;
                }
                super::background::continue_prepared(ui, Ok(()), || {
                    forget_live(ui, &reviewed_state, &delete_busy, review)
                });
            }
            Err(error) => dialog(
                ui,
                "error",
                hyper_gpu_support::ui_text::operations::profile_deletion::review::unavailable_title(
                ),
                &hyper_gpu_support::reporting::operator_error(
                    hyper_gpu_support::ui_text::operations::profile_deletion::review::failure_stage(
                    ),
                    &error,
                ),
            ),
        },
    )
}

#[cfg(windows)]
fn forget_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    approved: hyper_gpu_support::enrollment::RemovalReview,
) -> Result<(), String> {
    let finished_state = state.clone();
    let vm_id = approved.pair.vm_id.clone();
    let vm_name = ui.get_vm_name().to_string();
    let progress = progress_to_ui(ui.as_weak());
    let refresh_busy = busy.clone();
    background(
        ui,
        state,
        busy,
        Operation::new(
            Kind::Forget,
            hyper_gpu_support::ui_text::operations::profile_deletion::progress_title(),
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
                    hyper_gpu_support::ui_text::operations::profile_deletion::completed::title(),
                    &hyper_gpu_support::ui_text::operations::profile_deletion::completed::body(
                        &(vm_name),
                    )
                    .into_string(),
                ),
                Some(error) => dialog(
                    ui,
                    "error",
                    hyper_gpu_support::ui_text::operations::profile_deletion::incomplete::title(),
                    &hyper_gpu_support::ui_text::operations::profile_deletion::incomplete::body(
                        &vm_name,
                        &hyper_gpu_support::reporting::operator_error(
                            hyper_gpu_support::ui_text::operations::profile_deletion::failure_stage(
                            ),
                            &error,
                        ),
                    ),
                ),
            }
            refresh_after_result(ui, &finished_state, &refresh_busy);
        },
    )
}
