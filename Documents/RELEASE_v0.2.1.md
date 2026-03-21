## archinstall-rs v0.2.1

Release date: 2026-03-21

### Highlights
- **Storage**: One validated **storage plan** drives partitioning, mounts, and fstab; **btrfs subvolume presets** (flat / standard / extended); **pre-mounted** installs when `/mnt` is already set up.
- **LUKS**: Safer, quieter crypto setup (stdin passphrases, redacted logs), initramfs hooks and kernel cmdline that match your mkinitcpio style (`systemd`/`sd-encrypt` vs classic `encrypt`), and fewer false failures when `mkinitcpio` exits non-zero on warnings only.
- **Firmware & mounts**: **systemd-boot** is blocked on BIOS/legacy with a clear hint to use **GRUB**; ESP **`vfat`** mounting is more reliable (FAT support check, fallbacks, better recovery when the live ISO kernel and modules disagree).

### New
- Pre-mounted mode with cached mount/swap probes and clearer info in the disks flow.
- Optional `btrfs_subvolume_preset` in saved config (`[disks]`); omitted configs behave like before (flat).
- Release workflow assets: tagged builds publish the binary, `SHA256SUMS`, and `install.sh` for the documented curl-to-bash install path.

### Improvements & fixes
- LUKS: correct bootloader/kernel options when the root is on btrfs subvolumes; tighter mkinitcpio `HOOKS=` detection; non-interactive `cryptsetup` and stronger `dm-crypt` module loading.
- Manual partitioning: clearer **parted** byte-range rules (length vs end), overlap / past-100% rejection; btrfs preset available in manual mode when root is btrfs.
- BIOS **GRUB** in pre-mounted setups resolves the whole-disk target from mounts and disk selection.
- Preflight can warn when the live environment’s kernel and module versions look mismatched (a common ESP mount footgun).

### Breaking changes
- None for normal TUI/config users. Internal APIs used only by tests or forks may need updates (e.g. bootloader planning now takes the storage plan).

