//! Operator-facing harness for one verified transfer to the pinned disposable guest.

use std::path::PathBuf;
use std::process::ExitCode;

#[cfg(windows)]
use hyper_gpu_support::config::ProjectConfiguration;
#[cfg(windows)]
use hyper_gpu_support::guest::{GuestCredential, TransferRequest, transfer_verified};
#[cfg(windows)]
use hyper_gpu_support::windows_guest::WindowsGuestTransfer;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("guest copy error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(windows)]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let username = arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or("usage: hyper-gpu-guest-copy USER SOURCE RELATIVE_DESTINATION SHA256")?;
    let source = arguments
        .next()
        .map(PathBuf::from)
        .ok_or("usage: hyper-gpu-guest-copy USER SOURCE RELATIVE_DESTINATION SHA256")?;
    let destination = arguments
        .next()
        .map(PathBuf::from)
        .ok_or("usage: hyper-gpu-guest-copy USER SOURCE RELATIVE_DESTINATION SHA256")?;
    let sha256 = arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or("usage: hyper-gpu-guest-copy USER SOURCE RELATIVE_DESTINATION SHA256")?;
    if arguments.next().is_some() {
        return Err("usage: hyper-gpu-guest-copy USER SOURCE RELATIVE_DESTINATION SHA256".into());
    }

    let source = source.canonicalize()?;
    let request = TransferRequest::new(source, destination, sha256)?;
    let password = rpassword::prompt_password("Guest password: ")?;
    let credential = GuestCredential::new(username, password)?;
    let project = ProjectConfiguration::embedded()?;
    let adapter = WindowsGuestTransfer::new(project.slot.clone(), project.guest.clone());
    let receipt = transfer_verified(
        &project.guest,
        &project.slot.vm_id,
        &credential,
        &request,
        &adapter,
    )?;
    println!("status=transferred");
    println!("vm_id={}", receipt.vm_id);
    println!("computer_name={}", receipt.computer_name);
    println!("machine_guid={}", receipt.machine_guid);
    println!("destination={}", receipt.destination.display());
    println!("sha256={}", receipt.sha256);
    println!("bytes={}", receipt.bytes);
    Ok(())
}

#[cfg(not(windows))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _ = PathBuf::new();
    Err("PowerShell Direct requires Windows".into())
}
