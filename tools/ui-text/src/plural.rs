//! CLDR integer plural rules for the languages the catalogue can accept. One definition drives
//! build-time validation, the generated Rust selector and the generated Slint selector.

/// CLDR plural category names usable as message forms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl Category {
    /// Every category, in CLDR order.
    pub const ALL: [Category; 6] = [
        Category::Zero,
        Category::One,
        Category::Two,
        Category::Few,
        Category::Many,
        Category::Other,
    ];

    /// The TOML form name, such as `one` or `other`.
    pub fn name(self) -> &'static str {
        match self {
            Category::Zero => "zero",
            Category::One => "one",
            Category::Two => "two",
            Category::Few => "few",
            Category::Many => "many",
            Category::Other => "other",
        }
    }

    /// The matching variant of the application's runtime `Category` enum.
    pub fn variant(self) -> &'static str {
        match self {
            Category::Zero => "Zero",
            Category::One => "One",
            Category::Two => "Two",
            Category::Few => "Few",
            Category::Many => "Many",
            Category::Other => "Other",
        }
    }

    /// Parse a TOML form name.
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }
}

/// A condition on a non-negative integer count `n`.
#[derive(Clone, Copy, Debug)]
enum Condition {
    /// `lo <= n <= hi`
    Range(u64, u64),
    /// `lo <= n % m <= hi`
    Mod(u64, u64, u64),
    /// not `lo <= n % m <= hi`
    NotMod(u64, u64, u64),
}

impl Condition {
    fn holds(self, n: u64) -> bool {
        match self {
            Condition::Range(lo, hi) => (lo..=hi).contains(&n),
            Condition::Mod(m, lo, hi) => (lo..=hi).contains(&(n % m)),
            Condition::NotMod(m, lo, hi) => !(lo..=hi).contains(&(n % m)),
        }
    }

    fn rust(self, n: &str) -> String {
        match self {
            Condition::Range(lo, hi) if lo == hi => format!("{n} == {lo}"),
            Condition::Range(lo, hi) => format!("({lo}..={hi}).contains(&{n})"),
            Condition::Mod(m, lo, hi) => format!("({lo}..={hi}).contains(&({n} % {m}))"),
            Condition::NotMod(m, lo, hi) => format!("!({lo}..={hi}).contains(&({n} % {m}))"),
        }
    }

    fn slint(self, n: &str) -> String {
        match self {
            Condition::Range(lo, hi) if lo == hi => format!("{n} == {lo}"),
            Condition::Range(lo, hi) => format!("({n} >= {lo} && {n} <= {hi})"),
            Condition::Mod(m, lo, hi) => {
                format!("(Math.mod({n}, {m}) >= {lo} && Math.mod({n}, {m}) <= {hi})")
            }
            Condition::NotMod(m, lo, hi) => {
                format!("(Math.mod({n}, {m}) < {lo} || Math.mod({n}, {m}) > {hi})")
            }
        }
    }
}

/// Integer plural rule families. Add a family (and its languages in [`PluralRule::for_language`])
/// before adding a catalogue for a language that is not yet listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluralRule {
    /// No plural distinction (for example Japanese, Chinese, Korean).
    None,
    /// `one` for 1 (for example English, German, Spanish).
    OneIsOne,
    /// `one` for 0 and 1 (for example French, Brazilian Portuguese).
    OneIsZeroOrOne,
    /// `one`/`few`/`many` by final digits (Russian, Ukrainian, Belarusian).
    EastSlavic,
    /// `one`/`few`/`other` by final digits (Croatian, Serbian, Bosnian).
    SouthSlavic,
    /// `one` for 1, `few`/`many` by final digits (Polish).
    Polish,
    /// `one` for 1, `few` for 2–4 (Czech, Slovak).
    CzechSlovak,
}

const ONE: Condition = Condition::Range(1, 1);
const SLAVIC_ONE: &[Condition] = &[Condition::Mod(10, 1, 1), Condition::NotMod(100, 11, 11)];
const SLAVIC_FEW: &[Condition] = &[Condition::Mod(10, 2, 4), Condition::NotMod(100, 12, 14)];

impl PluralRule {
    /// Rule for a BCP 47 tag, chosen by its primary language subtag (and region for Portuguese).
    pub fn for_language(tag: &str) -> Option<Self> {
        let lower = tag.to_ascii_lowercase();
        let language = lower.split('-').next().unwrap_or_default();
        Some(match language {
            "ja" | "zh" | "ko" | "th" | "vi" | "id" | "ms" | "lo" | "my" | "km" => Self::None,
            "pt" if lower != "pt-pt" => Self::OneIsZeroOrOne,
            "fr" | "hi" | "bn" | "fa" => Self::OneIsZeroOrOne,
            "ru" | "uk" | "be" => Self::EastSlavic,
            "hr" | "sr" | "bs" => Self::SouthSlavic,
            "pl" => Self::Polish,
            "cs" | "sk" => Self::CzechSlovak,
            "en" | "de" | "nl" | "sv" | "da" | "nb" | "nn" | "no" | "fi" | "et" | "it" | "es"
            | "ca" | "gl" | "eu" | "el" | "hu" | "bg" | "tr" | "pt" | "af" | "sq" => Self::OneIsOne,
            _ => return None,
        })
    }

