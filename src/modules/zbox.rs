/// Module: zbox container/package manager.
use crate::config::Config;
use crate::color::{RESET, _RED, _YELLOW};
use crate::runner::{command_exists, run_cmd, RunResult};
use crate::ui::box_line;

pub fn run(cfg: &Config) -> bool {
    if !command_exists("zbox") {
        box_line(&format!("{}✘ zbox not found{}", _RED.code(), RESET.code()));
        return false;
    }

    match run_cmd(&["zbox", "update", "--all"], cfg) {
        RunResult::DryRun | RunResult::Success => true,
        RunResult::Failed(_) => {
            box_line(&format!("{}◆ zbox update returned non-zero{}", _YELLOW.code(), RESET.code()));
            false
        }
        RunResult::Error(e) => {
            box_line(&format!("{}✘ zbox error: {e}{}", _RED.code(), RESET.code()));
            false
        }
    }
}