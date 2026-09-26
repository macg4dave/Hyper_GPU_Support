//! Administrator-installed fixed runner for the enrolled disposable Hyper-V slot.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use hyper_gpu_support::runner::{parse_reset_result, verify_policy};

const INSTALL_ROOT: &str = r"C:\ProgramData\HyperGpuSupport\Runner";
const POLICY_PATH: &str = r"C:\ProgramData\HyperGpuSupport\Runner\policy-v1.json";
const TIMEOUT: Duration = Duration::from_secs(300);
const OUTPUT_LIMIT: usize = 64 * 1024;

fn main() -> ExitCode {
    match run() {
        Ok(operation_id) => {
            println!("reset-slot completed: {operation_id}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("runner error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, Box<dyn std::error::Error>> {
    let mut arguments = std::env::args_os().skip(1);
    if arguments.next().as_deref() != Some("reset-slot".as_ref()) || arguments.next().is_some() {
        return Err("only the fixed reset-slot operation is accepted".into());
    }
    let installed_policy = fs::read_to_string(POLICY_PATH)?;
    verify_policy(&installed_policy)?;

    let state_directory = Path::new(INSTALL_ROOT).join("state");
    let result_directory = Path::new(INSTALL_ROOT).join("results");
    let audit_directory = Path::new(INSTALL_ROOT).join("audit");
    fs::create_dir_all(&state_directory)?;
    fs::create_dir_all(&result_directory)?;
    fs::create_dir_all(&audit_directory)?;
    let lock_path = state_directory.join("reset-slot.lock");
    let _lock = LockFile::acquire(&lock_path)?;

    let operation_id = operation_id()?;
    append_audit(&audit_directory, &operation_id, "started", "")?;
    let output = match run_bounded() {
        Ok(output) => output,
        Err(error) => {
            append_audit(&audit_directory, &operation_id, "failed", &error)?;
            return Err(error.into());
        }
    };
    let result = parse_reset_result(&output)?;
    let result_json = format!(
        concat!(
            "{{\n  \"schema\": 1,\n  \"operation_id\": \"{}\",\n",
            "  \"operation\": \"reset-slot\",\n  \"status\": \"succeeded\",\n",
            "  \"vm_id\": \"{}\",\n  \"child\": \"{}\",\n",
            "  \"parent\": \"{}\",\n  \"parent_sha256\": \"{}\"\n}}\n"
        ),
        operation_id,
        result.vm_id,
        json_escape(&result.child),
        json_escape(&result.parent),
        result.parent_sha256
    );
    write_atomic(&result_directory, &operation_id, &result_json)?;
    append_audit(&audit_directory, &operation_id, "succeeded", "")?;
    Ok(operation_id)
}

struct LockFile {
    path: PathBuf,
    _file: File,
}

impl LockFile {
    fn acquire(path: &Path) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        Ok(Self {
            path: path.to_owned(),
            _file: file,
        })
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn operation_id() -> Result<String, std::time::SystemTimeError> {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(format!(
        "{}-{:09}",
        duration.as_secs(),
        duration.subsec_nanos()
    ))
}

fn append_audit(
    directory: &Path,
    operation_id: &str,
    status: &str,
    diagnostic: &str,
) -> Result<(), std::io::Error> {
    let path = directory.join("events.jsonl");
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(
        file,
        "{{\"operation_id\":\"{}\",\"operation\":\"reset-slot\",\"status\":\"{}\",\"diagnostic\":\"{}\"}}",
        operation_id,
        status,
        json_escape(diagnostic)
    )?;
    file.flush()
}

fn write_atomic(directory: &Path, operation_id: &str, contents: &str) -> std::io::Result<()> {
    let temporary = directory.join(format!("{operation_id}.tmp"));
    let final_path = directory.join(format!("{operation_id}.json"));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()?;
    drop(file);
    fs::rename(temporary, final_path)
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

fn run_bounded() -> Result<String, String> {
    let mut child = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            RESET_SCRIPT,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot launch fixed Hyper-V adapter: {error}"))?;
    let stdout = child.stdout.take().ok_or("stdout unavailable")?;
    let stderr = child.stderr.take().ok_or("stderr unavailable")?;
    let stdout_reader = thread::spawn(move || read_bounded(stdout));
    let stderr_reader = thread::spawn(move || read_bounded(stderr));
    let deadline = Instant::now() + TIMEOUT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(50)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("fixed Hyper-V adapter timed out".into());
            }
            Err(error) => return Err(format!("cannot wait for fixed adapter: {error}")),
        }
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| "stdout reader failed")?
        .map_err(|error| error.to_string())?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| "stderr reader failed")?
        .map_err(|error| error.to_string())?;
    if !status.success() {
        return Err(format!(
            "fixed adapter failed with {:?}: {}",
            status.code(),
            String::from_utf8_lossy(&stderr)
                .chars()
                .filter(|character| !character.is_control() || *character == ' ')
                .take(512)
                .collect::<String>()
        ));
    }
    String::from_utf8(stdout).map_err(|_| "fixed adapter output was not UTF-8".into())
}

fn read_bounded(mut reader: impl Read) -> std::io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            return Ok(output);
        }
        if output.len() + count > OUTPUT_LIMIT {
            return Err(std::io::Error::other("adapter output exceeded limit"));
        }
        output.extend_from_slice(&buffer[..count]);
    }
}

