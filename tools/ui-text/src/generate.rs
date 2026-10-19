//! Source generation from a validated catalogue. Wording is emitted as escaped literals only;
//! catalogue text never becomes code.
use crate::{
    ArgKind, Catalogue, Category, Forms, Message, template::Segment, template::slint_literal,
};
use std::collections::BTreeMap;

const HEADER: &str = "// Generated from resources/i18n; edit the catalogue, not this file.\n";

/// Deterministic UTF-8 (LF) sources for the build script.
pub struct Sources {
    /// `ui_text.rs`: included by the library's `ui_text` module (runtime helpers live there).
    pub rust: String,
    /// `ui-text.slint`: the `UiText` global, initialised with reference wording.
    pub slint: String,
    /// `ui_text_apply.rs`: included where the Slint `UiText` type is in scope; defines
    /// `apply_ui_text`, which copies the active language into the global.
    pub slint_apply: String,
}

impl Catalogue {
    /// Generate all sources. `library` is the application's path to the runtime `ui_text`
    /// module as seen from the Slint-owning crate, for example `hyper_gpu_support::ui_text`.
    pub fn generate(&self, library: &str) -> Sources {
        Sources {
            rust: self.rust(),
            slint: self.slint(),
            slint_apply: self.slint_apply(library),
        }
    }

    fn rust(&self) -> String {
        let mut out = String::from(HEADER);
        out.push_str("/// Every compiled language; the reference language is first.\n");
        out.push_str(&format!(
            "pub static LOCALES: [Locale; {}] = [\n",
            self.locales.len()
        ));
        for locale in &self.locales {
            out.push_str(&format!(
                "    Locale {{ tag: {:?}, name: {:?} }},\n",
                locale.tag, locale.name
            ));
        }
        out.push_str("];\n\n/// Plural category of `n` in the language at `locale`.\n");
        out.push_str("#[allow(dead_code, unused_variables)]\n");
        out.push_str(
            "fn plural_category(locale: usize, n: u64) -> Category {\n    match locale {\n",
        );
        for (index, locale) in self.locales.iter().enumerate() {
            out.push_str(&format!(
                "        {index} => {},\n",
                locale.plural.rust_selector("n")
            ));
        }
        out.push_str("        _ => Category::Other,\n    }\n}\n");

        let mut root = Module::default();
        for message in &self.messages {
            let mut module = &mut root;
            let segments: Vec<&str> = message.key.split('.').collect();
            let Some((name, path)) = segments.split_last() else {
                continue;
            };
            for segment in path {
                module = module.modules.entry(*segment).or_default();
            }
            module.functions.insert(*name, message);
        }
        root.write_rust("", &mut out);
        out
    }

    fn slint(&self) -> String {
        let mut out = String::from(HEADER);
        out.push_str("export global UiText {\n");
        out.push_str("    // Index of the active language in the compiled list; set from Rust.\n");
        out.push_str("    in-out property <int> locale-index: 0;\n");
        out.push_str(&format!(
            "    in-out property <string> locale: {};\n",
            slint_literal(&self.locales[0].tag)
        ));
        for message in self.messages.iter().filter(|m| m.args.is_empty()) {
            let Forms::Text(segments) = &message.forms[0] else {
                continue;
            };
            out.push_str(&format!(
                "    in-out property <string> {}: {};\n",
                message.slint_name(),
                slint_concat(segments, &[])
            ));
        }
        for message in self.messages.iter().filter(|m| !m.args.is_empty()) {
            if message.args.iter().any(|a| a.kind == ArgKind::U64) {
                continue; // Slint has no 64-bit integer; these messages are Rust-only.
            }
            let parameters: Vec<_> = message
                .args
                .iter()
                .map(|a| {
                    let kind = match a.kind {
                        ArgKind::Usize | ArgKind::I32 => "int",
                        _ => "string",
                    };
                    format!("{}: {kind}", a.name)
                })
                .collect();
            out.push_str(&format!(
                "    public pure function {}({}) -> string {{\n",
                message.slint_name(),
                parameters.join(", ")
            ));
            let names: Vec<_> = message.args.iter().map(|a| a.name.as_str()).collect();
            for (index, forms) in message.forms.iter().enumerate().skip(1) {
                if forms != &message.forms[0] {
                    out.push_str(&format!("        if locale-index == {index} {{\n"));
                    self.slint_body(message, index, &names, "            ", &mut out);
                    out.push_str("        }\n");
                }
            }
            self.slint_body(message, 0, &names, "        ", &mut out);
            out.push_str("    }\n");
        }
        out.push_str("}\n");
        out
    }

