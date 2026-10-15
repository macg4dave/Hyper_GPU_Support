//! Live GUI orchestration over shared state and protected operations.
use super::{
    AppWindow, GpuRow, dialog, failure_dialog,
    live_state::{OperationReadback, State, review_summary, review_text},
    model,
    operation::{Kind, Operation},
    result_notice::ResultNotice,
    set_details, wording,
};
use hyper_gpu_support::model::{Resource, Target};
use slint::{ComponentHandle, Model};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
pub(super) mod polling;
pub(super) mod presentation;
mod progress;
mod progress_state;
mod source;
use presentation::{
    draft_target, observed_row, pending_summary, present, present_actions, resource_controls,
    update_model,
};
#[cfg(windows)]
use progress::progress_to_ui;
pub(super) use source::Source;
use source::{discover_status, protected_worker_available};

type SharedState = Arc<Mutex<State>>;

#[cfg(windows)]
mod profile_removal;
#[cfg(windows)]
use profile_removal::{forget_live, review_forget_live};

fn credential_request_cancelled(ui: &AppWindow, error: &str, recovery: bool) -> bool {
    #[cfg(windows)]
    if hyper_gpu_support::credentials::is_prompt_cancelled(error) {
        dialog(
            ui,
            "info",
            hyper_gpu_support::ui_text::operations::credentials::CANCELLED_TITLE,
            wording::credential_cancellation(recovery),
        );
        return true;
    }
    #[cfg(not(windows))]
    let _ = (ui, error, recovery);
    false
}

fn with_state(
    ui: &AppWindow,
    state: &SharedState,
    operation: impl FnOnce(&mut State) -> Result<(), String>,
) -> bool {
    let result = state
        .lock()
        .map_err(|_| "Live presentation state is unavailable; restart the application.".to_owned())
        .and_then(|mut state| {
            let result = operation(&mut state);
            present_actions(ui, &state);
            result
        });
    if let Err(error) = result {
        ui.set_execution_ready(false);
        ui.set_eligible(false);
        dialog(
            ui,
            "error",
            hyper_gpu_support::ui_text::operations::action::UNAVAILABLE_TITLE,
            &error,
        );
        return false;
    }
    true
}

/// A failed completion read must not replace the operation's original failure.
#[cfg(windows)]
fn with_completion_state<T>(
    ui: &AppWindow,
    state: &SharedState,
    outcome: &Result<T, String>,
    operation: impl FnOnce(&mut State) -> Result<(), String>,
) -> bool {
    let completed = with_state(ui, state, operation);
    if !completed && let Err(primary) = outcome {
        let (body, details) = super::diagnostics::completion_failure(
            ui.get_dialog_body().as_str(),
            ui.get_dialog_details().as_str(),
            primary,
        );
        ui.set_dialog_body(body.into());
        ui.set_dialog_details(details.into());
    }
    completed
}
// overwriting newer session state. Only the event loop touches Slint models.
fn background<T: Send + 'static>(
    ui: &AppWindow,
    busy: &Arc<AtomicBool>,
    operation: Operation<'_>,
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
    complete: impl FnOnce(&AppWindow, Result<T, String>) + Send + 'static,
) -> Result<(), String> {
    background_for_vm(ui, busy, operation, None, false, task, complete)
}

/// Automatic reads leave foreground admission and visible UI state untouched.
fn background_refresh<T: Send + 'static>(
    ui: &AppWindow,
    busy: &Arc<AtomicBool>,
    poll: Option<polling::Ticket>,
    operation: Operation<'_>,
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
    complete: impl FnOnce(&AppWindow, Result<T, String>) + Send + 'static,
) -> Result<(), String> {
    let Some(ticket) = poll else {
        return background(ui, busy, operation, task, complete);
    };
    let weak = ui.as_weak();
    let foreground_busy = busy.clone();
    std::thread::Builder::new()
        .name("gui-status-poll".into())
        .spawn(move || {
            let result = run_background_task(task);
            let _ = weak.upgrade_in_event_loop(move |ui| {
                if ticket.can_publish(
                    foreground_busy.load(Ordering::Acquire),
                    !ui.get_dialog_kind().is_empty() || ui.get_running(),
                ) {
                    complete(&ui, result);
                }
                // Ticket drop releases polling admission even for discarded results.
            });
        })
        .map_err(|error| format!("Cannot start status poll: {error}"))?;
    Ok(())
}

fn run_background_task<T>(task: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(task)).unwrap_or_else(|_| {
        Err("The backend task ended without a terminal outcome. Refresh and inspect protected recovery before retrying.".into())
    })
}

/// Gather protected files while still busy, without blocking Slint completion.
#[cfg(windows)]
#[allow(
    clippy::too_many_arguments,
    reason = "Keep operation state, progress context, task and completion explicit at this orchestration boundary"
)]
fn background_operation<T: Send + 'static>(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    operation: Operation<'_>,
    vm_name: Option<&str>,
    guest_access: bool,
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
    complete: impl FnOnce(&AppWindow, Result<T, String>, Result<OperationReadback, String>)
    + Send
    + 'static,
) -> Result<(), String> {
    let finished_state = state.clone();
    let refresh_busy = busy.clone();
    let readback_ui = ui.as_weak();
    background_for_vm(
        ui,
        busy,
        operation,
        vm_name,
        guest_access,
        move || {
            let outcome = run_background_task(task);
            let _ = readback_ui.upgrade_in_event_loop(|ui| {
                ui.set_current_stage("Reading saved settings and recovery records".into());
                ui.set_transfer_fraction(-1.0);
            });
            Ok((outcome, Source.operation_readback()))
        },
        move |ui, result| {
            match result {
                Ok((outcome, readback)) => complete(ui, outcome, readback),
                Err(error) => complete(ui, Err(error.clone()), Err(error)),
            }
            refresh_after_result(ui, &finished_state, &refresh_busy);
        },
    )
}

fn background_for_vm<T: Send + 'static>(
    ui: &AppWindow,
    busy: &Arc<AtomicBool>,
    operation: Operation<'_>,
    vm_name: Option<&str>,
    guest_access: bool,
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
    complete: impl FnOnce(&AppWindow, Result<T, String>) + Send + 'static,
) -> Result<(), String> {
    if busy.swap(true, Ordering::AcqRel) {
        return Err("Another GUI operation is still running.".into());
    }
    ui.set_updating(true);
    let silent =
        operation.kind.silent() || (ui.get_startup_loading() && operation.kind == Kind::Read);
    let preparing = operation.kind.review_preparation();
    ui.set_review_preparing(preparing);
    if !silent {
        ui.set_running(true);
    }
    if !silent {
        let guest_message = wording::guest_progress(
            operation.kind,
            vm_name.unwrap_or(hyper_gpu_support::ui_text::operations::selection::FALLBACK),
        );
        dialog(
            ui,
            "progress",
            operation.title,
            if guest_access {
                &guest_message
            } else if operation.kind == Kind::Apply {
                hyper_gpu_support::ui_text::operations::apply::HOST_PROGRESS
            } else if operation.kind == Kind::Reconcile {
                hyper_gpu_support::ui_text::operations::recovery::HOST_PROGRESS
            } else if preparing {
                hyper_gpu_support::ui_text::operations::plan::PROGRESS_BODY
            } else if operation.kind == Kind::SaveOnly {
                hyper_gpu_support::ui_text::operations::save_only::PROGRESS_BODY
            } else {
                hyper_gpu_support::ui_text::operations::background::READING
            },
        );
    }
    #[cfg(windows)]
    if operation.kind == Kind::Reconcile {
        progress::recovery_preparation(ui, guest_access);
    }
    let weak = ui.as_weak();
    let worker_busy = busy.clone();
    if let Err(error) = std::thread::Builder::new()
        .name("gui-backend-read".into())
        .spawn(move || {
            let result = run_background_task(task);
            let completion_busy = worker_busy.clone();
            if weak
                .upgrade_in_event_loop(move |ui| {
                    completion_busy.store(false, Ordering::Release);
                    ui.set_updating(false);
                    // Cancel abandons this read's result, including failures. It
                    // never terminates a protected effect or publishes a late plan.
                    if preparing && !ui.get_review_preparing() {
                        return;
                    }
                    ui.set_review_preparing(false);
                    if !silent {
                        ui.set_running(false);
                        ui.set_dialog_kind("".into());
                    }
                    complete(&ui, result);
                })
                .is_err()
            {
                worker_busy.store(false, Ordering::Release);
            }
        })
    {
        busy.store(false, Ordering::Release);
        ui.set_review_preparing(false);
        ui.set_updating(false);
        if !silent {
            ui.set_running(false);
            ui.set_dialog_kind("".into());
        }
        return Err(format!("Cannot start backend task: {error}"));
    }
    Ok(())
}

