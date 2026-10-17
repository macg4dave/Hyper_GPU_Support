//! Compiled layout regression using a software window; no native GUI or VM access.
use super::AppWindow;
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, PlatformError, WindowAdapter, WindowEvent};
use slint::private_unstable_api::re_exports::{
    AccessibilityAction, AccessibleRole, AccessibleStringProperty, ItemRc, WindowInner,
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
    // A stepper label can share its text with the corresponding action.
    // Resolve the interactive control by role rather than its caption alone.
    let buttons: Vec<_> = found
        .iter()
        .filter(|item| {
            matches!(
                item.accessible_role(),
                AccessibleRole::Button | AccessibleRole::Checkbox
            )
        })
        .cloned()
        .collect();
    if !buttons.is_empty() {
        found = buttons;
    }
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
fn install_dialog_keeps_actions_reachable_and_log_copyable_in_both_themes() {
    use hyper_gpu_support::ui_text::{diagnostics, dialogs::install, settings};
    use std::cell::{Cell, RefCell};
    use std::time::Duration;
    struct InstallPlatform {
        window: Rc<MinimalSoftwareWindow>,
        clipboard: Rc<RefCell<String>>,
        clock: Rc<Cell<Duration>>,
    }
    impl Platform for InstallPlatform {
        fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
            Ok(self.window.clone())
        }
        fn set_clipboard_text(&self, text: &str, _: slint::platform::Clipboard) {
            *self.clipboard.borrow_mut() = text.into();
        }
        fn duration_since_start(&self) -> Duration {
            self.clock.get()
        }
    }
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    let clipboard = Rc::new(RefCell::new(String::new()));
    let clock = Rc::new(Cell::new(Duration::ZERO));
    slint::platform::set_platform(Box::new(InstallPlatform {
        window: window.clone(),
        clipboard: clipboard.clone(),
        clock: clock.clone(),
    }))
    .unwrap();
    let ui = AppWindow::new().unwrap();
    ui.set_startup_loading(false);
    let actions = Rc::new(RefCell::new(Vec::new()));
    let captured = actions.clone();
    ui.on_action(move |action| captured.borrow_mut().push(action.to_string()));
    ui.show().unwrap();
    window.set_size(LogicalSize::new(1240., 860.));
    let render = || {
        draw(&window);
        clock.set(clock.get() + Duration::from_millis(250));
        slint::platform::update_timers_and_animations();
        draw(&window);
    };
    for (theme, label) in [
        ("light", settings::appearance::LIGHT),
        ("dark", settings::appearance::DARK),
    ] {
        ui.set_dialog_kind("".into());
        ui.set_page(2);
        render();
        control(&ui, label).accessible_action(&AccessibilityAction::Default);
        ui.set_page(0);
        for (width, height) in [(1240., 860.), (700., 520.)] {
            window.set_size(LogicalSize::new(width, height));
            for (completed, guest_work) in
                [(false, true), (true, true), (false, false), (true, false)]
            {
                ui.set_dialog_kind(
                    if completed {
                        "install-complete"
                    } else {
                        "install-progress"
                    }
                    .into(),
                );
                super::dialog_presentation::begin(&ui, super::operation::Kind::Apply);
                if completed {
                    super::dialog_presentation::readback(&ui);
                    super::dialog_presentation::opened(&ui, "install-complete", true);
                }
                ui.set_running(!completed);
                ui.set_dialog_title(
                    if completed {
                        install::COMPLETED
                    } else {
                        install::APPLYING
                    }
                    .into(),
                );
                ui.set_install_guest_work(guest_work);
                ui.set_install_vm_name("win_26300.9457 VM1".into());
                ui.set_dialog_body(
                    if completed {
                        install::SUCCESS_OFF
                    } else if guest_work {
                        install::WAIT
                    } else {
                        hyper_gpu_support::ui_text::operations::apply::HOST_PROGRESS
                    }
                    .into(),
                );
                ui.set_current_stage(
                    if guest_work {
                        install::PREPARING_FILES
                    } else {
                        hyper_gpu_support::ui_text::progress::stage::AUTHENTICATE
                    }
                    .into(),
                );
                ui.set_transfer_fraction(if guest_work { 0.23 } else { -1.0 });
                ui.set_install_transfer(if guest_work { "Copying files: 799.9 MB of 3.4 GB (23%)\n82 of 273 files; verification pending" } else { "" }.into());
                ui.set_install_steps(super::model(
                    (0..if guest_work { 7 } else { 5 })
                        .map(|index| super::InstallStep {
                            label: if !guest_work && index == 4 {
                                "Finalising and shutting down"
                            } else if index < 4 {
                                [
                                    "Checking protected worker",
                                    "Checking VM and driver files",
                                    "Reading current VM state",
                                    "Reading selected GPU and driver version",
                                ][index]
                            } else {
                                [
                                    "Preparing GPU driver files",
                                    "Installing GPU driver files",
                                    "Finalising and shutting down",
                                ][index - 4]
                            }
                            .into(),
                            status: if index < 4 {
                                2
                            } else if index == 4 {
                                1
                            } else {
                                0
                            },
                            elapsed: "00:04".into(),
                        })
                        .collect(),
                ));
                ui.set_install_facts(super::model(vec![
                    super::InstallFact {
                        label: install::SUPPORT.into(),
                        value: "Enabled".into(),
                    },
                    super::InstallFact {
                        label: install::GPU.into(),
                        value: "NVIDIA GeForce RTX 5060".into(),
                    },
                    super::InstallFact {
                        label: install::MEMORY.into(),
                        value: "7 GB".into(),
                    },
                    super::InstallFact {
                        label: install::PREPARATION.into(),
                        value: if guest_work {
                            "Completed"
                        } else {
                            install::NOT_REQUIRED
                        }
                        .into(),
                    },
                    super::InstallFact {
                        label: install::CHECKS.into(),
                        value: if guest_work {
                            "Completed"
                        } else {
                            install::NOT_REQUIRED
                        }
                        .into(),
                    },
                ]));
                let log = (0..40)
                    .map(|index| format!("00:{index:02}   Checked driver files · Complete\n"))
                    .collect::<String>();
                ui.set_dialog_details(log.clone().into());
                render();
                retain_render(&format!(
                    "install-{theme}-{width}x{height}-{completed}-{guest_work}"
                ));
                let left = control(
                    &ui,
                    if completed {
                        install::VIEW_LOG
                    } else {
                        diagnostics::SHOW_LABEL
                    },
                );
                let right = control(
                    &ui,
                    if completed {
                        hyper_gpu_support::ui_text::dialogs::close::LABEL
                    } else {
                        install::CANCEL
                    },
                );
                let origin = right.map_to_window(right.geometry().origin);
                let left_origin = left.map_to_window(left.geometry().origin);
                assert!((origin.y - left_origin.y).abs() < 1., "inline footer");
                assert!(origin.y > 0. && origin.y + right.geometry().height() <= height);
                assert!(origin.x + right.geometry().width() <= width);
                actions.borrow_mut().clear();
                right.accessible_action(&AccessibilityAction::Default);
                assert_eq!(
                    actions.borrow().as_slice(),
                    if completed {
                        &["close"]
                    } else {
                        &[] as &[&str]
                    }
                );
                actions.borrow_mut().clear();
                ui.set_updating(true);
                right.accessible_action(&AccessibilityAction::Default);
                assert_eq!(
                    actions.borrow().as_slice(),
                    if completed {
                        &["close"]
                    } else {
                        &[] as &[&str]
                    },
                    "read-only refresh must not trap a terminal dialog"
                );
                ui.set_updating(false);
                left.accessible_action(&AccessibilityAction::Default);
                render();
                retain_render(&format!(
                    "install-log-{theme}-{width}x{height}-{completed}-{guest_work}"
                ));
                let mut outputs = Vec::new();
                ItemRc::new_root(WindowInner::from_pub(ui.window()).component())
                    .visit_descendants::<()>(|item| {
                        if item.accessible_role() == AccessibleRole::Groupbox
                            && item.accessible_string_property(AccessibleStringProperty::Label).as_deref()
                                == Some(hyper_gpu_support::ui_text::dashboard::diagnostics::OUTPUT_LABEL)
                        {
                            outputs.push(item.clone());
                        }
                        ControlFlow::Continue(())
                    });
                assert_eq!(outputs.len(), 1);
                let output = outputs.pop().unwrap();
                let output_top = output.map_to_window(output.geometry().origin).y;
                assert!(output_top >= 0.);
                assert!(
                    output_top + output.geometry().height() < origin.y,
                    "expanded log must be visible above the fixed footer"
                );
                control(&ui, install::COPY_LOG).accessible_action(&AccessibilityAction::Default);
                assert_eq!(*clipboard.borrow(), log);
                let toggle = control(
                    &ui,
                    if completed {
                        install::HIDE_LOG
                    } else {
                        diagnostics::HIDE_LABEL
                    },
                );
                toggle.accessible_action(&AccessibilityAction::Default);
                render();
                // Updating progress text must not collapse a log the user opened.
                control(
                    &ui,
                    if completed {
                        install::VIEW_LOG
                    } else {
                        diagnostics::SHOW_LABEL
                    },
                )
                .accessible_action(&AccessibilityAction::Default);
                ui.set_dialog_details(format!("{log}01:00   Final checks\n").into());
                render();
                control(&ui, install::COPY_LOG);
                ui.set_dialog_kind("".into());
            }
        }
    }
    ui.hide().unwrap();
}

