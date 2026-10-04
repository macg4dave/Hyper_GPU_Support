//! CORE-009 manifest inspection and bounded disposable-guest staging harness.

use std::io::{self, Write};
use std::process::ExitCode;

use hyper_gpu_support::config::ProjectConfiguration;
use hyper_gpu_support::staging::{StageStatus, inspect_driver_package, stage_driver_package};

#[cfg(windows)]
use hyper_gpu_support::guest::GuestCredential;
#[cfg(windows)]
use hyper_gpu_support::windows_guest::WindowsGuestTransfer;

fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if !valid_arguments(&arguments) {
        eprintln!("staging manifest error: {}", usage());
        return ExitCode::from(2);
    }
    match run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("staging manifest error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    match arguments {
        [] => inspect(),
        [command] if command == "inspect" => inspect(),
        [command, username] if command == "apply" => apply(username),
        _ => Err(usage().into()),
    }
}

fn valid_arguments(arguments: &[String]) -> bool {
    arguments.is_empty()
        || matches!(arguments, [command] if command == "inspect")
        || matches!(arguments, [command, username] if command == "apply" && !username.is_empty())
}

fn inspect() -> Result<(), Box<dyn std::error::Error>> {
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

#[cfg(windows)]
fn apply(username: &str) -> Result<(), Box<dyn std::error::Error>> {
    let project = ProjectConfiguration::embedded()?;
    let username = if username == "--interactive" {
        prompt_username()?
    } else {
        username.to_owned()
    };
    let password = rpassword::prompt_password_with_config(
        "Guest password: ",
        rpassword::ConfigBuilder::new()
            .password_feedback_mask('*')
            .build(),
    )?;
    let credential = GuestCredential::new(username, password)?;
    let adapter = WindowsGuestTransfer::new(project.slot.clone(), project.guest.clone());
    let receipt = stage_driver_package(
        &project.driver_manifest,
        &project.guest,
        &project.slot.vm_id,
        &credential,
        &adapter,
    )?;
    println!(
        "status={}",
        match receipt.status {
            StageStatus::Applied => "applied",
            StageStatus::AlreadyApplied => "already-applied",
        }
    );
    println!("vm_id={}", receipt.vm_id);
    println!("computer_name={}", receipt.computer_name);
    println!("machine_guid={}", receipt.machine_guid);
    println!("manifest_id={}", receipt.manifest_id);
    println!("manifest_sha256={}", receipt.manifest_sha256);
    println!("qualified_host_build={}", receipt.qualified_host_build);
    println!("measured_host_build={}", receipt.measured_host_build);
    println!("measured_guest_build={}", receipt.measured_guest_build);
    if receipt.has_host_qualification_drift() {
        println!("warning=host-build-outside-qualified-baseline");
    }
    if receipt.has_host_guest_build_drift() {
        println!("warning=host-guest-build-drift");
    }
    println!("pending_delete_count={}", receipt.pending_delete_count);
    for source in &receipt.pending_delete_sources {
        println!("pending_delete_source={source}");
    }
    if receipt.has_pending_delete_cleanup() {
        println!("warning=host-pending-delete-cleanup");
    }
    println!(
        "package_destination={}",
        receipt.package_destination.display()
    );
    println!("cuda_alias={}", receipt.cuda_alias.display());
    println!("alias_method={}", receipt.alias_method);
    println!("files={}", receipt.files);
    println!("bytes={}", receipt.bytes);
    Ok(())
}

#[cfg(not(windows))]
fn apply(_username: &str) -> Result<(), Box<dyn std::error::Error>> {
    Err("PowerShell Direct staging requires Windows".into())
}

#[cfg(windows)]
fn prompt_username() -> io::Result<String> {
    eprint!("Guest username: ");
    io::stderr().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    Ok(username.trim_end_matches(['\r', '\n']).to_owned())
}

fn usage() -> &'static str {
    "usage: hyper-gpu-stage [inspect]\n       hyper-gpu-stage apply USER\n       hyper-gpu-stage apply --interactive"
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
