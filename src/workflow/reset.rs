//! Reviewed host restoration after interrupted enable; never replay the failed Apply.
use super::{Backend, GuestCheck, Journal};
use crate::configuration_store::{Revision, SavePhase, SaveRecord};
use crate::model::{Gpu, Power, Resource, VmState};
use serde::{Deserialize, Serialize};

/// Fresh facts approved separately for restoring an initially unattached VM.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResetReview {
    /// Complete fresh host state, including every observed allocation.
    pub observed: VmState,
    /// Fresh GPU identity, driver and capabilities.
    pub gpu: Gpu,
    /// Exact pending journal; retained in the archived recovery receipt.
    pub journal: Journal,
}

fn same(a: &impl Serialize, b: &impl Serialize) -> Result<bool, String> {
    Ok(serde_json::to_value(a).map_err(|e| e.to_string())?
        == serde_json::to_value(b).map_err(|e| e.to_string())?)
}

impl ResetReview {
    /// Admit only an interrupted enable whose original host state can be restored
    /// without booting, guest access, allocation writes, or changing another GPU.
    pub fn validate(&self, record: &SaveRecord, revision: &Revision) -> Result<(), String> {
        record.target.validate()?;
        let initial = &record.initial;
        if record.schema != 1
            || record.operation_id.len() != 32
            || !record.operation_id.bytes().all(|b| b.is_ascii_hexdigit())
            || !matches!(record.phase, SavePhase::EffectsStarted)
            || !record.publication_required
            || !record.target.enabled
            || record.retirement.is_some()
            || (record.requires_graphics() && !self.failed_first_setup(record))
            // First preparation precedes all adapter/settings writes. Without its
            // receipt, later host changes cannot be attributed to this attempt.
            || (self.failed_first_setup(record)
                && (!self.observed.gpus.is_empty() || self.observed.settings != initial.settings))
            || record.expected != *revision
            || (!self.failed_first_setup(record) && record.final_power()? != Power::Off)
            || initial.vm_id != record.target.vm_id
            || initial.generation != 2
            || !(initial.power == Power::Off
                || (self.failed_first_setup(record) && initial.power == Power::Running))
            || !initial.gpus.is_empty()
            || Resource::ALL
                .into_iter()
                .any(|r| initial.allocation(r).is_some())
            || self.observed.vm_id != initial.vm_id
            || self.observed.generation != 2
            || !(self.observed.power == Power::Off
                || (self.failed_first_setup(record) && self.observed.power == Power::Running))
            || !(self.observed.gpus.is_empty()
                || self.observed.gpus == [record.target.gpu_interface.clone()])
            || (self.observed.gpus.is_empty()
                && Resource::ALL
                    .into_iter()
                    .any(|r| self.observed.allocation(r).is_some()))
            || !(self.observed.settings == initial.settings
                || self.observed.settings == record.expected_settings)
            || self.gpu.interface != record.target.gpu_interface
            || self.gpu.driver_version != record.driver_version
            || self.journal.schema != 1
            || self.journal.vm_id != initial.vm_id
            || self.journal.gpu_interface != record.target.gpu_interface
            || !self.journal.pending
            || self.journal.verification_only
            || self.journal.restore_power != record.final_power()?
            || self.journal.original != initial.settings
            || self.journal.applied != record.expected_settings
            || (!self.failed_first_setup(record) && self.journal.prepared != record.payload_digest)
        {
            return Err("Partial GPU removal is unavailable for these recorded facts. Retain recovery and Refresh.".into());
        }
        Ok(())
    }

    // A failed first preparation has no trusted guest receipt. Never extend this
    // disposition to an existing managed installation or a successful guest check.
    fn failed_first_setup(&self, record: &SaveRecord) -> bool {
        record.initial_journal.is_none()
            && self.journal.prepared.is_none()
            && self.journal.last_verified.is_none()
            && self.journal.preparation_started == Some(true)
            && matches!(
                self.journal.latest_check,
                Some(GuestCheck::Interrupted | GuestCheck::Failed)
            )
    }

