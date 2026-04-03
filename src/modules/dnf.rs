/// Module: dnf package manager (Fedora/RHEL).
use crate::config::Config;
use crate::color::{RESET, _RED, _YELLOW};
use crate::runner::{command_exists, run_cmd, RunResult};
use crate::state::State;
use crate::ui::box_line;

pub fn run(cfg: &Config, state: &mut State) -> bool {
    // prefer dnf5 if available
    let bin = if command_exists("dnf5") { "dnf5" } else if command_exists("dnf") { "dnf" } else {
        box_line(&format!("{}✘ dnf/dnf5 not found{}", _RED.code(), RESET.code()));
        return false;
    };

    match run_cmd(&["sudo", bin, "upgrade", "-y"], cfg) {
        RunResult::DryRun | RunResult::Success => {}
        _ => return false,
    }

    if cfg.maintain {
        run_cmd(&["sudo", bin, "autoremove", "-y"], cfg);
        run_cmd(&["sudo", bin, "clean", "all"], cfg);
    }

    // needs-restarting -r exits 1 when a reboot is needed, 0 when not.
    // run_cmd_output treats non-zero as Err, so we must use Command directly.
    if !cfg.dry_run {
        let reboot_needed = std::process::Command::new("sudo")
            .args(["needs-restarting", "-r"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.code() == Some(1))
            .unwrap_or(false);
        if reboot_needed {
            state.reboot_required = true;
            box_line(&format!("{}◆ reboot required{}", _YELLOW.code(), RESET.code()));
        }
    }

    true
}