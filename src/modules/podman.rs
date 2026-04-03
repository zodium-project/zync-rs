/// Module: Podman container / image updates.
use crate::config::Config;
use crate::color::{RESET, _RED};
use crate::runner::{command_exists, run_cmd};
use crate::ui::box_line;

pub fn run(cfg: &Config) -> bool {
    if !command_exists("podman") {
        box_line(&format!("{}✘ podman not found{}", _RED.code(), RESET.code()));
        return false;
    }

    // `podman auto-update` is best-effort.
    run_cmd(&["podman", "auto-update"], cfg);

    if cfg.maintain {
        run_cmd(&["podman", "image",     "prune", "-f"], cfg);
        run_cmd(&["podman", "container", "prune", "-f"], cfg);
        run_cmd(&["podman", "system",    "prune", "-f"], cfg);
    }

    true
}