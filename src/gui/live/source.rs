//! Protected reads and planning; availability observations never grant authority.
use crate::gui::live_state::OperationReadback;
use hyper_gpu_support::{gui_model::Inventory, model::Target, workflow::Plan};

type InventoryRead = (Inventory, Option<String>);

// Artifact authentication hashes installed files; call only inside background jobs.
// This availability observation never substitutes for the worker's fresh checks.
pub(super) fn protected_worker_available() -> bool {
    #[cfg(windows)]
    {
        hyper_gpu_support::worker::available().is_ok()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

#[derive(Clone)]
pub(in crate::gui) struct Source;

impl Source {
    pub(super) fn enrollment_recovery(
        &self,
    ) -> Result<Option<hyper_gpu_support::enrollment::PendingPair>, String> {
        #[cfg(windows)]
        {
            hyper_gpu_support::runner::pending_pair()
        }
        #[cfg(not(windows))]
        {
            Ok(None)
        }
    }
    pub(super) fn import_recovery(
        &self,
    ) -> Result<Option<hyper_gpu_support::configuration_store::ImportRecord>, String> {
        #[cfg(windows)]
        {
            hyper_gpu_support::runner::import_record()
        }
        #[cfg(not(windows))]
        {
            Ok(None)
        }
    }
    pub(super) fn recovery(
        &self,
    ) -> Result<Option<hyper_gpu_support::configuration_store::SaveRecord>, String> {
        #[cfg(windows)]
        {
            hyper_gpu_support::runner::recovery_record()
        }
        #[cfg(not(windows))]
        {
            Ok(None)
        }
    }
    pub(super) fn committed(
        &self,
    ) -> Option<hyper_gpu_support::configuration_store::StoreSnapshot> {
        #[cfg(windows)]
        {
            Some(hyper_gpu_support::configuration_store::read_committed())
        }
        #[cfg(not(windows))]
        {
            None
        }
    }
    pub(super) fn discover(&self) -> Result<InventoryRead, String> {
        discover()
    }
    pub(super) fn plan(
        &self,
        target: Target,
        reinstall: bool,
        authenticate: bool,
    ) -> Result<Plan, String> {
        plan(target, reinstall, authenticate)
    }
    pub(super) fn credentials(&self, id: &str, name: &str) -> Result<bool, String> {
        credential_dialog(id, name)
    }

    pub(super) fn operation_readback(&self) -> Result<OperationReadback, String> {
        Ok(OperationReadback {
            committed: self
                .committed()
                .ok_or("Saved configuration requires Windows")?,
            recovery: self.recovery()?,
        })
    }
}

#[cfg(windows)]
fn discover() -> Result<(Inventory, Option<String>), String> {
    use hyper_gpu_support::{process, runner, windows_hyperv};
    if runner::pending_setup()?.is_some() || !runner::installed()? {
        return hyper_gpu_support::setup::discover().map(|inventory| {
            (
                inventory,
                Some(hyper_gpu_support::ui_text::system::discovery::INITIAL_SETUP_NOTICE.into()),
            )
        });
    }
    match runner::submit(runner::request(runner::Operation::Discover, None, None)) {
        Ok(value) => serde_json::from_value(value)
            .map(|i| (i, None))
            .map_err(|e| format!("Invalid protected inventory reply: {e}")),
        Err(error) if process::is_elevated()? => {
            // Preserve standalone elevated discovery; attachment never grants enrollment.
            Ok((
                Inventory {
                    discovery: windows_hyperv::discover()?,
                    managed: Default::default(),
                    enrolled: vec![],
                },
                Some(format!(
                    "{}. Native inventory only; editing is blocked.",
                    hyper_gpu_support::reporting::operator_error(
                        hyper_gpu_support::ui_text::system::discovery::PERMISSION_STAGE,
                        &error
                    )
                )),
            ))
        }
        Err(error) => Err(error),
    }
}
#[cfg(not(windows))]
fn discover() -> Result<(Inventory, Option<String>), String> {
    Err("Hyper-V discovery requires Windows x64".into())
}

/// Poll only installed inventory; never initiate first-install elevation.
#[cfg(windows)]
pub(super) fn discover_status() -> Result<InventoryRead, String> {
    use hyper_gpu_support::runner;
    if runner::pending_setup()?.is_some() || !runner::installed()? {
        return Err("Setup requires attention. Use Refresh to continue.".into());
    }
    let value = runner::submit(runner::request(runner::Operation::Discover, None, None))?;
    serde_json::from_value(value)
        .map(|inventory| (inventory, None))
        .map_err(|e| format!("Invalid protected inventory reply: {e}"))
}

#[cfg(not(windows))]
pub(super) fn discover_status() -> Result<InventoryRead, String> {
    Err("Hyper-V discovery requires Windows x64".into())
}

#[cfg(windows)]
fn plan(target: Target, reinstall: bool, authenticate: bool) -> Result<Plan, String> {
    use hyper_gpu_support::runner::{self, Operation};
    let operation = if reinstall {
        Operation::ReinstallPlan
    } else if authenticate {
        Operation::Plan
    } else {
        Operation::PreliminaryPlan
    };
    let value = runner::submit(runner::request(operation, Some(target), None))?;
    serde_json::from_value(value).map_err(|e| format!("Invalid plan reply: {e}"))
}
#[cfg(not(windows))]
fn plan(_: Target, _: bool, _: bool) -> Result<Plan, String> {
    Err("Planning requires Windows x64".into())
}

#[cfg(windows)]
fn credential_dialog(id: &str, name: &str) -> Result<bool, String> {
    hyper_gpu_support::credentials::prompt_named(
        id,
        name,
        windows::Win32::Foundation::HWND::default(),
    )?;
    // This settings action has no guest operation to consume ephemeral input.
    // Apply/Verify prompt separately; confirm actual opt-in vault storage here.
    Ok(hyper_gpu_support::credentials::read(id)?.is_some())
}
#[cfg(not(windows))]
fn credential_dialog(_: &str, _: &str) -> Result<bool, String> {
    Err("Guest credentials require Windows".into())
}
