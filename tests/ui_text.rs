//! Compile/runtime presentation contracts. Expected copy stays in TOML fixtures.
use hyper_gpu_support::{model::Power, ui_text as display};
use std::{collections::BTreeMap, path::Path};

#[test]
fn british_english_is_the_only_compiled_language() {
    assert_eq!(display::DEFAULT_LOCALE, "en-GB");
    assert_eq!(display::LOCALE, "en-GB");
    assert_eq!(display::AVAILABLE_LOCALES, &["en-GB"]);
}

fn entries(directory: &str) -> BTreeMap<String, String> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join(directory);
    let mut result = BTreeMap::new();
    for feature in ["common", "operations", "credentials"] {
        let source = std::fs::read_to_string(directory.join(format!("{feature}.toml"))).unwrap();
        let document: toml::Value = toml::from_str(&source).unwrap();
        for (key, value) in document["messages"].as_table().unwrap() {
            result.insert(key.clone(), value["text"].as_str().unwrap().to_owned());
        }
    }
    result
}

fn compose(text: &str, values: &[(&str, &str)]) -> String {
    ui_text::template::parse(text)
        .unwrap()
        .iter()
        .map(|segment| match segment {
            ui_text::template::Segment::Literal(text) => text.as_str(),
            ui_text::template::Segment::Parameter(name) => {
                values.iter().find(|(key, _)| *key == name).unwrap().1
            }
        })
        .collect()
}

#[test]
fn catalogue_active_bindings_match_resource_bytes_and_keep_insertions_as_data() {
    let messages = entries("resources/i18n/en-GB");
    for (key, actual) in [
        (
            "operations.verify.title",
            display::operations::verify::title(),
        ),
        ("common.power.off", display::power(&Power::Off)),
        ("common.power.running", display::power(&Power::Running)),
        ("common.state.unknown", display::power(&Power::Other(42))),
        (
            "credentials.native.caption",
            display::credentials::native::caption(),
        ),
        (
            "credentials.explanation.caption",
            display::credentials::explanation::caption(),
        ),
    ] {
        assert_eq!(actual.as_str(), compose(&messages[key], &[]));
    }
    for power in [Power::Off, Power::Running, Power::Other(42)] {
        let power = display::power(&power);
        assert_eq!(
            display::operations::apply::power_consent(&power).as_str(),
            compose(
                &messages["operations.apply.power_consent"],
                &[("final_power", power.as_str())]
            )
        );
    }
    let name = "λ \" \\{final_power}\n😀";
    let message = display::credentials::guest_sign_in::message(name);
    let expected = compose(
        &messages["credentials.guest_sign_in.message"],
        &[("vm_name", name)],
    );
    assert_eq!(message.as_str(), expected);
    assert_eq!(
        display::credentials::explanation::wrapper(&message).as_str(),
        compose(
            &messages["credentials.explanation.wrapper"],
            &[("guest_message", &expected)]
        )
    );
}

#[test]
fn catalogue_restored_baseline_is_verbatim() {
    let actual = entries("resources/i18n/en-GB");
    for (key, expected) in entries("tests/fixtures/ui-text/validator/baseline") {
        assert_eq!(actual[&key], expected, "{key}");
    }
}
