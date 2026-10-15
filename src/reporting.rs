//! Bounded public diagnostics. Adapter text is untrusted and may contain secrets.
use serde::{Deserialize, Serialize};

/// Fixed prefix identifying the updating transfer row, separate from stage outcomes.
pub const TRANSFER_PROGRESS_PREFIX: &str = "Copying files: ";

/// Present acknowledged transfer counters; reaching 100% never implies verification.
pub fn transfer_progress(bytes: u64, total: u64, files: usize, count: usize) -> String {
    if bytes > total || files > count {
        return format!("{TRANSFER_PROGRESS_PREFIX}progress unavailable; verification pending");
    }
    let sizes = if total == 0 {
        "0 bytes".into()
    } else {
        // Integer arithmetic avoids overflow and never rounds an incomplete copy
        // up to 100%. A zero total has no defined percentage.
        let percent = u128::from(bytes) * 100 / u128::from(total);
        format!(
            "{} of {} ({percent}%)",
            transfer_size(bytes),
            transfer_size(total)
        )
    };
    format!("{TRANSFER_PROGRESS_PREFIX}{sizes}; {files} of {count} files; verification pending")
}

fn transfer_size(bytes: u64) -> String {
    if bytes < 1000 {
        return format!("{bytes} {}", if bytes == 1 { "byte" } else { "bytes" });
    }
    const UNITS: [&str; 7] = ["bytes", "KB", "MB", "GB", "TB", "PB", "EB"];
    let bytes = u128::from(bytes);
    let mut unit = 1;
    let mut divisor = 1000;
    loop {
        let tenths = (bytes * 10 + divisor / 2) / divisor;
        if tenths >= 10_000 && unit + 1 < UNITS.len() {
            divisor *= 1000;
            unit += 1;
        } else {
            return format!("{}.{:01} {}", tenths / 10, tenths % 10, UNITS[unit]);
        }
    }
}

/// Application-owned failure context. No adapter message, path or user input is
/// carried across the authenticated worker boundary.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Component {
    /// Native Hyper-V management provider.
    HyperV,
    /// Bounded PowerShell Direct transport/guest worker.
    GuestTransport,
    /// Independent preparation receipt validation.
    GuestReceipt,
    /// Current guest driver files compared with the authenticated host manifest.
    GuestPayload,
    /// Checked guest hardware rendering.
    GuestGraphics,
}

/// A cause established at its adapter source, rather than inferred from text.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum FailureCause {
    /// The source has not established a more specific safe cause.
    Unclassified,
    /// Windows returned a failing HRESULT.
    NativeCall,
    /// A native invocation lacks independently established terminal completion.
    NativeRecoveryRequired,
    /// The provider returned a nonzero method/job result.
    ProviderRejected,
    /// A bounded guest process exited unsuccessfully or had no terminal exit.
    ProcessFailed,
    /// A guest reply could not be parsed.
    InvalidReply,
    /// Independent receipt or selected GPU identity did not match.
    ReadbackMismatch,
    /// The checked probe did not establish hardware rendering.
    RenderingFailed,
    /// The guest display device reported a nonzero Configuration Manager status.
    DeviceUnhealthy,
    /// A required driver payload file is absent.
    FileMissing,
    /// A required driver payload file has a different length or hash.
    FileChanged,
    /// The current guest observation was denied.
    AccessDenied,
    /// The bounded observation exceeded its deadline.
    TimedOut,
    /// Current guest facts could not be established.
    Unavailable,
}

/// Typed numeric code namespace; values never originate in adapter text.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum FailureCode {
    /// Native Windows HRESULT bits.
    HResult(u32),
    /// Hyper-V method/job return value.
    Provider(u64),
    /// Guest bridge process exit code.
    Exit(i32),
    /// Observed guest Configuration Manager device status.
    DeviceStatus(u32),
}

/// Fixed operation stages; labels cannot carry arbitrary adapter output.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum FailureStage {
    /// Durable operation journaling.
    RecoveryPersistence,
    /// Guest shutdown or startup.
    GuestPower,
    /// Selected GPU attach/detach.
    GpuAssignment,
    /// VM compatibility settings.
    Compatibility,
    /// Explicit provider resource write.
    Allocation,
    /// Transfer and independently checked preparation.
    Preparation,
    /// Guest health and rendering verification.
    Verification,
}