/// Finish presentation only; failure leaves the splash available for continuation.
pub(super) fn finish_startup(ui: &AppWindow, result: Result<(), String>) {
    match result {
        Ok(()) => ui.set_startup_loading(false),
        Err(error) => {
            ui.set_startup_error(error.into());
            ui.set_dialog_kind("".into());
        }
    }
}

fn refresh(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
    automatic: bool,
) -> Result<(), String> {
    let notice = if !automatic
        && matches!(
            ui.get_dialog_kind().as_str(),
            "status-error" | "recovery-blocked"
        ) {
        let state = state.lock().map_err(|_| "Presentation state unavailable")?;
        if let Some(notice) = &state.result_notice {
            if !notice.matches_selection(ui.get_selected_id().as_str()) {
                return Err(
                    "The result belongs to another VM. Close it before refreshing the selected VM."
                        .into(),
                );
            }
            Some(notice.clone())
        } else {
            Some(capture_result(ui))
        }
    } else {
        None
    };
    refresh_with_result(ui, state, busy, source, automatic, notice)
}

// Keep the result visible while independently updating inventory. Continuation
// buttons open new reviews; this never submits a subsequent effect.
fn capture_result(ui: &AppWindow) -> ResultNotice {
    ResultNotice {
        vm_id: ui.get_selected_id().to_string(),
        kind: ui.get_dialog_kind().to_string(),
        title: ui.get_dialog_title().to_string(),
        body: ui.get_dialog_body().to_string(),
        details: ui.get_dialog_details().to_string(),
    }
}

fn refresh_after_result(ui: &AppWindow, state: &SharedState, busy: &Arc<AtomicBool>) {
    let notice = capture_result(ui);
    if !with_state(ui, state, |state| {
        state.view.needs_readback = true;
        state.result_notice = Some(notice.clone());
        present(ui, state, false);
        Ok(())
    }) {
        return;
    }
    if let Err(error) = refresh_with_result(ui, state, busy, &Source, false, Some(notice.clone())) {
        dialog(
            ui,
            "status-error",
            &notice.title,
            &format!(
                "{}\n\nVM status update could not start: {error}\nRetry the status update before another review.",
                notice.body
            ),
        );
        ui.set_dialog_details(notice.details.into());
    }
}

fn refresh_with_result(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
    automatic: bool,
    notice: Option<ResultNotice>,
) -> Result<(), String> {
    let poll = {
        let state = state.lock().map_err(|_| "Presentation state unavailable")?;
        if automatic {
            // First-install observations remain usable until an explicit refresh.
            if !state.can_poll() {
                return Ok(());
            }
            let Some(ticket) = state.polling.begin() else {
                return Ok(());
            };
            Some(ticket)
        } else {
            state.polling.invalidate();
            None
        }
    };
    let state = state.clone();
    let source = source.clone();
    let startup = ui.get_startup_loading();
    let startup_ui = ui.as_weak();
    background_refresh(
        ui,
        busy,
        poll,
        if automatic || notice.is_some() {
            Operation::new(
                Kind::StatusRefresh,
                hyper_gpu_support::ui_text::dashboard::inventory::UPDATING,
            )
        } else {
            Operation::new(
                Kind::Read,
                hyper_gpu_support::ui_text::operations::inventory::REFRESH_TITLE,
            )
        },
        move || {
            let (inventory, warning) = if automatic {
                discover_status()?
            } else {
                source.discover()?
            };
            let mut completed = 0;
            let mut advance = || {
                completed += 1;
                if startup {
                    let progress = completed as f32 / 6.0;
                    let _ = startup_ui
                        .upgrade_in_event_loop(move |ui| ui.set_startup_progress(progress));
                }
            };
            advance();
            let committed = source.committed();
            advance();
            let recovery = source.recovery()?;
            advance();
            let import_recovery = source.import_recovery()?;
            advance();
            let enrollment_recovery = source.enrollment_recovery()?;
            advance();
            let worker_available = protected_worker_available();
            advance();
            Ok((
                inventory,
                warning,
                committed,
                recovery,
                import_recovery,
                enrollment_recovery,
                worker_available,
            ))
        },
        move |ui, result| {
            let result_notice = notice;
            let published = with_state(ui, &state, |state| {
                match result {
                    Ok((
                        inventory,
                        warning,
                        committed,
                        recovery,
                        import_recovery,
                        enrollment_recovery,
                        worker_available,
                    )) => {
                        ui.set_execution_ready(false);
                        state.enrollment_review = None;
                        state.setup_artifacts = None;
                        state.enrollment_recovery_revision = None;
                        state.approved_plan = None;
                        state.verification_plan = None;
                        state.reset_review = None;
                        // A conflicting draft blocks that editor, not fresh VM/GPU
                        // observations or readable intent for other VMs.
                        let configuration_warning =
                            committed.and_then(|committed| state.load_committed(committed).err());
                        ui.set_presenting(true);
                        let rows = inventory
                            .discovery
                            .vms
                            .iter()
                            .map(|vm| observed_row(vm, &inventory, false))
                            .collect();
                        ui.set_vms(update_model(ui.get_vms(), rows));
                        ui.set_gpu_information(update_model(ui.get_gpu_information(), inventory.discovery.gpus.iter().map(|gpu| GpuRow {
                        name: gpu.name.clone().into(),
                        description: format!("Interface: {}\nDriver: {}\n{}\nPreparation adapter: {}\nAttachment does not prove guest graphics health; sharing is not qualified.", gpu.interface, gpu.driver_version, Resource::ALL.into_iter().map(|resource| gpu.resource_text(resource)).collect::<Vec<_>>().join("\n"), if gpu.preparation_supported { "available" } else { hyper_gpu_support::ui_text::system::PREPARATION_UNSUPPORTED }).into(),
                    }).collect()));
                        ui.set_gpu_options(update_model(
                            ui.get_gpu_options(),
                            inventory
                                .discovery
                                .gpus
                                .iter()
                                .map(|gpu| gpu.name.clone().into())
                                .collect(),
                        ));
                        ui.set_presenting(false);
                        state.refresh(inventory);
                        let recovery_warning = state
                            .sync_recovery(recovery, import_recovery, enrollment_recovery)
                            .err();
                        ui.set_execution_ready(recovery_warning.is_none() && worker_available);
                        let reset = !ui.get_dirty();
                        // Bind the draft GPU by identity across provider enumeration changes.
                        if !reset {
                            ui.set_gpu_index(
                                state
                                    .view
                                    .inventory
                                    .as_ref()
                                    .and_then(|i| {
                                        i.discovery.gpus.iter().position(|g| {
                                            Some(&g.interface) == state.draft_gpu.as_ref()
                                        })
                                    })
                                    .map_or(-1, |i| i as i32),
                            );
                        }
                        present(ui, state, reset);
                        if ui.get_dirty() {
                            let validation = draft_target(ui, state);
                            ui.set_draft_valid(validation.is_ok());
                            ui.set_validation(validation.err().unwrap_or_default().into());
                        }
                        ui.set_system_summary(
                            hyper_gpu_support::ui_text::operations::inventory::SUMMARY.into(),
                        );
                        let issues = &state
                            .view
                            .inventory
                            .as_ref()
                            .ok_or("inventory unavailable")?
                            .discovery
                            .issues;
                        let mut notice = if issues.is_empty() {
                            warning.unwrap_or_default()
                        } else {
                            format!(
                                "Partial inventory: {}. Other successful observations remain available.",
                                issues
                                    .iter()
                                    .map(|issue| issue.text())
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            )
                        };
                        if let Some(warning) = &configuration_warning {
                            notice.push_str(&format!(" {warning}"));
                        }
                        if let Some(warning) = &recovery_warning {
                            notice.push_str(&format!(" {warning}"));
                        }
                        if !state.committed.errors.is_empty() {
                            notice.push_str(
                                hyper_gpu_support::ui_text::operations::inventory::SAVED_UNREADABLE,
                            );
                        }
                        if let Some(summary) =
                            presentation::recovery_summary(state, ui.get_selected_id().as_str())
                        {
                            notice.push_str(&format!(" {summary}"));
                        }
                        ui.set_inventory_details(notice.clone().into());
                        if !issues.is_empty()
                            && configuration_warning.is_none()
                            && recovery_warning.is_none()
                            && state.committed.errors.is_empty()
                        {
                            notice =
                                hyper_gpu_support::ui_text::operations::inventory::DETAILS_PROMPT
                                    .into();
                        }
                        ui.set_notice(notice.into());
                        if let Some(notice) = &result_notice {
                            if notice.matches_selection(ui.get_selected_id().as_str()) {
                                dialog(ui, &notice.kind, &notice.title, &notice.body);
                                ui.set_dialog_details(notice.details.clone().into());
                            }
                            state.result_notice = None;
                        }
                        Ok(())
                    }
                    Err(error) => {
                        state.approved_plan = None;
                        state.verification_plan = None;
                        state.enrollment_review = None;
                        state.setup_artifacts = None;
                        state.enrollment_recovery_revision = None;
                        state.reset_review = None;
                        ui.set_execution_ready(false);
                        let error = hyper_gpu_support::reporting::operator_error(
                            hyper_gpu_support::ui_text::operations::inventory::FAILURE_STAGE,
                            &error,
                        );
                        state.view.needs_readback = true;
                        if let Some(inventory) = state.view.inventory.as_ref() {
                            ui.set_vms(model(
                                inventory
                                    .discovery
                                    .vms
                                    .iter()
                                    .map(|vm| observed_row(vm, inventory, true))
                                    .collect(),
                            ));
                        }
                        present(ui, state, false);
                        ui.set_draft_valid(false);
                        ui.set_validation(
                            hyper_gpu_support::ui_text::operations::inventory::DRAFT_RETAINED
                                .into(),
                        );
                        ui.set_inventory_details(error.clone().into());
                        ui.set_notice(
                            hyper_gpu_support::ui_text::operations::inventory::REFRESH_FAILED
                                .into(),
                        );
                        ui.set_system_summary(
                            format!("Live discovery unavailable: {error}").into(),
                        );
                        if let Some(notice) = &result_notice {
                            state.result_notice = Some(notice.clone());
                            dialog(
                                ui,
                                "status-error",
                                &notice.title,
                                &notice.unavailable_body(&error),
                            );
                            ui.set_dialog_details(notice.details.clone().into());
                            Ok(())
                        } else if automatic {
                            Ok(())
                        } else {
                            Err(error)
                        }
                    }
                }
            });
            if startup {
                finish_startup(
                    ui,
                    if published {
                        Ok(())
                    } else {
                        Err(ui.get_dialog_body().to_string())
                    },
                );
            }
        },
    )
}

