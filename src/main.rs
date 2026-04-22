/// zync — Unified Atomic Update Orchestrator
///
/// Entry point: parse args → acquire lock → run modules → summary → optional reboot.
mod color;
mod config;
mod lock;
mod logger;
mod modules;
mod runner;
mod state;
mod ui;

use std::time::Instant;

use config::parse_args;
use lock::{Lock, LockError};
use logger::{init as log_init, warning};
use runner::run_module;
use state::State;
use ui::{confirm, header, print_summary, section, section_end, box_line};

fn main() {
    // ── Parse CLI ──────────────────────────────────────────────────────────
    let cfg = match parse_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("zync: {e}");
            std::process::exit(1);
        }
    };

    // ── Init logger ────────────────────────────────────────────────────────
    log_init(&cfg);

    // ── Acquire single-instance lock ───────────────────────────────────────
    let _lock = match Lock::acquire() {
        Ok(l) => l,
        Err(LockError::AlreadyRunning) => {
            eprintln!("zync: another instance is already running");
            std::process::exit(1);
        }
        Err(LockError::CannotCreate(msg)) => {
            eprintln!("zync: {msg}");
            std::process::exit(1);
        }
    };

    // ── Print header (with any suppression notices inline) ─────────────────
    let notices: Vec<String> = cfg.suppressed_modules.iter().map(|name| {
        format!(
            "{y}◆{r} {d}{name}{r} skipped {d}— managed by image-based system{r}",
            y = color::_YELLOW.code(),
            d = color::DIM.code(),
            r = color::RESET.code(),
        )
    }).collect();
    header(if cfg.dry_run { Some("dry-run") } else { None }, &notices);

    let wall_start = Instant::now();
    let mut state = State::default();

    // ── Guard: --auto-updates (exclusive) ─────────────────────────────────
    if cfg.auto_updates {
        ui::set_box_width(68);
        state.total += 1;
        let ok = modules::auto_updates::run(&cfg);
        if ok { state.success += 1; } else { state.failed += 1; }
        ui::reset_box_width();
        finalize(&cfg, &state, wall_start.elapsed().as_secs());
        return;
    }

    // ── Guard: --rollback (exclusive) ─────────────────────────────────────
    if cfg.rollback {
        if cfg.modules.any() {
            warning("--rollback ignores all other module flags");
        }
        state.total += 1;
        let ok = modules::rollback::run(&cfg, &mut state);
        if ok { state.success += 1; } else { state.failed += 1; }
        finalize(&cfg, &state, wall_start.elapsed().as_secs());
        return;
    }

    // ── Normal module pipeline ─────────────────────────────────────────────
    if cfg.modules.rpm_ostree {
        state.total += 1;
        section("rpm-ostree");
        let start = Instant::now();
        let ok = modules::rpm_ostree::run(&cfg, &mut state);
        finish_module(ok, start.elapsed().as_secs(), &mut state);
        section_end();
    }

    if cfg.modules.apt {
        state.total += 1;
        section("apt");
        let start = Instant::now();
        let ok = modules::apt::run(&cfg, &mut state);
        finish_module(ok, start.elapsed().as_secs(), &mut state);
        section_end();
    }

    if cfg.modules.dnf {
        state.total += 1;
        section("dnf");
        let start = Instant::now();
        let ok = modules::dnf::run(&cfg, &mut state);
        finish_module(ok, start.elapsed().as_secs(), &mut state);
        section_end();
    }

    if cfg.modules.pacman {
        run_module("pacman", |c| modules::pacman::run(c), &cfg, &mut state);
    }

    if cfg.modules.flatpak {
        run_module("Flatpak", |c| modules::flatpak::run(c), &cfg, &mut state);
    }

    if cfg.modules.brew {
        run_module("Homebrew", |c| modules::brew::run(c), &cfg, &mut state);
    }

    if cfg.modules.distrobox {
        run_module("Distrobox", |c| modules::distrobox::run(c), &cfg, &mut state);
    }

    if cfg.modules.podman {
        run_module("Podman", |c| modules::podman::run(c), &cfg, &mut state);
    }

    if cfg.modules.zbox {
        run_module("zbox", |c| modules::zbox::run(c), &cfg, &mut state);
    }

    if cfg.modules.vscode {
        run_module("VS Code", |c| modules::vscode::run(c), &cfg, &mut state);
    }

    if cfg.modules.nix {
        run_module("Nix", |c| modules::nix::run(c), &cfg, &mut state);
    }

    if cfg.modules.nodejs {
        run_module("Node.js", |c| modules::nodejs::run(c), &cfg, &mut state);
    }

    if cfg.modules.python {
        run_module("Python", |c| modules::python::run(c), &cfg, &mut state);
    }

    if cfg.modules.rust {
        run_module("Rust", |c| modules::rust::run(c), &cfg, &mut state);
    }

    if cfg.modules.firmware {
        state.total += 1;
        section("Firmware");
        let start = Instant::now();
        let ok = modules::firmware::run(&cfg, &mut state);
        finish_module(ok, start.elapsed().as_secs(), &mut state);
        section_end();
    }

    if cfg.modules.zypper {
        state.total += 1;
        section("zypper");
        let start = Instant::now();
        let ok = modules::zypper::run(&cfg, &mut state);
        finish_module(ok, start.elapsed().as_secs(), &mut state);
        section_end();
    }

    if cfg.modules.bootc {
        state.total += 1;
        section("bootc");
        let start = Instant::now();
        let ok = modules::bootc::run(&cfg, &mut state);
        finish_module(ok, start.elapsed().as_secs(), &mut state);
        section_end();
    }

    // ── Custom backends ───────────────────────────────────────────────────
    for backend in cfg.custom_backends.clone() {
        run_module(&backend.name, |c| modules::custom::run_one(c, &backend), &cfg, &mut state);
    }

    finalize(&cfg, &state, wall_start.elapsed().as_secs());
}

// ── Helpers ───────────────────────────────────────────────────────────────

/// Update state counters and log the module result line.
fn finish_module(ok: bool, elapsed: u64, state: &mut State) {
    if ok {
        state.success += 1;
        box_line(&color::_GREEN.apply(&format!("✔ completed in {elapsed}s")));
    } else {
        state.failed += 1;
        box_line(&color::_RED.apply(&format!("✘ failed after {elapsed}s")));
    }
}

/// Print summary, handle reboot prompt, and exit with the right code.
fn finalize(cfg: &config::Config, state: &State, duration: u64) {
    print_summary(
        state.total,
        state.success,
        state.failed,
        state.reboot_required,
        duration,
    );

    if cfg.prompt_reboot && state.reboot_required {
        if confirm("Reboot now?", cfg.non_interactive) {
            runner::run_cmd(&["sudo", "reboot"], cfg);
        }
    } else if state.reboot_required {
        warning("Reboot skipped due to --no-reboot; please reboot manually.");
    }

    if state.failed > 0 {
        std::process::exit(2);
    }
}