//! CUDA Driver API identity companion for the unchanged `vectorAddDrv` sample.

use std::ffi::{CStr, c_char};
use std::process::ExitCode;

use hyper_gpu_support::config::ProjectConfiguration;
use hyper_gpu_support::probe::ExitClass;
use hyper_gpu_support::windows_probe::select_configured_gpu;
use serde::Serialize;
use windows::Win32::Foundation::{FARPROC, FreeLibrary, HMODULE};
use windows::Win32::System::LibraryLoader::{
    GetProcAddress, LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW,
};
use windows::core::{PCSTR, w};

type CuResult = i32;
type CuDevice = i32;
type CuInit = unsafe extern "system" fn(u32) -> CuResult;
type CuDeviceGetCount = unsafe extern "system" fn(*mut i32) -> CuResult;
type CuDeviceGet = unsafe extern "system" fn(*mut CuDevice, i32) -> CuResult;
type CuDeviceGetName = unsafe extern "system" fn(*mut c_char, i32, CuDevice) -> CuResult;
type CuDeviceComputeCapability =
    unsafe extern "system" fn(*mut i32, *mut i32, CuDevice) -> CuResult;
type CuDeviceGetLuid = unsafe extern "system" fn(*mut c_char, *mut u32, CuDevice) -> CuResult;
type CuDeviceGetUuid = unsafe extern "system" fn(*mut CuUuid, CuDevice) -> CuResult;
type CuDriverGetVersion = unsafe extern "system" fn(*mut i32) -> CuResult;

const CUDA_SUCCESS: CuResult = 0;

#[repr(C)]
struct CuUuid {
    bytes: [u8; 16],
}

struct CudaApi {
    _library: CudaLibrary,
    init: CuInit,
    device_get_count: CuDeviceGetCount,
    device_get: CuDeviceGet,
    device_get_name: CuDeviceGetName,
    device_compute_capability: CuDeviceComputeCapability,
    device_get_luid: CuDeviceGetLuid,
    device_get_uuid: CuDeviceGetUuid,
    driver_get_version: CuDriverGetVersion,
}

struct CudaLibrary(HMODULE);

#[allow(unsafe_code)]
impl Drop for CudaLibrary {
    fn drop(&mut self) {
        // SAFETY: `module` is the successful LoadLibraryExW result owned by this
        // object and is released exactly once after the function pointers expire.
        let _ = unsafe { FreeLibrary(self.0) };
    }
}

#[derive(Serialize)]
struct IdentityReport {
    schema: u32,
    probe: &'static str,
    status: &'static str,
    ordinal: i32,
    device_count: i32,
    name: String,
    uuid: String,
    compute_capability: String,
    luid: String,
    device_node_mask: u32,
    driver_version: i32,
    dxgi_luid: String,
}

struct Failure {
    class: ExitClass,
    message: String,
}

impl Failure {
    fn new(class: ExitClass, message: impl Into<String>) -> Self {
        Self {
            class,
            message: message.into(),
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(report) => match serde_json::to_string(&report) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("failed to serialize CUDA identity: {error}");
                ExitCode::from(ExitClass::Internal.code())
            }
        },
        Err(failure) => {
            eprintln!("{}", failure.message);
            ExitCode::from(failure.class.code())
        }
    }
}

#[allow(unsafe_code)]
fn run() -> Result<IdentityReport, Failure> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let guest_compute = match arguments.as_slice() {
        [] => false,
        [mode] if mode == "--guest-compute" => true,
        _ => {
            return Err(Failure::new(
                ExitClass::Internal,
                "usage: cuda-identity [--guest-compute]",
            ));
        }
    };
    let project = ProjectConfiguration::embedded().map_err(|error| {
        Failure::new(
            ExitClass::Adapter,
            format!("project GPU configuration is invalid: {error}"),
        )
    })?;
    let api = CudaApi::load()?;
    cuda_call(unsafe { (api.init)(0) }, "cuInit")?;

    let mut count = 0;
    cuda_call(
        unsafe { (api.device_get_count)(&mut count) },
        "cuDeviceGetCount",
    )?;
    if count < 1 {
        return Err(Failure::new(
            ExitClass::Adapter,
            "CUDA device 0 is unavailable",
        ));
    }

    let ordinal = 0;
    let mut device = 0;
    cuda_call(
        unsafe { (api.device_get)(&mut device, ordinal) },
        "cuDeviceGet",
    )?;
    let mut name_bytes = [0i8; 256];
    cuda_call(
        unsafe {
            (api.device_get_name)(
                name_bytes.as_mut_ptr(),
                i32::try_from(name_bytes.len()).expect("fixed name buffer fits i32"),
                device,
            )
        },
        "cuDeviceGetName",
    )?;
    // SAFETY: the driver writes a NUL-terminated string on success into the
    // fixed buffer passed above.
    let name = unsafe { CStr::from_ptr(name_bytes.as_ptr()) }
        .to_str()
        .map_err(|_| Failure::new(ExitClass::Adapter, "CUDA device name is not UTF-8"))?
        .to_owned();

    let mut major = 0;
    let mut minor = 0;
    cuda_call(
        unsafe { (api.device_compute_capability)(&mut major, &mut minor, device) },
        "cuDeviceComputeCapability",
    )?;
    let mut luid_bytes = [0i8; 8];
    let mut node_mask = 0;
    cuda_call(
        unsafe { (api.device_get_luid)(luid_bytes.as_mut_ptr(), &mut node_mask, device) },
        "cuDeviceGetLuid",
    )?;
    let luid = format_luid(luid_bytes.map(|byte| byte as u8));

    let mut uuid = CuUuid { bytes: [0; 16] };
    cuda_call(
        unsafe { (api.device_get_uuid)(&mut uuid, device) },
        "cuDeviceGetUuid_v2",
    )?;
    let mut driver_version = 0;
    cuda_call(
        unsafe { (api.driver_get_version)(&mut driver_version) },
        "cuDriverGetVersion",
    )?;

    let selected = select_configured_gpu().map_err(|error| {
        Failure::new(
            ExitClass::Adapter,
            format!("physical DXGI adapter validation failed: {error}"),
        )
    })?;
    let adapter = selected.identity();
    if name != project.slot.gpu_name
        || (major, minor)
            != (
                project.slot.cuda_compute_capability_major,
                project.slot.cuda_compute_capability_minor,
            )
        || (!guest_compute && (luid == "00000000:00000000" || luid != adapter.luid))
        || (guest_compute && (count != 1 || !adapter.paravirtualized))
    {
        return Err(Failure::new(
            ExitClass::Adapter,
            format!(
                "CUDA identity mismatch: name={name:?}, capability={major}.{minor}, luid={luid}, dxgi_luid={}",
                adapter.luid
            ),
        ));
    }

    Ok(IdentityReport {
        schema: 1,
        probe: "cuda-identity",
        status: "pass",
        ordinal,
        device_count: count,
        name,
        uuid: format_uuid(uuid.bytes),
        compute_capability: format!("{major}.{minor}"),
        luid,
        device_node_mask: node_mask,
        driver_version,
        dxgi_luid: adapter.luid.clone(),
    })
}