fn review(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
) -> Result<(), String> {
    review_operation(ui, state, busy, source, false)
}

fn review_operation(
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
                let memory_summary = super::live_state::memory_review(&plan);
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

#[cfg(windows)]
fn review_enrollment(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    target: Target,
) -> Result<(), String> {
    let reviewed_state = state.clone();
    background(
        ui,
        busy,
        Operation::new(
            Kind::Read,
            hyper_gpu_support::ui_text::operations::enrollment::review::TITLE,
        ),
        move || {
            let pending = hyper_gpu_support::runner::pending_setup()?;
            if pending.is_none() && hyper_gpu_support::runner::installed()? {
                hyper_gpu_support::runner::review_pair(&target)
                    .map(|review| (review, None, protected_worker_available()))
            } else {
                let approved = hyper_gpu_support::enrollment::PairReview {
                    vm_id: target.vm_id,
                    gpu_interface: target.gpu_interface,
                    expected: hyper_gpu_support::configuration_store::Revision::Missing,
                    operator_sid: hyper_gpu_support::setup::operator_sid()?,
                };
                approved.target()?;
                let artifacts = hyper_gpu_support::setup::source_artifacts()?;
                if let Some(record) = pending
                    && (record.review != approved || record.artifacts != artifacts)
                {
                    return Err(
                        "Interrupted first-install setup belongs to another original pair/package. Select its original VM/GPU and reviewed package; setup recovery cannot replace authority."
                            .into(),
                    );
                }
                Ok((approved, Some(artifacts), true))
            }
        },
        move |ui, result| match result {
            Ok((approved, artifacts, worker_available)) => {
                let setup = artifacts.is_some();
                let mut summary = hyper_gpu_support::ui_text::operations::enrollment::review::body(
                    &(ui.get_vm_name()),
                    &(ui.get_gpu_options()
                        .row_data(usize::try_from(ui.get_gpu_index()).unwrap_or(usize::MAX))
                        .unwrap_or_default()),
                )
                .into_string();
                if setup {
                    summary.push_str(
                        hyper_gpu_support::ui_text::operations::enrollment::INSTALL_NOTICE,
                    );
                } else {
                    summary.push_str(
                        hyper_gpu_support::ui_text::operations::enrollment::ELEVATION_NOTICE,
                    );
                }
                let mut body = format!(
                    "VM ID: {}\nGPU interface: {}\nOperator: {}\nPolicy revision: {:?}\n\nAuthorize this exact VM/GPU pair through Windows elevation, preserving all other enrolled VMs. Enrollment does not enable GPU support, change guest power, prepare drivers or save desired configuration. Your draft is retained. After fresh authority readback, Apply obtains a separate GPU operation plan.",
                    approved.vm_id,
                    approved.gpu_interface,
                    approved.operator_sid,
                    approved.expected
                );
                if let Some(pins) = &artifacts {
                    body.push_str(
                        hyper_gpu_support::ui_text::operations::enrollment::INSTALL_DETAILS,
                    );
                    for (name, hash) in pins {
                        body.push_str(
                            &hyper_gpu_support::ui_text::operations::enrollment::artifact_pin(
                                name, hash,
                            )
                            .into_string(),
                        );
                    }
                }
                if !with_state(ui, &reviewed_state, |state| {
                    state.enrollment_review = Some(approved);
                    state.setup_artifacts = artifacts;
                    Ok(())
                }) {
                    return;
                }
                ui.set_execution_ready(worker_available);
                ui.set_needs_shutdown(false);
                dialog(
                    ui,
                    "enroll-review",
                    hyper_gpu_support::ui_text::operations::enrollment::review::TITLE,
                    &summary,
                );
                set_details(ui, &body);
            }
            Err(error) => dialog(ui, "error", "Permission review unavailable", &error),
        },
    )
}

#[cfg(windows)]
fn enroll_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    _source: &Source,
) -> Result<(), String> {
    let (approved, artifacts) = {
        let mut state = state.lock().map_err(|_| "Presentation state unavailable")?;
        let approved = state
            .enrollment_review
            .take()
            .ok_or("Review exact enrollment first")?;
        let draft = draft_target(ui, &state)?;
        if draft.vm_id != approved.vm_id || draft.gpu_interface != approved.gpu_interface {
            return Err("Pair changed after enrollment review; review again".into());
        }
        let artifacts = state.setup_artifacts.take();
        (approved, artifacts)
    };
    let finished_state = state.clone();
    let refresh_busy = busy.clone();
    let progress = progress_to_ui(ui.as_weak());
    background(
        ui,
        busy,
        Operation::new(
            Kind::Read,
            hyper_gpu_support::ui_text::operations::enrollment::PROGRESS_TITLE,
        ),
        move || {
            if let Some(artifacts) = artifacts {
                hyper_gpu_support::setup::install(approved, artifacts)
            } else {
                hyper_gpu_support::worker::enroll_pair(approved, progress).map(|_| ())
            }
        },
        move |ui, result| match result {
            Ok(()) => {
                dialog(
                    ui,
                    "permission-saved",
                    hyper_gpu_support::ui_text::operations::enrollment::completed::TITLE,
                    hyper_gpu_support::ui_text::operations::enrollment::completed::BODY,
                );
                refresh_after_result(ui, &finished_state, &refresh_busy);
            }
            Err(error) => {
                if !with_state(ui, &finished_state, |state| {
                    state.view.needs_readback = true;
                    present(ui, state, false);
                    Ok(())
                }) {
                    return;
                }
                ui.set_eligible(false);
                ui.set_execution_ready(false);
                failure_dialog(
                    ui,
                    hyper_gpu_support::ui_text::operations::enrollment::incomplete::TITLE,
                    &error,
                    "Your edits are kept. Use Refresh and Review recovery before retrying this permission step. GPU setup has not been confirmed.",
                );
            }
        },
    )
}

