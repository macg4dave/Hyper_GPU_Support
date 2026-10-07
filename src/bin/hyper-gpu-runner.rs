//! Installed one-shot product privilege boundary.
fn main() -> std::process::ExitCode {
    if std::env::args_os().len() != 1 {
        eprintln!("runner accepts no command or target arguments");
        return std::process::ExitCode::from(2);
    }
    #[cfg(windows)]
    match hyper_gpu_support::runner::serve() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("runner: {e}");
            std::process::ExitCode::FAILURE
        }
    }
    #[cfg(not(windows))]
    {
        std::process::ExitCode::FAILURE
    }
}
