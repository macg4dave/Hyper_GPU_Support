// Development-only disposable VM automation; never compiled by default.
#[allow(unsafe_code)]
pub(crate) fn recreate_child(project: &ProjectConfiguration) -> Result<()> {
    verify_child(project, true)?;
    if project
        .slot
        .child_path
        .try_exists()
        .map_err(|e| e.to_string())?
    {
        std::fs::remove_file(&project.slot.child_path)
            .map_err(|e| format!("delete verified disposable child: {e}"))?;
    }
    no_reparse(&project.slot.child_path, true)?;
    let storage = VIRTUAL_STORAGE_TYPE {
        DeviceId: VIRTUAL_STORAGE_TYPE_DEVICE_VHDX,
        VendorId: VIRTUAL_STORAGE_TYPE_VENDOR_MICROSOFT,
    };
    let parent = wide(&project.slot.parent_path);
    let params = CREATE_VIRTUAL_DISK_PARAMETERS {
        Version: CREATE_VIRTUAL_DISK_VERSION_2,
        Anonymous: CREATE_VIRTUAL_DISK_PARAMETERS_0 {
            Version2: CREATE_VIRTUAL_DISK_PARAMETERS_0_1 {
                ParentPath: PCWSTR(parent.as_ptr()),
                ParentVirtualStorageType: storage,
                ..Default::default()
            },
        },
    };
    let mut handle = HANDLE::default();
    // SAFETY: version 2 parameters create only the validated configured differencing
    // child. Parent path is pinned and held protected by the caller's ParentGuard.
    unsafe {
        CreateVirtualDisk(
            &storage,
            PCWSTR(wide(&project.slot.child_path).as_ptr()),
            VIRTUAL_DISK_ACCESS_NONE,
            None,
            CREATE_VIRTUAL_DISK_FLAG_NONE,
            0,
            &params,
            None,
            &mut handle,
        )
    }
    .ok()
    .map_err(|e| format!("create differencing child: {e}"))?;
    let _disk = Disk(handle);
    verify_child(project, false)
}

