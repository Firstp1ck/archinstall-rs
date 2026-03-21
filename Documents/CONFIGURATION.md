# Configuration file (TOML)

The installer can save and load configuration as TOML for reproducible installs, sharing presets, and automation-oriented workflows.

- **In-app**: Save and load from the TUI works today.
- **CLI**: The `--config` flag is not implemented yet; loading from the command line is planned.

Repository examples:

- [`archinstall-rs.config.toml`](../archinstall-rs.config.toml) — fuller example from development
- Minimal illustrative schema (may omit fields your build defaults):

```toml
[users]

users = []
additional_packages = []

[locales]
keyboard_layout = "us"
locale_language = "en_US.UTF-8"
locale_encoding = "UTF-8"

[mirrors]
regions = ["United States"]
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
