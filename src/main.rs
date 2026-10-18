//! GUI startup, fixed internal modes and retained administrator record/recovery commands.
#![cfg_attr(windows, windows_subsystem = "windows")]
mod gui;
use std::process::ExitCode;
const HELP: &str = "Hyper GPU Support\n\nLaunch without arguments to open the GUI. Use the GUI for inventory, GPU settings, verification and credentials.\n\nAdministrator record/recovery commands retained for existing data:\n  install --config FILE              Install/update the protected runner or recover legacy installation\n  import --config FILE               Import intent into absent per-VM configuration files\n  save-only --operation ID           Retry saving a recorded verified outcome\n  reconcile --operation ID [--approve-shutdown]\n                                    Check a recorded outcome without replaying Apply\n  import-reconcile --operation ID    Resume the recorded import destination set\n  native-review --job REFERENCE      Review exact local terminal Hyper-V job evidence\n  native-reconcile --review TOKEN    Approve retirement of reviewed provider admission\n\n  --help                            Show this help\n  --version                         Show version\n\nConfiguration files use runtime TOML schema 2 or 3. Import never enrolls pairs or applies GPU state.\n";
fn main() -> ExitCode {
    #[cfg(windows)]
    {
        let first = std::env::args().nth(1);
        if first
            .as_deref()
            .is_some_and(|arg| arg != "--internal-worker" && arg != "--internal-setup")
        {
            hyper_gpu_support::console::attach_parent();
        }
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!(
                "error: {}",
                hyper_gpu_support::reporting::operator_error("Command", &e)
            );
            ExitCode::FAILURE
        }
    }
}

