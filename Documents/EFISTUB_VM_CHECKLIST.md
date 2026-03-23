# EFISTUB on VM Checklist

## Firmware and VM Settings

- [ ] VM is configured for **UEFI** (not BIOS/Legacy/CSM).
- [ ] UEFI variable store (NVRAM/OVMF vars) is **persistent** across reboots.
- [ ] Secure Boot state is known and matches your boot artifacts (off, or signed artifacts).
- [ ] Virtual disk/controller layout has not changed since boot entry creation.

## In-Guest UEFI Preconditions

- [ ] `/sys/firmware/efi` exists.
- [ ] `efivarfs` is mounted (`mount | grep efivars`).
- [ ] `efibootmgr` runs without errors.
- [ ] ESP is mounted at expected path (for this project, usually `/boot` in chroot context).

## ESP File Layout Validation

- [ ] `EFI/Linux/` exists on the ESP.
- [ ] `EFI/Linux/vmlinuz-<kernel>` exists.
- [ ] `EFI/Linux/initramfs-<kernel>.img` exists.
- [ ] `EFI/Linux/initramfs-<kernel>-fallback.img` exists.
- [ ] Microcode image exists in `EFI/Linux/` when used (`intel-ucode.img` or `amd-ucode.img`).
- [ ] Fallback path exists: `EFI/BOOT/BOOTX64.EFI` (recommended).
- [ ] `startup.nsh` exists on ESP for firmware fallback behavior.

## NVRAM Entry Validation

- [ ] `efibootmgr --verbose` shows an `Arch Linux` entry.
- [ ] The `File(...)` path in that entry points to `\EFI\Linux\...` (not stale device/path).
- [ ] `BootOrder` places the intended Arch entry before non-working entries.
- [ ] `BootNext` is set to the intended Arch entry for immediate reboot testing.

## Reboot Behavior Checks

- [ ] First reboot from installer/chroot succeeds without dropping to firmware shell.
- [ ] Second reboot also succeeds (confirms persistent NVRAM behavior).
- [ ] Entry still exists after power-off + power-on (not just soft reboot).

## If Boot Fails with "Not found"

- [ ] Re-check VM disk/controller mapping; recreate boot entries if PCI path changed.
- [ ] Re-copy kernel/initramfs artifacts into `EFI/Linux/`.
- [ ] Recreate NVRAM entries with `efibootmgr --create` using current ESP disk/partition.
- [ ] Reapply `BootOrder` and `BootNext` after recreating entries.
- [ ] Use fallback (`BOOTX64.EFI` or `startup.nsh`) to confirm firmware can read ESP.

## Post-Install Maintenance

- [ ] Confirm pacman hook refreshes `EFI/Linux` artifacts after kernel upgrades.
- [ ] After each kernel update, verify `efibootmgr --verbose` still points to valid files.
- [ ] Keep at least one known-good fallback path (`BOOTX64.EFI` or startup script).
