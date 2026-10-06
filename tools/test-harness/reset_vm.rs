// Development-only disposable VM automation; never compiled by default.
fn reset_child(p: &ProjectConfiguration, s: State, hash: &str) -> Result<String> {
    if s.state != "Off" || !s.adapters.is_empty() {
        return Err("reset requires Off VM and no GPU adapters".into());
    }
    let fresh = State::read(p, Duration::from_secs(30), false, true)?;
    if fresh.state != "Off" || !fresh.adapters.is_empty() || fresh.drives.len() != s.drives.len() {
        return Err("reset preimage changed".into());
    }
    let s = &fresh;
    // Retain the validated attachment definition for recovery and reattachment.
    let drive = if let Some(d) = s.drives.first() {
        d.clone()
    } else {
        let all = s
            .session
            .related(&s.settings, "Msvm_ResourceAllocationSettingData")?;
        let mut disks = all
            .iter()
            .cloned()
            .filter_map(|d| match d.string("ResourceSubType") {
                Ok(t) if t == "Microsoft:Hyper-V:Synthetic Disk Drive" => Some(Ok(d)),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<Vec<_>>>()?;
        if disks.is_empty() {
            // Legacy reset can leave both storage and drive resources absent.
            // Recover only port 0 on the enrolled VM's existing SCSI controller 0.
            let controller = one(all
                .into_iter()
                .filter_map(|o| match o.string("ResourceSubType") {
                    Ok(t) if t == "Microsoft:Hyper-V:Synthetic SCSI Controller" => Some(Ok(o)),
                    Ok(_) => None,
                    Err(e) => Some(Err(e)),
                })
                .collect::<Result<Vec<_>>>()?)?;
            if !controller.string("InstanceID")?.ends_with("\\0") {
                return Err("reset requires existing SCSI controller 0".into());
            }
            let disk = s.session.template("Msvm_ResourceAllocationSettingData")?;
            disk.set("ResourceType", VARIANT::from(17i32))?;
            disk.set(
                "ResourceSubType",
                VARIANT::from(BSTR::from("Microsoft:Hyper-V:Synthetic Disk Drive")),
            )?;
            disk.set("Parent", VARIANT::from(BSTR::from(controller.path()?)))?;
            disk.set("AddressOnParent", VARIANT::from(BSTR::from("0")))?;
            s.session.invoke(
                &s.service,
                "AddResourceSettings",
                vec![
                    (
                        "AffectedConfiguration",
                        VARIANT::from(BSTR::from(s.settings.path()?)),
                    ),
                    ("ResourceSettings", string_array(&[disk.xml()?])?),
                ],
            )?;
            disks = s
                .session
                .related(&s.settings, "Msvm_ResourceAllocationSettingData")?
                .into_iter()
                .filter_map(|o| match o.string("ResourceSubType") {
                    Ok(t) if t == "Microsoft:Hyper-V:Synthetic Disk Drive" => Some(Ok(o)),
                    Ok(_) => None,
                    Err(e) => Some(Err(e)),
                })
                .collect::<Result<Vec<_>>>()?;
        }
        let disk = one(disks)?;
        if disk.string("AddressOnParent")? != "0" {
            return Err("reset disk port mismatch".into());
        }
        let t = s.session.template("Msvm_StorageAllocationSettingData")?;
        t.set("ResourceType", VARIANT::from(31i32))?;
        t.set(
            "ResourceSubType",
            VARIANT::from(BSTR::from("Microsoft:Hyper-V:Virtual Hard Disk")),
        )?;
        t.set("Parent", VARIANT::from(BSTR::from(disk.path()?)))?;
        t.set_strings(
            "HostResource",
            &[p.slot.child_path.to_string_lossy().into_owned()],
        )?;
        t
    };
    let xml = drive.xml()?;
    if let Some(d) = s.drives.first() {
        s.session.invoke(
            &s.service,
            "RemoveResourceSettings",
            vec![("ResourceSettings", string_array(&[d.path()?])?)],
        )?;
    }
    recreate_child(p)?;
    s.session.invoke(
        &s.service,
        "AddResourceSettings",
        vec![
            (
                "AffectedConfiguration",
                VARIANT::from(BSTR::from(s.settings.path()?)),
            ),
            ("ResourceSettings", string_array(&[xml])?),
        ],
    )?;
    let fresh = State::read(p, Duration::from_secs(30), false, false)?;
    verify_child(p, false)?;
    if fresh.state != "Off" || !fresh.adapters.is_empty() {
        return Err("reset independent readback failed".into());
    }
    let policy = embedded_policy().map_err(|e| e.to_string())?;
    Ok(format!(
        "status\tok\nvm_id\t{}\nchild\t{}\nparent\t{}\nparent_sha256\t{hash}\n",
        p.slot.vm_id,
        policy.child(),
        policy.parent()
    ))
}

