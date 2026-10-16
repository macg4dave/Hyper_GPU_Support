//! App-profile deletion review and completion; no host or guest effects.
use super::State;
use hyper_gpu_support::model::Target;

impl State {
    /// Profile deletion changes app records only, including when GPU support remains active.
    pub fn forget_target(&self) -> Result<Target, String> {
        let id = self.view.selected.as_deref().ok_or("Select a VM first.")?;
        let inventory = self.view.inventory.as_ref().ok_or("Refresh first.")?;
        let mut target = inventory
            .enrolled
            .iter()
            .find(|t| t.vm_id == id)
            .cloned()
            .ok_or("This VM has no authorized pairing; Refresh for current state.")?;
        target.enabled = false;
        target.clear_allocations();
        target.validate()?;
        Ok(target)
    }

    /// The worker's successful terminal reply already certifies protected readback.
    /// Failure retains cached records as unknown until Refresh, never as proof of retention.
    pub fn finish_profile_deletion(
        &mut self,
        id: &str,
        result: Result<(), String>,
    ) -> Result<(), String> {
        self.cancel_review();
        self.view.needs_readback = true;
        result?;
        self.view
            .configuration
            .targets
            .retain(|target| target.vm_id != id);
        self.committed.documents.remove(id);
        self.committed.errors.remove(&format!("{id}.toml"));
        self.recoveries.retain(|record| record.target.vm_id != id);
        if self
            .recovery
            .as_ref()
            .is_some_and(|record| record.target.vm_id == id)
        {
            self.recovery = None;
        }
        if self
            .view
            .draft
            .as_ref()
            .is_some_and(|target| target.vm_id == id)
        {
            self.view.draft = None;
        }
        if self
            .view
            .verified_unsaved
            .as_ref()
            .is_some_and(|target| target.vm_id == id)
        {
            self.view.verified_unsaved = None;
            self.view.unsaved = false;
        }
        if let Some(inventory) = &mut self.view.inventory {
            inventory.enrolled.retain(|target| target.vm_id != id);
            inventory.managed.remove(id);
        }
        Ok(())
    }

    fn check_forget_review(
        &self,
        review: &hyper_gpu_support::enrollment::RemovalReview,
    ) -> Result<(), String> {
        let target = self.forget_target()?;
        let mut revisions = self
            .committed
            .documents
            .iter()
            .map(|(id, d)| (id.clone(), d.revision.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        revisions
            .entry(target.vm_id.clone())
            .or_insert(hyper_gpu_support::configuration_store::Revision::Missing);
        if review.pair.target()? != target
            || (!self.view.needs_readback
                && self.committed.errors.is_empty()
                && review.configurations != revisions)
        {
            return Err(
                "Selection, saved profiles or pairing changed; review Delete profile again.".into(),
            );
        }
        Ok(())
    }

    pub fn accept_forget_review(
        &mut self,
        review: hyper_gpu_support::enrollment::RemovalReview,
    ) -> Result<(), String> {
        self.removal_review = None;
        self.check_forget_review(&review)?;
        self.removal_review = Some(review);
        Ok(())
    }

    pub fn reviewed_forget(&self) -> Result<hyper_gpu_support::enrollment::RemovalReview, String> {
        let review = self
            .removal_review
            .as_ref()
            .ok_or("Open a fresh Delete profile review first.")?;
        self.check_forget_review(review)?;
        Ok(review.clone())
    }
}
