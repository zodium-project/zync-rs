/// Module: VS Code / VSCodium extension updates.
use crate::color::{DIM, RESET, _CYAN, _GREEN, _RED};
use crate::config::Config;
use crate::runner::{command_exists, run_cmd_output};
use crate::ui::box_line;

pub fn run(cfg: &Config) -> bool {
    let bin = ["code", "codium", "code-insiders"]
        .iter()
        .find(|&&b| command_exists(b))
        .copied();

    let bin = match bin {
        Some(b) => b,
        None => {
            box_line(&format!("{}✘ no VS Code binary found (tried code, codium, code-insiders){}", _RED.code(), RESET.code()));
            return false;
        }
    };

    box_line(&format!(
        "{d}◈ updating {cy}{bin}{r} extensions",
        d  = DIM.code(),
        cy = _CYAN.code(),
        r  = RESET.code(),
    ));

    if cfg.dry_run {
        box_line(&format!("{}[dry-run]{} would update extensions", DIM.code(), RESET.code()));
        return true;
    }

    let list = match run_cmd_output(&[bin, "--list-extensions"], cfg) {
        Ok(s) => s,
        Err(e) => {
            box_line(&format!("{}✘ failed to list extensions: {e}{}", _RED.code(), RESET.code()));
            return false;
        }
    };

    let mut any_failed = false;
    for ext in list.lines().map(str::trim).filter(|s| !s.is_empty()) {
        match run_cmd_output(&[bin, "--install-extension", ext, "--force"], cfg) {
            Ok(_)  => box_line(&format!("{}↑{} {ext}", _GREEN.code(), RESET.code())),
            Err(e) => {
                box_line(&format!("{}✘ failed to update {ext}: {e}{}", _RED.code(), RESET.code()));
                any_failed = true;
            }
        }
    }

    !any_failed
}