//! Selected installed-driver discovery, independent of baseline pins.
use crate::payload::DriverDiscovery;
use crate::windows_com::{
    Apartment, DriverDiscoveryError, connect, native, property, query, quoted,
};
use std::{collections::BTreeMap, path::PathBuf, time::Instant};
use windows::{
    Win32::System::Wmi::{IWbemServices, WBEM_GENERIC_FLAG_TYPE},
    core::BSTR,
};
fn exactly_one(
    mut rows: Vec<BTreeMap<String, String>>,
) -> Result<BTreeMap<String, String>, DriverDiscoveryError> {
    if rows.len() != 1 {
        return Err(DriverDiscoveryError::Invalid(
            "selected WMI object is absent or ambiguous",
        ));
    }
    rows.pop().ok_or(DriverDiscoveryError::Invalid(
        "selected WMI object disappeared",
    ))
}
fn take(row: &mut BTreeMap<String, String>, key: &str) -> Result<String, DriverDiscoveryError> {
    row.remove(key)
        .filter(|value| !value.is_empty())
        .ok_or(DriverDiscoveryError::Invalid(
            "required WMI fact is missing",
        ))
}

/// Discover the exact partitionable GPU, PnP service, signed driver and all its
/// associated data files through native WMI, without a PowerShell subprocess.
///
/// # Errors
/// Preserves COM/WMI HRESULTs and rejects ambiguity, failed enumeration and identity
/// changes. Normal-account denial in root/virtualization/v2 requires the approved
/// elevated read-only development path, not a different GPU selection.
pub fn discover_driver_environment(
    target: &crate::model::Target,
    timeout: std::time::Duration,
) -> Result<DriverDiscovery, DriverDiscoveryError> {
    let mut trace = crate::diagnostics::Span::start("driver-discovery", &target.vm_id)
        .map_err(|_| DriverDiscoveryError::Invalid("diagnostic logging unavailable"))?;
    let result = (|| {
        let device_id = crate::payload::physical_device_id(&target.gpu_interface)
            .map_err(|_| DriverDiscoveryError::Invalid("configured interface is malformed"))?;
        let _apartment = Apartment::initialize()?;
        let virtualization = connect(r"ROOT\virtualization\v2")?;
        let deadline = Instant::now() + timeout;
        let gpu_query = format!(
            "SELECT Name FROM Msvm_PartitionableGpu WHERE Name='{}'",
            quoted(&target.gpu_interface)
        );
        let mut gpu = exactly_one(query(&virtualization, &gpu_query, &["Name"], deadline)?)?;
        let gpu_interface = take(&mut gpu, "Name")?;
        let cim = connect(r"ROOT\cimv2")?;
        let pnp_query = format!(
            "SELECT Service FROM Win32_PnPEntity WHERE DeviceID='{}'",
            quoted(&device_id)
        );
        let mut pnp = exactly_one(query(&cim, &pnp_query, &["Service"], deadline)?)?;
        let service = take(&mut pnp, "Service")?;
        let signed_query = format!(
            "SELECT DeviceName,DriverVersion,InfName FROM Win32_PNPSignedDriver WHERE DeviceID='{}' AND IsSigned=TRUE",
            quoted(&device_id)
        );
        let mut signed = exactly_one(query(
            &cim,
            &signed_query,
            &["DeviceName", "DriverVersion", "InfName"],
            deadline,
        )?)?;
        let service_query = format!(
            "SELECT PathName FROM Win32_SystemDriver WHERE Name='{}'",
            quoted(&service)
        );
        let mut kernel = exactly_one(query(&cim, &service_query, &["PathName"], deadline)?)?;
        // Win32_PNPSignedDriver reports a null Name key and __PATH. ASSOCIATORS
        // rejects a DeviceID key path; preserve the provider's association Antecedent
        // representation and resolve its returned file references through WMI.
        let mut computer = exactly_one(query(
            &cim,
            "SELECT Name FROM Win32_ComputerSystem",
            &["Name"],
            deadline,
        )?)?;
        let antecedent = signed_driver_antecedent(&take(&mut computer, "Name")?, &device_id);
        let association_query = associated_file_query(&antecedent);
        let mut associated_files = Vec::new();
        for mut file in query(&cim, &association_query, &["Dependent"], deadline)? {
            if Instant::now() >= deadline {
                return Err(DriverDiscoveryError::Timeout);
            }
            associated_files.push(PathBuf::from(dependent_file_name(
                &cim,
                &take(&mut file, "Dependent")?,
            )?));
        }
        Ok(DriverDiscovery {
            gpu_interface,
            device_id,
            name: take(&mut signed, "DeviceName")?,
            version: take(&mut signed, "DriverVersion")?,
            inf_name: take(&mut signed, "InfName")?,
            service,
            service_binary: PathBuf::from(take(&mut kernel, "PathName")?.trim_matches('"')),
            associated_files,
        })
    })();
    trace
        .finish(&result)
        .map_err(|_| DriverDiscoveryError::Invalid("diagnostic logging unavailable"))?;
    result
}

fn signed_driver_antecedent(hostname: &str, device_id: &str) -> String {
    let escaped = device_id.replace('\\', "\\\\").replace('"', "\\\"");
    format!(r#"\\{hostname}\ROOT\cimv2:Win32_PNPSignedDriver.DeviceID="{escaped}""#)
}

fn associated_file_query(antecedent: &str) -> String {
    format!(
        "SELECT Dependent FROM Win32_PNPSignedDriverCIMDataFile WHERE Antecedent='{}'",
        quoted(antecedent)
    )
}

#[allow(unsafe_code)]
fn dependent_file_name(
    services: &IWbemServices,
    path: &str,
) -> Result<String, DriverDiscoveryError> {
    let mut trace = crate::diagnostics::Span::start("resolve-associated-file", path)
        .map_err(|_| DriverDiscoveryError::Invalid("diagnostic logging unavailable"))?;
    let result = (|| {
        let mut object = None;
        // SAFETY: live services, provider-returned path and initialized output. WMI
        // resolves quoted/backslash-containing file references rather than our parser.
        unsafe {
            services.GetObject(
                &BSTR::from(path),
                WBEM_GENERIC_FLAG_TYPE(0),
                None,
                Some(&mut object),
                None,
            )
        }
        .map_err(|error| native("resolve associated data file", error))?;
        property(
            &object.ok_or(DriverDiscoveryError::Invalid(
                "associated file object absent",
            ))?,
            "Name",
        )
    })();
    trace
        .finish(&result)
        .map_err(|_| DriverDiscoveryError::Invalid("diagnostic logging unavailable"))?;
    result
}