#[cfg(windows)]
fn reconcile_enrollment_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
) -> Result<(), String> {
    let (operation_id, expected) = {
        let mut state = state.lock().map_err(|_| "Presentation state unavailable")?;
        let id = state
            .enrollment_recovery
            .as_ref()
            .ok_or("Refresh enrollment recovery first")?
            .operation_id
            .clone();
        let expected = state
            .enrollment_recovery_revision
            .take()
            .ok_or("Review authority readback first")?;
        (id, expected)
    };
    let finished_state = state.clone();
    let refresh_busy = busy.clone();
    let source = source.clone();
    let progress = progress_to_ui(ui.as_weak());
    background(
        ui,
        busy,
        Operation::new(
            Kind::Read,
            hyper_gpu_support::ui_text::operations::enrollment::recovery::PROGRESS_TITLE,
        ),
        move || hyper_gpu_support::worker::reconcile_pair(operation_id, expected, progress),
        move |ui, result| match result {
            Ok(_) => {
                if let Err(error) = refresh(ui, &finished_state, &refresh_busy, &source, false) {
                    dialog(ui, "error", hyper_gpu_support::ui_text::operations::enrollment::recovery::STATUS_UNAVAILABLE_TITLE, &error);
                }
            }
            Err(error) => {
                ui.set_execution_ready(false);
                dialog(
                    ui,
                    "error",
                    hyper_gpu_support::ui_text::operations::enrollment::recovery::RETAINED_TITLE,
                    &error,
                );
            }
        },
    )
}

