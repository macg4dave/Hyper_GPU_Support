//! Draft planning and acceptance of fresh ordinary or reinstall reviews.
use super::SharedState;
#[cfg(windows)]
use super::enrollment::review_enrollment;
#[cfg(windows)]
use super::presentation::draft_target;
use crate::gui::AppWindow;
use std::sync::{Arc, atomic::AtomicBool};

pub(super) fn review(
    ui: &AppWindow,
    state: &SharedState,
    busy: &Arc<AtomicBool>,
    reinstall: bool,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        let target = {
            let state = state.lock().map_err(|_| {
                hyper_gpu_support::ui_text::common::state::live_presentation_unavailable()
            })?;
            let target = draft_target(ui, &state)?;
            state.view.proposal_eligibility(&target)?;
            (!state.enrolled(&target)).then_some(target)
        };
        if let Some(target) = target {
            if reinstall {
                return Err(
                    hyper_gpu_support::ui_text::dashboard::actions::reinstall_requires_pair()
                        .into(),
                );
            }
            return review_enrollment(ui, state, busy, target);
        }
    }
    super::apply::apply_live(ui, state, busy, reinstall)
}
