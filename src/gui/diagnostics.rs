//! Additional diagnostic facts only; arbitrary adapter text stays redacted.
use super::live_state::State;
use hyper_gpu_support::reporting::technical_error;

/// Build both safe views from the original failure, before summary formatting.
pub(super) fn inventory_failure(error: &str) -> (String, String) {
    use hyper_gpu_support::ui_text::operations::inventory;
    let reason = if error.contains("release is unconfirmed")
        || error.contains("has not confirmed release")
        || error.contains("launch outcome is unconfirmed")
    {
        inventory::READER_UNRELEASED
    } else if error.contains("Invalid protected observation")
        || error.contains("Observation configuration revision differs")
        || error.contains("Protected observation mode differs")
    {
        inventory::INCONSISTENT
    } else if hyper_gpu_support::reporting::technical_facts(error)
        .native_codes
        .iter()
        .any(|code| matches!(code.as_str(), "0x80070005" | "0x80041003"))
    {
        inventory::ACCESS_DENIED
    } else if error.to_ascii_lowercase().contains("timed out")
        || error.to_ascii_lowercase().contains("deadline")
    {
        inventory::TIMED_OUT
    } else {
        ""
    };
    let summary = if reason.is_empty() {
        inventory::REFRESH_FAILED.to_owned()
    } else {
        format!("{} {reason}", inventory::REFRESH_FAILED)
    };
    (summary, technical_error(inventory::FAILURE_STAGE, error))
}

