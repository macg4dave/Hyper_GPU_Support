//! Catalogue loading, validation and generation, using isolated temporary locale roots.
use std::{
    fs,
    path::{Path, PathBuf},
};
use ui_text::{Category, Forms, LANGUAGE_NAME_KEY, REFERENCE_LOCALE, load};

/// A temporary locale root removed on drop.
struct Root(PathBuf);

impl Root {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("ui-text-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn file(&self, locale: &str, feature: &str, text: &str) -> &Self {
        let directory = self.0.join(locale);
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join(format!("{feature}.toml")), text).unwrap();
        self
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const REFERENCE: &str = r#"
[language]
name = "English (United Kingdom)"

[vm]
title = "Virtual machines"

[vm.sign_in]
args = ["vm_name: text"]
text = "Sign in to {vm_name}"

[vm.count]
args = ["count: usize"]
plural = "count"
one = "{count} VM"
other = "{count} VMs"
"#;

fn reference(name: &str) -> Root {
    let root = Root::new(name);
    root.file(REFERENCE_LOCALE, "common", REFERENCE);
    root
}

fn errors(root: &Root) -> String {
    load(&root.0).unwrap_err().join("\n")
}

#[test]
fn repository_catalogue_is_valid() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../resources/i18n");
    let loaded = load(&root).unwrap_or_else(|e| panic!("{}", e.join("\n")));
    assert_eq!(loaded.catalogue.locales[0].tag, REFERENCE_LOCALE);
    assert!(
        loaded
            .catalogue
            .messages
            .iter()
            .any(|m| m.key == LANGUAGE_NAME_KEY)
    );
}

#[test]
fn translations_fall_back_per_message_with_a_warning() {
    let root = reference("fallback");
    root.file(
        "fr-FR",
        "common",
        "[language]\nname = \"Français (France)\"\n\n[vm]\ntitle = \"Machines virtuelles\"\n",
    );
    let loaded = load(&root.0).unwrap();
    let catalogue = &loaded.catalogue;
    assert_eq!(
        catalogue
            .locales
            .iter()
            .map(|l| l.tag.as_str())
            .collect::<Vec<_>>(),
        ["en-GB", "fr-FR"]
    );
    assert_eq!(catalogue.locales[1].name, "Français (France)");
    assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);
    let sign_in = catalogue
        .messages
        .iter()
        .find(|m| m.key == "common.vm.sign_in")
        .unwrap();
    assert_eq!(sign_in.forms[0], sign_in.forms[1]);
    let sources = catalogue.generate("app::ui_text");
    assert!(
        sources
            .rust
            .contains(r#"crate::ui_text::pick(&["Virtual machines", "Machines virtuelles"])"#)
    );
    assert!(
        sources
            .slint
            .contains("in-out property <string> common-vm-title: \"Virtual machines\";")
    );
    assert!(
        sources
            .slint_apply
            .contains("text.set_common_vm_title(t::common::vm::title().into());")
    );
}

#[test]
fn plural_forms_follow_each_language_rule() {
    let root = reference("plural");
    root.file(
        "fr-FR",
        "common",
        "[language]\nname = \"Français\"\n\n[vm.count]\none = \"{count} machine\"\nother = \"{count} machines\"\n",
    );
    let catalogue = load(&root.0).unwrap().catalogue;
    let count = catalogue
        .messages
        .iter()
        .find(|m| m.key == "common.vm.count")
        .unwrap();
    assert!(
        matches!(&count.forms[1], Forms::Plural(f) if f.iter().any(|(c, _)| *c == Category::One))
    );
    let sources = catalogue.generate("app::ui_text");
    assert!(
        sources
            .rust
            .contains("crate::ui_text::render_plural(u64::try_from(count).unwrap_or(u64::MAX)")
    );
    assert!(
        sources
            .slint
            .contains("public pure function common-vm-count(count: int) -> string")
    );
    assert!(sources.slint.contains("if locale-index == 1 {"));
    // French `one` covers 0 and 1.
    assert!(sources.slint.contains("(count >= 0 && count <= 1)"));

    let root = reference("plural-invalid");
    root.file(
        "fr-FR",
        "common",
        "[language]\nname = \"Français\"\n\n[vm.count]\nmany = \"{count}\"\nother = \"{count}\"\n",
    );
    assert!(errors(&root).contains("plural forms must be among: one, other"));
}

#[test]
fn rejects_translation_mismatches() {
    let cases = [
        ("[vm]\nunknown = \"x\"\n", "unknown key common.vm.unknown"),
        (
            "[vm]\nsign_in = \"Connexion à {vm}\"\n",
            "unknown placeholder {vm}",
        ),
        (
            "[vm]\nsign_in = \"Connexion\"\n",
            "argument vm_name is never used",
        ),
        (
            "[vm.sign_in]\nargs = [\"vm_name: text\"]\ntext = \"{vm_name}\"\n",
            "args is declared only in en-GB",
        ),
        ("[vm]\ntitle = \"open {brace\"\n", "unmatched opening brace"),
    ];
    for (index, (text, expected)) in cases.into_iter().enumerate() {
        let root = reference(&format!("mismatch-{index}"));
        root.file(
            "de-DE",
            "common",
            &format!("[language]\nname = \"Deutsch\"\n\n{text}"),
        );
        let found = errors(&root);
        assert!(found.contains(expected), "expected {expected:?} in {found}");
    }
}

#[test]
fn rejects_invalid_roots_and_reference_declarations() {
    let root = Root::new("no-reference");
    root.file("fr-FR", "common", "[language]\nname = \"Français\"\n");
    assert!(errors(&root).contains("reference locale en-GB is required"));

    let root = reference("unknown-language");
    root.file("xx-YY", "common", "[language]\nname = \"?\"\n");
    assert!(errors(&root).contains("no plural rule for this language"));

    let root = reference("missing-name");
    root.file("es-ES", "common", "[vm]\ntitle = \"Máquinas virtuales\"\n");
    assert!(errors(&root).contains("es-ES: common.language.name is required"));

    let root = reference("unknown-file");
    root.file("es-ES", "common", "[language]\nname = \"Español\"\n");
    root.file("es-ES", "extra", "[a]\nb = \"c\"\n");
    assert!(errors(&root).contains("unknown catalogue file"));

    let declarations = [
        (
            "[a.b]\nargs = [\"n: float\"]\ntext = \"{n}\"\n",
            "unknown type",
        ),
        (
            "[a.b]\nargs = [\"n: text\"]\nplural = \"n\"\nother = \"{n}\"\n",
            "plural must name a numeric argument",
        ),
        (
            "[a.b]\nargs = [\"n: usize\"]\nother = \"{n}\"\n",
            "plural forms need",
        ),
        (
            "[a.b]\nargs = [\"n: usize\"]\nplural = \"n\"\none = \"{n}\"\n",
            "plural forms must include `other`",
        ),
        ("[a]\ntype = \"x\"\n", "invalid or reserved key name"),
        (
            "[a]\nb_c = \"x\"\n[a.b]\nc = \"y\"\n",
            "map to the same Slint name",
        ),
    ];
    for (index, (text, expected)) in declarations.into_iter().enumerate() {
        let root = reference(&format!("declaration-{index}"));
        root.file(REFERENCE_LOCALE, "extra", text);
        let found = errors(&root);
        assert!(found.contains(expected), "expected {expected:?} in {found}");
    }
}
