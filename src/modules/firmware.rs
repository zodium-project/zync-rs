/// Module: Firmware updates via fwupdmgr.
use crate::config::Config;
use crate::color::{RESET, _RED, _YELLOW};
use crate::runner::{command_exists, run_cmd, RunResult};
use crate::state::State;
use crate::ui::box_line;

pub fn run(cfg: &Config, state: &mut State) -> bool {
    if !command_exists("fwupdmgr") {
        box_line(&format!("{}✘ fwupdmgr not found{}", _RED.code(), RESET.code()));
        return false;
    }

    // Refresh metadata — best-effort.
    run_cmd(&["fwupdmgr", "refresh", "--force"], cfg);

    // fwupdmgr update exits 2 when a reboot is required to complete firmware update.
    let update_result = run_cmd(&["fwupdmgr", "update"], cfg);
    match &update_result {
        RunResult::Failed(s) if s.code() == Some(2) => {
            state.reboot_required = true;
        }
        RunResult::Failed(_) => {
            box_line(&format!("{}◆ fwupdmgr update returned non-zero (possibly no updates){}", _YELLOW.code(), RESET.code()));
        }
        RunResult::Error(e) => {
            box_line(&format!("{}✘ fwupdmgr error: {e}{}", _RED.code(), RESET.code()));
        }
        _ => {}
    }

    true
}