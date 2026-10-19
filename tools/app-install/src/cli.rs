//! Strict separation of the source launcher, original-user and privileged helpers.

use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Action {
    Check,
    Reset,
    Verify,
    Uninstall,
}

#[derive(Debug, PartialEq)]
pub(super) enum Mode {
    Operator {
        yes: bool,
        skip_build: bool,
    },
    User {
        reset: bool,
        log: Option<PathBuf>,
    },
    Package {
        action: Action,
        release: Option<PathBuf>,
        log: PathBuf,
    },
}

pub(super) fn parse_args(args: &[String]) -> Result<Mode, String> {
    match args.first().map(String::as_str) {
        Some("--package-check-user" | "--package-reset-user") => {
            let log = match &args[1..] {
                [] => None,
                [flag, value] if flag == "--log" => Some(absolute_path(flag, value)?),
                _ => return Err("original-user helpers accept only an optional --log path".into()),
            };
            Ok(Mode::User {
                reset: args[0] == "--package-reset-user",
                log,
            })
        }
        Some("--package-check") => package_args(Action::Check, &args[1..]),
        Some("--package-reset") => package_args(Action::Reset, &args[1..]),
        Some("--package-verify") => package_args(Action::Verify, &args[1..]),
        Some("--package-uninstall") => package_args(Action::Uninstall, &args[1..]),
        _ => operator_args(args),
    }
}

fn absolute_path(flag: &str, value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(format!("{flag} needs an absolute path"));
    }
    Ok(path)
}

fn package_args(action: Action, args: &[String]) -> Result<Mode, String> {
    let (mut release, mut log) = (None, None);
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        let slot = match flag.as_str() {
            "--release-dir" if action != Action::Uninstall => &mut release,
            "--log" => &mut log,
            _ => return Err(format!("unknown package option {flag}")),
        };
        let value = rest
            .next()
            .ok_or_else(|| format!("{flag} needs an absolute path"))?;
        if slot.replace(absolute_path(flag, value)?).is_some() {
            return Err(format!("{flag} given twice"));
        }
    }
    if action != Action::Uninstall && release.is_none() {
        return Err("--release-dir is required".into());
    }
    Ok(Mode::Package {
        action,
        release,
        log: log.ok_or("--log is required")?,
    })
}

fn operator_args(args: &[String]) -> Result<Mode, String> {
    let (mut yes, mut skip_build) = (false, false);
    for flag in args {
        let slot = match flag.as_str() {
            "--yes" => &mut yes,
            "--skip-build" => &mut skip_build,
            "--no-archive" => return Err("--no-archive is no longer supported: installation must archive protected evidence before resetting it".into()),
            "-h" | "--help" => return Err(usage()),
            other => return Err(format!("unknown option {other}\n\n{}", usage())),
        };
        if *slot {
            return Err(format!("{flag} given twice"));
        }
        *slot = true;
    }
    Ok(Mode::Operator { yes, skip_build })
}

fn usage() -> String {
    "Usage: cargo run -p app-install -- [--yes] [--skip-build]\n\n  --yes         confirm the clean installation (Windows still requests UAC approval)\n  --skip-build  package existing release binaries without rebuilding\n\nRequires Inno Setup 6 and invokes the same unsigned installer built by scripts/build-installer.ps1.".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).into()).collect()
    }

    #[test]
    fn operator_flags_parse_and_unknown_flags_fail() {
        assert_eq!(
            parse_args(&args(&[])).unwrap(),
            Mode::Operator {
                yes: false,
                skip_build: false
            }
        );
        assert_eq!(
            parse_args(&args(&["--yes", "--skip-build"])).unwrap(),
            Mode::Operator {
                yes: true,
                skip_build: true
            }
        );
        for bad in [
            ["--yes", "--yes"],
            ["--skip-build", "--skip-build"],
            ["--force", "--help"],
            ["--no-archive", "--yes"],
        ] {
            assert!(parse_args(&args(&bad)).is_err());
        }
    }

    #[test]
    fn elevated_mode_requires_paths_and_rejects_duplicates() {
        let root = std::env::temp_dir();
        let release = root.join("payload").to_string_lossy().into_owned();
        let log = root.join("setup.log").to_string_lossy().into_owned();
        for action in ["--package-check", "--package-reset", "--package-verify"] {
            assert!(parse_args(&args(&[action, "--release-dir", &release, "--log", &log])).is_ok());
            for bad in [
                vec![action],
                vec![action, "--release-dir", &release],
                vec![action, "--release-dir", &release, "--log"],
                vec![
                    action,
                    "--release-dir",
                    &release,
                    "--release-dir",
                    &release,
                    "--log",
                    &log,
                ],
                vec![action, "--release-dir", &release, "--log", &log, "--yes"],
                vec![action, "--release-dir", "relative", "--log", &log],
            ] {
                assert!(parse_args(&args(&bad)).is_err(), "{bad:?}");
            }
        }
        assert!(parse_args(&args(&["--package-uninstall", "--log", &log])).is_ok());
        assert!(
            parse_args(&args(&[
                "--package-uninstall",
                "--release-dir",
                &release,
                "--log",
                &log
            ]))
            .is_err()
        );
        for action in ["--package-check-user", "--package-reset-user"] {
            assert!(parse_args(&args(&[action])).is_ok());
            assert!(parse_args(&args(&[action, "--log", &log])).is_ok());
            assert!(parse_args(&args(&[action, "--yes"])).is_err());
        }
    }
}
