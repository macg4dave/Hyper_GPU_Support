//! Synthetic edits assert validation rules, not English catalogue expectations.
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use ui_text::{
    contract::{self, Message, Parameter, ParameterType, Placeholder},
    template::{self, Segment},
};

fn baseline() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(
        if Path::new(env!("CARGO_MANIFEST_DIR")).ends_with("standalone") {
            "../../../resources/i18n/en-GB"
        } else {
            "../../resources/i18n/en-GB"
        },
    )
}
fn documents() -> BTreeMap<String, Vec<u8>> {
    fs::read_dir(baseline())
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (
                entry.file_name().to_str().unwrap().into(),
                fs::read(entry.path()).unwrap(),
            )
        })
        .collect()
}
fn fixtures() -> PathBuf {
    baseline()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures/ui-text/validator")
}
fn change(
    documents: &mut BTreeMap<String, Vec<u8>>,
    file: &str,
    edit: impl FnOnce(&mut toml::Value),
) {
    let mut value = toml::from_str(std::str::from_utf8(&documents[file]).unwrap()).unwrap();
    edit(&mut value);
    documents.insert(file.into(), toml::to_string(&value).unwrap().into_bytes());
}
fn errors(documents: BTreeMap<String, Vec<u8>>, contract: &[Message]) -> Vec<String> {
    match ui_text::validate_documents(documents, contract) {
        Ok(_) => panic!("invalid input accepted"),
        Err(errors) => errors,
    }
}
fn entry<'a>(document: &'a mut toml::Value, key: &str) -> &'a mut toml::Value {
    &mut document["messages"][key]
}

#[test]
fn wide_numeric_values_cannot_be_narrowed_into_slint_functions() {
    let mut contract = contract::active();
    let count = contract
        .iter_mut()
        .find(|message| message.key == "dashboard.vm_list.count")
        .unwrap();
    count.parameters[0].declaration.kind = ParameterType::U64;
    let mut docs = documents();
    change(&mut docs, "dashboard.toml", |document| {
        entry(document, "dashboard.vm_list.count")["placeholders"][0]["type"] = "u64".into();
    });
    assert!(
        errors(docs, &contract)
            .iter()
            .any(|error| error.contains("u64 values must be formatted by Rust"))
    );
}

#[test]
fn baseline_validates_and_generates_deterministically_without_normalizing_text() {
    let docs = documents();
    let forward = ui_text::validate_documents(docs.clone(), &contract::active())
        .unwrap()
        .generate();
    let mut contract = contract::active();
    contract.reverse();
    let reverse = ui_text::validate_documents(docs.into_iter().rev(), &contract)
        .unwrap()
        .generate();
    assert_eq!(forward.rust, reverse.rust);
    assert_eq!(forward.slint, reverse.slint);
    assert!(!forward.rust.contains(env!("CARGO_MANIFEST_DIR")));
    assert!(!forward.slint.contains('\r'));
    let mut docs = documents();
    let edited = " \tλ 😀 \r\n\" \\ {{x}} \\{{root.action(1)}} ";
    change(&mut docs, "operations.toml", |doc| {
        entry(doc, "operations.verify.title")["text"] = edited.into()
    });
    let sources = ui_text::validate_documents(docs, &contract)
        .unwrap()
        .generate();
    let literal = edited.replace("{{", "{").replace("}}", "}");
    assert!(sources.rust.contains(&format!("{literal:?}")));
    assert!(sources.slint.contains(&template::slint_literal(&literal)));
    assert!(!sources.slint.contains(" \\{root.action(1)}"));
}

