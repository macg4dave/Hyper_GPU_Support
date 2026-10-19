//! Locale discovery and validation. Only fully validated catalogues are returned; any error
//! fails the build, while untranslated messages fall back to the reference with a warning.
use crate::{
    Arg, ArgKind, Catalogue, Category, Forms, LANGUAGE_NAME_KEY, Locale, Message, PluralRule,
    REFERENCE_LOCALE,
    template::{self, Segment, identifier},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

/// A validated catalogue plus build integration details.
#[derive(Debug)]
pub struct Loaded {
    pub catalogue: Catalogue,
    /// Non-fatal findings, such as untranslated messages using the reference wording.
    pub warnings: Vec<String>,
    /// Every directory and file read, for build-script change tracking.
    pub inputs: Vec<PathBuf>,
}

/// Fields of a message table. Key segments may not use these names.
const MESSAGE_FIELDS: [&str; 3] = ["text", "args", "plural"];

const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "gen", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
    "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true", "try", "type",
    "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield",
];

/// Raw wording before template parsing.
enum Raw {
    Text(String),
    Plural(BTreeMap<Category, String>),
}

/// A message as written in one locale's files.
struct Entry {
    file: PathBuf,
    raw: Raw,
    /// Reference only: declared arguments and plural selector.
    args: Vec<Arg>,
    plural: Option<String>,
}

/// Load and validate every locale under `root`.
pub fn load(root: &Path) -> Result<Loaded, Vec<String>> {
    let mut errors = Vec::new();
    let mut inputs = vec![root.to_path_buf()];
    let tags = discover(root, &mut errors);
    if !tags.iter().any(|t| t == REFERENCE_LOCALE) {
        errors.push(format!(
            "{}: reference locale {REFERENCE_LOCALE} is required",
            root.display()
        ));
        return Err(errors);
    }
    let reference = read_locale(root, REFERENCE_LOCALE, true, &mut inputs, &mut errors);
    let mut messages = reference_messages(&reference, &mut errors);
    let features: BTreeSet<_> = messages.keys().map(|k| feature(k).to_owned()).collect();
    if !messages.contains_key(LANGUAGE_NAME_KEY) {
        errors.push(format!(
            "{REFERENCE_LOCALE}: {LANGUAGE_NAME_KEY} is required"
        ));
    }

    let mut locales = Vec::new();
    let mut warnings = Vec::new();
    let ordered = std::iter::once(REFERENCE_LOCALE.to_owned())
        .chain(tags.into_iter().filter(|t| t != REFERENCE_LOCALE));
    for (index, tag) in ordered.enumerate() {
        let Some(plural) = PluralRule::for_language(&tag) else {
            errors.push(format!(
                "{tag}: no plural rule for this language; add it in tools/ui-text/src/plural.rs"
            ));
            continue;
        };
        let entries = if index == 0 {
            None
        } else {
            Some(read_locale(root, &tag, false, &mut inputs, &mut errors))
        };
        if let Some(entries) = &entries {
            for (key, entry) in entries {
                if !features.contains(feature(key)) {
                    errors.push(format!("{}: unknown catalogue file", entry.file.display()));
                } else if !messages.contains_key(key) {
                    errors.push(format!("{}: unknown key {key}", entry.file.display()));
                }
            }
        }
        let mut missing = 0;
        let mut name = None;
        for (key, message) in messages.iter_mut() {
            let forms = match entries.as_ref().map(|e| e.get(key)) {
                // The reference wording was validated by `reference_messages`.
                None => message.forms[0].clone(),
                Some(Some(entry)) => match translated(entry, message, plural) {
                    Ok(forms) => forms,
                    Err(error) => {
                        errors.push(format!("{}: {key}: {error}", entry.file.display()));
                        continue;
                    }
                },
                Some(None) => {
                    if key == LANGUAGE_NAME_KEY {
                        errors.push(format!("{tag}: {LANGUAGE_NAME_KEY} is required"));
                    }
                    missing += 1;
                    fallback(&message.forms[0], plural)
                }
            };
            if key == LANGUAGE_NAME_KEY
                && let Forms::Text(segments) = &forms
            {
                name = Some(literal(segments));
            }
            if index > 0 {
                message.forms.push(forms);
            } else if plural_categories_invalid(&forms, plural) {
                errors.push(format!(
                    "{tag}: {key}: plural forms do not match the language rule"
                ));
            }
        }
        if missing > 0 {
            warnings.push(format!(
                "{tag}: {missing} untranslated message(s) use {REFERENCE_LOCALE} wording"
            ));
        }
        locales.push(Locale {
            tag,
            name: name.unwrap_or_default(),
            plural,
        });
    }

    let mut slint_names = BTreeMap::new();
    for message in messages.values() {
        if let Some(other) = slint_names.insert(message.slint_name(), message.key.clone()) {
            errors.push(format!(
                "{} and {other} map to the same Slint name",
                message.key
            ));
        }
    }
    if errors.is_empty() {
        inputs.sort();
        inputs.dedup();
        Ok(Loaded {
            catalogue: Catalogue {
                locales,
                messages: messages.into_values().collect(),
            },
            warnings,
            inputs,
        })
    } else {
        Err(errors)
    }
}

