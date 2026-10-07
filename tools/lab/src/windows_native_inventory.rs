//! Native host and exact-target inventory, executed only by the bounded worker.
//! Registry/system APIs supply host facts. Existing COM/WMI bindings supply GPU
//! driver and Hyper-V facts; no manifest hashing or payload rediscovery is needed.

use crate::{
    config::ProjectConfiguration,
    driver_environment::physical_device_id,
    inventory::{Fact, FactStatus, InventoryError, InventoryReport},
    windows_driver_environment::{Apartment, DriverDiscoveryError, connect, query, quoted},
};
use std::{collections::BTreeMap, time::Instant};
use windows::Win32::System::Wmi::IWbemServices;

type Row = BTreeMap<String, String>;
type Observation<T> = Result<T, Failure>;

#[derive(Debug, Clone)]
struct Failure {
    status: FactStatus,
    detail: String,
}
impl Failure {
    fn invalid(detail: &str) -> Self {
        Self {
            status: FactStatus::Unavailable,
            detail: detail.to_owned(),
        }
    }
    fn missing(detail: &str) -> Self {
        Self {
            status: FactStatus::Missing,
            detail: detail.to_owned(),
        }
    }
    fn code(code: i32, context: &str) -> Self {
        let status = match code as u32 {
            0x80070005 | 0x80041003 => FactStatus::Denied,
            0x80070002 | 0x80041002 | 0x8004100e | 0x80041010 => FactStatus::Missing,
            _ => FactStatus::Unavailable,
        };
        Self {
            status,
            detail: format!("{context}: HRESULT 0x{:08X}", code as u32),
        }
    }
}
impl From<DriverDiscoveryError> for Failure {
    fn from(error: DriverDiscoveryError) -> Self {
        match error {
            DriverDiscoveryError::Native {
                operation, source, ..
            } => Self::code(source.code().0, &operation),
            other => Self::invalid(&other.to_string()),
        }
    }
}

fn add(facts: &mut Vec<Fact>, key: &str, value: Observation<String>) -> Result<(), InventoryError> {
    let (status, value) = match value {
        Ok(value) if !value.is_empty() => (FactStatus::Known, value),
        Ok(_) => (
            FactStatus::Unavailable,
            "provider returned an empty value".into(),
        ),
        Err(failure) => (failure.status, failure.detail),
    };
    facts.push(Fact::new(key, status, value)?);
    Ok(())
}
fn field(row: &Row, name: &str) -> Observation<String> {
    row.get(name)
        .filter(|s| !s.is_empty())
        .cloned()
        .ok_or_else(|| Failure::invalid(&format!("missing/empty provider property {name}")))
}

