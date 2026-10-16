//! Current OS facts inside the guest worker; no host Hyper-V or disk inspection.
use crate::guest_compatibility::{WindowsIdentity, require_supported, unknown};
use windows::{
    Win32::System::Registry::{HKEY_LOCAL_MACHINE, RRF_NOEXPAND, RRF_RT_REG_SZ, RegGetValueW},
    core::PCWSTR,
};

#[allow(unsafe_code)]
fn installation_type() -> Result<String, String> {
    let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let subkey = wide(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion");
    let name = wide("InstallationType");
    let mut data = [0u16; 128];
    let mut bytes = std::mem::size_of_val(&data) as u32;
    // SAFETY: borrowed predefined key, terminated names, bounded writable buffer.
    // Only a string value is read; no hive loading, privileges or writes.
    unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(subkey.as_ptr()),
            PCWSTR(name.as_ptr()),
            RRF_RT_REG_SZ | RRF_NOEXPAND,
            None,
            Some(data.as_mut_ptr().cast()),
            Some(&mut bytes),
        )
    }
    .ok()
    .map_err(|_| unknown())?;
    let end = data.iter().position(|c| *c == 0).ok_or_else(unknown)?;
    String::from_utf16(&data[..end]).map_err(|_| unknown())
}

pub(super) fn require_local() -> Result<(), String> {
    let observe = || {
        let session = crate::windows_wmi::Session::in_namespace(
            r"ROOT\cimv2",
            std::time::Duration::from_secs(30),
        )?;
        let os = session.one("SELECT * FROM Win32_OperatingSystem")?;
        WindowsIdentity::from_facts(
            &os.string("Version")?,
            &os.string("BuildNumber")?,
            os.number("ProductType")?,
            &installation_type()?,
        )
    };
    require_supported(&observe().map_err(|_: String| unknown())?)
}