/// Safe failure envelope over existing String-based backend contracts.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentFailure {
    /// Enclosing effect stage, if the worker established it.
    #[serde(default)]
    pub stage: Option<FailureStage>,
    /// Fixed component identity.
    pub component: Component,
    /// Fixed cause established by the adapter.
    pub cause: FailureCause,
    /// Actual numeric source code, if available.
    pub code: Option<FailureCode>,
}

impl ComponentFailure {
    /// Retain typed facts without changing operation/recovery error semantics.
    pub fn into_error(self) -> String {
        format!(
            "HGFAIL1:{}",
            serde_json::to_string(&self).expect("fixed failure serialization")
        )
    }

    /// Decode bounded typed diagnostic facts for shared frontend presentation.
    /// A parsed diagnostic is not authentication or authorization evidence.
    pub fn decode(error: &str) -> Option<Self> {
        if error.len() > 8192 {
            return None;
        }
        // Existing adapters may prepend fixed operation context. Parsing must
        // consume the whole remaining envelope; appended text is never accepted.
        let (_, envelope) = error.split_once("HGFAIL1:")?;
        serde_json::from_str(envelope).ok()
    }

    /// Publish only fixed typed facts from the guest worker's failure output.
    pub fn guest_error(error: &str) -> String {
        Self::decode(error)
            .unwrap_or(Self {
                stage: None,
                component: Component::GuestTransport,
                cause: FailureCause::Unclassified,
                code: None,
            })
            .into_error()
    }

    fn text(&self) -> String {
        let code = match self.code {
            Some(FailureCode::HResult(code)) => format!("HRESULT 0x{code:08X}"),
            Some(FailureCode::Provider(code)) => format!("provider code {code}"),
            Some(FailureCode::Exit(code)) => format!("process exit {code}"),
            Some(FailureCode::DeviceStatus(code)) => format!("device status {code}"),
            None => "no terminal numeric code available".into(),
        };
        format!(
            "Stage: {:?}\nComponent: {:?}\nCause: {:?}\nCode: {code}",
            self.stage, self.component, self.cause
        )
    }
}

