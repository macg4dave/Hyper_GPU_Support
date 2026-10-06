//! Native COM/WMI discovery for the validated driver/runtime association closure.
//!
//! Queries are read-only and fixed to the configured physical GPU. This adapter
//! requires Hyper-V read access for partitionability verification; invoke it via
//! the approved elevated development path when the normal token lacks that access.
//! Enumeration waits are bounded. Local COM connection/query setup is synchronous
//! and not cancellable here; ConnectServer uses Microsoft's maximum-wait option.

use crate::{
    config::ProjectConfiguration,
    driver_environment::{DriverDiscovery, physical_device_id},
};
use std::{collections::BTreeMap, fmt, path::PathBuf, time::Instant};
use windows::{
    Win32::System::{
        Com::{
            CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
            CoSetProxyBlanket, CoUninitialize, EOAC_NONE, RPC_C_AUTHN_LEVEL_CALL,
            RPC_C_IMP_LEVEL_IMPERSONATE,
        },
        Variant::{VARIANT, VT_BSTR, VariantClear},
        Wmi::{
            IWbemClassObject, IWbemLocator, IWbemServices, WBEM_FLAG_CONNECT_USE_MAX_WAIT,
            WBEM_FLAG_ENSURE_LOCATABLE, WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY,
            WBEM_GENERIC_FLAG_TYPE, WbemLocator,
        },
    },
    core::{BSTR, PCWSTR},
};

/// Explicit discovery failure; denied access is never treated as missing files.
#[derive(Debug)]
pub enum DriverDiscoveryError {
    /// Windows supplied a failing HRESULT for the named operation.
    Native {
        /// Fixed operation context.
        operation: &'static str,
        /// Original COM/WMI error.
        source: windows::core::Error,
    },
    /// Returned facts were absent, ambiguous, malformed or changed.
    Invalid(&'static str),
    /// A required WMI string was null or had another VARIANT type.
    Property {
        /// Requested property name.
        name: String,
        /// Actual VARIANT tag.
        variant_type: u16,
    },
    /// Enumerator did not complete within the configured discovery deadline.
    Timeout,
}
impl fmt::Display for DriverDiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native { operation, source } => {
                write!(f, "{operation}: {}: {source}", source.code())
            }
            Self::Invalid(reason) => write!(f, "driver discovery: {reason}"),
            Self::Property { name, variant_type } => write!(
                f,
                "driver discovery: property {name} has VARIANT type {variant_type}, expected BSTR"
            ),
            Self::Timeout => f.write_str("driver discovery enumeration timed out"),
        }
    }
}
impl std::error::Error for DriverDiscoveryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Native { source, .. } => Some(source),
            _ => None,
        }
    }
}
fn native(operation: &'static str, source: windows::core::Error) -> DriverDiscoveryError {
    DriverDiscoveryError::Native { operation, source }
}

struct Apartment;
#[allow(unsafe_code)]
impl Apartment {
    fn initialize() -> Result<Self, DriverDiscoveryError> {
        // SAFETY: initialize COM on this calling thread. On success this guard
        // remains alive until all WMI interfaces/variants have been released.
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
            .ok()
            .map_err(|error| native("initialize WMI apartment", error))?;
        Ok(Self)
    }
}
#[allow(unsafe_code)]
impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: balances this thread's successful CoInitializeEx call.
        unsafe { CoUninitialize() };
    }
}

#[allow(unsafe_code)]
fn connect(namespace: &str) -> Result<IWbemServices, DriverDiscoveryError> {
    // SAFETY: COM apartment is initialized; COM owns/refcounts the returned interface.
    let locator: IWbemLocator =
        unsafe { CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER) }
            .map_err(|error| native("create WMI locator", error))?;
    let empty = BSTR::new();
    // SAFETY: all BSTRs live throughout the call; no credentials/context are supplied.
    let services = unsafe {
        locator.ConnectServer(
            &BSTR::from(namespace),
            &empty,
            &empty,
            &empty,
            WBEM_FLAG_CONNECT_USE_MAX_WAIT.0,
            &empty,
            None,
        )
    }
    .map_err(|error| native("connect local WMI", error))?;
    // RPC_C_AUTHN_WINNT=10 / RPC_C_AUTHZ_NONE=0 are Windows RPC protocol constants.
    // SAFETY: services is a live COM proxy. Impersonation uses the current token,
    // without changing its rights or providing an alternative identity.
    unsafe {
        CoSetProxyBlanket(
            &services,
            10,
            0,
            PCWSTR::null(),
            RPC_C_AUTHN_LEVEL_CALL,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            None,
            EOAC_NONE,
        )
    }
    .map_err(|error| native("secure WMI proxy", error))?;
    Ok(services)
}

