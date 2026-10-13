//! A single parse; inserted parameter values are never parsed recursively.
#[derive(Clone, Debug, PartialEq, Eq)]
/// Decoded literal text or a validated named argument, in template order.
pub enum Segment {
    Literal(String),
    Parameter(String),
}

/// Catalogue identifier grammar, shared by keys and placeholder names.
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

/// Validate template grammar and preserve literal Unicode/whitespace exactly.
pub fn parse(text: &str) -> Result<Vec<Segment>, &'static str> {
    if text.trim().is_empty() {
        return Err("text must contain non-whitespace characters");
    }
    if text
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\t' | '\r' | '\n'))
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
                    return Err("invalid named placeholder");
                }
                segments.push(Segment::Parameter(name));
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

/// Serialize already validated literal text using Slint-specific escapes.
pub fn slint_literal(text: &str) -> String {
    let mut result = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\t' | '\r' => result.push_str(&format!("\\u{{{:x}}}", c as u32)),
            _ => result.push(c),
        }
    }
    result.push('"');
    result
}
