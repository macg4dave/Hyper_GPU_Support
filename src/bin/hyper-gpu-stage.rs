//! Complete native driver environment inspection and bounded guest staging harness.

use std::io::{self, Write};
use std::process::ExitCode;

use hyper_gpu_support::config::ProjectConfiguration;
use hyper_gpu_support::{
    driver_environment::{DriverEnvironmentManifest, inspect_driver_environment},
    environment_staging::stage_driver_environment,
    windows_driver_environment::discover_driver_environment,
    windows_environment_staging::WindowsEnvironmentStager,
    windows_paths::windows_directory,
};

#[cfg(windows)]
use hyper_gpu_support::guest::GuestCredential;

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
    let manifest = discover_manifest(&project)?;
    let manifest_sha256 = manifest.sha256()?;
    println!("status=validated");
    println!("manifest_sha256={manifest_sha256}");
    println!("device_id={}", manifest.device_id);
    println!("driver_version={}", manifest.driver_version);
    println!("files={}", manifest.files.len());
    Ok(())
}

fn discover_manifest(
    project: &ProjectConfiguration,
) -> Result<DriverEnvironmentManifest, Box<dyn std::error::Error>> {
    Ok(inspect_driver_environment(
        project,
        &windows_directory()?,
        discover_driver_environment(project)?,
    )?)
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
    eprintln!("Discovering and verifying the configured driver environment...");
    let manifest = discover_manifest(&project)?;
    eprintln!(
        "Verifying sources and staging the complete environment; this can take several minutes."
    );
    let adapter = WindowsEnvironmentStager::new(project.clone());
    let result = stage_driver_environment(
        &project,
        &windows_directory()?,
        &manifest,
        &credential,
        &adapter,
    )?;
    let receipt = result.receipt;
    println!(
        "status={}",
        if result.already_applied {
            "already-applied"
        } else {
            "applied"
        }
    );
    println!("vm_id={}", receipt.vm_id);
    println!("computer_name={}", receipt.computer_name);
    println!("machine_guid={}", receipt.machine_guid);
    println!("manifest_sha256={}", receipt.manifest_sha256);
    println!("qualified_host_build={}", receipt.qualified_host_build);
    println!("measured_host_build={}", receipt.measured_host_build);
    println!("measured_guest_build={}", receipt.measured_guest_build);
    if receipt.measured_host_build != receipt.qualified_host_build {
        println!("warning=host-build-outside-qualified-baseline");
    }
    if receipt.measured_host_build != receipt.measured_guest_build {
        println!("warning=host-guest-build-drift");
    }
    println!(
        "pending_delete_count={}",
        receipt.pending_delete_sources.len()
    );
    for source in &receipt.pending_delete_sources {
        println!("pending_delete_source={source}");
    }
    if !receipt.pending_delete_sources.is_empty() {
        println!("warning=host-pending-delete-cleanup");
    }
    println!("windows_root={}", receipt.windows_root);
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
    // Keep interactive prompts visible when a harness retains stderr diagnostics.
    let mut console = std::fs::OpenOptions::new().write(true).open("CONOUT$")?;
    console.write_all(b"Guest username: ")?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    Ok(username.trim_end_matches(['\r', '\n']).to_owned())
}

fn usage() -> &'static str {
    "usage: hyper-gpu-stage [inspect]\n       hyper-gpu-stage apply USER\n       hyper-gpu-stage apply --interactive"
}
