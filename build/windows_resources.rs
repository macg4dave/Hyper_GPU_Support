//! Windows version resources shared by the product and installer helper builds.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

/// Embed a consistent product identity into each executable, including helpers.
pub fn compile(binaries: &[&str], icon_path: &Path) -> Result<(), String> {
    println!("cargo:rerun-if-env-changed=RC");
    println!("cargo:rerun-if-changed={}", icon_path.display());
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return Ok(());
    }
    let manifest =
        env::var_os("CARGO_MANIFEST_DIR").ok_or("Cargo CARGO_MANIFEST_DIR is missing")?;
    let icon_path = PathBuf::from(manifest).join(icon_path);
    let icon = icon_path
        .to_str()
        .ok_or("Windows icon path must be valid UTF-8")?
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let version = env::var("CARGO_PKG_VERSION").map_err(|error| error.to_string())?;
    let numbers = version
        .split('.')
        .map(str::parse::<u16>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!("Windows releases need a numeric major.minor.patch version: {error}")
        })?;
    let [major, minor, patch] = numbers.as_slice() else {
        return Err("Windows releases need a numeric major.minor.patch version".into());
    };
    let target = env::var("TARGET").map_err(|error| error.to_string())?;
    let compiler = match env::var_os("RC") {
        Some(path) => PathBuf::from(path),
        None => match find_msvc_tools::find_windows_sdk(&target)
            .and_then(|sdk| sdk.path().map(|path| path.join("rc.exe")).find(|path| path.is_file()))
        {
            Some(compiler) => compiler,
            None => sdk_compiler_from_program_files()?.ok_or("Windows SDK resource compiler rc.exe was not found; install the Windows SDK or set RC")?,
        },
    };
    let output = PathBuf::from(env::var_os("OUT_DIR").ok_or("Cargo OUT_DIR is missing")?);
    for binary in binaries {
        let rc = output.join(format!("{binary}-version.rc"));
        let resource = output.join(format!("{binary}-version.res"));
        let contents = format!(
            r#"LANGUAGE 9, 2
1 ICON "{icon}"
1 VERSIONINFO
FILEVERSION {major},{minor},{patch},0
PRODUCTVERSION {major},{minor},{patch},0
FILEFLAGSMASK 0x3fL
FILEFLAGS 0
FILEOS 0x40004L
FILETYPE 0x1L
FILESUBTYPE 0
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "080904b0"
    BEGIN
      VALUE "CompanyName", "macg4dave"
      VALUE "FileDescription", "Hyper GPU Support - {binary}"
      VALUE "FileVersion", "{version}.0"
      VALUE "InternalName", "{binary}"
      VALUE "LegalCopyright", "Copyright (c) 2026 macg4dave"
      VALUE "OriginalFilename", "{binary}.exe"
      VALUE "ProductName", "Hyper GPU Support"
      VALUE "ProductVersion", "{version}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x0809, 1200
  END
END
"#
        );
        fs::write(&rc, contents).map_err(|error| format!("write {}: {error}", rc.display()))?;
        let status = Command::new(&compiler)
            .arg("/nologo")
            .arg("/fo")
            .arg(&resource)
            .arg(&rc)
            .status()
            .map_err(|error| format!("run {}: {error}", compiler.display()))?;
        if !status.success() {
            return Err(format!(
                "Windows version resource compilation failed for {binary}: {status}"
            ));
        }
        println!("cargo:rustc-link-arg-bin={binary}={}", resource.display());
    }
    Ok(())
}

// Some portable C++ toolchains have the SDK files without its registry entries.
// Prefer the configured SDK above, then discover installed SDK versions on disk.
fn sdk_compiler_from_program_files() -> Result<Option<PathBuf>, String> {
    let Some(program_files) = env::var_os("ProgramFiles(x86)") else {
        return Ok(None);
    };
    let root = PathBuf::from(program_files).join("Windows Kits/10/bin");
    if !root.exists() {
        return Ok(None);
    }
    let host = match env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        "x86" => "x86",
        _ => return Ok(None),
    };
    let mut candidates = Vec::new();
    for entry in root
        .read_dir()
        .map_err(|error| format!("read SDK directory {}: {error}", root.display()))?
    {
        let entry = entry.map_err(|error| format!("read SDK version entry: {error}"))?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let version = name
            .split('.')
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>();
        if let Ok(version) = version {
            let compiler = entry.path().join(host).join("rc.exe");
            if compiler.is_file() {
                candidates.push((version, compiler));
            }
        }
    }
    candidates.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(candidates.pop().map(|(_, path)| path))
}
