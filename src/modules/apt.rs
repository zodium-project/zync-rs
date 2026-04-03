/// Module: apt package manager (Debian/Ubuntu).
/// Uses apt-get (stable scripting interface) rather than apt (UI-only, unstable output).
use crate::config::Config;
use crate::color::{RESET, _RED, _YELLOW};
use crate::runner::{command_exists, run_cmd, RunResult};
use crate::state::State;
use crate::ui::box_line;

pub fn run(cfg: &Config, state: &mut State) -> bool {
    if !command_exists("apt-get") {
        box_line(&format!("{}✘ apt-get not found{}", _RED.code(), RESET.code()));
        return false;
    }

    match run_cmd(&["sudo", "apt-get", "update"], cfg) {
        RunResult::DryRun | RunResult::Success => {}
        _ => return false,
    }

    match run_cmd(&["sudo", "apt-get", "upgrade", "-y"], cfg) {
        RunResult::DryRun | RunResult::Success => {}
        _ => return false,
    }

    if cfg.maintain {
        run_cmd(&["sudo", "apt-get", "autoremove", "-y"], cfg);
        run_cmd(&["sudo", "apt-get", "clean"], cfg);
    }

    // /var/run/reboot-required is written by update-notifier-common after
    // certain package installs (e.g. kernel, libc).
    if !cfg.dry_run && std::path::Path::new("/var/run/reboot-required").exists() {
        state.reboot_required = true;
        box_line(&format!("{}◆ reboot required{}", _YELLOW.code(), RESET.code()));
    }

    true
}