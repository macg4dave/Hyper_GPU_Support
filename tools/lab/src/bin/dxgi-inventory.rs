//! Read-only DXGI adapter inventory used to diagnose probe identity failures.

use std::process::ExitCode;

use hyper_gpu_support::windows_probe::enumerate_adapter_identities;
use serde::Serialize;

#[derive(Serialize)]
struct Report {
    schema: u32,
    adapters: Vec<hyper_gpu_support::probe::AdapterIdentity>,
}

fn main() -> ExitCode {
    let adapters = match enumerate_adapter_identities() {
        Ok(adapters) => adapters,
        Err(error) => {
            eprintln!("inventory error: {error}");
            return ExitCode::from(3);
        }
    };
    match serde_json::to_string(&Report {
        schema: 1,
        adapters,
    }) {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(_) => {
            eprintln!("inventory error: cannot serialize DXGI report");
            ExitCode::from(7)
        }
    }
}
