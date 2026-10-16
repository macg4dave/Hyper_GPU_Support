//! Native workflow adapter for Hyper-V, payloads, managed journals and guest operations.
use super::storage::atomic_json;
use crate::{
    credentials::Credential,
    model::{Allocation, Gpu, Power, Resource, Settings, Target, VmState},
    payload, process, windows_hyperv,
    workflow::{Backend, Journal},
};
use std::{fs, io::Read, path::PathBuf, time::Duration};

pub(crate) struct NativeBackend {
    pub(crate) credential: Option<Credential>,
    pub(crate) manifest: Option<payload::Manifest>,
    pub(crate) data: PathBuf,
}
impl Backend for NativeBackend {
    fn check_installation_os(&mut self, t: &Target) -> Result<(), String> {
        match windows_hyperv::inspect(&t.vm_id)?.power {
            Power::Running => crate::guest::check_installation_os(
                t,
                self.credential
                    .as_ref()
                    .ok_or_else(crate::guest_compatibility::unknown)?,
            ),
            _ => Err(crate::guest_compatibility::unknown()),
        }
    }
    fn inspect(&mut self, t: &Target) -> Result<VmState, String> {
        crate::diagnostics::run("inspect-vm", &t.vm_id, || windows_hyperv::inspect(&t.vm_id))
    }
    fn gpu(&mut self, t: &Target) -> Result<Gpu, String> {
        crate::diagnostics::run("selected-gpu", &t.vm_id, || windows_hyperv::selected_gpu(t))
    }
    fn payload(&mut self, t: &Target) -> Result<String, String> {
        crate::diagnostics::run("payload-processing", &t.vm_id, || {
            let manifest = process::background_work(|| {
                let discovery =
                    crate::windows_driver::discover_driver_environment(t, Duration::from_secs(300))
                        .map_err(|e| e.to_string())?;
                let manifest = payload::discover(
                    &crate::windows_paths::windows_directory().map_err(|e| e.to_string())?,
                    discovery,
                )?;
                crate::guest::validate_trust(&manifest)?;
                Ok(manifest)
            })?;
            let digest = manifest.digest()?;
            self.manifest = Some(manifest);
            Ok(digest)
        })
    }
    fn journal(&mut self, t: &Target) -> Result<Option<Journal>, String> {
        let path = self
            .data
            .join(format!("{}.json", t.vm_id.to_ascii_lowercase()));
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.to_string()),
            Ok(metadata) if !metadata.is_file() => {
                return Err("journal is not a regular file".into());
            }
            Ok(_) => {}
        }
        crate::security::verify(&path)?;
        let mut b = Vec::new();
        fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(65537)
            .read_to_end(&mut b)
            .map_err(|e| e.to_string())?;
        if b.len() > 65536 {
            return Err("invalid journal size".into());
        }
        serde_json::from_slice(&b)
            .map(Some)
            .map_err(|e| e.to_string())
    }
    fn save(&mut self, t: &Target, j: &Journal) -> Result<(), String> {
        atomic_json(
            &self
                .data
                .join(format!("{}.json", t.vm_id.to_ascii_lowercase())),
            j,
        )
    }
    fn power(&mut self, t: &Target, p: Power) -> Result<(), String> {
        windows_hyperv::power(t, p)
    }
    fn assign(&mut self, t: &Target, e: bool) -> Result<(), String> {
        windows_hyperv::assign(t, e)
    }
    fn settings(&mut self, t: &Target, s: &Settings) -> Result<(), String> {
        windows_hyperv::configure(t, s)
    }
    fn allocation(&mut self, t: &Target, resource: Resource, v: &Allocation) -> Result<(), String> {
        windows_hyperv::allocation(t, resource, v)
    }
    fn prepare(&mut self, t: &Target) -> Result<String, String> {
        crate::guest::prepare(
            t,
            self.manifest
                .as_ref()
                .ok_or("payload must be discovered before preparation")?,
            self.credential
                .as_ref()
                .ok_or("guest credentials required")?,
        )
    }
    fn verify(&mut self, t: &Target, g: &Gpu) -> Result<(), String> {
        if self.manifest.is_none() {
            self.payload(t)?;
        }
        crate::guest::verify(
            t,
            g,
            self.manifest
                .as_ref()
                .ok_or("current manifest unavailable for verification")?,
            self.credential
                .as_ref()
                .ok_or("guest credentials required")?,
        )
    }
    fn verify_with_progress(
        &mut self,
        t: &Target,
        g: &Gpu,
        progress: &mut dyn FnMut(crate::workflow::PreparationProgress) -> Result<(), String>,
    ) -> Result<(), String> {
        if self.manifest.is_none() {
            self.payload(t)?;
        }
        crate::guest::verify_with_progress(
            t,
            g,
            self.manifest
                .as_ref()
                .ok_or("current manifest unavailable for verification")?,
            self.credential
                .as_ref()
                .ok_or("guest credentials required")?,
            progress,
        )
    }
    fn prepare_with_progress(
        &mut self,
        t: &Target,
        progress: &mut dyn FnMut(crate::workflow::PreparationProgress) -> Result<(), String>,
    ) -> Result<String, String> {
        crate::guest::prepare_with_progress(
            t,
            self.manifest
                .as_ref()
                .ok_or("payload must be discovered before preparation")?,
            self.credential
                .as_ref()
                .ok_or("guest credentials required")?,
            progress,
        )
    }
}
