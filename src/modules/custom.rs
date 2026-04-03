/// Module: user-defined custom backends from /etc/zync/config.toml.
use crate::config::{Config, CustomBackend};
use crate::color::{RESET, _RED, _YELLOW};
use crate::runner::{command_exists, run_cmd, RunResult};
use crate::ui::box_line;

/// Run a single custom backend. Called per-backend via `run_module` in main.
pub fn run_one(cfg: &Config, backend: &CustomBackend) -> bool {
    if !command_exists(&backend.command) {
        box_line(&format!("{}✘ command '{}' not found{}", _RED.code(), backend.command, RESET.code()));
        return false;
    }

    let parts: Vec<&str> = std::iter::once(backend.command.as_str())
        .chain(backend.args.iter().map(|s| s.as_str()))
        .collect();

    match run_cmd(&parts, cfg) {
        RunResult::DryRun | RunResult::Success => true,
        RunResult::Failed(_) => {
            box_line(&format!("{}◆ '{}' returned non-zero{}", _YELLOW.code(), backend.command, RESET.code()));
            false
        }
        RunResult::Error(e) => {
            box_line(&format!("{}✘ '{}' error: {e}{}", _RED.code(), backend.command, RESET.code()));
            false
        }
    }
}