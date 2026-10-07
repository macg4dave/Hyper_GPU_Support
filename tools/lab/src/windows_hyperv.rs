//! Native fixed-target Hyper-V inspection, settings and lifecycle operations.
//! Invoke only inside the controlled runner's contained native worker.

#[cfg(feature = "dev-harness")]
use crate::windows_hyperv_disk::recreate_child;
use crate::{
    config::{ProjectConfiguration, VmProfile},
    runner::{Operation, embedded_policy},
    vm_settings::{GpuResources, SettingsSnapshot, Triple},
    windows_driver_environment::quoted,
    windows_hyperv_disk::{DirectoryGuard, ParentGuard, verify_child},
    windows_hyperv_wmi::{Object, Session, one, string_array},
};
use std::{
    path::Path,
    time::{Duration, Instant},
};
use windows::{Win32::System::Variant::VARIANT, core::BSTR};

type Result<T> = std::result::Result<T, String>;

/// The fixed worker operation. No operator-provided query or target is accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeAction {
    /// Full target/chain/parent/GPU inspection.
    Inspect,
    /// Inspection allowing the repairable automatic-checkpoint flag.
    SettingsInspect,
    /// Read the full profile from fresh native provider objects.
    ReadProfile,
    /// Apply the desired profile after checking the protected expected preimage.
    ApplySettings,
    /// Attach only the configured GPU to the configured Off VM.
    Attach,
    /// Remove only the exact configured GPU adapter.
    Remove,
    /// Start the enrolled Off guest.
    Start,
    /// Graceful guest shutdown through its integration component.
    Shutdown,
    /// Reserved development reset; accepted only in explicit dev-harness builds.
    Reset,
}
impl NativeAction {
    /// Fixed executable worker mode.
    pub fn mode(self) -> &'static str {
        match self {
            Self::Inspect => "native-inspect",
            Self::SettingsInspect => "native-settings-inspect",
            Self::ReadProfile => "native-profile-read",
            Self::ApplySettings => "native-settings-apply",
            Self::Attach => "native-attach",
            Self::Remove => "native-remove",
            Self::Start => "native-start",
            Self::Shutdown => "native-shutdown",
            Self::Reset => "native-reset",
        }
    }
    /// Parse only the fixed worker modes.
    pub fn parse(mode: &str) -> Option<Self> {
        [
            Self::Inspect,
            Self::SettingsInspect,
            Self::ReadProfile,
            Self::ApplySettings,
            Self::Attach,
            Self::Remove,
            Self::Start,
            Self::Shutdown,
            #[cfg(feature = "dev-harness")]
            Self::Reset,
        ]
        .into_iter()
        .find(|a| a.mode() == mode)
    }
    /// Mutation authorization must match the protected parent operation marker.
    pub fn operation(self) -> Option<Operation> {
        match self {
            Self::Inspect | Self::SettingsInspect | Self::ReadProfile => None,
            Self::ApplySettings => Some(Operation::ConfigureSlot),
            Self::Attach => Some(Operation::AssignGpu),
            Self::Remove => Some(Operation::RemoveGpu),
            Self::Start => Some(Operation::StartSlot),
            Self::Shutdown => Some(Operation::ShutdownSlot),
            Self::Reset => Some(Operation::ResetSlot),
        }
    }
}