fn cuda_call(result: CuResult, operation: &str) -> Result<(), Failure> {
    if result == CUDA_SUCCESS {
        Ok(())
    } else {
        Err(Failure::new(
            ExitClass::Execution,
            format!("{operation} failed with CUresult {result}"),
        ))
    }
}

fn format_luid(bytes: [u8; 8]) -> String {
    let low = u32::from_ne_bytes(bytes[0..4].try_into().expect("fixed LUID low part"));
    let high = u32::from_ne_bytes(bytes[4..8].try_into().expect("fixed LUID high part"));
    format!("{high:08x}:{low:08x}")
}

fn format_uuid(bytes: [u8; 16]) -> String {
    bytes
        .iter()
        .enumerate()
        .map(|(index, byte)| {
            let separator = if matches!(index, 4 | 6 | 8 | 10) {
                "-"
            } else {
                ""
            };
            format!("{separator}{byte:02x}")
        })
        .collect()
}

#[allow(unsafe_code)]
impl CudaApi {
    fn load() -> Result<Self, Failure> {
        // SAFETY: search is restricted to System32, which is the supported
        // location for the installed NVIDIA driver API DLL.
        let module =
            unsafe { LoadLibraryExW(w!("nvcuda.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32) }
                .map_err(|_| Failure::new(ExitClass::Runtime, "nvcuda.dll is unavailable"))?;
        let library = CudaLibrary(module);

        macro_rules! symbol {
            ($name:literal, $kind:ty) => {{
                let address =
                    unsafe { GetProcAddress(module, PCSTR(concat!($name, "\0").as_ptr())) };
                let address = require_symbol(address, $name)?;
                // SAFETY: NVIDIA's published CUDA Driver API fixes each named
                // symbol to the declared Win64 ABI and signature.
                unsafe {
                    std::mem::transmute::<unsafe extern "system" fn() -> isize, $kind>(address)
                }
            }};
        }

        let loaded = Self {
            _library: library,
            init: symbol!("cuInit", CuInit),
            device_get_count: symbol!("cuDeviceGetCount", CuDeviceGetCount),
            device_get: symbol!("cuDeviceGet", CuDeviceGet),
            device_get_name: symbol!("cuDeviceGetName", CuDeviceGetName),
            device_compute_capability: symbol!(
                "cuDeviceComputeCapability",
                CuDeviceComputeCapability
            ),
            device_get_luid: symbol!("cuDeviceGetLuid", CuDeviceGetLuid),
            device_get_uuid: symbol!("cuDeviceGetUuid_v2", CuDeviceGetUuid),
            driver_get_version: symbol!("cuDriverGetVersion", CuDriverGetVersion),
        };
        Ok(loaded)
    }
}

fn require_symbol(
    address: FARPROC,
    name: &str,
) -> Result<unsafe extern "system" fn() -> isize, Failure> {
    address.ok_or_else(|| {
        Failure::new(
            ExitClass::Runtime,
            format!("CUDA Driver API symbol is unavailable: {name}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{format_luid, format_uuid};

    #[test]
    fn formats_windows_luid_in_dxgi_order() {
        assert_eq!(
            format_luid([0xbe, 0x49, 0x01, 0, 0, 0, 0, 0]),
            "00000000:000149be"
        );
    }

    #[test]
    fn formats_uuid_with_canonical_grouping() {
        assert_eq!(
            format_uuid([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
            "00010203-0405-0607-0809-0a0b0c0d0e0f"
        );
    }
}