fn feature(key: &str) -> &str {
    key.split('.').next().unwrap_or_default()
}

/// BCP 47-style tag: a 2–3 letter lowercase language, then 2–8 character alphanumeric subtags.
fn valid_tag(tag: &str) -> bool {
    let mut parts = tag.split('-');
    let language = parts.next().unwrap_or_default();
    (2..=3).contains(&language.len())
        && language.bytes().all(|b| b.is_ascii_lowercase())
        && parts.all(|p| (2..=8).contains(&p.len()) && p.bytes().all(|b| b.is_ascii_alphanumeric()))
}

/// Locale directory names. Only directories and a `README.md` may appear at the root.
fn discover(root: &Path, errors: &mut Vec<String>) -> Vec<String> {
    let mut tags = Vec::new();
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) => {
            errors.push(format!("{}: {error}", root.display()));
            return tags;
        }
    };
    for entry in entries {
        let Ok(entry) = entry else {
            errors.push(format!("{}: unreadable entry", root.display()));
            continue;
        };
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_dir() && valid_tag(&name) => tags.push(name),
            Ok(metadata) if metadata.is_file() && name == "README.md" => {}
            Ok(_) => errors.push(format!(
                "{}: expected a locale directory named like en-GB, or README.md",
                path.display()
            )),
            Err(error) => errors.push(format!("{}: {error}", path.display())),
        }
    }
    tags.sort();
    tags
}

/// Read every feature file of one locale into flat dotted keys.
fn read_locale(
    root: &Path,
    tag: &str,
    reference: bool,
    inputs: &mut Vec<PathBuf>,
    errors: &mut Vec<String>,
) -> BTreeMap<String, Entry> {
    let directory = root.join(tag);
    inputs.push(directory.clone());
    let mut entries = BTreeMap::new();
    let mut files: Vec<_> = match fs::read_dir(&directory) {
        Ok(files) => files.filter_map(Result::ok).map(|f| f.path()).collect(),
        Err(error) => {
            errors.push(format!("{}: {error}", directory.display()));
            return entries;
        }
    };
    files.sort();
    for file in files {
        let feature = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_owned();
        let regular = fs::symlink_metadata(&file).is_ok_and(|m| m.is_file());
        if !regular || file.extension().is_none_or(|e| e != "toml") || !identifier(&feature) {
            errors.push(format!(
                "{}: expected a feature file such as common.toml",
                file.display()
            ));
            continue;
        }
        inputs.push(file.clone());
        let table = match fs::read_to_string(&file)
            .map_err(|e| e.to_string())
            .and_then(|text| text.parse::<toml::Table>().map_err(|e| e.to_string()))
        {
            Ok(table) => table,
            Err(error) => {
                errors.push(format!("{}: {error}", file.display()));
                continue;
            }
        };
        walk(&file, &feature, &table, reference, &mut entries, errors);
    }
    entries
}

