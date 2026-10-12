//! Fixed Rust guest preparation and verification worker.
fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        eprintln!("guest worker requires a fixed operation, protected bundle and hardware IDs");
        return std::process::ExitCode::from(2);
    }
    #[cfg(windows)]
    {
        let result = (|| {
            let vendor = args[2].parse().map_err(|_| "invalid vendor ID")?;
            let device = args[3].parse().map_err(|_| "invalid device ID")?;
            hyper_gpu_support::guest::worker_with_progress(
                &args[0],
                std::path::Path::new(&args[1]),
                vendor,
                device,
                &mut |phase| {
                    use hyper_gpu_support::workflow::PreparationProgress;
                    use std::io::Write;
                    let code = match phase {
                        PreparationProgress::CheckingCopiedFiles => "checking-copied",
                        PreparationProgress::InstallingFiles => "installing",
                        PreparationProgress::CheckingInstalledFiles => "checking-installed",
                        PreparationProgress::CheckingGuestFiles => "checking-guest",
                        PreparationProgress::CheckingGraphics => "graphics",
                        _ => return Err("unexpected guest worker phase".into()),
                    };
                    let mut stdout = std::io::stdout().lock();
                    writeln!(stdout, "HGPREPARATION {code}")
                        .and_then(|_| stdout.flush())
                        .map_err(|_| "guest progress delivery failed".into())
                },
            )
        })();
        match result {
            Ok(text) => {
                println!("{text}");
                std::process::ExitCode::SUCCESS
            }
            Err(e) => {
                // The fixed bridge forwards stdout and the exit code; never
                // transport raw error text that could contain credential input.
                println!(
                    "{}",
                    hyper_gpu_support::reporting::ComponentFailure::guest_error(&e)
                );
                std::process::ExitCode::FAILURE
            }
        }
    }
    #[cfg(not(windows))]
    {
        std::process::ExitCode::FAILURE
    }
}
