//! Source installer launcher and fixed-purpose helpers for the Inno Setup package.
//! Package helpers never own the application's files or Inno uninstall metadata.

mod cli;
mod maintenance;
#[cfg(windows)]
mod native;
mod operator;
mod payload;
mod storage;

use std::{fs, io::Write, path::Path};

/// Product processes that must be closed before an installation changes state.
const ARTIFACTS: [&str; 4] = [
    "hyper-gpu-support.exe",
    "hyper-gpu-runner.exe",
    "hyper-gpu-guest.exe",
    "d3d11-probe.exe",
];

/// Product executables, including this package's maintenance helper.
const PAYLOAD: [&str; 5] = [
    ARTIFACTS[0],
    ARTIFACTS[1],
    ARTIFACTS[2],
    ARTIFACTS[3],
    "app-install.exe",
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = cli::parse_args(&args).and_then(run);
    if let Err(error) = result {
        eprintln!("app-install: {error}");
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn run(_: cli::Mode) -> Result<(), String> {
    Err("app-install only runs on Windows".into())
}

#[cfg(windows)]
fn run(mode: cli::Mode) -> Result<(), String> {
    match mode {
        cli::Mode::Operator { yes, skip_build } => operator::run(yes, skip_build),
        cli::Mode::User { reset, log } => {
            if let Some(path) = log {
                let mut log = Log::create(&path)?;
                report(maintenance::user(reset), &mut log)
            } else {
                maintenance::user(reset)
            }
        }
        cli::Mode::Package {
            action,
            release,
            log,
        } => {
            let mut log = Log::create(&log)?;
            let result = maintenance::package(action, release.as_deref(), &mut log);
            report(result, &mut log)
        }
    }
}

#[cfg(windows)]
fn report(result: Result<(), String>, log: &mut Log) -> Result<(), String> {
    if let Err(error) = &result {
        log.line(&format!("FAILED: {error}"))?;
    }
    result
}

/// Persist every diagnostic before reporting success to the setup wizard.
struct Log(fs::File);

impl Log {
    fn create(path: &Path) -> Result<Self, String> {
        fs::File::create(path)
            .map(Self)
            .map_err(|error| format!("log {}: {error}", path.display()))
    }

    fn line(&mut self, text: &str) -> Result<(), String> {
        println!("{text}");
        writeln!(self.0, "{text}").map_err(|error| format!("write installer log: {error}"))?;
        self.0
            .flush()
            .map_err(|error| format!("flush installer log: {error}"))
    }
}