struct State {
    vm: Object,
    settings: Object,
    service: Object,
    adapters: Vec<Object>,
    #[cfg(feature = "dev-harness")]
    drives: Vec<Object>,
    state: String,
    // Drop every provider proxy before Session balances this thread's COM init.
    session: Session,
}
impl State {
    fn read(
        p: &ProjectConfiguration,
        timeout: Duration,
        allow_auto: bool,
        reset: bool,
    ) -> Result<Self> {
        let session = Session::new(timeout)?;
        let vm = session.one(&format!(
            "SELECT * FROM Msvm_ComputerSystem WHERE Caption='Virtual Machine' AND Name='{}'",
            quoted(&p.slot.vm_id)
        ))?;
        if !vm.string("Name")?.eq_ignore_ascii_case(&p.slot.vm_id)
            || vm.string("ElementName")? != p.slot.vm_name
        {
            return Err("enrolled VM identity mismatch".into());
        }
        let state = match vm.number("EnabledState")? {
            2 => "Running",
            3 => "Off",
            _ => return Err("enrolled VM state rejected".into()),
        }
        .to_owned();
        let settings=session.one(&format!("SELECT * FROM Msvm_VirtualSystemSettingData WHERE VirtualSystemType='Microsoft:Hyper-V:System:Realized' AND VirtualSystemIdentifier='{}'",quoted(&p.slot.vm_id)))?;
        if settings.string("VirtualSystemSubType")? != "Microsoft:Hyper-V:SubType:2"
            || settings.string("Version")? != "12.0"
        {
            return Err("enrolled generation/configuration version mismatch".into());
        }
        let snapshots=session.query(&format!("SELECT * FROM Msvm_VirtualSystemSettingData WHERE VirtualSystemIdentifier='{}' AND VirtualSystemType<>'Microsoft:Hyper-V:System:Realized'",quoted(&p.slot.vm_id)))?;
        if !snapshots.is_empty()
            || (!allow_auto && settings.boolean("AutomaticSnapshotsEnabled")?)
        {
            return Err("checkpoint state rejected".into());
        }
        let adapters = session.related(&settings, "Msvm_GpuPartitionSettingData")?;
        if adapters.len() > 1 {
            return Err("duplicate guest GPU adapters".into());
        }
        for a in &adapters {
            if a.strings("HostResource")? != vec![p.slot.gpu_interface.clone()] {
                return Err("GPU adapter identity rejected".into());
            }
        }
        let drives = session
            .related(&settings, "Msvm_StorageAllocationSettingData")?
            .into_iter()
            .filter_map(|d| match d.string("ResourceSubType") {
                Ok(s) if s == "Microsoft:Hyper-V:Virtual Hard Disk" => Some(Ok(d)),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<Vec<_>>>()?;
        if drives.len() > 1 || (!reset && drives.len() != 1) {
            return Err("enrolled child attachment count mismatch".into());
        }
        for d in &drives {
            if d.strings("HostResource")? != vec![p.slot.child_path.to_string_lossy().into_owned()]
            {
                return Err("enrolled child attachment mismatch".into());
            }
        }
        let service = session.one("SELECT * FROM Msvm_VirtualSystemManagementService")?;
        // Exact host GPU and all capabilities are validated independently.
        crate::windows_hyperv_read::collect(p)?;
        Ok(Self {
            session,
            vm,
            settings,
            service,
            adapters,
            #[cfg(feature = "dev-harness")]
            drives,
            state,
        })
    }
    fn related_one(&self, class: &str) -> Result<Object> {
        one(self.session.related(&self.settings, class)?)
    }
    fn profile(&self, p: &ProjectConfiguration) -> Result<SettingsSnapshot> {
        let adapter = profile_adapter(&self.state, &self.adapters)?;
        let memory = self.related_one("Msvm_MemorySettingData")?;
        let cpu = self.related_one("Msvm_ProcessorSettingData")?;
        let security = self.related_one("Msvm_SecuritySettingData")?;
        decode_profile(
            p,
            &self.state,
            &ProfileObjects {
                settings: &self.settings,
                memory: &memory,
                cpu: &cpu,
                security: &security,
                adapter,
            },
            crate::windows_hyperv_read::collect(p)?.limits,
        )
    }
    fn reject_other_assignments(&self, p: &ProjectConfiguration) -> Result<()> {
        for vm in self
            .session
            .query("SELECT * FROM Msvm_ComputerSystem WHERE Caption='Virtual Machine'")?
        {
            let id = vm.string("Name")?;
            if id.eq_ignore_ascii_case(&p.slot.vm_id) {
                continue;
            }
            let settings=self.session.one(&format!("SELECT * FROM Msvm_VirtualSystemSettingData WHERE VirtualSystemType='Microsoft:Hyper-V:System:Realized' AND VirtualSystemIdentifier='{}'",quoted(&id)))?;
            if !self
                .session
                .related(&settings, "Msvm_GpuPartitionSettingData")?
                .is_empty()
            {
                return Err("another VM GPU assignment rejected".into());
            }
        }
        Ok(())
    }
    fn immediate(&self, p: &ProjectConfiguration, expected: &str) -> Result<()> {
        let fresh = State::read(p, Duration::from_secs(30), false, false)?;
        if fresh.state != expected
            || fresh.adapters.len() != self.adapters.len()
            || fresh.settings.path()? != self.settings.path()?
        {
            return Err("native preimage changed before effect".into());
        }
        verify_child(p, false)
    }
}
// The same provider objects feed decoding and writing, so fixtures exercise
// native property names and representations without a privileged provider.
struct ProfileObjects<'a> {
    settings: &'a Object,
    memory: &'a Object,
    cpu: &'a Object,
    security: &'a Object,
    adapter: &'a Object,
}
fn profile_adapter<'a>(state: &str, adapters: &'a [Object]) -> Result<&'a Object> {
    if state != "Off" || adapters.len() != 1 {
        return Err("settings require Off VM with exactly one adapter".into());
    }
    adapters
        .first()
        .ok_or_else(|| "settings require Off VM with exactly one adapter".into())
}
fn decode_profile(
    p: &ProjectConfiguration,
    state: &str,
    o: &ProfileObjects<'_>,
    limits: GpuResources,
) -> Result<SettingsSnapshot> {
    let checkpoint_type = match o.settings.number("UserSnapshotType")? {
        2 => "Disabled",
        3 => "Production",
        4 => "ProductionOnly",
        5 => "Standard",
        _ => return Err("unknown checkpoint policy".into()),
    }
    .to_owned();
    let automatic_stop_action = match o.settings.number("AutomaticShutdownAction")? {
        2 => "TurnOff",
        3 => "Save",
        4 => "ShutDown",
        _ => return Err("unknown automatic stop policy".into()),
    }
    .to_owned();
    let automatic_checkpoints = o.settings.boolean("AutomaticSnapshotsEnabled")?;
    let secure_boot_template = if o
        .settings
        .string("SecureBootTemplateId")?
        .eq_ignore_ascii_case("1734C6E8-3154-4DDA-BA5F-A874CC483422")
    {
        "MicrosoftWindows".into()
    } else {
        o.settings.string("SecureBootTemplateId")?
    };
    let mib = |o: &Object, n: &str| {
        o.number(n)?
            .checked_mul(1024 * 1024)
            .ok_or_else(|| format!("{n}: byte conversion overflow"))
    };
    let result = SettingsSnapshot {
        vm_id: p.slot.vm_id.clone(),
        state: state.to_owned(),
        gpu_interface: p.slot.gpu_interface.clone(),
        gpu_adapters: 1,
        secure_boot: o.settings.boolean("SecureBootEnabled")?,
        secure_boot_template,
        tpm_enabled: o.security.boolean("TpmEnabled")?,
        dynamic_memory: o.memory.boolean("DynamicMemoryEnabled")?,
        profile: VmProfile {
            memory_bytes: mib(o.memory, "VirtualQuantity")?,
            processors: u32::try_from(o.cpu.number("VirtualQuantity")?)
                .map_err(|_| "CPU count overflow")?,
            low_mmio_bytes: mib(o.settings, "LowMmioGapSize")?,
            high_mmio_bytes: mib(o.settings, "HighMmioGapSize")?,
            guest_controlled_cache_types: o.settings.boolean("GuestControlledCacheTypes")?,
            expose_virtualization_extensions: o.cpu.boolean("ExposeVirtualizationExtensions")?,
            checkpoints_disabled: checkpoint_type == "Disabled" && !automatic_checkpoints,
            automatic_stop_guest_shutdown: automatic_stop_action == "ShutDown",
        },
        checkpoint_type,
        automatic_checkpoints,
        automatic_stop_action,
        resources: resources(o.adapter)?,
        limits,
    };
    result.validate(p).map_err(str::to_owned)?;
    Ok(result)
}
fn resources(o: &Object) -> Result<GpuResources> {
    let t = |n: &str| -> Result<Triple> {
        Ok(Triple {
            minimum: o.number(&format!("MinPartition{n}"))?,
            maximum: o.number(&format!("MaxPartition{n}"))?,
            optimal: o.number(&format!("OptimalPartition{n}"))?,
        })
    };
    Ok(GpuResources {
        vram: t("VRAM")?,
        encode: t("Encode")?,
        decode: t("Decode")?,
        compute: t("Compute")?,
    })
}
fn inspect_output(p: &ProjectConfiguration, s: &State, hash: &str) -> Result<String> {
    let policy = embedded_policy().map_err(|e| e.to_string())?;
    Ok(format!(
        "status\tok\nvm_id\t{}\nstate\t{}\ngpu_adapters\t{}\nchild\t{}\nparent\t{}\nparent_sha256\t{}\ngpu_interface\t{}\n",
        p.slot.vm_id,
        s.state,
        s.adapters.len(),
        policy.child(),
        policy.parent(),
        hash,
        policy.gpu_interface()
    ))
}