    fn slint_body(
        &self,
        message: &Message,
        locale: usize,
        names: &[&str],
        indent: &str,
        out: &mut String,
    ) {
        match &message.forms[locale] {
            Forms::Text(segments) => {
                out.push_str(&format!(
                    "{indent}return {};\n",
                    slint_concat(segments, names)
                ));
            }
            Forms::Plural(forms) => {
                let count = &message.args[message.plural.unwrap_or_default()].name;
                let form = |category: Category| {
                    forms
                        .iter()
                        .find(|(c, _)| *c == category)
                        .or_else(|| forms.iter().find(|(c, _)| *c == Category::Other))
                        .map_or_else(String::new, |(_, s)| slint_concat(s, names))
                };
                for (category, condition) in self.locales[locale].plural.slint_conditions(count) {
                    if condition == "true" {
                        out.push_str(&format!("{indent}return {};\n", form(category)));
                        return;
                    }
                    out.push_str(&format!(
                        "{indent}if {condition} {{ return {}; }}\n",
                        form(category)
                    ));
                }
                out.push_str(&format!("{indent}return {};\n", form(Category::Other)));
            }
        }
    }

    fn slint_apply(&self, library: &str) -> String {
        let mut out = String::from(HEADER);
        out.push_str("/// Copy the active language's wording into the Slint `UiText` global.\n");
        out.push_str("pub(crate) fn apply_ui_text(text: &UiText<'_>) {\n");
        out.push_str(&format!("    use {library} as t;\n"));
        out.push_str(
            "    text.set_locale_index(i32::try_from(t::current_index()).unwrap_or(0));\n",
        );
        out.push_str("    text.set_locale(t::current().tag.into());\n");
        for message in self.messages.iter().filter(|m| m.args.is_empty()) {
            out.push_str(&format!(
                "    text.set_{}(t::{}().into());\n",
                message.slint_name().replace('-', "_"),
                message.key.replace('.', "::")
            ));
        }
        out.push_str("}\n");
        out
    }
}

#[derive(Default)]
struct Module<'a> {
    modules: BTreeMap<&'a str, Module<'a>>,
    functions: BTreeMap<&'a str, &'a Message>,
}

impl Module<'_> {
    fn write_rust(&self, path: &str, out: &mut String) {
        for (name, message) in &self.functions {
            rust_function(name, message, out);
        }
        for (name, module) in &self.modules {
            let path = if path.is_empty() {
                (*name).to_owned()
            } else {
                format!("{path}.{name}")
            };
            out.push_str(&format!("/// `{path}` messages.\npub mod {name} {{\n"));
            out.push_str("#[allow(unused_imports)]\nuse crate::ui_text::{Category, DisplayValue, Piece, UiMessage};\n");
            module.write_rust(&path, out);
            out.push_str("}\n");
        }
    }
}

