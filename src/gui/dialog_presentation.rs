//! Display-only workflow headers. Neither translated text nor UI phase state selects effects.
use super::{AppWindow, model, operation::Kind};
use hyper_gpu_support::ui_text::{dialogs, progress};
use slint::Model;

pub(super) fn task(kind: Kind) -> &'static str {
    use dialogs::workflow as text;
    match kind {
        Kind::Read | Kind::StatusRefresh => text::READ,
        Kind::Plan => text::PLAN,
        Kind::PermissionPlan => text::PERMISSION_REVIEW,
        Kind::PermissionRecovery => text::PERMISSION_RECOVERY,
        Kind::VerifyPlan => text::VERIFY_PLAN,
        Kind::Apply => text::APPLY,
        Kind::Verify => text::VERIFY,
        Kind::Reconcile => text::RECOVER,
        Kind::SaveOnly => text::SAVE,
        Kind::Setup => text::PERMISSIONS,
        Kind::Install => text::INSTALL,
        Kind::Remove => text::REMOVE,
        Kind::Forget => text::DELETE,
        Kind::Credentials => text::CREDENTIALS,
    }
}

pub(super) fn begin(ui: &AppWindow, kind: Kind) {
    set(ui, &[task(kind)], 0);
}

fn set(ui: &AppWindow, phases: &[&str], active: usize) {
    ui.set_dialog_phases(model(phases.iter().map(|text| (*text).into()).collect()));
    ui.set_dialog_phase(i32::try_from(active).unwrap_or(0));
}

/// Called only after the task has returned, before independent protected readback.
pub(super) fn readback(ui: &AppWindow) {
    let mut phases: Vec<_> = ui.get_dialog_phases().iter().collect();
    phases.push(dialogs::workflow::READBACK.into());
    ui.set_dialog_phase(i32::try_from(phases.len() - 1).unwrap_or(0));
    ui.set_dialog_phases(model(phases));
}

pub(super) fn opened(ui: &AppWindow, kind: &str, continuing: bool) {
    if matches!(kind, "progress" | "install-progress") {
        return;
    }
    if continuing {
        let mut phases: Vec<_> = ui.get_dialog_phases().iter().collect();
        // Do not suggest future work after a failure or an unsaved outcome.
        let failed = matches!(kind, "error" | "operation-error" | "save");
        if failed {
            phases.truncate(usize::try_from(ui.get_dialog_phase() + 1).unwrap_or(0));
        }
        phases.push(
            if failed {
                progress::status::FAILED
            } else {
                dialogs::workflow::RESULT
            }
            .into(),
        );
        ui.set_dialog_phase(i32::try_from(phases.len() - 1).unwrap_or(0));
        ui.set_dialog_phases(model(phases));
    } else {
        // Notices, help and choices have no execution history. In particular,
        // credential cancellation must not retain Apply's stages or green result.
        set(ui, &[], 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operation_headers_follow_typed_work_and_do_not_leak_into_notices() {
        slint::platform::set_platform(Box::new(super::super::catalogue_tests::Headless)).unwrap();
        let ui = AppWindow::new().unwrap();
        for kind in [
            Kind::Read,
            Kind::Plan,
            Kind::PermissionPlan,
            Kind::PermissionRecovery,
            Kind::VerifyPlan,
            Kind::Apply,
            Kind::Verify,
            Kind::Reconcile,
            Kind::SaveOnly,
            Kind::Setup,
            Kind::Install,
            Kind::Remove,
            Kind::Forget,
            Kind::Credentials,
        ] {
            begin(&ui, kind);
            let labels: Vec<_> = ui.get_dialog_phases().iter().collect();
            assert_eq!(labels.last().unwrap(), task(kind));
            assert_eq!(
                labels.len(),
                usize::try_from(ui.get_dialog_phase()).unwrap() + 1
            );
            if kind != Kind::Setup {
                assert!(
                    !labels
                        .iter()
                        .any(|label| label == dialogs::workflow::PERMISSIONS)
                );
            }
            if kind != Kind::Apply {
                assert!(!labels.iter().any(|label| label == dialogs::workflow::APPLY));
            }
            readback(&ui);
            assert_eq!(
                ui.get_dialog_phases()
                    .row_data(ui.get_dialog_phase() as usize)
                    .unwrap(),
                dialogs::workflow::READBACK
            );
            opened(&ui, "operation-complete", true);
            assert_eq!(
                ui.get_dialog_phases()
                    .row_data(ui.get_dialog_phase() as usize)
                    .unwrap(),
                dialogs::workflow::RESULT
            );
            opened(&ui, "info", false);
            assert_eq!(ui.get_dialog_phases().row_count(), 0);
        }
        begin(&ui, Kind::Apply);
        opened(&ui, "operation-error", true);
        assert_eq!(
            ui.get_dialog_phases().iter().last().unwrap(),
            progress::status::FAILED
        );
        assert!(
            !ui.get_dialog_phases()
                .iter()
                .any(|label| label == dialogs::workflow::RESULT)
        );
    }
}