/// Collect host facts and the configured GPU/VM through native read-only APIs.
///
/// # Errors
/// Report construction failures are returned; native provider failures instead
/// retain known/missing/denied/unavailable states on the affected facts. Calls can
/// block inside COM and must run under the fixed worker's outer process deadline.
pub fn collect_native(project: &ProjectConfiguration) -> Result<InventoryReport, InventoryError> {
    let mut facts = Vec::new();
    add(
        &mut facts,
        "worker.configuration",
        Ok(crate::probe::sha256_hex(include_bytes!(
            "../../../config/project.toml"
        ))),
    )?;
    for (key, name) in [
        ("host.edition", "EditionID"),
        ("host.version", "DisplayVersion"),
    ] {
        add(&mut facts, key, registry_string(name))?;
    }
    let build = registry_string("CurrentBuild").and_then(|build| {
        if build.parse::<u32>().is_err() {
            return Err(Failure::invalid("malformed host build"));
        }
        registry_ubr().map(|ubr| format!("{build}.{ubr}"))
    });
    add(&mut facts, "host.build", build)?;
    add(&mut facts, "host.architecture", architecture())?;
    let deadline = Instant::now() + project.inventory.timeout;
    let apartment = Apartment::initialize();
    // All interfaces are scoped below the apartment guard, including failures.
    let cim = match &apartment {
        Ok(_) => connect(r"ROOT\CIMV2").map_err(Failure::from),
        Err(error) => Err(Failure::invalid(&error.to_string())),
    };
    let device = physical_device_id(&project.slot.gpu_interface)
        .map_err(|_| InventoryError::InvalidProtocol)?;
    let drivers = read(
        &cim,
        &driver_query(&device),
        &["DeviceID", "DeviceName", "DriverVersion", "InfName"],
        deadline,
    );
    gpu_facts(&mut facts, project, drivers)?;
    // The provider can silently filter VM enumeration for a UAC-filtered token.
    // Do not turn that empty enumeration into a claim that the target is absent.
    let hyperv = management_access().and_then(|()| match &apartment {
        Ok(_) => connect(r"ROOT\virtualization\v2").map_err(Failure::from),
        Err(error) => Err(Failure::invalid(&error.to_string())),
    });
    let interfaces = read(
        &hyperv,
        "SELECT Name FROM Msvm_PartitionableGpu",
        &["Name"],
        deadline,
    );
    let interface =
        interfaces.and_then(|rows| select_interface(&rows, &project.slot.gpu_interface));
    add(&mut facts, "gpup.interface", interface)?;
    let vm_count = read(
        &hyperv,
        "SELECT Name FROM Msvm_ComputerSystem WHERE Caption='Virtual Machine'",
        &["Name"],
        deadline,
    )
    .map(|rows| rows.len());
    let vms = read(
        &hyperv,
        &vm_query(&project.slot.vm_id),
        &["Name", "ElementName", "EnabledState"],
        deadline,
    );
    let settings = read(
        &hyperv,
        &settings_query(&project.slot.vm_id),
        &["VirtualSystemIdentifier", "VirtualSystemSubType", "Version"],
        deadline,
    );
    vm_facts(&mut facts, project, vm_count, vms, settings)?;
    InventoryReport::new(facts)
}

// Select by a validated, WQL-escaped full identity before providers decode any
// detail properties. Unrelated null driver/settings fields cannot poison target
// inventory. Registered VM count queries only the provider's Name key.
fn driver_query(device: &str) -> String {
    format!(
        "SELECT DeviceID,DeviceName,DriverVersion,InfName FROM Win32_PnPSignedDriver WHERE DeviceClass='DISPLAY' AND DeviceID='{}'",
        quoted(device)
    )
}
fn vm_query(id: &str) -> String {
    format!(
        "SELECT Name,ElementName,EnabledState FROM Msvm_ComputerSystem WHERE Caption='Virtual Machine' AND Name='{}'",
        quoted(id)
    )
}
fn settings_query(id: &str) -> String {
    format!(
        "SELECT VirtualSystemIdentifier,VirtualSystemSubType,Version FROM Msvm_VirtualSystemSettingData WHERE VirtualSystemType='Microsoft:Hyper-V:System:Realized' AND VirtualSystemIdentifier='{}'",
        quoted(id)
    )
}

fn read(
    services: &Observation<IWbemServices>,
    wql: &str,
    properties: &[&str],
    deadline: Instant,
) -> Observation<Vec<Row>> {
    match services {
        Ok(services) => query(services, wql, properties, deadline).map_err(Failure::from),
        Err(failure) => Err(Failure {
            status: failure.status,
            detail: failure.detail.clone(),
        }),
    }
}

fn selected<'a>(rows: &'a [Row], property: &str, identity: &str) -> Observation<&'a Row> {
    let mut matching = Vec::new();
    for row in rows {
        if field(row, property)?.eq_ignore_ascii_case(identity) {
            matching.push(row);
        }
    }
    match matching.as_slice() {
        [row] => Ok(*row),
        [] => Err(Failure::missing("configured target not found")),
        _ => Err(Failure::invalid("configured target is ambiguous")),
    }
}

