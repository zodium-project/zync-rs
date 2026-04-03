# zync

**Unified Atomic Update Orchestrator** Tuned for Fedora Atomic, and bootc-based systems.

Handles system upgrades, Flatpak apps, Homebrew packages, Distrobox containers, Podman images, firmware, and rollbacks — all from one command.

---

## Features

- **Rollback** rpm-ostree deployments interactively
- **Auto-update management** — enable/edit/disable systemd timers for unattended updates
- **Reboot detection** — prompts to reboot when a staged deployment or firmware update requires it
- **musl-compatible** — builds as a fully static binary

---

## Installation

### Build from source
```bash
# Standard (glibc) build
cargo build --release

# Fully static musl build
cargo build --release --target x86_64-unknown-linux-musl

sudo install -Dm755 target/release/zync /usr/bin/zync
```

---

## Usage
```
zync [OPTIONS]
```

### Module options

| Flag | Description |
|---|---|
| `--all` | Run all standard modules |
| `--rpm-ostree` | Update rpm-ostree system |
| `--flatpak` | Update Flatpak apps |
| `--brew` | Update Homebrew packages |
| `--distrobox` | Upgrade all Distrobox containers |
| `--podman` | Update Podman containers/images |
| `--firmware` | Update firmware via fwupdmgr |
| `--bootc` | Update via bootc (rpm-ostree successor) |

### Special

| Flag | Description |
|---|---|
| `--rollback` | Roll back rpm-ostree deployment |
| `--auto-updates` | Interactively manage automatic update timers |

### Modes

| Flag | Description |
|---|---|
| `--maintain` | Remove unused packages/images after updates |
| `--dry-run` | Print commands without executing |
| `--no-reboot` | Skip reboot prompt even when required |
| `--non-interactive` / `-y` | Assume yes to all prompts (CI/automation) |

### Output

| Flag | Description |
|---|---|
| `--quiet` | Suppress terminal output |
| `--verbose` | Show debug messages |
| `--version` | Print version |
| `--help` / `-h` | Show help |

---

## Examples
```bash
# Upgrade everything
zync --all

# System + firmware, skip reboot prompt
zync --rpm-ostree --firmware --no-reboot

# Dry-run without making any changes
zync --all --dry-run

# Roll back the last rpm-ostree deployment
zync --rollback

# Unattended CI update
zync --all --non-interactive --no-reboot

# Enable/configure automatic weekly updates
zync --auto-updates
```

---

## Log file

Logs are written (ANSI-stripped, timestamped) to:
```
$XDG_STATE_HOME/zync.log
# or if unset:
~/.local/state/zync.log
```
---
## Implemented Backends
```
Pacman Managers ;
Arch Linxux - Pacman , Aura , Yay , Paru
Fedora      - Dnf , Bootc , Rpm-ostree
Opensuse    - zypper
ubuntu      - apt-get/apt
debian      - apt-get/apt

Others ;
Vscode      - extensions updating (all derivatives of vscode also work)
Flatpak     - updates & cleanup
brew        - updates & cleanup
podman      - updates & cleanup
distrobox   - updates
zbox        - updates & cleanup
firmwares   - updates & upgrades
```
---

## Exit codes

| Code | Meaning |
|---|---|
| `0` | All modules succeeded |
| `1` | Argument/lock error |
| `2` | One or more modules failed |