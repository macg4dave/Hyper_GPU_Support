use super::*;

#[test]
fn review_distinguishes_requested_triples_and_no_write_semantics() {
    let mut plan = review_plan();
    let first = review_text(&plan);
    let summary = review_summary(&plan);
    assert!(summary.contains("GPU support: Disabled → Enabled"));
    assert!(!summary.contains("10 / 50 / 100"));
    assert!(!summary.contains("GPU interface"));
    assert!(first.contains("VRAM draft (Minimum / Optimal / Maximum): 10 / 50 / 100"));
    assert!(first.contains("Planned allocation write: 10 / 50 / 100"));
    plan.desired.vram = Some(Allocation {
        minimum: 20,
        optimal: 60,
        maximum: 90,
    });
    plan.preview.allocation = plan.desired.vram.clone();
    assert_ne!(first, review_text(&plan));
    plan.preview.allocation = None;
    assert!(review_text(&plan).contains("No allocation write planned"));
    plan.desired.vram = None;
    assert!(review_text(&plan).contains("Unspecified; no explicit VRAM request"));
}

#[test]
fn memory_review_shows_readable_values_and_retains_exact_bytes_in_details() {
    let mut plan = review_plan();
    plan.observed.vram = Some(Allocation {
        minimum: 0,
        optimal: 1_000_000_000,
        maximum: 1_000_000_000,
    });
    plan.preview.allocation = Some(Allocation {
        minimum: 1,
        optimal: 500_000_001,
        maximum: 900_000_003,
    });
    let text = memory_review(&plan);
    assert!(text.contains("0 GB / 1 GB / 1 GB"));
    assert!(text.contains("0.500000001 GB"));
    assert!(!text.contains("1000000000"));
    assert!(review_text(&plan).contains("1 / 500000001 / 900000003"));
    // Unknown readback cannot become a default or zero observation.
    plan.observed.vram = None;
    assert!(memory_review(&plan).contains("Unknown → 0.000000001 GB"));
    plan.preview.allocation = Some(Allocation {
        minimum: 4_000_000_000,
        optimal: 4_000_000_000,
        maximum: 4_000_000_000,
    });
    assert!(memory_review(&plan).contains("Unknown → 4 GB"));
}

#[test]
fn memory_review_distinguishes_saved_request_from_a_planned_write() {
    let mut plan = review_plan();
    plan.preview.allocation = None;
    let text = memory_review(&plan);
    assert!(text.contains("0.00000001 GB / 0.00000005 GB / 0.0000001 GB"));
    assert!(text.contains("No VRAM write is planned"));
    assert!(!text.contains('→'));
    plan.desired.vram = None;
    let text = memory_review(&plan);
    assert!(text.contains("no explicit request"));
    assert!(!text.contains("bytes:"));
}

#[test]
fn review_reports_ordered_power_steps_changed_settings_and_time_warnings() {
    use hyper_gpu_support::workflow::{PreviewAction, SettingsPreview};
    let mut plan = review_plan();
    plan.observed.power = Power::Running;
    plan.preview.guest_downtime = true;
    plan.preview.restore_power = Power::Running;
    plan.preview.actions = vec![
        PreviewAction::Shutdown,
        PreviewAction::Start,
        PreviewAction::Prepare,
        PreviewAction::Shutdown,
        PreviewAction::Settings,
        PreviewAction::Attach,
        PreviewAction::Start,
        PreviewAction::Verify,
    ];
    let before = plan.observed.settings.clone();
    let mut after = before.clone();
    after.high_mmio = 32768;
    plan.preview.settings = Some(SettingsPreview { before, after });
    let summary = review_summary(&plan);
    assert!(summary.contains("VM: Renamable VM"));
    assert!(summary.contains("1. Gracefully stop the VM."));
    assert!(summary.contains("2. Start the VM."));
    assert!(summary.contains("4. Gracefully stop the VM."));
    assert!(summary.contains("Expected final VM power after successful completion: Running."));
    assert!(summary.contains("High MMIO (MiB): 0 → 32768"));
    assert!(!summary.contains("Low MMIO (MiB)"));
    assert!(summary.contains("Save work inside it"));
    assert!(summary.contains("several minutes"));
    assert!(summary.contains("Shutdown refusal"));
    assert!(summary.contains("graphics check may take additional time"));
    plan.preview.actions.clear();
    plan.preview.guest_downtime = false;
    plan.preview.settings = None;
    let summary = review_summary(&plan);
    assert!(summary.contains("No VM or guest changes are needed"));
    assert!(!summary.contains("several minutes"));
    assert!(!summary.contains("Save work inside it"));
    assert!(!summary.contains("graphics check may take additional time"));
}

#[test]
fn stopped_installation_review_shows_os_check_before_preparation() {
    use hyper_gpu_support::workflow::PreviewAction;
    let mut plan = review_plan();
    plan.preview.actions = vec![
        PreviewAction::Start,
        PreviewAction::CheckOs,
        PreviewAction::Shutdown,
        PreviewAction::Start,
        PreviewAction::Prepare,
    ];
    let text = review_summary(&plan);
    assert!(text.contains("1. Start the VM."));
    assert!(text.contains("2. Identify the running guest OS."));
    assert!(text.contains("failed identification stops setup"));
    assert!(text.contains("3. Gracefully stop the VM."));
    assert!(text.contains("5. Prepare the guest"));
}

