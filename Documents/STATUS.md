# Status and known issues

The installer is still under active development. Treat destructive operations (partitioning, encryption) with care and keep backups.

## What works today

- 64-bit x86_64 systems
- UEFI and BIOS
- Best-effort partitioning and simple manual partitioning
- Bootloaders: GRUB and systemd-boot only
- Network: NetworkManager and “copy ISO network” (systemd-networkd / resolved on target)
- Experience mode: “Desktop environment” is the primary path (NetworkManager-related constraints)
- LUKS on automatic / best-effort btrfs layouts (still experimental)
- Not yet: custom servers/repos in the sense advertised as complete, unified kernel images / Secure Boot

## Known issue: log view at end of install

Installation can finish successfully while the log view does not fully complete. If the TUI appears stuck after a successful install, press **Ctrl+C** to exit, then reboot.