#[cfg(windows)]
fn apply_live(ui: &AppWindow, state: &SharedState, busy: &Arc<AtomicBool>) -> Result<(), String> {
    if !ui.get_execution_ready() {
        return Err("The reviewed protected worker is not available.".into());
    }
    let (approved, revision) = {
        let state = state.lock().map_err(|_| "Presentation state unavailable")?;
        let reviewed = state.reviewed_plan(&draft_target(ui, &state)?)?;
        (reviewed.plan, reviewed.revision)
    };
    let selected = approved.desired.clone();
    let finished_state = state.clone();
    let progress = progress_to_ui(ui.as_weak());
    let vm_name = approved.observed.name.clone();
    let result_vm_name = vm_name.clone();
    let setup_required = approved
        .preview
        .actions
        .contains(&hyper_gpu_support::workflow::PreviewAction::Verify);
    background_operation(
        ui,
        state,
        busy,
        Operation::new(
            Kind::Apply,
            hyper_gpu_support::ui_text::operations::apply::PROGRESS_TITLE,
        ),
        Some(&vm_name),
        setup_required,
        move || {
            let credential = if approved.preview.credentials_required {
                Some(
                    hyper_gpu_support::credentials::read(&approved.desired.vm_id)?.map_or_else(
                        || {
                            hyper_gpu_support::credentials::prompt_named(
                                &approved.desired.vm_id,
                                &approved.observed.name,
                                windows::Win32::Foundation::HWND::default(),
                            )
                        },
                        Ok,
                    )?,
                )
            } else {
                None
            };
            let shutdown = approved.preview.guest_downtime;
            hyper_gpu_support::worker::apply(approved, revision, shutdown, credential, progress)
        },
        move |ui, result, readback| match result {
            Ok(outcome) => {
                let mut published = outcome.saved;
                if !with_state(ui, &finished_state, |state| {
                    let status = state.finish_apply(
                        &selected,
                        &outcome.operation_id,
                        outcome.saved,
                        readback,
                    );
                    published = state.publication_confirmed(&selected);
                    ui.set_dirty(state.view.draft.is_some());
                    present(ui, state, !ui.get_dirty() && published);
                    status
                }) {
                    return;
                }
                if published {
                    dialog(
                        ui,
                        "info",
                        if setup_required {
                            hyper_gpu_support::ui_text::operations::setup::completed::TITLE
                        } else {
                            hyper_gpu_support::ui_text::operations::apply::completed::TITLE
                        },
                        &if setup_required {
                            format!(
                                "{result_vm_name}: GPU support {}. VRAM request: {}. Hardware rendering passed, graceful final shutdown was confirmed, and your settings were saved. The VM is off; start it through Hyper-V when ready.",
                                if selected.enabled {
                                    hyper_gpu_support::ui_text::common::support::ENABLED_SENTENCE
                                } else {
                                    hyper_gpu_support::ui_text::common::support::DISABLED_SENTENCE
                                },
                                selected.vram.as_ref().map_or_else(
                                    || hyper_gpu_support::ui_text::common::allocation::PROVIDER_DEFAULT.to_owned(),
                                    |v| super::memory::format_gb(v.optimal)
                                )
                            )
                        } else {
                            hyper_gpu_support::ui_text::operations::apply::completed::body(
                                &(result_vm_name),
                            )
                            .into_string()
                        },
                    );
                } else {
                    dialog(
                        ui,
                        "save",
                        if setup_required {
                            hyper_gpu_support::ui_text::operations::setup::unsaved::TITLE
                        } else {
                            hyper_gpu_support::ui_text::operations::apply::unsaved::TITLE
                        },
                        &format!(
                            "{}{}\nThe verified intent is retained separately from committed configuration. Retry saving only; do not repeat Apply.",
                            if setup_required {
                                hyper_gpu_support::ui_text::operations::setup::unsaved::effect_summary(&(result_vm_name)).into_string()
                            } else {
                                hyper_gpu_support::ui_text::operations::apply::unsaved::effect_summary(&(result_vm_name)).into_string()
                            },
                            outcome.save_error.unwrap_or_default(),
                        ),
                    );
                }
            }
            Err(error) => {
                if credential_request_cancelled(ui, &error, false) {
                    with_state(ui, &finished_state, |state| {
                        state.approved_plan = None;
                        Ok(())
                    });
                    return;
                }
                let stages: Vec<_> = ui.get_stages().iter().collect();
                let reportable = ui.get_stage_reportable();
                if !with_completion_state(
                    ui,
                    &finished_state,
                    &Err::<(), _>(error.clone()),
                    |state| {
                        state.view.needs_readback = true;
                        state.approved_plan = None;
                        state.require_authenticated_review = true;
                        let status = state.finish_readback(readback);
                        present(ui, state, false);
                        status
                    },
                ) {
                    return;
                }
                ui.set_stages(model(stages));
                ui.set_stage_reportable(reportable);
                let last_stage = ui.get_stages().iter().last().map(|stage| stage.to_string());
                dialog(
                    ui,
                    "error",
                    if setup_required {
                        hyper_gpu_support::ui_text::operations::setup::incomplete::TITLE
                    } else {
                        hyper_gpu_support::ui_text::operations::apply::incomplete::TITLE
                    },
                    &error,
                );
                let reason = hyper_gpu_support::reporting::operator_error(
                    hyper_gpu_support::ui_text::operations::apply::FAILURE_STAGE,
                    &error,
                );
                ui.set_dialog_body(format!(
                    "{reason}\n{}\nYour draft is retained. Changes may have started; the final VM state has not been confirmed. Use Refresh and inspect recovery before trying again. Do not repeat Apply while recovery is required.",
                    last_stage.map_or_else(|| hyper_gpu_support::ui_text::operations::apply::STAGE_ABSENT.into(), |stage| format!("Last reported step: {stage}")),
                ).into());
            }
        },
    )
}
#[cfg(windows)]
fn save_only_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let record = state
        .lock()
        .map_err(|_| "Presentation state unavailable")?
        .save_only_record()?;
    let finished_state = state.clone();
    let target = record.target.clone();
    let progress = progress_to_ui(ui.as_weak());
    background_operation(
        ui,
        state,
        busy,
        Operation::new(
            Kind::SaveOnly,
            hyper_gpu_support::ui_text::operations::save_only::PROGRESS_TITLE,
        ),
        None,
        false,
        move || hyper_gpu_support::worker::save_only(record.operation_id, progress),
        move |ui, result, readback| {
            let mut readback = Some(readback);
            if result.is_err()
                && !with_completion_state(ui, &finished_state, &result, |state| {
                    let status = state.finish_readback(
                        readback
                            .take()
                            .ok_or("Completion readback was already consumed")?,
                    );
                    present(ui, state, false);
                    status
                })
            {
                return;
            }
            match result {
                Ok(outcome) if outcome.saved => {
                    if !with_state(ui, &finished_state, |state| {
                        let status = state.finish_apply(
                            &target,
                            &outcome.operation_id,
                            true,
                            readback
                                .take()
                                .ok_or("Completion readback was already consumed")?,
                        );
                        ui.set_dirty(state.view.draft.is_some());
                        present(ui, state, !ui.get_dirty());
                        status
                    }) {
                        return;
                    }
                    dialog(
                        ui,
                        "info",
                        hyper_gpu_support::ui_text::operations::save_only::completed::TITLE,
                        hyper_gpu_support::ui_text::operations::save_only::completed::BODY,
                    );
                }
                Ok(_) => dialog(
                    ui,
                    "error",
                    hyper_gpu_support::ui_text::operations::save_only::unsaved::TITLE,
                    hyper_gpu_support::ui_text::operations::save_only::unsaved::BODY,
                ),
                Err(error) => failure_dialog(
                    ui,
                    hyper_gpu_support::ui_text::operations::save_only::blocked::TITLE,
                    &error,
                    "The protected recovery record remains; no GPU operation was repeated.",
                ),
            }
        },
    )
}
#[cfg(windows)]
fn review_verification(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let target = {
        let state = state.lock().map_err(|_| "Presentation state unavailable")?;
        if state.view.draft.is_some() {
            return Err(
                "Discard or Apply the draft before reviewing a separate graphics check.".into(),
            );
        }
        state.editor_eligibility(ui.get_selected_id().as_str())?;
        state.target(ui.get_selected_id().as_str())?
    };
    let review_state = state.clone();
    background(
        ui,
        busy,
        Operation::new(
            Kind::VerifyPlan,
            hyper_gpu_support::ui_text::operations::verify::review::PROGRESS_TITLE,
        ),
        move || {
            let value = hyper_gpu_support::runner::submit(hyper_gpu_support::runner::request(
                hyper_gpu_support::runner::Operation::VerifyPlan,
                Some(target),
                None,
            ))?;
            serde_json::from_value::<hyper_gpu_support::workflow::VerificationPlan>(value)
                .map_err(|e| e.to_string())
                .map(|plan| (plan, protected_worker_available()))
        },
        move |ui, result| match result {
            Ok((plan, worker_available)) => {
                ui.set_execution_ready(worker_available);
                ui.set_needs_shutdown(plan.observed.power == hyper_gpu_support::model::Power::Off);
                ui.set_power_consent(wording::verify_consent(&plan).into());
                let body = wording::verify_review(&plan);
                let details = wording::verify_details(&plan);
                if !with_state(ui, &review_state, |state| {
                    state.verification_plan = Some(plan);
                    Ok(())
                }) {
                    return;
                }
                dialog(
                    ui,
                    "verify-review",
                    hyper_gpu_support::ui_text::operations::verify::review::TITLE,
                    &body,
                );
                set_details(ui, &details);
            }
            Err(error) => dialog(ui, "error", "Verify GPU setup unavailable", &error),
        },
    )
}
#[cfg(windows)]
fn verify_live(ui: &AppWindow, state: &SharedState, busy: &Arc<AtomicBool>) -> Result<(), String> {
    let approved = state
        .lock()
        .map_err(|_| "Presentation state unavailable")?
        .verification_plan
        .clone()
        .ok_or("Review verification first")?;
    let shutdown = approved.observed.power == hyper_gpu_support::model::Power::Off;
    let progress = progress_to_ui(ui.as_weak());
    let finished_state = state.clone();
    let vm_name = approved.observed.name.clone();
    let result_vm_name = vm_name.clone();
    background_operation(
        ui,
        state,
        busy,
        Operation::new(
            Kind::Verify,
            hyper_gpu_support::ui_text::operations::verify::title().as_str(),
        ),
        Some(&vm_name),
        true,
        move || {
            let credential = hyper_gpu_support::credentials::read(&approved.target.vm_id)?
                .map_or_else(
                    || {
                        hyper_gpu_support::credentials::prompt_named(
                            &approved.target.vm_id,
                            &approved.observed.name,
                            windows::Win32::Foundation::HWND::default(),
                        )
                    },
                    Ok,
                )?;
            hyper_gpu_support::worker::verify(approved, shutdown, credential, progress)
        },
        move |ui, result, readback| {
            if let Err(error) = &result
                && credential_request_cancelled(ui, error, false)
            {
                with_state(ui, &finished_state, |state| {
                    state.verification_plan = None;
                    Ok(())
                });
                return;
            }
            if !with_completion_state(ui, &finished_state, &result, |state| {
                state.verification_plan = None;
                state.view.needs_readback = true;
                let status = state.finish_readback(readback);
                present(ui, state, false);
                status
            }) {
                return;
            }
            match result {
                Ok(outcome) if outcome.finished => dialog(
                    ui,
                    "info",
                    hyper_gpu_support::ui_text::operations::verify::completed::TITLE,
                    &hyper_gpu_support::ui_text::operations::verify::completed::body(
                        &(result_vm_name),
                    )
                    .into_string(),
                ),
                Ok(_) => dialog(
                    ui,
                    "error",
                    hyper_gpu_support::ui_text::operations::verify::unknown::TITLE,
                    &hyper_gpu_support::ui_text::operations::verify::unknown::body(
                        &(result_vm_name),
                    )
                    .into_string(),
                ),
                Err(error) => failure_dialog(
                    ui,
                    hyper_gpu_support::ui_text::operations::verify::incomplete::TITLE,
                    &error,
                    &format!(
                        "{result_vm_name}: The check did not complete. Use Refresh and Review recovery before another operation; prior successful checks do not establish current health."
                    ),
                ),
            }
        },
    )
}
#[cfg(windows)]
fn reconcile_live(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
) -> Result<(), String> {
    let record = state
        .lock()
        .map_err(|_| "Presentation state unavailable")?
        .recovery
        .clone()
        .ok_or("No reviewed operation recovery is loaded")?;
    let progress = progress_to_ui(ui.as_weak());
    let finished_state = state.clone();
    let target = record.target.clone();
    let publication = record.publication_required;
    let (vm_name, restored_candidate) = {
        let s = state.lock().map_err(|_| "Presentation state unavailable")?;
        let vm = s
            .view
            .vm(&target.vm_id)
            .ok_or("Recovery VM unavailable; Refresh first")?;
        (vm.name.clone(), s.recovery_without_guest(&record)?)
    };
    let prompt_name = vm_name.clone();
    background_operation(
        ui,
        state,
        busy,
        Operation::new(
            Kind::Reconcile,
            hyper_gpu_support::ui_text::operations::recovery::PROGRESS_TITLE,
        ),
        Some(&vm_name),
        record.requires_graphics() && !restored_candidate,
        move || {
            let graphics = record.requires_graphics() && !restored_candidate;
            let credential = if graphics && !restored_candidate {
                Some(
                    hyper_gpu_support::credentials::read(&record.target.vm_id)?.map_or_else(
                        || {
                            hyper_gpu_support::credentials::prompt_named(
                                &record.target.vm_id,
                                &prompt_name,
                                windows::Win32::Foundation::HWND::default(),
                            )
                        },
                        Ok,
                    )?,
                )
            } else {
                None
            };
            hyper_gpu_support::worker::reconcile(
                record.operation_id,
                graphics,
                credential,
                progress,
            )
        },
        move |ui, result, readback| {
            if let Err(error) = &result
                && credential_request_cancelled(ui, error, true)
            {
                return;
            }
            if !with_completion_state(ui, &finished_state, &result, |state| {
                let status = if let Ok(outcome) = &result
                    && publication
                    && outcome.operation.is_some()
                {
                    state.finish_apply(&target, &outcome.operation_id, outcome.saved, readback)
                } else {
                    state.finish_readback(readback)
                };
                ui.set_dirty(state.view.draft.is_some());
                present(ui, state, !ui.get_dirty());
                status
            }) {
                return;
            }
            match result {
                Ok(outcome) if outcome.finished && outcome.operation.is_none() => dialog(
                    ui,
                    "recovery-ended",
                    hyper_gpu_support::ui_text::operations::recovery::ended::TITLE,
                    hyper_gpu_support::ui_text::operations::recovery::ended::BODY,
                ),
                Ok(outcome) if outcome.finished => dialog(
                    ui,
                    "info",
                    hyper_gpu_support::ui_text::operations::recovery::confirmed::TITLE,
                    hyper_gpu_support::ui_text::operations::recovery::confirmed::BODY,
                ),
                Ok(outcome) => dialog(
                    ui,
                    "save",
                    hyper_gpu_support::ui_text::operations::recovery::unsaved::TITLE,
                    &format!(
                        "{}\nRetry configuration saving only.",
                        outcome.save_error.unwrap_or_default()
                    ),
                ),
                Err(error) => {
                    dialog(
                        ui,
                        "error",
                        hyper_gpu_support::ui_text::operations::recovery::blocked::TITLE,
                        &error,
                    );
                    ui.set_dialog_kind("recovery-blocked".into());
                    ui.set_dialog_body(format!(
                        "{}\nReview partial removal here. For an interrupted first setup or partial enable, Remove partial GPU setup can restore its recorded starting host configuration. If that review cannot establish safe removal, recovery stays blocked and its explanation names the unavailable facts.",
                        hyper_gpu_support::reporting::operator_error(hyper_gpu_support::ui_text::operations::recovery::FAILURE_STAGE, &error),
                    ).into());
                }
            }
        },
    )
}

