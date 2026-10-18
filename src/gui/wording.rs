//! User-facing operation wording derived from reviewed facts, never inferred health.
use super::operation::{GuestPurpose, Kind};
use hyper_gpu_support::{model::Power, workflow::VerificationPlan};

fn power_name(power: &Power) -> hyper_gpu_support::ui_text::UiMessage {
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
    let action = if restored {
        "Independently checking the recorded restoration. This does not retry the guest check or report success; changed facts retain recovery."
    } else if record.requires_graphics() {
        "Checking current guest files, devices and hardware rendering after fresh host checks. The selected action includes its required VM power transitions."
    } else {
        "Independently checking the recorded host outcome without guest sign-in or replaying Apply."
    };
    format!(
        "{name}: {action}\nRecorded final power: {}. Uncertain outcomes retain their recovery hold.",
        record.final_power().map_or_else(
            |_| "unavailable".into(),
            |power| power_name(&power).into_string()
        )
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
