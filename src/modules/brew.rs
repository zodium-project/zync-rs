/// Module: Homebrew package updates.
use crate::config::Config;
use crate::color::{RESET, _RED, _YELLOW};
use crate::runner::{command_exists, run_cmd, RunResult};
use crate::ui::box_line;

pub fn run(cfg: &Config) -> bool {
    if !command_exists("brew") {
        box_line(&format!("{}✘ brew not found{}", _RED.code(), RESET.code()));
        return false;
    }

    match run_cmd(&["brew", "update"], cfg) {
        RunResult::DryRun | RunResult::Success => {}
        _ => return false,
    }

    // `brew upgrade` may return non-zero if nothing to upgrade — that's fine.
    match run_cmd(&["brew", "upgrade"], cfg) {
        RunResult::Error(e) => {
            box_line(&format!("{}✘ brew upgrade error: {e}{}", _RED.code(), RESET.code()));
        }
        RunResult::Failed(_) => {
            box_line(&format!("{}◆ brew upgrade returned non-zero (possibly nothing to upgrade){}", _YELLOW.code(), RESET.code()));
        }
        _ => {}
    }

    if cfg.maintain {
        run_cmd(&["brew", "cleanup"], cfg);
        run_cmd(&["brew", "autoremove"], cfg);
    }

    true
}