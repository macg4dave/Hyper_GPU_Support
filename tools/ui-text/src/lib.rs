//! Build-time multilingual UI catalogue.
//!
//! [`load`] reads `resources/i18n/<locale>/<feature>.toml`, validates every language against the
//! British English reference catalogue, and returns a [`Catalogue`] whose
//! [`generate`](Catalogue::generate) emits the application's Rust accessors and Slint `UiText`
//! global. All languages are compiled in; the application selects one at runtime.
mod generate;
mod load;
mod plural;
mod template;

pub use generate::Sources;
pub use load::{Loaded, load};
pub use plural::{Category, PluralRule};
pub use template::Segment;

/// The reference language: it defines every key and argument and is the fallback for
/// untranslated messages.
pub const REFERENCE_LOCALE: &str = "en-GB";

/// Key of the native language name shown in the language picker. Every catalogue must define it.
pub const LANGUAGE_NAME_KEY: &str = "common.language.name";

/// Rust type of a message argument, declared once in the reference catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgKind {
    /// Plain data such as a VM name (`&str`).
    Text,
    /// Text that is itself a catalogue message (`&str`).
    Message,
    /// A code-formatted value (`&DisplayValue`).
    Formatted,
    /// `usize` count, formatted with `to_string`.
    Usize,
    /// `u64` value, formatted with `to_string`.
    U64,
    /// `i32` value, formatted with `to_string`.
    I32,
}

impl ArgKind {
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "text" => Self::Text,
            "message" => Self::Message,
            "formatted" => Self::Formatted,
            "usize" => Self::Usize,
            "u64" => Self::U64,
            "i32" => Self::I32,
            _ => return None,
        })
    }

    fn numeric(self) -> bool {
        matches!(self, Self::Usize | Self::U64 | Self::I32)
    }
}

/// A named message argument.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arg {
    pub name: String,
    pub kind: ArgKind,
}

/// One language's wording for a message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Forms {
    /// A single template.
    Text(Vec<Segment>),
    /// Plural forms keyed by category; always contains [`Category::Other`].
    Plural(Vec<(Category, Vec<Segment>)>),
}

/// A validated message with wording for every compiled language (fallbacks resolved).
#[derive(Clone, Debug)]
pub struct Message {
    /// Dotted key such as `dashboard.vm_list.count`; the first segment is the feature file.
    pub key: String,
    /// Arguments in the reference declaration order, which is the generated parameter order.
    pub args: Vec<Arg>,
    /// Index into `args` of the count that selects the plural form, if any.
    pub plural: Option<usize>,
    /// Wording per language, in [`Catalogue::locales`] order.
    pub forms: Vec<Forms>,
}

impl Message {
    /// Slint property/function name: `.` and `_` become `-`.
    pub fn slint_name(&self) -> String {
        self.key.replace(['.', '_'], "-")
    }
}

/// A compiled language.
#[derive(Clone, Debug)]
pub struct Locale {
    /// BCP 47 tag, which is also the catalogue directory name.
    pub tag: String,
    /// Native language name from [`LANGUAGE_NAME_KEY`].
    pub name: String,
    pub plural: PluralRule,
}

/// Every compiled language and message. The reference language is always first.
#[derive(Clone, Debug)]
pub struct Catalogue {
    pub locales: Vec<Locale>,
    pub messages: Vec<Message>,
}
