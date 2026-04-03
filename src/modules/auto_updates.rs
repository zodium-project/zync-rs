/// Module: interactive systemd timer management for automatic updates.
use std::io::Write;

use crate::color::{BOLD, DIM, RESET, _CYAN, _GREEN, _RED};
use crate::config::Config;
use crate::runner::{command_exists, run_cmd, RunResult};
use crate::ui::{bare_prompt, box_line, info_box, section, section_end};

const SERVICE_FILE: &str = "/etc/systemd/system/zync-auto-update.service";
const TIMER_FILE:   &str = "/etc/systemd/system/zync-auto-update.timer";
const DEFAULT_CMD:  &str = "/usr/bin/zync --rpm-ostree --no-reboot";
const DEFAULT_INTERVAL: &str = "weekly";

pub fn run(cfg: &Config) -> bool {
    if timer_is_enabled() {
        handle_enabled(cfg)
    } else {
        handle_disabled(cfg)
    }
}

fn timer_is_enabled() -> bool {
    std::process::Command::new("systemctl")
        .args(["is-enabled", "zync-auto-update.timer"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ── enabled flow ──────────────────────────────────────────────────────────

fn handle_enabled(cfg: &Config) -> bool {
    let current_interval = read_unit_field(TIMER_FILE, "OnCalendar=")
        .unwrap_or_else(|| "unknown".into());
    let current_cmd = read_unit_field(SERVICE_FILE, "ExecStart=")
        .unwrap_or_else(|| "unknown".into());
    let next_run = systemctl_property("zync-auto-update.timer", "NextElapseUSecRealtime")
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".into());

    let status_line = format!(
        "{}status{}  {}enabled{}",
        DIM.code(), RESET.code(), _GREEN.code(), RESET.code(),
    );
    let sched_line  = format!("{}schedule{} {}", DIM.code(), RESET.code(), interval_friendly(&current_interval));
    let cmd_line    = format!("{}command{}  {}", DIM.code(), RESET.code(), current_cmd);
    let next_line   = format!("{}next run{} {}", DIM.code(), RESET.code(), next_run);
    info_box("* Status", 8, &[&status_line, &sched_line, &cmd_line, &next_line]);

    info_box(
        &format!("{cy}{bold}* Options{r}", cy = _CYAN.code(), bold = BOLD.code(), r = RESET.code()),
        9,
        &[&format!(
            "  {cy}[1]{r} Disable   {cy}[2]{r} Edit   {cy}[3]{r} Cancel",
            cy = _CYAN.code(), r = RESET.code(),
        )],
    );

    let choice = bare_prompt("choice [1/2/3]");

    match choice.trim() {
        "1" => {
            section("Disabling");
            run_cmd(&["sudo", "systemctl", "disable", "--now", "zync-auto-update.timer"], cfg);
            run_cmd(&["sudo", "systemctl", "stop", "zync-auto-update.service"], cfg);
            run_cmd(&["sudo", "rm", "-f", SERVICE_FILE, TIMER_FILE], cfg);
            run_cmd(&["sudo", "systemctl", "daemon-reload"], cfg);
            box_line(&format!("{}✔ automatic updates disabled{}", _GREEN.code(), RESET.code()));
            section_end();
            true
        }
        "2" => edit_flow(&current_interval, &current_cmd, cfg),
        _   => { box_line("cancelled"); true }
    }
}

fn edit_flow(current_interval: &str, current_cmd: &str, cfg: &Config) -> bool {
    info_box(
        &format!("{cy}{bold}* Time Interval{r}", cy = _CYAN.code(), bold = BOLD.code(), r = RESET.code()),
        15,
        &[
            &format!("{}current:{} {}", DIM.code(), RESET.code(), interval_friendly(current_interval)),
            &format!("  {cy}[1]{r} Daily   {cy}[2]{r} Weekly   {cy}[3]{r} Fortnightly",
                cy = _CYAN.code(), r = RESET.code()),
        ],
    );

    let ic = bare_prompt("interval [1/2/3, blank = keep]");
    let new_interval = match ic.trim() {
        "1" => "daily".to_owned(),
        "2" => "weekly".to_owned(),
        "3" => "*-*-01,15 00:00:00".to_owned(),
        _   => current_interval.to_owned(),
    };

    if !validate_calendar(&new_interval) {
        box_line(&format!("{}✘ invalid schedule{}", _RED.code(), RESET.code()));
        return false;
    }

    info_box(
        &format!("{cy}{bold}* Command To Run{r}", cy = _CYAN.code(), bold = BOLD.code(), r = RESET.code()),
        16,
        &[&format!("{}current:{} {}", DIM.code(), RESET.code(), current_cmd)],
    );

    let nc = bare_prompt("new command [blank = keep]");
    let new_cmd = if nc.trim().is_empty() { current_cmd.to_owned() } else { nc.trim().to_owned() };

    if contains_shell_operators(&new_cmd) {
        box_line(&format!("{}✘ shell operators not allowed{}", _RED.code(), RESET.code()));
        return false;
    }
    if !binary_exists(&new_cmd) {
        box_line(&format!("{}✘ command not found: {}{}", _RED.code(), first_word(&new_cmd), RESET.code()));
        return false;
    }

    if !write_units(&new_cmd, &new_interval, cfg) { return false; }

    section("Applying");
    run_cmd(&["sudo", "systemctl", "daemon-reload"], cfg);
    match run_cmd(&["sudo", "systemctl", "restart", "zync-auto-update.timer"], cfg) {
        RunResult::DryRun | RunResult::Success => {}
        _ => { section_end(); return false; }
    }
    box_line(&format!("{}✔ automatic updates updated{}", _GREEN.code(), RESET.code()));
    box_line(&format!("  schedule  {}", interval_friendly(&new_interval)));
    box_line(&format!("  command   {new_cmd}"));
    section_end();
    true
}

// ── disabled flow ─────────────────────────────────────────────────────────

fn handle_disabled(cfg: &Config) -> bool {
    info_box(
        &format!("{cy}{bold}* Status{r}", cy = _CYAN.code(), bold = BOLD.code(), r = RESET.code()),
        8,
        &[&format!("{}status{}  {}disabled{}", DIM.code(), RESET.code(), _RED.code(), RESET.code())],
    );

    let ans = bare_prompt("enable automatic updates? [Y/n]");
    if matches!(ans.trim(), "n" | "N") {
        return true;
    }

    info_box(
        &format!("{cy}{bold}* Time Interval{r}", cy = _CYAN.code(), bold = BOLD.code(), r = RESET.code()),
        15,
        &[&format!(
            "  {cy}[1]{r} Daily   {cy}[2]{r} Weekly   {cy}[3]{r} Fortnightly",
            cy = _CYAN.code(), r = RESET.code(),
        )],
    );

    let ic = bare_prompt("interval [1/2/3, default: 2]");
    let new_interval = match ic.trim() {
        "1" => "daily".to_owned(),
        "3" => "*-*-01,15 00:00:00".to_owned(),
        _   => DEFAULT_INTERVAL.to_owned(),
    };

    if !validate_calendar(&new_interval) {
        box_line(&format!("{}✘ invalid schedule{}", _RED.code(), RESET.code()));
        return false;
    }

    info_box(
        &format!("{cy}{bold}* Command To Run{r}", cy = _CYAN.code(), bold = BOLD.code(), r = RESET.code()),
        16,
        &[&format!("{}default:{} {DEFAULT_CMD}", DIM.code(), RESET.code())],
    );

    let nc = bare_prompt("command [blank = default]");
    let new_cmd = if nc.trim().is_empty() { DEFAULT_CMD.to_owned() } else { nc.trim().to_owned() };

    if contains_shell_operators(&new_cmd) {
        box_line(&format!("{}✘ shell operators not allowed{}", _RED.code(), RESET.code()));
        return false;
    }
    if !binary_exists(&new_cmd) {
        box_line(&format!("{}✘ command not found: {}{}", _RED.code(), first_word(&new_cmd), RESET.code()));
        return false;
    }

    if !write_units(&new_cmd, &new_interval, cfg) { return false; }

    section("Enabling");
    run_cmd(&["sudo", "systemctl", "daemon-reload"], cfg);
    match run_cmd(&["sudo", "systemctl", "enable", "--now", "zync-auto-update.timer"], cfg) {
        RunResult::DryRun | RunResult::Success => {}
        _ => { section_end(); return false; }
    }
    box_line(&format!("{}✔ automatic updates enabled{}", _GREEN.code(), RESET.code()));
    box_line(&format!("  schedule  {}", interval_friendly(&new_interval)));
    box_line(&format!("  command   {new_cmd}"));
    section_end();
    true
}

// ── unit file writing ─────────────────────────────────────────────────────

fn write_units(cmd: &str, interval: &str, cfg: &Config) -> bool {
    let service = format!(
        "[Unit]\nDescription=zync Automatic Update\nAfter=network-online.target\n\
         Wants=network-online.target\n\n[Service]\nType=oneshot\nExecStart={cmd}\n\
         StandardOutput=journal\nStandardError=journal\nNice=10\n\
         IOSchedulingClass=best-effort\nTimeoutStartSec=2h\n"
    );
    let timer = format!(
        "[Unit]\nDescription=zync Automatic Update Timer\nAfter=network-online.target\n\
         ConditionACPower=true\n\n[Timer]\nOnCalendar={interval}\nPersistent=true\n\
         RandomizedDelaySec=10min\n\n[Install]\nWantedBy=timers.target\n"
    );

    if cfg.dry_run {
        box_line(&format!("{}[dry-run]{} write {SERVICE_FILE}", DIM.code(), RESET.code()));
        box_line(&format!("{}[dry-run]{} write {TIMER_FILE}",   DIM.code(), RESET.code()));
        return true;
    }

    if !tee_write(SERVICE_FILE, &service) {
        box_line(&format!("{}✘ failed to write {SERVICE_FILE}{}", _RED.code(), RESET.code()));
        return false;
    }
    if !tee_write(TIMER_FILE, &timer) {
        box_line(&format!("{}✘ failed to write {TIMER_FILE}{}", _RED.code(), RESET.code()));
        return false;
    }
    true
}

fn tee_write(path: &str, content: &str) -> bool {
    let mut child = match std::process::Command::new("sudo")
        .args(["tee", path])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
    {
        Ok(c)  => c,
        Err(_) => return false,
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(content.as_bytes());
    }
    child.wait().map(|s| s.success()).unwrap_or(false)
}

// ── helpers ───────────────────────────────────────────────────────────────

fn read_unit_field(path: &str, prefix: &str) -> Option<String> {
    std::fs::read_to_string(path).ok()?
        .lines()
        .find(|l| l.starts_with(prefix))
        .map(|l| l[prefix.len()..].trim().to_owned())
}

fn systemctl_property(unit: &str, prop: &str) -> Option<String> {
    let out = std::process::Command::new("systemctl")
        .args(["show", unit, "-p", prop])
        .output().ok()?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find(|l| l.contains('='))
        .and_then(|l| l.splitn(2, '=').nth(1))
        .map(|v| v.trim().to_owned())
}

fn validate_calendar(expr: &str) -> bool {
    std::process::Command::new("systemd-analyze")
        .args(["calendar", expr])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn binary_exists(cmd: &str) -> bool {
    command_exists(first_word(cmd))
}

fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or(s)
}

fn contains_shell_operators(s: &str) -> bool {
    s.contains(';') || s.contains('&') || s.contains('|')
        || s.contains('$') || s.contains('`')
}

fn interval_friendly(interval: &str) -> &str {
    match interval {
        "daily"  => "Daily",
        "weekly" => "Weekly",
        s if s.contains("01,15") => "Fortnightly",
        other => other,
    }
}