//! Windows GPU-PV foundation executable.

use std::io::{self, Write};
use std::process::ExitCode;

use hyper_gpu_support::cli::{self, Command};
use hyper_gpu_support::config::ErrorCategory;
#[cfg(windows)]
use hyper_gpu_support::inventory::InventorySource;

fn main() -> ExitCode {
    let command = match cli::parse(std::env::args_os().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            return report_error(&mut io::stderr().lock(), &error, 2);
        }
    };
    let output = match command {
        Command::Help => cli::HELP.to_owned(),
        Command::Version => cli::VERSION.to_owned(),
        Command::Inventory => match inventory_output() {
            Ok(output) => output,
            Err(error) => return report_error(&mut io::stderr().lock(), &error, 1),
        },
        Command::Declared(operation) => {
            return report_error(
                &mut io::stderr().lock(),
                &format_args!("{} is declared but not implemented", operation.as_str()),
                ErrorCategory::Implementation.exit_code(),
            );
        }
    };
    match write_output(&mut io::stdout().lock(), &output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => report_error(
            &mut io::stderr().lock(),
            &format_args!("cannot write output: {error}"),
            1,
        ),
    }
}

#[cfg(windows)]
fn inventory_output() -> Result<String, hyper_gpu_support::inventory::InventoryError> {
    hyper_gpu_support::windows_inventory::WindowsInventory
        .collect()
        .map(|report| report.render())
}

#[cfg(not(windows))]
fn inventory_output() -> Result<String, hyper_gpu_support::inventory::InventoryError> {
    Err(
        hyper_gpu_support::inventory::InventoryError::AdapterLaunch {
            kind: std::io::ErrorKind::Unsupported,
            code: None,
        },
    )
}

fn write_output(writer: &mut impl Write, output: &str) -> io::Result<()> {
    writeln!(writer, "{output}")?;
    writer.flush()
}

fn report_error(writer: &mut impl Write, error: &dyn std::fmt::Display, code: u8) -> ExitCode {
    // If stderr is unavailable, the failure exit code is the remaining signal.
    if writeln!(writer, "error: {error}")
        .and_then(|()| writer.flush())
        .is_err()
    {
        return ExitCode::FAILURE;
    }
    ExitCode::from(code)
}

#[cfg(test)]
mod tests {
    use super::{report_error, write_output};
    use std::io::{self, Write};
    use std::process::ExitCode;

    struct FailedWriter {
        fail_flush: bool,
    }

    impl Write for FailedWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.fail_flush {
                Ok(bytes.len())
            } else {
                Err(io::ErrorKind::BrokenPipe.into())
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
    }

    #[test]
    fn output_propagates_write_and_flush_failures() {
        for fail_flush in [false, true] {
            let error = write_output(&mut FailedWriter { fail_flush }, "version").unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
        }
    }

    #[test]
    fn error_reporting_preserves_code_or_signals_unavailable_stderr() {
        let mut output = Vec::new();
        assert_eq!(
            report_error(&mut output, &"invalid input", 2),
            ExitCode::from(2)
        );
        assert_eq!(output, b"error: invalid input\n");
        for fail_flush in [false, true] {
            assert_eq!(
                report_error(&mut FailedWriter { fail_flush }, &"invalid input", 2),
                ExitCode::FAILURE
            );
        }
    }
}
