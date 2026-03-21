## archinstall-rs v0.2.0

Release date: 2025-10-14

### Highlights
- Optional LUKS encryption for the root partition
- Info popup rendering supports multi-line messages (line breaks are preserved), improving readability of longer messages.

### New
- Optional LUKS encryption for the root partition; opens as `cryptroot` when enabled.
- Info popup rendering supports multi-line messages (line breaks are preserved), improving readability of longer messages.

### Improvements
- Safer bootloader setup flows:
  - Systemd-boot: installs via `bootctl`, writes `loader.conf`, and creates standard `arch.conf` and fallback entries
  - GRUB: supports both UEFI and BIOS installs
- More robust handling of device names such as NVMe (`/dev/nvme0n1pX`) during mounting.
- Clearer live logging throughout install sections and better debug messages.

### Breaking changes
- None.

### Known limitations
- Advanced storage layouts (LVM/RAID, complex Btrfs subvolumes) are not yet implemented in the automatic partitioner.

### Thanks
Thanks to everyone testing early builds and reporting issues—your feedback directly shaped this release.


