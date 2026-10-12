//! Bounded, ordered acknowledgements from the fixed guest transfer bridge.
use crate::{process::StderrMonitor, workflow::PreparationProgress};
use std::collections::VecDeque;
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

const PHASE_PREFIX: &str = "HGPREPARATION ";
const PREFIX: &str = "HGTRANSFER ";
const FILE_LIMIT: Duration = Duration::from_secs(300);
const IDLE_LIMIT: Duration = Duration::from_secs(60);
const GAP_LIMIT: Duration = Duration::from_secs(240);
/// Fixed maximum transfer chunk, also injected into the PowerShell sender.
pub(crate) const CHUNK_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Clone, Copy)]
pub(crate) struct Progress {
    pub bytes: u64,
    pub total: u64,
    pub files: usize,
    pub count: usize,
    pub done: bool,
}

struct State {
    index: usize,
    bytes: u64,
    completed: u64,
    started: Option<Instant>,
    advanced: Instant,
    done: bool,
    pending: Option<Progress>,
    reported: Instant,
    phase_index: usize,
    phases: VecDeque<PreparationProgress>,
}

pub(crate) struct Monitor {
    sizes: Vec<u64>,
    total: u64,
    verification: bool,
    state: Mutex<State>,
}

impl Monitor {
    #[cfg(test)]
    pub fn new(sizes: Vec<u64>, now: Instant) -> Result<Self, String> {
        Self::for_operation(sizes, now, false)
    }

    pub fn for_operation(
        sizes: Vec<u64>,
        now: Instant,
        verification: bool,
    ) -> Result<Self, String> {
        let total = sizes
            .iter()
            .try_fold(0_u64, |sum, bytes| sum.checked_add(*bytes))
            .ok_or("transfer byte total overflow")?;
        Ok(Self {
            sizes,
            total,
            verification,
            state: Mutex::new(State {
                index: 0,
                bytes: 0,
                completed: 0,
                started: None,
                advanced: now,
                done: false,
                pending: None,
                reported: now,
                phase_index: 0,
                phases: VecDeque::new(),
            }),
        })
    }

    fn accept(&self, text: &str, now: Instant) -> Result<(), String> {
        let fields: Vec<_> = text.split_ascii_whitespace().collect();
        if fields.len() != 3 {
            return Err("malformed transfer acknowledgement".into());
        }
        let index: usize = fields[1].parse().map_err(|_| "invalid transfer index")?;
        let bytes: u64 = fields[2]
            .parse()
            .map_err(|_| "invalid transfer byte count")?;
        let mut s = self.state.lock().map_err(|_| "transfer monitor poisoned")?;
        Self::check_state(&s, now)?;
        if s.done || index != s.index {
            return Err("out-of-order transfer acknowledgement".into());
        }
        match fields[0] {
            "begin" if s.started.is_none() && index < self.sizes.len() && bytes == 0 => {
                s.started = Some(now);
                s.advanced = now;
            }
            "bytes"
                if s.started.is_some()
                    && bytes > s.bytes
                    && bytes <= self.sizes[index]
                    && bytes - s.bytes <= CHUNK_BYTES =>
            {
                s.bytes = bytes;
                s.advanced = now;
            }
            "end" if s.started.is_some() && bytes == s.bytes && bytes == self.sizes[index] => {
                s.completed += bytes;
                s.index += 1;
                s.bytes = 0;
                s.started = None;
                s.advanced = now;
            }
            "done" if s.started.is_none() && index == self.sizes.len() && bytes == s.completed => {
                s.done = true;
            }
            _ => return Err("invalid transfer transition or byte count".into()),
        }
        s.pending = Some(Progress {
            bytes: s.completed + s.bytes,
            total: self.total,
            files: s.index,
            count: self.sizes.len(),
            done: s.done,
        });
        Ok(())
    }

    fn accept_phase(&self, code: &str) -> Result<(), String> {
        let sequence: &[(&str, PreparationProgress)] = if self.verification {
            &[
                ("checking-guest", PreparationProgress::CheckingGuestFiles),
                ("graphics", PreparationProgress::CheckingGraphics),
            ]
        } else {
            &[
                ("checking-copied", PreparationProgress::CheckingCopiedFiles),
                ("installing", PreparationProgress::InstallingFiles),
                (
                    "checking-installed",
                    PreparationProgress::CheckingInstalledFiles,
                ),
            ]
        };
        let mut state = self.state.lock().map_err(|_| "transfer monitor poisoned")?;
        let Some((expected, event)) = sequence.get(state.phase_index) else {
            return Err("unexpected guest phase event".into());
        };
        if !state.done || code != *expected {
            return Err("out-of-order guest phase event".into());
        }
        state.phase_index += 1;
        state.phases.push_back(*event);
        Ok(())
    }