fn gpu_facts(
    facts: &mut Vec<Fact>,
    project: &ProjectConfiguration,
    drivers: Observation<Vec<Row>>,
) -> Result<(), InventoryError> {
    let gpu = drivers.and_then(|rows| {
        let device = physical_device_id(&project.slot.gpu_interface)
            .map_err(|_| Failure::invalid("invalid configured GPU interface"))?;
        selected(&rows, "DeviceID", &device).cloned()
    });
    for (key, property) in [
        ("gpu.model", "DeviceName"),
        ("gpu.pciid", "DeviceID"),
        ("gpu.driverversion", "DriverVersion"),
        ("gpu.driverinf", "InfName"),
    ] {
        let value = match &gpu {
            Ok(row) => field(row, property).and_then(|value| {
                if property == "DeviceID" {
                    value
                        .split('\\')
                        .nth(1)
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned)
                        .ok_or_else(|| Failure::invalid("malformed PCI device identity"))
                } else {
                    Ok(value)
                }
            }),
            Err(f) => Err(Failure {
                status: f.status,
                detail: f.detail.clone(),
            }),
        };
        add(facts, key, value)?;
    }
    Ok(())
}
fn select_interface(rows: &[Row], identity: &str) -> Observation<String> {
    field(selected(rows, "Name", identity)?, "Name")
}

fn vm_facts(
    facts: &mut Vec<Fact>,
    project: &ProjectConfiguration,
    count: Observation<usize>,
    vms: Observation<Vec<Row>>,
    settings: Observation<Vec<Row>>,
) -> Result<(), InventoryError> {
    add(facts, "vm.count", count.map(|count| count.to_string()))?;
    let vm = match vms {
        Ok(rows) => selected(&rows, "Name", &project.slot.vm_id).and_then(|row| {
            if !field(row, "ElementName")?.eq_ignore_ascii_case(&project.slot.vm_name) {
                return Err(Failure::invalid("configured VM name does not match GUID"));
            }
            Ok(row.clone())
        }),
        Err(f) => Err(f),
    };
    for (key, property) in [("vm.selection", "Name"), ("vm.state", "EnabledState")] {
        let value = match &vm {
            Ok(row) => field(row, property).and_then(|value| {
                if property == "EnabledState" {
                    vm_state(&value)
                } else {
                    Ok(value.to_ascii_lowercase())
                }
            }),
            Err(f) => Err(Failure {
                status: f.status,
                detail: f.detail.clone(),
            }),
        };
        add(facts, key, value)?;
    }
    let settings = match &vm {
        Ok(_) => settings.and_then(|rows| {
            selected(&rows, "VirtualSystemIdentifier", &project.slot.vm_id).cloned()
        }),
        Err(f) => Err(Failure {
            status: f.status,
            detail: f.detail.clone(),
        }),
    };
    for (key, property) in [
        ("vm.generation", "VirtualSystemSubType"),
        ("vm.version", "Version"),
    ] {
        let value = match &settings {
            Ok(row) => field(row, property).and_then(|value| {
                if property == "VirtualSystemSubType" {
                    generation(&value)
                } else if value.split_once('.').is_some_and(|(major, minor)| {
                    major.parse::<u32>().is_ok() && minor.parse::<u32>().is_ok()
                }) {
                    Ok(value)
                } else {
                    Err(Failure::invalid("malformed VM configuration version"))
                }
            }),
            Err(f) => Err(Failure {
                status: f.status,
                detail: f.detail.clone(),
            }),
        };
        add(facts, key, value)?;
    }
    Ok(())
}