#[allow(unsafe_code)]
fn property(object: &IWbemClassObject, name: &str) -> Result<String, DriverDiscoveryError> {
    let mut value = VARIANT::default();
    let wide = name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    // SAFETY: live object, NUL-terminated property name and initialized VARIANT output.
    let get = unsafe { object.Get(PCWSTR(wide.as_ptr()), 0, &mut value, None, None) };
    let result = if let Err(error) = get {
        Err(native("read WMI property", error))
    } else {
        // SAFETY: Get initialized the VARIANT; inspect its tag before its BSTR arm.
        let inner = unsafe { &value.Anonymous.Anonymous };
        if name == "ConfigManagerErrorCode" && inner.vt == windows::Win32::System::Variant::VT_I4 {
            // SAFETY: VT_I4 selects this signed 32-bit arm. WMI uint32 uses VT_I4.
            let code = unsafe { inner.Anonymous.lVal } as u32;
            Ok(code.to_string())
        } else if inner.vt != VT_BSTR {
            Err(DriverDiscoveryError::Property {
                name: name.to_owned(),
                variant_type: inner.vt.0,
            })
        } else {
            // SAFETY: VT_BSTR selects this arm. Copy the string before VariantClear.
            let bstr = unsafe { &inner.Anonymous.bstrVal };
            String::from_utf16(bstr)
                .map_err(|_| DriverDiscoveryError::Invalid("invalid UTF-16 WMI property"))
        }
    };
    // SAFETY: clear the initialized VARIANT exactly once, including failed Get paths.
    let cleared = unsafe { VariantClear(&mut value) };
    if result.is_ok() {
        cleared.map_err(|error| native("release WMI property", error))?;
    }
    result
}

#[allow(unsafe_code)]
fn query(
    services: &IWbemServices,
    wql: &str,
    properties: &[&str],
    deadline: Instant,
) -> Result<Vec<BTreeMap<String, String>>, DriverDiscoveryError> {
    // SAFETY: services is live; query BSTRs remain alive until the call returns.
    let enumerator = unsafe {
        services.ExecQuery(
            &BSTR::from("WQL"),
            &BSTR::from(wql),
            WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY | WBEM_FLAG_ENSURE_LOCATABLE,
            None,
        )
    }
    .map_err(|error| native("execute driver WMI query", error))?;
    let mut rows = Vec::new();
    loop {
        if Instant::now() >= deadline {
            return Err(DriverDiscoveryError::Timeout);
        }
        let mut objects = [None];
        let mut returned = 0;
        // SAFETY: the one-element output array and returned count are writable;
        // the enumerator owns references returned in that array. Wait <= 1 second.
        let status = unsafe { enumerator.Next(1000, &mut objects, &mut returned) };
        status
            .ok()
            .map_err(|error| native("enumerate driver WMI query", error))?;
        if returned == 0 {
            if status.0 == 1 {
                break;
            } // WBEM_S_FALSE: complete enumeration.
            if status.0 == 0x40004 {
                continue;
            } // WBEM_S_TIMEDOUT: bounded wait expired.
            return Err(DriverDiscoveryError::Invalid(
                "unexpected empty WMI enumeration",
            ));
        }
        if returned != 1 || rows.len() >= 16_384 {
            return Err(DriverDiscoveryError::Invalid(
                "WMI result exceeds safety bound",
            ));
        }
        let object = objects[0]
            .take()
            .ok_or(DriverDiscoveryError::Invalid("WMI returned no object"))?;
        let mut row = BTreeMap::new();
        for name in properties {
            row.insert((*name).to_owned(), property(&object, name)?);
        }
        rows.push(row);
    }
    Ok(rows)
}

fn quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

