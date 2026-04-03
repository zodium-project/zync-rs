/// Terminal UI helpers.
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

use crate::color::{
    BOLD, DIM, RESET,
    _CYAN, _BLUE, _WHITE, _GREEN, _RED, _ORANGE,
};

const W_DEFAULT: usize = 42;
static W_OVERRIDE: AtomicUsize = AtomicUsize::new(0);
/// Holds the live ProgressBar that acts as the floating box bottom while a
/// section is running.  None means no section is currently open.
static LIVE_BOTTOM: OnceLock<Mutex<Option<ProgressBar>>> = OnceLock::new();

fn w() -> usize {
    let ov = W_OVERRIDE.load(Ordering::Relaxed);
    if ov == 0 { W_DEFAULT } else { ov }
}

fn live_bottom() -> std::sync::MutexGuard<'static, Option<ProgressBar>> {
    LIVE_BOTTOM.get_or_init(|| Mutex::new(None)).lock().unwrap()
}

pub fn set_box_width(width: usize) {
    W_OVERRIDE.store(width, Ordering::Relaxed);
}

pub fn reset_box_width() {
    W_OVERRIDE.store(0, Ordering::Relaxed);
}

// ── ANSI-aware helpers ────────────────────────────────────────────────────

pub fn vlen(s: &str) -> usize {
    let mut n = 0;
    let mut esc = false;
    for c in s.chars() {
        match c {
            '\x1b'     => esc = true,
            'm' if esc => esc = false,
            _ if esc   => {}
            _          => n += 1,
        }
    }
    n
}

fn rpad(s: &str, width: usize) -> String {
    let v = vlen(s);
    if v >= width { s.to_string() } else { format!("{}{}", s, " ".repeat(width - v)) }
}

// ── box primitives ────────────────────────────────────────────────────────

fn box_top(label: &str, label_vis: usize) -> String {
    let fill = w().saturating_sub(label_vis + 4);
    format!(
        "{d}╭─{r} {label} {d}{fill}─╮{r}",
        d    = DIM.code(),
        r    = RESET.code(),
        fill = "─".repeat(fill),
    )
}

fn box_div() -> String {
    format!("{}├{}┤{}", DIM.code(), "─".repeat(w()), RESET.code())
}

fn box_bot() -> String {
    format!("{}╰{}╯{}", DIM.code(), "─".repeat(w()), RESET.code())
}

fn box_row(content: &str) -> String {
    let body = if vlen(content) > w() - 2 {
        let mut out = String::new();
        let mut count = 0;
        let mut esc = false;
        for c in content.chars() {
            match c {
                '\x1b'     => { esc = true;  out.push(c); }
                'm' if esc => { esc = false; out.push(c); }
                _ if esc   => { out.push(c); }
                _ => {
                    if count >= w() - 5 { break; }
                    out.push(c);
                    count += 1;
                }
            }
        }
        format!("{out}{d}...{r}", d = DIM.code(), r = RESET.code())
    } else {
        rpad(content, w() - 2)
    };
    format!(
        "{d}│{r} {body} {d}│{r}",
        d    = DIM.code(),
        r    = RESET.code(),
        body = body,
    )
}

// ── public API ────────────────────────────────────────────────────────────

/// `notices` are rendered as box rows between the title and the bottom border.
pub fn header(subtitle: Option<&str>, notices: &[String]) {
    let brand = format!(
        "{bold}{cy}Zy{bl}nc{r}",
        bold = BOLD.code(), cy = _CYAN.code(),
        bl   = _BLUE.code(), r  = RESET.code(),
    );
    let (label, vis) = match subtitle {
        Some(s) => (
            format!("{brand}{d}  ·  {s}{r}", d = DIM.code(), r = RESET.code()),
            4 + 5 + s.len(),
        ),
        None => (brand, 4),
    };
    println!("{}", box_top(&label, vis));
    for notice in notices {
        println!("{}", box_row(notice));
    }
    println!("{}", box_bot());
}

