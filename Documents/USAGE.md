# Usage

## Screenshots

- Main TUI: `Images/example_v0.0.1.png`
- Install flow: `Images/install_v0.0.1.png`

See [STATUS.md](STATUS.md) for current limitations and known issues.

## Basic installation flow

1. Boot the Arch Linux live environment.
2. Ensure network connectivity.
3. Get the installer (release script, clone + `boot.sh`, or `cargo run` from a dev build).
4. From the ISO TTY, `./boot.sh` is recommended: it brings up a small graphical session, prints progress, and logs to the path shown at start. It exits with an error if no GUI can be prepared.
5. Navigate sections with the keyboard, configure options, review the summary, then run the install.

### Optional: minimal GUI on a bare TTY

```bash
./boot.sh
```

Logs go to the path printed at startup.

## Navigation

### Global

| Key | Action |
|-----|--------|
| `↑/↓` or `k/j` | Navigate items (menu or content, based on focus) |
| `←/→` or `h/l` | Change value/selection in the active control |
| `Enter` | Select / confirm / run |
| `Esc` or `q` | Close popup or return focus to menu (does not quit the app) |
| `Ctrl+C` | Quit |
| `Tab` / `Shift+Tab` | Next / previous field (Locales screen) |
| `:` | Open command line (Locales screen) |

### Popups

| Key | Action |
|-----|--------|
| `↑/↓` or `k/j` | Move selection |
| `←/→` or `h/l` | Change option / side |
| `Enter` | Confirm / apply |
| `Esc` or `q` | Close popup |
| `Space` | Toggle checkbox/selection when supported |
| `/` | Start search filter; type to filter, `Backspace` to edit |

### Additional packages

| Key | Action |
|-----|--------|
| `↑/↓` | Move in package list |
| `j/k` | Switch between action buttons (Add / Continue) |
| `Space` | Toggle selection on the current package |
| `Backspace` / `Delete` | Remove current or all checked packages |

### Command-line mode (Locales)

| Key | Action |
|-----|--------|
| `:` | Open command line while editing locales |
| `Esc` | Exit command line |
| `Enter` | Execute the current command |

| Command | Effect |
|---------|--------|
| `w` | Apply changes |
| `wq` | Apply and return to menu |
| `q` | Discard and return to menu |
| any other text | Close without action |

### Reboot prompt (after successful install)

| Key | Action |
|-----|--------|
| `Y` / `y` / `J` / `j` / `Enter` | Reboot now |
| `N` / `n` / `Esc` | Cancel reboot |

## Configuration sections (overview)

1. **Locales** — Keyboard layout, language, encoding.
2. **Mirrors & repositories** — Regions, optional repos (e.g. multilib), custom mirrors/repos.
3. **Disks** — Automatic best-effort layout with btrfs subvolume presets (flat / standard / extended) when btrfs is root; pre-mounted mode if targets are already under `/mnt` (`findmnt`); manual selection and filesystem options.
4. **Disk encryption** — Experimental LUKS on supported automatic layouts; passwords; per-layout options.
5. **Swap** — Enable/disable, sizing, custom swap.
6. **Bootloader** — systemd-boot (UEFI only; blocked on legacy BIOS — use GRUB); GRUB (UEFI/BIOS).
7. **Unified kernel images** — UKI-related options (see [STATUS.md](STATUS.md) for current support).
8. **System** — Hostname, root password, users, sudo.
9. **Experience mode** — Desktop environment, display manager, base package sets.
10. **Audio** — PulseAudio, PipeWire, ALSA-only, or none.
11. **Kernels** — e.g. `linux`, `linux-lts`, `linux-hardened`, `linux-zen`.
12. **Network** — NetworkManager; copy ISO network; manual options. KDE/GNOME paths may prompt for NetworkManager.
13. **Additional packages** — Extra packages and AUR helper options where applicable.
14. **Timezone & time sync** — Timezone and NTP-style sync options.