    fn stopped(&self) -> VmState {
        let mut state = self.observed.clone();
        state.power = Power::Off;
        state
    }

    fn detached(&self) -> VmState {
        let mut state = self.stopped();
        state.gpus.clear();
        state.vram = None;
        state.compute = None;
        state.encode = None;
        state.decode = None;
        state
    }

    fn restored(&self, record: &SaveRecord) -> VmState {
        let mut state = self.detached();
        state.settings = record.initial.settings.clone();
        state
    }

    fn ended(&self, record: &SaveRecord) -> Journal {
        let mut journal = self.journal.clone();
        journal.pending = false;
        journal.prepared = None;
        journal.applied = record.initial.settings.clone();
        journal.settings_released = true;
        journal.latest_check = Some(GuestCheck::Unavailable);
        journal.check_restored = false;
        journal
    }

    fn check(
        &self,
        backend: &mut impl Backend,
        record: &SaveRecord,
        expected: &VmState,
        ended: bool,
    ) -> Result<(), String> {
        let journal = if ended {
            self.ended(record)
        } else {
            self.journal.clone()
        };
        if !same(&backend.inspect(&record.target)?, expected)?
            || !same(&backend.gpu(&record.target)?, &self.gpu)?
            || !same(&backend.journal(&record.target)?, &Some(journal))?
        {
            return Err(
                "Partial GPU removal facts changed or are unavailable; retain recovery.".into(),
            );
        }
        Ok(())
    }
}

