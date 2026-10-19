//! Machine reset and original-user cleanup remain separate authority domains.

use crate::{ARTIFACTS, Log, cli::Action, payload, storage};
use std::{
    fs,
    path::{Path, PathBuf},
};

const TASKS: [(&str, &str); 3] = [
    (r"\", "HyperGpuSupport-Product-v2"),
    (r"\HyperGpuSupport\", "Runner-v1"),
    (r"\HyperGpuSupport\", "ResetSlot-v1"),
];
const CREDENTIAL_PREFIX: &str = "HyperGpuSupport/guest/";

#[derive(Debug)]
struct Layout {
    app: PathBuf,
    product: PathBuf,
    product_data: PathBuf,
    legacy_data: PathBuf,
    archives: PathBuf,
}

impl Layout {
    fn new(program_files: &Path, program_data: &Path) -> Self {
        Self {
            app: program_files.join("HyperGpuSupport"),
            product: program_files.join("HyperGpuSupportProduct"),
            product_data: program_data.join("HyperGpuSupportProduct"),
            legacy_data: program_data.join("HyperGpuSupport"),
            archives: program_data.join("HyperGpuSupport-archives"),
        }
    }

    #[cfg(windows)]
    fn current() -> Result<Self, String> {
        use crate::native::{Folder, folder};
        Ok(Self::new(
            &folder(Folder::ProgramFiles)?,
            &folder(Folder::ProgramData)?,
        ))
    }

    /// Application files and Inno metadata are deliberately outside maintenance ownership.
    fn removed(&self) -> [&Path; 3] {
        [&self.product, &self.product_data, &self.legacy_data]
    }

    fn archived(&self) -> [(&Path, &'static str); 2] {
        [
            (&self.product_data, "ProgramData-HyperGpuSupportProduct"),
            (&self.legacy_data, "ProgramData-HyperGpuSupport"),
        ]
    }
}

#[cfg(windows)]
pub(super) fn package(action: Action, release: Option<&Path>, log: &mut Log) -> Result<(), String> {
    if !crate::native::is_elevated()? {
        return Err("package maintenance requires the administrator setup process".into());
    }
    let layout = Layout::current()?;
    for path in layout
        .removed()
        .into_iter()
        .chain([layout.app.as_path(), layout.archives.as_path()])
    {
        storage::require_plain_path(path)?;
    }
    if let Some(directory) = release {
        payload::verify(directory)?;
        if action == Action::Verify {
            if directory
                .canonicalize()
                .map_err(|error| error.to_string())?
                != layout
                    .app
                    .canonicalize()
                    .map_err(|error| error.to_string())?
            {
                return Err(
                    "installed verification must target the fixed application directory".into(),
                );
            }
            return log.line("Installed executable hashes match the package manifest");
        }
    }
    require_closed()?;
    require_idle_tasks()?;
    let _operation_locks = operation_locks(&layout)?;
    if action == Action::Check {
        return log.line("Installation preflight passed; no state was changed");
    }
    // Repeat after acquiring locks; no running product operation may be reset.
    require_closed()?;
    require_idle_tasks()?;
    prepare_archive_root(&layout.archives)?;
    archive_state(&layout, log)?;
    for (folder, name) in TASKS {
        if task_state(folder, name)?.is_some() {
            powershell(&format!(
                "Unregister-ScheduledTask -TaskPath {} -TaskName {} -Confirm:$false -ErrorAction Stop",
                ps_quote(folder),
                ps_quote(name)
            ))?;
            log.line(&format!("Removed task {folder}{name}"))?;
        }
    }
    remove_empty_task_folder()?;
    for path in layout.removed() {
        if storage::remove_tree_if_present(path)? {
            log.line(&format!("Removed {}", path.display()))?;
        }
    }
    log.line("Protected product reset completed; application files remain owned by Inno Setup")
}

#[cfg(windows)]
pub(super) fn user(reset: bool) -> Result<(), String> {
    if crate::native::is_elevated()? {
        return Err("user preferences must be handled as the original unelevated launching user; start Setup from an ordinary user session".into());
    }
    let data = crate::native::folder(crate::native::Folder::LocalAppData)?.join("HyperGpuSupport");
    storage::require_plain_path(&data)?;
    require_closed()?;
    if reset {
        storage::remove_tree_if_present(&data)?;
        let removed = crate::native::delete_credentials(CREDENTIAL_PREFIX)?;
        println!(
            "Reset original-user preferences and {} remembered guest credentials",
            removed.len()
        );
    }
    Ok(())
}

/// Protect archived audit/recovery records before copying. A user-owned archive
/// root is rejected rather than turning it into a trusted destination in place.
#[cfg(windows)]
fn prepare_archive_root(path: &Path) -> Result<(), String> {
    storage::require_plain_path(path)?;
    let text = path.to_str().ok_or("archive path is not valid Unicode")?;
    powershell(&format!(
        "$ErrorActionPreference = 'Stop'; $path = {}; \
         $acl = New-Object System.Security.AccessControl.DirectorySecurity; \
         $acl.SetAccessRuleProtection($true, $false); \
         $admin = New-Object System.Security.Principal.SecurityIdentifier('S-1-5-32-544'); \
         $acl.SetOwner($admin); \
         foreach ($sid in @('S-1-5-18', 'S-1-5-32-544')) {{ \
           $identity = New-Object System.Security.Principal.SecurityIdentifier($sid); \
           $rule = New-Object System.Security.AccessControl.FileSystemAccessRule($identity, 'FullControl', 'ContainerInherit,ObjectInherit', 'None', 'Allow'); \
           $acl.AddAccessRule($rule) \
         }}; \
         [void][System.IO.Directory]::CreateDirectory($path, $acl); \
         $actual = Get-Acl -LiteralPath $path; \
         $owner = $actual.GetOwner([System.Security.Principal.SecurityIdentifier]).Value; \
         if ($owner -notin @('S-1-5-18', 'S-1-5-32-544') -or -not $actual.AreAccessRulesProtected) {{ throw 'Archive root has untrusted ownership or inherited permissions' }}; \
         foreach ($rule in $actual.GetAccessRules($true, $true, [System.Security.Principal.SecurityIdentifier])) {{ \
           if ($rule.IdentityReference.Value -notin @('S-1-5-18', 'S-1-5-32-544')) {{ throw 'Archive root grants access outside administrators and SYSTEM' }} \
         }}",
        ps_quote(text)
    ))?;
    storage::require_plain_path(path)
}

fn archive_state(layout: &Layout, log: &mut Log) -> Result<(), String> {
    let sources: Vec<_> = layout
        .archived()
        .into_iter()
        .map(|(path, name)| storage::exists(path).map(|present| (path, name, present)))
        .collect::<Result<_, _>>()?;
    if !sources.iter().any(|(_, _, present)| *present) {
        return log.line("No prior protected state to archive");
    }
    storage::require_plain_path(&layout.archives)?;
    fs::create_dir_all(&layout.archives)
        .map_err(|error| format!("create archive root: {error}"))?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let archive = layout
        .archives
        .join(format!("install-{stamp}-{}", std::process::id()));
    fs::create_dir(&archive).map_err(|error| format!("create unique archive: {error}"))?;
    for (path, name, present) in sources {
        if present {
            storage::copy_tree(path, &archive.join(name), log).map_err(|error| format!("archive failed ({error}); protected state was not reset; partial archive retained at {}", archive.display()))?;
        }
    }
    log.line(&format!(
        "Verified protected-state archive: {}",
        archive.display()
    ))
}

#[cfg(windows)]
fn operation_locks(layout: &Layout) -> Result<Vec<fs::File>, String> {
    use std::os::windows::fs::OpenOptionsExt;
    let mut locks = Vec::new();
    for data in [&layout.product_data, &layout.legacy_data] {
        let path = data.join("operation.lock");
        if storage::exists(&path)? {
            storage::require_plain_path(&path)?;
            // Runner opens read/write with no sharing. Holding this existing lock
            // blocks that admission, while READ|DELETE permits archival and reset.
            let lock = fs::OpenOptions::new().read(true).write(true).share_mode(5).open(&path)
                .map_err(|error| format!("another protected operation is active, or its lock cannot be checked ({}): {error}", path.display()))?;
            locks.push(lock);
        }
    }
    Ok(locks)
}

fn require_closed() -> Result<(), String> {
    let output = storage::system_command("tasklist.exe")?
        .args(["/FO", "CSV", "/NH"])
        .output()
        .map_err(|error| format!("tasklist: {error}"))?;
    if !output.status.success() {
        return Err("tasklist failed; cannot confirm the app is closed".into());
    }
    let running = running_artifacts(&String::from_utf8_lossy(&output.stdout));
    if !running.is_empty() {
        return Err(format!(
            "close Hyper GPU Support and let any operation finish first (running: {})",
            running.join(", ")
        ));
    }
    Ok(())
}

fn running_artifacts(tasklist_csv: &str) -> Vec<String> {
    let mut running: Vec<_> = tasklist_csv
        .lines()
        .filter_map(|line| line.split(',').next())
        .map(|image| image.trim().trim_matches('"').to_ascii_lowercase())
        .filter(|image| ARTIFACTS.contains(&image.as_str()))
        .collect();
    running.sort();
    running.dedup();
    running
}

fn require_idle_tasks() -> Result<(), String> {
    for (folder, name) in TASKS {
        if matches!(
            task_state(folder, name)?.as_deref(),
            Some("Running" | "Queued" | "Unknown")
        ) {
            return Err(format!(
                "scheduled task {folder}{name} has an active operation; wait for it to finish"
            ));
        }
    }
    Ok(())
}

fn task_state(folder: &str, name: &str) -> Result<Option<String>, String> {
    let state = powershell(&format!(
        "$t = Get-ScheduledTask -ErrorAction Stop | Where-Object {{ $_.TaskPath -eq {} -and $_.TaskName -eq {} }}; if ($t) {{ [string]$t.State }}",
        ps_quote(folder),
        ps_quote(name)
    ))?;
    parse_task_state(&state)
}

fn parse_task_state(state: &str) -> Result<Option<String>, String> {
    match state.trim() {
        "" => Ok(None),
        value @ ("Unknown" | "Disabled" | "Queued" | "Ready" | "Running") => Ok(Some(value.into())),
        other => Err(format!(
            "unexpected scheduler state {other}; cannot confirm product inactivity"
        )),
    }
}

fn remove_empty_task_folder() -> Result<(), String> {
    powershell("$s = New-Object -ComObject Schedule.Service; $s.Connect(); $root = $s.GetFolder('\\'); $folder = @($root.GetFolders(0) | Where-Object Name -eq 'HyperGpuSupport'); if ($folder.Count -eq 1 -and $folder[0].GetTasks(1).Count -eq 0 -and $folder[0].GetFolders(0).Count -eq 0) { $root.DeleteFolder('HyperGpuSupport', 0) }").map(drop)
}

fn powershell(script: &str) -> Result<String, String> {
    let output = storage::system_command("powershell.exe")?
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|error| format!("powershell: {error}"))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_removes_app_product_and_both_state_folders_only() {
        let dir = storage::temp("layout");
        let layout = Layout::new(&dir.join("PF"), &dir.join("PD"));
        assert_eq!(
            layout.removed(),
            [&layout.product, &layout.product_data, &layout.legacy_data]
        );
        assert!(!layout.removed().contains(&layout.app.as_path()));
        assert!(!layout.removed().contains(&layout.archives.as_path()));
        fs::create_dir_all(&layout.app).unwrap();
        fs::write(layout.app.join("unins000.exe"), "inno metadata").unwrap();
        for path in layout.removed() {
            fs::create_dir_all(path).unwrap();
            fs::write(path.join("state"), "state").unwrap();
        }
        let mut log = Log::create(&dir.join("log")).unwrap();
        archive_state(&layout, &mut log).unwrap();
        for path in layout.removed() {
            storage::remove_tree_if_present(path).unwrap();
        }
        assert_eq!(
            fs::read_to_string(layout.app.join("unins000.exe")).unwrap(),
            "inno metadata"
        );
        let archives = fs::read_dir(&layout.archives)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(archives.len(), 1);
        assert_eq!(
            fs::read_to_string(
                archives[0]
                    .path()
                    .join("ProgramData-HyperGpuSupportProduct/state")
            )
            .unwrap(),
            "state"
        );
        drop(log);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn running_artifacts_match_only_app_images() {
        let csv = "\"explorer.exe\",\"10\"\r\n\"Hyper-GPU-Support.exe\",\"11\"\r\n\"hyper-gpu-runner.exe\",\"12\"\r\n\"hyper-gpu-runner.exe\",\"13\"\r\n\"app-install.exe\",\"14\"\r\n";
        assert_eq!(
            running_artifacts(csv),
            ["hyper-gpu-runner.exe", "hyper-gpu-support.exe"]
        );
        assert!(running_artifacts("").is_empty());
    }

    #[test]
    fn powershell_quoting_escapes_single_quotes() {
        assert_eq!(ps_quote(r"\HyperGpuSupport\"), r"'\HyperGpuSupport\'");
        assert_eq!(ps_quote("a'b"), "'a''b'");
    }

    #[test]
    fn absent_task_is_none_not_a_failure() {
        assert_eq!(parse_task_state("").unwrap(), None);
        assert_eq!(
            parse_task_state("Ready\r\n").unwrap().as_deref(),
            Some("Ready")
        );
        assert!(parse_task_state("Ready\nRunning").is_err());
    }

    #[cfg(windows)]
    #[test]
    fn active_operation_lock_blocks_reset_and_idle_lock_allows_archive_cleanup() {
        use std::os::windows::fs::OpenOptionsExt;
        let dir = storage::temp("operation-lock");
        let layout = Layout::new(&dir.join("PF"), &dir.join("PD"));
        fs::create_dir_all(&layout.product_data).unwrap();
        let path = layout.product_data.join("operation.lock");
        fs::write(&path, "lock evidence").unwrap();
        let active = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .open(&path)
            .unwrap();
        assert!(operation_locks(&layout).is_err());
        assert!(layout.product_data.exists());
        drop(active);
        let locks = operation_locks(&layout).unwrap();
        assert!(
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .share_mode(0)
                .open(&path)
                .is_err()
        );
        let mut log = Log::create(&dir.join("log")).unwrap();
        archive_state(&layout, &mut log).unwrap();
        storage::remove_tree_if_present(&layout.product_data).unwrap();
        drop(locks);
        assert!(!layout.product_data.exists());
        drop(log);
        fs::remove_dir_all(dir).unwrap();
    }
}
