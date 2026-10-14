//! User-facing operation wording derived from reviewed facts, never inferred health.
use super::operation::{GuestPurpose, Kind};
use hyper_gpu_support::{
    model::Power,
    workflow::{Plan, VerificationPlan},
};

fn power_name(power: &Power) -> hyper_gpu_support::ui_text::UiMessage {
    hyper_gpu_support::ui_text::power(power)
}

pub(super) fn apply_consent(plan: &Plan) -> String {
    hyper_gpu_support::ui_text::operations::apply::power_consent(&power_name(
        &plan.preview.restore_power,
    ))
    .into_string()
}

pub(super) fn recovery_consent(final_power: &Power) -> String {
    hyper_gpu_support::ui_text::operations::recovery::power_consent(&power_name(final_power))
        .into_string()
}

pub(super) fn verify_review(plan: &VerificationPlan) -> String {
    let lifecycle = match plan.observed.power {
        Power::Off => hyper_gpu_support::ui_text::operations::verify::lifecycle::off(),
        Power::Running => hyper_gpu_support::ui_text::operations::verify::lifecycle::running(),
        Power::Other(_) => hyper_gpu_support::ui_text::operations::verify::lifecycle::unknown(),
    };
    hyper_gpu_support::ui_text::operations::verify::review::body(
        &plan.observed.name,
        &plan.gpu.name,
        &lifecycle,
        &power_name(&plan.observed.power),
    )
    .into_string()
}

pub(super) fn verify_details(plan: &VerificationPlan) -> String {
    hyper_gpu_support::ui_text::operations::verify::review::details(
        &plan.target.vm_id,
        &plan.target.gpu_interface,
        &plan.gpu.driver_version,
        &plan.payload_digest,
        &hyper_gpu_support::ui_text::power_debug(&plan.observed.power),
    )
    .into_string()
}

pub(super) fn verify_consent(plan: &VerificationPlan) -> String {
    match plan.observed.power {
        Power::Off => hyper_gpu_support::ui_text::operations::verify::consent::OFF.into(),
        Power::Running => hyper_gpu_support::ui_text::operations::verify::consent::RUNNING.into(),
        Power::Other(_) => hyper_gpu_support::ui_text::operations::verify::consent::UNKNOWN.into(),
    }
}

pub(super) fn verification_recovery(
    record: &hyper_gpu_support::configuration_store::SaveRecord,
    name: &str,
    restored: bool,
) -> String {
    let action = if restored {
        "Check recorded outcome independently confirms the recorded restoration before ending this failed check. It does not retry the guest check or report success. Changed facts retain recovery."
    } else {
        "Check recorded outcome runs a new guest file, device and rendering check after fresh host checks. It requires guest credentials and separate power consent. This is a retry of verification, not GPU setup or driver repair."
    };
    format!(
        "{name}: GPU verification did not complete. The recovery hold blocks new changes on this host.\n\n{action}\n\nRecorded final power: {}. No saved GPU/VRAM settings or driver files are changed by this check. Previous successful checks remain historical. Close keeps the hold and performs no work.",
        record.final_power().map_or_else(
            |_| "unavailable; Refresh before continuing".into(),
            |power| power_name(&power).into_string()
        ),
    )
}

pub(super) fn guest_progress(operation: Kind, name: &str) -> String {
    let purpose = match operation.guest_purpose() {
        GuestPurpose::Apply => hyper_gpu_support::ui_text::operations::apply::guest_progress(),
        GuestPurpose::Verify => hyper_gpu_support::ui_text::operations::verify::guest_progress(),
        GuestPurpose::Reconcile => {
            hyper_gpu_support::ui_text::operations::recovery::guest_progress()
        }
        GuestPurpose::Credentials => {
            hyper_gpu_support::ui_text::operations::credentials::guest_progress()
        }
        GuestPurpose::Reviewed => hyper_gpu_support::ui_text::operations::guest::generic_progress(),
    };
    hyper_gpu_support::ui_text::operations::guest::progress_body(name, &purpose).into_string()
}

pub(super) fn credential_cancellation(recovery: bool) -> &'static str {
    if recovery {
        hyper_gpu_support::ui_text::operations::credentials::CANCELLED_RECOVERY
    } else {
        hyper_gpu_support::ui_text::operations::credentials::CANCELLED_ACTION
    }
}
