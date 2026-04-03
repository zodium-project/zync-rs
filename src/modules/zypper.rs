/// Module: zypper package manager (openSUSE Leap/Tumbleweed).
use crate::color::{RESET, _RED, _YELLOW};
use crate::config::Config;
use crate::runner::{command_exists, run_cmd, RunResult};
use crate::state::State;
use crate::ui::box_line;

pub fn run(cfg: &Config, state: &mut State) -> bool {
    if !command_exists("zypper") {
        box_line(&format!("{}✘ zypper not found{}", _RED.code(), RESET.code()));
        return false;
    }

    match run_cmd(&["sudo", "zypper", "refresh"], cfg) {
        RunResult::DryRun | RunResult::Success => {}
        _ => return false,
    }

    match run_cmd(&["sudo", "zypper", "update", "-y"], cfg) {
        RunResult::DryRun | RunResult::Success => {}
        _ => return false,
    }

    if cfg.maintain {
        run_cmd(&["sudo", "zypper", "clean", "--all"], cfg);
    }

    // zypper needs-rebooting exits 100 when reboot is required
    if !cfg.dry_run {
        let status = std::process::Command::new("sudo")
            .args(["zypper", "needs-rebooting"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        if let Ok(s) = status {
            if s.code() == Some(100) {
                state.reboot_required = true;
                box_line(&format!("{}◆ reboot required{}", _YELLOW.code(), RESET.code()));
            }
        }
    }

    true
}