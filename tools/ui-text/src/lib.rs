//! Strict catalogue validation shared by contributor CLI and future build generation.
pub mod contract;
mod generate;
pub mod template;

use contract::{Consumer, Message, Metadata, Placeholder};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
use template::Segment;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema: u32,
    locale: String,
    feature: String,
    messages: BTreeMap<String, toml::Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    text: String,
    screen: String,
    state: String,
    audience: String,
    intent: String,
    consequences: String,
    severity: String,
    placeholders: Vec<Placeholder>,
    space_hint: String,
}
impl Entry {
    fn metadata(&self) -> Metadata {
        Metadata {
            screen: self.screen.clone(),
            state: self.state.clone(),
            audience: self.audience.clone(),
            intent: self.intent.clone(),
            consequences: self.consequences.clone(),
            severity: self.severity.clone(),
            space_hint: self.space_hint.clone(),
        }
    }
}

/// Sorted actionable errors. Resource values are not printed as diagnostic context.
pub type Errors = Vec<String>;

pub(crate) struct ValidMessage {
    pub contract: Message,
    pub segments: Vec<Segment>,
}
/// Only successful whole-set validation constructs this value; no partial fallback.
pub struct Catalogue {
    pub(crate) messages: BTreeMap<String, ValidMessage>,
}

impl Catalogue {
    /// Number of entries in the exact validated active set.
    pub fn len(&self) -> usize {
        self.messages.len()
    }
    /// Whether the validated contract has no active messages.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
    /// Deterministic source bytes. Compilation/integration proof belongs to LANG-004.
    pub fn generate(&self) -> generate::Sources {
        generate::sources(self)
    }
}
pub use generate::Sources;

const FEATURES: &[&str] = &[
    "common",
    "dashboard",
    "system",
    "settings",
    "resources",
    "dialogs",
    "operations",
    "progress",
    "diagnostics",
    "credentials",
];
const RESERVED: &[&str] = &[
    "as",
    "async",
    "await",
    "break",
    "const",
    "continue",
    "crate",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "gen",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "abstract",
    "become",
    "box",
    "do",
    "final",
    "macro",
    "override",
    "priv",
    "typeof",
    "unsized",
    "virtual",
    "yield",
    "try",
    "component",
    "global",
    "property",
    "callback",
    "export",
    "import",
    "inherits",
];

fn contract_errors(contract: &[Message], errors: &mut BTreeSet<String>) {
    if contract.is_empty() {
        errors.insert("contract: active message set must not be empty".into());
    }
    let mut keys = BTreeSet::new();
    let mut slint = BTreeMap::new();
    for message in contract {
        let key = &message.key;
        if matches!(message.consumer, Consumer::Fixed) != message.parameters.is_empty() {
            errors.insert(format!(
                "contract:{key}: consumer kind does not match parameters"
            ));
        }
        let parts: Vec<_> = key.split('.').collect();
        if parts.len() < 2
            || parts
                .iter()
                .any(|p| !template::identifier(p) || RESERVED.contains(p))
            || !FEATURES.contains(&message.feature())
        {
            errors.insert(format!(
                "contract:{key}: invalid key or reserved generated identifier"
            ));
        }
        if !keys.insert(key) {
            errors.insert(format!("contract:{key}: duplicate key"));
        }
        if let Some(previous) = slint.insert(message.slint_name(), key) {
            errors.insert(format!(
                "contract:{key}: Slint symbol collides with {previous}"
            ));
        }
        let metadata = &message.metadata;
        if [
            &metadata.screen,
            &metadata.state,
            &metadata.intent,
            &metadata.consequences,
            &metadata.space_hint,
        ]
        .iter()
        .any(|s| s.trim().is_empty())
            || !["operator", "administrator"].contains(&metadata.audience.as_str())
            || !["neutral", "information", "warning", "error"].contains(&metadata.severity.as_str())
        {
            errors.insert(format!("contract:{key}: invalid metadata"));
        }
        let mut parameters = BTreeSet::new();
        for parameter in &message.parameters {
            let name = &parameter.declaration.name;
            if !template::identifier(name)
                || RESERVED.contains(&name.as_str())
                || !parameters.insert(name)
                || parameter.minimum == 0
                || parameter.maximum.is_some_and(|max| max < parameter.minimum)
                || parameter.source.trim().is_empty()
            {
                errors.insert(format!("contract:{key}:{name}: invalid parameter contract"));
            }
        }
    }
    for key in &keys {
        let parts: Vec<_> = key.split('.').collect();
        for end in 1..parts.len() {
            let prefix = parts[..end].join(".");
            if keys.contains(&prefix) {
                errors.insert(format!(
                    "contract:{key}: Rust module/function collision with {prefix}"
                ));
            }
        }
    }
}