#[cfg(windows)]
fn partial_removal_candidate(
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
fn review_reset(ui: &AppWindow, state: &SharedState) -> Result<(), String> {
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
        "Virtual machine: {}\nVM GUID: {}\nGPU: {}\nGPU interface: {}\n\n{host_action} A running VM will be gracefully shut down with your separate consent and left off. No guest sign-in or driver installation is performed.\n\nKeep the saved settings, guest files and archived failure evidence. Invalidate old setup eligibility so a later setup performs fresh checks. After removal, review the next step here. A fresh enrollment may require deleting the inactive app profile first; deletion has its own review and confirmation.\n\nStarting compatibility settings to restore: low MMIO {} MB, high MMIO {} MB, cache types {}, automatic checkpoints {}. This does not complete the interrupted allocation or save its requested settings.",
        approved.observed.name,
        record.target.vm_id,
        approved.gpu.name,
        record.target.gpu_interface,
        record.initial.settings.low_mmio,
        record.initial.settings.high_mmio,
        record.initial.settings.cache_types,
        record.initial.settings.automatic_checkpoints
    );
    ui.set_needs_shutdown(approved.observed.power == hyper_gpu_support::model::Power::Running);
    ui.set_power_consent(
        "I approve graceful shutdown of this recovery VM and leaving it off.".into(),
    );
    state.reset_review = Some((revision, approved));
    dialog(
        ui,
        "reset-review",
        hyper_gpu_support::ui_text::operations::partial_removal::review::TITLE,
        &body,
    );
    Ok(())
}