    /// Small ordered phase queue, independent of coalesced transfer counters.
    pub fn take_phases(&self) -> Result<Vec<PreparationProgress>, String> {
        let mut state = self.state.lock().map_err(|_| "transfer monitor poisoned")?;
        Ok(state.phases.drain(..).collect())
    }

    /// A successful worker reply must include every required phase boundary.
    pub fn require_worker_phases(&self) -> Result<(), String> {
        let state = self.state.lock().map_err(|_| "transfer monitor poisoned")?;
        let expected = if self.verification { 2 } else { 3 };
        if !state.done || state.phase_index != expected {
            return Err("guest worker ended without complete phase evidence".into());
        }
        Ok(())
    }

    fn check_state(s: &State, now: Instant) -> Result<(), String> {
        if s.done {
            return Ok(());
        }
        if s.started
            .is_some_and(|start| now.duration_since(start) >= FILE_LIMIT)
        {
            return Err("guest transfer exceeded per-file deadline; partial preparation requires reconciliation".into());
        }
        let limit = if s.started.is_some() {
            IDLE_LIMIT
        } else {
            GAP_LIMIT
        };
        if now.duration_since(s.advanced) >= limit {
            return Err("guest transfer made no acknowledged progress; partial preparation requires reconciliation".into());
        }
        Ok(())
    }

    // Coalesce reader-thread records. IPC runs only on the supervising caller,
    // outside the state lock, at most four times a second except final completion.
    pub fn take_progress(&self, now: Instant) -> Result<Option<Progress>, String> {
        let mut s = self.state.lock().map_err(|_| "transfer monitor poisoned")?;
        if !s.done && now.duration_since(s.reported) < Duration::from_millis(250) {
            return Ok(None);
        }
        s.reported = now;
        Ok(s.pending.take())
    }
}

