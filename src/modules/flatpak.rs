/// Module: Flatpak application updates.
use crate::config::Config;
use crate::color::{RESET, _RED};
use crate::runner::{command_exists, run_cmd, RunResult};
use crate::ui::box_line;

pub fn run(cfg: &Config) -> bool {
    if !command_exists("flatpak") {
        box_line(&format!("{}✘ flatpak not found{}", _RED.code(), RESET.code()));
        return false;
    }

    match run_cmd(&["flatpak", "update", "-y"], cfg) {
        RunResult::DryRun | RunResult::Success => {}
        _ => return false,
    }

    if cfg.maintain {
        run_cmd(&["flatpak", "uninstall", "--unused", "-y"], cfg);
    }

    true
}