#[test]
fn strict_schema_duplicates_utf8_and_file_sets_fail_closed() {
    let mut duplicate_table = documents();
    duplicate_table
        .get_mut("common.toml")
        .unwrap()
        .extend_from_slice(b"\n[messages.\"common.power.off\"]\n");
    assert!(
        errors(duplicate_table, &contract::active())
            .iter()
            .any(|e| e.contains("duplicate"))
    );
    for field in ["schema", "locale", "feature", "messages"] {
        let mut docs = documents();
        change(&mut docs, "common.toml", |doc| {
            doc.as_table_mut().unwrap().remove(field);
        });
        assert!(
            errors(docs, &contract::active())
                .iter()
                .any(|e| e.contains("schema"))
        );
    }
    for field in [
        "text",
        "screen",
        "state",
        "audience",
        "intent",
        "consequences",
        "severity",
        "placeholders",
        "space_hint",
    ] {
        let mut docs = documents();
        change(&mut docs, "common.toml", |doc| {
            entry(doc, "common.power.off")
                .as_table_mut()
                .unwrap()
                .remove(field);
        });
        assert!(!errors(docs, &contract::active()).is_empty());
    }
    for fixture in ["malformed.toml", "duplicate-field.toml"] {
        let mut docs = documents();
        docs.insert(
            "common.toml".into(),
            fs::read(fixtures().join("invalid").join(fixture)).unwrap(),
        );
        assert!(
            errors(docs, &contract::active())
                .iter()
                .any(|e| e.contains("common.toml:schema:"))
        );
    }
    for bytes in [
        vec![0xff],
        [vec![0xef, 0xbb, 0xbf], documents()["common.toml"].clone()].concat(),
    ] {
        let mut docs = documents();
        docs.insert("common.toml".into(), bytes);
        assert!(
            errors(docs, &contract::active())
                .iter()
                .any(|e| e.contains("UTF-8"))
        );
    }
    for (field, value) in [
        ("schema", toml::Value::Integer(2)),
        ("locale", "x".into()),
        ("feature", "x".into()),
        ("extra", true.into()),
    ] {
        let mut docs = documents();
        change(&mut docs, "common.toml", |doc| {
            doc.as_table_mut().unwrap().insert(field.into(), value);
        });
        assert!(!errors(docs, &contract::active()).is_empty());
    }
    let mut docs = documents();
    docs.remove("common.toml");
    docs.insert("other.toml".into(), documents()["common.toml"].clone());
    let failures = errors(docs, &contract::active());
    assert!(failures.iter().any(|e| e.contains("missing required file")));
    assert!(failures.iter().any(|e| e.contains("unknown file")));
}

