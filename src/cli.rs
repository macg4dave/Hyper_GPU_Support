//! Small command-line boundary; arguments exclude the executable name.

use std::ffi::OsString;
use std::fmt;

/// Help for the currently implemented commands.
pub const HELP: &str = "hyper-gpu-support - Windows GPU-PV project foundation

Usage: hyper-gpu-support [COMMAND] [OPTIONS]

Commands:
  inventory        Report read-only host, GPU and Hyper-V facts

Options:
  -h, --help       Display help
  -V, --version    Display version

Mutating GPU-PV operations are not implemented yet.
";

/// Application version, taken from the package metadata.
pub const VERSION: &str = concat!("hyper-gpu-support ", env!("CARGO_PKG_VERSION"));

/// A supported informational request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Show usage and supported options.
    Help,
    /// Show the application version.
    Version,
    /// Report read-only inventory facts.
    Inventory,
}

/// Invalid arguments. User-provided content is omitted from error output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsageError;

impl fmt::Display for UsageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid arguments; use --help for usage")
    }
}

impl std::error::Error for UsageError {}

/// Parse zero or one informational option without accessing the environment.
///
/// No arguments displays help. Options cannot be combined.
///
/// # Errors
/// Returns [`UsageError`] for unknown, non-Unicode or extra arguments.
///
/// ```
/// use hyper_gpu_support::cli::{Command, parse};
/// assert_eq!(parse(["--version".into()]), Ok(Command::Version));
/// ```
pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Command, UsageError> {
    let mut arguments = arguments.into_iter();
    let Some(argument) = arguments.next() else {
        return Ok(Command::Help);
    };
    let command = match argument.to_str() {
        Some("--help" | "-h") => Command::Help,
        Some("--version" | "-V") => Command::Version,
        Some("inventory") => Command::Inventory,
        None | Some(_) => return Err(UsageError),
    };
    if arguments.next().is_some() {
        return Err(UsageError);
    }
    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::{Command, UsageError, parse};

    #[test]
    fn no_arguments_displays_help() {
        assert_eq!(parse([]), Ok(Command::Help));
    }

    #[test]
    fn accepts_supported_commands_and_aliases() {
        for (argument, expected) in [
            ("--help", Command::Help),
            ("-h", Command::Help),
            ("--version", Command::Version),
            ("-V", Command::Version),
            ("inventory", Command::Inventory),
        ] {
            assert_eq!(parse([argument.into()]), Ok(expected));
        }
    }

    #[test]
    fn rejects_unknown_empty_and_extra_arguments() {
        for arguments in [vec![""], vec!["apply"], vec!["--help", "--version"]] {
            assert_eq!(
                parse(arguments.into_iter().map(Into::into)),
                Err(UsageError)
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn rejects_unpaired_utf16_surrogate() {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        assert_eq!(parse([OsString::from_wide(&[0xd800])]), Err(UsageError));
    }
}
