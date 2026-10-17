//! Read admission, cooperative cancellation and stale-result suppression.
use hyper_gpu_support::read_context::ReadControl;
use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

#[derive(Default)]
struct Reads {
    next: u64,
    poll: Option<(u64, ReadControl)>,
    foreground: Option<ReadControl>,
}
#[derive(Default)]
pub(in crate::gui) struct Polling {
    generation: AtomicU64,
    reads: Mutex<Reads>,
    released: Condvar,
}
impl Polling {
    pub(in crate::gui) fn begin(self: &Arc<Self>) -> Option<Ticket> {
        let mut reads = self.reads.lock().ok()?;
        if reads.poll.is_some() || reads.foreground.is_some() {
            return None;
        }
        reads.next = reads.next.checked_add(1)?;
        let id = reads.next;
        let control = ReadControl::new(Duration::from_secs(10));
        reads.poll = Some((id, control.clone()));
        Some(Ticket {
            polling: self.clone(),
            generation: self.generation.load(Ordering::Acquire),
            id,
            control,
        })
    }
    pub(in crate::gui) fn invalidate(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        if let Ok(reads) = self.reads.lock()
            && let Some((_, control)) = &reads.poll
        {
            control.cancel();
        }
    }
    pub(super) fn foreground(self: &Arc<Self>) -> Result<Foreground, String> {
        let mut reads = self
            .reads
            .lock()
            .map_err(|_| "Read admission unavailable")?;
        if reads.foreground.is_some() {
            return Err("A protected reader has not confirmed release".into());
        }
        self.generation.fetch_add(1, Ordering::AcqRel);
        if let Some((_, poll)) = &reads.poll {
            poll.cancel();
        }
        let control = ReadControl::new(Duration::from_secs(360));
        reads.foreground = Some(control.clone());
        Ok(Foreground {
            polling: self.clone(),
            control,
        })
    }
    pub(super) fn cancel_preparation(&self) {
        if let Ok(reads) = self.reads.lock()
            && let Some(control) = &reads.foreground
        {
            control.cancel();
        }
    }
}

pub(in crate::gui) struct Ticket {
    polling: Arc<Polling>,
    generation: u64,
    id: u64,
    control: ReadControl,
}
impl Ticket {
    pub(super) fn read<T>(&self, action: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
        self.control.scope(action)
    }
    pub(super) fn backend_finished(&self) {
        if !self.control.backend_released() {
            return;
        }
        if let Ok(mut reads) = self.polling.reads.lock()
            && reads.poll.as_ref().is_some_and(|(id, _)| *id == self.id)
        {
            reads.poll = None;
            self.polling.released.notify_all();
        }
    }
    pub(super) fn can_publish(&self, foreground_busy: bool, modal_open: bool) -> bool {
        !foreground_busy
            && !modal_open
            && self.generation == self.polling.generation.load(Ordering::Acquire)
            && self
                .polling
                .reads
                .lock()
                .is_ok_and(|reads| reads.next == self.id)
    }
}
impl Drop for Ticket {
    fn drop(&mut self) {
        self.backend_finished();
    }
}

pub(super) struct Foreground {
    polling: Arc<Polling>,
    control: ReadControl,
}
impl Foreground {
    pub(super) fn run<T>(
        &self,
        readonly: bool,
        action: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        let mut reads = self
            .polling
            .reads
            .lock()
            .map_err(|_| "Read admission unavailable")?;
        while reads.poll.is_some() {
            self.control.check()?;
            reads = self
                .polling
                .released
                .wait_timeout(reads, Duration::from_millis(25))
                .map_err(|_| "Read release observation unavailable")?
                .0;
        }
        drop(reads);
        self.control.check()?;
        if readonly {
            self.control.scope(action)
        } else {
            self.control.track(action)
        }
    }
    pub(super) fn backend_released(&self) -> bool {
        self.control.backend_released()
    }
}
impl Drop for Foreground {
    fn drop(&mut self) {
        if self.control.backend_released()
            && let Ok(mut reads) = self.polling.reads.lock()
        {
            reads.foreground = None;
            self.polling.released.notify_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn foreground_waits_for_poll_release_and_obsolete_ticket_cannot_release_new_poll() {
        let polling = Arc::new(Polling::default());
        let ticket = polling.begin().unwrap();
        let foreground = polling.foreground().unwrap();
        let (started, observed) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            foreground.run(false, || {
                started.send(()).unwrap();
                Ok(())
            })
        });
        assert!(observed.try_recv().is_err());
        assert!(
            ticket.read(|| Ok(())).is_err(),
            "foreground cancels read-only work"
        );
        ticket.backend_finished();
        observed.recv_timeout(Duration::from_secs(1)).unwrap();
        worker.join().unwrap().unwrap();
        assert!(!ticket.can_publish(false, false));
        let newer = polling.begin().unwrap();
        drop(ticket);
        assert!(
            polling.begin().is_none(),
            "old delivery cannot release a newer read"
        );
        drop(newer);
        assert!(polling.begin().is_some());
    }

    #[test]
    fn a_later_poll_invalidates_delivery_even_without_a_user_interaction() {
        let polling = Arc::new(Polling::default());
        let old = polling.begin().unwrap();
        old.backend_finished();
        let new = polling.begin().unwrap();
        assert!(!old.can_publish(false, false));
        assert!(new.can_publish(false, false));
    }

    #[test]
    fn polling_does_not_claim_foreground_admission_and_prevents_overlapping_reads() {
        let polling = Arc::new(Polling::default());
        let foreground = AtomicBool::new(false);
        let ticket = polling.begin().unwrap();
        assert!(polling.begin().is_none());
        assert!(!foreground.swap(true, Ordering::AcqRel));
        assert!(!ticket.can_publish(foreground.load(Ordering::Acquire), false));
        foreground.store(false, Ordering::Release);
        assert!(ticket.can_publish(false, false));
        drop(ticket);
        assert!(polling.begin().is_some());
    }

    #[test]
    fn review_opened_and_closed_during_poll_discards_old_result() {
        let polling = Arc::new(Polling::default());
        let ticket = polling.begin().unwrap();
        assert!(!ticket.can_publish(false, true));
        polling.invalidate();
        assert!(!ticket.can_publish(false, false));
        drop(ticket);
        assert!(polling.begin().unwrap().can_publish(false, false));
    }

    #[test]
    fn failed_or_panicked_read_releases_poll_admission() {
        let polling = Arc::new(Polling::default());
        let task_polling = polling.clone();
        assert!(
            std::thread::spawn(move || {
                let _ticket = task_polling.begin().unwrap();
                panic!("read fixture failed");
            })
            .join()
            .is_err()
        );
        assert!(polling.begin().is_some());
    }
}