#[test]
fn keys_and_metadata_cannot_be_redefined_by_resources() {
    let metadata = [
        "screen",
        "state",
        "audience",
        "intent",
        "consequences",
        "severity",
        "space_hint",
    ];
    for field in metadata.into_iter().chain(["extra"]) {
        let mut docs = documents();
        change(&mut docs, "operations.toml", |doc| {
            entry(doc, "operations.verify.title")
                .as_table_mut()
                .unwrap()
                .insert(field.into(), "x".into());
        });
        assert!(
            errors(docs, &contract::active())
                .iter()
                .any(|e| e.contains(field))
        );
    }
    let mut docs = documents();
    change(&mut docs, "common.toml", |doc| {
        doc["messages"]
            .as_table_mut()
            .unwrap()
            .remove("common.power.off");
    });
    assert!(
        errors(docs, &contract::active())
            .iter()
            .any(|e| e.contains("missing required key"))
    );
    let mut docs = documents();
    change(&mut docs, "common.toml", |doc| {
        let copied = entry(doc, "common.power.off").clone();
        doc["messages"]
            .as_table_mut()
            .unwrap()
            .insert("common.extra".into(), copied);
    });
    assert!(
        errors(docs, &contract::active())
            .iter()
            .any(|e| e.contains("unknown key"))
    );
    let copied =
        toml::from_str::<toml::Value>(std::str::from_utf8(&documents()["common.toml"]).unwrap())
            .unwrap()["messages"]["common.power.off"]
            .clone();
    let mut docs = documents();
    change(&mut docs, "operations.toml", |doc| {
        doc["messages"]
            .as_table_mut()
            .unwrap()
            .insert("common.power.off".into(), copied);
    });
    let failures = errors(docs, &contract::active());
    assert!(
        failures
            .iter()
            .any(|e| e.contains("duplicate cross-file key"))
    );
    assert!(
        failures
            .iter()
            .any(|e| e.contains("wrong feature assignment"))
    );
    assert!(failures.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn placeholder_grammar_types_declarations_and_occurrence_limits_are_independent() {
    let fixture: toml::Value = toml::from_str(
        &fs::read_to_string(fixtures().join("invalid/invalid-templates.toml")).unwrap(),
    )
    .unwrap();
    for text in fixture["templates"].as_array().unwrap() {
        assert!(template::parse(text.as_str().unwrap()).is_err());
    }
    for text in ["{x}", "{final_power} {final_power}", "{{final_power}}"] {
        let mut docs = documents();
        change(&mut docs, "operations.toml", |doc| {
            entry(doc, "operations.apply.power_consent")["text"] = text.into()
        });
        assert!(!errors(docs, &contract::active()).is_empty());
    }
    for field in ["name", "type", "extra"] {
        let mut docs = documents();
        change(&mut docs, "operations.toml", |doc| {
            entry(doc, "operations.apply.power_consent")["placeholders"][0]
                .as_table_mut()
                .unwrap()
                .insert(field.into(), "x".into());
        });
        assert!(!errors(docs, &contract::active()).is_empty());
    }
    for field in ["name", "type"] {
        let mut docs = documents();
        change(&mut docs, "operations.toml", |doc| {
            entry(doc, "operations.apply.power_consent")["placeholders"][0]
                .as_table_mut()
                .unwrap()
                .remove(field);
        });
        assert!(!errors(docs, &contract::active()).is_empty());
    }
    let mut docs = documents();
    change(&mut docs, "operations.toml", |doc| {
        let p = entry(doc, "operations.apply.power_consent")["placeholders"][0].clone();
        entry(doc, "operations.apply.power_consent")["placeholders"]
            .as_array_mut()
            .unwrap()
            .push(p);
    });
    assert!(
        errors(docs, &contract::active())
            .iter()
            .any(|e| e.contains("duplicate name"))
    );
    let mut docs = documents();
    change(&mut docs, "operations.toml", |doc| {
        entry(doc, "operations.apply.power_consent")["text"] = "{final_power}".into();
        entry(doc, "operations.apply.power_consent")["placeholders"][0]["type"] = "text".into();
    });
    assert!(
        errors(docs, &contract::active())
            .iter()
            .any(|e| e.contains("names/types"))
    );
}

#[test]
fn repeated_reordered_parameters_and_code_like_input_are_appended_once_as_data() {
    let mut contract = contract::active();
    let spec = contract
        .iter_mut()
        .find(|m| m.key == "operations.apply.power_consent")
        .unwrap();
    spec.parameters[0].maximum = Some(2);
    spec.parameters.push(Parameter {
        declaration: Placeholder {
            name: "count".into(),
            kind: ParameterType::U64,
        },
        minimum: 1,
        maximum: Some(1),
        source: "Typed count".into(),
        maximum_utf16: Some(20),
    });
    let mut docs = documents();
    change(&mut docs, "operations.toml", |doc| {
        let message = entry(doc, "operations.apply.power_consent");
        message["text"] = "{count} {final_power} {{count}} {final_power}".into();
        let p = toml::from_str::<toml::Value>("name = 'count'\ntype = 'u64'").unwrap();
        message["placeholders"].as_array_mut().unwrap().insert(0, p);
    });
    let sources = ui_text::validate_documents(docs, &contract)
        .unwrap()
        .generate();
    assert!(
        sources
            .rust
            .contains("power_consent(final_power: &UiMessage, count: u64)")
    );
    let body = sources
        .rust
        .split_once("power_consent(final_power: &UiMessage, count: u64)")
        .unwrap()
        .1
        .split_once("\n}\n")
        .unwrap()
        .0;
    assert_eq!(body.matches("let count = count.to_string();").count(), 1);
    assert_eq!(
        body.matches("__ui_text_output.push_str(final_power.as_str());")
            .count(),
        2
    );
    assert!(body.contains(&format!("__ui_text_output.push_str({:?});", " {count} ")));
    let parsed = template::parse("{{x}} {x}").unwrap();
    let value = "{y} \" \\ λ\n";
    let actual: String = parsed
        .iter()
        .map(|s| match s {
            Segment::Literal(s) => s.as_str(),
            Segment::Parameter(_) => value,
        })
        .collect();
    assert_eq!(actual, format!("{{x}} {value}"));
}

#[test]
fn native_budget_counts_every_utf16_occurrence_and_the_terminator() {
    for (key, file, text, allowed) in [
        (
            "credentials.native.caption",
            "credentials.toml",
            "x".repeat(127),
            true,
        ),
        (
            "credentials.native.caption",
            "credentials.toml",
            "x".repeat(128),
            false,
        ),
        (
            "credentials.guest_sign_in.message",
            "credentials.toml",
            format!("{}{{vm_name}}", "😀".repeat(255)),
            true,
        ),
        (
            "credentials.guest_sign_in.message",
            "credentials.toml",
            format!("{}{{vm_name}}", "😀".repeat(256)),
            false,
        ),
    ] {
        let mut docs = documents();
        change(&mut docs, file, |doc| entry(doc, key)["text"] = text.into());
        assert_eq!(
            ui_text::validate_documents(docs, &contract::active()).is_ok(),
            allowed
        );
    }
    let mut contract = contract::active();
    let prompt = contract
        .iter_mut()
        .find(|m| m.key == "credentials.guest_sign_in.message")
        .unwrap();
    prompt.parameters[0].maximum = Some(2);
    let mut docs = documents();
    change(&mut docs, "credentials.toml", |doc| {
        entry(doc, "credentials.guest_sign_in.message")["text"] = "{vm_name}{vm_name}".into()
    });
    assert!(
        errors(docs, &contract)
            .iter()
            .any(|e| e.contains("native budget"))
    );
    let prompt = contract
        .iter_mut()
        .find(|m| m.key == "credentials.guest_sign_in.message")
        .unwrap();
    prompt.parameters[0].maximum_utf16 = None;
    assert!(
        errors(documents(), &contract)
            .iter()
            .any(|e| e.contains("unknown bound"))
    );
}

#[test]
fn generated_names_reject_module_leaf_reserved_and_slint_equivalence_collisions() {
    assert!(
        errors(BTreeMap::new(), &[])
            .iter()
            .any(|e| e.contains("must not be empty"))
    );
    for keys in [
        vec!["operations.a", "operations.a.b"],
        vec!["operations.a.b_c", "operations.a_b.c"],
        vec!["operations.type"],
        vec!["operations.a__b"],
        vec!["operations.a-b"],
    ] {
        let mut contract = contract::active();
        for key in keys {
            let mut spec = contract[0].clone();
            spec.key = key.into();
            contract.push(spec);
        }
        assert!(
            errors(documents(), &contract)
                .iter()
                .any(|e| e.starts_with("contract:"))
        );
    }
}

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "ui-text-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn copy(&self) {
        for (file, bytes) in documents() {
            fs::write(self.0.join(file), bytes).unwrap();
        }
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn locale_loading_defaults_and_falls_back_without_changing_catalogue_bytes() {
    let contract = contract::active();
    let root = baseline().parent().unwrap().to_owned();
    let original = ui_text::validate_directory(&baseline(), &contract)
        .unwrap()
        .generate();
    for requested in [
        None,
        Some("en-GB"),
        Some("unavailable"),
        Some("../../outside"),
    ] {
        let loaded = ui_text::locales::load(&root, requested, &contract).unwrap();
        assert_eq!(loaded.locale, "en-GB");
        assert_eq!(loaded.available, ["en-GB"]);
        let unchanged = loaded.catalogue.generate();
        assert_eq!(original.rust, unchanged.rust);
        assert_eq!(original.slint, unchanged.slint);
        let generated = loaded.generate();
        assert!(
            generated
                .rust
                .contains("pub const LOCALE: &str = \"en-GB\"")
        );
        assert!(
            generated
                .slint
                .contains("out property <string> locale: \"en-GB\"")
        );
    }
    // A translation header cannot pick another locale or change fallback policy.
    assert!(ui_text::validate_locale_documents(documents(), &contract, "unavailable").is_err());
    // Exercise locale-independent validation in memory with unchanged English.
    // This private-use test tag is never installed or made available in the app.
    let mut tagged = documents();
    for file in tagged.keys().cloned().collect::<Vec<_>>() {
        change(&mut tagged, &file, |document| {
            document["locale"] = "en-GB-x-test".into();
        });
    }
    let tagged = ui_text::validate_locale_documents(tagged, &contract, "en-GB-x-test")
        .unwrap()
        .generate();
    assert_eq!(original.rust, tagged.rust);
    assert_eq!(original.slint, tagged.slint);
}

#[test]
fn invalid_locale_roots_or_catalogues_never_fall_back_silently() {
    let root = Temp::new();
    let contract = contract::active();
    assert!(ui_text::locales::load(&root.0, None, &contract).is_err());
    let locale = root.0.join("en-GB");
    fs::create_dir(&locale).unwrap();
    for (file, bytes) in documents() {
        fs::write(locale.join(file), bytes).unwrap();
    }
    assert!(ui_text::locales::load(&root.0, None, &contract).is_ok());
    let destination = Temp::new();
    let output = destination.0.join("generated");
    let invoke = || {
        Command::new(env!("CARGO_BIN_EXE_ui-text"))
            .arg(&root.0)
            .args(["--locale", "unavailable", "--generate"])
            .arg(&output)
            .output()
            .unwrap()
    };
    assert!(invoke().status.success());
    let before = fs::read(output.join("ui_text.rs")).unwrap();
    let common = fs::read(locale.join("common.toml")).unwrap();
    fs::write(locale.join("common.toml"), b"invalid TOML").unwrap();
    assert!(ui_text::locales::load(&root.0, Some("unavailable"), &contract).is_err());
    assert!(!invoke().status.success());
    assert_eq!(fs::read(output.join("ui_text.rs")).unwrap(), before);
    fs::write(locale.join("common.toml"), common).unwrap();
    fs::write(root.0.join("unexpected.toml"), b"").unwrap();
    assert!(ui_text::locales::load(&root.0, None, &contract).is_err());
    fs::remove_file(root.0.join("unexpected.toml")).unwrap();
    fs::remove_file(locale.join("common.toml")).unwrap();
    assert!(ui_text::locales::load(&root.0, None, &contract).is_err());
}

#[test]
fn links_and_windows_reparse_directories_are_rejected_without_reading_targets() {
    let catalogue = Temp::new();
    catalogue.copy();
    let target = Temp::new();
    let link = catalogue.0.join("linked");
    #[cfg(windows)]
    {
        let result = Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&link)
            .arg(&target.0)
            .output()
            .unwrap();
        assert!(result.status.success());
    }
    #[cfg(not(windows))]
    std::os::unix::fs::symlink(&target.0, &link).unwrap();
    assert!(ui_text::validate_directory(&catalogue.0, &contract::active()).is_err());
    assert!(ui_text::validate_directory(&link, &contract::active()).is_err());
    assert!(ui_text::locales::load(&link, None, &contract::active()).is_err());
    #[cfg(windows)]
    fs::remove_dir(&link).unwrap();
    #[cfg(not(windows))]
    fs::remove_file(&link).unwrap();
    assert!(target.0.is_dir());
    assert!(ui_text::validate_directory(&catalogue.0, &contract::active()).is_ok());
}

#[test]
fn filesystem_and_cli_failures_do_not_produce_partial_generated_outputs() {
    let temp = Temp::new();
    temp.copy();
    let destination = Temp::new();
    let output = destination.0.join("output");
    let result = Command::new(env!("CARGO_BIN_EXE_ui-text"))
        .arg(&temp.0)
        .arg("--generate")
        .arg(&output)
        .output()
        .unwrap();
    assert!(result.status.success());
    assert!(output.join("ui_text.rs").is_file());
    assert!(ui_text::validate_directory(&temp.0, &contract::active()).is_ok());
    let nested = temp.0.join("output");
    let result = Command::new(env!("CARGO_BIN_EXE_ui-text"))
        .arg(&temp.0)
        .arg("--generate")
        .arg(&nested)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!nested.exists());
    fs::remove_dir_all(&output).unwrap();
    fs::write(temp.0.join("other.txt"), b"x").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_ui-text"))
        .arg(&temp.0)
        .arg("--generate")
        .arg(&output)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!output.exists());
    fs::remove_file(temp.0.join("other.txt")).unwrap();
    fs::create_dir(temp.0.join("nested")).unwrap();
    assert!(ui_text::validate_directory(&temp.0, &contract::active()).is_err());
    fs::remove_dir(temp.0.join("nested")).unwrap();
    fs::rename(temp.0.join("common.toml"), temp.0.join("common.TOML")).unwrap();
    assert!(ui_text::validate_directory(&temp.0, &contract::active()).is_err());
    assert!(ui_text::validate_directory(&temp.0.join("missing"), &contract::active()).is_err());
}