pub fn section(title: &str) {
    // Close any previous unclosed section before opening a new one.
    // Drop the guard before calling section_end() to avoid re-entrant lock.
    let already_open = live_bottom().is_some();
    if already_open {
        section_end();
    }

    let label = format!(
        "{cy}{bold}◈ {title}{r}",
        cy = _CYAN.code(), bold = BOLD.code(), r = RESET.code(),
    );
    println!("{}", box_top(&label, 2 + title.len()));

    // The progress bar acts as a "floating sticky footer" — its rendered line
    // is always the last thing on screen while the section runs.
    //
    // Visible layout of the animated bottom (same total width as box_bot):
    //   ╰─ ⠋ ──────────────────────────────────────╯
    //   ^^ ^ ^^   = 5 fixed visible chars (╰─·X·) + fill + ──╯ (2)
    //   so fill = w() - 5
    let spinner_fill = "─".repeat(w().saturating_sub(5));
    let animated_bot = format!(
        "{d}╰─{r} {{spinner}} {d}{fill}─╯{r}",
        d    = DIM.code(),
        r    = RESET.code(),
        fill = spinner_fill,
    );

    let pb = ProgressBar::with_draw_target(None, ProgressDrawTarget::stderr());
    pb.set_style(
        ProgressStyle::with_template(&animated_bot)
            .unwrap()
            .tick_strings(&[
                "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏",
            ]),
    );
    // enable_steady_tick drives the animation from a background thread.
    // 80 ms ≈ ~12 fps — smooth but cheap.
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    *live_bottom() = Some(pb);
}

pub fn section_end() {
    if let Some(pb) = live_bottom().take() {
        // Stop the tick thread, erase the animated line, print the plain bottom.
        pb.finish_and_clear();
        println!("{}", box_bot());
    }
}

/// Print a fully closed box with a title and content lines. No spinner.
pub fn info_box(title: &str, title_vis: usize, lines: &[&str]) {
    println!("{}", box_top(title, title_vis));
    for line in lines {
        println!("{}", box_row(line));
    }
    println!("{}", box_bot());
}

/// Print a bare prompt outside any box, return trimmed input.
pub fn bare_prompt(label: &str) -> String {
    print!(
        "  {cy}?{r} {label}: ",
        cy = _CYAN.code(),
        r  = RESET.code(),
    );
    let _ = io::stdout().flush();
    let mut ln = String::new();
    let _ = io::stdin().lock().read_line(&mut ln);
    ln.trim().to_owned()
}

pub fn box_line(content: &str) {
    if vlen(content) == 0 { return; }
    let row = box_row(content);
    if let Some(pb) = live_bottom().as_ref() {
        // pb.println() atomically: erases bottom, prints row, redraws bottom.
        pb.println(&row);
    } else {
        println!("{row}");
    }
}

pub fn confirm(prompt: &str, non_interactive: bool) -> bool {
    if non_interactive { return true; }
    let prompt_str = format!(
        "{cy}  ?{r}  {prompt} {d}[y/N]{r}  ",
        cy = _CYAN.code(), d = DIM.code(), r = RESET.code(),
    );
    let mut answer = String::new();
    if let Some(pb) = live_bottom().as_ref() {
        // suspend() erases the spinner line and redraws after the closure.
        // stdin.read_line() must live inside the suspension window so the
        // spinner thread cannot race with the prompt on stderr.
        pb.suspend(|| {
            print!("{prompt_str}");
            let _ = io::stdout().flush();
            let _ = io::stdin().lock().read_line(&mut answer);
        });
    } else {
        print!("{prompt_str}");
        let _ = io::stdout().flush();
        let _ = io::stdin().lock().read_line(&mut answer);
    }
    matches!(answer.trim(), "y" | "Y")
}

// ── summary ───────────────────────────────────────────────────────────────

pub fn print_summary(
    total:           u32,
    success_count:   u32,
    fail_count:      u32,
    reboot_required: bool,
    duration_secs:   u64,
) {
    let title = format!(
        "{cy}{bold}* Summary{r}",
        cy = _CYAN.code(), bold = BOLD.code(), r = RESET.code(),
    );
    println!("{}", box_top(&title, 9));
    println!("{}", box_div());

    let kv = |k: &str, v: &str| {
        let key = format!("{d}{k}{r}", d = DIM.code(), r = RESET.code());
        let key_padded = rpad(&key, 8);
        println!("{}", box_row(&format!("{key_padded}  {v}")));
    };

    let fail_col   = if fail_count > 0 { _RED.code() } else { _GREEN.code() };
    let reboot_col = if reboot_required { _ORANGE.code() } else { _GREEN.code() };

    kv("total",  &format!("{}{total}{}",         _WHITE.code(),  RESET.code()));
    kv("ok",     &format!("{}{success_count}{}", _GREEN.code(),  RESET.code()));
    kv("failed", &format!("{}{fail_count}{}",    fail_col,       RESET.code()));
    kv("reboot", &format!("{}{}{}",              reboot_col,
                           if reboot_required { "yes" } else { "no" }, RESET.code()));
    kv("time",   &format!("{}{duration_secs}s{}", DIM.code(),    RESET.code()));

    println!("{}", box_div());
    println!("{}", box_bot());
}