#[test]
fn settings_gear_routes_to_existing_subpages_and_preserves_drafts() {
    use hyper_gpu_support::ui_text::{dashboard::navigation, dialogs, settings};
    use std::cell::RefCell;

    fn buttons(ui: &AppWindow, label: &str) -> Vec<ItemRc> {
        let root = ItemRc::new_root(WindowInner::from_pub(ui.window()).component());
        let mut found = Vec::new();
        root.visit_descendants::<()>(|item| {
            if item.accessible_role() == AccessibleRole::Button
                && item
                    .accessible_string_property(AccessibleStringProperty::Label)
                    .as_deref()
                    == Some(label)
            {
                found.push(item.clone());
            }
            ControlFlow::Continue(())
        });
        found
    }

    fn button(ui: &AppWindow, label: &str) -> ItemRc {
        let mut found = buttons(ui, label);
        assert_eq!(found.len(), 1, "one navigation button: {label}");
        found.pop().unwrap()
    }

    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(LayoutPlatform(window.clone()))).unwrap();
    let ui = AppWindow::new().unwrap();
    ui.set_startup_loading(false);
    ui.set_selected_id("12345678-1234-1234-1234-123456789abc".into());
    ui.set_dirty(true);
    ui.set_memory_input("5.9".into());
    let actions = Rc::new(RefCell::new(Vec::new()));
    let captured = actions.clone();
    ui.on_action(move |action| captured.borrow_mut().push(action.to_string()));
    ui.show().unwrap();

    for (width, height) in [(1240., 860.), (700., 520.)] {
        window.set_size(LogicalSize::new(width, height));
        ui.set_page(0);
        draw(&window);
        assert!(buttons(&ui, navigation::DASHBOARD).is_empty());
        assert!(buttons(&ui, navigation::SYSTEM).is_empty());
        assert!(buttons(&ui, navigation::ABOUT).is_empty());
        let gear = button(&ui, navigation::SETTINGS);
        let geometry = gear.geometry();
        let origin = gear.map_to_window(geometry.origin);
        assert!(origin.x > width - 100. && origin.x + geometry.width() <= width);
        assert!(origin.y >= 0. && origin.y + geometry.height() <= 64.);
        gear.accessible_action(&AccessibilityAction::Default);
        assert_eq!(ui.get_page(), 2);
        draw(&window);
        retain_render(&format!("settings-{width}"));
        gear.accessible_action(&AccessibilityAction::Default);
        assert_eq!(ui.get_page(), 0, "gear closes Settings");
        gear.accessible_action(&AccessibilityAction::Default);
        assert_eq!(ui.get_page(), 2);
        draw(&window);

        for (label, page) in [(navigation::SYSTEM, 1), (navigation::ABOUT, 3)] {
            let link = button(&ui, label);
            let origin = link.map_to_window(link.geometry().origin);
            assert!(link.is_visible());
            assert!(origin.x >= 28. && origin.x < width / 2.);
            assert!(origin.y > height - 110. && origin.y + link.geometry().height() < height - 30.);
            let close = button(&ui, dialogs::close::LABEL);
            let close_origin = close.map_to_window(close.geometry().origin);
            assert!(close_origin.x > width / 2.);
            assert!((close_origin.x + close.geometry().width() - (width - 28.)).abs() < 1.);
            assert!((close_origin.y - origin.y).abs() < 1., "one footer row");
            // Even direct assistive activation must respect a modal operation.
            ui.set_dialog_kind("more".into());
            // Deliver Slint's deferred modal-state change before the next input,
            // as the real event loop does between user interactions.
            slint::platform::update_timers_and_animations();
            link.accessible_action(&AccessibilityAction::Default);
            close.accessible_action(&AccessibilityAction::Default);
            assert_eq!(ui.get_page(), 2);
            ui.set_dialog_kind("".into());
            slint::platform::update_timers_and_animations();
            link.accessible_action(&AccessibilityAction::Default);
            assert_eq!(ui.get_page(), page);
            draw(&window);
            retain_render(&format!("settings-{page}-{width}"));
            let back = button(&ui, settings::navigation::BACK);
            ui.set_dialog_kind("more".into());
            slint::platform::update_timers_and_animations();
            back.accessible_action(&AccessibilityAction::Default);
            button(&ui, navigation::SETTINGS).accessible_action(&AccessibilityAction::Default);
            assert_eq!(ui.get_page(), page);
            ui.set_dialog_kind("".into());
            slint::platform::update_timers_and_animations();
            if page == 3 {
                button(&ui, settings::help::OPEN_LABEL)
                    .accessible_action(&AccessibilityAction::Default);
                assert_eq!(actions.borrow().as_slice(), &["shortcuts"]);
                actions.borrow_mut().clear();
            }
            back.accessible_action(&AccessibilityAction::Default);
            assert_eq!(ui.get_page(), 2);
            draw(&window);
            button(&ui, label).accessible_action(&AccessibilityAction::Default);
            assert_eq!(ui.get_page(), page);
            gear.accessible_action(&AccessibilityAction::Default);
            assert_eq!(ui.get_page(), 0, "gear also closes subpages");
            gear.accessible_action(&AccessibilityAction::Default);
            draw(&window);
        }
        button(&ui, dialogs::close::LABEL).accessible_action(&AccessibilityAction::Default);
        assert_eq!(ui.get_page(), 0);
        assert_eq!(ui.get_selected_id(), "12345678-1234-1234-1234-123456789abc");
        assert!(ui.get_dirty());
        assert_eq!(ui.get_memory_input(), "5.9");
        assert!(actions.borrow().is_empty(), "navigation emits no operation");
    }
    ui.hide().unwrap();
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
    use hyper_gpu_support::ui_text::dashboard::startup;
    for stage in [
        startup::INVENTORY,
        startup::CONFIGURATION,
        startup::RECOVERY,
        startup::IMPORT_RECOVERY,
        startup::ENROLLMENT_RECOVERY,
        startup::WORKER,
    ] {
        ui.set_startup_stage(stage.into());
        draw(&window);
        let label = control(&ui, stage);
        let origin = label.map_to_window(label.geometry().origin);
        assert!(label.is_visible());
        assert!(origin.y >= 0. && origin.y + label.geometry().height() <= 520.);
    }
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
fn all_dialog_families_share_reachable_modal_actions_and_copyable_details() {
    use hyper_gpu_support::ui_text::{diagnostics, dialogs, settings};
    use std::cell::RefCell;
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(LayoutPlatform(window.clone()))).unwrap();
    let ui = AppWindow::new().unwrap();
    ui.set_startup_loading(false);
    ui.set_vm_name("Reviewed VM".into());
    ui.set_execution_ready(true);
    ui.set_repair_allowed(true);
    ui.set_forget_allowed(true);
    let actions = Rc::new(RefCell::new(Vec::new()));
    let captured = actions.clone();
    ui.on_action(move |action| captured.borrow_mut().push(action.to_string()));
    ui.show().unwrap();
    window.set_size(LogicalSize::new(1240., 860.));
    for (theme, label) in [
        ("light", settings::appearance::LIGHT),
        ("dark", settings::appearance::DARK),
    ] {
        ui.set_dialog_kind("".into());
        ui.set_page(2);
        draw(&window);
        control(&ui, label).accessible_action(&AccessibilityAction::Default);
        ui.set_page(0);
        for (width, height) in [(1240., 860.), (700., 520.)] {
            window.set_size(LogicalSize::new(width, height));
            for kind in [
                "info",
                "error",
                "status-error",
                "switch",
                "exit",
                "more",
                "recovery-needed",
                "recovery-blocked",
                "recovery-ended",
                "progress",
                "cancelling",
                "operation-error",
                "operation-complete",
                "save",
                "profile-deleted",
                "reset-complete",
            ] {
                ui.set_dialog_kind("".into());
                ui.set_operation_dialog_active(false);
                ui.set_updating(false);
                ui.set_running(matches!(kind, "progress" | "cancelling"));
                ui.set_review_preparing(false);
                let operation = match kind {
                    "operation-error" | "operation-complete" => {
                        Some(super::operation::Kind::Verify)
                    }
                    "save" => Some(super::operation::Kind::SaveOnly),
                    "profile-deleted" => Some(super::operation::Kind::Forget),
                    "reset-complete" => Some(super::operation::Kind::Remove),
                    _ => None,
                };
                super::dialog_presentation::begin(
                    &ui,
                    operation.unwrap_or(super::operation::Kind::Read),
                );
                ui.set_operation_dialog_active(operation.is_some());
                let title = format!("{kind}: selected workflow");
                super::dialog(
                    &ui,
                    kind,
                    &title,
                    &"Details of this selected action.\n".repeat(8),
                );
                ui.set_dialog_details("Captured event details\n".repeat(40).into());
                draw(&window);
                assert!(control(&ui, &title).is_visible(), "actual title: {kind}");
                let dismiss = match kind {
                    "switch" => dialogs::switch::STAY_LABEL,
                    "exit" => dialogs::draft::KEEP_LABEL,
                    "forget-review" => dialogs::cancel::LABEL,
                    "progress" | "cancelling" => dialogs::busy::LABEL,
                    _ => dialogs::close::LABEL,
                };
                let close = control(&ui, dismiss);
                let bounds = close.geometry();
                let origin = close.map_to_window(bounds.origin);
                assert!(
                    origin.y >= 0. && origin.y + bounds.height() <= height,
                    "fixed footer: {kind}"
                );
                assert!(
                    origin.x >= 0. && origin.x + bounds.width() <= width,
                    "reachable dismiss: {kind}"
                );
                let panel_left = (width - 660_f32.min(width - 40.)) / 2.;
                assert!(
                    origin.x >= panel_left + 24.
                        && origin.x + bounds.width() <= width - panel_left - 24.,
                    "dismiss stays inside footer viewport: {kind}"
                );
                let toggle = if matches!(
                    kind,
                    "operation-error"
                        | "operation-complete"
                        | "save"
                        | "profile-deleted"
                        | "reset-complete"
                ) {
                    dialogs::install::VIEW_LOG
                } else {
                    diagnostics::SHOW_LABEL
                };
                control(&ui, toggle).accessible_action(&AccessibilityAction::Default);
                draw(&window);
                let copy = control(&ui, dialogs::install::COPY_LOG);
                let copy_origin = copy.map_to_window(copy.geometry().origin);
                assert!(
                    copy_origin.y >= 0. && copy_origin.y + copy.geometry().height() <= origin.y,
                    "log remains above footer: {kind}"
                );
                retain_render(&format!("dialog-{kind}-{theme}-{width}x{height}"));
                actions.borrow_mut().clear();
                close.accessible_action(&AccessibilityAction::Default);
                assert_eq!(
                    actions.borrow().len(),
                    usize::from(!matches!(kind, "progress" | "cancelling")),
                    "busy dismissal: {kind}"
                );
            }
        }
    }
    ui.hide().unwrap();
}

