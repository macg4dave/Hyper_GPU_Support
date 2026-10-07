//! Windows DXGI adapter selection for standalone GPU probes.

use std::fmt;

use windows::Wdk::Graphics::Direct3D::{
    D3DKMT_ADAPTERTYPE, D3DKMT_CLOSEADAPTER, D3DKMT_OPENADAPTERFROMLUID,
    D3DKMT_PHYSICAL_ADAPTER_COUNT, D3DKMT_QUERY_DEVICE_IDS, D3DKMT_QUERYADAPTERINFO,
    D3DKMTCloseAdapter, D3DKMTOpenAdapterFromLuid, D3DKMTQueryAdapterInfo, KMTQAITYPE_ADAPTERTYPE,
    KMTQAITYPE_PHYSICALADAPTERCOUNT, KMTQAITYPE_PHYSICALADAPTERDEVICEIDS,
};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_ERROR_NOT_FOUND, IDXGIAdapter1,
    IDXGIFactory1,
};

use crate::config::ProjectConfiguration;
use crate::probe::AdapterIdentity;

/// One unambiguous hardware adapter selected by exact PCI identity, using
/// D3DKMT physical device IDs when GPU-PV's DXGI descriptor omits host fields.
#[derive(Clone)]
pub struct SelectedAdapter {
    adapter: IDXGIAdapter1,
    identity: AdapterIdentity,
}

impl SelectedAdapter {
    /// Borrow the selected DXGI adapter for device creation.
    #[must_use]
    pub fn adapter(&self) -> &IDXGIAdapter1 {
        &self.adapter
    }

    /// Return the checked serializable adapter identity.
    #[must_use]
    pub fn identity(&self) -> &AdapterIdentity {
        &self.identity
    }
}

/// Failure to obtain exactly one configured hardware adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterSelectionError {
    /// DXGI factory/enumeration failed unexpectedly.
    Runtime,
    /// A D3DKMT operation failed with the preserved NTSTATUS value.
    Native(i32),
    /// No matching hardware adapter exists.
    Missing,
    /// More than one matching adapter exists, so selection is ambiguous.
    Ambiguous,
}

impl fmt::Display for AdapterSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Runtime => formatter.write_str("DXGI adapter enumeration failed"),
            Self::Native(status) => {
                write!(formatter, "D3DKMT adapter query failed: {status:#010x}")
            }
            Self::Missing => formatter.write_str("configured hardware adapter unavailable"),
            Self::Ambiguous => formatter.write_str("configured adapter selection is ambiguous"),
        }
    }
}

impl std::error::Error for AdapterSelectionError {}