#[cfg(windows)]
fn reset_live(ui: &AppWindow, state: &SharedState, busy: &Arc<AtomicBool>) -> Result<(), String> {
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
            Kind::Read,
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
#[cfg(not(windows))]
fn review_verification(_: &AppWindow, _: &SharedState, _: &Arc<AtomicBool>) -> Result<(), String> {
    Err("Verify GPU setup requires Windows".into())
}
#[cfg(not(windows))]
fn verify_live(_: &AppWindow, _: &SharedState, _: &Arc<AtomicBool>) -> Result<(), String> {
    Err("Verify GPU setup requires Windows".into())
}
#[cfg(not(windows))]
fn reconcile_live(_: &AppWindow, _: &SharedState, _: &Arc<AtomicBool>) -> Result<(), String> {
    Err("Reconciliation requires Windows".into())
}
#[cfg(not(windows))]
fn apply_live(_: &AppWindow, _: &SharedState, _: &Arc<AtomicBool>) -> Result<(), String> {
    Err("GPU operations require Windows".into())
}
#[cfg(not(windows))]
fn save_only_live(_: &AppWindow, _: &SharedState, _: &Arc<AtomicBool>) -> Result<(), String> {
    Err("Protected publication requires Windows".into())
}

pub(super) fn run(ui: AppWindow, source: Source) -> Result<(), Box<dyn std::error::Error>> {
    ui.set_vms(model(Vec::new()));
    ui.set_selected_id("".into());
    ui.set_gpu_information(model(Vec::new()));
    ui.set_gpu_options(model(Vec::new()));
    ui.set_values(model(vec!["".into(); 12]));
    ui.set_eligible(false);
    ui.set_system_summary(hyper_gpu_support::ui_text::operations::inventory::INITIAL_NOTICE.into());
    let busy = Arc::new(AtomicBool::new(false));
    let state = Arc::new(Mutex::new(State::default()));
    if let Ok(mut state) = state.lock() {
        state.view.selected = super::selection::load();
    }
    let weak = ui.as_weak();
    let selection_state = state.clone();
    ui.on_select_vm(move |id| {
        if let Some(ui) = weak.upgrade() {
            if ui.get_presenting() {
                return;
            }
            with_state(&ui, &selection_state, |state| {
                if ui.get_running() || id == ui.get_selected_id() {
                    return Ok(());
                }
                if ui.get_dirty() {
                    state.pending_selection = Some(id.to_string());
                    dialog(
                        &ui,
                        "switch",
                        hyper_gpu_support::ui_text::operations::draft::switch::TITLE,
                        hyper_gpu_support::ui_text::operations::draft::switch::BODY,
                    );
                } else {
                    state.view.selected = Some(id.to_string());
                    present(&ui, state, true);
                }
                Ok(())
            });
        }
    });
    let weak = ui.as_weak();
    let edit_state = state.clone();
    ui.on_edited(move || {
        if let Some(ui) = weak.upgrade() {
            if ui.get_presenting() {
                return;
            }
            with_state(&ui, &edit_state, |state| {
                if let Some(index) = state.fixed_gpu_index()
                    && ui.get_gpu_index() != index
                {
                    ui.set_gpu_index(index);

                    return Ok(());
                }
                state.approved_plan = None;
                state.enrollment_review = None;
                state.setup_artifacts = None;
                state.draft_gpu = state
                    .view
                    .inventory
                    .as_ref()
                    .and_then(|i| {
                        usize::try_from(ui.get_gpu_index())
                            .ok()
                            .and_then(|index| i.discovery.gpus.get(index))
                    })
                    .map(|g| g.interface.clone());
                resource_controls(&ui, state);
                let baseline = state.target(ui.get_selected_id().as_str())?;
                let raw_values = ui
                    .get_values()
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>();
                let changed = state.draft_changed(
                    &baseline,
                    ui.get_gpu_enabled(),
                    state.draft_gpu.as_deref(),
                    &raw_values,
                );
                ui.set_dirty(changed);
                ui.set_pending(
                    if changed {
                        pending_summary(ui.get_gpu_enabled(), Some(baseline.enabled), &raw_values)
                    } else {
                        hyper_gpu_support::ui_text::dashboard::draft::ABSENT.to_owned()
                    }
                    .into(),
                );
                if !changed {
                    state.view.draft = None;
                    state.draft_gpu = None;
                }
                match draft_target(&ui, state) {
                    Ok(target) => {
                        if changed {
                            state.view.draft = Some(target);
                        }
                        ui.set_draft_valid(true);
                        ui.set_validation("".into());
                    }
                    Err(error) => {
                        // A marker retains the VM even for invalid text; raw edits stay in the UI.
                        if changed {
                            state.view.draft = Some(baseline);
                        }
                        ui.set_draft_valid(false);
                        ui.set_validation(
                            hyper_gpu_support::reporting::operator_error(
                                hyper_gpu_support::ui_text::operations::validation::STAGE,
                                &error,
                            )
                            .into(),
                        );
                    }
                }
                Ok(())
            });
        }
    });
    let weak = ui.as_weak();
    let memory_state = state.clone();
    ui.on_memory_edited(move |position| {
        let Some(ui) = weak.upgrade() else {
            return;
        };
        if ui.get_presenting() || ui.get_running() || !ui.get_dialog_kind().is_empty() {
            return;
        }
        let values = ui
            .get_values()
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>();
        let selection = memory_state
            .lock()
            .map_err(|_| "Presentation state unavailable".to_owned())
            .and_then(|state| {
                state.memory_values(
                    ui.get_selected_id().as_str(),
                    ui.get_gpu_index(),
                    &values,
                    position,
                )
            });
        match selection {
            Ok(values) => {
                // The shared presentation helper changes only VRAM. The normal
                // draft callback revalidates all retained compatibility intent.
                ui.set_values(model(values.into_iter().map(Into::into).collect()));
                // The state guard above is released before invoking this callback.
                ui.invoke_edited();
            }
            Err(error) => {
                // This is application-owned numeric validation text, with no
                // adapter output. Keep the selected draft/position unchanged.
                ui.set_memory_hint(error.into());
            }
        }
    });
    let weak = ui.as_weak();
    ui.on_memory_text_acceptable(super::memory::acceptable_input);
    let validation_weak = ui.as_weak();
    ui.on_memory_text_valid(move |text| {
        validation_weak
            .upgrade()
            .is_some_and(|ui| super::memory::valid_input(&text, &ui.get_memory_capacity()))
    });
    let numeric_weak = ui.as_weak();
    let numeric_state = state.clone();
    ui.on_memory_text_edited(move |text| {
        let Some(ui) = numeric_weak.upgrade() else {
            return;
        };
        if ui.get_presenting()
            || ui.get_updating()
            || ui.get_running()
            || !ui.get_dialog_kind().is_empty()
        {
            return;
        }
        let values = ui
            .get_values()
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>();
        let selection = numeric_state
            .lock()
            .map_err(|_| "Presentation state unavailable".to_owned())
            .and_then(|state| {
                state.memory_text_values(
                    ui.get_selected_id().as_str(),
                    ui.get_gpu_index(),
                    &values,
                    &text,
                )
            });
        match selection {
            Ok(values) => {
                ui.set_values(model(values.into_iter().map(Into::into).collect()));
                ui.invoke_edited();
            }
            Err(error) => ui.set_memory_hint(error.into()),
        }
    });
    let action_state = state.clone();
    let action_busy = busy.clone();
    let action_source = source.clone();
    ui.on_action(move |action| {
        let Some(ui) = weak.upgrade() else { return; };
        if ui.get_running() && !(action == "close" && ui.get_review_preparing()) { return; }
        if let Ok(state) = action_state.lock() {
            state.polling.invalidate();
        }
        if ui.get_dialog_kind().is_empty() { ui.set_focus_return_action(action.clone()); }
        if action_busy.load(Ordering::Acquire)
            && matches!(action.as_str(), "refresh" | "review" | "reapply" | "repair" | "remove-gpu" | "forget" | "confirm-forget" | "verify" | "confirm" | "confirm-enroll" | "confirm-verify" | "reconcile" | "review-reset" | "confirm-reset" | "reconcile-enrollment" | "save-only" | "credentials" | "discard-review-forget")
        {

            return;
        }
        if matches!(action.as_str(), "verify" | "repair" | "remove-gpu" | "forget") {
            let message = action_state.lock()
                .map_err(|_| "Live presentation state unavailable".to_owned())
                .map(|state| state.action_recovery_message());
            match message {
                Ok(Some(body)) => {
                    dialog(&ui, "recovery-needed", hyper_gpu_support::ui_text::operations::recovery::REVIEW_FIRST_TITLE, &body);
                    return;
                }
                Err(error) => {
                    dialog(&ui, "error", hyper_gpu_support::ui_text::operations::recovery::UNREADABLE_TITLE, &error);
                    return;
                }
                Ok(None) => {}
            }
            if !ui.get_execution_ready() {
                dialog(&ui, "info", hyper_gpu_support::ui_text::operations::action::UNAVAILABLE_TITLE, hyper_gpu_support::ui_text::operations::action::SETUP_UNAVAILABLE);
                return;
            }
            let availability = action_state.lock()
                .map_err(|_| "Live presentation state unavailable".to_owned())
                .map(|state| {
                    let values = ui.get_values().iter().map(|value| value.to_string()).collect::<Vec<_>>();
                    let actions = state.selected_actions(ui.get_gpu_index(), &values);
                    (actions.allowed(action.as_str()).unwrap_or(true), actions.reason(action.as_str()).unwrap_or_default().to_owned())
                });
            match availability {
                Ok((false, reason)) => {
                    dialog(&ui, "info", hyper_gpu_support::ui_text::operations::action::UNAVAILABLE_TITLE, &reason);
                    return;
                }
                Err(error) => {
                    dialog(&ui, "error", hyper_gpu_support::ui_text::operations::action::UNREADABLE_TITLE, &error);
                    return;
                }
                _ => {}
            }
        }
        let result = match action.as_str() {
            "refresh" => refresh(&ui, &action_state, &action_busy, &action_source, false),
            #[cfg(windows)]
            "discard-review-forget" if ui.get_dialog_kind() == "reset-complete" => {
                if !with_state(&ui, &action_state, |state| {
                    if state.action_recovery_message().is_some() { return Err("Review unfinished recovery first.".into()); }
                    state.discard();
                    present(&ui, state, true);
                    Ok(())
                }) { return; }
                review_forget_live(&ui, &action_state, &action_busy)
            }
            #[cfg(windows)]
            "forget" => review_forget_live(&ui, &action_state, &action_busy),
            #[cfg(windows)]
            "confirm-forget" if ui.get_dialog_kind() == "forget-review" => forget_live(&ui, &action_state, &action_busy),
            "review" => review(&ui, &action_state, &action_busy, &action_source),
            #[cfg(windows)]
            "confirm-enroll" => enroll_live(&ui, &action_state, &action_busy, &action_source),
            #[cfg(windows)]
            "reconcile-enrollment" => reconcile_enrollment_live(&ui, &action_state, &action_busy, &action_source),
            "confirm" if ui.get_dialog_kind() == "review" => apply_live(&ui, &action_state, &action_busy),
            "confirm" => Err("Open a fresh review before Apply.".into()),
            "save-only" => save_only_live(&ui, &action_state, &action_busy),
            "verify" => review_verification(&ui, &action_state, &action_busy),
            "confirm-verify" => verify_live(&ui, &action_state, &action_busy),
            "reconcile" => reconcile_live(&ui, &action_state, &action_busy),
            #[cfg(windows)]
            "review-reset" => review_reset(&ui, &action_state),
            #[cfg(windows)]
            "confirm-reset" if ui.get_dialog_kind() == "reset-review" => reset_live(&ui, &action_state, &action_busy),
            "reapply" => {
                // Preserve allocation edits while deliberately previewing enabled support.
                ui.set_gpu_enabled(true);
                ui.invoke_edited();
                review(&ui, &action_state, &action_busy, &action_source)
            }
            "repair" => {
                // Reuse the enabled draft; reinstall intent belongs only to this review.
                ui.set_gpu_enabled(true);
                ui.invoke_edited();
                review_operation(&ui, &action_state, &action_busy, &action_source, true)
            }
            "remove-gpu" => {
                let removal = action_state.lock().map_err(|_| "Presentation state unavailable".to_owned())
                    .and_then(|state| state.removal_draft(
                        ui.get_selected_id().as_str(), ui.get_gpu_index(),
                        &ui.get_values().iter().map(|v| v.to_string()).collect::<Vec<_>>(),
                    ));
                removal.and_then(|_| {
                    ui.set_gpu_enabled(false);
                    ui.invoke_edited();
                    review(&ui, &action_state, &action_busy, &action_source)
                })
            }
            "close" => {
                if ui.get_review_preparing() {
                    ui.set_review_preparing(false);
                    ui.set_running(false);
                    // The background admission remains held until the abandoned read finishes.
                }
                with_state(&ui, &action_state, |state| {
                    state.cancel_review();
                    ui.set_dialog_kind("".into());
                    Ok(())
                });
                Ok(())
            },
            "exit" => { ui.set_dirty(false); slint::quit_event_loop().map_err(|e| e.to_string()) },
            "discard" | "switch" => {
                with_state(&ui, &action_state, |state| {
                    state.discard();
                    if action == "switch" { state.view.selected = state.pending_selection.take(); }
                    present(&ui, state, true); ui.set_dirty(false); ui.set_dialog_kind("".into());
                     Ok(())
                }); Ok(())
            }
            "more" => {
                if !with_state(&ui, &action_state, |_| Ok(())) { return; }
                let body = hyper_gpu_support::ui_text::operations::action::review_prompt(&(ui.get_vm_name())).into_string();
                dialog(&ui, "more", hyper_gpu_support::ui_text::operations::action::MENU_TITLE, &body); Ok(())
            },
            "credentials" => {
                let selected = ui.get_selected_id().to_string();
                let vm_name = ui.get_vm_name().to_string();
                let prompt_name = vm_name.clone();
                let credential_source = action_source.clone();
                action_state.lock().map_err(|_| "Live presentation state unavailable".to_owned()).and_then(|s| s.target(&selected))
                    .and_then(|target| background_for_vm(&ui, &action_busy, Operation::new(Kind::Credentials, hyper_gpu_support::ui_text::credentials::settings::TITLE), Some(&vm_name), true, move || credential_source.credentials(&target.vm_id, &prompt_name), |ui, result| match result {
                        Ok(true) => dialog(ui, "info", hyper_gpu_support::ui_text::operations::credentials::stored::TITLE, hyper_gpu_support::ui_text::operations::credentials::stored::BODY),
                        Ok(false) => dialog(ui, "info", hyper_gpu_support::ui_text::operations::credentials::absent::TITLE, hyper_gpu_support::ui_text::operations::credentials::absent::BODY),
                        Err(error) => {
                            if !credential_request_cancelled(ui, &error, false) {
                                dialog(ui, "error", hyper_gpu_support::ui_text::operations::credentials::UNAVAILABLE_TITLE, &error);
                            }
                        },
                    }))
            }
            "diagnostics" | "recovery" => {
                with_state(&ui, &action_state, |state| {
                    #[cfg(windows)]
                    if action == "recovery"
                        && let Some(record) = &state.enrollment_recovery {
                        let revision = hyper_gpu_support::runner::pair_revision()?;
                        let known = revision == record.resulting || revision == record.review.expected;
                        if !known { return Err("Authority does not match either recorded revision. Recovery remains held for administrator inspection.".into()); }
                        let outcome = if record.removal.is_some() {
                            hyper_gpu_support::ui_text::operations::enrollment::recovery::PARTIAL_DELETION
                        } else if revision == record.resulting {
                            hyper_gpu_support::ui_text::operations::enrollment::recovery::POLICY_PUBLISHED
                        } else {
                            hyper_gpu_support::ui_text::operations::enrollment::recovery::POLICY_ORIGINAL
                        };
                        let body = format!("Operation: {}\nVM ID: {}\nGPU: {}\n\n{outcome}", record.operation_id, record.review.vm_id, record.review.gpu_interface);
                        let name = state.view.vm(&record.review.vm_id).map_or(hyper_gpu_support::ui_text::operations::enrollment::recovery::NAME_UNAVAILABLE, |vm| vm.name.as_str());
                        let summary = format!("Virtual machine: {name}\n\n{outcome}\n\nReviewing this record makes no changes. Confirmation is required for the recovery action shown.");
                        state.enrollment_recovery_revision = Some(revision);
                        ui.set_needs_shutdown(false);
                        dialog(&ui, "enrollment-recovery", if record.removal.is_some() { hyper_gpu_support::ui_text::operations::profile_deletion::recovery::TITLE } else { hyper_gpu_support::ui_text::operations::enrollment::recovery::TITLE }, &summary);
                        set_details(&ui, &body);
                        return Ok(());
                    }
                    if action == "recovery" && let Some(record) = &state.import_recovery {
                        dialog(&ui, "info", hyper_gpu_support::ui_text::operations::configuration_import::recovery::TITLE, &hyper_gpu_support::ui_text::operations::configuration_import::recovery::body(&(record.operation_id), record.documents.len(), record.documents.values().filter(|item| item.published).count()).into_string());
                        return Ok(());
                    }
                    if action == "recovery"
                        && let Some(record) = &state.recovery {
                        ui.set_needs_shutdown(record.requires_graphics());
                        ui.set_power_consent(wording::recovery_consent(&record.final_power()?).into());
                        match record.phase {
                            hyper_gpu_support::configuration_store::SavePhase::Verified(_) | hyper_gpu_support::configuration_store::SavePhase::Published(_) => {
                                if record.publication_required {
                                    dialog(&ui, "save", hyper_gpu_support::ui_text::operations::save_only::review::TITLE, hyper_gpu_support::ui_text::operations::save_only::review::BODY);
                                    set_details(&ui, &format!("Operation: {}\nVM ID: {}\nGPU: {}\nReviewed configuration revision: {:?}\nDriver: {}\nExpected final power: {:?}", record.operation_id, record.target.vm_id, record.target.gpu_interface, record.expected, record.driver_version, record.final_power()?));
                                } else {
                                    dialog(&ui, "recovery", hyper_gpu_support::ui_text::operations::verify::recovery::TITLE, hyper_gpu_support::ui_text::operations::verify::recovery::BODY);
                                    ui.set_needs_shutdown(false);
                                }
                            }
                            hyper_gpu_support::configuration_store::SavePhase::Admitted | hyper_gpu_support::configuration_store::SavePhase::EffectsStarted => {
                                let name = state.view.vm(&record.target.vm_id).map_or(record.initial.name.as_str(), |vm| vm.name.as_str());
                                if !record.publication_required {
                                    let restored = state.recovery_without_guest(record)?;
                                    let body = wording::verification_recovery(record, name, restored);
                                    dialog(&ui, "recovery", hyper_gpu_support::ui_text::operations::verify::recovery::TITLE, &body);
                                    ui.set_needs_shutdown(!restored);
                                } else if !record.requires_graphics() {
                                    dialog(&ui, "recovery", hyper_gpu_support::ui_text::operations::apply::recovery::TITLE, &hyper_gpu_support::ui_text::operations::apply::recovery::body(name).into_string());
                                } else {
                                    dialog(&ui, "recovery", hyper_gpu_support::ui_text::operations::recovery::review::TITLE, &hyper_gpu_support::ui_text::operations::recovery::review::body(name).into_string());
                                }
                                set_details(&ui, &format!("Operation: {}\nVM ID: {}\nGPU: {}\nOriginal guest power: {:?}\nReviewed final guest power: {:?}", record.operation_id, record.target.vm_id, record.target.gpu_interface, record.initial.power, record.final_power()?));
                            }
                        }
                        return Ok(());
                    }
                    state.view.inventory.as_ref().ok_or("Refresh inventory first.")?;
                    let body = super::diagnostics::selected_vm_details(state);
                    let provenance = hyper_gpu_support::ui_text::operations::inventory::PROVENANCE;
                    dialog(&ui, "info", hyper_gpu_support::ui_text::operations::inventory::DETAILS_TITLE, &format!("{provenance}\n\nTechnical Details contains the host observation, recorded history and authorization from that inventory. Missing or old records do not establish current guest health. Opening these details performs no new checks or changes."));
                    set_details(&ui, &body); Ok(())
                }); Ok(())
            }
            "shortcuts" => {
                let help = hyper_gpu_support::ui_text::operations::help::BODY;
                dialog(&ui, "info", hyper_gpu_support::ui_text::operations::help::TITLE, help); Ok(())
            },
            _ => Err("This action is not connected yet. Use the supported CLI; no operation was performed.".into()),
        };
        if let Err(error) = result { dialog(&ui, "error", hyper_gpu_support::ui_text::operations::action::UNAVAILABLE_TITLE, &error); }
    });
    let weak = ui.as_weak();
    ui.window().on_close_requested(move || {
        if let Some(ui) = weak.upgrade() {
            if ui.get_running() {
                return slint::CloseRequestResponse::KeepWindowShown;
            }
            if ui.get_dirty() {
                dialog(
                    &ui,
                    "exit",
                    hyper_gpu_support::ui_text::operations::draft::exit::TITLE,
                    hyper_gpu_support::ui_text::operations::draft::exit::BODY,
                );
                return slint::CloseRequestResponse::KeepWindowShown;
            }
        }
        slint::CloseRequestResponse::HideWindow
    });
    // Show the splash before scheduling discovery; all provider/file work stays off-thread.
    ui.show()?;
    if let Err(error) = refresh(&ui, &state, &busy, &source, false) {
        finish_startup(&ui, Err(error));
    }
    // Only the installed read path polls. First-install elevation remains explicit.
    let status_timer = slint::Timer::default();
    {
        let weak = ui.as_weak();
        let state = state.clone();
        status_timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_secs(15),
            move || {
                if let Some(ui) = weak.upgrade()
                    && !ui.get_startup_loading()
                    && !busy.load(Ordering::Acquire)
                    && ui.get_dialog_kind().is_empty()
                    && let Err(error) = refresh(&ui, &state, &busy, &source, true)
                {
                    ui.set_notice(
                        format!("VM status refresh unavailable: {error}. Use Refresh to retry.")
                            .into(),
                    );
                }
            },
        );
    }
    ui.run()?;
    if let Ok(state) = state.lock() {
        super::selection::save(state.view.selected.as_deref())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_failure_or_panic_never_becomes_a_completed_operation() {
        assert_eq!(run_background_task(|| Ok(7)).unwrap(), 7);
        assert_eq!(
            run_background_task::<()>(|| Err("read failed".into())).unwrap_err(),
            "read failed"
        );
        let error = run_background_task::<()>(|| panic!("fixture panic payload")).unwrap_err();
        assert!(error.contains("without a terminal outcome"));
        assert!(!error.contains("fixture panic payload"));
    }
}