fn walk(
    file: &Path,
    prefix: &str,
    table: &toml::Table,
    reference: bool,
    entries: &mut BTreeMap<String, Entry>,
    errors: &mut Vec<String>,
) {
    if table.is_empty() {
        errors.push(format!("{}: {prefix}: empty section", file.display()));
    }
    for (name, value) in table {
        let key = format!("{prefix}.{name}");
        if !identifier(name)
            || RUST_KEYWORDS.contains(&name.as_str())
            || MESSAGE_FIELDS.contains(&name.as_str())
            || Category::parse(name).is_some()
        {
            errors.push(format!(
                "{}: {key}: invalid or reserved key name",
                file.display()
            ));
            continue;
        }
        let raw = match value {
            toml::Value::String(text) => Some(Entry {
                file: file.into(),
                raw: Raw::Text(text.clone()),
                args: vec![],
                plural: None,
            }),
            toml::Value::Table(table) if is_message(table) => {
                match message_entry(table, reference) {
                    Ok((raw, args, plural)) => Some(Entry {
                        file: file.into(),
                        raw,
                        args,
                        plural,
                    }),
                    Err(error) => {
                        errors.push(format!("{}: {key}: {error}", file.display()));
                        None
                    }
                }
            }
            toml::Value::Table(table) => {
                walk(file, &key, table, reference, entries, errors);
                None
            }
            _ => {
                errors.push(format!(
                    "{}: {key}: expected text or a section",
                    file.display()
                ));
                None
            }
        };
        if let Some(entry) = raw {
            entries.insert(key, entry);
        }
    }
}

fn is_message(table: &toml::Table) -> bool {
    table
        .keys()
        .any(|k| MESSAGE_FIELDS.contains(&k.as_str()) || Category::parse(k).is_some())
}

/// Parse a message table: `text = "..."` or plural forms, plus (reference only) `args`/`plural`.
fn message_entry(
    table: &toml::Table,
    reference: bool,
) -> Result<(Raw, Vec<Arg>, Option<String>), String> {
    let mut text = None;
    let mut forms = BTreeMap::new();
    let mut args = Vec::new();
    let mut plural = None;
    for (field, value) in table {
        match (field.as_str(), value) {
            ("text", toml::Value::String(s)) => text = Some(s.clone()),
            ("args" | "plural", _) if !reference => {
                return Err(format!("{field} is declared only in {REFERENCE_LOCALE}"));
            }
            ("args", toml::Value::Array(items)) => {
                for item in items {
                    args.push(argument(item)?);
                }
            }
            ("plural", toml::Value::String(s)) => plural = Some(s.clone()),
            (name, toml::Value::String(s)) if Category::parse(name).is_some() => {
                forms.insert(Category::parse(name).unwrap_or(Category::Other), s.clone());
            }
            _ => return Err(format!("unexpected field {field}")),
        }
    }
    let raw = match (text, forms.is_empty()) {
        (Some(text), true) => Raw::Text(text),
        (None, false) => Raw::Plural(forms),
        _ => return Err("use either text or plural forms".into()),
    };
    Ok((raw, args, plural))
}

/// `"name: kind"`, for example `"vm_name: text"`.
fn argument(item: &toml::Value) -> Result<Arg, String> {
    let declaration = item
        .as_str()
        .ok_or("args entries must be strings like \"name: text\"")?;
    let (name, kind) = declaration
        .split_once(':')
        .ok_or_else(|| format!("argument {declaration:?} must be \"name: kind\""))?;
    let (name, kind) = (name.trim(), kind.trim());
    if !identifier(name) {
        return Err(format!("invalid argument name {name:?}"));
    }
    let kind = ArgKind::parse(kind).ok_or_else(|| {
        format!(
            "argument {name}: unknown type {kind:?} (text, message, formatted, usize, u64, i32)"
        )
    })?;
    Ok(Arg {
        name: name.to_owned(),
        kind,
    })
}

