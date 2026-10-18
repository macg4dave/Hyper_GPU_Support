//! Restricted elevated execution for approved plans, with private process-bound IPC.
//!
//! Public entrypoints remain here through re-exports. Private modules own wire
//! contracts, launch identity, client/server transport, audited dispatch, command
//! handlers and progress observation; each imports its dependencies explicitly.
mod client;
mod configuration_import;
mod dispatch;
mod operations;
mod pairing;
mod preparation;
mod process;
mod progress;
mod protocol;
mod recovery;
pub mod runner;
mod server;

pub use client::{
    enroll_pair, forget_pair, import, reconcile, reconcile_pair, reset_partial, resume_import,
    save_only,
};
pub use preparation::{Authorization, PreparedReview, apply_prepared, verify_prepared};
pub use process::available;
pub use protocol::{Outcome, Progress, StageStatus};
pub use server::serve;

pub(crate) use process::{OwnedProcess, launch_setup, pipe_name, random_session, valid_session};

#[cfg(test)]
mod test_support;