const RESET_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$vmId = [guid]'2627E735-5B33-4104-B739-622727DD3A40'
$vmName = 'HyperGpuSupport-Disposable-01'
$parentPath = 'Z:\HyperGpuSupport\images\golden\win11-pro-25h2-26200.9457-x64-v1\parent.vhdx'
$parentHash = '0fb4dfe6dd51eed64d36e482e4f58d67c19922802e34aa6ae2d1f4ccd5daeb07'
$childPath = 'Z:\HyperGpuSupport\images\disposable\gpu-pv-slot-01\child.vhdx'
$principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'administrator token required' }
$vm = Get-VM -Id $vmId
if ($vm.Name -ne $vmName -or $vm.State -ne 'Off' -or $vm.Generation -ne 2 -or [string]$vm.Version -ne '12.0') { throw 'enrolled VM identity or state mismatch' }
if ($vm.AutomaticCheckpointsEnabled -or @(Get-VMSnapshot -VM $vm -ErrorAction SilentlyContinue).Count -ne 0) { throw 'checkpoint state rejected' }
if (@(Get-VMGpuPartitionAdapter -VM $vm -ErrorAction SilentlyContinue).Count -ne 0) { throw 'GPU adapter must be removed before reset' }
$parent = Get-Item -LiteralPath $parentPath
if (-not $parent.IsReadOnly -or ($parent.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'parent protection mismatch' }
$hash = (Get-FileHash -LiteralPath $parentPath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($hash -ne $parentHash) { throw 'parent hash mismatch' }
$parentVhd = Get-VHD -Path $parentPath
if ($parentVhd.ParentPath -or $parentVhd.VhdType -ne 'Dynamic' -or $parentVhd.Attached) { throw 'parent VHD state mismatch' }
$drives = @(Get-VMHardDiskDrive -VM $vm)
if ($drives.Count -gt 1 -or ($drives.Count -eq 1 -and $drives[0].Path -ine $childPath)) { throw 'enrolled child attachment mismatch' }
if (Test-Path -LiteralPath $childPath) {
    $childItem = Get-Item -LiteralPath $childPath
    if ($childItem.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'child reparse point rejected' }
    $child = Get-VHD -Path $childPath
    if ($child.ParentPath -ine $parentPath -or $child.VhdType -ne 'Differencing') { throw 'child chain mismatch' }
}
if ($drives.Count -eq 1) { Remove-VMHardDiskDrive -VMHardDiskDrive $drives[0] }
if (Test-Path -LiteralPath $childPath) { Remove-Item -LiteralPath $childPath }
New-VHD -Path $childPath -ParentPath $parentPath -Differencing | Out-Null
Add-VMHardDiskDrive -VM $vm -ControllerType SCSI -ControllerNumber 0 -ControllerLocation 0 -Path $childPath
$verifiedDrive = Get-VMHardDiskDrive -VM $vm
$verifiedChild = Get-VHD -Path $verifiedDrive.Path
if ($verifiedDrive.Path -ine $childPath -or $verifiedChild.ParentPath -ine $parentPath -or -not (Get-Item -LiteralPath $parentPath).IsReadOnly) { throw 'reset verification failed' }
[Console]::Out.WriteLine('status' + "`t" + 'ok')
[Console]::Out.WriteLine('vm_id' + "`t" + $vmId.ToString().ToLowerInvariant())
[Console]::Out.WriteLine('child' + "`t" + $verifiedChild.Path)
[Console]::Out.WriteLine('parent' + "`t" + $verifiedChild.ParentPath)
[Console]::Out.WriteLine('parent_sha256' + "`t" + $hash)
"#;

#[cfg(test)]
mod tests {
    use super::{RESET_SCRIPT, json_escape};
    use hyper_gpu_support::runner::POLICY_V1;

    #[test]
    fn fixed_script_has_no_external_input_or_broad_vm_deletion() {
        assert!(POLICY_V1.contains("reset-slot"));
        assert!(!RESET_SCRIPT.contains("Remove-VM "));
        assert!(!RESET_SCRIPT.contains("Invoke-Expression"));
        assert!(!RESET_SCRIPT.contains("param("));
        assert!(RESET_SCRIPT.contains("Remove-VMHardDiskDrive"));
        assert!(RESET_SCRIPT.contains("Get-FileHash"));
    }

    #[test]
    fn audit_json_escapes_control_characters() {
        assert_eq!(
            json_escape("one\\two\r\n\"three"),
            "one\\\\two\\r\\n\\\"three"
        );
    }
}
