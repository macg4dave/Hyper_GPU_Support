//! Additional diagnostic facts only; arbitrary adapter text stays redacted.
use super::live_state::State;
use hyper_gpu_support::reporting::{ComponentFailure, technical_error};

fn normalized(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Preserve two independently redacted failures when completion reads also fail.
#[cfg(any(windows, test))]
pub(super) fn completion_failure(summary: &str, details: &str, primary: &str) -> (String, String) {
    let original = hyper_gpu_support::reporting::operator_error("Original operation", primary);
    let primary_details = failure_details("Original operation", primary, &[]);
    (
        format!(
            "{summary}\n\n{original}\nCompletion records could not be confirmed. Use Refresh and inspect recovery before another operation."
        ),
        [details, primary_details.as_str()]
            .into_iter()
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"),
    )
}

/// Callers supply application-owned or reporting-redacted diagnostic content.
pub(super) fn additional_details(summary: &str, details: &str) -> String {
    if normalized(details).is_empty() || normalized(details) == normalized(summary) {
        String::new()
    } else {
        details.trim().to_owned()
    }
}

pub(super) fn failure_details(title: &str, error: &str, stages: &[String]) -> String {
    let details = technical_error(title, error);
    let collected = ComponentFailure::decode(error).is_some()
        || details.lines().any(|line| {
            line.strip_prefix("Native error codes: ")
                .is_some_and(|value| value != "not available")
                || line
                    .strip_prefix("Component message: ")
                    .is_some_and(|value| value != "not available as a safe structured message")
        });
    // Discard suspicious free text rather than exposing a malformed status row.
    let safe_stages = stages
        .iter()
        .filter(|stage| {
            let lower = stage.to_ascii_lowercase();
            !["password", "token", "secret", "credential", "bearer"]
                .iter()
                .any(|word| lower.contains(word))
        })
        .cloned()
        .collect::<Vec<_>>();
    if !collected && safe_stages.is_empty() {
        return String::new();
    }
    let mut result = "Source: this reported failure and its last reported stages; current VM state may require Refresh.\n".to_owned();
    if collected {
        result.push_str(&details);
    }
    if !safe_stages.is_empty() {
        result.push_str("\nLast reported stages (not proof of final state):\n");
        result.push_str(&safe_stages.join("\n"));
    }
    result
}

pub(super) fn selected_vm_details(state: &State) -> String {
    let Some(id) = state.view.selected.as_deref() else {
        return String::new();
    };
    let saved = state.committed.revision(id);
    let inventory = state.view.inventory.as_ref();
    let facts = serde_json::json!({
        "vm_id": id,
        "observed_host_state": state.view.vm(id),
        "saved_intent_revision": saved.as_ref().ok(),
        "saved_intent_read_error": saved.as_ref().err(),
        "pending_edits": if state.view.draft.is_some() { "Unapplied draft; see editor for current raw values" } else { "No draft recorded" },
        "saved_intent": if saved.is_ok() { state.committed.documents.get(id).map(|document| &document.target) } else { None },
        "authorized_pair": inventory.and_then(|inventory| inventory.enrolled.iter().find(|target| target.vm_id == id)),
        "recorded_managed_history": inventory.and_then(|inventory| inventory.managed.get(id)).and_then(Option::as_ref),
        "recorded_history_status": inventory.map(|inventory| inventory.recorded_state(id)),
    });
    let source = if state.view.needs_readback || state.view.vm(id).is_none() {
        "Current observation unavailable. Retained host observations/history below are prior reads, not current health or execution authority; use Refresh."
    } else {
        "Source: latest host inventory and saved app records. No fresh guest check; setup/rendering history is historical, not current guest health."
    };
    let facts = serde_json::to_string_pretty(&facts)
        .unwrap_or_else(|_| "Diagnostic formatting unavailable".into());
    format!(
        "{source}\n{facts}\n\nHost comparison:\n{}\n\nApplied operation:\n{}\n\nVRAM values are exact byte requests/readback; provider values do not establish reserved memory or enforced isolation. Missing records and null fields mean unavailable/absent facts, not proof of success.",
        state.host_configuration_text(id),
        state.applied_text(id)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_read_failure_keeps_the_original_typed_failure_without_adapter_secrets() {
        use hyper_gpu_support::reporting::{Component, ComponentFailure, FailureCause};
        let primary = ComponentFailure {
            stage: None,
            component: Component::GuestPayload,
            cause: FailureCause::FileMissing,
            code: None,
        }
        .into_error();
        let (body, details) =
            completion_failure("Completion read unavailable", "Readback facts", &primary);
        assert!(body.contains("Completion read unavailable"));
        assert!(body.contains("driver files are missing"));
        assert!(body.contains("inspect recovery"));
        assert!(details.contains("Readback facts") && details.contains("GuestPayload"));
        let (body, details) =
            completion_failure("Read unavailable", "", "unlabelled-private-value");
        assert!(!body.contains("unlabelled-private-value"));
        assert!(!details.contains("unlabelled-private-value"));
    }
    #[test]
    fn empty_and_duplicate_details_do_not_create_a_second_summary() {
        assert!(additional_details("Refresh first", "").is_empty());
        assert!(additional_details("Refresh first", " \nRefresh   first\n").is_empty());
        assert_eq!(
            additional_details("Refresh first", "VM ID: recorded-id"),
            "VM ID: recorded-id"
        );
    }
    #[test]
    fn generic_failure_has_no_details_without_collected_facts() {
        assert!(failure_details("Apply", "opaque adapter failure", &[]).is_empty());
        assert!(failure_details("Apply", "password=private-secret", &[]).is_empty());
        let details = failure_details("Apply", "Access denied (0x80070005); private-content", &[]);
        assert!(details.contains("0x80070005"));
        assert!(!details.contains("private-content"));
    }
    #[test]
    fn failure_stage_context_is_additional_and_suspicious_rows_are_omitted() {
        let stages = vec![
            "Starting the VM · Complete".into(),
            "token=private-secret".into(),
        ];
        let details = failure_details("Apply", "opaque adapter failure", &stages);
        assert!(details.contains("Starting the VM"));
        assert!(details.contains("not proof of final state"));
        assert!(!details.contains("private-secret"));
        assert!(
            failure_details(
                "Apply",
                "opaque failure",
                &["password=private-secret".into()]
            )
            .is_empty()
        );
    }
}
