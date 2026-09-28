//! Administrator-only exact-SID account-right setup helper.

use std::process::ExitCode;

use hyper_gpu_support::windows_account_rights::update_runner_rights;

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let operation = arguments.next();
    let sid = arguments.next();
    if arguments.next().is_some() || sid.is_none() {
        eprintln!("rights helper requires exactly: grant|revoke <SID>");
        return ExitCode::FAILURE;
    }
    let grant = match operation.as_deref() {
        Some("grant") => true,
        Some("revoke") => false,
        _ => {
            eprintln!("rights helper accepts only grant or revoke");
            return ExitCode::FAILURE;
        }
    };
    match update_runner_rights(sid.as_deref().expect("checked above"), grant) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rights helper failed: {error}");
            ExitCode::FAILURE
        }
    }
}
