//! Local COM/WMI query primitives.
use std::{collections::BTreeMap, fmt, time::Instant};
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
            WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY, WbemLocator,
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
        operation: String,
        /// Original HRESULT without apartment-bound COM error information.
        source: windows::core::Error,
        /// Native message copied before the calling COM apartment is released.
        message: String,
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
            Self::Native {
                operation,
                source,
                message,
            } => {
                write!(f, "{operation}: {}: {message}", source.code())
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
pub(crate) fn native(
    operation: impl Into<String>,
    source: windows::core::Error,
) -> DriverDiscoveryError {
    // windows::core::Error can own IErrorInfo. Copy its message and release that
    // object while this thread's COM apartment is still alive; an error returned
    // to main must not call/release an apartment-bound proxy after CoUninitialize.
    let message = source.message();
    DriverDiscoveryError::Native {
        operation: operation.into(),
        source: windows::core::Error::from_hresult(source.code()),
        message,
    }
}

pub(crate) struct Apartment;
#[allow(unsafe_code)]
impl Apartment {
    pub(crate) fn initialize() -> Result<Self, DriverDiscoveryError> {
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
pub(crate) fn connect(namespace: &str) -> Result<IWbemServices, DriverDiscoveryError> {
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
pub(crate) fn property(
    object: &IWbemClassObject,
    name: &str,
) -> Result<String, DriverDiscoveryError> {
    let mut value = VARIANT::default();
    let wide = name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    // SAFETY: live object, NUL-terminated property name and initialized VARIANT output.
    let get = unsafe { object.Get(PCWSTR(wide.as_ptr()), 0, &mut value, None, None) };
    let result = if let Err(error) = get {
        Err(native("read WMI property", error))
    } else {
        // SAFETY: Get initialized the VARIANT; inspect its tag before its BSTR arm.
        let inner = unsafe { &value.Anonymous.Anonymous };
        if matches!(name, "ConfigManagerErrorCode" | "EnabledState")
            && inner.vt == windows::Win32::System::Variant::VT_I4
        {
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
pub(crate) fn query(
    services: &IWbemServices,
    wql: &str,
    properties: &[&str],
    deadline: Instant,
) -> Result<Vec<BTreeMap<String, String>>, DriverDiscoveryError> {
    let mut trace = crate::diagnostics::Span::start("driver-wmi-query", wql)
        .map_err(|_| DriverDiscoveryError::Invalid("diagnostic logging unavailable"))?;
    let result = (|| {
        // SAFETY: services is live; query BSTRs remain alive until the call returns.
        let enumerator = unsafe {
            services.ExecQuery(
                &BSTR::from("WQL"),
                &BSTR::from(wql),
                // Only named data properties are consumed. Do not ask providers to
                // synthesize system paths for projected objects with null key fields.
                WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
                None,
            )
        }
        .map_err(|error| native(format!("execute driver WMI query ({wql})"), error))?;
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
                .map_err(|error| native(format!("enumerate driver WMI query ({wql})"), error))?;
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
    })();
    trace
        .finish(&result)
        .map_err(|_| DriverDiscoveryError::Invalid("diagnostic logging unavailable"))?;
    result
}

pub(crate) fn quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}
