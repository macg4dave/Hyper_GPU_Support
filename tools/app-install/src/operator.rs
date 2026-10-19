//! Local source installation always goes through the same generated setup wizard.

use std::{
    fs,
    io::{self, Write},
    path::Path,
};

pub(super) fn run(yes: bool, skip_build: bool) -> Result<(), String> {
    #[cfg(windows)]
    if crate::native::is_elevated()? {
        return Err("run the source installer from an ordinary user terminal so Setup can reset the correct user's preferences".into());
    }
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    if !skip_build
        && executable
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|name| name == "release")
    {
        return Err("the source launcher cannot rebuild its own running release executable; use cargo run -p app-install without --release, or --skip-build to package an existing build".into());
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|error| error.to_string())?;
    println!(
        "Build and run the local unsigned Inno Setup installer.\nThe installer archives and resets app-owned protected state and your preferences.\nVM, guest and data-root state are not touched."
    );
    if !yes {
        confirm()?;
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let output = std::env::temp_dir().join(format!(
        "hyper-gpu-installer-path-{}-{stamp}.txt",
        std::process::id()
    ));
    let mut build = build_command(&root, &output, skip_build)?;
    let status = build
        .status()
        .map_err(|error| format!("build installer: {error}"))?;
    if !status.success() {
        return Err("installer build failed; no installation was started".into());
    }
    let installer = fs::read_to_string(&output)
        .map_err(|error| format!("read generated installer path: {error}"))?;
    fs::remove_file(&output)
        .map_err(|error| format!("remove temporary installer path file: {error}"))?;
    let installer = Path::new(installer.trim());
    if !installer.is_absolute() || !installer.is_file() {
        return Err("build script did not return an absolute installer executable path".into());
    }
    #[cfg(windows)]
    {
        let code = crate::native::run_setup(installer)?;
        if code != 0 {
            return Err(format!(
                "Setup did not finish successfully (exit code {code})"
            ));
        }
    }
    #[cfg(not(windows))]
    return Err("Setup requires Windows".into());
    println!("Setup completed successfully.");
    Ok(())
}

fn build_command(
    root: &Path,
    output: &Path,
    skip_build: bool,
) -> Result<std::process::Command, String> {
    let mut build = crate::storage::system_command("powershell.exe")?;
    build
        .args(["-NoProfile", "-NonInteractive", "-File"])
        .arg(root.join("scripts/build-installer.ps1"))
        .args(["-Stage", "Local", "-InstallerPathFile"])
        .arg(output)
        .current_dir(root);
    if skip_build {
        build.arg("-SkipBuild");
    }
    Ok(build)
}

fn confirm() -> Result<(), String> {
    use std::io::IsTerminal;
    if !io::stdin().is_terminal() {
        return Err("confirmation needs a terminal; pass --yes to proceed".into());
    }
    print!("Type yes to continue: ");
    io::stdout().flush().map_err(|error| error.to_string())?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|error| error.to_string())?;
    if answer.trim() != "yes" {
        return Err("cancelled; nothing was changed".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    #[test]
    fn relaunch_command_line_survives_spaces_and_trailing_backslashes() {
        use std::{ffi::OsString, path::Path};
        let root = Path::new(r"C:\Source tree\HyperGpuSupport\");
        let output = Path::new(r"C:\User temp\installer path.txt");
        let command = super::build_command(root, output, true).unwrap();
        let expected: Vec<OsString> = ["-NoProfile", "-NonInteractive", "-File"]
            .into_iter()
            .map(OsString::from)
            .chain([root.join("scripts/build-installer.ps1").into_os_string()])
            .chain(
                ["-Stage", "Local", "-InstallerPathFile"]
                    .into_iter()
                    .map(OsString::from),
            )
            .chain([output.as_os_str().to_owned(), OsString::from("-SkipBuild")])
            .collect();
        assert_eq!(command.get_args().collect::<Vec<_>>(), expected);
        assert_eq!(command.get_current_dir(), Some(root));
        assert!(Path::new(command.get_program()).is_absolute());
    }
}
