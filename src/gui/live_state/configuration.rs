//! Readable saved intent and latched conflicts with retained drafts.
use super::State;
use crate::gui::host_configuration;
use hyper_gpu_support::model::Target;

impl State {
    /// Accept fresh readable intent; an error means the retained draft is blocked,
    /// not that the new store snapshot or unrelated inventory should be discarded.
    pub fn load_committed(
        &mut self,
        committed: hyper_gpu_support::configuration_store::StoreSnapshot,
    ) -> Result<(), String> {
        if let Some(draft) = &self.view.draft {
            let unchanged = match (
                self.committed.revision(&draft.vm_id),
                committed.revision(&draft.vm_id),
            ) {
                (Ok(previous), Ok(current)) => previous == current,
                _ => false,
            };
            if !unchanged {
                self.committed_conflict = Some(draft.vm_id.clone());
                self.approved_plan = None;
            }
        }
        // Refresh all readable intent even when one draft conflicts. This never
        // changes the draft or grants enrollment, and the conflict remains latched.
        self.view.configuration = committed.configuration();
        self.committed = committed;
        if let Some(id) = &self.committed_conflict {
            return self.editor_eligibility(id);
        }
        Ok(())
    }

    /// Current readable publication, separately from an earlier terminal reply.
    pub fn publication_confirmed(&self, target: &Target) -> bool {
        !self.view.unsaved
            && self.committed.revision(&target.vm_id).is_ok()
            && self
                .committed
                .documents
                .get(&target.vm_id)
                .is_some_and(|d| d.target == *target)
    }

    pub fn saved_desired_text(&self, id: &str) -> String {
        match self.committed.revision(id) {
            Ok(_) => self.view.saved_desired_text(id),
            Err(_) => "Committed configuration unavailable; saved intent is unknown. Editing is blocked until the file can be read and validated.".into(),
        }
    }

    pub fn host_configuration_text(&self, id: &str) -> String {
        host_configuration::comparison(self, id)
    }
}
