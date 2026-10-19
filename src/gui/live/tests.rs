//! Callback integration without discovery, native prompts, background jobs or effects.
use super::{SharedState, Source, actions, editor};
use crate::gui::{AppWindow, catalogue_tests::Headless, live_state::State, model};
use slint::Model;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

#[test]
fn failed_startup_diagnostics_open_without_inventory_and_preserve_safe_original_codes() {
    slint::platform::set_platform(Box::new(Headless)).unwrap();
    let ui = AppWindow::new().unwrap();
    let state: SharedState = Arc::new(Mutex::new(State::default()));
    let busy = Arc::new(AtomicBool::new(false));
    actions::register(&ui, &state, &busy, &Source);
    super::refresh::finish_startup(&ui, Err(
        "Protected reader release is unconfirmed; foreground admission remains held: 0x80070005 private-adapter-content".into(),
    ));
    assert!(ui.get_startup_loading());
    assert!(
        ui.get_startup_error()
            .contains("status check couldn't confirm")
    );
    assert!(ui.get_inventory_details().contains("0x80070005"));
    assert!(
        !ui.get_inventory_details()
            .contains("private-adapter-content")
    );
    ui.set_startup_loading(false);
    ui.invoke_action("diagnostics".into());
    assert_eq!(ui.get_dialog_kind(), "info");
    assert_eq!(ui.get_dialog_body(), ui.get_inventory_error());
    assert!(ui.get_dialog_details().contains("0x80070005"));
    assert!(!ui.get_dialog_body().contains("Action unavailable"));
    assert!(state.lock().unwrap().view.inventory.is_none());
    assert!(
        !busy.load(Ordering::Acquire),
        "inspection starts no backend work"
    );
    ui.invoke_action("close".into());
    assert!(ui.get_dialog_kind().is_empty());
}

#[test]
fn selection_callback_keeps_dirty_draft_and_modal_memory_edits_on_original_vm() {
    slint::platform::set_platform(Box::new(Headless)).unwrap();
    let ui = AppWindow::new().unwrap();
    let state: SharedState = Arc::new(Mutex::new(State::default()));
    editor::register(&ui, &state);
    let original = "12345678-1234-1234-1234-123456789abc";
    let next = "abcdef12-1234-1234-1234-123456789abc";
    ui.set_selected_id(original.into());
    ui.set_dirty(true);
    ui.set_values(model(vec!["6000000000".into(); 12]));

    ui.invoke_select_vm(next.into());
    assert_eq!(ui.get_selected_id(), original);
    assert_eq!(ui.get_dialog_kind(), "switch");
    assert_eq!(
        state.lock().unwrap().pending_selection.as_deref(),
        Some(next)
    );
    assert!(ui.get_dirty());

    ui.invoke_memory_edited(20);
    ui.invoke_memory_text_edited("2".into());
    assert!(ui.get_values().iter().all(|value| value == "6000000000"));
    assert_eq!(ui.get_selected_id(), original);
    assert_eq!(ui.get_dialog_kind(), "switch");
}

#[test]
fn close_callback_abandons_review_but_retains_busy_admission_and_draft() {
    slint::platform::set_platform(Box::new(Headless)).unwrap();
    let ui = AppWindow::new().unwrap();
    let state: SharedState = Arc::new(Mutex::new(State::default()));
    let busy = Arc::new(AtomicBool::new(true));
    actions::register(&ui, &state, &busy, &Source);
    state.lock().unwrap().result_notice = Some(crate::gui::result_notice::ResultNotice {
        vm_id: "12345678-1234-1234-1234-123456789abc".into(),
        kind: "save".into(),
        title: "Unsaved result".into(),
        body: "Earlier result".into(),
        details: "Earlier log".into(),
    });
    ui.set_dirty(true);
    ui.set_running(true);
    ui.set_review_preparing(true);
    ui.set_updating(true);
    ui.set_dialog_kind("install-progress".into());
    ui.set_operation_dialog_active(true);

    ui.invoke_action("close".into());
    assert!(!ui.get_review_preparing());
    assert!(ui.get_running());
    assert_eq!(ui.get_dialog_kind(), "cancelling");
    assert!(!ui.get_operation_dialog_active());
    assert!(ui.get_dirty());
    assert!(ui.get_updating());
    assert!(busy.load(Ordering::Acquire));
    assert!(state.lock().unwrap().result_notice.is_none());
}

#[test]
fn cancellation_waits_for_admission_release_then_closes_without_continuing() {
    slint::platform::set_platform(Box::new(Headless)).unwrap();
    let ui = AppWindow::new().unwrap();
    let state: SharedState = Arc::new(Mutex::new(State::default()));
    let busy = Arc::new(AtomicBool::new(true));
    actions::register(&ui, &state, &busy, &Source);
    for kind in [
        crate::gui::operation::Kind::Verify,
        crate::gui::operation::Kind::Apply,
    ] {
        busy.store(true, Ordering::Release);
        ui.set_running(true);
        ui.set_updating(true);
        ui.set_review_preparing(true);
        ui.set_dialog_kind("install-progress".into());
        ui.invoke_action("close".into());
        assert_eq!(ui.get_dialog_kind(), "cancelling", "{kind:?}");
        assert!(ui.get_running());
        assert!(busy.load(Ordering::Acquire));
        for action in ["close", "more", "verify", "repair", "review", "refresh"] {
            ui.invoke_action(action.into());
            assert_eq!(
                ui.get_dialog_kind(),
                "cancelling",
                "wait remains modal: {action}"
            );
        }
        // Match the worker's release order. Both successful and failed late
        // results take this same discard branch before the completion callback.
        busy.store(false, Ordering::Release);
        ui.set_updating(false);
        assert!(super::background::finish_cancelled_preparation(&ui, true));
        assert!(ui.get_dialog_kind().is_empty());
        assert!(!ui.get_running());
        assert!(!ui.get_updating());
        ui.invoke_action("more".into());
        assert_eq!(ui.get_dialog_kind(), "more");
        ui.invoke_action("close".into());
        assert!(ui.get_dialog_kind().is_empty());
    }
    ui.set_review_preparing(true);
    assert!(!super::background::finish_cancelled_preparation(&ui, true));
    assert!(!super::background::finish_cancelled_preparation(&ui, false));
}