/// Read the single virtual-render devnode. The worker's outer process watchdog
/// bounds synchronous COM setup in addition to WMI enumeration.
pub(crate) fn virtual_render_sample(
    deadline: Instant,
) -> Result<(String, String, u32), DriverDiscoveryError> {
    let _apartment = Apartment::initialize()?;
    let services = connect("ROOT\\CIMV2")?;
    let mut driver = exactly_one(query(
        &services,
        "SELECT DeviceName,DeviceID FROM Win32_PnPSignedDriver WHERE InfName='vrd.inf' AND DeviceClass='DISPLAY'",
        &["DeviceName", "DeviceID"],
        deadline,
    )?)?;
    let name = take(&mut driver, "DeviceName")?;
    let id = take(&mut driver, "DeviceID")?;
    let mut device = exactly_one(query(
        &services,
        &format!(
            "SELECT ConfigManagerErrorCode FROM Win32_PnPEntity WHERE DeviceID='{}'",
            quoted(&id)
        ),
        &["ConfigManagerErrorCode"],
        deadline,
    )?)?;
    let code = take(&mut device, "ConfigManagerErrorCode")?
        .parse()
        .map_err(|_| DriverDiscoveryError::Invalid("invalid PnP problem code"))?;
    Ok((id, name, code))
}

pub(crate) fn operating_system_version() -> Result<String, DriverDiscoveryError> {
    let _apartment = Apartment::initialize()?;
    let services = connect("ROOT\\CIMV2")?;
    let mut row = exactly_one(query(
        &services,
        "SELECT Version FROM Win32_OperatingSystem",
        &["Version"],
        Instant::now() + std::time::Duration::from_secs(5),
    )?)?;
    take(&mut row, "Version")
}
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
    project: &ProjectConfiguration,
) -> Result<DriverDiscovery, DriverDiscoveryError> {
    let device_id = physical_device_id(&project.slot.gpu_interface)
        .map_err(|_| DriverDiscoveryError::Invalid("configured interface is malformed"))?;
    let _apartment = Apartment::initialize()?;
    let virtualization = connect(r"ROOT\virtualization\v2")?;
    let deadline = Instant::now() + project.driver_manifest.discovery_timeout;
    let gpu_query = format!(
        "SELECT Name FROM Msvm_PartitionableGpu WHERE Name='{}'",
        quoted(&project.slot.gpu_interface)
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
        "SELECT DeviceName,DriverVersion,InfName FROM Win32_PNPSignedDriver WHERE DeviceID='{}'",
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
    // Resolve the selected signed driver's association Antecedent. The provider returns
    // null __PATH for projected signed-driver objects even with ENSURE_LOCATABLE.
    let mut computer = exactly_one(query(
        &cim,
        "SELECT Name FROM Win32_ComputerSystem",
        &["Name"],
        deadline,
    )?)?;
    let hostname = take(&mut computer, "Name")?;
    let antecedent = signed_driver_antecedent(&hostname, &device_id);
    let association_query = format!(
        "SELECT Dependent FROM Win32_PNPSignedDriverCIMDataFile WHERE Antecedent='{}'",
        quoted(&antecedent)
    );
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
}

fn signed_driver_antecedent(hostname: &str, device_id: &str) -> String {
    let escaped = device_id.replace('\\', "\\\\").replace('"', "\\\"");
    format!(r#"\\{hostname}\ROOT\cimv2:Win32_PNPSignedDriver.DeviceID="{escaped}""#)
}

#[allow(unsafe_code)]
fn dependent_file_name(
    services: &IWbemServices,
    path: &str,
) -> Result<String, DriverDiscoveryError> {
    let mut object = None;
    // SAFETY: live services, a provider-returned object path and initialized output.
    // Resolve through WMI so quoted/backslash-containing paths are not hand-parsed.
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escapes_wql_and_rejects_absent_ambiguous_and_empty_facts() {
        assert_eq!(
            signed_driver_antecedent("HOST", r"PCI\x"),
            r#"\\HOST\ROOT\cimv2:Win32_PNPSignedDriver.DeviceID="PCI\\x""#
        );
        assert_eq!(quoted("PCI\\x'y"), "PCI\\\\x\\'y");
        assert!(exactly_one(vec![]).is_err());
        assert!(exactly_one(vec![BTreeMap::new(), BTreeMap::new()]).is_err());
        let mut row = exactly_one(vec![BTreeMap::from([("Name".into(), "GPU".into())])]).unwrap();
        assert_eq!(take(&mut row, "Name").unwrap(), "GPU");
        assert!(take(&mut row, "Name").is_err());
        row.insert("Name".into(), String::new());
        assert!(take(&mut row, "Name").is_err());
    }
}
