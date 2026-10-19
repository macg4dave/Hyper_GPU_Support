//! Clean reinstall of the Hyper GPU Support build.
//!
//! Run as the operator: `cargo run -p app-install [-- --yes] [--skip-build] [--no-archive]`.
//! The tool builds the release binaries, confirms the app is closed, then relaunches
//! itself through Windows UAC to archive and remove the previous installation and
//! its app-owned state before copying the new build into Program Files.
//! VM, guest and `Z:` data-root state are never touched.

#[cfg(windows)]
mod native;

use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

/// The four artifacts first-run setup pins and installs.
const ARTIFACTS: [&str; 4] = [
    "hyper-gpu-support.exe",
    "hyper-gpu-runner.exe",
    "hyper-gpu-guest.exe",
    "d3d11-probe.exe",
];

/// Current product task and the legacy runner tasks, as (folder, name).
const TASKS: [(&str, &str); 3] = [
    (r"\", "HyperGpuSupport-Product-v2"),
    (r"\HyperGpuSupport\", "Runner-v1"),
    (r"\HyperGpuSupport\", "ResetSlot-v1"),
];

/// Legacy scheduler folder, removed once its tasks are gone.
const LEGACY_TASK_FOLDER: &str = "HyperGpuSupport";

/// Vault prefix for remembered guest credentials (see `src/credentials.rs`).
const CREDENTIAL_PREFIX: &str = "HyperGpuSupport/guest/";

#[derive(Debug, PartialEq)]
enum Mode {
    Operator {
        yes: bool,
        skip_build: bool,
        no_archive: bool,
    },
    Elevated {
        release: PathBuf,
        log: PathBuf,
        archive: Option<PathBuf>,
    },
}

fn parse_args(args: &[String]) -> Result<Mode, String> {
    if args.first().map(String::as_str) == Some("--elevated") {
        let (mut release, mut log, mut archive) = (None, None, None);
        let mut rest = args[1..].iter();
        while let Some(flag) = rest.next() {
            let slot = match flag.as_str() {
                "--release-dir" => &mut release,
                "--log" => &mut log,
                "--archive" => &mut archive,
                other => return Err(format!("unknown elevated option {other}")),
            };
            let value = rest.next().ok_or(format!("{flag} needs a path"))?;
            if slot.replace(PathBuf::from(value)).is_some() {
                return Err(format!("{flag} given twice"));
            }
        }
        return Ok(Mode::Elevated {
            release: release.ok_or("--release-dir is required")?,
            log: log.ok_or("--log is required")?,
            archive,
        });
    }
    let (mut yes, mut skip_build, mut no_archive) = (false, false, false);
    for flag in args {
        let slot = match flag.as_str() {
            "--yes" => &mut yes,
            "--skip-build" => &mut skip_build,
            "--no-archive" => &mut no_archive,
            "-h" | "--help" => return Err(usage()),
            other => return Err(format!("unknown option {other}\n\n{}", usage())),
        };
        *slot = true;
    }
    Ok(Mode::Operator {
        yes,
        skip_build,
        no_archive,
    })
}

fn usage() -> String {
    "Usage: cargo run -p app-install -- [--yes] [--skip-build] [--no-archive]\n\
     \n  --yes         skip the typed confirmation (the UAC prompt still appears)\
     \n  --skip-build  install the existing release build without rebuilding\
     \n  --no-archive  do not copy the removed protected state to the archive folder"
        .into()
}

/// Machine locations owned by the app.
#[derive(Debug)]
struct Layout {
    /// Installed GUI package; first-run setup installs the product from here.
    app: PathBuf,
    /// Protected product installation registered with the scheduler.
    product: PathBuf,
    /// Protected product state: enrollment, configuration, audit, recovery.
    product_data: PathBuf,
    /// Legacy runner installation and state.
    legacy_data: PathBuf,
}

impl Layout {
    fn new(program_files: &Path, program_data: &Path) -> Self {
        Self {
            app: program_files.join("HyperGpuSupport"),
            product: program_files.join("HyperGpuSupportProduct"),
            product_data: program_data.join("HyperGpuSupportProduct"),
            legacy_data: program_data.join("HyperGpuSupport"),
        }
    }

    fn from_env() -> Result<Self, String> {
        let get = |name| {
            std::env::var_os(name)
                .map(PathBuf::from)
                .ok_or(format!("{name} is not set"))
        };
        Ok(Self::new(&get("ProgramFiles")?, &get("ProgramData")?))
    }

    fn removed(&self) -> [&Path; 4] {
        [
            &self.app,
            &self.product,
            &self.product_data,
            &self.legacy_data,
        ]
    }

    /// State that may hold audit or recovery evidence, as (path, archive name).
    fn archived(&self) -> [(&Path, &'static str); 2] {
        [
            (&self.product_data, "ProgramData-HyperGpuSupportProduct"),
            (&self.legacy_data, "ProgramData-HyperGpuSupport"),
        ]
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = parse_args(&args).and_then(run);
    if let Err(error) = result {
        eprintln!("app-install: {error}");
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn run(_: Mode) -> Result<(), String> {
    Err("app-install only runs on Windows".into())
}

#[cfg(windows)]
fn run(mode: Mode) -> Result<(), String> {
    match mode {
        Mode::Operator {
            yes,
            skip_build,
            no_archive,
        } => operator(yes, skip_build, no_archive),
        Mode::Elevated {
            release,
            log,
            archive,
        } => {
            let mut log = Log::create(&log)?;
            let result = elevated(&release, archive.as_deref(), &mut log);
            if let Err(error) = &result {
                log.line(&format!("FAILED: {error}"));
            }
            result
        }
    }
}

#[cfg(windows)]
fn operator(yes: bool, skip_build: bool, no_archive: bool) -> Result<(), String> {
    let root = workspace_root()?;
    if !skip_build {
        build(&root)?;
    }
    let release = target_dir(&root)
        .join("x86_64-pc-windows-msvc")
        .join("release");
    artifact_hashes(&release)?;
    require_closed()?;

    let layout = Layout::from_env()?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let archive = if no_archive {
        None
    } else {
        let profile = std::env::var_os("USERPROFILE").ok_or("USERPROFILE is not set")?;
        Some(
            PathBuf::from(profile)
                .join("HyperGpuSupport-archives")
                .join(format!("install-{stamp}")),
        )
    };
    let user_data = user_data_dir()?;

    println!("Clean reinstall of Hyper GPU Support");
    println!("  new build:   {}", release.display());
    println!("  remove scheduled tasks:");
    for (folder, name) in TASKS {
        println!("    {folder}{name}");
    }
    println!("  remove folders (elevated):");
    for path in layout.removed() {
        println!("    {}", path.display());
    }
    println!("  remove your preferences: {}", user_data.display());
    println!("  remove remembered guest credentials: {CREDENTIAL_PREFIX}*");
    match &archive {
        Some(path) => println!("  archive protected state first to: {}", path.display()),
        None => println!("  protected state is NOT archived (--no-archive)"),
    }
    println!("  install to:  {}", layout.app.display());
    println!("  VM, guest and data-root state are not touched.");
    if !yes {
        confirm()?;
    }

    let log = std::env::temp_dir().join(format!("hyper-gpu-app-install-{stamp}.log"));
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut args = vec![
        "--elevated".to_owned(),
        "--release-dir".into(),
        path_arg(&release)?,
        "--log".into(),
        path_arg(&log)?,
    ];
    if let Some(archive) = &archive {
        args.extend(["--archive".into(), path_arg(archive)?]);
    }
    println!("Requesting administrator approval (UAC)...");
    let code = native::run_elevated(&exe, &args);
    if let Ok(text) = fs::read_to_string(&log) {
        print!("{text}");
        let _ = fs::remove_file(&log);
    }
    match code? {
        0 => {}
        code => return Err(format!("elevated install failed (exit code {code})")),
    }

    remove_tree_if_present(&user_data)?;
    for target in native::delete_credentials(CREDENTIAL_PREFIX)? {
        println!("Removed credential {target}");
    }
    println!(
        "Installed. Launch {} to run first-time setup.",
        layout.app.join(ARTIFACTS[0]).display()
    );
    Ok(())
}

#[cfg(windows)]
fn elevated(release: &Path, archive: Option<&Path>, log: &mut Log) -> Result<(), String> {
    if !native::is_elevated()? {
        return Err("the elevated phase must run as administrator".into());
    }
    let layout = Layout::from_env()?;
    let hashes = artifact_hashes(release)?;
    require_closed()?;
    for (folder, name) in TASKS {
        if task_state(folder, name)?.as_deref() == Some("Running") {
            return Err(format!(
                "scheduled task {folder}{name} is running an operation; wait for it to finish"
            ));
        }
    }

    if let Some(archive) = archive {
        for (path, name) in layout.archived() {
            if path.exists() {
                log.line(&format!("Archiving {}", path.display()));
                copy_tree(path, &archive.join(name), log).map_err(|e| {
                    format!(
                        "archive of {} failed ({e}); nothing was removed. \
                         Re-run with --no-archive to reset without the archive",
                        path.display()
                    )
                })?;
            }
        }
        log.line(&format!("Archive: {}", archive.display()));
    }

    for (folder, name) in TASKS {
        if task_state(folder, name)?.is_some() {
            unregister_task(folder, name)?;
            log.line(&format!("Removed task {folder}{name}"));
        }
    }
    remove_task_folder(LEGACY_TASK_FOLDER);
    for path in layout.removed() {
        if remove_tree_if_present(path)? {
            log.line(&format!("Removed {}", path.display()));
        }
    }

    fs::create_dir_all(&layout.app).map_err(|e| e.to_string())?;
    for (name, hash) in &hashes {
        let to = layout.app.join(name);
        fs::copy(release.join(name), &to).map_err(|e| format!("copy {name}: {e}"))?;
        if &hash_file(&to)? != hash {
            return Err(format!("installed {name} does not match the build"));
        }
        log.line(&format!("Installed {name} sha256={hash}"));
    }
    Ok(())
}

/// Elevated output goes to a file the operator console prints afterwards.
#[cfg(windows)]
struct Log(fs::File);

#[cfg(windows)]
impl Log {
    fn create(path: &Path) -> Result<Self, String> {
        fs::File::create(path)
            .map(Self)
            .map_err(|e| format!("log {}: {e}", path.display()))
    }

    fn line(&mut self, text: &str) {
        println!("{text}");
        let _ = writeln!(self.0, "{text}");
    }
}

#[cfg(not(windows))]
struct Log;

#[cfg(not(windows))]
impl Log {
    fn line(&mut self, _: &str) {}
}

fn workspace_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|e| e.to_string())
}

fn target_dir(root: &Path) -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map(|dir| root.join(dir))
        .unwrap_or_else(|| root.join("target"))
}

fn build(root: &Path) -> Result<(), String> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    println!("Building release binaries...");
    let status = Command::new(cargo)
        .args([
            "build",
            "--locked",
            "--release",
            "--bins",
            "-p",
            "hyper-gpu-support",
        ])
        .current_dir(root)
        .status()
        .map_err(|e| format!("cargo build: {e}"))?;
    if !status.success() {
        return Err("release build failed; nothing was changed".into());
    }
    Ok(())
}

fn user_data_dir() -> Result<PathBuf, String> {
    std::env::var_os("LOCALAPPDATA")
        .map(|dir| PathBuf::from(dir).join("HyperGpuSupport"))
        .ok_or("LOCALAPPDATA is not set".into())
}

fn confirm() -> Result<(), String> {
    use std::io::IsTerminal;
    if !io::stdin().is_terminal() {
        return Err("confirmation needs a terminal; pass --yes to proceed".into());
    }
    print!("Type yes to continue: ");
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|e| e.to_string())?;
    if answer.trim() != "yes" {
        return Err("cancelled; nothing was changed".into());
    }
    Ok(())
}

/// Quote arguments for the Windows command-line parser used by the relaunch.
fn native_command_line(args: &[String]) -> String {
    args.iter()
        .map(|arg| {
            let mut quoted = String::from('"');
            let mut slashes = 0;
            for c in arg.chars() {
                match c {
                    '\\' => slashes += 1,
                    '"' => {
                        quoted.extend(std::iter::repeat_n('\\', slashes * 2 + 1));
                        slashes = 0;
                    }
                    _ => slashes = 0,
                }
                quoted.push(c);
            }
            quoted.extend(std::iter::repeat_n('\\', slashes));
            quoted.push('"');
            quoted
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn path_arg(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or(format!("{} is not valid Unicode", path.display()))
}

/// Hash every artifact, failing before any change if one is missing.
fn artifact_hashes(release: &Path) -> Result<Vec<(&'static str, String)>, String> {
    ARTIFACTS
        .iter()
        .map(|name| {
            let path = release.join(name);
            if !path.is_file() {
                return Err(format!(
                    "{} is missing; build first or drop --skip-build",
                    path.display()
                ));
            }
            Ok((*name, hash_file(&path)?))
        })
        .collect()
}

fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 1 << 16];
    loop {
        let read = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// Refuse while any app process could be mid-operation.
fn require_closed() -> Result<(), String> {
    let output = Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .output()
        .map_err(|e| format!("tasklist: {e}"))?;
    if !output.status.success() {
        return Err("tasklist failed; cannot confirm the app is closed".into());
    }
    let running = running_artifacts(&String::from_utf8_lossy(&output.stdout));
    if running.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "close Hyper GPU Support and let any operation finish first (running: {})",
            running.join(", ")
        ))
    }
}

fn running_artifacts(tasklist_csv: &str) -> Vec<String> {
    let mut running: Vec<String> = tasklist_csv
        .lines()
        .filter_map(|line| line.split(',').next())
        .map(|image| image.trim().trim_matches('"').to_ascii_lowercase())
        .filter(|image| ARTIFACTS.contains(&image.as_str()))
        .collect();
    running.sort();
    running.dedup();
    running
}

/// Scheduler state names are enum values, so they are not localised.
/// Filtering the full listing keeps an absent task a success while scheduler
/// failures still exit non-zero.
fn task_state(folder: &str, name: &str) -> Result<Option<String>, String> {
    let state = powershell(&format!(
        "$t = Get-ScheduledTask -ErrorAction Stop | \
         Where-Object {{ $_.TaskPath -eq {} -and $_.TaskName -eq {} }}; \
         if ($t) {{ [string]$t.State }}",
        ps_quote(folder),
        ps_quote(name)
    ))?;
    Ok((!state.is_empty()).then_some(state))
}

fn unregister_task(folder: &str, name: &str) -> Result<(), String> {
    powershell(&format!(
        "Unregister-ScheduledTask -TaskPath {} -TaskName {} -Confirm:$false -ErrorAction Stop",
        ps_quote(folder),
        ps_quote(name)
    ))
    .map(drop)
}

/// Best effort: the folder only disappears when empty.
fn remove_task_folder(name: &str) {
    let _ = powershell(&format!(
        "$s = New-Object -ComObject Schedule.Service; $s.Connect(); \
         try {{ $s.GetFolder('\\').DeleteFolder({}, 0) }} catch {{}}",
        ps_quote(name)
    ));
}

fn powershell(script: &str) -> Result<String, String> {
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| format!("powershell: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "powershell failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn ps_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "''"))
}

/// Copy a tree without following reparse points; a failure aborts the reset.
fn copy_tree(from: &Path, to: &Path, log: &mut Log) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let kind = fs::symlink_metadata(entry.path())?.file_type();
        let target = to.join(entry.file_name());
        if kind.is_symlink() {
            log.line(&format!("  skipped link {}", entry.path().display()));
        } else if kind.is_dir() {
            copy_tree(&entry.path(), &target, log)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Remove a tree, taking ownership of protected ACLs if plain removal is denied.
fn remove_tree_if_present(path: &Path) -> Result<bool, String> {
    if fs::symlink_metadata(path).is_err() {
        return Ok(false);
    }
    if fs::remove_dir_all(path).is_ok() {
        return Ok(true);
    }
    let text = path_arg(path)?;
    // Removal is already committed here; restore administrator access and retry.
    let _ = Command::new("takeown")
        .args(["/F", &text, "/R", "/A", "/D", "Y"])
        .output();
    let _ = Command::new("icacls")
        .args([&text, "/grant", "*S-1-5-32-544:F", "/T", "/C", "/Q"])
        .output();
    clear_readonly(path);
    fs::remove_dir_all(path).map_err(|e| format!("remove {}: {e}", path.display()))?;
    Ok(true)
}

fn clear_readonly(path: &Path) {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        let mut permissions = metadata.permissions();
        if permissions.readonly() {
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            let _ = fs::set_permissions(path, permissions);
        }
        if metadata.is_dir()
            && let Ok(entries) = fs::read_dir(path)
        {
            for entry in entries.flatten() {
                clear_readonly(&entry.path());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).into()).collect()
    }

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("app-install-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn operator_flags_parse_and_unknown_flags_fail() {
        assert_eq!(
            parse_args(&args(&[])).unwrap(),
            Mode::Operator {
                yes: false,
                skip_build: false,
                no_archive: false
            }
        );
        assert_eq!(
            parse_args(&args(&["--yes", "--skip-build", "--no-archive"])).unwrap(),
            Mode::Operator {
                yes: true,
                skip_build: true,
                no_archive: true
            }
        );
        assert!(parse_args(&args(&["--force"])).is_err());
        assert!(parse_args(&args(&["--help"])).is_err());
    }

    #[test]
    fn elevated_mode_requires_paths_and_rejects_duplicates() {
        assert_eq!(
            parse_args(&args(&[
                "--elevated",
                "--release-dir",
                r"C:\b",
                "--log",
                r"C:\l.log"
            ]))
            .unwrap(),
            Mode::Elevated {
                release: r"C:\b".into(),
                log: r"C:\l.log".into(),
                archive: None
            }
        );
        for bad in [
            vec!["--elevated"],
            vec!["--elevated", "--release-dir", "a"],
            vec!["--elevated", "--release-dir", "a", "--log"],
            vec![
                "--elevated",
                "--release-dir",
                "a",
                "--release-dir",
                "b",
                "--log",
                "l",
            ],
            vec!["--elevated", "--release-dir", "a", "--log", "l", "--yes"],
        ] {
            assert!(parse_args(&args(&bad)).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn layout_removes_app_product_and_both_state_folders_only() {
        let layout = Layout::new(Path::new(r"C:\PF"), Path::new(r"C:\PD"));
        assert_eq!(
            layout.removed(),
            [
                Path::new(r"C:\PF\HyperGpuSupport"),
                Path::new(r"C:\PF\HyperGpuSupportProduct"),
                Path::new(r"C:\PD\HyperGpuSupportProduct"),
                Path::new(r"C:\PD\HyperGpuSupport"),
            ]
        );
        let archived: Vec<_> = layout.archived().iter().map(|(path, _)| *path).collect();
        assert_eq!(archived, [&layout.product_data, &layout.legacy_data]);
    }

    #[test]
    fn running_artifacts_match_only_app_images() {
        let csv = "\"explorer.exe\",\"10\",\"Console\",\"1\",\"9 K\"\r\n\
                   \"Hyper-GPU-Support.exe\",\"11\",\"Console\",\"1\",\"9 K\"\r\n\
                   \"hyper-gpu-runner.exe\",\"12\",\"Services\",\"0\",\"9 K\"\r\n\
                   \"hyper-gpu-runner.exe\",\"13\",\"Services\",\"0\",\"9 K\"\r\n\
                   \"app-install.exe\",\"14\",\"Console\",\"1\",\"9 K\"\r\n";
        assert_eq!(
            running_artifacts(csv),
            ["hyper-gpu-runner.exe", "hyper-gpu-support.exe"]
        );
        assert!(running_artifacts("").is_empty());
    }

    #[test]
    fn relaunch_command_line_survives_spaces_and_trailing_backslashes() {
        assert_eq!(
            native_command_line(&args(&["--release-dir", r"C:\Program Files\x\", r#"a"b"#])),
            r#""--release-dir" "C:\Program Files\x\\" "a\"b""#
        );
    }

    #[test]
    fn powershell_quoting_escapes_single_quotes() {
        assert_eq!(ps_quote(r"\HyperGpuSupport\"), r"'\HyperGpuSupport\'");
        assert_eq!(ps_quote("a'b"), "'a''b'");
    }

    #[cfg(windows)]
    #[test]
    fn absent_task_is_none_not_a_failure() {
        assert_eq!(
            task_state(r"\HyperGpuSupport\", "app-install-test-absent-task").unwrap(),
            None
        );
    }

    #[test]
    fn missing_artifact_fails_before_any_change() {
        let dir = temp("missing");
        for name in &ARTIFACTS[..3] {
            fs::write(dir.join(name), name).unwrap();
        }
        let error = artifact_hashes(&dir).unwrap_err();
        assert!(error.contains("d3d11-probe.exe"), "{error}");
        fs::write(dir.join(ARTIFACTS[3]), "probe").unwrap();
        let hashes = artifact_hashes(&dir).unwrap();
        assert_eq!(hashes.len(), 4);
        assert_eq!(hashes[3].1, hash_file(&dir.join(ARTIFACTS[3])).unwrap());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn hash_is_lowercase_sha256() {
        let dir = temp("hash");
        fs::write(dir.join("f"), "abc").unwrap();
        assert_eq!(
            hash_file(&dir.join("f")).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn archive_copies_nested_state_and_removal_handles_readonly() {
        let dir = temp("archive");
        let source = dir.join("state");
        fs::create_dir_all(source.join("audit")).unwrap();
        fs::write(source.join("enrollment.json"), "{}").unwrap();
        fs::write(source.join("audit/events.jsonl"), "e").unwrap();
        let mut permissions = fs::metadata(source.join("enrollment.json"))
            .unwrap()
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(source.join("enrollment.json"), permissions).unwrap();

        #[cfg(windows)]
        let mut log = Log::create(&dir.join("log")).unwrap();
        #[cfg(not(windows))]
        let mut log = Log;
        copy_tree(&source, &dir.join("copy"), &mut log).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("copy/audit/events.jsonl")).unwrap(),
            "e"
        );

        assert!(remove_tree_if_present(&source).unwrap());
        assert!(!source.exists());
        assert!(!remove_tree_if_present(&source).unwrap());
        drop(log);
        clear_readonly(&dir);
        fs::remove_dir_all(dir).unwrap();
    }
}
