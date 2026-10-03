//! Operator-facing harness for one verified transfer to the pinned disposable guest.

use std::io::{self, Write};
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
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    let (username, source, destination, sha256) = match arguments.as_slice() {
        [interactive, source, destination, sha256] if interactive == "--interactive" => (
            prompt_username()?,
            PathBuf::from(source),
            PathBuf::from(destination),
            sha256
                .to_str()
                .ok_or("SHA-256 is not valid Unicode")?
                .to_owned(),
        ),
        [username, source, destination, sha256] => (
            username
                .to_str()
                .ok_or("guest username is not valid Unicode")?
                .to_owned(),
            PathBuf::from(source),
            PathBuf::from(destination),
            sha256
                .to_str()
                .ok_or("SHA-256 is not valid Unicode")?
                .to_owned(),
        ),
        _ => return Err(usage().into()),
    };

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

#[cfg(windows)]
fn prompt_username() -> io::Result<String> {
    eprint!("Guest username: ");
    io::stderr().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    Ok(username.trim_end_matches(['\r', '\n']).to_owned())
}

#[cfg(windows)]
fn usage() -> &'static str {
    "usage: hyper-gpu-guest-copy USER SOURCE DESTINATION_FILENAME SHA256\n       hyper-gpu-guest-copy --interactive SOURCE DESTINATION_FILENAME SHA256"
}

#[cfg(not(windows))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _ = PathBuf::new();
    Err("PowerShell Direct requires Windows".into())
}
