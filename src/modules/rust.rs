/// Module: Rust toolchain updates — rustup, cargo-install-update, cargo itself.
use crate::config::Config;
use crate::color::{DIM, RESET, _CYAN, _RED, _YELLOW};
use crate::runner::{command_exists, run_cmd, RunResult};
use crate::ui::box_line;

pub fn run(cfg: &Config) -> bool {
    if !command_exists("rustup") && !command_exists("cargo") {
        box_line(&format!(
            "{}✘ no Rust toolchain found (tried rustup, cargo){}",
            _RED.code(), RESET.code()
        ));
        return false;
    }

    let mut any_failed = false;

    // ── rustup — toolchain update ──────────────────────────────────────────
    if command_exists("rustup") {
        box_line(&format!(
            "{d}◈ updating {cy}rustup{r} toolchains",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));
        match run_cmd(&["rustup", "update"], cfg) {
            RunResult::DryRun | RunResult::Success => {}
            RunResult::Failed(_) => {
                box_line(&format!(
                    "{}◆ rustup update returned non-zero{}",
                    _YELLOW.code(), RESET.code()
                ));
                any_failed = true;
            }
            RunResult::Error(e) => {
                box_line(&format!("{}✘ rustup update error: {e}{}", _RED.code(), RESET.code()));
                any_failed = true;
            }
        }

        // Update rustup itself.
        match run_cmd(&["rustup", "self", "update"], cfg) {
            RunResult::DryRun | RunResult::Success => {}
            RunResult::Failed(_) => {
                box_line(&format!(
                    "{}◆ rustup self update returned non-zero{}",
                    _YELLOW.code(), RESET.code()
                ));
            }
            RunResult::Error(e) => {
                box_line(&format!(
                    "{}✘ rustup self update error: {e}{}",
                    _RED.code(), RESET.code()
                ));
            }
        }
    }

    // ── cargo-update — upgrade installed binaries ──────────────────────────
    // `cargo install-update` is provided by the `cargo-update` crate.
    if command_exists("cargo") {
        // Check if cargo-install-update subcommand is available.
        let has_install_update = {
            let result = std::process::Command::new("cargo")
                .args(["install-update", "--version"])
                .output();
            matches!(result, Ok(o) if o.status.success())
        };

        if has_install_update {
            box_line(&format!(
                "{d}◈ upgrading {cy}cargo{r} installed binaries",
                d  = DIM.code(),
                cy = _CYAN.code(),
                r  = RESET.code(),
            ));
            match run_cmd(&["cargo", "install-update", "--all"], cfg) {
                RunResult::DryRun | RunResult::Success => {}
                RunResult::Failed(_) => {
                    box_line(&format!(
                        "{}◆ cargo install-update --all returned non-zero{}",
                        _YELLOW.code(), RESET.code()
                    ));
                }
                RunResult::Error(e) => {
                    box_line(&format!(
                        "{}✘ cargo install-update error: {e}{}",
                        _RED.code(), RESET.code()
                    ));
                    any_failed = true;
                }
            }
        } else {
            box_line(&format!(
                "{d}◈ {cy}cargo-update{r}{d} not installed — skipping binary upgrades{r}",
                d = DIM.code(),
                cy = _CYAN.code(),
                r  = RESET.code(),
            ));
            box_line(&format!(
                "{d}  install it with: cargo install cargo-update{r}",
                d = DIM.code(),
                r = RESET.code(),
            ));
        }
    }

    !any_failed
}