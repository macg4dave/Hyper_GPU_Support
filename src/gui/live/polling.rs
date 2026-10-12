//! Independent admission and cancellation for read-only inventory polling.
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

#[derive(Default)]
pub(in crate::gui) struct Polling {
    active: AtomicBool,
    generation: AtomicU64,
}

impl Polling {
    pub(in crate::gui) fn begin(self: &Arc<Self>) -> Option<Ticket> {
        if self.active.swap(true, Ordering::AcqRel) {
            return None;
        }
        Some(Ticket {
            polling: self.clone(),
            generation: self.generation.load(Ordering::Acquire),
        })
    }

    /// An interaction invalidates even a poll that finishes after the dialog closes.
    pub(in crate::gui) fn invalidate(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
    }
}

pub(in crate::gui) struct Ticket {
    polling: Arc<Polling>,
    generation: u64,
}

impl Ticket {
    pub(super) fn can_publish(&self, foreground_busy: bool, modal_open: bool) -> bool {
        !foreground_busy
            && !modal_open
            && self.generation == self.polling.generation.load(Ordering::Acquire)
    }
}

impl Drop for Ticket {
    fn drop(&mut self) {
        self.polling.active.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
