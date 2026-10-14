//! Progress decisions from immutable worker input, before editable row rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Status {
    Running,
    Done,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Identity {
    Transfer,
    Stage(String),
}

#[derive(Clone, Debug)]
pub(super) struct Event {
    pub identity: Identity,
    pub status: Status,
    pub fraction: Option<f32>,
    pub persistence: bool,
    pub copy_complete: bool,
    pub reportable: bool,
}

impl Event {
    pub fn from_worker(stage: &str, status: Status) -> Self {
        let transfer = stage.starts_with(hyper_gpu_support::reporting::TRANSFER_PROGRESS_PREFIX);
        let fraction = transfer
            .then(|| {
                stage
                    .split_once('(')?
                    .1
                    .split_once("%)")?
                    .0
                    .parse::<u8>()
                    .ok()
            })
            .flatten()
            .filter(|percent| *percent <= 100)
            .map(|percent| f32::from(percent) / 100.0);
        Self {
            identity: if transfer {
                Identity::Transfer
            } else {
                Identity::Stage(stage.into())
            },
            status,
            fraction,
            persistence: stage == "Persist operation recovery",
            copy_complete: stage == "File copying complete; waiting for guest checks",
            reportable: hyper_gpu_support::reporting::safe_progress_input(stage),
        }
    }
}

#[derive(Default)]
pub(super) struct State {
    history: Vec<(Identity, Status, bool)>,
    steps: Vec<Identity>,
    active: Vec<Identity>,
}

impl State {
    /// Returns the history slot, coalescing only adjacent running transfer events.
    pub fn record(&mut self, event: &Event) -> usize {
        if !event.persistence {
            if event.copy_complete {
                self.active
                    .retain(|identity| *identity != Identity::Transfer);
            }
            if event.status == Status::Running {
                self.active.retain(|identity| *identity != event.identity);
                self.active.push(event.identity.clone());
            } else if let Some(slot) = self
                .active
                .iter()
                .position(|identity| *identity == event.identity)
            {
                // An enclosing stage can fail before its children close (for
                // example a transfer loses its terminal acknowledgement).
                self.active.truncate(slot);
            }
        }
        if let Some(index) = self.history.len().checked_sub(1)
            && self.history[index].0 == Identity::Transfer
            && self.history[index].1 == Status::Running
            && event.identity == Identity::Transfer
            && event.status == Status::Running
        {
            self.history[index] = (event.identity.clone(), event.status, event.reportable);
            index
        } else {
            self.history
                .push((event.identity.clone(), event.status, event.reportable));
            self.history.len() - 1
        }
    }

    pub fn step(&mut self, identity: &Identity) -> usize {
        if let Some(index) = self.steps.iter().position(|previous| previous == identity) {
            index
        } else {
            self.steps.push(identity.clone());
            self.steps.len() - 1
        }
    }

