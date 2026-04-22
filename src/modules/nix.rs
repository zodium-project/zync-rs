/// Module: Nix package manager updates (nix-env, Home Manager, NixOS).
use crate::config::Config;
use crate::color::{DIM, RESET, _CYAN, _RED, _YELLOW};
use crate::runner::{command_exists, run_cmd, RunResult};
use crate::ui::box_line;

pub fn run(cfg: &Config) -> bool {
    if !command_exists("nix") && !command_exists("nix-env") {
        box_line(&format!("{}✘ nix not found{}", _RED.code(), RESET.code()));
        return false;
    }

    let use_modern = command_exists("nix");

    if use_modern {
        box_line(&format!(
            "{d}◈ using {cy}nix{r} (modern CLI)",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));

        // Flake update — best-effort only, not all systems use flakes.
        // Errors and non-zero exits are completely expected here.
        match run_cmd(&["nix", "flake", "update"], cfg) {
            RunResult::DryRun | RunResult::Success => {
                box_line(&format!(
                    "{}◆ flake inputs updated{}",
                    _CYAN.code(), RESET.code()
                ));
            }
            _ => {
                // Not a flake system or no flake.lock present — silently skip.
            }
        }

        // nix-channel --update — best-effort, many modern nix setups have no
        // channels configured at all. A failure here is not an error.
        if command_exists("nix-channel") {
            match run_cmd(&["nix-channel", "--update"], cfg) {
                RunResult::DryRun | RunResult::Success => {}
                _ => {
                    box_line(&format!(
                        "{}◆ no channels configured — skipping channel update{}",
                        _YELLOW.code(), RESET.code()
                    ));
                }
            }
        }

        // nix profile upgrade — the real user-profile upgrade.
        // Try modern first, fall back to nix-env on failure.
        let profile_ok = match run_cmd(&["nix", "profile", "upgrade", "--all"], cfg) {
            RunResult::DryRun | RunResult::Success => true,
            _ => {
                // Fall back to nix-env --upgrade.
                box_line(&format!(
                    "{d}◈ nix profile upgrade failed — retrying with {cy}nix-env{r}",
                    d  = DIM.code(),
                    cy = _CYAN.code(),
                    r  = RESET.code(),
                ));
                match run_cmd(&["nix-env", "--upgrade"], cfg) {
                    RunResult::DryRun | RunResult::Success => true,
                    RunResult::Failed(_) => {
                        box_line(&format!(
                            "{}◆ nix-env --upgrade returned non-zero (possibly nothing to upgrade){}",
                            _YELLOW.code(), RESET.code()
                        ));
                        true // nothing to upgrade is not a failure
                    }
                    RunResult::Error(e) => {
                        box_line(&format!(
                            "{}✘ nix-env error: {e}{}",
                            _RED.code(), RESET.code()
                        ));
                        false
                    }
                }
            }
        };

        if !profile_ok {
            return false;
        }
    } else {
        // Legacy path: only nix-env available.
        box_line(&format!(
            "{d}◈ using {cy}nix-env{r} (legacy)",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));

        if command_exists("nix-channel") {
            match run_cmd(&["nix-channel", "--update"], cfg) {
                RunResult::DryRun | RunResult::Success => {}
                _ => {
                    box_line(&format!(
                        "{}◆ no channels configured — skipping channel update{}",
                        _YELLOW.code(), RESET.code()
                    ));
                }
            }
        }

        match run_cmd(&["nix-env", "--upgrade"], cfg) {
            RunResult::DryRun | RunResult::Success => {}
            RunResult::Failed(_) => {
                box_line(&format!(
                    "{}◆ nix-env --upgrade returned non-zero (possibly nothing to upgrade){}",
                    _YELLOW.code(), RESET.code()
                ));
            }
            RunResult::Error(e) => {
                box_line(&format!("{}✘ nix-env error: {e}{}", _RED.code(), RESET.code()));
                return false;
            }
        }
    }

    // Home Manager — best-effort, soft failure only.
    if command_exists("home-manager") {
        box_line(&format!(
            "{d}◈ running {cy}home-manager switch{r}",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));
        match run_cmd(&["home-manager", "switch"], cfg) {
            RunResult::DryRun | RunResult::Success => {}
            RunResult::Failed(_) | RunResult::Error(_) => {
                box_line(&format!(
                    "{}◆ home-manager switch returned non-zero{}",
                    _YELLOW.code(), RESET.code()
                ));
            }
        }
    }

    // NixOS rebuild — best-effort, soft failure only.
    if command_exists("nixos-rebuild") {
        box_line(&format!(
            "{d}◈ running {cy}nixos-rebuild switch{r}",
            d  = DIM.code(),
            cy = _CYAN.code(),
            r  = RESET.code(),
        ));
        match run_cmd(&["sudo", "nixos-rebuild", "switch", "--upgrade"], cfg) {
            RunResult::DryRun | RunResult::Success => {}
            RunResult::Failed(_) | RunResult::Error(_) => {
                box_line(&format!(
                    "{}◆ nixos-rebuild switch returned non-zero{}",
                    _YELLOW.code(), RESET.code()
                ));
            }
        }
    }

    if cfg.maintain {
        run_cmd(&["nix-collect-garbage", "-d"], cfg);
    }

    true
}