#[test]
fn operation_preparation_and_execution_have_no_review_confirmation_controls() {
    use hyper_gpu_support::ui_text::{dialogs, settings};
    use slint::Model;
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(LayoutPlatform(window.clone()))).unwrap();
    let ui = AppWindow::new().unwrap();
    ui.set_startup_loading(false);
    ui.show().unwrap();
    window.set_size(LogicalSize::new(1240., 860.));
    for theme in [settings::appearance::LIGHT, settings::appearance::DARK] {
        ui.set_dialog_kind("".into());
        ui.set_page(2);
        draw(&window);
        control(&ui, theme).accessible_action(&AccessibilityAction::Default);
        ui.set_page(0);
        for (width, height) in [(1240., 860.), (700., 520.)] {
            window.set_size(LogicalSize::new(width, height));
            for kind in [
                super::operation::Kind::PermissionPlan,
                super::operation::Kind::Apply,
                super::operation::Kind::Verify,
                super::operation::Kind::Install,
                super::operation::Kind::Forget,
                super::operation::Kind::Remove,
                super::operation::Kind::Reconcile,
            ] {
                super::dialog_presentation::begin(&ui, kind);
                ui.set_running(true);
                ui.set_updating(true);
                ui.set_review_preparing(kind.review_preparation());
                super::dialog(
                    &ui,
                    "install-progress",
                    super::dialog_presentation::task(kind),
                    "Preparing and running the selected request.",
                );
                draw(&window);
                let root = ItemRc::new_root(WindowInner::from_pub(ui.window()).component());
                let mut confirmations = 0;
                root.visit_descendants::<()>(|item| {
                    if item.accessible_role() == AccessibleRole::Checkbox {
                        confirmations += 1;
                    }
                    if item.accessible_role() == AccessibleRole::Button
                        && item
                            .accessible_string_property(AccessibleStringProperty::Label)
                            .as_deref()
                            == Some(dialogs::apply::LABEL)
                    {
                        confirmations += 1;
                    }
                    ControlFlow::Continue(())
                });
                assert_eq!(confirmations, 0, "no review controls: {kind:?}");
                assert!(
                    !ui.get_dialog_phases()
                        .iter()
                        .any(|phase| phase == dialogs::install::REVIEW)
                );
                let dismiss = control(
                    &ui,
                    if kind.review_preparation() {
                        dialogs::cancel::LABEL
                    } else {
                        dialogs::install::CANCEL
                    },
                );
                assert!(dismiss.is_visible());
                retain_render(&format!("direct-{kind:?}-{theme}-{width}x{height}"));
            }
        }
    }
    ui.hide().unwrap();
}

