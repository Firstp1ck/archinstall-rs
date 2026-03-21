# Configuration file (TOML)

The installer can save and load configuration as TOML for reproducible installs, sharing presets, and automation-oriented workflows.

- **In-app**: Save and load from the TUI works today.
- **CLI**: The `--config` flag is not implemented yet; loading from the command line is planned.

Repository examples:

- [`archinstall-rs.config.toml`](../archinstall-rs.config.toml) — fuller example from development (includes disk selection from a real machine).
- [`configs/examples/`](../configs/examples/) — portable presets (no chosen disk). The Load Configuration table uses [`manifest.toml`](../configs/examples/manifest.toml) (`[[preset]]`: `file`, `country`, `language`, `desktop`, `additional`); files without an entry use `—` and a short fallback from the preset’s first comment line.
  - **Popular stacks** (roughly aligned with [Arch pkgstats](https://pkgstats.archlinux.de/) desktop rankings and recurring r/archlinux survey write-ups such as [Linuxiac’s summary](https://linuxiac.com/arch-linux-community-survey-results)): [KDE Plasma (US)](../configs/examples/popular-kde-plasma-us.toml), [GNOME (Germany)](../configs/examples/popular-gnome-de.toml), [Hyprland (US)](../configs/examples/popular-hyprland-us.toml), [Xfce4 (UK)](../configs/examples/popular-xfce4-gb.toml).
  - **Locale / region**: [Japan + KDE](../configs/examples/locale-jp-kde.toml), [Brazil + GNOME](../configs/examples/locale-br-gnome.toml), [France + GNOME](../configs/examples/locale-fr-gnome.toml).

`[mirrors].regions` entries must match a line from `reflector --list-countries` exactly (including spacing); mirror counts change when the upstream list changes, so re-copy that line from the ISO if a preset no longer loads a region.

Minimal illustrative schema (may omit fields your build defaults). For a working desktop install, include `[experience.desktop_env_packages]` for each selected desktop (see the `configs/examples` files); otherwise the saved package list for that environment may be empty after load.

```toml
additional_packages = []

[locales]
keyboard_layout = "us"
locale_language = "en_US.UTF-8"
locale_encoding = "UTF-8"

[mirrors]
regions = ["United States          US   189"]
optional_repos = ["multilib"]
custom_servers = []
custom_repos = []

[disks]
mode = "Best-effort partition layout"
# Optional: "flat" | "standard" | "extended" — btrfs subvolumes for automatic btrfs root (defaults to flat if omitted)
# btrfs_subvolume_preset = "flat"

[disk_encryption]
encryption_type = "None"

[swap]
enabled = true

[bootloader]
kind = "systemd-boot"

[system]
hostname = "archlinux"
root_password_hash = "" # optional, SHA256 hex
automatic_time_sync = true
timezone = "Europe/London"

[experience]
mode = "Desktop"
desktop_envs = ["KDE Plasma"]
# login_manager = "sddm"
# login_manager_user_set = false

[audio]
kind = "pipewire"

[kernels]
selected = ["linux", "linux-lts"]

[network]
mode = "NetworkManager"

[unified_kernel_images]
enabled = false

# Optional: define users
[[users]]
username = "myuser"
password_hash = "..." # SHA256 hex
is_sudo = true

# Optional: additional packages
#[[additional_packages]]
#repo = "extra"
#name = "firefox"
#version = ""
#description = "Web browser"
```

Field names and sections follow the types in `src/app/config/types.rs` and I/O in `src/app/config/io.rs`.

## Loading example presets in the TUI

When you select **Load Configuration** in the TUI, a popup lists the current `archinstall-rs.config.toml` plus all bundled example presets. Select one and press Enter to apply it.

The installer discovers example `.toml` files using these directories (first match wins):

| Source | Path |
|--------|------|
| `ARCHINSTALL_RS_EXAMPLES` env var | directory of `.toml` files |
| `ARCHINSTALL_RS_REPO` env var | `{value}/configs/examples` |
| Next to binary | `{parent(current_exe)}/configs/examples` |
| Walk up from cwd | ancestor `configs/examples` directory |

When installed via `install.sh`, the script extracts the bundled `config-examples.tar.gz` next to the binary so presets are discovered automatically. When running from a cloned repository (`cargo run` or the binary anywhere inside the repo tree), the cwd ancestor walk finds `configs/examples` at the repo root.
