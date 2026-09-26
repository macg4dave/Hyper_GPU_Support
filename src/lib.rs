//! Core library for the Windows GPU-PV CLI.
//!
//! The inventory boundary is read-only and keeps Windows process access behind a
//! replaceable source so report logic can be tested without Hyper-V or a GPU.

pub mod cli;
pub mod config;
pub mod inventory;
pub mod runner;

#[cfg(windows)]
pub mod windows_inventory;
