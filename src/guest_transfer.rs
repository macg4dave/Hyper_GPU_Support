//! Bounded, ordered acknowledgements from the fixed guest transfer bridge.
use crate::process::StderrMonitor;
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

const PREFIX: &str = "HGTRANSFER ";
const FILE_LIMIT: Duration = Duration::from_secs(300);
const IDLE_LIMIT: Duration = Duration::from_secs(60);
const GAP_LIMIT: Duration = Duration::from_secs(240);
pub(crate) const CHUNK_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Copy)]
pub(crate) struct Progress {
    pub bytes: u64,
    pub total: u64,
    pub files: usize,
    pub count: usize,
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
}

pub(crate) struct Monitor {
    sizes: Vec<u64>,
    total: u64,
    state: Mutex<State>,
}

impl Monitor {
    pub fn new(sizes: Vec<u64>, now: Instant) -> Result<Self, String> {
        let total = sizes
            .iter()
            .try_fold(0_u64, |sum, bytes| sum.checked_add(*bytes))
            .ok_or("transfer byte total overflow")?;
        Ok(Self {
            sizes,
            total,
            state: Mutex::new(State {
                index: 0,
                bytes: 0,
                completed: 0,
                started: None,
                advanced: now,
                done: false,
                pending: None,
                reported: now,
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
        });
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
        m.accept("done 2 7", now).unwrap();
        m.finish().unwrap();
        let p = m.take_progress(now).unwrap().unwrap();
        assert_eq!((p.bytes, p.total, p.files, p.count), (7, 7, 2, 2));
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
