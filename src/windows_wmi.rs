//! Scoped Hyper-V COM objects and typed VARIANT/SAFEARRAY ownership.

use crate::windows_com::{Apartment, connect};
use std::time::{Duration, Instant};
use windows::{
    Win32::System::{
        Com::{CLSCTX_INPROC_SERVER, CoCreateInstance},
        Ole::{
            SafeArrayCreateVector, SafeArrayGetDim, SafeArrayGetElement, SafeArrayGetLBound,
            SafeArrayGetUBound, SafeArrayPutElement,
        },
        Variant::{VARIANT, VT_ARRAY, VT_BOOL, VT_BSTR, VT_EMPTY, VT_I4, VT_NULL, VariantClear},
        Wmi::{
            IWbemClassObject, IWbemObjectTextSrc, IWbemServices, WBEM_FLAG_ENSURE_LOCATABLE,
            WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_GENERIC_FLAG_TYPE,
            WbemObjectTextSrc,
        },
    },
    core::{BSTR, PCWSTR},
};

type Result<T> = std::result::Result<T, String>;
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingOperation {
    schema: u32,
    method: String,
    target: String,
    job: Option<String>,
}
fn operation_path() -> Result<std::path::PathBuf> {
    Ok(crate::runner::data_directory()?.join("provider-operation.json"))
}
fn clear_operation() -> Result<()> {
    let path = operation_path()?;
    crate::security::verify(&path)?;
    std::fs::remove_file(path).map_err(|e| e.to_string())
}
pub(crate) fn require_no_pending_operation() -> Result<()> {
    match std::fs::symlink_metadata(operation_path()?) {
        Ok(_) => Err("unfinished native operation requires explicit manual reconciliation".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}
pub(crate) fn reconcile_pending_operation() -> Result<()> {
    let path = operation_path()?;
    match std::fs::symlink_metadata(&path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    }
    crate::security::verify(&path)?;
    if std::fs::metadata(&path).map_err(|e| e.to_string())?.len() > 65536 {
        return Err("invalid pending provider operation size".into());
    }
    let pending: PendingOperation =
        serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|_| "invalid pending provider operation")?;
    if pending.schema != 1 {
        return Err("unsupported pending provider operation".into());
    }
    let job = pending.job_path()?;
    let s = Session::new(Duration::from_secs(30))?;
    let job = s
        .get(local_job_path(job)?)
        .map_err(|e| format!("previous Hyper-V job cannot be reconciled: {e}"))?;
    let state = job.number("JobState")?;
    require_terminal_operation(state)?;
    clear_operation()
}
impl PendingOperation {
    fn job_path(&self) -> Result<&str> {
        local_job_path(self.job.as_deref().ok_or("previous Hyper-V call has an unknown outcome; administrator reconciliation is required before another mutation")?)
    }
}
fn require_terminal_operation(state: u64) -> Result<()> {
    if (7..=10).contains(&state) {
        Ok(())
    } else {
        Err(format!(
            "previous Hyper-V job is unresolved (state {state}); wait for terminal reconciliation before retry"
        ))
    }
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn win(e: windows::core::Error) -> String {
    format!("Hyper-V WMI {}: {}", e.code(), e.message())
}

pub(crate) struct OwnedVariant(VARIANT);
impl From<VARIANT> for OwnedVariant {
    fn from(value: VARIANT) -> Self {
        Self(value)
    }
}
#[allow(unsafe_code)]
impl Drop for OwnedVariant {
    fn drop(&mut self) {
        // SAFETY: exclusively owned initialized VARIANT; balances its allocation.
        let _ = unsafe { VariantClear(&mut self.0) };
    }
}

#[derive(Clone)]
pub(crate) struct Object(IWbemClassObject);

#[allow(unsafe_code)]
impl Object {
    fn value(&self, name: &str) -> Result<OwnedVariant> {
        let mut result = OwnedVariant(VARIANT::default());
        // SAFETY: live object, initialized writable VARIANT, terminated name.
        unsafe {
            self.0
                .Get(PCWSTR(wide(name).as_ptr()), 0, &mut result.0, None, None)
        }
        .map_err(win)?;
        Ok(result)
    }
    pub(crate) fn string(&self, name: &str) -> Result<String> {
        let value = self.value(name)?;
        // SAFETY: initialized VARIANT; only inspect an arm after checking its tag.
        let inner = unsafe { &value.0.Anonymous.Anonymous };
        if inner.vt != VT_BSTR {
            return Err(format!("{name}: expected WMI string, got {}", inner.vt.0));
        }
        // SAFETY: VT_BSTR selects bstrVal; copy before variant is released.
        String::from_utf16(unsafe { &inner.Anonymous.bstrVal })
            .map_err(|_| format!("{name}: invalid UTF-16"))
    }
    pub(crate) fn number(&self, name: &str) -> Result<u64> {
        let value = self.value(name)?;
        // SAFETY: initialized VARIANT; select only checked scalar arms.
        let inner = unsafe { &value.0.Anonymous.Anonymous };
        match inner.vt {
            VT_BSTR => {
                // SAFETY: checked VT_BSTR. WMI uint64 is represented as decimal BSTR.
                let s = String::from_utf16(unsafe { &inner.Anonymous.bstrVal })
                    .map_err(|_| format!("{name}: invalid UTF-16"))?;
                if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(format!("{name}: invalid unsigned value"));
                }
                s.parse().map_err(|_| format!("{name}: unsigned overflow"))
            }
            VT_I4 => {
                // SAFETY: WMI uint16/uint32 use VT_I4; preserve the full uint32 bits.
                Ok(u64::from(unsafe { inner.Anonymous.lVal } as u32))
            }
            _ => Err(format!("{name}: unexpected integer VARIANT {}", inner.vt.0)),
        }
    }
    pub(crate) fn optional_number(&self, name: &str) -> Result<Option<u64>> {
        let value = self.value(name)?;
        // SAFETY: initialized VARIANT; inspect only its discriminator.
        if unsafe { value.0.Anonymous.Anonymous.vt } == VT_NULL {
            Ok(None)
        } else {
            self.number(name).map(Some)
        }
    }
    pub(crate) fn boolean(&self, name: &str) -> Result<bool> {
        let value = self.value(name)?;
        // SAFETY: initialized VARIANT; checked VT_BOOL before boolVal.
        let inner = unsafe { &value.0.Anonymous.Anonymous };
        if inner.vt != VT_BOOL {
            return Err(format!("{name}: expected bool"));
        }
        // SAFETY: VT_BOOL selects boolVal.
        match unsafe { inner.Anonymous.boolVal }.0 {
            0 => Ok(false),
            -1 => Ok(true),
            _ => Err(format!("{name}: invalid VARIANT_BOOL")),
        }
    }
    pub(crate) fn strings(&self, name: &str) -> Result<Vec<String>> {
        let value = self.value(name)?;
        // SAFETY: initialized VARIANT; checked array tag before reading parray.
        let inner = unsafe { &value.0.Anonymous.Anonymous };
        if inner.vt == VT_NULL || inner.vt == VT_EMPTY {
            return Ok(Vec::new());
        }
        if inner.vt.0 != (VT_ARRAY.0 | VT_BSTR.0) {
            return Err(format!("{name}: expected string array"));
        }
        // SAFETY: tag selects the owned SAFEARRAY; valid until value is dropped.
        let array = unsafe { inner.Anonymous.parray };
        if array.is_null() || unsafe { SafeArrayGetDim(array) } != 1 {
            return Err(format!("{name}: invalid array dimension"));
        }
        // SAFETY: valid one-dimensional SAFEARRAY and dimension index 1.
        let low = unsafe { SafeArrayGetLBound(array, 1) }.map_err(win)?;
        let high = unsafe { SafeArrayGetUBound(array, 1) }.map_err(win)?;
        if i64::from(high) - i64::from(low) + 1 > 16384 {
            return Err(format!("{name}: excessive array"));
        }
        let mut result = Vec::new();
        for index in low..=high {
            let mut item = BSTR::new();
            // SAFETY: initialized BSTR output receives a separately owned copy.
            unsafe { SafeArrayGetElement(array, &index, (&mut item as *mut BSTR).cast()) }
                .map_err(win)?;
            result.push(
                String::from_utf16(&item).map_err(|_| format!("{name}: invalid array UTF-16"))?,
            );
        }
        Ok(result)
    }
    pub(crate) fn path(&self) -> Result<String> {
        self.string("__RELPATH")
    }
    pub(crate) fn set(&self, name: &str, value: VARIANT) -> Result<()> {
        let value = OwnedVariant(value);
        self.set_owned(name, &value)
    }
    fn set_owned(&self, name: &str, value: &OwnedVariant) -> Result<()> {
        // SAFETY: initialized input VARIANT and live object; Put copies the input.
        unsafe { self.0.Put(PCWSTR(wide(name).as_ptr()), 0, &value.0, 0) }
            .map_err(|e| format!("write property {name}: {}", win(e)))
    }
    pub(crate) fn set_number(&self, name: &str, n: u64) -> Result<()> {
        // Preserve the provider's uint64 BSTR / uint16,32 VT_I4 representation.
        let original = self.value(name)?;
        // SAFETY: initialized variant, only reading its discriminant.
        let tag = unsafe { original.0.Anonymous.Anonymous.vt };
        let v = if tag == VT_BSTR {
            VARIANT::from(BSTR::from(n.to_string()))
        } else if tag == VT_I4 {
            VARIANT::from(u32::try_from(n).map_err(|_| format!("{name}: uint32 overflow"))? as i32)
        } else {
            return Err(format!("{name}: unknown numeric storage type"));
        };
        self.set(name, v)
    }
    pub(crate) fn set_strings(&self, name: &str, values: &[String]) -> Result<()> {
        self.set_owned(name, &string_array(values)?)
    }
    pub(crate) fn xml(&self) -> Result<String> {
        // SAFETY: apartment is initialized and returned COM interface is scoped.
        let source: IWbemObjectTextSrc =
            unsafe { CoCreateInstance(&WbemObjectTextSrc, None, CLSCTX_INPROC_SERVER) }
                .map_err(win)?;
        // SAFETY: live object, CIM DTD 2.0 (protocol format 1), no context.
        let text = unsafe { source.GetText(0, &self.0, 1, None) }
            .map_err(|e| format!("serialize CIM XML: {}", win(e)))?;
        String::from_utf16(&text).map_err(|_| "invalid WMI XML UTF-16".into())
    }
}

#[allow(unsafe_code)]
pub(crate) fn string_array(values: &[String]) -> Result<OwnedVariant> {
    if values.len() > 16384 {
        return Err("excessive WMI input array".into());
    }
    // SAFETY: supported BSTR element type, finite count, zero lower bound.
    let array = unsafe { SafeArrayCreateVector(VT_BSTR, 0, values.len() as u32) };
    if array.is_null() {
        return Err("cannot allocate WMI string array".into());
    }
    let mut owned = OwnedVariant(VARIANT::default());
    // SAFETY: initialize tag and matching array arm, transferring sole ownership.
    unsafe {
        (*owned.0.Anonymous.Anonymous).vt =
            windows::Win32::System::Variant::VARENUM(VT_ARRAY.0 | VT_BSTR.0);
        (*owned.0.Anonymous.Anonymous).Anonymous.parray = array;
    }
    for (index, value) in values.iter().enumerate() {
        let text = BSTR::from(value.as_str());
        // SAFETY: BSTR array copies the pointed-to BSTR value (not pointer-to-pointer).
        unsafe { SafeArrayPutElement(array, &(index as i32), text.as_ptr().cast()) }
            .map_err(win)?;
    }
    Ok(owned)
}

pub(crate) struct Session {
    services: IWbemServices,
    deadline: Instant,
    _apartment: Apartment,
}
#[allow(unsafe_code)]
impl Session {
    pub(crate) fn new(timeout: Duration) -> Result<Self> {
        let apartment = Apartment::initialize().map_err(|e| e.to_string())?;
        let services = connect(r"ROOT\virtualization\v2").map_err(|e| e.to_string())?;
        Ok(Self {
            services,
            deadline: Instant::now() + timeout,
            _apartment: apartment,
        })
    }
    pub(crate) fn check_deadline(&self) -> Result<()> {
        if Instant::now() >= self.deadline {
            Err("Hyper-V native deadline expired; reconcile before retry".into())
        } else {
            Ok(())
        }
    }
    pub(crate) fn query(&self, wql: &str) -> Result<Vec<Object>> {
        self.check_deadline()?;
        // SAFETY: live secured services and query BSTRs; returned enumerator owns refs.
        let e = unsafe {
            self.services.ExecQuery(
                &BSTR::from("WQL"),
                &BSTR::from(wql),
                WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY | WBEM_FLAG_ENSURE_LOCATABLE,
                None,
            )
        }
        .map_err(win)?;
        let mut result = Vec::new();
        loop {
            self.check_deadline()?;
            let mut objects = [None];
            let mut count = 0;
            // SAFETY: initialized one-object output and count, bounded wait.
            let status = unsafe { e.Next(1000, &mut objects, &mut count) };
            status.ok().map_err(win)?;
            if count == 0 {
                if status.0 == 1 {
                    break;
                }
                if status.0 == 0x40004 {
                    continue;
                }
                return Err("unexpected WMI enumeration status".into());
            }
            if count != 1 || result.len() >= 16384 {
                return Err("excessive WMI results".into());
            }
            result.push(Object(objects[0].take().ok_or("empty WMI object")?));
        }
        Ok(result)
    }
    pub(crate) fn one(&self, query: &str) -> Result<Object> {
        one(self.query(query)?)
    }
    pub(crate) fn get(&self, path: &str) -> Result<Object> {
        self.check_deadline()?;
        let mut object = None;
        // SAFETY: writable interface output, live services; returned object owns ref.
        unsafe {
            self.services.GetObject(
                &BSTR::from(path),
                WBEM_GENERIC_FLAG_TYPE(0),
                None,
                Some(&mut object),
                None,
            )
        }
        .map_err(win)?;
        Ok(Object(object.ok_or("missing WMI object")?))
    }
    pub(crate) fn related(&self, object: &Object, class: &str) -> Result<Vec<Object>> {
        self.query(&format!(
            "ASSOCIATORS OF {{{}}} WHERE ResultClass={class}",
            object.path()?
        ))
    }
    pub(crate) fn invoke(
        &self,
        target: &Object,
        method: &str,
        values: Vec<(&str, OwnedVariant)>,
    ) -> Result<Object> {
        // Arguments own their native allocations before even entering this call.
        self.check_deadline()?;
        require_no_pending_operation()?;
        let mut signature = None;
        // GetMethod is valid only on a class definition, never a queried VM or
        // management-service instance. ExecMethod still targets the exact instance.
        let definition = self.get(&target.string("__CLASS")?)?;
        // SAFETY: initialized signature output, live class, terminated fixed method.
        unsafe {
            definition.0.GetMethod(
                PCWSTR(wide(method).as_ptr()),
                0,
                &mut signature,
                std::ptr::null_mut(),
            )
        }
        .map_err(|e| format!("{method}: read method signature: {}", win(e)))?;
        let signature = signature.ok_or("missing WMI input signature")?;
        // SAFETY: provider method input class definition; owns spawned instance.
        let input = Object(
            unsafe { signature.SpawnInstance(0) }
                .map_err(|e| format!("{method}: create method input: {}", win(e)))?,
        );
        for (key, value) in values {
            input
                .set_owned(key, &value)
                .map_err(|e| format!("{method}: {e}"))?;
        }
        let mut output = None;
        let mut pending = PendingOperation {
            schema: 1,
            method: method.into(),
            target: target.path()?,
            job: None,
        };
        crate::runner::atomic_json(&operation_path()?, &pending)?;
        // SAFETY: secured services, live input, initialized output; fixed target method.
        unsafe {
            self.services.ExecMethod(
                &BSTR::from(pending.target.as_str()),
                &BSTR::from(method),
                WBEM_GENERIC_FLAG_TYPE(0),
                None,
                &input.0,
                Some(&mut output),
                None,
            )
        }
        .map_err(|e| format!("{method}: invoke method: {}", win(e)))?;
        let output = Object(output.ok_or("missing WMI method output")?);
        let code = output.number("ReturnValue")?;
        if code == 4096 {
            let path = output.string("Job")?;
            // Do not follow remote/arbitrary provider references.
            let path = local_job_path(&path)?;
            pending.job = Some(path.into());
            crate::runner::atomic_json(&operation_path()?, &pending)?;
            loop {
                self.check_deadline()?;
                let job = self.get(path)?;
                let state = job.number("JobState")?;
                if (7..=10).contains(&state) {
                    clear_operation()?;
                }
                match job_status(state).map_err(|reason| {
                    format!(
                        "{method}: {reason}; error code {}; {}",
                        job.number("ErrorCode")
                            .map_or_else(|e| e, |code| code.to_string()),
                        job.string("ErrorDescription").unwrap_or_else(|e| e)
                    )
                })? {
                    true => {
                        if job.number("ErrorCode")? != 0 {
                            return Err(format!(
                                "{method}: job failed: {}",
                                job.string("ErrorDescription")?
                            ));
                        }
                        break;
                    }
                    false => std::thread::sleep(Duration::from_millis(100)),
                }
            }
        } else if code != 0 {
            clear_operation()?;
            return Err(format!(
                "{method}: provider return code {code}; reconcile before retry"
            ));
        } else {
            clear_operation()?;
        }
        Ok(output)
    }
    pub(crate) fn modify(&self, service: &Object, object: &Object, system: bool) -> Result<()> {
        let xml = object.xml()?;
        let args = if system {
            vec![("SystemSettings", VARIANT::from(BSTR::from(xml)).into())]
        } else {
            vec![("ResourceSettings", string_array(&[xml])?)]
        };
        self.invoke(
            service,
            if system {
                "ModifySystemSettings"
            } else {
                "ModifyResourceSettings"
            },
            args,
        )?;
        Ok(())
    }
}
fn job_status(state: u64) -> Result<bool> {
    match state {
        7 => Ok(true),
        2..=6 => Ok(false),
        8..=10 => Err(format!(
            "Hyper-V job terminated in state {state}; reconcile before retry"
        )),
        _ => Err(format!(
            "unknown Hyper-V job state {state}; reconcile before retry"
        )),
    }
}
fn local_job_path(path: &str) -> Result<&str> {
    // Never follow the authority/namespace in an absolute provider reference.
    // Resolve only the validated relative object key on our secured local service.
    let relative = path.rsplit_once(':').map_or(path, |(_, relative)| relative);
    let prefix = "Msvm_ConcreteJob.InstanceID=";
    if relative
        .get(..prefix.len())
        .is_none_or(|head| !head.eq_ignore_ascii_case(prefix))
    {
        return Err("invalid Hyper-V job reference".into());
    }
    let key = &relative[prefix.len()..];
    if key.len() < 3
        || !key.starts_with('"')
        || !key.ends_with('"')
        || !key[1..key.len() - 1]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("invalid Hyper-V job key".into());
    }
    Ok(relative)
}

