//! Read activity extends an inactivity watchdog, never the overall execution budget.

use std::os::windows::io::AsRawHandle;
use std::process::Child;
use std::time::Duration;

use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::{GetProcessIoCounters, IO_COUNTERS};

#[allow(unsafe_code)] // Narrow process-observation FFI; no caller-selected handle.
pub(super) fn read_bytes(child: &Child) -> Result<u64, String> {
    let mut counters = IO_COUNTERS::default();
    // SAFETY: Child owns a live process handle for this call; counters is writable
    // and the API does not retain either argument.
    unsafe { GetProcessIoCounters(HANDLE(child.as_raw_handle()), &mut counters) }
        .map_err(|error| format!("cannot observe fixed adapter read activity: {error}"))?;
    Ok(counters.ReadTransferCount)
}

pub(super) struct Watchdog {
    idle_limit: Duration,
    total_limit: Duration,
    last_read_at: Duration,
    read_bytes: u64,
}

impl Watchdog {
    pub(super) fn new(idle_limit: Duration, total_limit: Duration) -> Self {
        Self {
            idle_limit,
            total_limit,
            last_read_at: Duration::ZERO,
            read_bytes: 0,
        }
    }

    pub(super) fn observe(&mut self, elapsed: Duration, read_bytes: u64) -> Result<(), String> {
        if read_bytes > self.read_bytes {
            self.read_bytes = read_bytes;
            self.last_read_at = elapsed;
        }
        let idle = elapsed.saturating_sub(self.last_read_at);
        let reason = if elapsed >= self.total_limit {
            "overall execution budget exhausted"
        } else if idle >= self.idle_limit {
            "no read progress within inactivity limit"
        } else {
            return Ok(());
        };
        Err(format!(
            "{reason}; elapsed={}ms, last_read={}ms, read_bytes={}",
            elapsed.as_millis(),
            self.last_read_at.as_millis(),
            self.read_bytes
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::Watchdog;
    use std::time::Duration;

    #[test]
    fn active_reads_survive_original_cutoff_but_cannot_extend_total_budget() {
        let mut monitor = Watchdog::new(Duration::from_secs(3), Duration::from_secs(10));
        for seconds in 0..10 {
            monitor
                .observe(Duration::from_secs(seconds), seconds * 4096)
                .unwrap();
        }
        assert!(
            monitor
                .observe(Duration::from_secs(10), 40960)
                .unwrap_err()
                .contains("overall execution budget exhausted")
        );
    }

    #[test]
    fn stall_is_measured_from_last_read_and_reports_evidence() {
        let mut monitor = Watchdog::new(Duration::from_secs(3), Duration::from_secs(10));
        monitor.observe(Duration::from_secs(2), 8192).unwrap();
        monitor.observe(Duration::from_secs(4), 8192).unwrap();
        let error = monitor.observe(Duration::from_secs(5), 8192).unwrap_err();
        assert!(error.contains("no read progress"));
        assert!(error.contains("last_read=2000ms, read_bytes=8192"));
    }

    #[test]
    fn native_read_activity_keeps_adapter_alive_beyond_idle_limit() {
        let project = crate::project_configuration().unwrap();
        let executable = std::env::current_exe().unwrap();
        let script = format!(
            "$file = [IO.File]::OpenRead({}); try {{ $buffer = New-Object byte[] 65536; for ($i = 0; $i -lt 30; $i++) {{ $file.Position = 0; [void]$file.Read($buffer, 0, $buffer.Length); Start-Sleep -Milliseconds 100 }}; [Console]::Out.Write('complete') }} finally {{ $file.Dispose() }}",
            crate::powershell_literal(&executable.to_string_lossy())
        );
        assert_eq!(
            crate::run_supervised(
                &script,
                Duration::from_secs(2),
                Duration::from_secs(8),
                &project.guest.powershell_path,
                "read fixture"
            )
            .unwrap(),
            "complete"
        );
    }

    #[test]
    fn stalled_adapter_is_reaped_with_phase_and_read_evidence() {
        let project = crate::project_configuration().unwrap();
        let started = std::time::Instant::now();
        let error = crate::run_supervised(
            "Start-Sleep -Seconds 10",
            Duration::from_secs(1),
            Duration::from_secs(8),
            &project.guest.powershell_path,
            "stall fixture",
        )
        .unwrap_err();
        assert!(error.starts_with("stall fixture: no read progress"));
        assert!(error.contains("read_bytes="));
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn delayed_request_spends_shared_budget_and_cannot_begin_unfunded_transition() {
        let total = Duration::from_secs(600);
        let transition = Duration::from_secs(120);
        assert_eq!(
            crate::adapter_budget(total, transition, Duration::from_secs(45)),
            Duration::from_secs(405)
        );
        assert!(
            crate::require_transition_budget(total, transition, Duration::from_secs(450)).is_ok()
        );
        assert!(
            crate::require_transition_budget(total, transition, Duration::from_secs(451)).is_err()
        );
        assert_eq!(
            crate::adapter_budget(total, transition, Duration::from_secs(700)),
            Duration::ZERO
        );
    }
}
