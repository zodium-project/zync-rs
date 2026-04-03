/// Module: bootc image upgrade (rpm-ostree successor).
use crate::config::Config;
use crate::color::{RESET, _RED, _YELLOW};
use crate::runner::{command_exists, run_cmd, run_cmd_output, RunResult};
use crate::state::State;
use crate::ui::box_line;
use crate::modules::rpm_ostree::json_val;

pub fn run(cfg: &Config, state: &mut State) -> bool {
    if !command_exists("bootc") {
        box_line(&format!("{}✘ bootc not found{}", _RED.code(), RESET.code()));
        return false;
    }

    box_line("Checking for bootc image upgrade...");

    match run_cmd(&["sudo", "bootc", "upgrade"], cfg) {
        RunResult::DryRun | RunResult::Success => {}
        RunResult::Failed(s) => {
            box_line(&format!("{}✘ bootc upgrade failed (exit {:?}){}", _RED.code(), s.code(), RESET.code()));
            return false;
        }
        RunResult::Error(e) => {
            box_line(&format!("{}✘ bootc upgrade error: {e}{}", _RED.code(), RESET.code()));
            return false;
        }
    }

    if cfg.dry_run {
        return true;
    }

    let json_str = match run_cmd_output(&["sudo", "bootc", "status", "--json"], cfg) {
        Ok(s) => s,
        Err(e) => {
            box_line(&format!("{}✘ failed to query bootc status: {e}{}", _RED.code(), RESET.code()));
            return false;
        }
    };

    if let Some(val) = json_val(&json_str) {
        let staged_version = val["status"]["staged"]["image"]["version"]
            .as_str().unwrap_or("").to_owned();

        if !staged_version.is_empty() {
            let booted_version = val["status"]["booted"]["image"]["version"]
                .as_str().unwrap_or("unknown").to_owned();
            state.reboot_required = true;
            box_line(&format!("Upgrade staged: {booted_version} → {staged_version}"));
            box_line(&format!("{}◆ Reboot required to apply new image{}", _YELLOW.code(), RESET.code()));
        } else {
            box_line("Already up to date; no new image staged");
        }
    }

    true
}