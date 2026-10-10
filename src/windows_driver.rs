//! Selected installed-driver discovery, independent of baseline pins.
use crate::payload::DriverDiscovery;
use crate::windows_com::{
    Apartment, DriverDiscoveryError, connect, native, property, query, quoted,
};
use std::{collections::BTreeMap, path::PathBuf, time::Instant};
use windows::{
    Win32::System::Wmi::{IWbemServices, WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_S_TIMEDOUT},
    core::{BSTR, HRESULT, Interface},
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
                deadline,
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
    deadline: Instant,
) -> Result<String, DriverDiscoveryError> {
    let mut trace = crate::diagnostics::Span::start("resolve-associated-file", path)
        .map_err(|_| DriverDiscoveryError::Invalid("diagnostic logging unavailable"))?;
    let result = (|| {
        lookup_wait_ms(deadline, Instant::now())?;
        let mut pending = None;
        // SAFETY: live services, provider-returned path and initialized output. WMI
        // resolves quoted/backslash-containing file references rather than our parser.
        unsafe {
            services.GetObject(
                &BSTR::from(path),
                WBEM_FLAG_RETURN_IMMEDIATELY,
                None,
                None,
                Some(&mut pending),
            )
        }
        .map_err(|error| native("resolve associated data file", error))?;
        let pending = pending.ok_or(DriverDiscoveryError::Invalid(
            "associated file call result absent",
        ))?;
        loop {
            let wait = lookup_wait_ms(deadline, Instant::now())?;
            let mut operation = 0;
            // SAFETY: live apartment-owned call result, writable HRESULT output.
            // Use the vtable to preserve WBEM_S_TIMEDOUT (a success HRESULT),
            // which the generated Result wrapper otherwise discards.
            let status =
                unsafe { (pending.vtable().GetCallStatus)(pending.as_raw(), wait, &mut operation) };
            if lookup_complete(status, HRESULT(operation))? {
                break;
            }
        }
        lookup_wait_ms(deadline, Instant::now())?;
        // SAFETY: GetCallStatus confirmed completion; zero means never wait for
        // the object. The generated interface owns/releases the returned object.
        let object = unsafe { pending.GetResultObject(0) }
            .map_err(|error| native("retrieve associated data file", error))?;
        property(&object, "Name")
    })();
    trace
        .finish(&result)
        .map_err(|_| DriverDiscoveryError::Invalid("diagnostic logging unavailable"))?;
    result
}

fn lookup_wait_ms(deadline: Instant, now: Instant) -> Result<i32, DriverDiscoveryError> {
    let remaining = deadline.saturating_duration_since(now);
    if remaining.is_zero() {
        return Err(DriverDiscoveryError::Timeout);
    }
    // One-second slices keep each provider wait within the discovery budget.
    // Sub-millisecond remainder polls rather than rounding beyond the deadline.
    Ok(remaining.as_millis().min(1000) as i32)
}

fn lookup_complete(status: HRESULT, operation: HRESULT) -> Result<bool, DriverDiscoveryError> {
    if status == HRESULT(WBEM_S_TIMEDOUT.0) {
        return Ok(false);
    }
    status
        .ok()
        .map_err(|error| native("wait for associated data file", error))?;
    if status != HRESULT(0) {
        return Err(DriverDiscoveryError::Invalid(
            "unexpected associated file completion status",
        ));
    }
    operation
        .ok()
        .map_err(|error| native("resolve associated data file", error))?;
    if operation != HRESULT(0) {
        return Err(DriverDiscoveryError::Invalid(
            "associated file operation did not complete",
        ));
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn lookup_wait_never_exceeds_remaining_discovery_budget() {
        let now = Instant::now();
        assert!(matches!(
            lookup_wait_ms(now, now),
            Err(DriverDiscoveryError::Timeout)
        ));
        assert!(lookup_wait_ms(now, now + Duration::from_secs(1)).is_err());
        assert_eq!(
            lookup_wait_ms(now + Duration::from_secs(10), now).unwrap(),
            1000
        );
        assert_eq!(
            lookup_wait_ms(now + Duration::from_millis(35), now).unwrap(),
            35
        );
        assert_eq!(
            lookup_wait_ms(now + Duration::from_micros(50), now).unwrap(),
            0
        );
    }

    #[test]
    fn lookup_timeout_is_pending_and_native_failures_are_preserved() {
        let failed = HRESULT(0x80041002_u32 as i32);
        assert!(!lookup_complete(HRESULT(WBEM_S_TIMEDOUT.0), failed).unwrap());
        assert!(lookup_complete(HRESULT(0), HRESULT(0)).unwrap());
        for (status, operation) in [(failed, HRESULT(0)), (HRESULT(0), failed)] {
            match lookup_complete(status, operation).unwrap_err() {
                DriverDiscoveryError::Native { source, .. } => assert_eq!(source.code(), failed),
                other => panic!("native error was lost: {other}"),
            }
        }
        assert!(lookup_complete(HRESULT(1), HRESULT(0)).is_err());
        assert!(lookup_complete(HRESULT(0), HRESULT(1)).is_err());
    }
}