#[test]
fn follow_up_and_technical_actions_share_the_dialog_footer_except_more_actions() {
    use hyper_gpu_support::ui_text::{diagnostics, dialogs};
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(LayoutPlatform(window.clone()))).unwrap();
    let ui = AppWindow::new().unwrap();
    ui.set_startup_loading(false);
    ui.set_execution_ready(true);
    ui.set_repair_allowed(true);
    ui.set_reset_allowed(true);
    ui.set_forget_allowed(true);
    ui.set_dirty(true);
    ui.show().unwrap();
    window.set_size(LogicalSize::new(1240., 860.));
    for theme in [
        hyper_gpu_support::ui_text::settings::appearance::LIGHT,
        hyper_gpu_support::ui_text::settings::appearance::DARK,
    ] {
        ui.set_dialog_kind("".into());
        ui.set_page(2);
        draw(&window);
        control(&ui, theme).accessible_action(&AccessibilityAction::Default);
        ui.set_page(0);
        for (kind, mut labels) in [
            (
                "status-error",
                vec![dialogs::close::LABEL, dialogs::status::RETRY_LABEL],
            ),
            (
                "permission-saved",
                vec![dialogs::close::LABEL, dialogs::setup::CONTINUE_LABEL],
            ),
            (
                "profile-deleted",
                vec![dialogs::close::LABEL, dialogs::setup::NEW_REVIEW_LABEL],
            ),
            (
                "reset-complete",
                vec![
                    dialogs::close::LABEL,
                    dialogs::repair::REVIEW_LABEL,
                    dialogs::profile_deletion::DISCARD_REVIEW_LABEL,
                ],
            ),
            (
                "recovery-blocked",
                vec![
                    dialogs::close::LABEL,
                    dialogs::profile_deletion::LABEL,
                    dialogs::status::RETRY_LABEL,
                ],
            ),
        ] {
            ui.set_dialog_title(kind.into());
            ui.set_dialog_body("Review the selected action.".into());
            ui.set_dialog_details("Diagnostic section\n".repeat(30).into());
            ui.set_dialog_kind(kind.into());
            if matches!(kind, "profile-deleted" | "reset-complete") {
                labels.retain(|label| *label != dialogs::close::LABEL);
                labels.insert(0, hyper_gpu_support::ui_text::dialogs::install::VIEW_LOG);
                labels.push(dialogs::close::LABEL);
            } else {
                labels.retain(|label| *label != dialogs::close::LABEL);
                labels.insert(0, diagnostics::SHOW_LABEL);
                labels.push(dialogs::close::LABEL);
            }
            for (width, height) in [(1240., 860.), (700., 520.)] {
                window.set_size(LogicalSize::new(width, height));
                draw(&window);
                let mut row_y: Option<f32> = None;
                let mut previous_right = f32::NEG_INFINITY;
                for label in &labels {
                    let button = control(&ui, label);
                    let geometry = button.geometry();
                    let origin = button.map_to_window(geometry.origin);
                    assert!(geometry.width() > 0. && geometry.height() > 0.);
                    assert!(
                        origin.x >= previous_right,
                        "non-overlapping actions: {kind}"
                    );
                    assert!(origin.y >= 0. && origin.y + geometry.height() <= height);
                    if let Some(y) = row_y {
                        assert!((origin.y - y).abs() < 1., "single footer row: {kind}");
                    }
                    row_y = Some(origin.y);
                    previous_right = origin.x + geometry.width();
                }
            }
        }
        ui.set_dialog_kind("more".into());
        draw(&window);
        let verify = control(&ui, hyper_gpu_support::ui_text::operations::verify::TITLE);
        let repair = control(&ui, dialogs::repair::LABEL);
        assert!(
            repair.map_to_window(repair.geometry().origin).y
                > verify.map_to_window(verify.geometry().origin).y,
            "More Actions keeps its vertical choices"
        );
    }
    ui.hide().unwrap();
}

