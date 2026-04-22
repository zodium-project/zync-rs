/// Module: Python toolchain updates — pipx, uv, pip.
use crate::config::Config;
use crate::color::{DIM, RESET, _CYAN, _RED, _YELLOW};
use crate::runner::{command_exists, run_cmd, run_cmd_output, RunResult};
use crate::ui::box_line;

pub fn run(cfg: &Config) -> bool {
    let has_pipx = command_exists("pipx");
    let has_uv   = command_exists("uv");
    let has_pip  = command_exists("pip") || command_exists("pip3");

    if !has_pipx && !has_uv && !has_pip {
        box_line(&format!(
            "{}✘ no Python package manager found (tried pipx, uv, pip/pip3){}",
            _RED.code(), RESET.code()
        ));
        return false;
    }

    let mut any_failed = false;

    // ── uv (fastest; handles its own toolchain too) ────────────────────────
    if has_uv {
        box_line(&format!(
            "{d}◈ updating {cy}uv{r} self",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));
        // Self-update uv binary.
        match run_cmd(&["uv", "self", "update"], cfg) {
            RunResult::DryRun | RunResult::Success => {}
            RunResult::Failed(_) => {
                box_line(&format!(
                    "{}◆ uv self update returned non-zero{}",
                    _YELLOW.code(), RESET.code()
                ));
            }
            RunResult::Error(e) => {
                box_line(&format!("{}✘ uv self update error: {e}{}", _RED.code(), RESET.code()));
                any_failed = true;
            }
        }

        box_line(&format!(
            "{d}◈ upgrading {cy}uv{r} tools",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));
        // Upgrade all uv-managed tools.
        match run_cmd(&["uv", "tool", "upgrade", "--all"], cfg) {
            RunResult::DryRun | RunResult::Success => {}
            RunResult::Failed(_) => {
                box_line(&format!(
                    "{}◆ uv tool upgrade --all returned non-zero{}",
                    _YELLOW.code(), RESET.code()
                ));
            }
            RunResult::Error(e) => {
                box_line(&format!("{}✘ uv tool upgrade error: {e}{}", _RED.code(), RESET.code()));
                any_failed = true;
            }
        }
    }

    // ── pipx ──────────────────────────────────────────────────────────────
    if has_pipx {
        box_line(&format!(
            "{d}◈ upgrading {cy}pipx{r} packages",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));
        match run_cmd(&["pipx", "upgrade-all"], cfg) {
            RunResult::DryRun | RunResult::Success => {}
            RunResult::Failed(_) => {
                box_line(&format!(
                    "{}◆ pipx upgrade-all returned non-zero{}",
                    _YELLOW.code(), RESET.code()
                ));
            }
            RunResult::Error(e) => {
                box_line(&format!("{}✘ pipx upgrade-all error: {e}{}", _RED.code(), RESET.code()));
                any_failed = true;
            }
        }
    }

    // ── pip / pip3 (user packages) ─────────────────────────────────────────
    // Only upgrade user-installed packages to avoid breaking system Python.
    let pip_bin = if command_exists("pip3") { "pip3" } else { "pip" };
    if has_pip {
        box_line(&format!(
            "{d}◈ upgrading {cy}{pip_bin}{r} user packages",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));

        if cfg.dry_run {
            box_line(&format!(
                "{}[dry-run]{} would upgrade all --user pip packages",
                DIM.code(), RESET.code()
            ));
        } else {
            // List outdated user packages, then upgrade each.
            let outdated = run_cmd_output(
                &[pip_bin, "list", "--outdated", "--user", "--format=freeze"],
                cfg,
            );
            match outdated {
                Err(e) => {
                    box_line(&format!(
                        "{}✘ {pip_bin} list --outdated error: {e}{}",
                        _RED.code(), RESET.code()
                    ));
                    any_failed = true;
                }
                Ok(out) => {
                    let packages: Vec<&str> = out
                        .lines()
                        .filter_map(|l| l.split("==").next())
                        .filter(|s| !s.trim().is_empty())
                        .collect();

                    if packages.is_empty() {
                        box_line(&format!(
                            "{}◆ no outdated pip user packages{}",
                            _YELLOW.code(), RESET.code()
                        ));
                    } else {
                        let mut args = vec![pip_bin, "install", "--upgrade", "--user"];
                        args.extend_from_slice(&packages);
                        match run_cmd(&args, cfg) {
                            RunResult::DryRun | RunResult::Success => {}
                            RunResult::Failed(_) => {
                                box_line(&format!(
                                    "{}◆ pip upgrade returned non-zero{}",
                                    _YELLOW.code(), RESET.code()
                                ));
                            }
                            RunResult::Error(e) => {
                                box_line(&format!(
                                    "{}✘ pip upgrade error: {e}{}",
                                    _RED.code(), RESET.code()
                                ));
                                any_failed = true;
                            }
                        }
                    }
                }
            }
        }
    }

    !any_failed
}