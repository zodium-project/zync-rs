/// Structured logger: writes to terminal (with ANSI) and to a log file
/// (ANSI-stripped, with timestamp and level prefix).
///
/// The log file location honours `$XDG_STATE_HOME` exactly as the original.
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::color::{
    strip_ansi, status_debug, status_failed, status_info, status_warning,
};
use crate::config::Config;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Failed,
    Info,
    Warning,
    Debug,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::Failed  => "FAILED",
            Level::Info    => "INFO",
            Level::Warning => "WARNING",
            Level::Debug   => "DEBUG",
        }
    }
}

// ──── Global config reference ────

static CFG: OnceLock<(bool, bool)> = OnceLock::new(); // (quiet, verbose)
static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
static LOG_MUTEX: Mutex<()> = Mutex::new(());

pub fn init(cfg: &Config) {
    CFG.get_or_init(|| (cfg.quiet, cfg.verbose));
    LOG_PATH.get_or_init(|| {
        let dir = std::env::var("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs_home().join(".local").join("state")
            });
        dir.join("zync.log")
    });

    // Ensure the log directory exists (best-effort, never fatal).
    if let Some(parent) = LOG_PATH.get().and_then(|p| p.parent()) {
        let _ = fs::create_dir_all(parent);
    }
    if let Some(path) = LOG_PATH.get() {
        // Touch the file so it exists.
        let _ = OpenOptions::new().create(true).append(true).open(path);
    }
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

/// Core log function — mirrors bash `log()`.
pub fn log(msg: &str, level: Level) {
    let (quiet, verbose) = CFG.get().copied().unwrap_or((false, false));

    // Suppress DEBUG unless --verbose.
    if level == Level::Debug && !verbose {
        return;
    }

    let prefix = match level {
        Level::Failed  => status_failed(),
        Level::Info    => status_info(),
        Level::Warning => status_warning(),
        Level::Debug   => status_debug(),
    };

    let line = format!("{prefix} {msg}");

    // Terminal output (suppressed when --quiet, unless it's a failure).
    if !quiet || level == Level::Failed {
        println!("{line}");
    }

    // File output (ANSI-stripped, always written).
    let plain = strip_ansi(&line);
    let timestamp = current_timestamp();
    let file_line = format!("{timestamp} | {} | {plain}\n", level.as_str());

    if let Some(path) = LOG_PATH.get() {
        let _guard = LOG_MUTEX.lock();
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = f.write_all(file_line.as_bytes());
            let _ = f.flush();
        }
    }
}

/// Convenience wrappers.
pub fn info(msg: &str)    { log(msg, Level::Info); }
pub fn warning(msg: &str) { log(msg, Level::Warning); }
pub fn debug(msg: &str)   { log(msg, Level::Debug); }

fn current_timestamp() -> String {
    // No external crate: read from /proc/driver/rtc or use a simple approach.
    // We shell out to `date` only once per call — this is log overhead, acceptable.
    // Actually: use std::time and manual formatting to stay dep-free.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format_unix_time(secs)
}

/// Format a Unix timestamp as "YYYY-MM-DD HH:MM:SS" without any crate.
fn format_unix_time(secs: u64) -> String {
    // Days since epoch → date via Gregorian calendar algorithm.
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;

    // Civil date from days since 1970-01-01 (Euclidean algorithm).
    let z = days as i64 + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };

    format!("{y:04}-{mo:02}-{d:02} {h:02}:{m:02}:{s:02}")
}