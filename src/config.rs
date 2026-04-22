/// CLI argument parsing, global configuration, and flag-conflict resolution.
use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(
    name = "zync",
    version,
    about = "Unified Atomic Update Orchestrator",
    long_about = None,
    disable_help_flag = true,
    styles = clap::builder::styling::Styles::styled()
        .header(clap::builder::styling::AnsiColor::Cyan.on_default().bold())
        .usage(clap::builder::styling::AnsiColor::White.on_default().bold())
        .literal(clap::builder::styling::AnsiColor::Cyan.on_default())
        .placeholder(clap::builder::styling::AnsiColor::White.on_default()),
)]
struct Cli {
    /// Run all detected/enabled modules (default when no module flag is given)
    #[arg(long)]
    all: bool,

    /// Update rpm-ostree system
    #[arg(long = "rpm-ostree")]
    rpm_ostree: bool,

    /// Update Flatpak apps
    #[arg(long)]
    flatpak: bool,

    /// Update Homebrew packages
    #[arg(long)]
    brew: bool,

    /// Upgrade all Distrobox containers
    #[arg(long)]
    distrobox: bool,

    /// Update Podman containers/images
    #[arg(long)]
    podman: bool,

    /// Update firmware via fwupdmgr
    #[arg(long)]
    firmware: bool,

    /// Update system via bootc
    #[arg(long)]
    bootc: bool,

    /// Update via apt-get (Debian/Ubuntu)
    #[arg(long)]
    apt: bool,

    /// Update via dnf/dnf5 (Fedora/RHEL)
    #[arg(long)]
    dnf: bool,

    /// Update via pacman/yay/paru/aura (Arch/CachyOS)
    #[arg(long)]
    pacman: bool,

    /// Update via zypper (openSUSE)
    #[arg(long)]
    zypper: bool,

    /// Update via zbox
    #[arg(long)]
    zbox: bool,

    /// Update VS Code / VSCodium extensions
    #[arg(long)]
    vscode: bool,

    /// Update Nix packages (nix profile, nix-env, home-manager, nixos-rebuild)
    #[arg(long)]
    nix: bool,

    /// Update Node.js global packages (npm, pnpm, yarn, bun)
    #[arg(long)]
    nodejs: bool,

    /// Update Python packages (pipx, uv, pip)
    #[arg(long)]
    python: bool,

    /// Update Rust toolchain and cargo-installed binaries (rustup, cargo-update)
    #[arg(long)]
    rust: bool,

    /// Roll back rpm-ostree deployment
    #[arg(long)]
    rollback: bool,

    /// Interactively manage automatic updates
    #[arg(long = "auto-updates")]
    auto_updates: bool,

    /// Remove unused packages/images after updates
    #[arg(long)]
    maintain: bool,

    /// Print commands without executing them
    #[arg(long = "dry-run")]
    dry_run: bool,

    /// Skip automatic reboot even when required
    #[arg(long = "no-reboot")]
    no_reboot: bool,

    /// Assume yes to all prompts
    #[arg(long = "non-interactive", short = 'y')]
    non_interactive: bool,

    /// Suppress all terminal output
    #[arg(long)]
    quiet: bool,

    /// Show debug messages
    #[arg(long)]
    verbose: bool,

    /// Show help
    #[arg(short = 'h', long, action = clap::ArgAction::Help)]
    help: Option<bool>,
}

/// A user-defined custom backend from config.toml.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CustomBackend {
    /// Display name shown in section header
    pub name: String,
    /// Binary to run (must be on PATH)
    pub command: String,
    /// Arguments passed after the command
    #[serde(default)]
    pub args: Vec<String>,
}

/// Raw shape of /etc/zync/config.toml (all fields optional).
#[derive(Debug, Deserialize, Default)]
struct RawConfig {
    /// Modules to enable by default (autodetect if absent)
    #[serde(default)]
    default_modules: Vec<String>,
    /// Custom backends
    #[serde(default)]
    custom: Vec<CustomBackend>,
}

/// Load /etc/zync/config.toml, silently returning default if missing/invalid.
fn load_file_config() -> RawConfig {
    let path = std::path::Path::new("/etc/zync/config.toml");
    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return RawConfig::default(),
    };
    toml::from_str(&content).unwrap_or_default()
}

