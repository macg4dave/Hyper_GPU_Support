//! Compiled layout regression using a software window; no native GUI or VM access.
use super::AppWindow;
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, PlatformError, WindowAdapter, WindowEvent};
use slint::private_unstable_api::re_exports::{
    AccessibilityAction, AccessibleStringProperty, ItemRc, WindowInner,
};
use slint::{ComponentHandle, LogicalPosition, LogicalSize, Rgb8Pixel};
use std::{ops::ControlFlow, rc::Rc};

struct LayoutPlatform(Rc<MinimalSoftwareWindow>);
impl Platform for LayoutPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.0.clone())
    }
}

fn control(ui: &AppWindow, label: &str) -> ItemRc {
    let root = ItemRc::new_root(WindowInner::from_pub(ui.window()).component());
    let mut found = Vec::new();
    root.visit_descendants::<()>(|item| {
        if item
            .accessible_string_property(AccessibleStringProperty::Label)
            .as_deref()
            == Some(label)
        {
            found.push(item.clone());
        }
        ControlFlow::Continue(())
    });
    assert_eq!(found.len(), 1, "unique control: {label}");
    found.pop().unwrap()
}

fn draw(window: &MinimalSoftwareWindow) {
    window.request_redraw();
    window.draw_if_needed(|renderer| {
        let size = window.size();
        let mut pixels = vec![Rgb8Pixel::default(); (size.width * size.height) as usize];
        renderer.render(&mut pixels, size.width as usize);
    });
}

#[test]
fn expanded_allocation_and_technical_output_scroll_above_fixed_actions_after_resize() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(LayoutPlatform(window.clone()))).unwrap();
    let ui = AppWindow::new().unwrap();
    ui.set_selected_id("12345678-1234-1234-1234-123456789abc".into());
    ui.set_vm_name("Layout test".into());
    ui.set_eligible(true);
    ui.set_vm_details("Diagnostic line\n".repeat(4).into());
    ui.show().unwrap();
    window.set_size(LogicalSize::new(1240., 860.));
    draw(&window);
    control(
        &ui,
        hyper_gpu_support::ui_text::resources::advanced::collapsed_accessible().as_str(),
    )
    .accessible_action(&AccessibilityAction::Default);
    draw(&window);
    control(
        &ui,
        hyper_gpu_support::ui_text::diagnostics::show_label().as_str(),
    )
    .accessible_action(&AccessibilityAction::Default);
    for (width, height, collapse) in [
        (1240., 860., false),
        (1000., 620., false),
        (700., 520., false),
        (1240., 860., true),
    ] {
        window.set_size(LogicalSize::new(width, height));
        draw(&window);
        if collapse {
            control(
                &ui,
                hyper_gpu_support::ui_text::resources::advanced::expanded_accessible().as_str(),
            )
            .accessible_action(&AccessibilityAction::Default);
            draw(&window);
        }
        ui.window().dispatch_event(WindowEvent::PointerScrolled {
            position: LogicalPosition::new(width - 70., 180.),
            delta_x: -10000.,
            delta_y: 0.,
        });
        draw(&window);
        for _ in 0..3 {
            ui.window().dispatch_event(WindowEvent::PointerScrolled {
                position: LogicalPosition::new(width - 70., 180.),
                delta_x: 0.,
                delta_y: -10000.,
            });
            draw(&window);
        }
        let output = control(
            &ui,
            hyper_gpu_support::ui_text::diagnostics::read_only_accessible().as_str(),
        );
        let top = output.map_to_window(Default::default()).y;
        let bottom = top + output.geometry().height();
        let actions = control(
            &ui,
            hyper_gpu_support::ui_text::dashboard::apply::label().as_str(),
        );
        let actions_top = actions.map_to_window(Default::default()).y;
        assert!(top >= 0., "output starts inside window: {top}");
        assert!(
            output.is_visible(),
            "diagnostics must be inside the scroll viewport"
        );
        assert!(
            bottom <= actions_top,
            "output bottom {bottom} must precede actions {actions_top} at {width}x{height}"
        );
        assert!(
            output.geometry().height() >= 100.,
            "diagnostics must retain a usable text viewport"
        );
    }
    ui.hide().unwrap();
}