/// Perform one authorized fixed native operation and return the existing protocol.
///
/// # Errors
/// Native/provider/guard failures retain errors; the parent retains reconciliation.
/// Mutating calls require the runner token and matching protected operation marker.
pub fn execute(
    p: &ProjectConfiguration,
    action: NativeAction,
    timeout: Duration,
) -> Result<String> {
    if action == NativeAction::Reset && !cfg!(feature = "dev-harness") {
        return Err("disposable reset requires a development harness build".into());
    }
    let _directories = DirectoryGuard::acquire(&[&p.slot.parent_path, &p.slot.child_path])?;
    let deadline = Instant::now() + timeout;
    let reset = action == NativeAction::Reset;
    let allow_auto = matches!(
        action,
        NativeAction::SettingsInspect | NativeAction::ReadProfile | NativeAction::ApplySettings
    );
    let s = State::read(p, timeout, allow_auto, reset)?;
    let parent = ParentGuard::verify(
        p,
        matches!(
            action,
            NativeAction::Inspect | NativeAction::SettingsInspect | NativeAction::Reset
        ),
        reset,
        deadline,
    )?;
    verify_child(p, reset)?;
    match action {
        NativeAction::Inspect | NativeAction::SettingsInspect => {
            inspect_output(p, &s, &parent.hash)
        }
        NativeAction::ReadProfile => {
            serde_json::to_string(&s.profile(p)?).map_err(|e| e.to_string())
        }
        NativeAction::ApplySettings => apply_settings(p),
        NativeAction::Attach | NativeAction::Remove => assignment(p, s, action, &parent.hash),
        NativeAction::Start | NativeAction::Shutdown => {
            lifecycle(p, s, action, &parent.hash, deadline)
        }
        #[cfg(feature = "dev-harness")]
        NativeAction::Reset => reset_child(p, s, &parent.hash),
        #[cfg(not(feature = "dev-harness"))]
        NativeAction::Reset => unreachable!("development operation rejected before effects"),
    }
}
fn apply_settings(p: &ProjectConfiguration) -> Result<String> {
    p.vm_profile.validate().map_err(|e| e.to_string())?;
    let expected: SettingsSnapshot = serde_json::from_slice(
        &std::fs::read(
            p.runner
                .data_directory
                .join("state/native-settings-preimage-v1.json"),
        )
        .map_err(|e| format!("read protected settings preimage: {e}"))?,
    )
    .map_err(|e| format!("parse protected settings preimage: {e}"))?;
    expected.validate(p).map_err(str::to_owned)?;
    let fresh = State::read(p, p.runner.gpu_assignment_timeout, true, false)?;
    let adapter = profile_adapter(&fresh.state, &fresh.adapters)?;
    let memory = fresh.related_one("Msvm_MemorySettingData")?;
    let cpu = fresh.related_one("Msvm_ProcessorSettingData")?;
    let security = fresh.related_one("Msvm_SecuritySettingData")?;
    let objects = ProfileObjects {
        settings: &fresh.settings,
        memory: &memory,
        cpu: &cpu,
        security: &security,
        adapter,
    };
    let observed = decode_profile(
        p,
        &fresh.state,
        &objects,
        crate::windows_hyperv_read::collect(p)?.limits,
    )?;
    apply_profile(p, &expected, &observed, &objects, |object, system| {
        fresh.session.modify(&fresh.service, object, system)
    })?;
    // Success belongs to the independent parent reader, never cached setters.
    Ok(String::new())
}
fn apply_profile(
    p: &ProjectConfiguration,
    expected: &SettingsSnapshot,
    observed: &SettingsSnapshot,
    o: &ProfileObjects<'_>,
    mut publish: impl FnMut(&Object, bool) -> Result<()>,
) -> Result<()> {
    p.vm_profile.validate().map_err(|e| e.to_string())?;
    expected.validate(p).map_err(str::to_owned)?;
    observed.validate(p).map_err(str::to_owned)?;
    if observed != expected {
        return Err("stale settings preimage rejected".into());
    }
    let profile = &p.vm_profile;
    o.settings
        .set_number("LowMmioGapSize", profile.low_mmio_bytes / (1024 * 1024))?;
    o.settings
        .set_number("HighMmioGapSize", profile.high_mmio_bytes / (1024 * 1024))?;
    o.settings.set(
        "GuestControlledCacheTypes",
        VARIANT::from(profile.guest_controlled_cache_types),
    )?;
    o.settings.set_number("UserSnapshotType", 2)?;
    o.settings
        .set("AutomaticSnapshotsEnabled", VARIANT::from(false))?;
    o.settings.set_number("AutomaticShutdownAction", 4)?;
    publish(o.settings, true)?;
    o.memory.set("DynamicMemoryEnabled", VARIANT::from(false))?;
    o.memory
        .set_number("VirtualQuantity", profile.memory_bytes / (1024 * 1024))?;
    publish(o.memory, false)?;
    o.cpu
        .set_number("VirtualQuantity", u64::from(profile.processors))?;
    o.cpu.set(
        "ExposeVirtualizationExtensions",
        VARIANT::from(profile.expose_virtualization_extensions),
    )?;
    publish(o.cpu, false)?;
    let r = GpuResources::desired(p).map_err(|e| e.to_string())?;
    for (name, t) in [
        ("VRAM", r.vram),
        ("Encode", r.encode),
        ("Decode", r.decode),
        ("Compute", r.compute),
    ] {
        for (prefix, n) in [
            ("Min", t.minimum),
            ("Max", t.maximum),
            ("Optimal", t.optimal),
        ] {
            o.adapter
                .set_number(&format!("{prefix}Partition{name}"), n)?;
        }
    }
    publish(o.adapter, false)?;
    Ok(())
}