/// Which update modules to run.
#[derive(Debug, Default, Clone)]
pub struct Modules {
    pub rpm_ostree: bool,
    pub flatpak:    bool,
    pub brew:       bool,
    pub distrobox:  bool,
    pub podman:     bool,
    pub firmware:   bool,
    pub bootc:      bool,
    pub apt:        bool,
    pub dnf:        bool,
    pub pacman:     bool,
    pub zypper:     bool,
    pub zbox:       bool,
    pub vscode:     bool,
    pub nix:        bool,
    pub nodejs:     bool,
    pub python:     bool,
    pub rust:       bool,
}

impl Modules {
    pub fn any(&self) -> bool {
        self.rpm_ostree || self.flatpak  || self.brew     || self.distrobox
            || self.podman  || self.firmware || self.bootc    || self.apt
            || self.dnf     || self.pacman   || self.zypper   || self.zbox
            || self.vscode  || self.nix      || self.nodejs   || self.python
            || self.rust
    }

    /// Enable all standard modules (mirrors `--all`).
    pub fn enable_all(&mut self) {
        self.rpm_ostree = true;
        self.flatpak    = true;
        self.brew       = true;
        self.distrobox  = true;
        self.podman     = true;
        self.firmware   = true;
        self.apt        = true;
        self.dnf        = true;
        self.pacman     = true;
        self.zypper     = true;
        self.zbox       = true;
        self.vscode     = true;
        self.nix        = true;
        self.nodejs     = true;
        self.python     = true;
        self.rust       = true;
        // bootc intentionally excluded from --all
    }

    /// Enable only the modules whose tools are actually present on PATH.
    /// Called when no explicit flags and no config default_modules are set.
    pub fn autodetect(&mut self) {
        use crate::runner::command_exists;
        if command_exists("rpm-ostree") { self.rpm_ostree = true; }
        if command_exists("flatpak")    { self.flatpak    = true; }
        if command_exists("brew")       { self.brew       = true; }
        if command_exists("distrobox")  { self.distrobox  = true; }
        if command_exists("podman")     { self.podman     = true; }
        if command_exists("fwupdmgr")  { self.firmware   = true; }
        if command_exists("apt-get")    { self.apt        = true; }
        if command_exists("dnf5") || command_exists("dnf") { self.dnf = true; }
        if command_exists("pacman")     { self.pacman     = true; }
        if command_exists("zypper")     { self.zypper     = true; }
        if command_exists("zbox")       { self.zbox       = true; }
        if command_exists("code") || command_exists("codium") { self.vscode = true; }
        if command_exists("nix") || command_exists("nix-env") { self.nix    = true; }
        if command_exists("npm") || command_exists("pnpm")
            || command_exists("yarn") || command_exists("bun") { self.nodejs = true; }
        if command_exists("pipx") || command_exists("uv")
            || command_exists("pip3") || command_exists("pip") { self.python = true; }
        if command_exists("rustup") || command_exists("cargo") { self.rust   = true; }
        // bootc intentionally excluded from autodetect
    }

    /// Apply mutual-exclusion rules between image-based and package-based managers.
    ///
    /// Rules:
    ///   - rpm-ostree present  →  disable dnf  (rpm-ostree owns the base OS)
    ///   - bootc present       →  disable dnf  (bootc owns the base OS)
    ///   - rpm-ostree present  →  disable bootc (they are mutually exclusive image managers)
    ///
    /// These suppressions are silently applied; the caller may warn the user.
    pub fn apply_exclusions(&mut self) -> Vec<&'static str> {
        let mut suppressed = Vec::new();
        if self.rpm_ostree {
            if self.dnf   { self.dnf   = false; suppressed.push("dnf");   }
            if self.bootc { self.bootc = false; suppressed.push("bootc"); }
        }
        if self.bootc {
            if self.dnf { self.dnf = false; suppressed.push("dnf"); }
        }
        suppressed
    }

    /// Enable modules listed by name in config.toml default_modules.
    fn enable_from_list(&mut self, list: &[String]) {
        for name in list {
            match name.as_str() {
                "rpm-ostree" | "rpm_ostree" => self.rpm_ostree = true,
                "flatpak"                   => self.flatpak    = true,
                "brew"                      => self.brew       = true,
                "distrobox"                 => self.distrobox  = true,
                "podman"                    => self.podman     = true,
                "firmware"                  => self.firmware   = true,
                "bootc"                     => self.bootc      = true,
                "apt"                       => self.apt        = true,
                "dnf"                       => self.dnf        = true,
                "pacman"                    => self.pacman     = true,
                "zypper"                    => self.zypper     = true,
                "zbox"                      => self.zbox       = true,
                "vscode"                    => self.vscode     = true,
                "nix"                       => self.nix        = true,
                "nodejs" | "node"           => self.nodejs     = true,
                "python" | "pip"            => self.python     = true,
                "rust" | "cargo"            => self.rust       = true,
                _ => {}
            }
        }
    }
}

