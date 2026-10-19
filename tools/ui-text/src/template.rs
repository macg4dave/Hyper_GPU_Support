//! Message template grammar: literal text with `{name}` placeholders and `{{`/`}}` escapes.
//! A template is parsed once at build time; inserted values are never parsed again.

#[derive(Clone, Debug, PartialEq, Eq)]
/// Decoded literal text or a named argument, in template order.
pub enum Segment {
    Literal(String),
    Argument(String),
}

/// Identifier grammar shared by key segments and argument names: lowercase ASCII words joined
/// by single underscores, starting with a letter.
pub fn identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    if !bytes.next().is_some_and(|b| b.is_ascii_lowercase()) {
        return false;
    }
    let mut underscore = false;
    for b in bytes {
        if b == b'_' {
            if underscore {
                return false;
            }
            underscore = true;
        } else if b.is_ascii_lowercase() || b.is_ascii_digit() {
            underscore = false;
        } else {
            return false;
        }
    }
    !underscore
}

/// Validate template grammar, preserving literal Unicode and whitespace exactly.
pub fn parse(text: &str) -> Result<Vec<Segment>, &'static str> {
    if text.trim().is_empty() {
        return Err("text must contain non-whitespace characters");
    }
    if text
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\t' | '\n'))
    {
        return Err("text contains forbidden control characters");
    }
    let mut chars = text.chars().peekable();
    let mut literal = String::new();
    let mut segments = Vec::new();
    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                literal.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                literal.push('}');
            }
            '{' => {
                if !literal.is_empty() {
                    segments.push(Segment::Literal(std::mem::take(&mut literal)));
                }
                let mut name = String::new();
                loop {
                    match chars.next() {
                        Some('}') => break,
                        Some(c) => name.push(c),
                        None => return Err("unmatched opening brace"),
                    }
                }
                if !identifier(&name) {
                    return Err("invalid argument name in placeholder");
                }
                segments.push(Segment::Argument(name));
            }
            '}' => return Err("unmatched closing brace"),
            _ => literal.push(c),
        }
    }
    if !literal.is_empty() {
        segments.push(Segment::Literal(literal));
    }
    Ok(segments)
}

/// Argument names used by parsed segments, in first-use order without duplicates.
pub fn arguments(segments: &[Segment]) -> Vec<&str> {
    let mut names = Vec::new();
    for segment in segments {
        if let Segment::Argument(name) = segment
            && !names.contains(&name.as_str())
        {
            names.push(name.as_str());
        }
    }
    names
}

/// Serialize literal text as a Slint string literal.
pub fn slint_literal(text: &str) -> String {
    let mut result = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\u{9}"),
            _ => result.push(c),
        }
    }
    result.push('"');
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_arguments_and_escapes() {
        assert_eq!(
            parse("Sign in to {vm_name} {{now}}").unwrap(),
            vec![
                Segment::Literal("Sign in to ".into()),
                Segment::Argument("vm_name".into()),
                Segment::Literal(" {now}".into()),
            ]
        );
        assert_eq!(
            arguments(&parse("{a} and {b} then {a}").unwrap()),
            ["a", "b"]
        );
    }

    #[test]
    fn rejects_malformed_templates() {
        for bad in [
            "",
            "   ",
            "open {name",
            "close }",
            "{Name}",
            "{a__b}",
            "{}",
            "bell\u{7}",
        ] {
            assert!(parse(bad).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn slint_literals_escape_quotes_and_newlines() {
        assert_eq!(slint_literal("a\"b\\c\nd\te"), "\"a\\\"b\\\\c\\nd\\u{9}e\"");
    }
}