/// Execute/resume only the durably approved removal transitions. Caller holds
/// the operation lock and binds the unchanged recovery and configuration receipts.
pub(crate) fn reset_partial(
    backend: &mut impl Backend,
    record: &SaveRecord,
    revision: &Revision,
    shutdown_approved: bool,
) -> Result<(), String> {
    let review = record
        .reset
        .as_ref()
        .ok_or("Review partial GPU removal first")?;
    review.validate(record, revision)?;
    if review.observed.power == Power::Running && !shutdown_approved {
        return Err("Separately approve graceful shutdown for this recovery disposition".into());
    }
    let mut state = backend.inspect(&record.target)?;
    let restored = review.restored(record);
    if same(&state, &restored)?
        && same(
            &backend.journal(&record.target)?,
            &Some(review.ended(record)),
        )?
    {
        return review.check(backend, record, &restored, true);
    }
    if same(&state, &review.observed)? && state.power == Power::Running {
        review.check(backend, record, &review.observed, false)?;
        backend.power(&record.target, Power::Off)?;
        review.check(backend, record, &review.stopped(), false)?;
        state = backend.inspect(&record.target)?;
    }
    // Resume only a separately recorded expected stage; never recreate an adapter.
    if same(&state, &review.stopped())? {
        review.check(backend, record, &review.stopped(), false)?;
        if !state.gpus.is_empty() {
            backend.assign(&record.target, false)?;
        }
    } else if !same(&state, &review.detached())? && !same(&state, &restored)? {
        return Err("Partial GPU removal host state changed; retain recovery.".into());
    }
    let current = backend.inspect(&record.target)?;
    if !same(&current, &restored)? {
        review.check(backend, record, &review.detached(), false)?;
        backend.settings(&record.target, &record.initial.settings)?;
    }
    review.check(backend, record, &restored, false)?;
    backend.save(&record.target, &review.ended(record))?;
    review.check(backend, record, &restored, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::test_support::{fake, recovery_record, target};

    fn partial() -> (crate::workflow::test_support::Fake, SaveRecord) {
        let mut backend = fake();
        backend.state.power = Power::Off;
        let mut record = recovery_record(&backend, true);
        record.graphics_required = Some(false);
        record.expected_settings = record.initial.settings.for_gpu();
        backend.state.settings = record.expected_settings.clone();
        backend.state.gpus = vec![target().gpu_interface];
        backend.journal = Some(Journal {
            schema: 1,
            vm_id: record.target.vm_id.clone(),
            gpu_interface: record.target.gpu_interface.clone(),
            original: record.initial.settings.clone(),
            applied: record.expected_settings.clone(),
            prepared: record.payload_digest.clone(),
            pending: true,
            restore_power: Power::Off,
            verification_only: false,
            last_verified: Some(123),
            latest_check: None,
            preparation_started: None,
            check_restored: false,
            settings_released: false,
        });
        record.reset = Some(ResetReview {
            observed: backend.state.clone(),
            gpu: backend.gpu(&record.target).unwrap(),
            journal: backend.journal.clone().unwrap(),
        });
        (backend, record)
    }

    #[test]
    fn partial_removal_restores_host_without_guest_or_replaying_allocations() {
        let (mut backend, record) = partial();
        reset_partial(&mut backend, &record, &record.expected, true).unwrap();
        assert!(backend.state.gpus.is_empty());
        assert_eq!(backend.state.settings, record.initial.settings);
        assert_eq!(backend.state.power, Power::Off);
        assert_eq!(backend.events, ["detach"]);
        assert_eq!(backend.settings_writes, 1);
        assert_eq!(backend.allocation_writes, 0);
        let journal = backend.journal.as_ref().unwrap();
        assert!(!journal.pending && journal.settings_released && journal.prepared.is_none());
        assert!(!journal.setup_eligible());
        assert_eq!(journal.last_verified, Some(123));
        backend.events.clear();
        reset_partial(&mut backend, &record, &record.expected, true).unwrap();
        assert!(backend.events.is_empty());
    }

    fn failed_setup() -> (crate::workflow::test_support::Fake, SaveRecord) {
        let mut backend = fake();
        let mut record = recovery_record(&backend, true);
        record.expected_settings = record.initial.settings.for_gpu();
        backend.fail_prepare = true;
        assert!(crate::workflow::apply(&mut backend, &record.target).is_err());
        assert_eq!(backend.state.power, Power::Running);
        assert!(backend.state.gpus.is_empty());
        record.reset = Some(ResetReview {
            observed: backend.state.clone(),
            gpu: backend.gpu(&record.target).unwrap(),
            journal: backend.journal.clone().unwrap(),
        });
        backend.events.clear();
        (backend, record)
    }

    #[test]
    fn running_first_setup_failure_has_a_separate_host_only_disposition() {
        let (mut backend, record) = failed_setup();
        assert!(reset_partial(&mut backend, &record, &record.expected, false).is_err());
        assert!(backend.events.is_empty());
        assert!(backend.journal.as_ref().unwrap().pending);
        reset_partial(&mut backend, &record, &record.expected, true).unwrap();
        assert_eq!(backend.events, ["shutdown"]);
        assert_eq!(backend.state.power, Power::Off);
        assert_eq!(backend.state.settings, record.initial.settings);
        assert_eq!(backend.settings_writes, 0);
        assert_eq!(backend.allocation_writes, 0);
        let journal = backend.journal.as_ref().unwrap();
        assert!(!journal.pending && !journal.setup_eligible());
        assert_eq!(journal.preparation_started, Some(true));
        assert!(journal.prepared.is_none() && journal.last_verified.is_none());
        backend.events.clear();
        reset_partial(&mut backend, &record, &record.expected, true).unwrap();
        assert!(backend.events.is_empty());
    }

    #[test]
    fn failed_setup_disposition_retains_hold_on_shutdown_drift_or_unavailable_facts() {
        use crate::workflow::test_support::Drift;
        for drift in [
            Drift::Settings,
            Drift::Gpu,
            Drift::Assignment,
            Drift::Resource,
            Drift::Journal,
            Drift::Unavailable,
        ] {
            let (mut backend, record) = failed_setup();
            backend.drift_at = Some(("shutdown", drift));
            assert!(reset_partial(&mut backend, &record, &record.expected, true).is_err());
            assert_eq!(backend.events, ["shutdown"]);
            assert!(backend.journal.as_ref().unwrap().pending);
            assert_eq!(backend.settings_writes, 0);
        }
        for ignore in [false, true] {
            let (mut backend, record) = failed_setup();
            backend.fail_shutdown = !ignore;
            backend.ignore_shutdown = ignore;
            assert!(reset_partial(&mut backend, &record, &record.expected, true).is_err());
            assert!(backend.journal.as_ref().unwrap().pending);
        }
    }

    #[test]
    fn failed_setup_resumes_after_shutdown_without_replaying_it_or_preparation() {
        let (mut backend, record) = failed_setup();
        backend.fail_save_at = Some(backend.journal_writes + 1);
        assert!(reset_partial(&mut backend, &record, &record.expected, true).is_err());
        assert_eq!(backend.events, ["shutdown"]);
        backend.events.clear();
        backend.fail_save_at = None;
        reset_partial(&mut backend, &record, &record.expected, true).unwrap();
        assert!(backend.events.is_empty());
        for case in 0..5 {
            let (mut backend, mut record) = failed_setup();
            match case {
                0 => record.initial_journal = backend.journal.clone(),
                1 => record.reset.as_mut().unwrap().journal.prepared = Some("untrusted".into()),
                2 => record.reset.as_mut().unwrap().journal.last_verified = Some(123),
                3 => backend.inspect_unavailable = true,
                _ => record.expected = Revision::of(b"changed"),
            }
            assert!(reset_partial(&mut backend, &record, &Revision::Missing, true).is_err());
            assert!(backend.events.is_empty());
        }
    }

    #[test]
    fn unprepared_first_setup_never_acquires_external_adapter_or_settings_ownership() {
        for attached in [false, true] {
            let (mut backend, mut record) = failed_setup();
            if attached {
                backend.state.gpus.push(record.target.gpu_interface.clone());
            } else {
                backend.state.settings = record.expected_settings.clone();
            }
            record.reset.as_mut().unwrap().observed = backend.state.clone();
            assert!(reset_partial(&mut backend, &record, &record.expected, true).is_err());
            assert!(backend.events.is_empty());
            assert_eq!(backend.settings_writes, 0);
            assert!(backend.journal.as_ref().unwrap().pending);
        }
    }

    #[test]
    fn changed_review_or_host_or_journal_refuses_removal() {
        for case in 0..6 {
            let (mut backend, mut record) = partial();
            match case {
                0 => backend.state.power = Power::Running,
                1 => backend.state.gpus = vec!["another-gpu".into()],
                2 => backend.state.settings.high_mmio += 1,
                3 => backend.journal.as_mut().unwrap().original.high_mmio += 1,
                4 => backend.gpu_driver = "changed",
                _ => record.expected = Revision::of(b"changed"),
            }
            assert!(reset_partial(&mut backend, &record, &Revision::Missing, true).is_err());
            assert!(backend.events.is_empty());
            assert!(backend.journal.as_ref().unwrap().pending);
        }
    }

    #[test]
    fn interrupted_settings_or_journal_write_resumes_only_bound_transitions() {
        for journal_failure in [false, true] {
            let (mut backend, record) = partial();
            backend.fail_settings = !journal_failure;
            backend.fail_save_at = journal_failure.then_some(1);
            assert!(reset_partial(&mut backend, &record, &record.expected, true).is_err());
            assert!(backend.state.gpus.is_empty());
            assert!(backend.journal.as_ref().unwrap().pending);
            backend.fail_settings = false;
            backend.fail_save_at = None;
            backend.events.clear();
            reset_partial(&mut backend, &record, &record.expected, true).unwrap();
            assert!(!backend.events.contains(&"detach"));
            assert!(!backend.journal.as_ref().unwrap().pending);
        }
        let (mut backend, record) = partial();
        backend.change_settings_on_detach = true;
        assert!(reset_partial(&mut backend, &record, &record.expected, true).is_err());
        assert_eq!(backend.events, ["detach"]);
        assert!(backend.journal.as_ref().unwrap().pending);
    }
}
