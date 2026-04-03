/// Command execution engine.
use std::io::{BufRead, BufReader};
use std::process::{Command, ExitStatus, Stdio};
use std::time::Instant;

use crate::config::Config;
use crate::logger::{self, debug};
use crate::state::State;
use crate::ui::{box_line, section, section_end};

#[derive(Debug)]
pub enum RunResult {
    DryRun,
    Success,
    Failed(Box<ExitStatus>),
    Error(String),
}

/// Strip ANSI escapes and carriage returns — delegates to color::strip_ansi,
/// then additionally strips bare \r which appear in some tool output.
fn strip_ansi(s: &str) -> String {
    crate::color::strip_ansi(s).replace('\r', "")
}

/// Returns true for lines that are pure progress noise.
fn is_progress(s: &str) -> bool {
    let s = s.trim();
    // "Downloading…: 53%", "Idle…: 100%" etc.
    if let Some((_prefix, suffix)) = s.rsplit_once(':') {
        let v = suffix.trim();
        if let Some(digits) = v.strip_suffix('%') {
            if digits.trim().parse::<u32>().is_ok() {
                return true;
            }
        }
    }
    // "Updating 1/2…", "Updating 2/2… ████ 43%" etc.
    if s.starts_with("Updating ") && s.contains('/') {
        return true;
    }
    // lines containing block-element progress bars
    if s.contains('█') || s.contains('▏') || s.contains('▌')
        || s.contains('▍') || s.contains('▊') {
        return true;
    }
    false
}

/// Run a command, streaming each output line inside box walls.
pub fn run_cmd(args: &[&str], cfg: &Config) -> RunResult {
    if args.is_empty() {
        return RunResult::Error("empty command".into());
    }
    if cfg.dry_run {
        box_line(&format!(
            "{d}[dry-run]{r} {}",
            args.join(" "),
            d = crate::color::DIM.code(),
            r = crate::color::RESET.code(),
        ));
        return RunResult::DryRun;
    }

    debug(&format!("exec: {}", args.join(" ")));

    let mut child = match Command::new(args[0])
        .args(&args[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())   // capture stderr so it prints inside the box
        .spawn()
    {
        Ok(c)  => c,
        Err(e) => return RunResult::Error(e.to_string()),
    };

    // Stream both stdout and stderr through box_line.
    // Both paths apply the same is_progress dedup: only the last line of a
    // rapid progress burst is shown (e.g. fwupdmgr "Downloading…: N%").
    let stderr_handle = child.stderr.take().map(|stderr| {
        std::thread::spawn(move || {
            let mut pending: Option<String> = None;
            for line in BufReader::new(stderr).lines().flatten() {
                let trimmed = strip_ansi(line.trim());
                if trimmed.is_empty() { continue; }
                if is_progress(&trimmed) {
                    pending = Some(trimmed);
                } else {
                    if let Some(p) = pending.take() {
                        box_line(&p);
                    }
                    box_line(&trimmed);
                }
            }
            if let Some(p) = pending.take() {
                box_line(&p);
            }
        })
    });

    if let Some(stdout) = child.stdout.take() {
        let mut pending: Option<String> = None;
        for line in BufReader::new(stdout).lines().flatten() {
            let trimmed = strip_ansi(line.trim());
            if trimmed.is_empty() { continue; }
            if is_progress(&trimmed) {
                pending = Some(trimmed);
            } else {
                if let Some(p) = pending.take() {
                    box_line(&p);
                }
                box_line(&trimmed);
            }
        }
        if let Some(p) = pending.take() {
            box_line(&p);
        }
    }

    if let Some(h) = stderr_handle {
        let _ = h.join();
    }

    match child.wait() {
        Ok(s) if s.success() => RunResult::Success,
        Ok(s)                => RunResult::Failed(Box::new(s)),
        Err(e)               => RunResult::Error(e.to_string()),
    }
}

/// Like `run_cmd` but returns captured stdout — no streaming, no box output.
pub fn run_cmd_output(args: &[&str], cfg: &Config) -> Result<String, String> {
    if args.is_empty() {
        return Err("empty command".into());
    }
    if cfg.dry_run {
        logger::info(&format!("[dry-run] {} (captured)", args.join(" ")));
        return Ok(String::new());
    }

    debug(&format!("exec (capture): {}", args.join(" ")));

    let out = Command::new(args[0])
        .args(&args[1..])
        .output()
        .map_err(|e| e.to_string())?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).into_owned())
    }
}

pub fn command_exists(name: &str) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            if dir.is_empty() { continue; }
            if std::path::Path::new(dir).join(name).is_file() {
                return true;
            }
        }
    }
    false
}

/// Run `func` as a named module: open box, run, close box, update state.
pub fn run_module<F>(name: &str, func: F, cfg: &Config, state: &mut State)
where
    F: FnOnce(&Config) -> bool,
{
    state.total += 1;
    section(name);
    let start = Instant::now();
    let ok = func(cfg);
    let elapsed = start.elapsed().as_secs();
    if ok {
        state.success += 1;
        box_line(&crate::color::_GREEN.apply(&format!("✔ completed in {elapsed}s")));
    } else {
        state.failed += 1;
        box_line(&crate::color::_RED.apply(&format!("✘ failed after {elapsed}s")));
    }
    section_end();
}