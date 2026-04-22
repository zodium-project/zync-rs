/// Structured logger: writes to terminal (with ANSI) and to a log file
/// (ANSI-stripped, with timestamp and level prefix).
///
/// The log file location honours `$XDG_STATE_HOME` exactly as the original.
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use chrono;

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
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}