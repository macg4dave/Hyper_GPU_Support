//! Source serialization uses parsed segments, never resource syntax as code.
use crate::{
    Catalogue, ValidMessage,
    contract::{Consumer, ParameterType},
    template::{Segment, slint_literal},
};
use std::collections::BTreeMap;

/// Deterministic UTF-8 source files ready for build integration, without timestamps.
pub struct Sources {
    /// Contents of `ui_text.rs`.
    pub rust: String,
    /// Contents of `ui-text.slint`.
    pub slint: String,
}

#[derive(Default)]
struct Module<'a> {
    modules: BTreeMap<&'a str, Module<'a>>,
    functions: BTreeMap<&'a str, &'a ValidMessage>,
}

fn function(name: &str, message: &ValidMessage, output: &mut String) {
    let arguments = message
        .contract
        .parameters
        .iter()
        .map(|p| {
            let kind = match p.declaration.kind {
                ParameterType::Text => "&str",
                ParameterType::U64 => "u64",
                ParameterType::Usize => "usize",
                ParameterType::I32 => "i32",
                ParameterType::Message => "&UiMessage",
                ParameterType::Formatted => "&DisplayValue",
            };
            format!("{}: {kind}", p.declaration.name)
        })
        .collect::<Vec<_>>()
        .join(", ");
    output.push_str(&format!(
        "/// Compiled catalogue message.\npub fn {name}({arguments}) -> UiMessage {{\n"
    ));
    if message.contract.parameters.is_empty() {
        let text: String = message
            .segments
            .iter()
            .filter_map(|s| match s {
                Segment::Literal(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        // Stable borrows for existing &str presenters; still one catalogue source.
        output.push_str(&format!(
            "UiMessage(::std::borrow::Cow::Borrowed({}))\n",
            name.to_uppercase()
        ));
        output.push_str("}\n");
        output.push_str(&format!(
            "/// Compiled fixed display value.\npub const {}: &str = {text:?};\n",
            name.to_uppercase()
        ));
        return;
    } else {
        for parameter in &message.contract.parameters {
            if matches!(
                parameter.declaration.kind,
                ParameterType::U64 | ParameterType::Usize | ParameterType::I32
            ) {
                let name = &parameter.declaration.name;
                output.push_str(&format!("let {name} = {name}.to_string();\n"));
            }
        }
        output.push_str("let mut __ui_text_output = String::new();\n");
        for segment in &message.segments {
            match segment {
                Segment::Literal(text) => {
                    let mut characters = text.chars();
                    if let Some(character) = characters.next()
                        && characters.next().is_none()
                    {
                        output.push_str(&format!("__ui_text_output.push({character:?});\n"));
                    } else {
                        output.push_str(&format!("__ui_text_output.push_str({text:?});\n"));
                    }
                }
                Segment::Parameter(name) => {
                    let parameter = message
                        .contract
                        .parameters
                        .iter()
                        .find(|p| p.declaration.name == *name)
                        .expect("validated parameter");
                    let expression = match parameter.declaration.kind {
                        ParameterType::Text => name.clone(),
                        ParameterType::Message | ParameterType::Formatted => {
                            format!("{name}.as_str()")
                        }
                        _ => format!("&{name}"),
                    };
                    output.push_str(&format!("__ui_text_output.push_str({expression});\n"));
                }
            }
        }
        output.push_str("UiMessage(::std::borrow::Cow::Owned(__ui_text_output))\n");
    }
    output.push_str("}\n");
}

fn render_module(node: &Module<'_>, output: &mut String) {
    for (name, message) in &node.functions {
        function(name, message, output);
    }
    for (name, child) in &node.modules {
        output.push_str(&format!(
            "/// Catalogue feature or branch.\npub mod {name} {{\n#[allow(unused_imports)]\nuse super::{{UiMessage, DisplayValue}};\n"
        ));
        render_module(child, output);
        output.push_str("}\n");
    }
}

pub(crate) fn sources(catalogue: &Catalogue, locale: Option<&str>) -> Sources {
    let mut tree = Module::default();
    let mut slint =
        String::from("// Generated; edit resource text, not this file.\nexport global UiText {\n");
    if let Some(locale) = locale {
        slint.push_str(&format!(
            "    out property <string> locale: {};\n",
            slint_literal(locale)
        ));
    }
    for (key, message) in &catalogue.messages {
        let mut parts: Vec<_> = key.split('.').collect();
        let name = parts.pop().expect("validated key");
        let mut parent = &mut tree;
        for part in parts {
            parent = parent.modules.entry(part).or_default();
        }
        parent.functions.insert(name, message);
        if message.contract.parameters.is_empty() {
            let text: String = message
                .segments
                .iter()
                .filter_map(|s| match s {
                    Segment::Literal(s) => Some(s.as_str()),
                    _ => None,
                })
                .collect();
            slint.push_str(&format!(
                "    out property <string> {}: {};\n",
                message.contract.slint_name(),
                slint_literal(&text)
            ));
        } else if matches!(message.contract.consumer, Consumer::SlintParameterised) {
            let arguments = message
                .contract
                .parameters
                .iter()
                .map(|parameter| {
                    let kind = match parameter.declaration.kind {
                        ParameterType::Usize | ParameterType::I32 => "int",
                        _ => "string",
                    };
                    format!("{}: {kind}", parameter.declaration.name)
                })
                .collect::<Vec<_>>()
                .join(", ");
            let expression = std::iter::once(slint_literal(""))
                .chain(message.segments.iter().map(|segment| match segment {
                    Segment::Literal(text) => slint_literal(text),
                    Segment::Parameter(name) => name.clone(),
                }))
                .collect::<Vec<_>>()
                .join(" + ");
            slint.push_str(&format!(
                "    public pure function {}({arguments}) -> string {{ return {expression}; }}\n",
                message.contract.slint_name(),
            ));
        }
    }
    slint.push_str("}\n");
    let mut rust = String::from(
        "// Generated; edit resource text, not this file.\n\
/// Catalogue-owned display text, with no unchecked public constructor.\n\
pub struct UiMessage(::std::borrow::Cow<'static, str>);\n\
impl UiMessage {\n\
/// Borrow the composed display value.\n\
pub fn as_str(&self) -> &str { &self.0 }\n\
/// Convert to owned text for an existing presenter/model.\n\
pub fn into_string(self) -> String { self.0.into_owned() }\n}\n\
impl ::std::fmt::Display for UiMessage { fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result { f.write_str(self.as_str()) } }\n\
/// A value created only by code-owned numeric or typed-fact presenters.\n\
pub struct DisplayValue(String);\n\
impl DisplayValue {\n\
/// Borrow the prepared display value.\n\
pub fn as_str(&self) -> &str { &self.0 }\n}\n",
    );
    render_module(&tree, &mut rust);
    Sources { rust, slint }
}
