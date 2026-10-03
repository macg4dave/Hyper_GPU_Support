//! Read-only CORE-009 manifest inspection harness.

use std::process::ExitCode;

use hyper_gpu_support::config::ProjectConfiguration;
use hyper_gpu_support::staging::inspect_driver_package;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("staging manifest error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args_os().nth(1).is_some() {
        return Err("usage: hyper-gpu-stage".into());
    }
    let project = ProjectConfiguration::embedded()?;
    let manifest = inspect_driver_package(&project.driver_manifest)?;
    let manifest_sha256 = manifest.sha256()?;
    println!("status=validated");
    println!("manifest_id={}", manifest.id);
    println!("manifest_sha256={manifest_sha256}");
    println!("package_directory={}", manifest.package_directory);
    println!("package_tree_sha256={}", manifest.package_tree_sha256);
    println!("files={}", manifest.files.len());
    println!("bytes={}", manifest.byte_count);
    if project.driver_manifest.sha256 != manifest_sha256 {
        return Err("encoded manifest hash does not match project configuration".into());
    }
    Ok(())
}