/// Select exactly one non-software adapter matching `config/project.toml`.
///
/// # Errors
/// Returns an explicit missing/ambiguous/runtime classification. Enumeration order
/// is never used as a fallback.
#[allow(unsafe_code)]
pub fn select_configured_gpu() -> Result<SelectedAdapter, AdapterSelectionError> {
    let project = ProjectConfiguration::embedded().map_err(|_| AdapterSelectionError::Runtime)?;
    let vendor_id = project.slot.gpu_vendor_id;
    let device_id = project.slot.gpu_device_id;
    // SAFETY: `CreateDXGIFactory1` initializes and returns an owned COM interface;
    // the windows crate manages its reference count.
    let factory: IDXGIFactory1 =
        unsafe { CreateDXGIFactory1() }.map_err(|_| AdapterSelectionError::Runtime)?;
    let mut candidates = Vec::new();
    let mut index = 0;
    loop {
        // SAFETY: `factory` is a live COM interface and `index` is bounded by the
        // provider's `DXGI_ERROR_NOT_FOUND` terminator.
        let adapter = match unsafe { factory.EnumAdapters1(index) } {
            Ok(adapter) => adapter,
            Err(error) if error.code() == DXGI_ERROR_NOT_FOUND => break,
            Err(_) => return Err(AdapterSelectionError::Runtime),
        };
        index = index.checked_add(1).ok_or(AdapterSelectionError::Runtime)?;
        // SAFETY: the returned adapter interface is live for this call and the
        // method returns the descriptor by value.
        let description =
            unsafe { adapter.GetDesc1() }.map_err(|_| AdapterSelectionError::Runtime)?;
        let software = description.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0;
        if description.VendorId != vendor_id || description.DeviceId != device_id || software {
            continue;
        }
        let adapter_type = query_adapter_type(description.AdapterLuid)?;
        if adapter_type & (1 << 6) != 0 {
            continue;
        }
        let end = description
            .Description
            .iter()
            .position(|character| *character == 0)
            .unwrap_or(description.Description.len());
        let mut identity = AdapterIdentity {
            description: String::from_utf16_lossy(&description.Description[..end]),
            vendor_id: description.VendorId,
            device_id: description.DeviceId,
            subsystem_id: description.SubSysId,
            revision: description.Revision,
            dedicated_video_memory: description.DedicatedVideoMemory as u64,
            luid: format!(
                "{:08x}:{:08x}",
                description.AdapterLuid.HighPart as u32, description.AdapterLuid.LowPart
            ),
            software,
            indirect_display: adapter_type & (1 << 6) != 0,
            paravirtualized: adapter_type & (1 << 7) != 0,
        };
        if identity.paravirtualized {
            // VRD's DXGI descriptor omits subsystem/revision on the measured guest.
            // Query its host-backed physical identity; never fill gaps from config.
            let ids = query_physical_device_ids(description.AdapterLuid)?;
            identity.vendor_id = ids.VendorID;
            identity.device_id = ids.DeviceID;
            identity.subsystem_id = packed_subsystem_id(ids.SubVendorID, ids.SubSystemID)?;
            identity.revision = ids.RevisionID;
        }
        candidates.push(SelectedAdapter { adapter, identity });
    }
    let identities = candidates
        .iter()
        .map(|candidate| candidate.identity.clone())
        .collect::<Vec<_>>();
    let selected = select_identity_index(&identities)?;
    Ok(candidates.swap_remove(selected))
}

fn select_identity_index(identities: &[AdapterIdentity]) -> Result<usize, AdapterSelectionError> {
    // Windows can expose a display-paired proxy plus the actual render partition.
    // When a partition exists, select only that namespace and still require every
    // configured physical identity field. Never choose the first matching name.
    let partition_present = identities.iter().any(|identity| {
        identity.paravirtualized && !identity.software && !identity.indirect_display
    });
    let mut selected = None;
    for (index, identity) in identities.iter().enumerate() {
        if identity.paravirtualized != partition_present || identity.validate().is_err() {
            continue;
        }
        if selected.replace(index).is_some() {
            return Err(AdapterSelectionError::Ambiguous);
        }
    }
    selected.ok_or(AdapterSelectionError::Missing)
}

fn packed_subsystem_id(subvendor: u32, subsystem: u32) -> Result<u32, AdapterSelectionError> {
    if subvendor > u32::from(u16::MAX) || subsystem > u32::from(u16::MAX) {
        return Err(AdapterSelectionError::Runtime);
    }
    Ok((subsystem << 16) | subvendor)
}

