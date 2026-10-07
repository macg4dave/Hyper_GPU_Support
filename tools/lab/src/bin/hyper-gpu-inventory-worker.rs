//! Fixed native read-only inventory worker; accepts no paths, queries or targets.
use hyper_gpu_support::{
    config::ProjectConfiguration, windows_native_inventory::collect_native,
    windows_validation::WorkerDeadline,
};
use std::{io::Write, process::ExitCode};

fn main() -> ExitCode {
    if std::env::args_os().len() != 1 {
        eprintln!("inventory worker accepts no arguments");
        return ExitCode::from(2);
    }
    let result = (|| {
        let project = ProjectConfiguration::embedded().map_err(|e| e.to_string())?;
        let _watchdog = WorkerDeadline::start(project.inventory.timeout);
        let report = collect_native(&project).map_err(|e| e.to_string())?;
        std::io::stdout()
            .lock()
            .write_all(report.encode_protocol().as_bytes())
            .map_err(|e| e.to_string())
    })();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("native inventory: {e}");
            ExitCode::FAILURE
        }
    }
}
