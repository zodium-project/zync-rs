/// Terminal colour & style primitives — 256-colour palette, TTY-aware.
extern crate libc;
use std::sync::OnceLock;

static IS_TTY: OnceLock<bool> = OnceLock::new();

pub fn is_tty() -> bool {
    *IS_TTY.get_or_init(|| {
        // SAFETY: isatty is a pure POSIX query — no side effects.
        unsafe { libc::isatty(1) == 1 }
    })
}

pub struct Style(pub &'static str);

impl Style {
    #[inline]
    pub fn apply(&self, text: &str) -> String {
        if is_tty() {
            format!("{}{}{}", self.0, text, RESET.0)
        } else {
            text.to_owned()
        }
    }

    #[inline]
    pub fn code(&self) -> &'static str {
        if is_tty() { self.0 } else { "" }
    }
}

// ──── Base styles ────
pub static RESET:   Style = Style("\x1b[0m");
pub static BOLD:    Style = Style("\x1b[1m");
pub static DIM:     Style = Style("\x1b[2m");

// ──── 256-colour palette ────
pub static _WHITE:   Style = Style("\x1b[38;5;255m");
pub static _GREY:    Style = Style("\x1b[38;5;240m");
pub static _RED:     Style = Style("\x1b[38;5;196m");
pub static _GREEN:   Style = Style("\x1b[38;5;82m");
pub static _YELLOW:  Style = Style("\x1b[38;5;226m");
pub static _CYAN:    Style = Style("\x1b[38;5;51m");
pub static _BLUE:    Style = Style("\x1b[38;5;39m");
pub static _MAGENTA: Style = Style("\x1b[38;5;213m");
pub static _ORANGE:  Style = Style("\x1b[38;5;208m");

// ──── Semantic status symbols ────
pub fn status_failed()  -> String { _RED.apply("  ✘") }
pub fn status_info()    -> String { _CYAN.apply("  ◈") }
pub fn status_warning() -> String { _YELLOW.apply("  ◆") }
pub fn status_debug()   -> String { _GREY.apply("  ·") }

/// Strip ANSI escape sequences from a string (for log-file output).
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() { break; }
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}