// Hyper-V v2 provider constants, not configuration values. See Microsoft's
// Msvm_ComputerSystem.EnabledState and Msvm_VirtualSystemSettingData documentation.
fn generation(subtype: &str) -> Observation<String> {
    match subtype {
        "Microsoft:Hyper-V:SubType:1" => Ok("1".into()),
        "Microsoft:Hyper-V:SubType:2" => Ok("2".into()),
        _ => Err(Failure::invalid("unknown VM generation subtype")),
    }
}
fn vm_state(state: &str) -> Observation<String> {
    let label = match state {
        "2" => "Running",
        "3" => "Off",
        "4" => "Stopping",
        "10" => "Starting",
        "32768" => "Paused",
        "32769" => "Saved",
        "32770" => "Starting",
        "32771" => "Snapshotting",
        "32773" => "Saving",
        "32774" => "Stopping",
        "32776" => "Pausing",
        "32777" => "Resuming",
        _ => {
            return Err(Failure::invalid(&format!(
                "unknown Hyper-V EnabledState {state}"
            )));
        }
    };
    Ok(label.into())
}

#[allow(unsafe_code)]
fn architecture() -> Observation<String> {
    use windows::Win32::System::SystemInformation::{
        GetNativeSystemInfo, PROCESSOR_ARCHITECTURE_AMD64, PROCESSOR_ARCHITECTURE_ARM64,
        PROCESSOR_ARCHITECTURE_INTEL, SYSTEM_INFO,
    };
    let mut info = SYSTEM_INFO::default();
    // SAFETY: correctly sized writable SYSTEM_INFO; native OS architecture is
    // independent of this process's bitness. The API always fills this structure.
    unsafe { GetNativeSystemInfo(&mut info) };
    // SAFETY: GetNativeSystemInfo initialized the processor architecture arm.
    match unsafe { info.Anonymous.Anonymous.wProcessorArchitecture } {
        PROCESSOR_ARCHITECTURE_AMD64 => Ok("X64".into()),
        PROCESSOR_ARCHITECTURE_ARM64 => Ok("Arm64".into()),
        PROCESSOR_ARCHITECTURE_INTEL => Ok("X86".into()),
        _ => Err(Failure::invalid("unrecognized native architecture")),
    }
}

#[allow(unsafe_code)]
fn registry_string(name: &str) -> Observation<String> {
    use windows::{
        Win32::System::Registry::{
            HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RRF_SUBKEY_WOW6464KEY, RegGetValueW,
        },
        core::{PCWSTR, w},
    };
    let wide = name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let mut value = [0u16; 1024];
    let mut bytes = std::mem::size_of_val(&value) as u32;
    // SAFETY: fixed HKLM read-only key, terminated names and bounded aligned
    // UTF-16 output. RegGetValue validates REG_SZ and guarantees termination.
    let result = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion"),
            PCWSTR(wide.as_ptr()),
            RRF_RT_REG_SZ | RRF_SUBKEY_WOW6464KEY,
            None,
            Some(value.as_mut_ptr().cast()),
            Some(&mut bytes),
        )
    };
    result
        .ok()
        .map_err(|e| Failure::code(e.code().0, &format!("read host registry {name}")))?;
    if bytes < 2 || bytes as usize > std::mem::size_of_val(&value) || !bytes.is_multiple_of(2) {
        return Err(Failure::invalid("malformed host registry string length"));
    }
    let length = bytes as usize / 2;
    if value[length - 1] != 0 {
        return Err(Failure::invalid("unterminated host registry string"));
    }
    String::from_utf16(&value[..length - 1])
        .map_err(|_| Failure::invalid("malformed UTF-16 host registry string"))
}
#[allow(unsafe_code)]
fn registry_ubr() -> Observation<u32> {
    use windows::{
        Win32::System::Registry::{
            HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD, RRF_SUBKEY_WOW6464KEY, RegGetValueW,
        },
        core::w,
    };
    let mut value = 0u32;
    let mut bytes = std::mem::size_of_val(&value) as u32;
    // SAFETY: fixed read-only key/value and aligned DWORD output. Type is checked
    // by RegGetValue; the fixed buffer cannot be overrun.
    let result = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion"),
            w!("UBR"),
            RRF_RT_REG_DWORD | RRF_SUBKEY_WOW6464KEY,
            None,
            Some((&mut value as *mut u32).cast()),
            Some(&mut bytes),
        )
    };
    result
        .ok()
        .map_err(|e| Failure::code(e.code().0, "read host registry UBR"))?;
    if bytes != 4 {
        return Err(Failure::invalid("malformed host UBR length"));
    }
    Ok(value)
}

