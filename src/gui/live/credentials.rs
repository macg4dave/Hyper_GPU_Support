//! Local credential settings requests and consistent prompt-cancellation presentation.
use super::{SharedState, background::background_for_vm, source::Source};
#[cfg(windows)]
use crate::gui::wording;
use crate::gui::{
    AppWindow, dialog,
    operation::{Kind, Operation},
};
use std::sync::{Arc, atomic::AtomicBool};

pub(super) fn credential_request_cancelled(ui: &AppWindow, error: &str, recovery: bool) -> bool {
    #[cfg(windows)]
    if hyper_gpu_support::credentials::is_prompt_cancelled(error) {
        ui.set_operation_dialog_active(false);
        dialog(
            ui,
            "info",
            hyper_gpu_support::ui_text::operations::credentials::CANCELLED_TITLE,
            wording::credential_cancellation(recovery),
        );
        return true;
    }
    #[cfg(not(windows))]
    let _ = (ui, error, recovery);
    false
}

pub(super) fn review_credentials(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
) -> Result<(), String> {
    let selected = ui.get_selected_id().to_string();
    let vm_name = ui.get_vm_name().to_string();
    let prompt_name = vm_name.clone();
    let credential_source = source.clone();
    state.lock().map_err(|_| "Live presentation state unavailable".to_owned()).and_then(|s| s.target(&selected))
        .and_then(|target| background_for_vm(ui, busy, Operation::new(Kind::Credentials, hyper_gpu_support::ui_text::credentials::settings::TITLE), Some(&vm_name), true, move || credential_source.credentials(&target.vm_id, &prompt_name), |ui, result| match result {
            Ok(true) => dialog(ui, "info", hyper_gpu_support::ui_text::operations::credentials::stored::TITLE, hyper_gpu_support::ui_text::operations::credentials::stored::BODY),
            Ok(false) => dialog(ui, "info", hyper_gpu_support::ui_text::operations::credentials::absent::TITLE, hyper_gpu_support::ui_text::operations::credentials::absent::BODY),
            Err(error) => {
                if !credential_request_cancelled(ui, &error, false) {
                    dialog(ui, "error", hyper_gpu_support::ui_text::operations::credentials::UNAVAILABLE_TITLE, &error);
                }
            },
        }))
}
