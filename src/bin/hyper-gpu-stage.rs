//! Read-only CORE-009 manifest inspection harness.

use std::process::ExitCode;

use hyper_gpu_support::config::ProjectConfiguration;
use hyper_gpu_support::staging::inspect_driver_package;

fn main() -> ExitCode {
    if std::env::args_os().nth(1).is_some() {
        eprintln!("staging manifest error: usage: hyper-gpu-stage");
        return ExitCode::from(2);
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("staging manifest error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let project = ProjectConfiguration::embedded()?;
    let manifest = inspect_driver_package(&project.driver_manifest)?;
    let manifest_sha256 = manifest.sha256()?;
    verify_manifest_sha256(&project.driver_manifest.sha256, &manifest_sha256)?;
    println!("status=validated");
    println!("manifest_id={}", manifest.id);
    println!("manifest_sha256={manifest_sha256}");
    println!("package_directory={}", manifest.package_directory);
    println!("package_tree_sha256={}", manifest.package_tree_sha256);
    println!("files={}", manifest.files.len());
    println!("bytes={}", manifest.byte_count);
    Ok(())
}

fn verify_manifest_sha256(configured: &str, measured: &str) -> Result<(), &'static str> {
    if configured == measured {
        Ok(())
    } else {
        Err("encoded manifest hash does not match project configuration")
    }
}

#[cfg(test)]
mod tests {
    use super::verify_manifest_sha256;

    #[test]
    fn manifest_hash_mismatch_cannot_reach_success_output() {
        assert!(verify_manifest_sha256("expected", "expected").is_ok());
        assert_eq!(
            verify_manifest_sha256("configured", "measured"),
            Err("encoded manifest hash does not match project configuration")
        );
    }
}