fn rust_function(name: &str, message: &Message, out: &mut String) {
    out.push_str(&format!("/// `{}`\n", message.key));
    if message.args.is_empty() {
        let texts: Vec<_> = message
            .forms
            .iter()
            .map(|f| match f {
                Forms::Text(segments) => format!("{:?}", literal(segments)),
                Forms::Plural(_) => String::from("\"\""),
            })
            .collect();
        out.push_str(&format!(
            "pub fn {name}() -> &'static str {{\n    crate::ui_text::pick(&[{}])\n}}\n",
            texts.join(", ")
        ));
        return;
    }
    let parameters: Vec<_> = message
        .args
        .iter()
        .map(|a| {
            let kind = match a.kind {
                ArgKind::Text | ArgKind::Message => "&str",
                ArgKind::Formatted => "&DisplayValue",
                ArgKind::Usize => "usize",
                ArgKind::U64 => "u64",
                ArgKind::I32 => "i32",
            };
            format!("{}: {kind}", a.name)
        })
        .collect();
    if parameters.len() > 7 {
        // A message's arguments are its template's placeholders, not an API design choice.
        out.push_str("#[allow(clippy::too_many_arguments)]\n");
    }
    out.push_str(&format!(
        "pub fn {name}({}) -> UiMessage {{\n",
        parameters.join(", ")
    ));
    let mut values = Vec::new();
    for (index, arg) in message.args.iter().enumerate() {
        match arg.kind {
            ArgKind::Text | ArgKind::Message => values.push(arg.name.clone()),
            ArgKind::Formatted => values.push(format!("{}.as_str()", arg.name)),
            ArgKind::Usize | ArgKind::U64 | ArgKind::I32 => {
                out.push_str(&format!(
                    "    let __arg_{index} = {}.to_string();\n",
                    arg.name
                ));
                values.push(format!("&__arg_{index}"));
            }
        }
    }
    let names: Vec<_> = message.args.iter().map(|a| a.name.as_str()).collect();
    match message.plural {
        None => {
            let forms: Vec<_> = message
                .forms
                .iter()
                .map(|f| match f {
                    Forms::Text(segments) => rust_pieces(segments, &names),
                    Forms::Plural(_) => String::from("&[]"),
                })
                .collect();
            out.push_str(&format!(
                "    crate::ui_text::render(&[{}], &[{}])\n}}\n",
                forms.join(", "),
                values.join(", ")
            ));
        }
        Some(selector) => {
            let arg = &message.args[selector];
            let count = match arg.kind {
                ArgKind::Usize => format!("u64::try_from({}).unwrap_or(u64::MAX)", arg.name),
                ArgKind::I32 => format!("u64::from({}.unsigned_abs())", arg.name),
                _ => arg.name.clone(),
            };
            let forms: Vec<_> = message
                .forms
                .iter()
                .map(|f| match f {
                    Forms::Plural(forms) => format!(
                        "&[{}]",
                        forms
                            .iter()
                            .map(|(c, s)| format!(
                                "(Category::{}, {})",
                                c.variant(),
                                rust_pieces(s, &names)
                            ))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    Forms::Text(_) => String::from("&[]"),
                })
                .collect();
            out.push_str(&format!(
                "    crate::ui_text::render_plural({count}, &[{}], &[{}])\n}}\n",
                forms.join(", "),
                values.join(", ")
            ));
        }
    }
}

fn rust_pieces(segments: &[Segment], names: &[&str]) -> String {
    let pieces: Vec<_> = segments
        .iter()
        .map(|s| match s {
            Segment::Literal(text) => format!("Piece::Text({text:?})"),
            Segment::Argument(name) => {
                format!(
                    "Piece::Arg({})",
                    names.iter().position(|n| n == name).unwrap_or_default()
                )
            }
        })
        .collect();
    format!("&[{}]", pieces.join(", "))
}

fn literal(segments: &[Segment]) -> String {
    segments
        .iter()
        .filter_map(|s| match s {
            Segment::Literal(text) => Some(text.as_str()),
            Segment::Argument(_) => None,
        })
        .collect()
}

/// A Slint string expression concatenating literals and argument names.
fn slint_concat(segments: &[Segment], names: &[&str]) -> String {
    let parts: Vec<_> = segments
        .iter()
        .map(|s| match s {
            Segment::Literal(text) => slint_literal(text),
            Segment::Argument(name) if names.contains(&name.as_str()) => name.clone(),
            Segment::Argument(_) => String::from("\"\""),
        })
        .collect();
    match parts.as_slice() {
        [single] if single.starts_with('"') => single.clone(),
        _ => format!("\"\" + {}", parts.join(" + ")),
    }
}
