//! Typed, read-only inventory reporting independent of the Windows adapter.

use std::fmt;

/// Availability of one discovered fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactStatus {
    /// A value was returned.
    Known,
    /// The queried facility or target was absent.
    Missing,
    /// The caller lacked permission to query it.
    Denied,
    /// The facility existed but the query could not produce a value.
    Unavailable,
}

impl FactStatus {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "known" => Some(Self::Known),
            "missing" => Some(Self::Missing),
            "denied" => Some(Self::Denied),
            "unavailable" => Some(Self::Unavailable),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Known => "known",
            Self::Missing => "missing",
            Self::Denied => "denied",
            Self::Unavailable => "unavailable",
        }
    }
}

/// One stable inventory key and its observed state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    key: String,
    status: FactStatus,
    value: String,
}

impl Fact {
    /// Construct a fact after validating its machine-readable key.
    ///
    /// # Errors
    /// Returns [`InventoryError::InvalidKey`] for an empty or unsafe key.
    pub fn new(
        key: impl Into<String>,
        status: FactStatus,
        value: impl Into<String>,
    ) -> Result<Self, InventoryError> {
        let key = key.into();
        if key.is_empty()
            || !key
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'.')
        {
            return Err(InventoryError::InvalidKey);
        }
        Ok(Self {
            key,
            status,
            value: value.into(),
        })
    }

    /// Stable fact key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Availability classification.
    pub const fn status(&self) -> FactStatus {
        self.status
    }

    /// Observed value or a bounded explanation.
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// A complete inventory observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryReport {
    facts: Vec<Fact>,
}

impl InventoryReport {
    /// Build a report, rejecting duplicate keys.
    ///
    /// # Errors
    /// Returns [`InventoryError::DuplicateKey`] if a source repeats a key.
    pub fn new(mut facts: Vec<Fact>) -> Result<Self, InventoryError> {
        facts.sort_by(|left, right| left.key.cmp(&right.key));
        if facts.windows(2).any(|pair| pair[0].key == pair[1].key) {
            return Err(InventoryError::DuplicateKey);
        }
        Ok(Self { facts })
    }

    /// Facts sorted by stable key.
    pub fn facts(&self) -> &[Fact] {
        &self.facts
    }

    /// Render a deterministic, line-oriented report.
    pub fn render(&self) -> String {
        let mut output = String::from("inventory.schema=1\n");
        for fact in &self.facts {
            output.push_str(fact.key());
            output.push('=');
            output.push_str(fact.status().as_str());
            if !fact.value().is_empty() {
                output.push(':');
                output.push_str(&escape_value(fact.value()));
            }
            output.push('\n');
        }
        output
    }
}

fn escape_value(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

/// A replaceable source of inventory facts.
pub trait InventorySource {
    /// Collect one observation without changing the host or guest.
    ///
    /// # Errors
    /// Returns an adapter or protocol error when no trustworthy report can be made.
    fn collect(&self) -> Result<InventoryReport, InventoryError>;
}

/// Failure to collect or validate an inventory report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryError {
    /// The platform adapter could not be started or did not finish successfully.
    AdapterFailed,
    /// The adapter returned malformed structured output.
    InvalidProtocol,
    /// A fact key was empty or contained unsupported characters.
    InvalidKey,
    /// The adapter repeated a fact key.
    DuplicateKey,
}

impl fmt::Display for InventoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::AdapterFailed => "inventory adapter failed",
            Self::InvalidProtocol => "inventory adapter returned invalid data",
            Self::InvalidKey => "inventory adapter returned an invalid key",
            Self::DuplicateKey => "inventory adapter returned a duplicate key",
        })
    }
}

impl std::error::Error for InventoryError {}

/// Parse the adapter's tab-separated key/status/UTF-8-hex protocol.
///
/// # Errors
/// Returns an [`InventoryError`] for malformed fields, invalid UTF-8 or duplicate
/// keys. Empty output is invalid because it cannot establish adapter health.
pub fn parse_protocol(output: &str) -> Result<InventoryReport, InventoryError> {
    let mut facts = Vec::new();
    for line in output.lines() {
        let mut fields = line.split('\t');
        let (Some(key), Some(status), Some(hex), None) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            return Err(InventoryError::InvalidProtocol);
        };
        let status = FactStatus::parse(status).ok_or(InventoryError::InvalidProtocol)?;
        let value = decode_hex(hex)?;
        facts.push(Fact::new(key, status, value)?);
    }
    if facts.is_empty() {
        return Err(InventoryError::InvalidProtocol);
    }
    InventoryReport::new(facts)
}

fn decode_hex(value: &str) -> Result<String, InventoryError> {
    if !value.len().is_multiple_of(2) {
        return Err(InventoryError::InvalidProtocol);
    }
    let bytes = value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).map_err(|_| InventoryError::InvalidProtocol)?;
            u8::from_str_radix(text, 16).map_err(|_| InventoryError::InvalidProtocol)
        })
        .collect::<Result<Vec<_>, _>>()?;
    String::from_utf8(bytes).map_err(|_| InventoryError::InvalidProtocol)
}

#[cfg(test)]
mod tests {
    use super::{Fact, FactStatus, InventoryError, InventoryReport, parse_protocol};

    #[test]
    fn protocol_decodes_sorts_and_renders_all_states() {
        let report = parse_protocol(
            "vm.count\tmissing\t6E6F207265676973746572656420564D73\n\
             host.build\tknown\t32363230302E39343537\n\
             gpup.interface\tdenied\t\n\
             guest.os\tunavailable\t6E6F2073656C6563746564206775657374\n",
        )
        .unwrap();
        assert_eq!(report.facts()[0].key(), "gpup.interface");
        assert_eq!(report.facts()[0].status(), FactStatus::Denied);
        assert_eq!(report.facts()[1].value(), "no selected guest");
        assert_eq!(report.facts()[2].value(), "26200.9457");
        assert_eq!(report.facts()[3].status(), FactStatus::Missing);
        assert!(report.render().starts_with("inventory.schema=1\n"));
    }

    #[test]
    fn rejects_malformed_protocol_keys_and_duplicates() {
        for (input, expected) in [
            ("", InventoryError::InvalidProtocol),
            ("key\tknown\n", InventoryError::InvalidProtocol),
            ("key\tother\t\n", InventoryError::InvalidProtocol),
            ("key\tknown\t0\n", InventoryError::InvalidProtocol),
            ("key\tknown\tGG\n", InventoryError::InvalidProtocol),
            ("Bad-Key\tknown\t\n", InventoryError::InvalidKey),
            (
                "key\tknown\t31\nkey\tknown\t32\n",
                InventoryError::DuplicateKey,
            ),
        ] {
            assert_eq!(parse_protocol(input), Err(expected));
        }
    }

    #[test]
    fn report_rejects_duplicate_constructed_facts() {
        let fact = Fact::new("host.build", FactStatus::Known, "26200").unwrap();
        assert_eq!(
            InventoryReport::new(vec![fact.clone(), fact]),
            Err(InventoryError::DuplicateKey)
        );
    }

    #[test]
    fn rendering_escapes_record_delimiters_and_percent_signs() {
        let fact = Fact::new("test.value", FactStatus::Known, "one%\r\ntwo").unwrap();
        let report = InventoryReport::new(vec![fact]).unwrap();
        assert_eq!(
            report.render(),
            "inventory.schema=1\ntest.value=known:one%25%0D%0Atwo\n"
        );
    }
}
