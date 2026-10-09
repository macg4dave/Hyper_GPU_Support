//! GUI/CLI dispatch over the shared runtime product core.
mod gui;
use std::process::ExitCode;
const HELP: &str = "Hyper GPU Support\n\nUsage: hyper-gpu-support COMMAND [--config FILE] [--vm GUID]\n\nCommands:\n  inventory       Discover existing VMs and partitionable GPUs\n  plan            Preview selected VM changes without mutation\n  install         Install/enroll the protected runner (administrator console)\n  apply           Enable/disable selected targets and refresh stale preparation\n  enable          Enable one selected target\n  disable         Detach selected GPU; keep prepared guest files\n  status          Read effective Hyper-V configuration\n  verify          Check guest device health and hardware rendering\n  credentials     Store an opt-in guest credential in Windows Credential Manager\n  forget          Delete a stored guest credential\n\nOptions:\n  --config FILE   Runtime TOML schema 2 (required except inventory/help/version)\n  --vm GUID       Select one target from the configuration\n  --help          Show this help\n  --version       Show version\n\nGUI: hyper-gpu-support (no arguments; sample workspace until backend integration)\nVRAM values are provider-defined units, not proven hard memory limits.\n";
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() {
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
    ]
    .contains(&command.as_str())
    {
        return Err("unknown command; use --help".into());
    }
    let mut config_path = None;
    let mut vm = None;
    let mut iter = args[1..].iter();
    while let Some(option) = iter.next() {
        let value = iter.next().ok_or("option value missing")?;
        match option.as_str() {
            "--config" if config_path.is_none() => config_path = Some(value),
            "--vm" if vm.is_none() => vm = Some(value),
            _ => return Err("unknown or repeated option".into()),
        }
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
    let text = std::fs::read_to_string(config_path.ok_or("--config FILE is required")?)
        .map_err(|e| e.to_string())?;
    let config = Configuration::parse(&text)?;
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
        let credential = if !status && (t.enabled || command == "verify") {
            Some(credentials::read(&t.vm_id)?.map_or_else(|| prompt(&t.vm_id), Ok)?)
        } else {
            None
        };
        let result = runner::submit(runner::request(operation, Some(t.clone()), credential))?;
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?
        );
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
