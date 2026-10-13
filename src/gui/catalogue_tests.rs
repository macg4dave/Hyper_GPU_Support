//! Headless construction only. No native backend, rendering, discovery or effects.
use super::{AppWindow, UiText};
use slint::{
    ComponentHandle,
    platform::{
        Platform, PlatformError, WindowAdapter,
        software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
    },
};
use std::{path::Path, rc::Rc};

struct Headless;
impl Platform for Headless {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer))
    }
}

fn text(feature: &str, key: &str) -> String {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("resources/i18n/en-GB/{feature}.toml")),
    )
    .unwrap();
    let document: toml::Value = toml::from_str(&source).unwrap();
    ui_text::template::parse(document["messages"][key]["text"].as_str().unwrap())
        .unwrap()
        .into_iter()
        .map(|segment| match segment {
            ui_text::template::Segment::Literal(text) => text,
            _ => panic!("fixed property contains a parameter"),
        })
        .collect()
}

#[test]
fn catalogue_globals_initialize_in_two_independent_headless_windows() {
    slint::platform::set_platform(Box::new(Headless)).unwrap();
    let first = AppWindow::new().unwrap();
    let second = AppWindow::new().unwrap();
    let title = text("operations", "operations.verify.title");
    let unknown = text("common", "common.state.unknown");
    for window in [&first, &second] {
        assert_eq!(
            window.global::<UiText>().get_operations_verify_title(),
            title
        );
        assert_eq!(window.get_power(), unknown);
        assert_eq!(window.get_status(), unknown);
    }
    first.set_dialog_kind("more".into());
    assert!(second.get_dialog_kind().is_empty());
}