    pub fn reportable(&self) -> impl Iterator<Item = bool> + '_ {
        self.history.iter().map(|item| item.2)
    }

    /// A completed nested read returns attention to its still-running parent.
    /// No running stage means waiting for the terminal reply, never success.
    pub fn active_step(&self) -> Option<usize> {
        let identity = self.active.last()?;
        self.steps.iter().position(|step| step == identity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transfer(status: Status, fraction: Option<f32>) -> Event {
        Event {
            identity: Identity::Transfer,
            status,
            fraction,
            persistence: false,
            copy_complete: false,
            reportable: true,
        }
    }

    #[test]
    fn running_updates_reuse_identity_without_inspecting_display_rows() {
        let mut state = State::default();
        let first = transfer(Status::Running, Some(0.1));
        let next = transfer(Status::Running, Some(0.8));
        assert_eq!(state.record(&first), 0);
        assert_eq!(state.record(&next), 0);
        assert_eq!(state.step(&first.identity), state.step(&next.identity));
        assert_eq!(state.history.len(), 1);
        assert_eq!(next.fraction, Some(0.8));
    }

    #[test]
    fn terminal_events_and_different_stages_are_never_coalesced() {
        let mut state = State::default();
        assert_eq!(state.record(&transfer(Status::Running, None)), 0);
        assert_eq!(state.record(&transfer(Status::Failed, None)), 1);
        assert_eq!(state.record(&transfer(Status::Running, None)), 2);
        assert_eq!(state.record(&transfer(Status::Done, None)), 3);
        assert_eq!(state.record(&transfer(Status::Running, None)), 4);
        let event = Event {
            identity: Identity::Stage(String::from_iter(['x'])),
            ..transfer(Status::Running, None)
        };
        assert_eq!(state.record(&event), 5);
        assert_eq!(state.record(&transfer(Status::Running, None)), 6);
        assert_eq!(state.step(&event.identity), 0);
        assert_eq!(state.step(&Identity::Transfer), 1);
        assert_eq!(state.step(&event.identity), 0);
    }

    #[test]
    fn nested_reads_return_to_running_parent_then_wait_for_terminal_reply() {
        let mut state = State::default();
        let parent = Event::from_worker("Checking reviewed VM and driver files", Status::Running);
        let read = Event::from_worker("Authenticate current signed driver files", Status::Running);
        state.step(&parent.identity);
        state.step(&read.identity);
        state.record(&parent);
        assert_eq!(state.active_step(), Some(0));
        state.record(&read);
        assert_eq!(state.active_step(), Some(1));
        state.record(&Event {
            status: Status::Done,
            ..read
        });
        assert_eq!(state.active_step(), Some(0));
        // Durable journal progress must not obscure the current read.
        state.record(&Event::from_worker(
            "Persist operation recovery",
            Status::Running,
        ));
        assert_eq!(state.active_step(), Some(0));
        state.record(&Event {
            status: Status::Done,
            ..parent
        });
        assert_eq!(state.active_step(), None);
    }

    #[test]
    fn failed_read_and_repeated_reads_do_not_leave_a_completed_stage_active() {
        let mut state = State::default();
        let read = Event::from_worker("Read current VM state", Status::Running);
        state.step(&read.identity);
        for terminal in [Status::Failed, Status::Done] {
            state.record(&read);
            state.record(&read);
            assert_eq!(state.active_step(), Some(0));
            state.record(&Event {
                status: terminal,
                ..read.clone()
            });
            assert_eq!(state.active_step(), None);
        }
    }

    #[test]
    fn copy_completion_and_failed_parent_retire_unclosed_transfer_progress() {
        let mut state = State::default();
        let parent = Event::from_worker("Prepare current signed payload", Status::Running);
        let copy = transfer(Status::Running, Some(0.8));
        state.step(&parent.identity);
        state.step(&copy.identity);
        state.record(&parent);
        state.record(&copy);
        let copied = Event::from_worker(
            "File copying complete; waiting for guest checks",
            Status::Running,
        );
        state.step(&copied.identity);
        state.record(&copied);
        assert_eq!(state.active_step(), Some(2));
        state.record(&Event {
            status: Status::Done,
            ..copied
        });
        assert_eq!(state.active_step(), Some(0));
        state.record(&copy);
        state.record(&Event {
            status: Status::Failed,
            ..parent
        });
        assert_eq!(state.active_step(), None);
    }

    #[test]
    fn immutable_counter_format_supplies_only_bounded_copy_fraction() {
        let stage = hyper_gpu_support::reporting::transfer_progress(8, 10, 1, 2);
        let event = Event::from_worker(&stage, Status::Running);
        assert_eq!(event.identity, Identity::Transfer);
        assert_eq!(event.fraction, Some(0.8));
        let empty = hyper_gpu_support::reporting::transfer_progress(0, 0, 0, 0);
        let invalid = hyper_gpu_support::reporting::transfer_progress(11, 10, 0, 1);
        for stage in [&empty, &invalid] {
            assert_eq!(Event::from_worker(stage, Status::Running).fraction, None);
        }
        let phase = String::from_iter(['x']);
        assert_eq!(Event::from_worker(&phase, Status::Done).fraction, None);
    }
}
