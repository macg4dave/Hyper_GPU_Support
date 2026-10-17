//! Protected reads and planning; availability observations never grant authority.
use crate::gui::live_state::OperationReadback;
#[cfg(windows)]
use hyper_gpu_support::observation::Prepared;
use hyper_gpu_support::observation::{Bundle, Mode};
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
    ) -> Result<Vec<hyper_gpu_support::configuration_store::SaveRecord>, String> {
        #[cfg(windows)]
        {
            hyper_gpu_support::runner::recovery_records()
        }
        #[cfg(not(windows))]
        {
            Ok(Vec::new())
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
        discover().map(remember_os)
    }
    pub(super) fn plan(
        &self,
        target: Target,
        reinstall: bool,
        authenticate: bool,
    ) -> Result<(Plan, bool), String> {
        plan(target, reinstall, authenticate)
    }
    pub(super) fn credentials(&self, id: &str, name: &str) -> Result<bool, String> {
        credential_dialog(id, name)
    }

    pub(super) fn observe(
        &self,
        mode: Mode,
        target: Option<Target>,
    ) -> Result<Option<Bundle>, String> {
        #[cfg(windows)]
        {
            use hyper_gpu_support::runner::{self, Operation};
            if runner::pending_setup()?.is_some() || !runner::installed()? {
                if mode == Mode::Full {
                    return Ok(None);
                }
                return Err("Setup requires attention. Use Refresh to continue.".into());
            }
            let operation = match mode {
                Mode::Full => Operation::Observe,
                Mode::Poll => Operation::Poll,
                Mode::Completion => Operation::Completion,
            };
            let expected = target.clone();
            let value = match runner::submit(runner::request(operation, target, None)) {
                Ok(value) => value,
                Err(_)
                    if mode == Mode::Full
                        && hyper_gpu_support::read_context::tracked_backend_released()
                        && hyper_gpu_support::process::is_elevated()? =>
                {
                    return Ok(None);
                }
                Err(error) => return Err(error),
            };
            let mut bundle: Bundle = serde_json::from_value(value)
                .map_err(|e| format!("Invalid protected observation: {e}"))?;
            bundle.validate()?;
            if bundle.provenance.mode != mode || bundle.provenance.selected != expected {
                return Err("Protected observation mode differs".into());
            }
            if mode == Mode::Full {
                bundle.inventory = remember_os((bundle.inventory, None)).0;
            }
            Ok(Some(bundle))
        }
        #[cfg(not(windows))]
        {
            let _ = (mode, target);
            Ok(None)
        }
    }

    pub(super) fn operation_readback(&self, target: Target) -> Result<OperationReadback, String> {
        let bundle = self
            .observe(Mode::Completion, Some(target))?
            .ok_or("Protected completion unavailable")?;
        Ok(OperationReadback {
            committed: bundle.committed.clone(),
            recovery: bundle.recovery.clone(),
            observation: Some(bundle),
        })
    }
}

fn remember_os(result: InventoryRead) -> InventoryRead {
    #[cfg(windows)]
    {
        let (mut inventory, mut warning) = result;
        if let Err(error) = crate::gui::os_metadata::remember(&mut inventory) {
            let message = format!("Could not save VM icon metadata: {error}");
            warning = Some(warning.map_or_else(|| message.clone(), |w| format!("{w}\n{message}")));
        }
        (inventory, warning)
    }
    #[cfg(not(windows))]
    {
        result
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
        Err(error)
            if hyper_gpu_support::read_context::tracked_backend_released()
                && process::is_elevated()? =>
        {
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

#[cfg(windows)]
fn plan(target: Target, reinstall: bool, authenticate: bool) -> Result<(Plan, bool), String> {
    use hyper_gpu_support::runner::{self, Operation};
    let operation = if reinstall {
        Operation::ReinstallPlan
    } else if authenticate {
        Operation::Plan
    } else {
        Operation::PreliminaryPlan
    };
    let mut request = runner::request(operation, Some(target), None);
    request.include_observation = true;
    let value = runner::submit(request)?;
    let prepared: Prepared<Plan> =
        serde_json::from_value(value).map_err(|e| format!("Invalid plan reply: {e}"))?;
    Ok((prepared.value, prepared.worker_available))
}
#[cfg(not(windows))]
fn plan(_: Target, _: bool, _: bool) -> Result<(Plan, bool), String> {
    Err("Planning requires Windows x64".into())
}

#[cfg(windows)]
fn credential_dialog(id: &str, name: &str) -> Result<bool, String> {
    hyper_gpu_support::credentials::manage_named(
        id,
        name,
        windows::Win32::Foundation::HWND::default(),
    )
}
#[cfg(not(windows))]
fn credential_dialog(_: &str, _: &str) -> Result<bool, String> {
    Err("Guest credentials require Windows".into())
}