/// Attach fixed worker context to an already structured adapter failure.
pub(crate) fn failure_at_stage(error: String, name: &str) -> String {
    let Some(mut failure) = ComponentFailure::decode(&error) else {
        return error;
    };
    failure.stage = match name {
        "Persist operation recovery" => Some(FailureStage::RecoveryPersistence),
        "Graceful guest shutdown" | "Start guest" => Some(FailureStage::GuestPower),
        "Attach selected GPU" | "Detach selected GPU" => Some(FailureStage::GpuAssignment),
        "Compatibility settings" => Some(FailureStage::Compatibility),
        "Provider allocation" => Some(FailureStage::Allocation),
        "Prepare current signed payload" => Some(FailureStage::Preparation),
        "Check guest health and hardware rendering" => Some(FailureStage::Verification),
        _ => failure.stage,
    };
    failure.into_error()
}

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
        let text = match ComponentFailure::decode(error).and_then(|failure| failure.code) {
            Some(FailureCode::HResult(code)) => format!("0x{code:08x}"),
            _ => error.to_ascii_lowercase(),
        };
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
    if let Some((primary, restoration)) = verification_failure_parts(error) {
        return format!(
            "{} Power restoration also failed; recovery remains required. {}",
            operator_error(stage, primary),
            operator_error("Power restoration", restoration)
        );
    }
    if let Some((_, primary)) = restored_failure_parts(error) {
        return format!(
            "{} The restoration proof could not be saved or independently confirmed; recovery remains required.",
            operator_error(stage, primary)
        );
    }
    if let Some(failure) = ComponentFailure::decode(error) {
        let description = match (failure.component, failure.cause) {
            (Component::HyperV, FailureCause::NativeRecoveryRequired) => Some(
                "Native completion is unresolved. An administrator must correlate a terminal Hyper-V job with the exact VM and stage, run native-review --job LOCAL_JOB_REFERENCE, inspect its before/after facts, then approve native-reconcile --review TOKEN. If completion evidence is unavailable, preserve provider-operation.json, workflow recovery and Hyper-V event logs for administrator/provider support. Do not delete the marker or replay Apply.",
            ),
            (Component::GuestTransport, FailureCause::InvalidReply) => Some(
                "The guest check returned incomplete or invalid progress records. Its outcome is unknown; inspect recovery before another check.",
            ),
            (Component::GuestTransport, FailureCause::TimedOut) => Some(
                "Communication with the guest timed out. The guest check did not complete; inspect recovery and the recorded final power state.",
            ),
            (Component::GuestTransport, FailureCause::AccessDenied) => Some(
                "Access to the guest check transport was denied. Check local permissions and inspect recovery before retrying.",
            ),
            (Component::GuestTransport, FailureCause::Unavailable) => Some(
                "Communication with the guest was unavailable. The guest check did not complete; inspect recovery before retrying.",
            ),
            (Component::GuestGraphics, FailureCause::AccessDenied) => Some(
                "Access to guest graphics health information was denied; hardware rendering was not established.",
            ),
            (Component::GuestGraphics, FailureCause::DeviceUnhealthy) => Some(
                "The guest display device reported an unhealthy device status; hardware rendering was not checked.",
            ),
            (Component::GuestGraphics, FailureCause::Unavailable | FailureCause::InvalidReply) => {
                Some(
                    "Guest graphics health could not be established; this does not establish a rendering failure.",
                )
            }
            (Component::GuestGraphics, FailureCause::TimedOut) => Some(
                "The guest graphics health check timed out; hardware rendering was not established.",
            ),
            (Component::GuestPayload, FailureCause::FileMissing) => Some(
                "Required guest driver files are missing. Current preparation does not match the host payload.",
            ),
            (Component::GuestPayload, FailureCause::FileChanged) => {
                Some("Guest driver files differ from the current authenticated host payload.")
            }
            (Component::GuestReceipt, FailureCause::ReadbackMismatch) => Some(
                "The guest preparation receipt is absent or does not match the current payload and guest identity.",
            ),
            (Component::GuestPayload | Component::GuestReceipt, FailureCause::AccessDenied) => {
                Some("Current guest payload verification was denied; payload health is unknown.")
            }
            (Component::GuestPayload | Component::GuestReceipt, FailureCause::TimedOut) => {
                Some("Current guest payload verification timed out; payload health is unknown.")
            }
            (
                Component::GuestPayload | Component::GuestReceipt,
                FailureCause::Unavailable | FailureCause::InvalidReply,
            ) => Some("Current guest payload facts are unavailable; payload health is unknown."),
            _ => None,
        };
        if let Some(description) = description {
            return format!(
                "{stage}: {description} Refresh and inspect recovery before any reviewed repair; do not repeat Apply automatically."
            );
        }
        return format!(
            "{stage}: {:?} reported {:?}. Refresh and inspect recovery before retrying; reconcile uncertain effects rather than repeating Apply.",
            failure.component, failure.cause
        );
    }
    // Match application conditions, but emit only fixed text: a matching adapter
    // message can still include secrets and must never be echoed wholesale.
    let lower = error.to_ascii_lowercase();
    let guidance = [
        ("selected gpu is not attached", "The selected GPU is not attached on the host. Review GPU support changes or removal before Verify; no guest verification ran."),
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
        ("changed externally", "Configuration changed externally. Keep the draft; resolve the conflict, discard if appropriate, and refresh."),
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
        ("recorded outcome is not independently established", "The VM does not match the interrupted operation's approved outcome. An administrator must reconcile the partial state before new changes can be applied."),
        ("unfinished", "An unfinished operation requires manual reconciliation. Inspect its durable record before another operation."),
        ("another gui operation", "Another GUI operation is still running. Wait for its result before continuing."),
    ].into_iter().find_map(|(needle, message)| lower.contains(needle).then_some(message));
    if let Some(guidance) = guidance {
        return format!("{stage}: {guidance}");
    }
    Diagnostic::observation(stage, None, error).text()
}

