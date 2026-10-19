//! User-facing operation wording derived from reviewed facts, never inferred health.
use super::operation::{GuestPurpose, Kind};
use hyper_gpu_support::{model::Power, workflow::VerificationPlan};

fn power_name(power: &Power) -> &'static str {
    hyper_gpu_support::ui_text::power(power)
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

pub(super) fn recovery_progress(
    record: &hyper_gpu_support::configuration_store::SaveRecord,
    name: &str,
    restored: bool,
) -> String {
    use hyper_gpu_support::ui_text::operations::recovery::progress;
    let action = if restored {
        progress::restoration()
    } else if record.requires_graphics() {
        progress::graphics()
    } else {
        progress::host()
    };
    progress::body(
        name,
        action,
        record.final_power().map_or_else(
            |_| hyper_gpu_support::ui_text::common::state::unavailable_lower(),
            |power| power_name(&power),
        ),
    )
    .into_string()
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
    hyper_gpu_support::ui_text::operations::guest::progress_body(name, purpose).into_string()
}

pub(super) fn credential_cancellation(recovery: bool) -> &'static str {
    if recovery {
        hyper_gpu_support::ui_text::operations::credentials::cancelled_recovery()
    } else {
        hyper_gpu_support::ui_text::operations::credentials::cancelled_action()
    }
}
