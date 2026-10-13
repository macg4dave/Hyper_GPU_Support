//! Build-time locale loading shared by the product and contributor validation.
use crate::{Catalogue, Errors, contract::Message, linked, validate_locale_directory};
use std::{collections::BTreeMap, fs, path::Path};

/// The required default and fallback catalogue; independent of the host locale.
pub const DEFAULT_LOCALE: &str = "en-GB";

/// A selected complete catalogue and the sorted locales validated with it.
pub struct LoadedCatalogue {
    /// Resolved locale, which may differ from an unavailable requested locale.
    pub locale: String,
    /// Only complete, valid catalogues are available.
    pub available: Vec<String>,
    /// Selected catalogue used by both Rust and Slint generation.
    pub catalogue: Catalogue,
}

// Restrict names to language tags, never paths. This accepts language/script/region
// subtags without pretending to implement BCP 47 canonicalisation or negotiation.
fn locale_name(name: &str) -> bool {
    let mut parts = name.split('-');
    parts.next().is_some_and(|language| {
        (2..=8).contains(&language.len()) && language.bytes().all(|c| c.is_ascii_alphabetic())
    }) && parts.all(|part| {
        (1..=8).contains(&part.len()) && part.bytes().all(|c| c.is_ascii_alphanumeric())
    })
}

/// Discover and validate every locale before selecting one. Missing/unavailable
/// requests fall back to en-GB; invalid installed catalogues are build errors.
/// No partial catalogue or malformed translation is silently accepted.
pub fn load(
    root: &Path,
    requested: Option<&str>,
    contract: &[Message],
) -> Result<LoadedCatalogue, Errors> {
    let metadata = fs::symlink_metadata(root).map_err(|e| vec![format!("locale root: {e}")])?;
    if linked(&metadata) || !metadata.is_dir() {
        return Err(vec![
            "locale root: regular directory required; links/reparse points forbidden".into(),
        ]);
    }
    let mut catalogues = BTreeMap::new();
    let mut errors = Vec::new();
    for entry in fs::read_dir(root).map_err(|e| vec![format!("locale root: {e}")])? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                errors.push(format!("locale root entry: {error}"));
                continue;
            }
        };
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            errors.push("locale root: invalid UTF-8 locale name".into());
            continue;
        };
        let metadata = match fs::symlink_metadata(entry.path()) {
            Ok(metadata) => metadata,
            Err(error) => {
                errors.push(format!("{name}: {error}"));
                continue;
            }
        };
        // Keep contributor documentation beside, rather than inside, catalogues.
        if name == "README.md" && metadata.is_file() && !linked(&metadata) {
            continue;
        }
        if !locale_name(name) || linked(&metadata) || !metadata.is_dir() {
            errors.push(format!(
                "{name}: locale directory required; links/reparse points forbidden"
            ));
            continue;
        }
        match validate_locale_directory(&entry.path(), contract, name) {
            Ok(catalogue) => {
                catalogues.insert(name.to_owned(), catalogue);
            }
            Err(validation) => errors.extend(validation.into_iter().map(|e| format!("{name}/{e}"))),
        }
    }
    if !catalogues.contains_key(DEFAULT_LOCALE) {
        errors.push(format!(
            "{DEFAULT_LOCALE}: valid default catalogue required"
        ));
    }
    if !errors.is_empty() {
        errors.sort();
        return Err(errors);
    }
    let available = catalogues.keys().cloned().collect();
    let locale = requested
        .filter(|name| catalogues.contains_key(*name))
        .unwrap_or(DEFAULT_LOCALE)
        .to_owned();
    let catalogue = catalogues
        .remove(&locale)
        .expect("validated selected locale");
    Ok(LoadedCatalogue {
        locale,
        available,
        catalogue,
    })
}

impl LoadedCatalogue {
    /// Emit one resolved locale for both consumers, with no runtime file access.
    pub fn generate(&self) -> crate::Sources {
        let mut sources = crate::generate::sources(&self.catalogue, Some(&self.locale));
        sources.rust.push_str(&format!(
            "/// Default and fallback locale.\npub const DEFAULT_LOCALE: &str = {DEFAULT_LOCALE:?};\n\
             /// Locale compiled into both Rust and Slint.\npub const LOCALE: &str = {:?};\n\
             /// Complete catalogues validated at build time.\npub const AVAILABLE_LOCALES: &[&str] = &{:?};\n",
            self.locale, self.available
        ));
        sources
    }
}