#[cfg(test)]
#[path = "windows_hyperv_settings_tests.rs"]
mod settings_tests;
fn assignment(
    p: &ProjectConfiguration,
    s: State,
    action: NativeAction,
    hash: &str,
) -> Result<String> {
    let previous = s.adapters.len();
    let next = if action == NativeAction::Attach { 1 } else { 0 };
    if s.state != "Off" || previous != 1 - next {
        return Err("GPU assignment state/count mismatch".into());
    }
    if action == NativeAction::Attach {
        s.reject_other_assignments(p)?;
    }
    s.immediate(p, "Off")?;
    if action == NativeAction::Attach {
        let template = one(s
            .session
            .query("SELECT * FROM Msvm_GpuPartitionSettingData")?
            .into_iter()
            .filter_map(|o| match o.string("InstanceID") {
                Ok(id) if id.starts_with("Microsoft:Definition\\") && id.ends_with("\\Default") => {
                    Some(Ok(o))
                }
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<Vec<_>>>()?)?;
        template.set("InstanceID", VARIANT::from(BSTR::from("")))?;
        template.set_strings("HostResource", std::slice::from_ref(&p.slot.gpu_interface))?;
        s.session.invoke(
            &s.service,
            "AddResourceSettings",
            vec![
                (
                    "AffectedConfiguration",
                    VARIANT::from(BSTR::from(s.settings.path()?)),
                ),
                ("ResourceSettings", string_array(&[template.xml()?])?),
            ],
        )?;
    } else {
        s.session.invoke(
            &s.service,
            "RemoveResourceSettings",
            vec![("ResourceSettings", string_array(&[s.adapters[0].path()?])?)],
        )?;
    }
    let fresh = State::read(p, Duration::from_secs(30), false, false)?;
    if fresh.state != "Off" || fresh.adapters.len() != next {
        return Err("GPU assignment independent readback failed".into());
    }
    let policy = embedded_policy().map_err(|e| e.to_string())?;
    Ok(format!(
        "status\tok\nvm_id\t{}\nstate\tOff\nprevious_gpu_adapters\t{previous}\ngpu_adapters\t{next}\nchild\t{}\nparent\t{}\nparent_sha256\t{hash}\ngpu_interface\t{}\n",
        p.slot.vm_id,
        policy.child(),
        policy.parent(),
        policy.gpu_interface()
    ))
}
#[allow(unsafe_code)]
fn available_memory() -> Result<u64> {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut memory = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    // SAFETY: correct initialized size and writable native output.
    unsafe { GlobalMemoryStatusEx(&mut memory) }.map_err(|e| e.to_string())?;
    Ok(memory.ullAvailPhys)
}
fn lifecycle(
    p: &ProjectConfiguration,
    s: State,
    action: NativeAction,
    hash: &str,
    deadline: Instant,
) -> Result<String> {
    let (previous, next) = if action == NativeAction::Start {
        ("Off", "Running")
    } else {
        ("Running", "Off")
    };
    if s.state != previous {
        return Err("lifecycle preimage state mismatch".into());
    }
    if action == NativeAction::Start {
        s.reject_other_assignments(p)?;
        if available_memory()? < 12 * 1024 * 1024 * 1024 {
            return Err("host available memory below 12 GiB".into());
        }
    }
    s.immediate(p, previous)?;
    if action == NativeAction::Start {
        s.session.invoke(
            &s.vm,
            "RequestStateChange",
            vec![("RequestedState", VARIANT::from(2i32))],
        )?;
    } else {
        let shutdown = one(s.session.related(&s.vm, "Msvm_ShutdownComponent")?)?;
        s.session.invoke(
            &shutdown,
            "InitiateShutdown",
            vec![
                ("Force", VARIANT::from(false)),
                (
                    "Reason",
                    VARIANT::from(BSTR::from("HyperGpuSupport fixed guest shutdown")),
                ),
            ],
        )?;
    }
    loop {
        if Instant::now() >= deadline {
            return Err("guest lifecycle deadline expired; reconcile".into());
        }
        let vm = s.session.get(&s.vm.path()?)?;
        if vm.number("EnabledState")? == if next == "Running" { 2 } else { 3 } {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    let fresh = State::read(p, Duration::from_secs(30), false, false)?;
    if fresh.state != next || fresh.adapters.len() != s.adapters.len() {
        return Err("guest lifecycle independent readback failed".into());
    }
    let policy = embedded_policy().map_err(|e| e.to_string())?;
    Ok(format!(
        "status\tok\nvm_id\t{}\nprevious_state\t{previous}\nstate\t{next}\ngpu_adapters\t{}\nchild\t{}\nparent\t{}\nparent_sha256\t{hash}\n",
        p.slot.vm_id,
        fresh.adapters.len(),
        policy.child(),
        policy.parent()
    ))
}
#[cfg(feature = "dev-harness")]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../test-harness/reset_vm.rs"
));

/// Launch only this runner's fixed native mode under the existing contained supervisor.
///
/// # Errors
/// Launch, deadline, overflow and native operation failures are returned unchanged.
pub fn run_worker(action: NativeAction, timeout: Duration) -> Result<String> {
    let mut command =
        std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?);
    command.arg(action.mode());
    let output = crate::windows_validation::bounded_process_with_limit(command, timeout, 64 * 1024)
        .map_err(|e| e.to_string())?;
    if output.exit_code != Some(0) || !output.stderr.is_empty() {
        return Err(format!(
            "native {} failed {:?}: {}",
            action.mode(),
            output.exit_code,
            output.stderr
        ));
    }
    Ok(output.stdout)
}

/// Fixed protected settings preimage file used only under the parent's operation lock.
pub fn preimage_path(data: &Path) -> std::path::PathBuf {
    data.join("state/native-settings-preimage-v1.json")
}

#[cfg(test)]
mod boundary_tests {
    use super::*;

    #[test]
    fn reset_mode_requires_explicit_development_build() {
        assert_eq!(
            NativeAction::parse("native-reset"),
            cfg!(feature = "dev-harness").then_some(NativeAction::Reset)
        );
        assert_eq!(
            NativeAction::parse("native-inspect"),
            Some(NativeAction::Inspect)
        );
    }

    #[cfg(not(feature = "dev-harness"))]
    #[test]
    fn direct_reset_is_rejected_before_any_target_access() {
        let mut project = ProjectConfiguration::embedded().unwrap();
        project.slot.parent_path = "invalid\0parent".into();
        project.slot.child_path = "invalid\0child".into();
        assert_eq!(
            execute(&project, NativeAction::Reset, Duration::ZERO),
            Err("disposable reset requires a development harness build".into())
        );
    }
}