#[allow(unsafe_code)]
fn management_access() -> Observation<()> {
    use windows::{
        Win32::Security::{
            CheckTokenMembership, CreateWellKnownSid, PSID, WinBuiltinAdministratorsSid,
            WinBuiltinHyperVAdminsSid,
        },
        core::BOOL,
    };
    for kind in [WinBuiltinAdministratorsSid, WinBuiltinHyperVAdminsSid] {
        // DWORD-aligned buffer larger than SECURITY_MAX_SID_SIZE (68 bytes).
        let mut storage = [0u32; 17];
        let mut bytes = std::mem::size_of_val(&storage) as u32;
        let sid = PSID(storage.as_mut_ptr().cast());
        // SAFETY: writable aligned buffer, built-in non-domain SID and its size.
        unsafe { CreateWellKnownSid(kind, None, Some(sid), &mut bytes) }
            .map_err(|e| Failure::code(e.code().0, "construct management group SID"))?;
        let mut member = BOOL::default();
        // SAFETY: SID initialized above, writable BOOL. Null token means this
        // thread's effective token; disabled/deny-only admin SIDs do not qualify.
        unsafe { CheckTokenMembership(None, sid, &mut member) }
            .map_err(|e| Failure::code(e.code().0, "check effective management rights"))?;
        if member.as_bool() {
            return Ok(());
        }
    }
    Err(Failure { status: FactStatus::Denied, detail: "effective token lacks enabled Administrators/Hyper-V Administrators membership; VM enumeration may be filtered".into() })
}

