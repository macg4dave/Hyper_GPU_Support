//! Approval binding and invalidation for the current draft and saved revision.
use super::State;
use hyper_gpu_support::{model::Target, workflow::Plan};

impl State {
    /// Permission completion can open a fresh review only for the same draft
    /// after independent enrollment/inventory readback. It never approves Apply.
    pub fn enrollment_continuation(
        &self,
        expected: &Target,
        current: &Target,
    ) -> Result<(), String> {
        if expected != current
            || self.view.selected.as_deref() != Some(expected.vm_id.as_str())
            || !self.enrolled(expected)
        {
            return Err(
                hyper_gpu_support::ui_text::operations::enrollment::permission_changed().into(),
            );
        }
        self.editor_eligibility(&current.vm_id)?;
        self.view.proposal_eligibility(current)
    }

    /// A prepared permission request belongs to one captured selection and draft.
    /// The worker independently checks authority and package revisions.
    pub fn validate_permission_preparation(
        &self,
        expected: &Target,
        current: &Target,
        pair: &hyper_gpu_support::enrollment::PairReview,
    ) -> Result<(), String> {
        if current != expected
            || self.view.selected.as_deref() != Some(expected.vm_id.as_str())
            || current.vm_id != pair.vm_id
            || current.gpu_interface != pair.gpu_interface
        {
            return Err(
                hyper_gpu_support::ui_text::operations::enrollment::changed_during_preparation()
                    .into(),
            );
        }
        self.editor_eligibility(&current.vm_id)?;
        self.view.proposal_eligibility(current)
    }

    pub fn validate_prepared(
        &self,
        plan: &Plan,
        current: &Target,
        revision: &hyper_gpu_support::configuration_store::Revision,
    ) -> Result<(), String> {
        self.editor_eligibility(&current.vm_id)?;
        self.view.eligibility(current)?;
        if self.view.selected.as_deref() != Some(current.vm_id.as_str())
            || plan.desired != *current
            || plan.observed.vm_id != current.vm_id
            || self.committed.revision(&current.vm_id)? != *revision
        {
            return Err(
                hyper_gpu_support::ui_text::operations::apply::review::changed_while_planning()
                    .into(),
            );
        }
        Ok(())
    }
}
