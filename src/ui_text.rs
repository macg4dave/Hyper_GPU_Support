//! User-visible GUI wording, compiled from `resources/i18n/<locale>/*.toml` for every language.
//!
//! The generated accessors (for example `dashboard::vm_list::count`) read the active language,
//! which starts as the reference language and is changed with [`set_current`]. Wording is never
//! read from disk at runtime, and inserted values are never parsed as templates.
use std::sync::atomic::{AtomicUsize, Ordering};

include!(concat!(env!("OUT_DIR"), "/ui_text.rs"));

/// A compiled language.
#[derive(Debug)]
pub struct Locale {
    /// BCP 47 tag, such as `en-GB`.
    pub tag: &'static str,
    /// Native language name for the language picker.
    pub name: &'static str,
}

/// Rendered text of a message with arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiMessage(String);

impl UiMessage {
    /// Borrow the rendered text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Convert to owned text for an existing presenter/model.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::ops::Deref for UiMessage {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UiMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<UiMessage> for String {
    fn from(message: UiMessage) -> Self {
        message.0
    }
}

impl From<UiMessage> for slint::SharedString {
    fn from(message: UiMessage) -> Self {
        message.0.into()
    }
}

/// A value prepared by code-owned formatting (for example a debug representation) for insertion
/// into a message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayValue(String);

impl DisplayValue {
    /// Borrow the prepared value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// CLDR plural category used by generated plural messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[doc(hidden)]
pub enum Category {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

/// A parsed template piece in generated message data.
#[derive(Debug)]
#[doc(hidden)]
pub enum Piece {
    Text(&'static str),
    /// Index into the message's arguments.
    Arg(usize),
}

static CURRENT: AtomicUsize = AtomicUsize::new(0);

/// Index of the active language in [`LOCALES`].
pub fn current_index() -> usize {
    CURRENT.load(Ordering::Relaxed).min(LOCALES.len() - 1)
}

/// The active language.
pub fn current() -> &'static Locale {
    &LOCALES[current_index()]
}

/// Make the language at `index` in [`LOCALES`] active. Returns false for an unknown index.
/// Callers re-apply wording to the UI afterwards; existing rendered text is not changed.
pub fn set_current(index: usize) -> bool {
    if index < LOCALES.len() {
        CURRENT.store(index, Ordering::Relaxed);
        true
    } else {
        false
    }
}

/// Best compiled language for preferred tags, in preference order: an exact tag match first,
/// then the same primary language, otherwise the reference language (index 0).
pub fn best_match<S: AsRef<str>>(preferred: &[S]) -> usize {
    let language = |tag: &str| {
        tag.split(['-', '_'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase()
    };
    for tag in preferred {
        let tag = tag.as_ref().replace('_', "-");
        if let Some(index) = LOCALES
            .iter()
            .position(|l| l.tag.eq_ignore_ascii_case(&tag))
        {
            return index;
        }
        if let Some(index) = LOCALES
            .iter()
            .position(|l| language(l.tag) == language(&tag))
        {
            return index;
        }
    }
    0
}

/// Windows display languages in the user's preference order, such as `["fr-FR", "en-US"]`.
/// Empty when unavailable; callers then use the reference language.
#[cfg(windows)]
#[allow(unsafe_code)]
pub fn system_languages() -> Vec<String> {
    use windows::Win32::Globalization::{GetUserPreferredUILanguages, MUI_LANGUAGE_NAME};
    let mut count = 0u32;
    let mut length = 0u32;
    // SAFETY: the first call passes no buffer and only writes the two counters we own.
    let sized =
        unsafe { GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut count, None, &mut length) };
    if sized.is_err() || length == 0 || length > 4096 {
        return Vec::new();
    }
    let mut buffer = vec![0u16; length as usize];
    // SAFETY: `buffer` holds exactly `length` UTF-16 units, as reported by the sizing call, and
    // outlives the call; the API writes a double-NUL-terminated list within that length.
    let filled = unsafe {
        GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut count,
            Some(windows::core::PWSTR(buffer.as_mut_ptr())),
            &mut length,
        )
    };
    if filled.is_err() {
        return Vec::new();
    }
    buffer
        .split(|&unit| unit == 0)
        .filter(|tag| !tag.is_empty())
        .map(String::from_utf16_lossy)
        .collect()
}

/// Windows display languages; always empty on other platforms.
#[cfg(not(windows))]
pub fn system_languages() -> Vec<String> {
    Vec::new()
}

/// Wording of a message without arguments in the active language.
fn pick(texts: &'static [&'static str]) -> &'static str {
    texts[current_index()]
}

/// Render a message with arguments in the active language.
fn render(forms: &'static [&'static [Piece]], args: &[&str]) -> UiMessage {
    UiMessage(join(forms[current_index()], args))
}

/// Render a plural message, selecting the form for `count` in the active language.
fn render_plural(
    count: u64,
    forms: &'static [&'static [(Category, &'static [Piece])]],
    args: &[&str],
) -> UiMessage {
    let locale = current_index();
    let category = plural_category(locale, count);
    let forms = forms[locale];
    let pieces = forms
        .iter()
        .find(|(c, _)| *c == category)
        .or_else(|| forms.iter().find(|(c, _)| *c == Category::Other))
        .map_or(&[][..], |(_, pieces)| *pieces);
    UiMessage(join(pieces, args))
}

fn join(pieces: &[Piece], args: &[&str]) -> String {
    let mut output = String::new();
    for piece in pieces {
        match piece {
            Piece::Text(text) => output.push_str(text),
            Piece::Arg(index) => output.push_str(args.get(*index).copied().unwrap_or_default()),
        }
    }
    output
}

/// Select display wording from typed reviewed power, independently of wording.
pub fn power(power: &crate::model::Power) -> &'static str {
    match power {
        crate::model::Power::Off => common::power::off(),
        crate::model::Power::Running => common::power::running(),
        crate::model::Power::Other(_) => common::state::unknown(),
    }
}

/// Preserve the existing enum debug representation for technical details only.
pub fn power_debug(power: &crate::model::Power) -> DisplayValue {
    DisplayValue(format!("{power:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_language_is_first_and_named() {
        assert_eq!(LOCALES[0].tag, "en-GB");
        assert!(!LOCALES[0].name.is_empty());
        assert_eq!(current().tag, "en-GB");
        assert!(!set_current(LOCALES.len()));
    }

    #[test]
    fn preferred_languages_match_exact_then_language_then_reference() {
        assert_eq!(best_match(&["en-GB"]), 0);
        assert_eq!(best_match(&["en-US"]), 0);
        assert_eq!(best_match(&["en_gb"]), 0);
        assert_eq!(best_match::<&str>(&[]), 0);
        assert_eq!(best_match(&["xx-YY"]), 0);
    }

    #[test]
    fn messages_render_arguments_and_plural_forms() {
        assert_eq!(dashboard::vm_list::count(1).as_str(), "1 VM shown");
        assert_eq!(dashboard::vm_list::count(3).as_str(), "3 VMs shown");
        assert_eq!(
            join(
                &[Piece::Text("a "), Piece::Arg(0), Piece::Text("!")],
                &["{b}"]
            ),
            "a {b}!"
        );
    }
}