pub(crate) fn one(mut objects: Vec<Object>) -> Result<Object> {
    if objects.len() != 1 {
        return Err(format!(
            "expected one Hyper-V object, got {}",
            objects.len()
        ));
    }
    objects.pop().ok_or_else(|| "missing Hyper-V object".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uncertain_provider_work_never_allows_overlapping_mutation() {
        let mut operation = PendingOperation {
            schema: 1,
            method: "AddResourceSettings".into(),
            target: "fixed-vm".into(),
            job: None,
        };
        assert!(operation.job_path().is_err());
        operation.job = Some("other.InstanceID=\"123\"".into());
        assert!(operation.job_path().is_err());
        operation.job = Some("Msvm_ConcreteJob.InstanceID=\"123\"".into());
        assert!(operation.job_path().is_ok());
        for state in [0, 1, 2, 3, 4, 5, 6, 11, u64::MAX] {
            assert!(require_terminal_operation(state).is_err());
        }
        for state in 7..=10 {
            assert!(require_terminal_operation(state).is_ok());
        }
    }

    #[test]
    fn provider_job_terminal_failures_never_become_success() {
        for state in 2..=6 {
            assert!(!job_status(state).unwrap());
        }
        assert!(job_status(7).unwrap());
        for state in [0, 1, 8, 9, 10, 11, u64::MAX] {
            assert!(job_status(state).is_err());
        }
    }

    #[test]
    fn job_references_are_resolved_only_as_local_single_keys() {
        assert_eq!(
            local_job_path(
                r#"\\host\root\virtualization\v2:Msvm_ConcreteJob.InstanceID="0123-ABCD""#
            )
            .unwrap(),
            r#"Msvm_ConcreteJob.InstanceID="0123-ABCD""#
        );
        for path in [
            "other.InstanceID=\"123\"",
            "Msvm_ConcreteJob.InstanceID=\"\"",
            "Msvm_ConcreteJob.InstanceID=\"123\",Other=\"456\"",
            "éééééééééééééééééééééé",
        ] {
            assert!(local_job_path(path).is_err(), "{path}");
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn cim_values_round_trip_and_reject_wrong_types_without_hyperv() {
        let _apartment = Apartment::initialize().unwrap();
        // SAFETY: initialized apartment; test creates an in-memory CIM instance,
        // without provider connection, privileges, or host/guest effects.
        use windows::Win32::System::Wmi::{
            CIM_BOOLEAN, CIM_FLAG_ARRAY, CIM_STRING, CIM_UINT16, CIM_UINT32, CIM_UINT64,
            WbemClassObject,
        };
        let class: IWbemClassObject =
            unsafe { CoCreateInstance(&WbemClassObject, None, CLSCTX_INPROC_SERVER) }.unwrap();
        let name = OwnedVariant(VARIANT::from(BSTR::from("NativeFixture")));
        // SAFETY: live in-memory class; Put copies initialized inputs.
        unsafe { class.Put(PCWSTR(wide("__CLASS").as_ptr()), 0, &name.0, 0) }.unwrap();
        let mut absent = VARIANT::default();
        // SAFETY: null tag has no allocated union arm.
        unsafe {
            (*absent.Anonymous.Anonymous).vt = VT_NULL;
        }
        for (name, kind) in [
            ("Large", CIM_UINT64.0),
            ("Small", CIM_UINT32.0),
            ("Flag", CIM_BOOLEAN.0),
            ("Empty", CIM_UINT16.0),
            ("Text", CIM_STRING.0),
            ("Paths", CIM_STRING.0 | CIM_FLAG_ARRAY.0),
        ] {
            // SAFETY: create typed fixture properties with absent values.
            unsafe { class.Put(PCWSTR(wide(name).as_ptr()), 0, &absent, kind) }.unwrap();
        }
        // SAFETY: complete in-memory class definition; returned instance owns ref.
        let object = Object(unsafe { class.SpawnInstance(0) }.unwrap());
        object
            .set("Large", VARIANT::from(BSTR::from(u64::MAX.to_string())))
            .unwrap();
        object.set("Small", VARIANT::from(-1i32)).unwrap();
        object.set("Flag", VARIANT::from(true)).unwrap();
        object
            .set("Text", VARIANT::from(BSTR::from("plain")))
            .unwrap();
        assert_eq!(object.number("Large").unwrap(), u64::MAX);
        assert_eq!(object.number("Small").unwrap(), u64::from(u32::MAX));
        assert!(object.boolean("Flag").unwrap());
        assert!(object.number("Text").is_err());
        assert!(object.boolean("Small").is_err());
        assert_eq!(object.optional_number("Empty").unwrap(), None);
        assert!(object.optional_number("Text").is_err());
        assert!(object.set_number("Empty", 31).is_err());
        object.set("Empty", VARIANT::from(31i32)).unwrap();
        assert_eq!(object.number("Empty").unwrap(), 31);
        assert_eq!(object.optional_number("Empty").unwrap(), Some(31));
        object.set_number("Large", u64::MAX - 1).unwrap();
        object.set_number("Small", 17).unwrap();
        object.set("Flag", VARIANT::from(false)).unwrap();
        let paths = vec![r"C:\space name\file".into(), "unicode-λ".into()];
        object.set_strings("Paths", &paths).unwrap();
        assert_eq!(object.strings("Paths").unwrap(), paths);
        assert_eq!(object.number("Large").unwrap(), u64::MAX - 1);
        assert_eq!(object.number("Small").unwrap(), 17);
        assert!(!object.boolean("Flag").unwrap());
        object.set_strings("Paths", &[]).unwrap();
        assert!(object.strings("Paths").unwrap().is_empty());
        assert!(object.set_number("Small", u64::MAX).is_err());
        assert!(object.xml().unwrap().contains("NativeFixture"));
    }
}