#[test]
fn initial_setup_review_includes_graphics_and_successful_final_off() {
    use hyper_gpu_support::workflow::PreviewAction;
    let mut plan = review_plan();
    plan.initial_setup = true;
    plan.preview.restore_power = Power::Off;
    plan.preview.actions = vec![
        PreviewAction::Start,
        PreviewAction::Verify,
        PreviewAction::Shutdown,
    ];
    let summary = review_summary(&plan);
    assert!(summary.contains("Initial GPU setup"));
    assert!(summary.contains("hardware rendering"));
    assert!(summary.contains("Expected final VM power after successful completion: Stopped."));
    assert!(summary.contains("3. Gracefully stop the VM."));
    assert!(summary.contains("Start it through Hyper-V"));
}

#[test]
fn ordinary_review_has_final_shutdown_without_guest_work_or_restart() {
    use hyper_gpu_support::workflow::PreviewAction;
    let mut plan = review_plan();
    plan.preview.credentials_required = false;
    plan.preview.restore_power = Power::Off;
    plan.preview.actions = vec![PreviewAction::Shutdown, PreviewAction::Allocation];
    let summary = review_summary(&plan);
    assert!(summary.contains("Ordinary GPU change"));
    assert!(summary.contains("no guest access or graphics check"));
    assert!(summary.contains("Expected final VM power after successful completion: Stopped."));
    assert!(!summary.contains("several minutes"));
    assert!(!summary.contains("graphics check may take additional time"));
    assert!(!summary.contains("Start the VM."));
    plan.authentication_deferred = true;
    let summary = review_summary(&plan);
    assert!(summary.contains("Preliminary review"));
    assert!(summary.contains("requires a full review"));
    assert!(summary.contains("before any changes"));
}

#[test]
fn driver_update_review_explicitly_names_required_setup_and_time() {
    use hyper_gpu_support::workflow::PreviewAction;
    let mut plan = review_plan();
    plan.preview.credentials_required = true;
    plan.preview.actions = vec![
        PreviewAction::Prepare,
        PreviewAction::Verify,
        PreviewAction::Shutdown,
    ];
    plan.preview.restore_power = Power::Off;
    let summary = review_summary(&plan);
    assert!(summary.contains("GPU setup/update required"));
    assert!(summary.contains("several minutes"));
    assert!(summary.contains("graphics check may take additional time"));
    assert!(!summary.contains("Ordinary GPU change"));
}

#[test]
fn verify_review_distinguishes_current_checks_from_history_and_technical_identity() {
    use hyper_gpu_support::workflow::VerificationPlan;
    let s = state();
    let mut plan = VerificationPlan {
        target: s.target(id(&s)).unwrap(),
        observed: s.view.vm(id(&s)).unwrap().clone(),
        gpu: s.view.inventory.as_ref().unwrap().discovery.gpus[0].clone(),
        managed: None,
        payload_digest: "authenticated-fixture-digest".into(),
    };
    let summary = crate::gui::wording::verify_review(&plan);
    assert!(summary.starts_with("Virtual machine: Renamable VM"));
    assert!(summary.contains("current guest driver files and setup record"));
    assert!(summary.contains("past successful check alone does not establish current health"));
    assert!(summary.contains("Expected final power after successful completion: Off"));
    assert!(!summary.contains(&plan.payload_digest));
    assert!(!summary.contains(&plan.target.vm_id));
    let details = crate::gui::wording::verify_details(&plan);
    assert!(details.contains(&plan.target.vm_id));
    assert!(details.contains(&plan.payload_digest));
    assert!(crate::gui::wording::verify_consent(&plan).contains("temporarily starting"));
    plan.observed.power = Power::Running;
    assert!(crate::gui::wording::verify_review(&plan).contains("remain Running"));
    assert!(!crate::gui::wording::verify_consent(&plan).contains("shutting it down"));
    plan.observed.power = Power::Other(4);
    assert!(crate::gui::wording::verify_review(&plan).contains("Unknown"));
    assert!(crate::gui::wording::verify_consent(&plan).contains("Refresh"));
}

#[test]
fn apply_consent_uses_reviewed_final_power_instead_of_initial_power_restoration() {
    let mut plan = review_plan();
    plan.observed.power = Power::Running;
    plan.preview.restore_power = Power::Off;
    let consent = crate::gui::wording::apply_consent(&plan);
    assert!(consent.contains("successful completion: Off"));
    assert!(!consent.contains("restor"));
    plan.observed.power = Power::Off;
    plan.initial_setup = true;
    assert!(crate::gui::wording::apply_consent(&plan).contains("planned starts/shutdowns"));
    plan.preview.restore_power = Power::Other(4);
    assert!(crate::gui::wording::apply_consent(&plan).contains("Unknown"));
}

#[test]
fn verification_only_apply_review_does_not_claim_driver_preparation_is_required() {
    use hyper_gpu_support::workflow::PreviewAction;
    let mut plan = review_plan();
    plan.preview.credentials_required = true;
    plan.preview.actions = vec![
        PreviewAction::Start,
        PreviewAction::Verify,
        PreviewAction::Shutdown,
    ];
    plan.preview.restore_power = Power::Off;
    let summary = review_summary(&plan);
    assert!(summary.contains("Verify GPU setup required"));
    assert!(summary.contains("without reinstalling drivers"));
    assert!(!summary.contains("GPU setup/update required"));
    assert!(!summary.contains("Driver preparation and file transfer"));
    assert!(summary.contains("Expected final VM power after successful completion: Stopped"));
}
