/// Module: Distrobox container upgrades.
use crate::config::Config;
use crate::color::{RESET, _RED, _YELLOW};
use crate::runner::{command_exists, run_cmd, RunResult};
use crate::ui::box_line;

pub fn run(cfg: &Config) -> bool {
    if !command_exists("distrobox") {
        box_line(&format!("{}✘ distrobox not found{}", _RED.code(), RESET.code()));
        return false;
    }

    match run_cmd(&["distrobox", "upgrade", "--all"], cfg) {
        RunResult::Failed(_) => {
            box_line(&format!("{}◆ distrobox returned non-zero (possibly nothing to upgrade){}", _YELLOW.code(), RESET.code()));
        }
        RunResult::Error(e) => {
            box_line(&format!("{}✘ distrobox error: {e}{}", _RED.code(), RESET.code()));
        }
        _ => {}
    }

    true
}