/// The parsed, validated runtime configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub modules: Modules,

    // Custom backends from config.toml
    pub custom_backends: Vec<CustomBackend>,

    // Modules suppressed due to mutual-exclusion rules (for UI display).
    pub suppressed_modules: Vec<String>,

    // Special modes
    pub rollback:      bool,
    pub auto_updates:  bool,

    // Behaviour flags
    pub dry_run:         bool,
    pub maintain:        bool,
    pub prompt_reboot:   bool,
    pub non_interactive: bool,

    // Output flags
    pub quiet:   bool,
    pub verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            modules:            Modules::default(),
            custom_backends:    Vec::new(),
            suppressed_modules: Vec::new(),
            rollback:           false,
            auto_updates:       false,
            dry_run:            false,
            maintain:           false,
            prompt_reboot:      true,
            non_interactive:    false,
            quiet:              false,
            verbose:            false,
        }
    }
}

/// Errors that can occur during argument parsing.
#[derive(Debug)]
pub enum ParseError {
    ConflictingFlags(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ConflictingFlags(s) => write!(f, "{s}"),
        }
    }
}

/// Parse CLI args into a [`Config`].
pub fn parse_args() -> Result<Config, ParseError> {
    let cli = Cli::parse();
    let mut cfg = Config::default();

    // Load /etc/zync/config.toml
    let file_cfg = load_file_config();
    cfg.custom_backends = file_cfg.custom;

    if cli.all        { cfg.modules.enable_all(); }
    if cli.rpm_ostree { cfg.modules.rpm_ostree = true; }
    if cli.flatpak    { cfg.modules.flatpak    = true; }
    if cli.brew       { cfg.modules.brew       = true; }
    if cli.distrobox  { cfg.modules.distrobox  = true; }
    if cli.podman     { cfg.modules.podman     = true; }
    if cli.firmware   { cfg.modules.firmware   = true; }
    if cli.bootc      { cfg.modules.bootc      = true; }
    if cli.apt        { cfg.modules.apt        = true; }
    if cli.dnf        { cfg.modules.dnf        = true; }
    if cli.pacman     { cfg.modules.pacman     = true; }
    if cli.zypper     { cfg.modules.zypper     = true; }
    if cli.zbox       { cfg.modules.zbox       = true; }
    if cli.vscode     { cfg.modules.vscode     = true; }
    if cli.nix        { cfg.modules.nix        = true; }
    if cli.nodejs     { cfg.modules.nodejs     = true; }
    if cli.python     { cfg.modules.python     = true; }
    if cli.rust       { cfg.modules.rust       = true; }

    cfg.rollback        = cli.rollback;
    cfg.auto_updates    = cli.auto_updates;
    cfg.maintain        = cli.maintain;
    cfg.dry_run         = cli.dry_run;
    cfg.prompt_reboot   = !cli.no_reboot;
    cfg.non_interactive = cli.non_interactive;
    cfg.quiet           = cli.quiet;
    cfg.verbose         = cli.verbose;

    if cfg.quiet && cfg.verbose {
        eprintln!("zync: warning: --quiet and --verbose both set; --verbose ignored");
        cfg.verbose = false;
    }

    if cfg.auto_updates && cfg.modules.any() {
        return Err(ParseError::ConflictingFlags(
            "--auto-updates cannot be combined with module flags".into(),
        ));
    }
    if cfg.auto_updates && cfg.rollback {
        return Err(ParseError::ConflictingFlags(
            "--auto-updates cannot be combined with --rollback".into(),
        ));
    }

    // Default behaviour when no explicit module flags given:
    //   1. config.toml default_modules list (if set)
    //   2. autodetect from PATH
    if !cfg.modules.any() && !cfg.rollback && !cfg.auto_updates {
        if !file_cfg.default_modules.is_empty() {
            cfg.modules.enable_from_list(&file_cfg.default_modules);
        } else {
            cfg.modules.autodetect();
        }
    }

    // Apply mutual-exclusion rules (rpm-ostree/bootc vs dnf, rpm-ostree vs bootc).
    // Store suppressed names on Config so main can display them prettily after UI init.
    cfg.suppressed_modules = cfg.modules.apply_exclusions()
        .into_iter().map(|s| s.to_owned()).collect();

    Ok(cfg)
}