/// Validate decoded inputs without reading resource text during tool compilation.
/// File names and message order do not affect either errors or generated bytes.
pub fn validate_documents(
    documents: impl IntoIterator<Item = (String, Vec<u8>)>,
    contract: &[Message],
) -> Result<Catalogue, Errors> {
    let mut errors = BTreeSet::new();
    contract_errors(contract, &mut errors);
    let required_files: BTreeSet<_> = contract
        .iter()
        .map(|c| format!("{}.toml", c.feature()))
        .collect();
    let expected: BTreeMap<_, _> = contract.iter().map(|c| (c.key.as_str(), c)).collect();
    let mut files = BTreeSet::new();
    let mut keys = BTreeSet::new();
    let mut messages = BTreeMap::new();
    for (file, bytes) in documents {
        if !files.insert(file.clone()) {
            errors.insert(format!("{file}: duplicate file input"));
        }
        if !required_files.contains(&file) {
            errors.insert(format!("{file}: unknown file"));
        }
        let text = match std::str::from_utf8(&bytes) {
            Ok(text) if !text.starts_with('\u{feff}') => text,
            _ => {
                errors.insert(format!("{file}: strict UTF-8 without BOM required"));
                continue;
            }
        };
        let document: Document = match toml::from_str(text) {
            Ok(document) => document,
            Err(error) => {
                errors.insert(format!("{file}:schema: {}", error.message()));
                continue;
            }
        };
        if document.schema != 1 {
            errors.insert(format!("{file}:schema: expected 1"));
        }
        if document.locale != "en-GB" {
            errors.insert(format!("{file}:locale: expected en-GB"));
        }
        if file != format!("{}.toml", document.feature) {
            errors.insert(format!("{file}:feature: must equal filename stem"));
        }
        for (key, entry) in document.messages {
            let at = format!("{file}:{key}");
            if !keys.insert(key.clone()) {
                errors.insert(format!("{at}: duplicate cross-file key"));
            }
            if key.split('.').next() != Some(document.feature.as_str()) {
                errors.insert(format!("{at}: wrong feature assignment"));
            }
            let Some(&spec) = expected.get(key.as_str()) else {
                errors.insert(format!("{at}: unknown key"));
                continue;
            };
            let entry: Entry = match entry.try_into() {
                Ok(entry) => entry,
                Err(error) => {
                    errors.insert(format!("{at}:schema: {}", error.message()));
                    continue;
                }
            };
            let metadata = entry.metadata();
            let expected_metadata = &spec.metadata;
            for (field, value, expected) in [
                ("screen", &metadata.screen, &expected_metadata.screen),
                ("state", &metadata.state, &expected_metadata.state),
                ("audience", &metadata.audience, &expected_metadata.audience),
                ("intent", &metadata.intent, &expected_metadata.intent),
                (
                    "consequences",
                    &metadata.consequences,
                    &expected_metadata.consequences,
                ),
                ("severity", &metadata.severity, &expected_metadata.severity),
                (
                    "space_hint",
                    &metadata.space_hint,
                    &expected_metadata.space_hint,
                ),
            ] {
                if value != expected {
                    errors.insert(format!("{at}:{field}: differs from code contract"));
                }
            }
            let mut declared = BTreeMap::new();
            for placeholder in entry.placeholders {
                let name = placeholder.name;
                if declared.insert(name.clone(), placeholder.kind).is_some() {
                    errors.insert(format!("{at}:placeholders:{name}: duplicate name"));
                }
            }
            let wanted: BTreeMap<_, _> = spec
                .parameters
                .iter()
                .map(|p| (p.declaration.name.clone(), p.declaration.kind))
                .collect();
            if declared != wanted {
                errors.insert(format!(
                    "{at}:placeholders: names/types differ from code contract"
                ));
            }
            let segments = match template::parse(&entry.text) {
                Ok(segments) => segments,
                Err(error) => {
                    errors.insert(format!("{at}:text: {error}"));
                    continue;
                }
            };
            let mut counts = BTreeMap::<&str, usize>::new();
            let mut literal_units = 1usize; // Native terminating NUL.
            for segment in &segments {
                match segment {
                    Segment::Literal(text) => {
                        literal_units = literal_units.saturating_add(text.encode_utf16().count());
                    }
                    Segment::Parameter(name) => {
                        *counts.entry(name).or_default() += 1;
                    }
                }
            }
            for name in counts.keys() {
                if !wanted.contains_key(*name) {
                    errors.insert(format!("{at}:text:{name}: undeclared placeholder"));
                }
            }
            let mut units = Some(literal_units);
            for parameter in &spec.parameters {
                let name = parameter.declaration.name.as_str();
                let count = counts.get(name).copied().unwrap_or(0);
                if count < parameter.minimum || parameter.maximum.is_some_and(|max| count > max) {
                    errors.insert(format!(
                        "{at}:text:{name}: occurrence count outside contract"
                    ));
                }
                units = units
                    .zip(parameter.maximum_utf16)
                    .and_then(|(units, bound)| {
                        count
                            .checked_mul(bound)
                            .and_then(|extra| units.checked_add(extra))
                    });
            }
            if let Some(budget) = spec.native_budget
                && units.is_none_or(|units| units > budget)
            {
                errors.insert(format!("{at}:text: unknown bound or UTF-16 native budget exceeded (including terminator)"));
            }
            messages.insert(
                key,
                ValidMessage {
                    contract: spec.clone(),
                    segments,
                },
            );
        }
    }
    for file in required_files.difference(&files) {
        errors.insert(format!("{file}: missing required file"));
    }
    for key in expected.keys() {
        if !keys.contains(*key) {
            errors.insert(format!(
                "{}.toml:{key}: missing required key",
                expected[key].feature()
            ));
        }
    }
    if errors.is_empty() {
        Ok(Catalogue { messages })
    } else {
        Err(errors.into_iter().collect())
    }
}