impl StderrMonitor for Monitor {
    fn line(&self, line: &[u8]) -> Result<bool, String> {
        if let Some(code) = line.strip_prefix(PHASE_PREFIX.as_bytes()) {
            self.accept_phase(
                std::str::from_utf8(code).map_err(|_| "invalid guest phase encoding")?,
            )?;
            return Ok(true);
        }
        if !line.starts_with(PREFIX.as_bytes()) {
            return Ok(false);
        }
        let text = std::str::from_utf8(&line[PREFIX.len()..])
            .map_err(|_| "invalid transfer acknowledgement encoding")?;
        self.accept(text, Instant::now())?;
        Ok(true)
    }
    fn check(&self, now: Instant) -> Result<(), String> {
        let state = self.state.lock().map_err(|_| "transfer monitor poisoned")?;
        Self::check_state(&state, now)
    }
    fn finish(&self) -> Result<(), String> {
        if !self
            .state
            .lock()
            .map_err(|_| "transfer monitor poisoned")?
            .done
        {
            return Err("guest transfer ended without complete acknowledgements".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn worker_phases_are_ordered_mode_bound_and_never_inferred_from_copy_counters() {
        let now = Instant::now();
        let monitor = Monitor::new(vec![], now).unwrap();
        assert!(monitor.accept_phase("checking-copied").is_err());
        monitor.accept("done 0 0", now).unwrap();
        assert!(monitor.take_phases().unwrap().is_empty());
        assert!(monitor.require_worker_phases().is_err());
        for invalid in ["installing", "graphics", "unknown", "checking-copied extra"] {
            assert!(monitor.accept_phase(invalid).is_err());
        }
        monitor.accept_phase("checking-copied").unwrap();
        assert!(monitor.accept_phase("checking-copied").is_err());
        monitor.accept_phase("installing").unwrap();
        assert!(monitor.require_worker_phases().is_err());
        monitor.accept_phase("checking-installed").unwrap();
        monitor.require_worker_phases().unwrap();
        assert_eq!(
            monitor.take_phases().unwrap(),
            vec![
                PreparationProgress::CheckingCopiedFiles,
                PreparationProgress::InstallingFiles,
                PreparationProgress::CheckingInstalledFiles
            ]
        );
        assert!(monitor.take_phases().unwrap().is_empty());
        assert!(monitor.accept_phase("checking-installed").is_err());
        let verify = Monitor::for_operation(vec![], now, true).unwrap();
        verify.accept("done 0 0", now).unwrap();
        assert!(verify.accept_phase("checking-copied").is_err());
        verify.accept_phase("checking-guest").unwrap();
        verify.accept_phase("graphics").unwrap();
        assert_eq!(
            verify.take_phases().unwrap(),
            vec![
                PreparationProgress::CheckingGuestFiles,
                PreparationProgress::CheckingGraphics
            ]
        );
    }

    #[test]
    fn four_mib_chunks_and_partial_tail_preserve_exact_file_accounting() {
        let now = Instant::now();
        let chunk = 4 * 1024 * 1024;
        let total = chunk * 2 + 37;
        let m = Monitor::new(vec![total, 0], now).unwrap();
        m.accept("begin 0 0", now).unwrap();
        for bytes in [chunk, chunk * 2] {
            m.accept(&format!("bytes 0 {bytes}"), now).unwrap();
        }
        assert!(m.accept(&format!("end 0 {}", chunk * 2), now).is_err());
        m.accept(&format!("bytes 0 {total}"), now).unwrap();
        m.accept(&format!("end 0 {total}"), now).unwrap();
        m.accept("begin 1 0", now).unwrap();
        m.accept("end 1 0", now).unwrap();
        m.accept(&format!("done 2 {total}"), now).unwrap();
        m.finish().unwrap();
        let progress = m.take_progress(now).unwrap().unwrap();
        assert_eq!(
            (progress.bytes, progress.total, progress.files),
            (total, total, 2)
        );
    }

    #[test]
    fn oversized_or_repeated_four_mib_acknowledgements_do_not_advance_progress() {
        let now = Instant::now();
        let chunk = 4 * 1024 * 1024;
        let m = Monitor::new(vec![chunk * 3], now).unwrap();
        m.accept("begin 0 0", now).unwrap();
        assert!(m.accept(&format!("bytes 0 {}", chunk + 1), now).is_err());
        m.accept(&format!("bytes 0 {chunk}"), now).unwrap();
        assert!(m.accept(&format!("bytes 0 {chunk}"), now).is_err());
        assert!(
            m.accept(&format!("bytes 0 {}", chunk * 2 + 1), now)
                .is_err()
        );
        assert!(m.check(now + IDLE_LIMIT).is_err());
        assert!(m.finish().is_err());
    }

    #[test]
    fn acknowledged_bytes_are_ordered_and_never_imply_verification() {
        let now = Instant::now();
        let m = Monitor::new(vec![7, 0], now).unwrap();
        m.accept("begin 0 0", now).unwrap();
        m.accept("bytes 0 4", now).unwrap();
        assert!(m.accept("bytes 0 4", now).is_err());
        assert!(m.accept("bytes 0 8", now).is_err());
        assert!(m.accept("end 0 4", now).is_err());
        m.accept("bytes 0 7", now).unwrap();
        m.accept("end 0 7", now).unwrap();
        assert!(m.finish().is_err());
        m.accept("begin 1 0", now).unwrap();
        m.accept("end 1 0", now).unwrap();
        let copying = m
            .take_progress(now + Duration::from_millis(250))
            .unwrap()
            .unwrap();
        assert!(!copying.done); // 100% copying is not the post-copy worker boundary.
        m.accept("done 2 7", now + Duration::from_millis(250))
            .unwrap();
        m.finish().unwrap();
        let p = m.take_progress(now).unwrap().unwrap();
        assert_eq!((p.bytes, p.total, p.files, p.count), (7, 7, 2, 2));
        assert!(p.done);
        assert!(m.accept("done 2 7", now).is_err());
    }
    #[test]
    fn heartbeats_and_progress_cannot_extend_hard_file_deadline() {
        let now = Instant::now();
        let m = Monitor::new(vec![100], now).unwrap();
        assert!(m.check(now + GAP_LIMIT).is_err());
        m.accept("begin 0 0", now).unwrap();
        assert!(m.check(now + IDLE_LIMIT).is_err());
        for i in 1..6 {
            m.accept(&format!("bytes 0 {i}"), now + Duration::from_secs(i * 50))
                .unwrap();
        }
        assert!(m.accept("bytes 0 6", now + FILE_LIMIT).is_err());
    }
    #[test]
    fn malformed_skipped_oversized_and_incomplete_records_fail_closed() {
        let now = Instant::now();
        let m = Monitor::new(vec![CHUNK_BYTES + 1], now).unwrap();
        for record in [
            "begin 1 0",
            "done 0 0",
            "bytes 0 1",
            "begin 0 0 extra",
            "begin x 0",
        ] {
            assert!(m.accept(record, now).is_err());
        }
        m.accept("begin 0 0", now).unwrap();
        assert!(
            m.accept(&format!("bytes 0 {}", CHUNK_BYTES + 1), now)
                .is_err()
        );
        assert!(m.finish().is_err());
        assert!(Monitor::new(vec![u64::MAX, 1], now).is_err());
    }
}