#[test]
fn cancelled_reads_leave_dialog_close_usable_while_new_operations_stay_disabled() {
    use hyper_gpu_support::ui_text::dialogs;
    use std::cell::RefCell;
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(LayoutPlatform(window.clone()))).unwrap();
    let ui = AppWindow::new().unwrap();
    ui.set_startup_loading(false);
    ui.set_execution_ready(true);
    ui.set_verify_allowed(true);
    ui.set_repair_allowed(true);
    ui.set_updating(true);
    let actions = Rc::new(RefCell::new(Vec::new()));
    let captured = actions.clone();
    ui.on_action(move |action| captured.borrow_mut().push(action.to_string()));
    ui.show().unwrap();
    window.set_size(LogicalSize::new(700., 520.));
    for kind in ["more", "info", "operation-complete", "operation-error"] {
        ui.set_running(false);
        ui.set_review_preparing(false);
        ui.set_dialog_kind(kind.into());
        draw(&window);
        actions.borrow_mut().clear();
        if kind == "more" {
            for label in [
                hyper_gpu_support::ui_text::operations::verify::title().as_str(),
                dialogs::repair::LABEL,
            ] {
                control(&ui, label).accessible_action(&AccessibilityAction::Default);
            }
            assert!(actions.borrow().is_empty(), "new work remains disabled");
        }
        control(&ui, dialogs::close::LABEL).accessible_action(&AccessibilityAction::Default);
        assert_eq!(
            &*actions.borrow(),
            &["close"],
            "dismiss while reading: {kind}"
        );
    }
    ui.hide().unwrap();
}