fn linked(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

/// Read only flat lowercase TOML regular files, rejecting links and reparse points.
pub fn validate_directory(directory: &Path, contract: &[Message]) -> Result<Catalogue, Errors> {
    let metadata =
        fs::symlink_metadata(directory).map_err(|e| vec![format!("catalogue directory: {e}")])?;
    if linked(&metadata) || !metadata.is_dir() {
        return Err(vec![
            "catalogue directory: regular directory required; links/reparse points forbidden"
                .into(),
        ]);
    }
    let mut documents = Vec::new();
    let mut errors = BTreeSet::new();
    let entries = fs::read_dir(directory).map_err(|e| vec![format!("catalogue directory: {e}")])?;
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                errors.insert(format!("catalogue directory entry: {e}"));
                continue;
            }
        };
        let path = entry.path();
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            errors.insert("catalogue directory: invalid UTF-8 filename".into());
            continue;
        };
        let metadata = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                errors.insert(format!("{name}: {e}"));
                continue;
            }
        };
        if linked(&metadata)
            || !metadata.is_file()
            || !name.ends_with(".toml")
            || name != name.to_ascii_lowercase()
        {
            errors.insert(format!("{name}: lowercase TOML regular file required; links/reparse points and subdirectories forbidden"));
            continue;
        }
        match fs::read(&path) {
            Ok(bytes) => documents.push((name.to_owned(), bytes)),
            Err(e) => {
                errors.insert(format!("{name}: {e}"));
            }
        }
    }
    match validate_documents(documents, contract) {
        Ok(catalogue) if errors.is_empty() => Ok(catalogue),
        Ok(_) => Err(errors.into_iter().collect()),
        Err(validation) => {
            errors.extend(validation);
            Err(errors.into_iter().collect())
        }
    }
}
