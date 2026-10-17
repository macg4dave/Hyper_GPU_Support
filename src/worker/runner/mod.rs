//! Protected runtime enrollment and a bounded authenticated one-shot product runner.
//!
//! The crate-root `runner` re-export preserves the existing API. Private modules
//! own wire contracts, IPC/admission, audit, policy, pair authority, installation,
//! task scheduling, protected storage, recovery, and the native workflow adapter.
//! Cross-module imports name the owning module; this root only exposes contracts.

mod audit;
mod backend;
mod installation;
mod ipc;
mod observation;
mod pairing;
mod policy;
mod profile_removal;
mod protocol;
mod recovery;
mod reinstall;
mod scheduler;
mod storage;

pub(crate) use backend::NativeBackend;
pub(crate) use installation::{bootstrap_install, reconcile_setup};
pub use installation::{install, installed, pending_setup};
pub use ipc::{serve, submit};
pub(crate) use pairing::{enroll_pair, pair_pending_path, reconcile_pair};
pub use pairing::{pair_revision, pending_pair, review_pair};
pub(crate) use policy::{Enrollment, enrollment, verify_artifacts};
pub(crate) use profile_removal::forget_pair;
pub use profile_removal::review_removal;
pub use protocol::{FRAME_LIMIT, FrameError, Operation, Request, read_frame, request, write_frame};
pub(crate) use recovery::{
    import_path, new_recovery_path, prepare_vm_recovery, read_recovery_at, recovery_record_path,
    require_no_recovery, require_no_vm_recovery, retire_recovery_at, vm_receipt_name,
    write_recovery_at,
};
pub use recovery::{
    import_record, reconcile_native, recovery_for_operation, recovery_for_vm, recovery_records,
    review_native,
};
pub(crate) use storage::{atomic_json, folder, operation_lock, protect};
pub use storage::{data_directory, install_directory};
