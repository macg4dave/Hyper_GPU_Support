//! Cooperative read-only cancellation/deadlines, distinct from effect execution.
use std::{
    cell::RefCell,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

/// One read journey's cancellation, deadline and confirmed backend-release state.
#[derive(Clone, Debug)]
pub struct ReadControl {
    cancelled: Arc<AtomicBool>,
    pending: Arc<AtomicUsize>,
    deadline: Instant,
}

#[derive(Clone)]
struct Context {
    control: ReadControl,
    checked: bool,
}
thread_local! { static ACTIVE: RefCell<Option<Context>> = const { RefCell::new(None) }; }
struct Restore(Option<Context>);
fn deadline(timeout: Duration) -> Instant {
    let now = Instant::now();
    now.checked_add(timeout).unwrap_or(now)
}
impl Drop for Restore {
    fn drop(&mut self) {
        ACTIVE.with(|active| *active.borrow_mut() = self.0.take());
    }
}

impl ReadControl {
    /// Create a bounded read; this never cancels a VM/GPU/guest effect.
    pub fn new(timeout: Duration) -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            pending: Arc::new(AtomicUsize::new(0)),
            deadline: deadline(timeout),
        }
    }
    /// Request cooperative cancellation without claiming the backend has stopped.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
    /// Deny further read work once cancellation or the whole-read deadline occurs.
    pub fn check(&self) -> Result<(), String> {
        if self.cancelled.load(Ordering::Acquire) {
            Err("Read-only preparation cancelled".into())
        } else if Instant::now() >= self.deadline {
            Err("Read-only operation deadline exceeded".into())
        } else {
            Ok(())
        }
    }
    /// True only when every submitted reader has confirmed release, or none started.
    pub fn backend_released(&self) -> bool {
        self.pending.load(Ordering::Acquire) == 0
    }
    /// Run read-only work with one budget across nested provider/file/IPC reads.
    pub fn scope<T>(&self, action: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
        self.check()?;
        self.install(true, || {
            action().and_then(|value| {
                check()?;
                Ok(value)
            })
        })
    }
    /// Track reader releases around a foreground task without making its effects
    /// cancellable or changing its terminal outcome after execution.
    pub fn track<T>(&self, action: impl FnOnce() -> T) -> T {
        self.install(false, action)
    }
    fn install<T>(&self, checked: bool, action: impl FnOnce() -> T) -> T {
        let previous = ACTIVE.with(|active| {
            active.replace(Some(Context {
                control: self.clone(),
                checked,
            }))
        });
        let _restore = Restore(previous);
        action()
    }
    pub(crate) fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }
    pub(crate) fn shortened(&self, timeout: Duration) -> Self {
        Self {
            deadline: self.deadline.min(deadline(timeout)),
            ..self.clone()
        }
    }
    pub(crate) fn submitted(&self) -> Release {
        self.pending.fetch_add(1, Ordering::AcqRel);
        Release(self.clone())
    }
}

/// Current read control, or a fresh read budget sharing the effect task's release
/// tracker. This does not impose the effect's elapsed time on completion reads.
pub(crate) fn request() -> ReadControl {
    ACTIVE.with(|active| match active.borrow().as_ref() {
        Some(context) if context.checked => context.control.clone(),
        Some(context) => ReadControl {
            deadline: deadline(Duration::from_secs(360)),
            ..context.control.clone()
        },
        None => ReadControl::new(Duration::from_secs(360)),
    })
}

pub(crate) fn check() -> Result<(), String> {
    ACTIVE.with(|active| match active.borrow().as_ref() {
        Some(context) if context.checked => context.control.check(),
        _ => Ok(()),
    })
}

/// Whether an enclosing GUI journey has confirmed every submitted reader's
/// release. Absence of a tracker is not a release acknowledgement.
pub fn tracked_backend_released() -> bool {
    ACTIVE.with(|active| {
        active
            .borrow()
            .as_ref()
            .is_some_and(|context| context.control.backend_released())
    })
}

pub(crate) struct Release(ReadControl);
impl Release {
    /// Consumed only after a terminal reply and/or scheduler readback proves that
    /// the one-shot runner released its lock. Dropping this token proves nothing.
    pub(crate) fn confirm(self) {
        self.0.pending.fetch_sub(1, Ordering::AcqRel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_and_expiry_preserve_admission_until_confirmed_release() {
        for expired in [false, true] {
            let control = ReadControl::new(if expired {
                Duration::ZERO
            } else {
                Duration::from_secs(60)
            });
            let release = control.submitted();
            if expired {
                assert!(control.check().is_err());
            }
            control.cancel();
            assert!(control.check().is_err());
            assert!(!control.backend_released());
            release.confirm();
            assert!(control.backend_released());
        }
    }
    #[test]
    fn dropping_a_reader_without_acknowledgement_retains_its_admission() {
        let control = ReadControl::new(Duration::from_secs(60));
        drop(control.submitted());
        assert!(!control.backend_released());
        control.cancel();
        assert!(!control.backend_released());
    }
    #[test]
    fn a_larger_nested_budget_cannot_extend_the_original_deadline() {
        let control = ReadControl::new(Duration::ZERO);
        assert!(control.shortened(Duration::from_secs(360)).check().is_err());
        assert!(ReadControl::new(Duration::MAX).check().is_err());
    }
    #[test]
    fn effect_tracking_does_not_check_read_cancellation_and_nested_scope_restores() {
        let control = ReadControl::new(Duration::ZERO);
        control.cancel();
        control.track(|| {
            check().unwrap();
            assert!(control.scope(|| Ok(())).is_err());
            check().unwrap();
        });
        check().unwrap();
        let control = ReadControl::new(Duration::from_secs(60));
        assert!(std::panic::catch_unwind(|| control.scope::<()>(|| panic!("fixture"))).is_err());
        check().unwrap();
    }
}
