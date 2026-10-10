//! Bounded public diagnostics. Adapter text is untrusted and may contain secrets.
use serde::{Deserialize, Serialize};

/// Why an observation could not be obtained; absence is not successful discovery.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Availability {
    /// The requested object does not exist.
    Missing,
    /// The caller lacks access.
    Denied,
    /// The provider or transport is unavailable.
    Unavailable,
    /// This capability is not implemented or exposed.
    Unsupported,
    /// The adapter did not establish a more specific reason.
    Unknown,
}

/// Session observation failure, with no raw adapter output or credential material.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Diagnostic {
    /// Fixed operation stage supplied by the application.
    pub stage: String,
    /// Stable observation identity, when established independently of the error.
    pub target: Option<String>,
    /// Explicit observation availability.
    pub availability: Availability,
    /// Safe action; never an automatic mutation retry.
    pub next_action: String,
}
impl Diagnostic {
    /// Classify an adapter failure without copying its text into public output.
    pub fn observation(stage: &str, target: Option<String>, error: &str) -> Self {
        let text = error.to_ascii_lowercase();
        let availability = if text.contains("access denied")
            || text.contains("access is denied")
            || text.contains("0x80070005")
            || text.contains("0x80041003")
            || text.contains("permission denied")
        {
            Availability::Denied
        } else if text.contains("not found")
            || text.contains("0x80070002")
            || text.contains("0x80041002")
        {
            Availability::Missing
        } else if text.contains("unsupported")
            || text.contains("not supported")
            || text.contains("0x80041010")
        {
            Availability::Unsupported
        } else if text.contains("unavailable")
            || text.contains("timeout")
            || text.contains("timed out")
            || text.contains("0x800706ba")
        {
            Availability::Unavailable
        } else {
            Availability::Unknown
        };
        Self {
            stage: stage.chars().take(64).collect(), target, availability,
            next_action: match availability {
                Availability::Denied => "Check operator enrollment and provider permissions; refresh after correcting access.",
                Availability::Missing => "Refresh and check the selected identity; do not infer successful removal.",
                Availability::Unsupported => "Use a supported provider/capability; no capability result was established.",
                _ => "Refresh read-only state and inspect recovery before retrying an uncertain operation.",
            }.into(),
        }
    }
    /// Public diagnostic text shared by CLI and GUI. Raw error text is omitted.
    pub fn text(&self) -> String {
        format!(
            "{}: {:?}. {}",
            self.stage, self.availability, self.next_action
        )
    }
}
/// Present arbitrary failure text safely at the frontend boundary.
pub fn operator_error(stage: &str, error: &str) -> String {
    // Match application conditions, but emit only fixed text: a matching adapter
    // message can still include secrets and must never be echoed wholesale.
    let lower = error.to_ascii_lowercase();
    let guidance = [
        ("write-capable", "The protected installation or state has unsafe write permissions. Restore trusted administrator/SYSTEM ownership and restricted ACLs before installing or operating."),
        ("untrusted", "Protected ownership or integrity validation failed. Inspect the installed artifacts and state; do not bypass trust checks."),
        ("--config file is required", "Supply --config FILE with validated runtime configuration."),
        ("unknown command", "Unknown command; use --help for supported commands."),
        ("approved plan is stale", "The approved plan is stale. Refresh and review it again before effects."),
        ("approved plan changed", "The approved plan changed. Refresh and review it again before effects."),
        ("changed after review", "The draft changed after review. Review the current draft again."),
        ("--approve-shutdown", "Guest lifecycle consent is required. Review the plan, then supply --approve-shutdown to approve graceful shutdown and restoration."),
        ("shutdown was not separately approved", "Guest lifecycle consent is required. Review and approve graceful guest shutdown and restoration before effects."),
        ("shutdown were not approved", "Guest lifecycle consent is required. Review and approve temporary guest startup, graceful shutdown and restoration before effects."),
        ("changed externally", "Configuration or snapshot changed externally. Keep the draft; resolve the conflict, discard if appropriate, and refresh."),
        ("save-only", "Inspect the recorded recovery phase. Use save-only only for verified effects; reconcile uncertain effects before another operation."),
        ("publication incomplete", "Publication is incomplete. Retain the receipt and retry saving only; do not repeat GPU effects."),
        ("worker elevation refused", "Windows elevation was refused or unavailable. Check UAC consent and the protected installation; inspect recovery before retrying."),
        ("0x800704c7", "Windows elevation was cancelled. Approve the UAC prompt when ready; inspect recovery before retrying."),
        ("restricted worker", "The restricted worker is unavailable or the invocation is invalid. Use the supported frontend and update the protected product installation if necessary."),
        ("requires an elevated", "Use an elevated administrator console for this installation operation."),
        ("not enrolled", "The selected pair is not enrolled. Review configuration and enroll it through administrator install."),
        ("no longer present", "The selected VM is no longer present. Refresh and check its identity; keep or discard the draft deliberately."),
        ("inventory is historical", "Inventory is historical. Refresh successfully before further actions."),
        ("pending recovery", "Recovery is pending. Inspect and reconcile the recorded operation; do not replay Apply."),
        ("unfinished", "An unfinished operation requires manual reconciliation. Inspect its durable record before another operation."),
        ("another gui operation", "Another GUI operation is still running. Wait for its result before continuing."),
    ].into_iter().find_map(|(needle, message)| lower.contains(needle).then_some(message));
    if let Some(guidance) = guidance {
        return format!("{stage}: {guidance}");
    }
    Diagnostic::observation(stage, None, error).text()
}