    /// Ordered `(category, all-of conditions)` pairs; the first match wins, otherwise `other`.
    fn table(self) -> &'static [(Category, &'static [Condition])] {
        match self {
            Self::None => &[],
            Self::OneIsOne => &[(Category::One, &[ONE])],
            Self::OneIsZeroOrOne => &[(Category::One, &[Condition::Range(0, 1)])],
            Self::EastSlavic => &[
                (Category::One, SLAVIC_ONE),
                (Category::Few, SLAVIC_FEW),
                (Category::Many, &[]),
            ],
            Self::SouthSlavic => &[(Category::One, SLAVIC_ONE), (Category::Few, SLAVIC_FEW)],
            Self::Polish => &[
                (Category::One, &[ONE]),
                (Category::Few, SLAVIC_FEW),
                (Category::Many, &[]),
            ],
            Self::CzechSlovak => &[
                (Category::One, &[ONE]),
                (Category::Few, &[Condition::Range(2, 4)]),
            ],
        }
    }

    /// Categories a catalogue for this rule may define; `other` is always included.
    pub fn categories(self) -> Vec<Category> {
        let mut categories: Vec<_> = self.table().iter().map(|(c, _)| *c).collect();
        categories.push(Category::Other);
        categories
    }

    /// Select the category for a count (used by tests and to document the generated code).
    pub fn select(self, n: u64) -> Category {
        self.table()
            .iter()
            .find(|(_, conditions)| conditions.iter().all(|c| c.holds(n)))
            .map_or(Category::Other, |(c, _)| *c)
    }

    /// A Rust expression of type `Category` selecting the category for `n: u64`.
    pub fn rust_selector(self, n: &str) -> String {
        let mut expression = String::new();
        for (category, conditions) in self.table() {
            if conditions.is_empty() {
                // An unconditional entry is terminal; `other` is then unreachable for integers.
                expression.push_str(&format!("{{ Category::{} }}", category.variant()));
                return expression;
            }
            let test = conditions
                .iter()
                .map(|c| c.rust(n))
                .collect::<Vec<_>>()
                .join(" && ");
            expression.push_str(&format!(
                "if {test} {{ Category::{} }} else ",
                category.variant()
            ));
        }
        expression.push_str("{ Category::Other }");
        expression
    }

    /// Slint conditions for each non-`other` category of an `int` count, in match order.
    pub fn slint_conditions(self, n: &str) -> Vec<(Category, String)> {
        self.table()
            .iter()
            .map(|(category, conditions)| {
                let test = if conditions.is_empty() {
                    "true".to_owned()
                } else {
                    conditions
                        .iter()
                        .map(|c| c.slint(n))
                        .collect::<Vec<_>>()
                        .join(" && ")
                };
                (*category, test)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{Category::*, PluralRule};

    #[test]
    fn languages_map_to_cldr_rules() {
        assert_eq!(
            PluralRule::for_language("en-GB"),
            Some(PluralRule::OneIsOne)
        );
        assert_eq!(
            PluralRule::for_language("fr-FR"),
            Some(PluralRule::OneIsZeroOrOne)
        );
        assert_eq!(
            PluralRule::for_language("pt-BR"),
            Some(PluralRule::OneIsZeroOrOne)
        );
        assert_eq!(
            PluralRule::for_language("pt-PT"),
            Some(PluralRule::OneIsOne)
        );
        assert_eq!(PluralRule::for_language("ja-JP"), Some(PluralRule::None));
        assert_eq!(PluralRule::for_language("xx"), None);
    }

    #[test]
    fn selects_cldr_integer_categories() {
        let en = PluralRule::OneIsOne;
        assert_eq!([0, 1, 2].map(|n| en.select(n)), [Other, One, Other]);
        let fr = PluralRule::OneIsZeroOrOne;
        assert_eq!([0, 1, 2].map(|n| fr.select(n)), [One, One, Other]);
        let ru = PluralRule::EastSlavic;
        assert_eq!(
            [1, 2, 5, 11, 12, 21, 22, 25, 111].map(|n| ru.select(n)),
            [One, Few, Many, Many, Many, One, Few, Many, Many]
        );
        let pl = PluralRule::Polish;
        assert_eq!(
            [1, 2, 5, 21, 22].map(|n| pl.select(n)),
            [One, Few, Many, Many, Few]
        );
        let cs = PluralRule::CzechSlovak;
        assert_eq!([1, 3, 5].map(|n| cs.select(n)), [One, Few, Other]);
        assert_eq!(PluralRule::None.select(1), Other);
    }

    #[test]
    fn selectors_cover_every_rule_category() {
        let rule = PluralRule::EastSlavic;
        assert_eq!(rule.categories(), [One, Few, Many, Other]);
        assert!(rule.rust_selector("n").ends_with("{ Category::Many }"));
        assert!(
            PluralRule::OneIsOne
                .rust_selector("n")
                .ends_with("{ Category::Other }")
        );
        assert_eq!(rule.slint_conditions("count").len(), 3);
    }
}
