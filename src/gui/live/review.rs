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
    live_state::{review_summary, review_text},
    operation::{Kind, Operation},
    set_details, wording,
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
                let gpu_name = review_state
                    .lock()
                    .ok()
                    .and_then(|state| {
                        state
                            .view
                            .inventory
                            .as_ref()?
                            .discovery
                            .gpus
                            .iter()
                            .find(|gpu| gpu.interface == plan.desired.gpu_interface)
                            .map(|gpu| gpu.name.clone())
                    })
                    .unwrap_or_else(|| plan.desired.gpu_interface.clone());
                let memory_summary = crate::gui::live_state::memory_review(&plan);
                ui.set_execution_ready(worker_available);
                let accepted = review_state
                    .lock()
                    .map_err(|_| "Presentation state unavailable".to_owned())
                    .and_then(|mut state| {
                        let current = draft_target(ui, &state)?;
                        state.accept_review(plan.clone(), &current, committed_revision.clone())
                    });
                if let Err(error) = accepted {
                    dialog(
                        ui,
                        "error",
                        hyper_gpu_support::ui_text::operations::review::STALE_TITLE,
                        &error,
                    );
                    return;
                }
                ui.set_needs_shutdown(plan.preview.guest_downtime);
                ui.set_power_consent(wording::apply_consent(&plan).into());
                dialog(
                    ui,
                    "review",
                    if reinstall {
                        hyper_gpu_support::ui_text::operations::repair::review::TITLE
                    } else {
                        hyper_gpu_support::ui_text::operations::apply::review::TITLE
                    },
                    &format!(
                        "{}\nPhysical GPU: {gpu_name}\n{memory_summary}\n\nThese changes are still pending. Apply confirms this review; Cancel keeps your edits. Windows will request approval before changes. Settings are saved only after the required outcome checks pass.",
                        review_summary(&plan)
                    ),
                );
                set_details(ui, &review_text(&plan));
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
