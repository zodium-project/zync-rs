/// Module: Node.js toolchain updates — npm, pnpm, yarn, bun.
use crate::config::Config;
use crate::color::{DIM, RESET, _CYAN, _RED, _YELLOW};
use crate::runner::{command_exists, run_cmd, RunResult};
use crate::ui::box_line;

pub fn run(cfg: &Config) -> bool {
    let has_npm  = command_exists("npm");
    let has_pnpm = command_exists("pnpm");
    let has_yarn = command_exists("yarn");
    let has_bun  = command_exists("bun");

    if !has_npm && !has_pnpm && !has_yarn && !has_bun {
        box_line(&format!(
            "{}✘ no Node.js package manager found (tried npm, pnpm, yarn, bun){}",
            _RED.code(), RESET.code()
        ));
        return false;
    }

    let mut any_failed = false;

    // ── npm ────────────────────────────────────────────────────────────────
    if has_npm {
        box_line(&format!(
            "{d}◈ updating {cy}npm{r} itself",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));
        match run_cmd(&["npm", "install", "--global", "npm@latest"], cfg) {
            RunResult::DryRun | RunResult::Success => {}
            RunResult::Failed(_) => {
                box_line(&format!(
                    "{}◆ npm self-update returned non-zero{}",
                    _YELLOW.code(), RESET.code()
                ));
            }
            RunResult::Error(e) => {
                box_line(&format!("{}✘ npm self-update error: {e}{}", _RED.code(), RESET.code()));
                any_failed = true;
            }
        }

        box_line(&format!(
            "{d}◈ upgrading {cy}npm{r} global packages",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));
        match run_cmd(&["npm", "update", "--global"], cfg) {
            RunResult::DryRun | RunResult::Success => {}
            RunResult::Failed(_) => {
                box_line(&format!(
                    "{}◆ npm update --global returned non-zero{}",
                    _YELLOW.code(), RESET.code()
                ));
            }
            RunResult::Error(e) => {
                box_line(&format!(
                    "{}✘ npm update --global error: {e}{}",
                    _RED.code(), RESET.code()
                ));
                any_failed = true;
            }
        }
    }

    // ── pnpm ───────────────────────────────────────────────────────────────
    if has_pnpm {
        box_line(&format!(
            "{d}◈ updating {cy}pnpm{r} itself",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));
        match run_cmd(&["pnpm", "self-update"], cfg) {
            RunResult::DryRun | RunResult::Success => {}
            RunResult::Failed(_) => {
                box_line(&format!(
                    "{}◆ pnpm self-update returned non-zero{}",
                    _YELLOW.code(), RESET.code()
                ));
            }
            RunResult::Error(e) => {
                box_line(&format!("{}✘ pnpm self-update error: {e}{}", _RED.code(), RESET.code()));
            }
        }

        box_line(&format!(
            "{d}◈ upgrading {cy}pnpm{r} global packages",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));
        match run_cmd(&["pnpm", "update", "--global"], cfg) {
            RunResult::DryRun | RunResult::Success => {}
            RunResult::Failed(_) => {
                box_line(&format!(
                    "{}◆ pnpm update --global returned non-zero{}",
                    _YELLOW.code(), RESET.code()
                ));
            }
            RunResult::Error(e) => {
                box_line(&format!(
                    "{}✘ pnpm update --global error: {e}{}",
                    _RED.code(), RESET.code()
                ));
                any_failed = true;
            }
        }
    }

    // ── yarn ───────────────────────────────────────────────────────────────
    if has_yarn {
        box_line(&format!(
            "{d}◈ upgrading {cy}yarn{r} global packages",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));
        // `yarn global upgrade` works for Yarn 1 (Classic).
        // Yarn Berry (2+) doesn't have global packages by design — skip gracefully.
        match run_cmd(&["yarn", "global", "upgrade"], cfg) {
            RunResult::DryRun | RunResult::Success => {}
            RunResult::Failed(_) => {
                box_line(&format!(
                    "{}◆ yarn global upgrade returned non-zero (Yarn Berry has no globals){}",
                    _YELLOW.code(), RESET.code()
                ));
            }
            RunResult::Error(e) => {
                box_line(&format!(
                    "{}✘ yarn global upgrade error: {e}{}",
                    _RED.code(), RESET.code()
                ));
                any_failed = true;
            }
        }
    }

    // ── bun ────────────────────────────────────────────────────────────────
    if has_bun {
        box_line(&format!(
            "{d}◈ updating {cy}bun{r} itself",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));
        match run_cmd(&["bun", "upgrade"], cfg) {
            RunResult::DryRun | RunResult::Success => {}
            RunResult::Failed(_) => {
                box_line(&format!(
                    "{}◆ bun upgrade returned non-zero{}",
                    _YELLOW.code(), RESET.code()
                ));
            }
            RunResult::Error(e) => {
                box_line(&format!("{}✘ bun upgrade error: {e}{}", _RED.code(), RESET.code()));
                any_failed = true;
            }
        }

        box_line(&format!(
            "{d}◈ upgrading {cy}bun{r} global packages",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));
        match run_cmd(&["bun", "update", "--global"], cfg) {
            RunResult::DryRun | RunResult::Success => {}
            RunResult::Failed(_) => {
                box_line(&format!(
                    "{}◆ bun update --global returned non-zero{}",
                    _YELLOW.code(), RESET.code()
                ));
            }
            RunResult::Error(e) => {
                box_line(&format!(
                    "{}✘ bun update --global error: {e}{}",
                    _RED.code(), RESET.code()
                ));
                any_failed = true;
            }
        }
    }

    !any_failed
}