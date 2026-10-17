//! Draft planning and acceptance of fresh ordinary or reinstall reviews.
#[cfg(windows)]
use super::enrollment::review_enrollment;
#[cfg(windows)]
use super::presentation::draft_target;
use super::{SharedState, source::Source};
use crate::gui::AppWindow;
use std::sync::{Arc, atomic::AtomicBool};

pub(super) fn review(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
) -> Result<(), String> {
    review_operation(ui, state, busy, source, false)
}

pub(super) fn review_operation(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    source: &Source,
    reinstall: bool,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        let target = {
            let mut state = state
                .lock()
                .map_err(|_| "Live presentation state unavailable")?;
            state.enrollment_review = None;
            state.approved_plan = None;
            let target = draft_target(ui, &state)?;
            state.view.proposal_eligibility(&target)?;
            (!state.enrolled(&target)).then_some(target)
        };
        if let Some(target) = target {
            if reinstall {
                return Err(
                    "Repair / Reinstall requires an authorized VM/GPU pair. Review enrollment first.".into(),
                );
            }
            return review_enrollment(ui, state, busy, target);
        }
    }
    let _ = source;
    super::apply::apply_live(ui, state, busy, reinstall)
}
