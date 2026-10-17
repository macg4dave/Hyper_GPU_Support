//! Opt-in qualification tracing. A protected installation marker enables fixed,
//! administrator-owned per-process logs; no caller-selected privileged paths.
use crate::work_counts::{self, Counter, Counts};
use std::{
    fs::File,
    io::Write,
    path::Path,
    sync::{Mutex, OnceLock},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

static LOG: OnceLock<Result<Option<Mutex<File>>, String>> = OnceLock::new();
pub(crate) fn enabled() -> Result<bool, String> {
    Ok(logger()?.is_some())
}
fn logger() -> Result<Option<&'static Mutex<File>>, String> {
    LOG.get_or_init(|| {
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        let root = if executable
            .file_name()
            .is_some_and(|n| n == "hyper-gpu-guest.exe")
        {
            crate::runner::folder(&windows::Win32::UI::Shell::FOLDERID_ProgramFiles)?
                .join("HyperGpuSupport/Guest")
        } else {
            crate::runner::install_directory()?
        };
        open_logger(&root)
    })
    .as_ref()
    .map(|log| log.as_ref())
    .map_err(Clone::clone)
}
fn open_logger(root: &Path) -> Result<Option<Mutex<File>>, String> {
    // Frontend artifact checks also use these spans. The enrolled ordinary token
    // can read the installation but cannot create administrator-owned trace files.
    if !crate::process::is_elevated()? {
        return Ok(None);
    }
    let marker = root.join("diagnostics.enabled");
    if !marker.try_exists().map_err(|e| e.to_string())? {
        return Ok(None);
    }
    crate::security::verify(&marker)?;
    let directory = root.join("diagnostics");
    crate::security::verify(&directory)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let path = directory.join(format!("trace-{now}-{}.jsonl", std::process::id()));
    let file = crate::security::create_file(&path, "S-1-5-32-544")?;
    Ok(Some(Mutex::new(file)))
}
fn event(
    log: Option<&Mutex<File>>,
    stage: &str,
    context: &str,
    status: &str,
    elapsed: u128,
    error: Option<&str>,
    work: Option<Counts>,
) -> Result<(), String> {
    let Some(log) = log else {
        return Ok(());
    };
    let mut record = serde_json::json!({"utc_unix_ms":SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis(), "pid":std::process::id(), "thread":format!("{:?}", std::thread::current().id()), "stage":stage,"context":context,"status":status,"duration_ms":elapsed,"significant_delay":elapsed >= 5000,"error":error});
    if let Some(work) = work {
        record["work_schema"] = serde_json::json!(1);
        record["work"] = serde_json::to_value(work).map_err(|e| e.to_string())?;
    }
    let started = Instant::now();
    let mut file = log.lock().map_err(|_| "diagnostic log lock failed")?;
    serde_json::to_writer(&mut *file, &record).map_err(|e| e.to_string())?;
    let result = file
        .write_all(b"\n")
        .and_then(|_| file.sync_data())
        .map_err(|e| format!("durable diagnostic write failed: {e}"));
    work_counts::add(Counter::DiagnosticFlushes, 1);
    work_counts::add(
        Counter::DiagnosticFlushNs,
        started.elapsed().as_nanos().try_into().unwrap_or(u64::MAX),
    );
    result
}
/// Records entry before work, duration/outcome afterward, and interrupted unwinds.
pub(crate) struct Span<'a> {
    stage: String,
    context: String,
    start: Instant,
    finished: bool,
    before: Counts,
    log: Option<&'a Mutex<File>>,
    aggregate: bool,
}
impl Span<'static> {
    pub(crate) fn start(stage: &str, context: &str) -> Result<Self, String> {
        Self::with_log(stage, context, logger()?)
    }
}
impl<'a> Span<'a> {
    fn with_log(stage: &str, context: &str, log: Option<&'a Mutex<File>>) -> Result<Self, String> {
        let before = work_counts::snapshot();
        let start = Instant::now();
        event(log, stage, context, "start", 0, None, None)?;
        Ok(Self {
            stage: stage.into(),
            context: context.into(),
            start,
            finished: false,
            before,
            log,
            // Keep counters off existing per-file records. Inclusive operation
            // and payload/transport summaries are sufficient for comparison.
            aggregate: matches!(
                stage,
                "runner-request"
                    | "worker-command"
                    | "guest-worker"
                    | "payload-processing"
                    | "guest-transport"
                    | "artifact-authentication"
                    | "worker-availability"
            ),
        })
    }
    pub(crate) fn finish<T, E: std::fmt::Display>(
        &mut self,
        result: &Result<T, E>,
    ) -> Result<(), String> {
        let error = result.as_ref().err().map(|error| {
            let message = error.to_string();
            if self.aggregate {
                // Aggregate measurement records must not expand adapter prose,
                // request contents or credentials into another diagnostic sink.
                crate::reporting::ComponentFailure::decode(&message)
                    .map(crate::reporting::ComponentFailure::into_error)
                    .unwrap_or_else(|| "operation failed".into())
            } else {
                message
            }
        });
        event(
            self.log,
            &self.stage,
            &self.context,
            if result.is_ok() { "done" } else { "failed" },
            self.start.elapsed().as_millis(),
            error.as_deref(),
            self.aggregate
                .then(|| work_counts::snapshot().since(self.before)),
        )?;
        self.finished = true;
        Ok(())
    }
}
impl Drop for Span<'_> {
    fn drop(&mut self) {
        if !self.finished {
            // The enclosing adapter records the concrete error. Never mask it
            // with a second error while unwinding.
            if let Err(error) = event(
                self.log,
                &self.stage,
                &self.context,
                "incomplete",
                self.start.elapsed().as_millis(),
                None,
                self.aggregate
                    .then(|| work_counts::snapshot().since(self.before)),
            ) {
                eprintln!("{error}");
            }
        }
    }
}
pub(crate) fn run<T>(
    stage: &str,
    context: &str,
    action: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let mut span = Span::start(stage, context)?;
    let result = action();
    span.finish(&result)?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_records_attribute_nested_failure_and_incomplete_work() {
        let path = std::env::temp_dir().join(format!(
            "hypergpu-work-counts-{}-{}.jsonl",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let log = Mutex::new(File::create_new(&path).unwrap());
        {
            let mut outer = Span::with_log("runner-request", "fixed", Some(&log)).unwrap();
            work_counts::add(Counter::ProviderQueries, 2);
            let mut inner = Span::with_log("payload-processing", "fixed", Some(&log)).unwrap();
            work_counts::add(Counter::ProviderQueries, 3);
            inner
                .finish(&Err::<(), _>("fixture unavailable: private-value"))
                .unwrap();
            {
                let _interrupted = Span::with_log("guest-transport", "fixed", Some(&log)).unwrap();
                work_counts::add(Counter::ProcessStarts, 1);
            }
            outer.finish(&Ok::<(), String>(())).unwrap();
        }
        let mut file_span = Span::with_log("hash-file", "fixed", Some(&log)).unwrap();
        file_span.finish(&Ok::<(), String>(())).unwrap();
        drop(file_span);
        drop(log);
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        let records: Vec<serde_json::Value> = text
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let statuses: Vec<_> = records
            .iter()
            .map(|r| r["status"].as_str().unwrap())
            .collect();
        assert_eq!(
            statuses,
            [
                "start",
                "start",
                "failed",
                "start",
                "incomplete",
                "done",
                "start",
                "done"
            ]
        );
        assert_eq!(records[2]["work"]["provider_queries"], 3);
        assert_eq!(records[2]["error"], "operation failed");
        assert!(!text.contains("private-value"));
        assert_eq!(records[4]["work"]["process_starts"], 1);
        assert_eq!(records[5]["work"]["provider_queries"], 5);
        assert_eq!(records[5]["work"]["process_starts"], 1);
        // Includes entry and nested durable writes, excludes its own terminal
        // flush because a record cannot measure its subsequent sync_data call.
        assert_eq!(records[5]["work"]["diagnostic_flushes"], 5);
        assert_eq!(records[5]["work_schema"], 1);
        assert!(
            records[7].get("work").is_none(),
            "no per-file counter expansion"
        );
    }

    #[test]
    fn disabled_diagnostics_preserve_actions_results_and_do_not_flush() {
        assert!(!crate::process::is_elevated().unwrap());
        let before = work_counts::snapshot();
        let mut effects = 0;
        let result: Result<(), String> = run("fixture", "fixed", || {
            effects += 1;
            work_counts::add(Counter::SettingsWrites, 1);
            Err("fixture failure after effect".into())
        });
        let measured = work_counts::snapshot().since(before);
        assert_eq!(result.unwrap_err(), "fixture failure after effect");
        assert_eq!(effects, 1);
        assert_eq!(measured.get(Counter::SettingsWrites), 1);
        assert_eq!(measured.get(Counter::DiagnosticFlushes), 0);
        assert_eq!(measured.get(Counter::DiagnosticFlushNs), 0);
    }

    #[test]
    fn ordinary_frontend_ignores_diagnostic_opt_in_without_attempting_a_write() {
        assert!(
            !crate::process::is_elevated().unwrap(),
            "run non-hardware tests without elevation"
        );
        let root =
            std::env::temp_dir().join(format!("hypergpu-frontend-trace-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let marker = root.join("diagnostics.enabled");
        std::fs::write(&marker, b"").unwrap();
        let result = open_logger(&root);
        let entries = std::fs::read_dir(&root).unwrap().count();
        std::fs::remove_file(marker).unwrap();
        std::fs::remove_dir(root).unwrap();
        assert!(result.unwrap().is_none());
        assert_eq!(entries, 1, "ordinary frontend creates no diagnostic output");
    }
}
