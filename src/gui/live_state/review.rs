//! Approval binding and invalidation for the current draft and saved revision.
use super::{ReviewedPlan, State};
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
                "The target, draft or GPU management permission changed. Review again.".into(),
            );
        }
        self.editor_eligibility(&current.vm_id)?;
        self.view.proposal_eligibility(current)
    }

    pub fn accept_review(
        &mut self,
        plan: Plan,
        current: &Target,
        revision: hyper_gpu_support::configuration_store::Revision,
    ) -> Result<(), String> {
        self.approved_plan = None;
        self.editor_eligibility(&current.vm_id)?;
        self.view.eligibility(current)?;
        if self.view.selected.as_deref() != Some(current.vm_id.as_str())
            || plan.desired != *current
            || plan.observed.vm_id != current.vm_id
            || self.committed.revision(&current.vm_id)? != revision
        {
            return Err(
                "The target, draft or saved settings changed while planning. Review again.".into(),
            );
        }
        self.approved_plan = Some(ReviewedPlan { plan, revision });
        Ok(())
    }

    pub fn reviewed_plan(&self, current: &Target) -> Result<ReviewedPlan, String> {
        self.editor_eligibility(&current.vm_id)?;
        self.view.eligibility(current)?;
        let reviewed = self
            .approved_plan
            .as_ref()
            .ok_or("Review the current draft before Apply")?;
        if self.view.selected.as_deref() != Some(current.vm_id.as_str())
            || *current != reviewed.plan.desired
            || self.committed.revision(&current.vm_id)? != reviewed.revision
        {
            return Err(
                "The target, draft or saved settings changed after review. Review again.".into(),
            );
        }
        Ok(reviewed.clone())
    }

    pub fn cancel_review(&mut self) {
        self.result_notice = None;
        self.reset_review = None;
        self.removal_review = None;
        self.approved_plan = None;
        self.enrollment_review = None;
        self.setup_artifacts = None;
        self.verification_plan = None;
        self.enrollment_recovery_revision = None;
    }
}
