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
            hyper_gpu_support::guest::worker(
                &args[0],
                std::path::Path::new(&args[1]),
                vendor,
                device,
            )
        })();
        match result {
            Ok(text) => {
                println!("{text}");
                std::process::ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("guest worker: {e}");
                std::process::ExitCode::FAILURE
            }
        }
    }
    #[cfg(not(windows))]
    {
        std::process::ExitCode::FAILURE
    }
}
