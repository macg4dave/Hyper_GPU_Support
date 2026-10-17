//! Connections belong to one operation; provider objects are always fetched anew.
use super::*;
use windows::Win32::System::Wmi::IWbemServices;

pub(crate) struct ReadSession {
    cim: Option<IWbemServices>,
    hyperv: Session,
}
impl ReadSession {
    pub(crate) fn new() -> Result<Self, String> {
        Ok(Self {
            hyperv: Session::new(TIMEOUT)?,
            cim: None,
        })
    }
    fn begin(&mut self) -> Result<(), String> {
        self.hyperv.renew_deadline(TIMEOUT)
    }
    fn driver_connection(&mut self) -> Result<IWbemServices, String> {
        if self.cim.is_none() {
            self.cim = Some(crate::windows_com::connect(r"ROOT\cimv2").map_err(|e| e.to_string())?);
        }
        self.cim
            .clone()
            .ok_or_else(|| "driver connection unavailable".into())
    }
    pub(crate) fn inspect(&mut self, id: &str) -> Result<VmState, String> {
        self.begin()?;
        inspect_in(&self.hyperv, id)
    }
    pub(crate) fn gpu(&mut self, target: &Target) -> Result<Gpu, String> {
        target.validate()?;
        self.begin()?;
        let cim = self.driver_connection()?;
        let objects = self.hyperv.query(&format!(
            "SELECT * FROM Msvm_PartitionableGpu WHERE Name='{}'",
            quoted(&target.gpu_interface)
        ))?;
        let mut gpus = read_gpus(objects, &cim, false, None)?;
        if gpus.len() != 1 {
            return Err("selected GPU is absent or ambiguous".into());
        }
        gpus.pop().ok_or_else(|| "selected GPU disappeared".into())
    }
    pub(crate) fn discover(&mut self, display: bool) -> Result<Discovery, String> {
        self.begin()?;
        let cim = self.driver_connection()?;
        inventory::discover(&self.hyperv, &cim, display)
    }
}
