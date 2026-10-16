//! Callback integration without discovery, native prompts, background jobs or effects.
use super::{SharedState, Source, actions, editor};
use crate::gui::{AppWindow, catalogue_tests::Headless, live_state::State, model};
use slint::Model;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

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
    state.lock().unwrap().enrollment_review = Some(hyper_gpu_support::enrollment::PairReview {
        vm_id: "12345678-1234-1234-1234-123456789abc".into(),
        gpu_interface: "fixture-gpu".into(),
        expected: hyper_gpu_support::configuration_store::Revision::Missing,
        operator_sid: "fixture-operator".into(),
    });
    ui.set_dirty(true);
    ui.set_running(true);
    ui.set_review_preparing(true);
    ui.set_updating(true);
    ui.set_dialog_kind("progress".into());

    ui.invoke_action("close".into());
    assert!(!ui.get_review_preparing());
    assert!(!ui.get_running());
    assert!(ui.get_dialog_kind().is_empty());
    assert!(ui.get_dirty());
    assert!(ui.get_updating());
    assert!(busy.load(Ordering::Acquire));
    assert!(state.lock().unwrap().enrollment_review.is_none());
}