/// Preserve native error codes and classified context without echoing untrusted
/// adapter text, which can contain credential material even without key labels.
pub fn technical_error(stage: &str, error: &str) -> String {
    if let Some((primary, restoration)) = verification_failure_parts(error) {
        return format!(
            "{}\n\nPower restoration failed; recovery remains required.\n{}",
            technical_error(stage, primary),
            technical_error("Power restoration", restoration)
        );
    }
    if let Some((persistence, primary)) = restored_failure_parts(error) {
        return format!(
            "{}\n\nRestoration-proof persistence failed; recovery remains required.\n{}",
            technical_error(stage, primary),
            technical_error("Restoration proof", persistence)
        );
    }
    if let Some(failure) = ComponentFailure::decode(error) {
        return format!(
            "Operation: {stage}\n{}\n{}",
            failure.text(),
            operator_error(stage, error)
        );
    }
    let facts = raw_technical_facts(error);
    format!(
        "Operation: {stage}\nBackend result: {}\nComponent message: {}\nNative error codes: {}\nOther untrusted adapter text is omitted to protect credentials.",
        operator_error(stage, error),
        facts
            .component_message
            .unwrap_or("not available as a safe structured message"),
        if facts.native_codes.is_empty() {
            "not available".into()
        } else {
            facts.native_codes.join(", ")
        },
    )
}

/// Collected safe facts, independent of rendered diagnostic headings or absence copy.
#[derive(Default)]
pub struct TechnicalFacts {
    /// A valid typed terminal component envelope was collected.
    pub component: Option<ComponentFailure>,
    /// Permitted hexadecimal native codes; untrusted prose is never included.
    pub native_codes: Vec<String>,
    /// Exact application-owned allowlisted input, before editable presentation.
    pub component_message: Option<&'static str>,
}

impl TechnicalFacts {
    /// Whether diagnostics contain collected facts beyond generic guidance.
    pub fn is_available(&self) -> bool {
        self.component.is_some()
            || !self.native_codes.is_empty()
            || self.component_message.is_some()
    }
}

/// Classify raw failure facts without consulting any rendered text.
pub fn technical_facts(error: &str) -> TechnicalFacts {
    if let Some((primary, restoration)) = verification_failure_parts(error) {
        let primary = technical_facts(primary);
        return if primary.is_available() {
            primary
        } else {
            technical_facts(restoration)
        };
    }
    if let Some(component) = ComponentFailure::decode(error) {
        return TechnicalFacts {
            component: Some(component),
            ..TechnicalFacts::default()
        };
    }
    raw_technical_facts(error)
}

/// Check input safety before an editable stage label/status is composed.
pub fn safe_progress_input(stage: &str) -> bool {
    let lower = stage.to_ascii_lowercase();
    !["password", "token", "secret", "credential", "bearer"]
        .iter()
        .any(|word| lower.contains(word))
}

fn raw_technical_facts(error: &str) -> TechnicalFacts {
    let mut codes = Vec::new();
    let lower = error.to_ascii_lowercase();
    if !["password", "token", "secret", "credential", "bearer"]
        .iter()
        .any(|word| lower.contains(word))
    {
        for word in error.split(|c: char| !c.is_ascii_alphanumeric()).take(4096) {
            if word.len() == 10
                && word.starts_with("0x")
                && word[2..].bytes().all(|c| c.is_ascii_hexdigit())
                && !codes.iter().any(|code| code == word)
                && codes.len() < 16
            {
                codes.push(word.to_owned());
            }
        }
    }
    // These complete, application-owned messages contain no interpolated input.
    // Only exact matches can be retained; a prefix/suffix may contain secrets.
    let component_message = [
        "recorded outcome is not independently established; partial state requires administrator reconciliation, never Apply replay",
        "current payload differs from the recorded operation",
        "recorded pair is no longer enrolled",
        "state changed during reconciliation",
        "operation journal does not prove the reviewed preparation and power intent",
        "reconciliation independent readback changed",
    ].into_iter().find(|message| *message == error);
    TechnicalFacts {
        component: None,
        native_codes: codes,
        component_message,
    }
}

// Only split the fixed Verify lifecycle combination. Each part is redacted
// independently; never relax the component envelope's whole-input decoder.
fn verification_failure_parts(error: &str) -> Option<(&str, &str)> {
    if error.len() > 8192 {
        return None;
    }
    error
        .strip_prefix("GPU-PV verification failed: ")?
        .split_once("; power restoration failed: ")
}

