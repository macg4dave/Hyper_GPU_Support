//! Draft planning and acceptance of fresh ordinary or reinstall reviews.
#[cfg(windows)]
use super::enrollment::review_enrollment;
use super::{
    SharedState,
    background::background,
    presentation::draft_target,
    source::{Source, protected_worker_available},
};
use crate::gui::{
    AppWindow, dialog, failure_dialog,
    operation::{Kind, Operation},
};
use slint::ComponentHandle;
use std::sync::{Arc, atomic::AtomicBool};

pub(super) fn review(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
) -> Result<(), String> {
    review_operation(ui, state, busy, source, false)
}

pub(super) fn review_operation(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
    reinstall: bool,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        let target = {
            let mut state = state
                .lock()
                .map_err(|_| "Live presentation state unavailable")?;
            state.enrollment_review = None;
            state.approved_plan = None;
            let target = draft_target(ui, &state)?;
            state.view.proposal_eligibility(&target)?;
            (!state.enrolled(&target)).then_some(target)
        };
        if let Some(target) = target {
            if reinstall {
                return Err(
                    "Repair / Reinstall requires an authorized VM/GPU pair. Review enrollment first.".into(),
                );
            }
            return review_enrollment(ui, state, busy, target);
        }
    }
    let (target, committed_revision, authenticate) = state
        .lock()
        .map_err(|_| "Live presentation state unavailable".to_owned())
        .and_then(|mut state| {
            state.approved_plan = None;
            draft_target(ui, &state).and_then(|target| {
                let revision = state.committed.revision(&target.vm_id)?;
                Ok((target, revision, state.require_authenticated_review))
            })
        })?;
    let expected = target.clone();
    let planning_revision = committed_revision.clone();
    let source = source.clone();
    let review_state = state.clone();
    let apply_busy = busy.clone();
    let review_ui = ui.as_weak();
    background(
        ui,
        busy,
        Operation::new(
            Kind::Plan,
            hyper_gpu_support::ui_text::operations::plan::PROGRESS_TITLE,
        ),
        move || {
            if let Some(committed) = source.committed()
                && committed.revision(&target.vm_id)? != planning_revision
            {
                return Err(
                    "Committed configuration changed externally. Draft preserved; discard and Refresh.".into(),
                );
            }
            review_stage(
                &review_ui,
                if reinstall || authenticate {
                    hyper_gpu_support::ui_text::operations::plan::AUTHENTICATE
                } else {
                    hyper_gpu_support::ui_text::operations::plan::READ_HISTORY
                },
            );
            let plan = source.plan(target, reinstall, authenticate)?;
            review_stage(
                &review_ui,
                hyper_gpu_support::ui_text::operations::plan::CHECK_WORKER,
            );
            Ok((plan, protected_worker_available()))
        },
        move |ui, result| match result {
            Ok((plan, worker_available))
                if plan.desired == expected
                    && plan.observed.vm_id == expected.vm_id
                    && plan.reinstall == reinstall =>
            {
                ui.set_execution_ready(worker_available);
                let accepted = review_state
                    .lock()
                    .map_err(|_| "Presentation state unavailable".to_owned())
                    .and_then(|mut state| {
                        let current = draft_target(ui, &state)?;
                        state.accept_review(plan.clone(), &current, committed_revision.clone())
                    });
                super::background::continue_prepared(ui, accepted, || {
                    super::apply::apply_live(ui, &review_state, &apply_busy)
                });
            }
            Ok(_) => dialog(
                ui,
                "error",
                hyper_gpu_support::ui_text::operations::plan::UNAVAILABLE_TITLE,
                hyper_gpu_support::ui_text::operations::plan::TARGET_MISMATCH,
            ),
            Err(error) => failure_dialog(
                ui,
                hyper_gpu_support::ui_text::operations::plan::UNAVAILABLE_TITLE,
                &error,
                "Your draft is preserved. No VM or guest changes were performed.",
            ),
        },
    )
}

fn review_stage(weak: &slint::Weak<AppWindow>, stage: &'static str) {
    let _ = weak.upgrade_in_event_loop(move |ui| {
        // Queued updates from an abandoned read must not touch a newer dialog.
        if ui.get_review_preparing() {
            ui.set_current_stage(stage.into());
            ui.set_transfer_fraction(-1.0);
        }
    });
}