pub(crate) fn require_management_access() -> Result<(), String> {
    management_access().map_err(|error| error.detail)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(values: &[(&str, &str)]) -> Row {
        values
            .iter()
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect()
    }
    fn report(facts: Vec<Fact>) -> InventoryReport {
        InventoryReport::new(facts).unwrap()
    }
    fn fact<'a>(report: &'a InventoryReport, key: &str) -> &'a Fact {
        report.facts().iter().find(|f| f.key() == key).unwrap()
    }
    fn vm(project: &ProjectConfiguration) -> Row {
        row(&[
            ("Name", &project.slot.vm_id),
            ("ElementName", &project.slot.vm_name),
            ("EnabledState", "3"),
        ])
    }
    fn settings(project: &ProjectConfiguration) -> Row {
        row(&[
            ("VirtualSystemIdentifier", &project.slot.vm_id),
            ("VirtualSystemSubType", "Microsoft:Hyper-V:SubType:2"),
            ("Version", "12.0"),
        ])
    }
    #[test]
    fn native_failures_preserve_missing_denied_unavailable_and_hresult() {
        for (code, status) in [
            (0x80070005u32, FactStatus::Denied),
            (0x80041003, FactStatus::Denied),
            (0x8004100e, FactStatus::Missing),
            (0x80041010, FactStatus::Missing),
            (0x80041001, FactStatus::Unavailable),
        ] {
            let failure = Failure::from(DriverDiscoveryError::Native {
                operation: "fixed query".into(),
                source: windows::core::Error::from_hresult(windows::core::HRESULT(code as i32)),
                message: "provider message".into(),
            });
            assert_eq!(failure.status, status);
            assert!(failure.detail.contains(&format!("0x{code:08X}")));
        }
    }
    #[test]
    fn full_gpu_identity_distinguishes_same_model_devices() {
        let project = ProjectConfiguration::embedded().unwrap();
        let device = physical_device_id(&project.slot.gpu_interface).unwrap();
        let intended = row(&[
            ("DeviceID", &device),
            ("DeviceName", "configured device"),
            ("DriverVersion", "32.0.1.2"),
            ("InfName", "oem1.inf"),
        ]);
        let mut other = intended.clone();
        other.insert("DeviceID".into(), format!("{device}OTHER"));
        let mut facts = Vec::new();
        gpu_facts(&mut facts, &project, Ok(vec![other, intended])).unwrap();
        assert_eq!(
            fact(&report(facts), "gpu.model").value(),
            "configured device"
        );
        let duplicate = row(&[("DeviceID", &device)]);
        let mut facts = Vec::new();
        gpu_facts(&mut facts, &project, Ok(vec![duplicate.clone(), duplicate])).unwrap();
        assert_eq!(
            fact(&report(facts), "gpu.model").status(),
            FactStatus::Unavailable
        );
    }
    #[test]
    fn absent_gpu_and_failed_provider_never_become_known() {
        let project = ProjectConfiguration::embedded().unwrap();
        for (drivers, status) in [
            (Ok(vec![]), FactStatus::Missing),
            (
                Err(Failure::code(0x80041003u32 as i32, "query drivers")),
                FactStatus::Denied,
            ),
            (Ok(vec![row(&[("DeviceID", "")])]), FactStatus::Unavailable),
        ] {
            let mut facts = Vec::new();
            gpu_facts(&mut facts, &project, drivers).unwrap();
            assert_eq!(fact(&report(facts), "gpu.model").status(), status);
        }
    }
    #[test]
    fn partition_interface_requires_full_exact_identity() {
        let project = ProjectConfiguration::embedded().unwrap();
        let matching = row(&[("Name", &project.slot.gpu_interface.to_ascii_lowercase())]);
        let different = row(&[("Name", &format!("{}OTHER", project.slot.gpu_interface))]);
        assert!(
            select_interface(
                std::slice::from_ref(&different),
                &project.slot.gpu_interface
            )
            .is_err()
        );
        assert!(
            select_interface(&[different, matching.clone()], &project.slot.gpu_interface).is_ok()
        );
        assert_eq!(
            select_interface(&[matching.clone(), matching], &project.slot.gpu_interface)
                .unwrap_err()
                .status,
            FactStatus::Unavailable
        );
    }
    #[test]
    fn configured_vm_is_selected_among_unrelated_registered_vms() {
        let project = ProjectConfiguration::embedded().unwrap();
        let unrelated = row(&[
            ("Name", "unrelated"),
            ("ElementName", "other"),
            ("EnabledState", "2"),
        ]);
        let mut facts = Vec::new();
        vm_facts(
            &mut facts,
            &project,
            Ok(2),
            Ok(vec![unrelated, vm(&project)]),
            Ok(vec![settings(&project)]),
        )
        .unwrap();
        let report = report(facts);
        for (key, value) in [
            ("vm.count", "2"),
            ("vm.selection", project.slot.vm_id.as_str()),
            ("vm.state", "Off"),
            ("vm.generation", "2"),
            ("vm.version", "12.0"),
        ] {
            assert_eq!(fact(&report, key).value(), value);
            assert_eq!(fact(&report, key).status(), FactStatus::Known);
        }
    }
    #[test]
    fn wrong_duplicate_missing_vm_and_denied_visibility_are_distinct() {
        let project = ProjectConfiguration::embedded().unwrap();
        let mut renamed = vm(&project);
        renamed.insert("ElementName".into(), "wrong name".into());
        for (vms, status) in [
            (Ok(vec![]), FactStatus::Missing),
            (Ok(vec![renamed]), FactStatus::Unavailable),
            (
                Ok(vec![vm(&project), vm(&project)]),
                FactStatus::Unavailable,
            ),
            (
                Err(Failure::code(0x80041003u32 as i32, "filtered token")),
                FactStatus::Denied,
            ),
        ] {
            let mut facts = Vec::new();
            let count = vms.as_ref().map(Vec::len).map_err(Clone::clone);
            vm_facts(
                &mut facts,
                &project,
                count,
                vms,
                Ok(vec![settings(&project)]),
            )
            .unwrap();
            let report = report(facts);
            for key in ["vm.selection", "vm.state", "vm.generation", "vm.version"] {
                assert_eq!(fact(&report, key).status(), status);
            }
        }
    }
    #[test]
    fn malformed_or_ambiguous_settings_preserve_valid_vm_identity() {
        let project = ProjectConfiguration::embedded().unwrap();
        for settings_rows in [
            vec![],
            vec![settings(&project), settings(&project)],
            vec![row(&[
                ("VirtualSystemIdentifier", &project.slot.vm_id),
                ("VirtualSystemSubType", "unknown"),
                ("Version", "broken"),
            ])],
        ] {
            let mut facts = Vec::new();
            vm_facts(
                &mut facts,
                &project,
                Ok(1),
                Ok(vec![vm(&project)]),
                Ok(settings_rows),
            )
            .unwrap();
            let report = report(facts);
            assert_eq!(fact(&report, "vm.selection").status(), FactStatus::Known);
            assert_ne!(fact(&report, "vm.generation").status(), FactStatus::Known);
            assert_ne!(fact(&report, "vm.version").status(), FactStatus::Known);
        }
    }
    #[test]
    fn unknown_state_and_generation_remain_unavailable() {
        assert_eq!(vm_state("2").unwrap(), "Running");
        assert_eq!(vm_state("10").unwrap(), "Starting");
        assert_eq!(vm_state("32769").unwrap(), "Saved");
        for state in ["0", "-1", "bogus", "4294967295"] {
            assert_eq!(vm_state(state).unwrap_err().status, FactStatus::Unavailable);
        }
        assert_eq!(generation("Microsoft:Hyper-V:SubType:2").unwrap(), "2");
        assert!(generation("unknown").is_err());
    }

    #[test]
    fn provider_detail_queries_filter_exact_identity_before_property_decoding() {
        // WMI filters WHERE before projecting/decoding BSTR fields. Constrain
        // each detail query, so an unrelated row with null detail fields never
        // reaches the strict decoder. These hostile fixture identifiers also
        // verify that quotes/backslashes cannot widen the provider predicate.
        let device = r"PCI\VEN_1234&DEV_0001\selected' OR DeviceID='other";
        let id = "selected' OR Name='other";
        assert_eq!(
            driver_query(device),
            format!(
                "SELECT DeviceID,DeviceName,DriverVersion,InfName FROM Win32_PnPSignedDriver WHERE DeviceClass='DISPLAY' AND DeviceID='{}'",
                quoted(device)
            )
        );
        assert_eq!(
            vm_query(id),
            "SELECT Name,ElementName,EnabledState FROM Msvm_ComputerSystem WHERE Caption='Virtual Machine' AND Name='selected\\' OR Name=\\'other'"
        );
        assert_eq!(
            settings_query(id),
            "SELECT VirtualSystemIdentifier,VirtualSystemSubType,Version FROM Msvm_VirtualSystemSettingData WHERE VirtualSystemType='Microsoft:Hyper-V:System:Realized' AND VirtualSystemIdentifier='selected\\' OR Name=\\'other'"
        );
        let project = ProjectConfiguration::embedded().unwrap();
        let mut facts = Vec::new();
        vm_facts(
            &mut facts,
            &project,
            Ok(2),
            Ok(vec![vm(&project)]),
            Ok(vec![settings(&project)]),
        )
        .unwrap();
        let report = report(facts);
        assert_eq!(fact(&report, "vm.count").value(), "2");
        assert_eq!(fact(&report, "vm.selection").status(), FactStatus::Known);
    }
}
