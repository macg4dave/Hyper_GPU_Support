//! Live GUI state shared by private modules organised by responsibility.
//!
//! `State` remains the controller-facing API. Child modules implement its decisions
//! without owning effects; protected operations still run through the live controller.
mod configuration;
mod draft;
mod inventory;
mod profile;
mod recovery;
mod review;
mod review_text;

pub(super) use draft::allocation_values;
pub(super) use review_text::{allocation_description, memory_review, review_summary, review_text};

use hyper_gpu_support::{gui_model::View, model::Configuration, workflow::Plan};

pub(super) struct State {
    pub polling: std::sync::Arc<super::live::polling::Polling>,
    pub view: View,
    pub result_notice: Option<super::result_notice::ResultNotice>,
    pub pending_selection: Option<String>,
    pub draft_gpu: Option<String>,
    pub approved_plan: Option<ReviewedPlan>,
    /// A rejected Apply needs a full fresh review rather than repeating a prediction.
    pub require_authenticated_review: bool,
    pub removal_review: Option<hyper_gpu_support::enrollment::RemovalReview>,
    pub reset_review: Option<(
        hyper_gpu_support::configuration_store::Revision,
        hyper_gpu_support::workflow::ResetReview,
    )>,
    pub verification_plan: Option<hyper_gpu_support::workflow::VerificationPlan>,
    pub committed: hyper_gpu_support::configuration_store::StoreSnapshot,
    /// A draft's original committed revision changed; only discard releases it.
    committed_conflict: Option<String>,
    authority_conflict: Option<String>,
    pub recovery: Option<hyper_gpu_support::configuration_store::SaveRecord>,
    pub recoveries: Vec<hyper_gpu_support::configuration_store::SaveRecord>,
    pub import_recovery: Option<hyper_gpu_support::configuration_store::ImportRecord>,
    pub enrollment_review: Option<hyper_gpu_support::enrollment::PairReview>,
    pub enrollment_recovery: Option<hyper_gpu_support::enrollment::PendingPair>,
    pub enrollment_recovery_revision: Option<hyper_gpu_support::configuration_store::Revision>,
    pub setup_artifacts: Option<std::collections::BTreeMap<String, String>>,
}

/// The GPU effect review also binds the exact committed intent it was built against.
#[derive(Clone)]
pub(super) struct ReviewedPlan {
    pub plan: Plan,
    pub revision: hyper_gpu_support::configuration_store::Revision,
}

/// Protected reads gathered off the UI thread after an operation attempt.
pub(super) struct OperationReadback {
    pub committed: hyper_gpu_support::configuration_store::StoreSnapshot,
    pub recovery: Vec<hyper_gpu_support::configuration_store::SaveRecord>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            polling: Default::default(),
            view: View::new(Configuration {
                schema: 2,
                targets: vec![],
            }),
            result_notice: None,
            pending_selection: None,
            draft_gpu: None,
            approved_plan: None,
            require_authenticated_review: false,
            removal_review: None,
            reset_review: None,
            verification_plan: None,
            committed: Default::default(),
            committed_conflict: None,
            authority_conflict: None,
            recovery: None,
            recoveries: Vec::new(),
            import_recovery: None,
            enrollment_review: None,
            enrollment_recovery: None,
            enrollment_recovery_revision: None,
            setup_artifacts: None,
        }
    }
}

#[cfg(test)]
mod tests;
