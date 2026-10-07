//! Native Windows controls over the same GPU-PV core as the CLI.
#![cfg_attr(not(test), windows_subsystem = "windows")]
#[cfg(windows)]
#[path = "../windows_gui.rs"]
mod gui;
fn main() {
    #[cfg(windows)]
    if let Err(error) = gui::run() {
        gui::error(&error);
    }
}
