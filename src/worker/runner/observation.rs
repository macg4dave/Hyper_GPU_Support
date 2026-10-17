//! One operation-locked presentation observation; no reusable execution attestation.
use super::{
    backend::NativeBackend,
    policy::{Enrollment, enrollment_revision},
};
use crate::{
    configuration_store::{self, Revision},
    gui_model::Inventory,
    model::{Discovery, Target},
    observation::{Bundle, Mode, Provenance},
    workflow::Backend,
};
use std::path::PathBuf;

pub(super) fn read(
    e: &Enrollment,
    data: PathBuf,
    mode: Mode,
    selected: Option<Target>,
) -> Result<Bundle, String> {
    crate::read_context::check()?;
    let discovery = if mode == Mode::Completion {
        let target = selected.as_ref().ok_or("Completion target missing")?;
        target.validate()?;
        let mut reader = crate::windows_hyperv::ReadSession::new()?;
        let (vms, mut issues) = crate::reporting::collect_observations(
            "Completion VM",
            vec![(target.vm_id.clone(), reader.inspect(&target.vm_id))],
        );
        let (gpus, gpu_issues) = crate::reporting::collect_observations(
            "Completion GPU",
            vec![(target.gpu_interface.clone(), reader.gpu(target))],
        );
        issues.extend(gpu_issues);
        Discovery {
            vms,
            gpus,
            issues,
            guest_os: Default::default(),
        }
    } else {
        crate::windows_hyperv::ReadSession::new()?.discover(mode == Mode::Full)?
    };
    let mut inventory = Inventory {
        discovery,
        managed: Default::default(),
        enrolled: e.targets.clone(),
    };
    let mut reader = NativeBackend {
        reads: None,
        credential: None,
        manifest: None,
        payload_hold: Vec::new(),
        data,
    };
    for target in &e.targets {
        crate::read_context::check()?;
        match reader.journal(target) {
            Ok(journal) => {
                inventory.managed.insert(target.vm_id.clone(), journal);
            }
            Err(error) => {
                inventory
                    .discovery
                    .issues
                    .push(crate::reporting::Diagnostic::observation(
                        "Preparation record",
                        Some(target.vm_id.clone()),
                        &error,
                    ))
            }
        }
    }
    crate::read_context::check()?;
    let committed = configuration_store::read_committed();
    let revision = Revision::of(&serde_json::to_vec(&committed).map_err(|e| e.to_string())?);
    let result = Bundle {
        inventory,
        committed,
        recovery: super::recovery_records()?,
        import: super::import_record()?,
        enrollment: super::pending_pair()?,
        worker_available: e.artifacts.contains_key("hyper-gpu-support.exe"),
        provenance: Provenance {
            schema: 1,
            authority: enrollment_revision(e)?,
            configuration: revision,
            mode,
            selected,
        },
    };
    result.validate()?;
    crate::read_context::check()?;
    Ok(result)
}