fn normalized(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn failure_summary(title: &str, error: &str, context: &str) -> String {
    format!(
        "{}\n\n{context}",
        hyper_gpu_support::reporting::operator_error(title, error)
    )
}

/// Preserve two independently redacted failures when completion reads also fail.
#[cfg(any(windows, test))]
pub(super) fn completion_failure(summary: &str, details: &str, primary: &str) -> (String, String) {
    let original = hyper_gpu_support::reporting::operator_error(
        hyper_gpu_support::ui_text::diagnostics::stage::ORIGINAL,
        primary,
    );
    let primary_details = failure_details(
        hyper_gpu_support::ui_text::diagnostics::stage::ORIGINAL,
        primary,
        &[],
    );
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
    let summary = normalized(summary);
    // A subset of the summary is still repetition, including reordered rows.
    if details.lines().all(|line| {
        let line = normalized(line);
        line.is_empty() || summary.contains(&line)
    }) {
        String::new()
    } else {
        details.trim().to_owned()
    }
}

pub(super) fn failure_details(title: &str, error: &str, stages: &[String]) -> String {
    let details = technical_error(title, error);
    let collected = hyper_gpu_support::reporting::technical_facts(error).is_available();
    // Callers filter using immutable worker-input metadata before rendering.
    let safe_stages = stages;
    if !collected && safe_stages.is_empty() {
        return String::new();
    }
    let mut result = hyper_gpu_support::ui_text::diagnostics::failure::PROVENANCE.to_owned();
    if collected {
        result.push_str(&details);
    }
    if !safe_stages.is_empty() {
        result.push_str(hyper_gpu_support::ui_text::diagnostics::failure::STAGES_HEADING);
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
    let pair =
        inventory.and_then(|inventory| inventory.enrolled.iter().find(|target| target.vm_id == id));
    let gpu_interface = state
        .draft_gpu
        .as_deref()
        .or_else(|| pair.map(|pair| pair.gpu_interface.as_str()));
    let gpu = inventory.and_then(|inventory| {
        inventory
            .discovery
            .gpus
            .iter()
            .find(|gpu| Some(gpu.interface.as_str()) == gpu_interface)
    });
    let issues = inventory.map(|inventory| {
        inventory
            .discovery
            .issues
            .iter()
            .filter(|issue| {
                issue
                    .target
                    .as_deref()
                    .is_none_or(|target| target == id || Some(target) == gpu_interface)
            })
            .collect::<Vec<_>>()
    });
    let memory_notes = gpu.and_then(|gpu| {
        let capacity = gpu.physical_vram_bytes?;
        let limits = gpu.limits(hyper_gpu_support::model::Resource::Vram)?;
        let current = state.view.vm(id).and_then(|vm| vm.vram.as_ref());
        super::memory::MemorySlider::new(limits, current, capacity)
            .ok()?
            .range_note
    });
    let facts = serde_json::json!({
        "memory_display_notes": memory_notes,
        "vm_id": id,
        "observed_host_state": state.view.vm(id),
        "saved_intent_revision": saved.as_ref().ok(),
        "saved_intent_read_error": saved.as_ref().err(),
        "pending_edits": if state.view.draft.is_some() { hyper_gpu_support::ui_text::diagnostics::draft::PRESENT } else { hyper_gpu_support::ui_text::diagnostics::draft::ABSENT },
        "saved_intent": if saved.is_ok() { state.committed.documents.get(id).map(|document| &document.target) } else { None },
        "authorized_pair": pair,
        "selected_gpu_interface": gpu_interface,
        "observed_gpu": gpu,
        "inventory_diagnostics": issues,
        "recorded_managed_history": inventory.and_then(|inventory| inventory.managed.get(id)).and_then(Option::as_ref),
        "recorded_history_status": inventory.map(|inventory| inventory.recorded_state(id)),
    });
    let source = if state.view.needs_readback || state.view.vm(id).is_none() {
        hyper_gpu_support::ui_text::diagnostics::observation::HISTORICAL
    } else {
        hyper_gpu_support::ui_text::diagnostics::observation::CURRENT
    };
    let facts = serde_json::to_string_pretty(&facts).unwrap_or_else(|_| {
        hyper_gpu_support::ui_text::diagnostics::formatting::UNAVAILABLE.into()
    });
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
    fn unsupported_os_preflight_keeps_a_clear_block_reason() {
        use hyper_gpu_support::reporting::{
            Component, ComponentFailure, FailureCause, FailureStage,
        };
        let error = ComponentFailure {
            stage: Some(FailureStage::GuestOsPreflight),
            component: Component::GuestCompatibility,
            cause: FailureCause::UnsupportedGuest,
            code: None,
        }
        .into_error();
        assert!(hyper_gpu_support::reporting::installation_preflight_blocked(&error));
        let body = failure_summary(
            "GPU support installation blocked",
            &error,
            "This attempt stopped before driver installation or VM power changes.",
        );
        assert!(body.contains("unsupported"));
        assert!(body.contains("Windows 11"));
        assert!(body.contains("Linux"));
        assert!(!body.contains("Unknown"));
        assert!(!body.contains("Changes may have started"));
        assert!(failure_details("OS check", &error, &[]).contains("UnsupportedGuest"));
    }

    #[test]
    fn failure_context_does_not_corrupt_the_typed_envelope_or_echo_adapter_output() {
        use hyper_gpu_support::reporting::{Component, ComponentFailure, FailureCause};
        let error = ComponentFailure {
            stage: None,
            component: Component::GuestPayload,
            cause: FailureCause::FileMissing,
            code: None,
        }
        .into_error();
        let body = failure_summary("Verify", &error, "VM1: prior checks are historical.");
        assert!(body.contains("driver files are missing"));
        assert!(body.contains("prior checks are historical"));
        assert!(failure_details("Verify", &error, &[]).contains("GuestPayload"));
        assert!(
            !failure_summary("Verify", "unlabelled-private-value", "VM1")
                .contains("unlabelled-private-value")
        );
    }
    #[test]
    fn inventory_details_scope_failures_and_preserve_stale_gpu_facts_without_adapter_text() {
        use hyper_gpu_support::{
            gui_model::Inventory,
            model::{Discovery, Gpu},
            reporting::Diagnostic,
        };
        let mut state = State::default();
        state.view.selected = Some("selected-vm".into());
        state.draft_gpu = Some("selected-gpu".into());
        let gpu: Gpu = serde_json::from_value(serde_json::json!({
            "interface": "selected-gpu", "name": "Test card", "vendor": 4318,
            "device": 1, "driver_version": "test-version", "preparation_supported": true,
            "physical_vram_bytes": 8278507520u64, "total_vram": 1000000000u64
        }))
        .expect("isolated GPU fixture");
        state.view.inventory = Some(Inventory {
            discovery: Discovery {
                guest_os: Default::default(),
                vms: vec![],
                gpus: vec![gpu],
                issues: vec![
                    Diagnostic::observation(
                        "GPU inventory",
                        None,
                        "password=private-secret Access denied",
                    ),
                    Diagnostic::observation(
                        "VM readback",
                        Some("selected-vm".into()),
                        "unavailable",
                    ),
                    Diagnostic::observation(
                        "Physical capacity",
                        Some("selected-gpu".into()),
                        "unavailable",
                    ),
                    Diagnostic::observation("Unrelated VM", Some("other-vm".into()), "unavailable"),
                ],
            },
            managed: Default::default(),
            enrolled: vec![],
        });
        state.view.needs_readback = true;
        let details = selected_vm_details(&state);
        assert!(details.contains("prior reads, not current health"));
        for fact in [
            "8278507520",
            "1000000000",
            "test-version",
            "GPU inventory",
            "VM readback",
            "Physical capacity",
            "Denied",
        ] {
            assert!(details.contains(fact), "missing collected fact: {fact}");
        }
        assert!(!details.contains("private-secret"));
        assert!(!details.contains("other-vm"));
        state.view.selected = Some("other-vm".into());
        state.draft_gpu = None;
        let details = selected_vm_details(&state);
        assert!(!details.contains("selected-gpu"));
        assert!(!details.contains("test-version"));
        assert!(!details.contains("VM readback"));
        assert!(details.contains("Unrelated VM"));
    }

    #[test]
    fn selection_changes_clear_old_identity_and_redact_unreadable_saved_intent() {
        let first = "11111111-1111-1111-1111-111111111111";
        let second = "22222222-2222-2222-2222-222222222222";
        let mut state = State::default();
        assert!(selected_vm_details(&state).is_empty());
        state.view.selected = Some(first.into());
        state.view.needs_readback = true;
        state
            .committed
            .errors
            .insert(format!("{first}.toml"), "token=private-value".into());
        let details = selected_vm_details(&state);
        assert!(details.contains(first));
        assert!(details.contains("Current observation unavailable"));
        assert!(details.contains("saved_intent_read_error"));
        assert!(!details.contains("private-value"));
        state.view.selected = Some(second.into());
        let details = selected_vm_details(&state);
        assert!(details.contains(second));
        assert!(!details.contains(first));
        state.view.selected = None;
        assert!(selected_vm_details(&state).is_empty());
    }

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
    fn duplicate_presentation_keeps_additional_facts_under_edited_summaries() {
        let summary = String::from_iter(['x']);
        let additional = String::from_iter(['y', '{', '}']);
        assert!(additional_details(&summary, "").is_empty());
        assert!(additional_details(&summary, &format!(" \n{summary}\n")).is_empty());
        assert_eq!(additional_details(&summary, &additional), additional);
        let edited = String::from_iter(['z', '\n']);
        assert_eq!(additional_details(&edited, &additional), additional);
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
    fn stage_visibility_uses_original_input_metadata_even_when_labels_change() {
        let original = String::from_iter(['x']);
        let edited = String::from_iter(['t', 'o', 'k', 'e', 'n', '=', 'λ']);
        assert!(hyper_gpu_support::reporting::safe_progress_input(&original));
        assert!(!hyper_gpu_support::reporting::safe_progress_input(&edited));
        // The immutable original was safe; its edited label is retained as a fact.
        let details = failure_details("", "", std::slice::from_ref(&edited));
        assert!(details.contains(&edited));
        assert!(failure_details("", "", &[]).is_empty());
    }
}
