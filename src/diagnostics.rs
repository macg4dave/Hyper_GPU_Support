//! Opt-in qualification tracing. A protected installation marker enables fixed,
//! administrator-owned per-process logs; no caller-selected privileged paths.
use std::{
    fs::File,
    io::Write,
    sync::{Mutex, OnceLock},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

static LOG: OnceLock<Result<Option<Mutex<File>>, String>> = OnceLock::new();
pub(crate) fn enabled() -> Result<bool, String> {
    Ok(logger()?.is_some())
}
fn logger() -> Result<Option<&'static Mutex<File>>, String> {
    LOG.get_or_init(|| {
        if std::env::args().any(|a| a == "--mock-gui") {
            return Ok(None);
        }
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
    })
    .as_ref()
    .map(|log| log.as_ref())
    .map_err(Clone::clone)
}
fn event(
    stage: &str,
    context: &str,
    status: &str,
    elapsed: u128,
    error: Option<&str>,
) -> Result<(), String> {
    let Some(log) = logger()? else {
        return Ok(());
    };
    let record = serde_json::json!({"utc_unix_ms":SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis(), "pid":std::process::id(), "thread":format!("{:?}", std::thread::current().id()), "stage":stage,"context":context,"status":status,"duration_ms":elapsed,"significant_delay":elapsed >= 5000,"error":error});
    let mut file = log.lock().map_err(|_| "diagnostic log lock failed")?;
    serde_json::to_writer(&mut *file, &record).map_err(|e| e.to_string())?;
    file.write_all(b"\n")
        .and_then(|_| file.sync_data())
        .map_err(|e| format!("durable diagnostic write failed: {e}"))
}
/// Records entry before work, duration/outcome afterward, and interrupted unwinds.
pub(crate) struct Span {
    stage: String,
    context: String,
    start: Instant,
    finished: bool,
}
impl Span {
    pub(crate) fn start(stage: &str, context: &str) -> Result<Self, String> {
        event(stage, context, "start", 0, None)?;
        Ok(Self {
            stage: stage.into(),
            context: context.into(),
            start: Instant::now(),
            finished: false,
        })
    }
    pub(crate) fn finish<T, E: std::fmt::Display>(
        &mut self,
        result: &Result<T, E>,
    ) -> Result<(), String> {
        let error = result.as_ref().err().map(ToString::to_string);
        event(
            &self.stage,
            &self.context,
            if result.is_ok() { "done" } else { "failed" },
            self.start.elapsed().as_millis(),
            error.as_deref(),
        )?;
        self.finished = true;
        Ok(())
    }
}
impl Drop for Span {
    fn drop(&mut self) {
        if !self.finished {
            // The enclosing adapter records the concrete error. Never mask it
            // with a second error while unwinding.
            if let Err(error) = event(
                &self.stage,
                &self.context,
                "incomplete",
                self.start.elapsed().as_millis(),
                None,
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
