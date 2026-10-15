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
        if let Some(directory) = std::env::var_os("HYPER_GPU_LAYOUT_RENDER_DIR") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            let mut image = format!("P6\n{} {}\n255\n", size.width, size.height).into_bytes();
            for pixel in pixels {
                image.extend_from_slice(&[pixel.r, pixel.g, pixel.b]);
            }
            std::fs::write(directory.join("layout.ppm"), image).unwrap();
        }
    });
}

fn retain_render(name: &str) {
    if let Some(directory) = std::env::var_os("HYPER_GPU_LAYOUT_RENDER_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::copy(
            directory.join("layout.ppm"),
            directory.join(format!("{name}.ppm")),
        )
        .unwrap();
    }
}

#[test]
fn expanded_allocation_and_technical_output_scroll_above_fixed_actions_after_resize() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(LayoutPlatform(window.clone()))).unwrap();
    let ui = AppWindow::new().unwrap();
    assert!(ui.get_startup_loading());
    ui.show().unwrap();
    window.set_size(LogicalSize::new(700., 520.));
    draw(&window);
    retain_render("startup");
    super::live::finish_startup(
        &ui,
        Err("Discovery failed. Continue and use Refresh to retry.".into()),
    );
    assert!(ui.get_startup_loading());
    assert!(ui.get_startup_error().contains("Discovery failed"));
    draw(&window);
    let continue_button = control(
        &ui,
        hyper_gpu_support::ui_text::dashboard::startup::CONTINUE_LABEL,
    );
    assert!(continue_button.is_visible());
    continue_button.accessible_action(&AccessibilityAction::Default);
    assert!(!ui.get_startup_loading());
    ui.set_startup_loading(true);
    ui.set_startup_error("".into());
    super::live::finish_startup(&ui, Ok(()));
    assert!(!ui.get_startup_loading());
    ui.set_vms(super::model(
        ["windows", "linux", "unknown"]
            .into_iter()
            .map(|os| super::VmRow {
                id: os.into(),
                name: os.into(),
                os: os.into(),
                ..Default::default()
            })
            .collect(),
    ));
    ui.set_selected_id("12345678-1234-1234-1234-123456789abc".into());
    ui.set_vm_name("Layout test".into());
    ui.set_eligible(true);
    ui.set_support_summary("Wrapped support information. ".repeat(15).into());
    ui.set_resource_hints(super::model(vec![
        "Long capability information. ".repeat(20).into(),
    ]));
    ui.set_vm_details("Diagnostic line\n".repeat(40).into());
    ui.show().unwrap();
    window.set_size(LogicalSize::new(1240., 860.));
    draw(&window);
    for (theme, label) in [
        (
            "light",
            hyper_gpu_support::ui_text::settings::appearance::LIGHT,
        ),
        (
            "dark",
            hyper_gpu_support::ui_text::settings::appearance::DARK,
        ),
    ] {
        ui.set_page(2);
        draw(&window);
        control(&ui, label).accessible_action(&AccessibilityAction::Default);
        ui.set_page(0);
        draw(&window);
        for label in [
            hyper_gpu_support::ui_text::dashboard::os::WINDOWS_ACCESSIBLE,
            hyper_gpu_support::ui_text::dashboard::os::LINUX_ACCESSIBLE,
            hyper_gpu_support::ui_text::dashboard::os::UNKNOWN_ACCESSIBLE,
        ] {
            assert!(control(&ui, label).is_visible());
        }
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
        draw(&window);
        retain_render(&format!("icons-{theme}"));
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
            retain_render(&format!("details-{theme}-{width}x{height}-{collapse}"));
            let output = control(
                &ui,
                hyper_gpu_support::ui_text::dashboard::diagnostics::OUTPUT_LABEL,
            );
            let top = output.map_to_window(output.geometry().origin).y;
            let bottom = top + output.geometry().height();
            let actions = control(
                &ui,
                hyper_gpu_support::ui_text::dashboard::apply::label().as_str(),
            );
            let actions_top = actions.map_to_window(actions.geometry().origin).y;
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
    }
    ui.hide().unwrap();
}

#[test]
fn permission_review_actions_stay_on_one_row_at_supported_sizes_in_both_themes() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(LayoutPlatform(window.clone()))).unwrap();
    let ui = AppWindow::new().unwrap();
    ui.set_startup_loading(false);
    ui.set_execution_ready(true);
    ui.show().unwrap();
    window.set_size(LogicalSize::new(1240., 860.));
    for (theme, label) in [
        (
            "light",
            hyper_gpu_support::ui_text::settings::appearance::LIGHT,
        ),
        (
            "dark",
            hyper_gpu_support::ui_text::settings::appearance::DARK,
        ),
    ] {
        ui.set_dialog_kind("".into());
        ui.set_page(2);
        draw(&window);
        control(&ui, label).accessible_action(&AccessibilityAction::Default);
        ui.set_page(0);
        ui.set_dialog_title(
            hyper_gpu_support::ui_text::operations::enrollment::review::TITLE.into(),
        );
        ui.set_enrollment_heading("Virtual machine: VM1\nPhysical GPU: NVIDIA GPU".into());
        ui.set_enrollment_allocation("8.27850752 GB".into());
        ui.set_dialog_body(
            hyper_gpu_support::ui_text::operations::enrollment::review::STEPS.into(),
        );
        ui.set_dialog_details("Diagnostic section\n".repeat(30).into());
        ui.set_dialog_kind("enroll-review".into());
        for (width, height) in [(1240., 860.), (1000., 620.), (700., 520.)] {
            window.set_size(LogicalSize::new(width, height));
            draw(&window);
            for expanded in [false, true] {
                let labels = [
                    hyper_gpu_support::ui_text::dialogs::draft::KEEP_LABEL,
                    hyper_gpu_support::ui_text::dialogs::enrollment::CONFIRM_LABEL,
                    if expanded {
                        hyper_gpu_support::ui_text::diagnostics::HIDE_LABEL
                    } else {
                        hyper_gpu_support::ui_text::diagnostics::SHOW_LABEL
                    },
                ];
                let mut row_y: Option<f32> = None;
                let mut previous_right = 0.;
                for label in labels {
                    let button = control(&ui, label);
                    let geometry = button.geometry();
                    let origin = button.map_to_window(geometry.origin);
                    assert!(button.is_visible(), "visible action: {label}");
                    assert!(origin.x >= previous_right && origin.x + geometry.width() <= width);
                    assert!(origin.y >= 0. && origin.y + geometry.height() <= height);
                    if let Some(y) = row_y {
                        assert!(
                            (origin.y - y).abs() < 1.,
                            "actions share one row at {width}x{height}"
                        );
                    }
                    row_y = Some(origin.y);
                    previous_right = origin.x + geometry.width();
                }
                retain_render(&format!("permission-{theme}-{width}x{height}-{expanded}"));
                control(&ui, labels[2]).accessible_action(&AccessibilityAction::Default);
                draw(&window);
            }
        }
    }
    ui.hide().unwrap();
}
