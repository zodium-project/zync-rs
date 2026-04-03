/// Module: pacman / AUR helpers (Arch, CachyOS, Manjaro).
/// Prefers yay → paru → aura → pacman, in that order.
use crate::config::Config;
use crate::runner::{command_exists, run_cmd, RunResult};
use crate::ui::box_line;
use crate::color::{DIM, RESET, _CYAN};

pub fn run(cfg: &Config) -> bool {
    let bin = ["yay", "paru", "aura", "pacman"]
        .iter()
        .find(|&&b| command_exists(b))
        .copied();

    let bin = match bin {
        Some(b) => b,
        None => {
            box_line("✘ no pacman-compatible helper found (tried yay, paru, aura, pacman)");
            return false;
        }
    };

    box_line(&format!(
        "{d}using{r} {cy}{bin}{r}",
        d  = DIM.code(),
        cy = _CYAN.code(),
        r  = RESET.code(),
    ));

    // sudo rules:
    //   yay / paru  — run as regular user; they sudo internally when needed.
    //                 Running under sudo breaks AUR clone paths and home dir.
    //   aura        — requires sudo for repo operations; AUR handled via -Au.
    //   pacman      — always needs sudo.
    match bin {
        "yay" | "paru" => {
            // Upgrade repo + AUR packages in one shot.
            match run_cmd(&[bin, "-Syu", "--noconfirm"], cfg) {
                RunResult::DryRun | RunResult::Success => {}
                _ => return false,
            }
        }
        "aura" => {
            // Repo packages first, then AUR.
            match run_cmd(&["sudo", "aura", "-Syu", "--noconfirm"], cfg) {
                RunResult::DryRun | RunResult::Success => {}
                _ => return false,
            }
            match run_cmd(&["sudo", "aura", "-Ayu", "--noconfirm"], cfg) {
                RunResult::DryRun | RunResult::Success => {}
                _ => return false,
            }
        }
        _ => {
            // Plain pacman — repos only, no AUR.
            match run_cmd(&["sudo", "pacman", "-Syu", "--noconfirm"], cfg) {
                RunResult::DryRun | RunResult::Success => {}
                _ => return false,
            }
        }
    }

    if cfg.maintain {
        // Remove orphans only if any exist — empty stdin to `pacman -Rns -`
        // causes an error even with || true producing noise in the box.
        run_cmd(&["sudo", "sh", "-c",
            "orphans=$(pacman -Qdtq 2>/dev/null); [ -n \"$orphans\" ] && \
             echo \"$orphans\" | sudo pacman -Rns - --noconfirm || true"], cfg);
        // Clean package cache (keep last 2 versions).
        if command_exists("paccache") {
            run_cmd(&["sudo", "paccache", "-rk2"], cfg);
        }
    }

    true
}