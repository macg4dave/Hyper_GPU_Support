//! Reviewed provider completion, separate from workflow outcome acceptance.
use super::*;
use crate::model::{Allocation, Power, Resource, Settings, Target, VmState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Scope {
    pub target: Target,
    pub before: VmState,
    pub effect: Effect,
    pub recovery: Option<crate::configuration_store::Revision>,
    pub configuration: crate::configuration_store::Revision,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) enum Effect {
    Assignment(bool),
    Settings(Settings),
    Allocation(Resource, Allocation),
    Power(Power),
}

impl Scope {
    pub(crate) fn new(target: &Target, effect: Effect) -> Result<Self> {
        target.validate()?;
        Ok(Self {
            target: target.clone(),
            before: crate::windows_hyperv::inspect(&target.vm_id)?,
            effect,
            recovery: crate::runner::recovery_record()?
                .as_ref()
                .map(revision)
                .transpose()?,
            configuration: crate::configuration_store::protected::revision(&target.vm_id)?,
        })
    }
    fn job_type(&self) -> u64 {
        match self.effect {
            Effect::Assignment(true) => 30,
            Effect::Assignment(false) => 32,
            Effect::Settings(_) => 2,
            Effect::Allocation(..) => 31,
            Effect::Power(Power::Running) => 12,
            Effect::Power(_) => 16,
        }
    }
    fn method(&self) -> &'static str {
        match self.effect {
            Effect::Assignment(true) => "AddResourceSettings",
            Effect::Assignment(false) => "RemoveResourceSettings",
            Effect::Settings(_) => "ModifySystemSettings",
            Effect::Allocation(..) => "ModifyResourceSettings",
            Effect::Power(Power::Running) => "RequestStateChange",
            Effect::Power(_) => "InitiateShutdown",
        }
    }
    fn validate_state(&self, state: &VmState) -> Result<()> {
        self.target.validate()?;
        if self.before.vm_id != self.target.vm_id {
            return Err("Native VM binding changed".into());
        }
        if same(state, &self.before)? {
            return Ok(());
        }
        let mut expected = self.before.clone();
        match &self.effect {
            Effect::Assignment(enabled) => {
                expected.gpus = if *enabled {
                    vec![self.target.gpu_interface.clone()]
                } else {
                    vec![]
                };
                // A new adapter receives provider defaults. Removal releases its
                // allocations; neither action approves a requested allocation.
                expected.vram = state.vram.clone();
                expected.compute = state.compute.clone();
                expected.encode = state.encode.clone();
                expected.decode = state.decode.clone();
                if !enabled
                    && Resource::ALL
                        .into_iter()
                        .any(|r| state.allocation(r).is_some())
                {
                    return Err("Detached adapter still reports allocations".into());
                }
            }
            Effect::Settings(settings) => expected.settings = settings.clone(),
            Effect::Allocation(resource, value) => match resource {
                Resource::Vram => expected.vram = Some(value.clone()),
                Resource::Compute => expected.compute = Some(value.clone()),
                Resource::Encode => expected.encode = Some(value.clone()),
                Resource::Decode => expected.decode = Some(value.clone()),
            },
            Effect::Power(power) => expected.power = power.clone(),
        }
        if same(state, &expected)? {
            Ok(())
        } else {
            Err("Native before/after facts conflict; retain provider and workflow recovery".into())
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct JobEvidence {
    path: String,
    vm_id: String,
    kind: u64,
    submitted: String,
    state: u64,
    error: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    pending: crate::configuration_store::Revision,
    job: JobEvidence,
    observed: VmState,
}
fn same(a: &impl Serialize, b: &impl Serialize) -> Result<bool> {
    Ok(serde_json::to_value(a).map_err(|e| e.to_string())?
        == serde_json::to_value(b).map_err(|e| e.to_string())?)
}
fn revision(value: &impl Serialize) -> Result<crate::configuration_store::Revision> {
    Ok(crate::configuration_store::Revision::of(
        &serde_json::to_vec(value).map_err(|e| e.to_string())?,
    ))
}
fn read<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<T> {
    use std::io::Read;
    crate::security::verify(path)?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 1024 * 1024 {
        return Err("Provider evidence exceeds limit".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
fn evidence(s: &Session, path: &str) -> Result<JobEvidence> {
    let path = local_job_path(path)?;
    let job = s.get(path)?;
    let systems = s.related(&job, "Msvm_ComputerSystem")?;
    let vm = one(systems
        .into_iter()
        .filter(|vm| vm.string("Caption").is_ok_and(|c| c == "Virtual Machine"))
        .collect())?;
    if vm.string("Caption")? != "Virtual Machine" {
        return Err("Job has no unique affected VM".into());
    }
    Ok(JobEvidence {
        path: local_job_path(&job.path()?)?.into(),
        vm_id: vm.string("Name")?.to_ascii_lowercase(),
        kind: job.number("JobType")?,
        submitted: job.string("TimeSubmitted")?,
        state: job.number("JobState")?,
        error: job.number("ErrorCode")?,
    })
}
fn validate_binding(scope: &Scope) -> Result<()> {
    if crate::runner::recovery_record()?
        .as_ref()
        .map(revision)
        .transpose()?
        != scope.recovery
        || crate::configuration_store::protected::revision(&scope.target.vm_id)?
            != scope.configuration
    {
        return Err("Native workflow/configuration binding changed; retain recovery".into());
    }
    Ok(())
}
pub(super) fn validate_recorded(pending: &PendingOperation, s: &Session) -> Result<()> {
    if let Some(scope) = &pending.scope {
        validate_binding(scope)?;
        let job = evidence(s, pending.job_path()?)?;
        validate(
            pending,
            &job,
            &crate::windows_hyperv::inspect(&scope.target.vm_id)?,
        )?;
    }
    Ok(())
}
fn validate(pending: &PendingOperation, job: &JobEvidence, state: &VmState) -> Result<()> {
    let scope = pending.scope.as_ref().ok_or(
        "Legacy native record lacks VM/stage facts; preserve it for administrator provider support",
    )?;
    if pending.schema != 2
        || scope.method() != pending.method
        || scope.target.vm_id != job.vm_id
        || scope.job_type() != job.kind
        || job.submitted.is_empty()
        || pending.jobs_before.iter().any(|id| id == &job.path)
        || pending
            .job
            .as_ref()
            .is_some_and(|path| local_job_path(path).ok() != Some(job.path.as_str()))
    {
        return Err(
            "Provider completion is not bound to this VM, stage and invocation; retain recovery"
                .into(),
        );
    }
    require_terminal_operation(job.state)?;
    scope.validate_state(state)
}
pub(super) fn job_paths(s: &Session) -> Result<Vec<String>> {
    let jobs = s.query("SELECT * FROM Msvm_ConcreteJob")?;
    if jobs.len() > 128 {
        return Err("Provider job inventory exceeds recovery binding limit".into());
    }
    jobs.into_iter()
        .map(|j| Ok(local_job_path(&j.path()?)?.into()))
        .collect()
}
fn confirm_terminal_snapshot(recorded: &JobEvidence, fresh: Option<&JobEvidence>) -> Result<()> {
    require_terminal_operation(recorded.state)?;
    if let Some(fresh) = fresh
        && !same(recorded, fresh)?
    {
        return Err("Provider completion changed".into());
    }
    Ok(())
}

fn retire_reviewed(
    archive: impl FnOnce() -> Result<()>,
    recheck: impl FnOnce() -> Result<()>,
    clear: impl FnOnce() -> Result<()>,
) -> Result<()> {
    archive()?;
    recheck()?;
    clear()
}
fn require_quiescent(s: &Session, vm: &str) -> Result<()> {
    for job in s.query("SELECT * FROM Msvm_ConcreteJob")? {
        if !(7..=10).contains(&job.number("JobState")?) {
            // Conservative: unresolved jobs anywhere on the host prevent manual
            // retirement, including jobs without an affected-element association.
            return Err(format!(
                "Provider work remains active; retain recovery for VM {vm}"
            ));
        }
    }
    Ok(())
}
pub(crate) fn review(job: &str) -> Result<serde_json::Value> {
    if !crate::process::is_elevated()? {
        return Err("Native recovery review requires an administrator console".into());
    }
    let _lock = crate::runner::operation_lock()?;
    let pending: PendingOperation = read(&operation_path()?)?;
    validate_binding(pending.scope.as_ref().ok_or_else(recovery_handoff)?)?;
    let s = Session::new(Duration::from_secs(30))?;
    let job = evidence(&s, job)?;
    let state = crate::windows_hyperv::inspect(&job.vm_id)?;
    validate(&pending, &job, &state)?;
    require_quiescent(&s, &job.vm_id)?;
    let review = Review {
        pending: revision(&pending)?,
        job,
        observed: state,
    };
    crate::runner::atomic_json(
        &crate::runner::data_directory()?.join("provider-review.json"),
        &review,
    )?;
    let crate::configuration_store::Revision::Content(token) = revision(&review)? else {
        return Err("Missing review revision".into());
    };
    Ok(
        serde_json::json!({"review":token, "original":pending, "completion":review, "next":"Review VM, method, before/after facts and provider job correlation. Explicitly approve with native-reconcile --review TOKEN. This retires only provider admission, never workflow success."}),
    )
}
pub(crate) fn reconcile(token: &crate::configuration_store::Revision) -> Result<()> {
    if !crate::process::is_elevated()? {
        return Err("Native recovery requires an administrator console".into());
    }
    let _lock = crate::runner::operation_lock()?;
    let pending: PendingOperation = read(&operation_path()?)?;
    let review: Review = read(&crate::runner::data_directory()?.join("provider-review.json"))?;
    if &revision(&review)? != token || review.pending != revision(&pending)? {
        return Err("Native recovery review changed; review again".into());
    }
    validate_binding(pending.scope.as_ref().ok_or_else(recovery_handoff)?)?;
    let s = Session::new(Duration::from_secs(30))?;
    require_quiescent(&s, &review.job.vm_id)?;
    let state = crate::windows_hyperv::inspect(&review.job.vm_id)?;
    validate(&pending, &review.job, &state)?;
    if !same(&state, &review.observed)? {
        return Err("Native reviewed readback changed; retain recovery".into());
    }
    // Protected terminal evidence survives job expiry. A still-existing job must
    // retain the exact terminal identity and result; absence is allowed only for
    // WBEM_E_NOT_FOUND, never an unreadable provider.
    let fresh_job = if s.get_optional(&review.job.path)?.is_some() {
        Some(evidence(&s, &review.job.path)?)
    } else {
        None
    };
    confirm_terminal_snapshot(&review.job, fresh_job.as_ref())?;
    let audit = crate::runner::data_directory()?.join("audit");
    crate::security::verify(&audit)?;
    let crate::configuration_store::Revision::Content(key) = revision(&pending)? else {
        return Err("Missing native revision".into());
    };
    retire_reviewed(
        || {
            crate::runner::atomic_json(
                &audit.join(format!("native-{key}.json")),
                &serde_json::json!({"original":pending,"approved":review,"disposition":"ProviderTerminalOnly; workflow recovery retained"}),
            )
        },
        || {
            let current: PendingOperation = read(&operation_path()?)?;
            if revision(&current)? != review.pending {
                return Err("Native marker changed during retirement".into());
            }
            validate_binding(current.scope.as_ref().ok_or_else(recovery_handoff)?)?;
            require_quiescent(&s, &review.job.vm_id)?;
            if !same(
                &crate::windows_hyperv::inspect(&review.job.vm_id)?,
                &review.observed,
            )? {
                return Err("Native readback changed during retirement; retain recovery".into());
            }
            Ok(())
        },
        clear_operation,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration_store::Revision;

    fn fixture() -> (PendingOperation, JobEvidence, VmState) {
        let before = VmState {
            vm_id: "11111111-1111-1111-1111-111111111111".into(),
            name: "Test VM".into(),
            generation: 2,
            power: Power::Running,
            gpus: vec![],
            vram: None,
            compute: None,
            encode: None,
            decode: None,
            settings: Settings {
                low_mmio: 128,
                high_mmio: 512,
                cache_types: false,
                automatic_checkpoints: true,
            },
        };
        let scope = Scope {
            target: Target {
                vm_id: before.vm_id.clone(),
                gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#test\GPUPARAV".into(),
                enabled: true,
                vram: None,
                compute: None,
                encode: None,
                decode: None,
            },
            before: before.clone(),
            effect: Effect::Power(Power::Off),
            recovery: Some(Revision::of(b"workflow")),
            configuration: Revision::Missing,
        };
        let pending = PendingOperation {
            schema: 2,
            method: "InitiateShutdown".into(),
            target: "shutdown-component".into(),
            job: None,
            scope: Some(scope),
            jobs_before: vec!["Msvm_ConcreteJob.InstanceID=\"old\"".into()],
            failure: None,
        };
        let job = JobEvidence {
            path: "Msvm_ConcreteJob.InstanceID=\"new\"".into(),
            vm_id: before.vm_id.clone(),
            kind: 16,
            submitted: "provider-time".into(),
            state: 7,
            error: 0,
        };
        (pending, job, before)
    }

    #[test]
    fn lost_reply_needs_independent_terminal_job_and_scoped_before_or_after_state() {
        let (mut pending, job, mut state) = fixture();
        assert!(validate(&pending, &job, &state).is_ok());
        state.power = Power::Off;
        assert!(validate(&pending, &job, &state).is_ok());
        pending.job = Some(job.path.clone());
        assert!(validate(&pending, &job, &state).is_ok());
        // A matching instantaneous state cannot manufacture completion evidence.
        for terminal in [false, true] {
            let mut conflicting = job.clone();
            if terminal {
                conflicting.vm_id = "another-vm".into();
            } else {
                conflicting.state = 4;
            }
            assert!(validate(&pending, &conflicting, &state).is_err());
        }
    }

    #[test]
    fn unrelated_old_active_missing_or_conflicting_provider_evidence_keeps_marker() {
        for case in 0..9 {
            let (mut pending, mut job, mut state) = fixture();
            match case {
                0 => pending.scope = None,
                1 => pending.method = "AddResourceSettings".into(),
                2 => job.path = pending.jobs_before[0].clone(),
                3 => job.kind = 12,
                4 => job.state = 6,
                5 => state.settings.high_mmio += 1,
                6 => state.gpus.push("unrelated".into()),
                7 => job.submitted.clear(),
                _ => pending.job = Some("Msvm_ConcreteJob.InstanceID=\"different\"".into()),
            }
            assert!(validate(&pending, &job, &state).is_err(), "case {case}");
        }
    }

    #[test]
    fn provider_failure_retires_only_admission_and_survives_review_interruption() {
        let (pending, mut job, state) = fixture();
        job.state = 8;
        job.error = 42;
        validate(&pending, &job, &state).unwrap();
        let review = Review {
            pending: revision(&pending).unwrap(),
            job,
            observed: state,
        };
        let bytes = serde_json::to_vec(&review).unwrap();
        let reopened: Review = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(revision(&review).unwrap(), revision(&reopened).unwrap());
        assert_eq!(reopened.job.error, 42);
        confirm_terminal_snapshot(&reopened.job, None).unwrap();
        confirm_terminal_snapshot(&reopened.job, Some(&reopened.job)).unwrap();
        let mut active = reopened.job.clone();
        active.state = 4;
        assert!(confirm_terminal_snapshot(&reopened.job, Some(&active)).is_err());
        assert!(confirm_terminal_snapshot(&active, None).is_err());
        assert_eq!(reopened.pending, revision(&pending).unwrap());
        // Persisted completion remains independently usable after job expiry;
        // altered readback requires a new review, never an inferred success.
        let mut changed = reopened.observed.clone();
        changed.power = Power::Off;
        assert!(!same(&changed, &reopened.observed).unwrap());
        assert!(validate(&pending, &reopened.job, &reopened.observed).is_ok());
    }

    #[test]
    fn terminal_failed_detach_accepts_unchanged_adapter_and_allocations_without_claiming_success() {
        let (mut pending, mut job, mut state) = fixture();
        let scope = pending.scope.as_mut().unwrap();
        state.power = Power::Off;
        state.gpus = vec![scope.target.gpu_interface.clone()];
        state.vram = Some(Allocation {
            minimum: 1,
            optimal: 2,
            maximum: 3,
        });
        scope.before = state.clone();
        scope.effect = Effect::Assignment(false);
        pending.method = "RemoveResourceSettings".into();
        job.kind = 32;
        job.state = 8;
        job.error = 42;
        validate(&pending, &job, &state).unwrap();
        state.gpus.clear();
        assert!(validate(&pending, &job, &state).is_err()); // partial readback
        state.vram = None;
        validate(&pending, &job, &state).unwrap();
    }

    #[test]
    fn archive_failure_interruption_or_changed_readback_never_clears_native_hold() {
        use std::cell::{Cell, RefCell};
        for failure in [Some("archive"), Some("recheck"), Some("clear"), None] {
            let events = RefCell::new(Vec::new());
            let marker = Cell::new(true);
            let archive = Cell::new(false);
            let result = retire_reviewed(
                || {
                    events.borrow_mut().push("archive");
                    if failure == Some("archive") {
                        return Err("archive unavailable".into());
                    }
                    archive.set(true);
                    Ok(())
                },
                || {
                    events.borrow_mut().push("recheck");
                    if failure == Some("recheck") {
                        return Err("readback unavailable or changed".into());
                    }
                    Ok(())
                },
                || {
                    events.borrow_mut().push("clear");
                    if failure == Some("clear") {
                        return Err("marker removal unavailable".into());
                    }
                    marker.set(false);
                    Ok(())
                },
            );
            assert_eq!(result.is_err(), failure.is_some());
            assert_eq!(marker.get(), failure.is_some());
            if !marker.get() {
                assert!(archive.get());
            }
            assert_eq!(events.borrow()[0], "archive");
            if failure == Some("recheck") {
                assert_eq!(*events.borrow(), ["archive", "recheck"]);
            }
        }
    }
}