/// Validate reference entries and turn them into messages with one (reference) form each.
fn reference_messages(
    entries: &BTreeMap<String, Entry>,
    errors: &mut Vec<String>,
) -> BTreeMap<String, Message> {
    let mut messages = BTreeMap::new();
    for (key, entry) in entries {
        let names: Vec<_> = entry.args.iter().map(|a| a.name.as_str()).collect();
        if names
            .iter()
            .enumerate()
            .any(|(i, n)| names[..i].contains(n))
        {
            errors.push(format!(
                "{}: {key}: duplicate argument",
                entry.file.display()
            ));
            continue;
        }
        let plural = match (&entry.raw, &entry.plural) {
            (Raw::Plural(_), Some(selector)) => {
                match entry
                    .args
                    .iter()
                    .position(|a| &a.name == selector && a.kind.numeric())
                {
                    Some(index) => Some(index),
                    None => {
                        errors.push(format!(
                            "{}: {key}: plural must name a numeric argument",
                            entry.file.display()
                        ));
                        continue;
                    }
                }
            }
            (Raw::Text(_), None) => None,
            _ => {
                errors.push(format!(
                    "{}: {key}: plural forms need `plural = \"<count argument>\"`",
                    entry.file.display()
                ));
                continue;
            }
        };
        match forms(&entry.raw, &names) {
            Ok(forms) => {
                messages.insert(
                    key.clone(),
                    Message {
                        key: key.clone(),
                        args: entry.args.clone(),
                        plural,
                        forms: vec![forms],
                    },
                );
            }
            Err(error) => errors.push(format!("{}: {key}: {error}", entry.file.display())),
        }
    }
    messages
}

/// Parse wording and check it uses exactly the declared arguments.
fn forms(raw: &Raw, names: &[&str]) -> Result<Forms, String> {
    let mut used = BTreeSet::new();
    let mut check = |text: &str| -> Result<Vec<Segment>, String> {
        let segments = template::parse(text).map_err(str::to_owned)?;
        for name in template::arguments(&segments) {
            if !names.contains(&name) {
                return Err(format!("unknown placeholder {{{name}}}"));
            }
            used.insert(name.to_owned());
        }
        Ok(segments)
    };
    let forms = match raw {
        Raw::Text(text) => Forms::Text(check(text)?),
        Raw::Plural(map) => {
            if !map.contains_key(&Category::Other) {
                return Err("plural forms must include `other`".into());
            }
            let mut forms = Vec::new();
            for (category, text) in map {
                forms.push((*category, check(text)?));
            }
            Forms::Plural(forms)
        }
    };
    if let Some(name) = names.iter().find(|n| !used.contains(**n)) {
        return Err(format!("argument {name} is never used"));
    }
    Ok(forms)
}

/// Validate one translated message against its reference message and the locale's plural rule.
fn translated(entry: &Entry, message: &Message, rule: PluralRule) -> Result<Forms, String> {
    let names: Vec<_> = message.args.iter().map(|a| a.name.as_str()).collect();
    let raw = match (&entry.raw, message.plural.is_some()) {
        (Raw::Text(text), true) => &Raw::Plural(BTreeMap::from([(Category::Other, text.clone())])),
        (Raw::Plural(_), false) => return Err("this message has no plural forms".into()),
        (raw, _) => raw,
    };
    let forms = forms(raw, &names)?;
    if plural_categories_invalid(&forms, rule) {
        return Err(format!(
            "plural forms must be among: {}",
            rule.categories()
                .iter()
                .map(|c| c.name())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    Ok(forms)
}

fn plural_categories_invalid(forms: &Forms, rule: PluralRule) -> bool {
    let allowed = rule.categories();
    matches!(forms, Forms::Plural(f) if f.iter().any(|(c, _)| !allowed.contains(c)))
}

/// Reference wording used for an untranslated message, restricted to the locale's categories.
fn fallback(reference: &Forms, rule: PluralRule) -> Forms {
    match reference {
        Forms::Text(_) => reference.clone(),
        Forms::Plural(forms) => {
            let allowed = rule.categories();
            Forms::Plural(
                forms
                    .iter()
                    .filter(|(c, _)| allowed.contains(c))
                    .cloned()
                    .collect(),
            )
        }
    }
}

fn literal(segments: &[Segment]) -> String {
    segments
        .iter()
        .map(|s| match s {
            Segment::Literal(text) => text.as_str(),
            Segment::Argument(_) => "",
        })
        .collect()
}
