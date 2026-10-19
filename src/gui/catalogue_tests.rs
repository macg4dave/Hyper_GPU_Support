//! Headless construction only. No native backend, rendering, discovery or effects.
use super::{AppWindow, UiText, apply_ui_text};
use hyper_gpu_support::ui_text;
use slint::{
    ComponentHandle,
    platform::{
        Platform, PlatformError, WindowAdapter,
        software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
    },
};
use std::rc::Rc;

pub(super) struct Headless;
impl Platform for Headless {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer))
    }
}

#[test]
fn catalogue_globals_initialize_in_two_independent_headless_windows() {
    slint::platform::set_platform(Box::new(Headless)).unwrap();
    let first = AppWindow::new().unwrap();
    let second = AppWindow::new().unwrap();
    for window in [&first, &second] {
        let global = window.global::<UiText>();
        // Generated defaults are the reference wording, before any language is applied.
        assert_eq!(global.get_locale(), ui_text::LOCALES[0].tag);
        assert_eq!(
            global.get_operations_verify_title(),
            ui_text::operations::verify::title()
        );
        assert_eq!(window.get_power(), ui_text::common::state::unknown());
        assert_eq!(window.get_status(), ui_text::common::state::unknown());
        for count in [0, 1, 3, i32::MAX] {
            assert_eq!(
                global.invoke_dashboard_vm_list_count(count),
                ui_text::dashboard::vm_list::count(count as usize).as_str(),
            );
        }
        // Inserted values are data: braces and quotes are never re-parsed or escaped away.
        let data = "λ \" \\{field_label}\n😀";
        assert_eq!(
            global.invoke_resources_field_accessible(data.into()),
            ui_text::resources::field::accessible(data).as_str()
        );
        // The Rust accessor takes a code-formatted value; the Slint function must insert data as-is.
        assert!(
            global
                .invoke_resources_default_suffix(data.into())
                .starts_with(data)
        );
    }
    first.set_dialog_kind("more".into());
    assert!(second.get_dialog_kind().is_empty());
}

#[test]
fn applying_the_active_language_sets_every_global_value() {
    slint::platform::set_platform(Box::new(Headless)).unwrap();
    let window = AppWindow::new().unwrap();
    let global = window.global::<UiText>();
    global.set_locale("".into());
    global.set_dashboard_apply_label("".into());
    apply_ui_text(&global);
    assert_eq!(global.get_locale(), ui_text::current().tag);
    assert_eq!(global.get_locale_index(), 0);
    assert_eq!(
        global.get_dashboard_apply_label(),
        ui_text::dashboard::apply::label()
    );
    assert_eq!(global.get_common_language_name(), ui_text::current().name);
}
