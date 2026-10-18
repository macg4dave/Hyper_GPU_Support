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
pub(super) use review_text::{allocation_description, review_text};

use hyper_gpu_support::{gui_model::View, model::Configuration};

pub(super) struct State {
    pub polling: std::sync::Arc<super::live::polling::Polling>,
    pub observation: Option<hyper_gpu_support::observation::Provenance>,
    pub worker_available: bool,
    pub view: View,
    pub result_notice: Option<super::result_notice::ResultNotice>,
    pub pending_selection: Option<String>,
    pub draft_gpu: Option<String>,
    pub committed: hyper_gpu_support::configuration_store::StoreSnapshot,
    /// A draft's original committed revision changed; only discard releases it.
    committed_conflict: Option<String>,
    authority_conflict: Option<String>,
    pub recovery: Option<hyper_gpu_support::configuration_store::SaveRecord>,
    pub recoveries: Vec<hyper_gpu_support::configuration_store::SaveRecord>,
    pub import_recovery: Option<hyper_gpu_support::configuration_store::ImportRecord>,
    pub enrollment_recovery: Option<hyper_gpu_support::enrollment::PendingPair>,
}

/// Protected reads gathered off the UI thread after an operation attempt.
pub(super) struct OperationReadback {
    pub observation: Option<hyper_gpu_support::observation::Bundle>,
    pub committed: hyper_gpu_support::configuration_store::StoreSnapshot,
    pub recovery: Vec<hyper_gpu_support::configuration_store::SaveRecord>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            polling: Default::default(),
            observation: None,
            worker_available: false,
            view: View::new(Configuration {
                schema: 2,
                targets: vec![],
            }),
            result_notice: None,
            pending_selection: None,
            draft_gpu: None,
            committed: Default::default(),
            committed_conflict: None,
            authority_conflict: None,
            recovery: None,
            recoveries: Vec::new(),
            import_recovery: None,
            enrollment_recovery: None,
        }
    }
}

#[cfg(test)]
mod tests;
