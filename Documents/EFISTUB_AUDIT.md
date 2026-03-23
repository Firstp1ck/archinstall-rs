# EFISTUB Boot Path Audit

Date: 2026-03-23

This audit focuses on why EFISTUB installs can fail to boot even though install steps complete.

## Highest-risk issues

### 1) ESP mountpoint mismatch (`/boot` hardcoded, planner allows `/efi`)

**Where**
- `src/core/services/bootloader.rs`
- `src/core/storage/planner.rs`
- `src/core/storage/mod.rs`

**What can go wrong**
- EFISTUB setup resolves the ESP block device from `findmnt ... /boot`.
- The planner/validation allows UEFI setups where ESP is mounted at `/mnt/efi` (pre-mounted), and manual mode can also mount ESP differently.
- In those setups, EFISTUB command generation still assumes `/boot`, so `BOOTSRC`, `DISK`, or `PART` can be empty/invalid and `efibootmgr --create` fails.

**Recommended fix**
- Introduce a single helper in `StoragePlan`/bootloader service that returns the effective ESP mountpoint (`/boot` or `/efi`) and source device.
- Replace all `findmnt ... /boot` calls in bootloader setup with that resolved mountpoint.
- Add integration tests for pre-mounted UEFI with ESP at `/mnt/efi` for EFISTUB.

## High-risk issues

### 2) EFISTUB non-UKI path assumes kernel/initramfs are on ESP root

**Where**
- `src/core/services/bootloader.rs` (EFISTUB non-UKI branch)

**What can go wrong**
- Loader entry uses `--loader '\\vmlinuz-linux'` and `initrd=\\initramfs-linux.img`, which only works when kernel/initramfs are physically on the ESP (typical only when ESP is mounted at `/boot`).
- If user layout is `/boot` on ext4 and ESP at `/efi`, those files are not on the ESP root and firmware cannot load them.

**Recommended fix**
- Add an explicit guard for EFISTUB non-UKI: require ESP-at-`/boot` layout, otherwise block install with actionable error.
- Or implement copy/sync logic that places kernel + initramfs on ESP in a predictable path and reference that path in `efibootmgr`.

### 3) UKI paths are globally hardcoded to `/boot/EFI/Linux/...`

**Where**
- `src/core/services/sysconfig.rs`
- `src/core/services/bootloader.rs`

**What can go wrong**
- UKI generation and fallback copy assume ESP is mounted at `/boot`.
- On `/efi` layouts, UKI files may be written/read from wrong location, causing missing `arch-linux.efi` at boot registration time.

**Recommended fix**
- Resolve and use dynamic ESP mountpoint in sysconfig + bootloader phases.
- Store UKI output path relative to ESP (`/EFI/Linux/...`) and derive absolute path from mountpoint.
- Add tests for UKI + EFISTUB with ESP mounted at `/efi`.

## Medium-risk issues

### 4) Multi-kernel selection is not propagated to EFISTUB/UKI artifacts

**Where**
- `src/core/services/system.rs`
- `src/core/services/sysconfig.rs`
- `src/core/services/bootloader.rs`

**What can go wrong**
- Installer supports selecting kernels (`linux`, `linux-lts`, etc.), but EFISTUB and UKI command generation are hardcoded to `linux` filenames (`vmlinuz-linux`, `arch-linux.efi`, `linux.preset`).
- If user selects only `linux-lts`, generated boot entry can point to non-existent files.

**Recommended fix**
- Generate per-selected-kernel entries:
  - non-UKI EFISTUB entries per installed kernel image + initramfs
  - UKI preset patching for each `mkinitcpio.d/<kernel>.preset`
  - deterministic default kernel selection for primary entry
- Add coverage for `linux-lts`-only scenarios.

### 5) LUKS cmdline derivation does not handle stacked layouts reliably

**Where**
- `src/core/services/bootloader.rs` (`boot_options_script`)

**What can go wrong**
- Script checks `cryptsetup status $(basename rootdev)`. This works for simple `/dev/mapper/cryptroot` roots, but can miss encryption when root is LVM-on-LUKS or other layered device.
- Result: missing `rd.luks.name`/`cryptdevice` in cmdline, boot drops to emergency shell.

**Recommended fix**
- Prefer deriving required crypt parameters from `StoragePlan` encryption/stacks metadata, not runtime probing of mounted root only.
- Keep runtime probing as fallback, but first-class metadata should drive cmdline generation.

## Lower-risk issues

### 6) `efibootmgr` failures are downgraded to warnings without enforcing a reliable fallback

**Where**
- `src/core/services/bootloader.rs` (EFISTUB branches)

**What can go wrong**
- If NVRAM write fails, installer continues.
- Non-UKI fallback relies on `startup.nsh`, which only helps when firmware launches UEFI shell startup script; many systems will still not boot.

**Recommended fix**
- For non-UKI EFISTUB, provide a stronger fallback strategy (or fail install) when `efibootmgr` cannot create entries.
- At minimum: make fallback behavior explicit in UI/log and add post-install verification checks.

## Suggested implementation order

1. Fix dynamic ESP mountpoint resolution and replace `/boot` assumptions (issues 1-3).
2. Add kernel-selection-aware EFISTUB/UKI generation (issue 4).
3. Make cmdline derivation storage-plan-driven for encrypted stacks (issue 5).
4. Tighten `efibootmgr` failure handling and fallback guarantees (issue 6).

## Quick verification matrix after fixes

- UEFI auto layout, ESP at `/boot`, EFISTUB non-UKI
- UEFI pre-mounted, ESP at `/mnt/efi`, EFISTUB non-UKI
- UEFI + UKI, ESP at `/boot`
- UEFI + UKI, ESP at `/efi`
- `linux-lts` only
- LUKS root (simple) and LVM-on-LUKS root

