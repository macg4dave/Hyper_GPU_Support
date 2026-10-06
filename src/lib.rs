//! Core library for the Windows GPU-PV CLI.
//!
//! The inventory boundary is read-only and keeps Windows process access behind a
//! replaceable source so report logic can be tested without Hyper-V or a GPU.

pub mod account_rights;
pub mod cli;
pub mod config;
pub mod guest;
pub mod inventory;
pub mod probe;
pub mod runner;
pub mod staging;
pub mod validation;
pub mod vm_settings;

#[cfg(windows)]
pub mod driver_environment;
#[cfg(windows)]
pub mod environment_staging;
#[cfg(windows)]
pub mod windows_environment_staging;

#[cfg(windows)]
pub mod windows_account_rights;
#[cfg(windows)]
pub mod windows_driver_environment;
#[cfg(windows)]
pub mod windows_guest;
#[cfg(windows)]
pub mod windows_inventory;
#[cfg(windows)]
pub mod windows_native_inventory;
#[cfg(windows)]
pub mod windows_paths;
#[cfg(windows)]
pub mod windows_probe;
#[cfg(windows)]
pub mod windows_runner;
#[cfg(windows)]
pub mod windows_validation;
