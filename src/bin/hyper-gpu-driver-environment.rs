//! Read-only native discovery of the complete Easy-GPU-PV copy manifest.
use hyper_gpu_support::{
    config::ProjectConfiguration, driver_environment::inspect_driver_environment,
    windows_driver_environment::discover_driver_environment, windows_paths::windows_directory,
};
use std::process::ExitCode;

fn main() -> ExitCode {
    if std::env::args_os().len() != 1 {
        eprintln!("driver environment inspector accepts no target, path or command arguments");
        return ExitCode::from(2);
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("driver environment inspection failed: {error}");
            ExitCode::FAILURE
        }
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let project = ProjectConfiguration::embedded()?;
    let discovery = discover_driver_environment(&project)?;
    let manifest = inspect_driver_environment(&project, &windows_directory()?, discovery)?;
    use std::io::Write;
    std::io::stdout().lock().write_all(&manifest.encode()?)?;
    Ok(())
}
