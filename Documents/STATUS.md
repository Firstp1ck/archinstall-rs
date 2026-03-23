# Status and known issues

The installer is still under active development. Treat destructive operations (partitioning, encryption) with care and keep backups.

## What works today

- 64-bit x86_64 systems
- UEFI and BIOS
- Best-effort partitioning and simple manual partitioning
- Bootloaders: systemd-boot, GRUB, EFISTUB (UEFI), and Limine (UEFI/BIOS)
- Network: NetworkManager and “copy ISO network” (systemd-networkd / resolved on target)
- Experience mode: “Desktop environment” is the primary path (NetworkManager-related constraints)
- LUKS on automatic / best-effort btrfs layouts (still experimental)
- Unified kernel images (UKI): mkinitcpio preset, `/etc/kernel/cmdline`, and bootloader paths for systemd-boot, EFISTUB, and Limine when UKI is enabled (not GRUB). Extra kernels beyond `linux` still use a single default UKI/boot entry set.
- Not yet: custom servers/repos in the sense advertised as complete, Secure Boot signing

## Known issue: log view at end of install

Installation can finish successfully while the log view does not fully complete. If the TUI appears stuck after a successful install, press **Ctrl+C** to exit, then reboot.