// A bounded application-owned combination retains its terminal typed primary.
// Both parts are independently redacted; no free-form adapter output is echoed.
fn restored_failure_parts(error: &str) -> Option<(&str, &str)> {
    if error.len() > 8192 {
        return None;
    }
    let parts = error
        .strip_prefix("restoration-proof persistence failed: ")?
        .split_once("; original guest failure: ")?;
    ComponentFailure::decode(parts.1)?;
    Some(parts)
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
    #[test]
    fn verification_and_restoration_failures_keep_both_safe_component_facts() {
        let primary = ComponentFailure {
            stage: Some(FailureStage::Verification),
            component: Component::GuestGraphics,
            cause: FailureCause::DeviceUnhealthy,
            code: Some(FailureCode::DeviceStatus(43)),
        }
        .into_error();
        let restoration = ComponentFailure {
            stage: Some(FailureStage::GuestPower),
            component: Component::HyperV,
            cause: FailureCause::NativeCall,
            code: Some(FailureCode::HResult(0x80070005)),
        }
        .into_error();
        let combined = format!(
            "GPU-PV verification failed: {primary}; power restoration failed: {restoration}"
        );
        assert!(ComponentFailure::decode(&combined).is_none());
        let details = technical_error("Verify", &combined);
        assert!(details.contains("device status 43"));
        assert!(details.contains("HRESULT 0x80070005"));
        assert!(details.contains("recovery remains required"));
        assert!(matches!(
            technical_facts(&combined).component.unwrap().cause,
            FailureCause::DeviceUnhealthy
        ));
        let public = operator_error("Verify", &combined);
        assert!(public.contains("unhealthy device status"));
        assert!(public.contains("Power restoration also failed"));
        let private = format!(
            "GPU-PV verification failed: private-user-input; power restoration failed: {restoration}"
        );
        assert!(!technical_error("Verify", &private).contains("private-user-input"));
        assert!(!operator_error("Verify", &private).contains("private-user-input"));
        assert!(technical_facts(&private).is_available());
    }

    #[test]
    fn transport_failure_explains_missing_records_without_echoing_private_output() {
        use super::*;
        for (cause, expected) in [
            (FailureCause::InvalidReply, "invalid progress records"),
            (FailureCause::TimedOut, "timed out"),
            (FailureCause::Unavailable, "unavailable"),
        ] {
            let error = ComponentFailure {
                stage: Some(FailureStage::Verification),
                component: Component::GuestTransport,
                cause,
                code: None,
            }
            .into_error();
            let message = operator_error("Verify", &error);
            assert!(message.contains(expected));
            assert!(message.contains("recovery"));
            assert!(!message.contains("HGFAIL1"));
        }
    }
    #[test]
    fn fact_availability_is_typed_and_redacts_original_inputs() {
        use super::*;
        assert!(!technical_facts("").is_available());
        let code = format!("0x{:08X}", 0x80070005u32);
        let facts = technical_facts(&code);
        assert!(facts.is_available());
        assert_eq!(facts.native_codes, vec![code.clone()]);
        let unsafe_input = format!("{}={code}", String::from_iter(['t', 'o', 'k', 'e', 'n']));
        assert!(!technical_facts(&unsafe_input).is_available());
        let failure = ComponentFailure {
            stage: Some(FailureStage::Verification),
            component: Component::GuestTransport,
            cause: FailureCause::Unclassified,
            code: None,
        };
        let facts = technical_facts(&failure.into_error());
        let collected = facts.component.as_ref().unwrap();
        assert!(matches!(collected.stage, Some(FailureStage::Verification)));
        assert!(matches!(collected.component, Component::GuestTransport));
        assert!(matches!(collected.cause, FailureCause::Unclassified));
        assert!(facts.is_available());
    }

    use super::*;

    #[test]
    fn payload_failures_distinguish_mismatch_from_unknown_without_claiming_rollback() {
        for (cause, text) in [
            (FailureCause::FileMissing, "files are missing"),
            (FailureCause::FileChanged, "files differ"),
            (FailureCause::AccessDenied, "was denied"),
            (FailureCause::TimedOut, "timed out"),
            (FailureCause::Unavailable, "facts are unavailable"),
        ] {
            let error = ComponentFailure {
                stage: Some(FailureStage::Verification),
                component: Component::GuestPayload,
                cause,
                code: None,
            }
            .into_error();
            let message = operator_error("Verify", &error);
            assert!(message.contains(text), "{message}");
            assert!(!message.contains("rollback"));
        }
    }

    #[test]
    fn transfer_sizes_use_readable_decimal_units_and_promote_rounded_boundaries() {
        for (bytes, expected) in [
            (0, "0 bytes"),
            (1, "1 byte"),
            (999, "999 bytes"),
            (1000, "1.0 KB"),
            (1536, "1.5 KB"),
            (999_949, "999.9 KB"),
            (999_950, "1.0 MB"),
            (1_000_000, "1.0 MB"),
            (3_409_254_468, "3.4 GB"),
            (u64::MAX, "18.4 EB"),
        ] {
            assert_eq!(transfer_size(bytes), expected);
        }
    }

    #[test]
    fn transfer_percent_is_acknowledged_progress_without_verification_or_overflow() {
        assert_eq!(
            transfer_progress(500_000, 1_000_000, 1, 3),
            "Copying files: 500.0 KB of 1.0 MB (50%); 1 of 3 files; verification pending"
        );
        assert!(transfer_progress(999_999, 1_000_000, 2, 3).contains("(99%)"));
        assert!(transfer_progress(u64::MAX - 1, u64::MAX, 0, 1).contains("(99%)"));
        let complete = transfer_progress(u64::MAX, u64::MAX, 3, 3);
        assert!(complete.contains("(100%)"));
        assert!(complete.ends_with("verification pending"));
        let empty = transfer_progress(0, 0, 0, 0);
        assert!(!empty.contains('%'));
        assert!(empty.contains("0 bytes; 0 of 0 files"));
        for invalid in [transfer_progress(1, 0, 0, 1), transfer_progress(1, 1, 2, 1)] {
            assert!(invalid.contains("progress unavailable"));
            assert!(!invalid.contains('%'));
        }
    }
    #[test]
    fn typed_component_facts_survive_stage_binding_and_reject_untrusted_fields() {
        let error = ComponentFailure {
            stage: None,
            component: Component::HyperV,
            cause: FailureCause::NativeCall,
            code: Some(FailureCode::HResult(0x80070005)),
        }
        .into_error();
        let error = failure_at_stage(error, "Attach selected GPU");
        let details = technical_error("Apply", &error);
        assert!(details.contains("GpuAssignment"));
        assert!(details.contains("0x80070005"));
        assert_eq!(
            Diagnostic::observation("Apply", None, &error).availability,
            Availability::Denied
        );
        assert!(ComponentFailure::decode(&format!("{error}; private-secret")).is_none());
        let mut value: serde_json::Value =
            serde_json::from_str(error.strip_prefix("HGFAIL1:").unwrap()).unwrap();
        value["password"] = "private-secret".into();
        let injected = format!("HGFAIL1:{value}");
        assert!(ComponentFailure::decode(&injected).is_none());
        assert!(!technical_error("Apply", &injected).contains("private-secret"));
    }
    #[test]
    fn technical_diagnostics_preserve_codes_without_echoing_adapter_text() {
        let details = technical_error("Apply", "Access is denied (0x80070005); private-content");
        assert!(details.contains("0x80070005"));
        assert!(details.contains("Denied"));
        assert!(!details.contains("private-content"));
        let details = technical_error("Apply", "password=0x12345678 token=private-content");
        assert!(!details.contains("0x12345678"));
        assert!(!details.contains("private-content"));
    }
    #[test]
    fn reconciliation_failure_retains_exact_safe_reason_but_refuses_added_text() {
        let error = "recorded outcome is not independently established; partial state requires administrator reconciliation, never Apply replay";
        assert!(technical_error("Reconciliation", error).contains(error));
        assert!(operator_error("Reconciliation", error).contains("does not match"));
        let details = technical_error("Reconciliation", &format!("{error}; private-secret"));
        assert!(!details.contains(error));
        assert!(!details.contains("private-secret"));
    }
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