/// Enumerate DXGI adapter identities for read-only failure diagnosis.
///
/// # Errors
/// Returns [`AdapterSelectionError::Runtime`] when DXGI cannot complete a
/// trustworthy enumeration.
#[allow(unsafe_code)]
pub fn enumerate_adapter_identities() -> Result<Vec<AdapterIdentity>, AdapterSelectionError> {
    // SAFETY: `CreateDXGIFactory1` returns an owned COM interface managed by the
    // windows crate.
    let factory: IDXGIFactory1 =
        unsafe { CreateDXGIFactory1() }.map_err(|_| AdapterSelectionError::Runtime)?;
    let mut identities = Vec::new();
    let mut index = 0_u32;
    loop {
        // SAFETY: `factory` is live and enumeration ends on the documented
        // `DXGI_ERROR_NOT_FOUND` result.
        let adapter = match unsafe { factory.EnumAdapters1(index) } {
            Ok(adapter) => adapter,
            Err(error) if error.code() == DXGI_ERROR_NOT_FOUND => break,
            Err(_) => return Err(AdapterSelectionError::Runtime),
        };
        index = index.checked_add(1).ok_or(AdapterSelectionError::Runtime)?;
        // SAFETY: the adapter interface is live and the descriptor is returned by
        // value without caller-owned buffers.
        let description =
            unsafe { adapter.GetDesc1() }.map_err(|_| AdapterSelectionError::Runtime)?;
        let end = description
            .Description
            .iter()
            .position(|character| *character == 0)
            .unwrap_or(description.Description.len());
        let adapter_type = query_adapter_type(description.AdapterLuid)?;
        identities.push(AdapterIdentity {
            description: String::from_utf16_lossy(&description.Description[..end]),
            vendor_id: description.VendorId,
            device_id: description.DeviceId,
            subsystem_id: description.SubSysId,
            revision: description.Revision,
            dedicated_video_memory: description.DedicatedVideoMemory as u64,
            luid: format!(
                "{:08x}:{:08x}",
                description.AdapterLuid.HighPart as u32, description.AdapterLuid.LowPart
            ),
            software: description.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0,
            indirect_display: adapter_type & (1 << 6) != 0,
            paravirtualized: adapter_type & (1 << 7) != 0,
        });
    }
    Ok(identities)
}

#[allow(unsafe_code)]
fn query_adapter_type(
    luid: windows::Win32::Foundation::LUID,
) -> Result<u32, AdapterSelectionError> {
    let mut opened = D3DKMT_OPENADAPTERFROMLUID {
        AdapterLuid: luid,
        hAdapter: 0,
    };
    // SAFETY: `opened` is initialized writable storage and the API fills its
    // adapter handle for the exact LUID.
    let open_status = unsafe { D3DKMTOpenAdapterFromLuid(&mut opened) };
    if open_status.0 < 0 || opened.hAdapter == 0 {
        return Err(AdapterSelectionError::Native(open_status.0));
    }
    let mut adapter_type = D3DKMT_ADAPTERTYPE::default();
    let mut query = D3DKMT_QUERYADAPTERINFO {
        hAdapter: opened.hAdapter,
        Type: KMTQAITYPE_ADAPTERTYPE,
        pPrivateDriverData: std::ptr::addr_of_mut!(adapter_type).cast(),
        PrivateDriverDataSize: std::mem::size_of::<D3DKMT_ADAPTERTYPE>() as u32,
    };
    // SAFETY: the query points to writable storage of the exact advertised size,
    // and the handle was opened above. No pointer escapes the call.
    let query_status = unsafe { D3DKMTQueryAdapterInfo(&mut query) };
    let close = D3DKMT_CLOSEADAPTER {
        hAdapter: opened.hAdapter,
    };
    // SAFETY: closes exactly the handle returned by `D3DKMTOpenAdapterFromLuid`.
    let close_status = unsafe { D3DKMTCloseAdapter(&close) };
    if query_status.0 < 0 {
        return Err(AdapterSelectionError::Native(query_status.0));
    }
    if close_status.0 < 0 {
        return Err(AdapterSelectionError::Native(close_status.0));
    }
    // SAFETY: the successful query initialized the union and `Value` is the
    // documented complete bitfield representation.
    Ok(unsafe { adapter_type.Anonymous.Value })
}

