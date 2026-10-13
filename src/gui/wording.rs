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
    format!(
        "Approve reviewed power changes; expected final state if confirmed: {}.",
        power_name(final_power)
    )
}

pub(super) fn verify_review(plan: &VerificationPlan) -> String {
    let lifecycle = match plan.observed.power {
        Power::Off => {
            "Temporarily start this VM for the check, then gracefully shut it down to Off."
        }
        Power::Running => "This VM is already running and is expected to remain Running.",
        Power::Other(_) => {
            "The VM's power state is unknown or changing. Refresh before a new review."
        }
    };
    format!(
        "Virtual machine: {}\nPhysical GPU: {}\n\nVerify GPU setup: compare the current guest driver files and setup record with the approved driver package, then check the GPU's device health and hardware rendering. This checks current setup; a past successful check alone does not establish current health.\n\n{lifecycle}\nExpected final power after successful completion: {}.\n\nUse the Windows administrator sign-in for this VM if prompted. File checks, starting Windows and rendering can take time; no exact duration is promised. This refreshes the check tools and diagnostic information inside the VM, but does not reinstall drivers, change GPU assignment or save settings.\n\nVerify GPU setup confirms this review. Cancel keeps your settings and starts no check.",
        plan.observed.name,
        plan.gpu.name,
        power_name(&plan.observed.power)
    )
}

pub(super) fn verify_details(plan: &VerificationPlan) -> String {
    format!(
        "VM ID: {}\nGPU interface: {}\nDriver version: {}\nAuthenticated payload digest: {}\nObserved power: {:?}\n\nCompares current guest destinations and the preparation receipt with the authenticated host manifest before device health/rendering. Protected guest helpers, manifest metadata and optional diagnostics may be refreshed. Fresh protected admission independently checks the reviewed state.",
        plan.target.vm_id,
        plan.target.gpu_interface,
        plan.gpu.driver_version,
        plan.payload_digest,
        plan.observed.power
    )
}

pub(super) fn verify_consent(plan: &VerificationPlan) -> String {
    match plan.observed.power {
        Power::Off => {
            "Approve temporarily starting this VM, then gracefully shutting it down to Off.".into()
        }
        Power::Running => "This VM is expected to remain Running during Verify GPU setup.".into(),
        Power::Other(_) => {
            "Power state is unknown. Refresh and review again before continuing.".into()
        }
    }
}

pub(super) fn guest_progress(operation: Kind, name: &str) -> String {
    let purpose = match operation.guest_purpose() {
        GuestPurpose::Apply => {
            "Applying the changes you reviewed, including any required driver preparation and GPU setup checks. Successful completion leaves the VM off."
        }
        GuestPurpose::Verify => {
            "Checking the current driver files, setup record, device health and hardware rendering. Drivers are not being reinstalled. VM power follows the reviewed sequence."
        }
        GuestPurpose::Reconcile => {
            "Checking the interrupted operation and any reviewed guest/power steps. GPU assignment and driver preparation are not repeated. Uncertain results keep recovery blocked."
        }
        GuestPurpose::Credentials => {
            "Managing this VM's optional saved sign-in details. This does not connect to the VM, prepare drivers or check graphics."
        }
        GuestPurpose::Reviewed => {
            "Performing the guest steps shown in your review. Wait for the actual result before treating the operation as complete."
        }
    };
    format!(
        "Virtual machine: {name}\n\n{purpose}\n\nIf prompted, use the Windows administrator account inside this VM. Remembering it is optional. Progress appears below."
    )
}

pub(super) fn credential_cancellation(recovery: bool) -> &'static str {
    if recovery {
        "No recovery action was started. The unfinished operation and its recovery hold remain. Your pending edits are kept. Review recovery again when ready."
    } else {
        "No guest operation was started. Your pending edits are kept; this does not mean setup or verification completed. Review the action again when ready."
    }
}