/// Remove adapter error text from structured CLI results without altering facts.
pub fn public_result(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            for (key, value) in fields {
                if (key == "save_error" || key == "error") && value.is_string() {
                    *value = operator_error("Operation result", value.as_str().unwrap_or_default())
                        .into();
                } else {
                    public_result(value);
                }
            }
        }
        serde_json::Value::Array(values) => values.iter_mut().for_each(public_result),
        _ => {}
    }
}

/// Keep successful observations when one independently identified object fails.
pub fn collect_observations<T>(
    stage: &str,
    entries: impl IntoIterator<Item = (String, Result<T, String>)>,
) -> (Vec<T>, Vec<Diagnostic>) {
    let mut values = Vec::new();
    let mut issues = Vec::new();
    for (id, result) in entries {
        match result {
            Ok(value) => values.push(value),
            Err(error) => issues.push(Diagnostic::observation(stage, Some(id), &error)),
        }
    }
    (values, issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn partial_access_preserves_other_observations() {
        let (values, issues) = collect_observations(
            "VM inspection",
            [
                ("a".into(), Ok(1)),
                ("b".into(), Err("Access is denied (0x80070005)".into())),
                ("c".into(), Ok(3)),
            ],
        );
        assert_eq!(values, [1, 3]);
        assert_eq!(issues[0].target.as_deref(), Some("b"));
        assert_eq!(issues[0].availability, Availability::Denied);
    }
    #[test]
    fn missing_provider_is_not_an_empty_success() {
        assert_eq!(
            Diagnostic::observation("GPU provider", None, "Invalid class 0x80041010").availability,
            Availability::Unsupported
        );
        assert_eq!(
            Diagnostic::observation("VM", None, "not found").availability,
            Availability::Missing
        );
        assert_eq!(
            Diagnostic::observation("VM", None, "RPC unavailable").availability,
            Availability::Unavailable
        );
        assert_eq!(
            Diagnostic::observation("VM", None, "unexpected response").availability,
            Availability::Unknown
        );
    }
    #[test]
    fn application_errors_preserve_safe_actions_without_echoing_input() {
        for (error, expected) in [
            ("approved plan is stale; private-secret", "review"),
            (
                "supply --approve-shutdown private-secret",
                "graceful shutdown",
            ),
            (
                "verified operation awaits save-only private-secret",
                "only for verified effects",
            ),
            ("worker elevation refused private-secret", "UAC"),
            ("approved plan changed private-secret", "review"),
            (
                "graceful guest shutdown was not separately approved private-secret",
                "graceful guest shutdown",
            ),
        ] {
            let message = operator_error("Apply", error);
            assert!(message.contains(expected));
            assert!(!message.contains("private-secret"));
        }
    }
    #[test]
    fn secrets_and_oversized_adapter_output_never_reach_public_diagnostics() {
        for error in [
            "password=private-password\nuser=private-user",
            "TOML input: token='private-token'",
            "unlabelled-secret",
            &"x".repeat(100_000),
        ] {
            let public = operator_error("Operation", error);
            assert!(public.len() < 256);
            assert!(!public.contains("private-"));
            assert!(!public.contains("unlabelled-secret"));
        }
        let mut result = serde_json::json!({"saved":false,"save_error":"password=private-secret", "operation":{"error":"private-user"}});
        public_result(&mut result);
        assert!(!result.to_string().contains("private-"));
        assert_eq!(result["saved"], false);
    }
}
