//! Exercise compiled accessibility bindings without a native window or VM access.
use super::{AppWindow, catalogue_tests::Headless, memory::MemorySlider};
use hyper_gpu_support::model::Allocation;
use slint::ComponentHandle;
// The pinned Slint version exposes item inspection through its generated-code API.
// Keep this private API confined to tests rather than adding a production dependency.
use slint::private_unstable_api::re_exports::{
    AccessibilityAction, AccessibleStringProperty, ItemRc, WindowInner,
};
use std::{cell::RefCell, ops::ControlFlow, rc::Rc};

fn memory_slider(ui: &AppWindow) -> ItemRc {
    let root = ItemRc::new_root(WindowInner::from_pub(ui.window()).component());
    let label = hyper_gpu_support::ui_text::dashboard::memory::slider_accessible();
    let mut matches = Vec::new();
    root.visit_descendants::<()>(|item| {
        if item
            .accessible_string_property(AccessibleStringProperty::Label)
            .as_deref()
            == Some(label.as_str())
        {
            matches.push(item.clone());
        }
        ControlFlow::Continue(())
    });
    assert_eq!(matches.len(), 1, "one memory slider in the compiled UI");
    matches.pop().unwrap()
}

fn number(item: &ItemRc, property: AccessibleStringProperty) -> f64 {
    item.accessible_string_property(property)
        .unwrap()
        .parse()
        .unwrap()
}

#[test]
fn memory_accessibility_uses_gb_for_values_ranges_and_actions_and_obeys_editing_guards() {
    use AccessibilityAction::{Decrement, Increment, SetValue};
    use AccessibleStringProperty::{Description, Value, ValueMaximum, ValueMinimum, ValueStep};
    slint::platform::set_platform(Box::new(Headless)).unwrap();
    let ui = AppWindow::new().unwrap();
    ui.set_startup_loading(false);
    ui.set_selected_id("12345678-1234-1234-1234-123456789abc".into());
    ui.set_memory_available(true);
    ui.set_eligible(true);
    ui.set_memory_minimum(10);
    ui.set_memory_maximum(80);
    ui.set_memory_position(60);
    ui.set_memory_label("6 GB".into());
    let edits = Rc::new(RefCell::new(Vec::new()));
    let captured = edits.clone();
    let weak = ui.as_weak();
    ui.on_memory_edited(move |position| {
        captured.borrow_mut().push(position);
        weak.upgrade().unwrap().set_memory_position(position);
    });
    let slider = memory_slider(&ui);
    assert_eq!(number(&slider, Value), 6.0);
    assert_eq!(number(&slider, ValueMinimum), 1.0);
    assert_eq!(number(&slider, ValueMaximum), 8.0);
    assert!((number(&slider, ValueStep) - 0.1).abs() < 1e-6);
    assert_eq!(
        slider.accessible_string_property(Description).unwrap(),
        "6 GB"
    );

    // Both directions cross the UI boundary in tenths and keep native byte semantics.
    slider.accessible_action(&SetValue("6.1".into()));
    assert_eq!(&*edits.borrow(), &[61]);
    assert!((number(&slider, Value) - 6.1).abs() < 1e-6);
    let bounds = Allocation {
        minimum: 1,
        optimal: 6_000_000_000,
        maximum: 8_000_000_000,
    };
    let allocation = MemorySlider::new(&bounds, Some(&bounds), 8_000_000_000)
        .unwrap()
        .select(edits.borrow()[0])
        .unwrap();
    assert_eq!(allocation.optimal, 6_100_000_000);
    slider.accessible_action(&Increment);
    slider.accessible_action(&Decrement);
    assert_eq!(&*edits.borrow(), &[61, 62, 61]);
    for invalid in ["60", "0", "NaN", "inf", "-inf", "invalid"] {
        slider.accessible_action(&SetValue(invalid.into()));
    }
    assert_eq!(&*edits.borrow(), &[61, 62, 61]);

    for (position, action) in [(80, Increment), (10, Decrement)] {
        ui.set_memory_position(position);
        slider.accessible_action(&action);
    }
    assert_eq!(edits.borrow().len(), 3, "endpoints must not emit an edit");

    // Recovery, unsupported/unavailable editing, presentation and modals all
    // suppress actions even if an assistive client invokes a disabled control.
    ui.set_memory_position(60);
    for guard in 0..6 {
        ui.set_recovery(guard == 0);
        ui.set_eligible(guard != 1);
        ui.set_memory_available(guard != 2);
        ui.set_presenting(guard == 3);
        ui.set_dialog_kind(if guard == 4 { "more" } else { "" }.into());
        ui.set_initial_setup_required(guard == 5);
        slint::platform::update_timers_and_animations();
        let slider = memory_slider(&ui);
        assert_eq!(number(&slider, Value), 6.0);
        for action in [SetValue("7".into()), Increment, Decrement] {
            slider.accessible_action(&action);
        }
        assert_eq!(ui.get_memory_position(), 60, "editing guard {guard}");
        assert_eq!(edits.borrow().len(), 3);
    }
}
