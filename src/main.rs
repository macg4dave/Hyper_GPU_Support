//! GUI/CLI dispatch over the shared runtime product core.
#![cfg_attr(windows, windows_subsystem = "windows")]
mod gui;
use std::process::ExitCode;
const HELP: &str = "Hyper GPU Support\n\nUsage: hyper-gpu-support COMMAND [--config FILE] [--vm GUID]\n\nCommands:\n  inventory       Discover existing VMs and partitionable GPUs\n  plan            Preview selected VM changes without mutation\n  install         Install/enroll the protected runner (administrator console)\n  apply           Enable/disable selected targets and refresh stale preparation\n  enable          Enable one selected target\n  disable         Detach selected GPU; keep prepared guest files\n  status          Read effective Hyper-V configuration\n  verify          Check guest device health and hardware rendering\n  credentials     Store an opt-in guest credential in Windows Credential Manager\n  forget          Delete a stored guest credential\n\nOptions:\n  --config FILE   Runtime TOML schema 2 (required except inventory/help/version)\n  --vm GUID       Select one target from the configuration\n  --help          Show this help\n  --version       Show version\n\nGUI: hyper-gpu-support (live inventory; unconnected actions unavailable)\n     hyper-gpu-support --mock-gui (no-write simulated workflow)\nVRAM values are provider-defined units, not proven hard memory limits.\n";
fn main() -> ExitCode {
    #[cfg(windows)]
    {
        let first = std::env::args().nth(1);
        if first.as_deref().is_some_and(|arg| {
            arg != "--mock-gui" && arg != "--internal-worker" && arg != "--internal-setup"
        }) {
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

#[derive(Debug, PartialEq, Eq)]
enum GuiMode {
    Live,
    Fixtures,
    Snapshot(std::path::PathBuf),
    SnapshotConfiguration(std::path::PathBuf, std::path::PathBuf),
}

fn gui_mode(args: &[String]) -> Result<Option<GuiMode>, String> {
    match args {
        [] => Ok(Some(GuiMode::Live)),
        [flag] if flag == "--mock-gui" => Ok(Some(GuiMode::Fixtures)),
        [flag, option, path] if flag == "--mock-gui" && option == "--snapshot" && !path.is_empty() => {
            Ok(Some(GuiMode::Snapshot(path.as_str().into())))
        }
        [flag, option, path, config, candidate] if flag == "--mock-gui" && option == "--snapshot" && config == "--config" && !path.is_empty() && !candidate.is_empty() => {
            Ok(Some(GuiMode::SnapshotConfiguration(path.as_str().into(), candidate.as_str().into())))
        }
        [flag, ..] if flag == "--mock-gui" => Err(
            "Use --mock-gui for fixtures, or --mock-gui --snapshot FILE [--config GUID.toml] for historical rehearsal".into(),
        ),
        _ => Ok(None),
    }
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
    if let Some(mode) = gui_mode(&args)? {
        return match mode {
            GuiMode::Live => gui::run(false),
            GuiMode::Fixtures => gui::run(true),
            GuiMode::Snapshot(path) => gui::run_snapshot(path, None),
            GuiMode::SnapshotConfiguration(path, configuration) => {
                gui::run_snapshot(path, Some(configuration))
            }
        }
        .map_err(|error| error.to_string());
    }
    if matches!(args.as_slice(),[v]if v=="--help"||v=="-h") {
        print!(
            "{HELP}\nSnapshot rehearsal: --mock-gui --snapshot FILE [--config GUID.toml]\n  Read historical inventory and an optional per-VM candidate file; no backend effects or writes.\n  Enabled plan rehearsal requires a plans array of shared CLI plan JSON in the snapshot.\n"
        );
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
                let record = runner::recovery_record()?
                    .ok_or("no recorded operation requires reconciliation")?;
                if record.operation_id != operation_id {
                    return Err("reconciliation operation identity mismatch".into());
                }
                let credential = if record.target.enabled || !record.publication_required {
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
    if ![
        "inventory",
        "plan",
        "install",
        "apply",
        "enable",
        "disable",
        "status",
        "verify",
        "credentials",
        "forget",
        "import",
    ]
    .contains(&command.as_str())
    {
        return Err("unknown command; use --help".into());
    }
    let mut config_path = None;
    let mut vm = None;
    let mut shutdown_approved = false;
    let mut iter = args[1..].iter();
    while let Some(option) = iter.next() {
        if option == "--approve-shutdown" && !shutdown_approved {
            shutdown_approved = true;
            continue;
        }
        let value = iter.next().ok_or("option value missing")?;
        match option.as_str() {
            "--config" if config_path.is_none() => config_path = Some(value),
            "--vm" if vm.is_none() => vm = Some(value),
            _ => return Err("unknown or repeated option".into()),
        }
    }
    if shutdown_approved && !["apply", "enable", "disable", "verify"].contains(&command.as_str()) {
        return Err("--approve-shutdown is valid only for an effect command".into());
    }
    if command == "inventory" {
        if config_path.is_some() || vm.is_some() {
            return Err("inventory takes no target options".into());
        }
        let result = if hyper_gpu_support::process::is_elevated()? {
            serde_json::to_value(hyper_gpu_support::windows_hyperv::discover()?)
                .map_err(|e| e.to_string())?
        } else {
            runner::submit(runner::request(Operation::Discover, None, None))?
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(config_path.ok_or("--config FILE is required")?)
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
        if vm.is_some() {
            return Err("import requires approval of the complete bundle destination set".into());
        }
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
        return Ok(());
    }
    if command == "install" {
        if vm.is_some() {
            return Err("install enrolls the complete configuration".into());
        }
        runner::install(&config)?;
        println!("Product runner installed and existing targets enrolled.");
        return Ok(());
    }
    let mut targets: Vec<_> = config
        .targets
        .into_iter()
        .filter(|t| vm.is_none_or(|id| t.vm_id.eq_ignore_ascii_case(id)))
        .collect();
    if targets.is_empty() {
        return Err("selected VM is not in the runtime configuration".into());
    }
    if ["credentials", "forget", "enable", "disable"].contains(&command.as_str())
        && targets.len() != 1
    {
        return Err("this command requires --vm selecting exactly one target".into());
    }
    for t in &mut targets {
        if command == "enable" {
            t.enabled = true;
        }
        if command == "disable" {
            t.enabled = false;
        }
        if command == "forget" {
            credentials::forget(&t.vm_id)?;
            println!("Stored credential removed.");
            continue;
        }
        if command == "credentials" {
            let credential = prompt(&t.vm_id)?;
            credentials::store(&t.vm_id, &credential)?;
            println!("Credential stored in the current user's Windows vault.");
            continue;
        }
        let status = command == "status" || command == "plan";
        let operation = if command == "plan" {
            Operation::Plan
        } else if status {
            Operation::Status
        } else if command == "verify" {
            Operation::Verify
        } else {
            Operation::Apply
        };
        let mut result = if !status {
            let snapshot = hyper_gpu_support::configuration_store::read_committed();
            let expected = snapshot.revision(&t.vm_id)?;
            if command == "verify" {
                let approved: hyper_gpu_support::workflow::VerificationPlan =
                    serde_json::from_value(runner::submit(runner::request(
                        Operation::VerifyPlan,
                        Some(t.clone()),
                        None,
                    ))?)
                    .map_err(|e| e.to_string())?;
                if approved.observed.power == hyper_gpu_support::model::Power::Off
                    && !shutdown_approved
                {
                    return Err("verification temporarily starts and stops this guest; supply --approve-shutdown after reviewing the lifecycle".into());
                }
                let credential =
                    credentials::read(&t.vm_id)?.map_or_else(|| prompt(&t.vm_id), Ok)?;
                serde_json::to_value(hyper_gpu_support::worker::verify(
                    approved,
                    shutdown_approved,
                    credential,
                    |stage| eprintln!("{:?}: {}", stage.status, stage.stage),
                )?)
                .map_err(|e| e.to_string())?
            } else {
                let approved: hyper_gpu_support::workflow::Plan = serde_json::from_value(
                    runner::submit(runner::request(Operation::Plan, Some(t.clone()), None))?,
                )
                .map_err(|e| e.to_string())?;
                if approved.preview.guest_downtime && !shutdown_approved {
                    return Err("this plan requires graceful guest shutdown; review with plan, then supply --approve-shutdown".into());
                }
                let credential = if approved.preview.credentials_required {
                    Some(credentials::read(&t.vm_id)?.map_or_else(|| prompt(&t.vm_id), Ok)?)
                } else {
                    None
                };
                serde_json::to_value(hyper_gpu_support::worker::apply(
                    approved,
                    expected,
                    shutdown_approved,
                    credential,
                    |stage| eprintln!("{:?}: {}", stage.status, stage.stage),
                )?)
                .map_err(|e| e.to_string())?
            }
        } else {
            runner::submit(runner::request(operation, Some(t.clone()), None))?
        };
        hyper_gpu_support::reporting::public_result(&mut result);
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?
        );
        if result.get("finished").is_some_and(|v| v == false) {
            return Err("verified operation awaits configuration publication; use save-only with its operation_id".into());
        }
    }
    Ok(())
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
    fn snapshot_rehearsal_requires_explicit_input_and_does_not_change_cli_dispatch() {
        assert_eq!(gui_mode(&[]).unwrap(), Some(GuiMode::Live));
        assert_eq!(
            gui_mode(&args(&["--mock-gui"])).unwrap(),
            Some(GuiMode::Fixtures)
        );
        assert_eq!(
            gui_mode(&args(&[
                "--mock-gui",
                "--snapshot",
                "recorded inventory.json"
            ]))
            .unwrap(),
            Some(GuiMode::Snapshot("recorded inventory.json".into()))
        );
        for values in [
            vec!["--mock-gui", "--snapshot"],
            vec!["--mock-gui", "--snapshot", ""],
            vec!["--mock-gui", "--config", "intent.toml"],
            vec!["--mock-gui", "--snapshot", "inventory.json", "--vm", "id"],
        ] {
            assert!(gui_mode(&args(&values)).is_err());
        }
        assert_eq!(
            gui_mode(&args(&[
                "--mock-gui",
                "--snapshot",
                "capture.json",
                "--config",
                "vm.toml"
            ]))
            .unwrap(),
            Some(GuiMode::SnapshotConfiguration(
                "capture.json".into(),
                "vm.toml".into()
            ))
        );
        for values in [
            vec!["inventory"],
            vec!["--help"],
            vec!["status", "--config", "intent.toml"],
        ] {
            assert_eq!(gui_mode(&args(&values)).unwrap(), None);
        }
    }
}