/// Query index zero only after verifying that the adapter represents one physical
/// GPU. Both native failure and linked-adapter ambiguity fail closed.
#[allow(unsafe_code)]
fn query_physical_device_ids(
    luid: windows::Win32::Foundation::LUID,
) -> Result<windows::Wdk::Graphics::Direct3D::D3DKMT_DEVICE_IDS, AdapterSelectionError> {
    let mut opened = D3DKMT_OPENADAPTERFROMLUID {
        AdapterLuid: luid,
        hAdapter: 0,
    };
    // SAFETY: initialized output storage; the exact LUID is supplied by DXGI.
    let open_status = unsafe { D3DKMTOpenAdapterFromLuid(&mut opened) };
    if open_status.0 < 0 || opened.hAdapter == 0 {
        return Err(AdapterSelectionError::Native(open_status.0));
    }
    let mut count = D3DKMT_PHYSICAL_ADAPTER_COUNT::default();
    let mut query = D3DKMT_QUERYADAPTERINFO {
        hAdapter: opened.hAdapter,
        Type: KMTQAITYPE_PHYSICALADAPTERCOUNT,
        pPrivateDriverData: std::ptr::addr_of_mut!(count).cast(),
        PrivateDriverDataSize: std::mem::size_of::<D3DKMT_PHYSICAL_ADAPTER_COUNT>() as u32,
    };
    // SAFETY: live handle, initialized count buffer, and its exact ABI size/type.
    let count_status = unsafe { D3DKMTQueryAdapterInfo(&mut query) };
    let mut ids = D3DKMT_QUERY_DEVICE_IDS::default();
    let result = if count_status.0 < 0 {
        Err(AdapterSelectionError::Native(count_status.0))
    } else if count.Count != 1 {
        Err(AdapterSelectionError::Ambiguous)
    } else {
        query.Type = KMTQAITYPE_PHYSICALADAPTERDEVICEIDS;
        query.pPrivateDriverData = std::ptr::addr_of_mut!(ids).cast();
        query.PrivateDriverDataSize = std::mem::size_of::<D3DKMT_QUERY_DEVICE_IDS>() as u32;
        // SAFETY: index zero is valid for the verified single physical adapter;
        // `ids` has the initialized repr(C) layout and exact advertised size.
        let status = unsafe { D3DKMTQueryAdapterInfo(&mut query) };
        if status.0 < 0 {
            Err(AdapterSelectionError::Native(status.0))
        } else {
            Ok(ids.DeviceIds)
        }
    };
    let close = D3DKMT_CLOSEADAPTER {
        hAdapter: opened.hAdapter,
    };
    // SAFETY: release the owned adapter handle on every query outcome.
    let close_status = unsafe { D3DKMTCloseAdapter(&close) };
    if result.is_ok() && close_status.0 < 0 {
        return Err(AdapterSelectionError::Native(close_status.0));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{AdapterSelectionError, packed_subsystem_id, select_identity_index};
    use crate::{config::ProjectConfiguration, probe::AdapterIdentity};

    fn target(partition: bool) -> AdapterIdentity {
        let target = ProjectConfiguration::embedded().unwrap().slot;
        AdapterIdentity {
            description: target.gpu_name,
            vendor_id: target.gpu_vendor_id,
            device_id: target.gpu_device_id,
            subsystem_id: target.gpu_subsystem_id,
            revision: target.gpu_revision,
            dedicated_video_memory: 1,
            luid: "00000000:00000001".into(),
            software: false,
            indirect_display: false,
            paravirtualized: partition,
        }
    }

    #[test]
    fn selects_exact_host_or_partition_without_display_proxy_fallback() {
        assert_eq!(select_identity_index(&[target(false)]), Ok(0));
        let mut proxy = target(false);
        proxy.subsystem_id = 0;
        proxy.revision = 0;
        assert_eq!(select_identity_index(&[proxy, target(true)]), Ok(1));
        let mut wrong_partition = target(true);
        wrong_partition.revision ^= 1;
        assert_eq!(
            select_identity_index(&[target(false), wrong_partition]),
            Err(AdapterSelectionError::Missing)
        );
    }

    #[test]
    fn rejects_ambiguous_or_nonhardware_candidates() {
        assert_eq!(
            select_identity_index(&[target(true), target(true)]),
            Err(AdapterSelectionError::Ambiguous)
        );
        for change in [0, 1, 2, 3] {
            let mut identity = target(true);
            match change {
                0 => identity.software = true,
                1 => identity.indirect_display = true,
                2 => identity.subsystem_id = 0,
                _ => identity.luid = "00000000:00000000".into(),
            }
            assert_eq!(
                select_identity_index(&[identity]),
                Err(AdapterSelectionError::Missing)
            );
        }
    }

    #[test]
    fn combines_native_subsystem_fields_without_truncation() {
        assert_eq!(packed_subsystem_id(0x1043, 0x8a15), Ok(0x8a15_1043));
        assert!(packed_subsystem_id(0x10000, 0).is_err());
        assert!(packed_subsystem_id(0, 0x10000).is_err());
    }
}