/// Only an argument-free launch enters the live GUI. Explicit commands and
/// unknown flags cannot trigger GUI startup/elevation.
fn starts_gui(args: &[String]) -> bool {
    args.is_empty()
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    #[cfg(windows)]
    if args.first().is_some_and(|arg| arg == "--internal-setup") {
        return match args.as_slice() {
            [_, session, frontend] => hyper_gpu_support::setup::serve(
                session,
                frontend.parse().map_err(|_| "invalid frontend identity")?,
            ),
            _ => Err("invalid fixed setup mode".into()),
        };
    }
    #[cfg(windows)]
    if args.first().is_some_and(|arg| arg == "--internal-worker") {
        return match args.as_slice() {
            [_, session, frontend] => hyper_gpu_support::worker::serve(
                session,
                frontend.parse().map_err(|_| "invalid frontend identity")?,
            ),
            _ => Err("invalid restricted worker mode".into()),
        };
    }
    if starts_gui(&args) {
        return gui::run().map_err(|error| error.to_string());
    }
    if matches!(args.as_slice(),[v]if v=="--help"||v=="-h") {
        print!("{HELP}");
        return Ok(());
    }
    if matches!(args.as_slice(),[v]if v=="--version"||v=="-V") {
        println!("hyper-gpu-support {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    #[cfg(windows)]
    {
        execute(&args)
    }
    #[cfg(not(windows))]
    {
        Err("Hyper-V GPU-PV requires Windows x64".into())
    }
}

#[cfg(windows)]
fn execute(args: &[String]) -> Result<(), String> {
    use hyper_gpu_support::{
        credentials,
        model::Configuration,
        runner::{self, Operation},
    };
    let command = args.first().ok_or("command required")?;
    if command == "native-review" {
        let [_, flag, job] = args else {
            return Err("Use native-review --job LOCAL_JOB_REFERENCE".into());
        };
        if flag != "--job" {
            return Err("Use native-review --job LOCAL_JOB_REFERENCE".into());
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&runner::review_native(job)?).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if command == "native-reconcile" {
        let [_, flag, token] = args else {
            return Err("Use native-reconcile --review SHA256_TOKEN".into());
        };
        if flag != "--review"
            || token.len() != 64
            || !token
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err("Use native-reconcile --review SHA256_TOKEN".into());
        }
        runner::reconcile_native(&hyper_gpu_support::configuration_store::Revision::Content(
            token.clone(),
        ))?;
        println!(
            "Provider admission retired; review the separate workflow recovery before further changes."
        );
        return Ok(());
    }
    if ["save-only", "reconcile", "import-reconcile"].contains(&command.as_str()) {
        let (operation_id, shutdown) = match args {
            [_, option, id] if option == "--operation" => (id.clone(), false),
            [cmd, option, id, consent]
                if cmd == "reconcile"
                    && option == "--operation"
                    && consent == "--approve-shutdown" =>
            {
                (id.clone(), true)
            }
            _ => {
                return Err("use COMMAND --operation ID [--approve-shutdown for reconcile]".into());
            }
        };
        let progress = |stage: hyper_gpu_support::worker::Progress| {
            eprintln!("{:?}: {}", stage.status, stage.stage)
        };
        let result = match command.as_str() {
            "save-only" => hyper_gpu_support::worker::save_only(operation_id, progress)?,
            "import-reconcile" => hyper_gpu_support::worker::resume_import(operation_id, progress)?,
            _ => {
                let record = runner::recovery_for_operation(&operation_id)?
                    .ok_or("no recorded operation requires reconciliation")?;
                if record.operation_id != operation_id {
                    return Err("reconciliation operation identity mismatch".into());
                }
                // A proven negative completion ends this attempt without guest
                // work. These reads only select whether to prompt; the worker
                // independently validates the proof again under its lock.
                let negative_candidate = if record.requires_graphics() {
                    let inventory: hyper_gpu_support::gui_model::Inventory =
                        serde_json::from_value(runner::submit(runner::request(
                            Operation::Discover,
                            None,
                            None,
                        ))?)
                        .map_err(|e| format!("Invalid recovery inventory: {e}"))?;
                    let revision = hyper_gpu_support::configuration_store::read_committed()
                        .revision(&record.target.vm_id)?;
                    inventory
                        .discovery
                        .vms
                        .iter()
                        .find(|vm| vm.vm_id == record.target.vm_id)
                        .zip(
                            inventory
                                .discovery
                                .gpus
                                .iter()
                                .find(|gpu| gpu.interface == record.target.gpu_interface),
                        )
                        .is_some_and(|(vm, gpu)| {
                            record
                                .negative_completion(
                                    vm,
                                    &gpu.driver_version,
                                    &revision,
                                    inventory
                                        .managed
                                        .get(&record.target.vm_id)
                                        .and_then(Option::as_ref),
                                )
                                .is_ok_and(|candidate| candidate.is_some())
                        })
                } else {
                    false
                };
                let credential = if record.requires_graphics() && !negative_candidate {
                    Some(
                        credentials::read(&record.target.vm_id)?
                            .map_or_else(|| prompt(&record.target.vm_id), Ok)?,
                    )
                } else {
                    None
                };
                hyper_gpu_support::worker::reconcile(operation_id, shutdown, credential, progress)?
            }
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?
        );
        return if result.finished {
            Ok(())
        } else {
            Err("publication incomplete; retain the receipt and retry saving only".into())
        };
    }
    let config_path = record_config_path(args)?;
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(config_path)
        .map_err(|e| e.to_string())?
        .take(runner::FRAME_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > runner::FRAME_LIMIT {
        return Err("configuration input exceeds the 1 MiB limit".into());
    }
    let text = String::from_utf8(bytes).map_err(|_| "configuration must be UTF-8 TOML")?;
    let config = Configuration::parse(&text)?;
    if command == "import" {
        let snapshot = hyper_gpu_support::configuration_store::read_committed();
        let mut expected = std::collections::BTreeMap::new();
        for (filename, source) in config.vm_documents()? {
            let target = Configuration::parse_vm_file(&source, &filename)?
                .targets
                .remove(0);
            let revision = snapshot.revision(&target.vm_id)?;
            if revision != hyper_gpu_support::configuration_store::Revision::Missing {
                return Err(format!(
                    "import destination {filename} already exists; resolve the conflict explicitly"
                ));
            }
            eprintln!("Import destination: {filename}");
            expected.insert(filename, revision);
        }
        let result = hyper_gpu_support::worker::import(text, expected, |stage| {
            eprintln!("{:?}: {}", stage.status, stage.stage)
        })?;
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?
        );
    } else {
        runner::install(&config)?;
        println!("Product runner installed and existing targets enrolled.");
    }
    Ok(())
}

/// Validate the retained record commands before opening an input or contacting a runner.
#[cfg(any(windows, test))]
fn record_config_path(args: &[String]) -> Result<&str, String> {
    match args {
        [command, option, path]
            if matches!(command.as_str(), "install" | "import") && option == "--config" =>
        {
            Ok(path)
        }
        _ => Err(
            "Use the GUI for ordinary operations; see --help for record/recovery commands.".into(),
        ),
    }
}
#[cfg(windows)]
fn prompt(vm: &str) -> Result<hyper_gpu_support::credentials::Credential, String> {
    use std::io::{IsTerminal, Write};
    if !std::io::stdin().is_terminal() {
        return hyper_gpu_support::credentials::prompt(
            vm,
            windows::Win32::Foundation::HWND::default(),
        );
    }
    eprint!("Guest username: ");
    std::io::stderr().flush().map_err(|e| e.to_string())?;
    let mut username = String::new();
    std::io::stdin()
        .read_line(&mut username)
        .map_err(|e| e.to_string())?;
    let password = rpassword::prompt_password("Guest password: ")
        .map_err(|_| "credential prompt unavailable")?;
    let username = username.trim().to_owned();
    if username.is_empty() || username.contains('\0') {
        return Err("invalid guest username".into());
    }
    Ok(hyper_gpu_support::credentials::Credential { username, password })
}

#[cfg(test)]
mod dispatch_tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).into()).collect()
    }

    #[test]
    fn record_commands_accept_only_the_complete_bundle_without_target_or_effect_options() {
        for command in ["install", "import"] {
            assert_eq!(
                record_config_path(&args(&[command, "--config", "intent.toml"])).unwrap(),
                "intent.toml"
            );
            for values in [
                vec![command],
                vec![command, "--config"],
                vec![command, "--vm", "vm-id"],
                vec![command, "--config", "intent.toml", "--vm", "vm-id"],
                vec![command, "--config", "intent.toml", "--approve-shutdown"],
                vec![command, "--config", "a.toml", "--config", "b.toml"],
            ] {
                assert!(record_config_path(&args(&values)).is_err(), "{values:?}");
            }
        }
    }

    #[test]
    fn retired_ordinary_commands_cannot_reach_configuration_or_installation() {
        for command in [
            "inventory",
            "plan",
            "status",
            "apply",
            "enable",
            "disable",
            "verify",
            "credentials",
            "forget",
            "unknown",
        ] {
            for values in [
                vec![command],
                vec![command, "--config", "intent.toml"],
                vec![command, "--config", "intent.toml", "--approve-shutdown"],
            ] {
                let args = args(&values);
                assert!(!starts_gui(&args));
                assert!(record_config_path(&args).is_err(), "{values:?}");
            }
        }
    }

    #[test]
    fn only_argument_free_startup_selects_live_gui() {
        assert!(starts_gui(&[]));
        for values in [
            vec!["inventory"],
            vec!["--help"],
            vec!["--version"],
            vec!["status", "--config", "intent.toml"],
            vec!["--internal-worker", "session", "123"],
            vec!["--internal-setup", "session", "123"],
            vec!["--mock-gui"],
            vec!["--mock-gui", "--snapshot", "capture.json"],
            vec!["--snapshot", "capture.json"],
        ] {
            assert!(
                !starts_gui(&args(&values)),
                "unexpected GUI dispatch: {values:?}"
            );
        }
        assert!(!HELP.contains("--mock-gui"));
        assert!(!HELP.contains("--snapshot"));
    }
}
