# Boot and Kernel Delivery Implementation Plan

## Goal

Complete the bootloader matrix (EFISTUB, Limine) and a modern kernel boot pipeline, including optional Unified Kernel Images (UKI) integrated with mkinitcpio and the chosen UEFI boot path (systemd-boot / EFISTUB / Limine).

## Status

**Phases 1–3 and Phase 4 (except multi-kernel) are implemented in tree.** EFISTUB, Limine, and UKI (`BootloaderService::uki_requested`, `SysConfigService`, `SystemService::build_pacstrap_plan`) generate real install commands. Remaining follow-up: **multi-kernel** bootloader entries and UKI presets when `selected_kernels` contains more than `linux`. **Secure Boot signing** remains P1/P2 (see [TODO_SUMMARY.md](../../TODO_SUMMARY.md) Workstream 2).

## Context (at plan inception)

Workstream 1 ([STORAGE_LAYOUT_PLAN.md](STORAGE_LAYOUT_PLAN.md)) delivered a normalized `StoragePlan`, LUKS-aware cmdline logic in systemd-boot and GRUB, and reliable mount/fstab behavior. This plan addressed the next P0 layer because:

- EFISTUB and Limine were TODO placeholders in `BootloaderService::build_plan()`.
- UKI was togglable in the TUI but produced no chroot commands.

## Dependencies and Unblocks

| Depends on | Unblocks |
|------------|----------|
| Stable `StoragePlan` with ESP at `/mnt/boot` on UEFI (done) | Secure Boot signing pipeline (future P1/P2) |
| LUKS hook alignment in `SysConfigService` + `boot_options_script` pattern (done) | All four bootloaders producing consistent encrypted-root cmdlines |
| `efibootmgr` in pacstrap for all UEFI installs (done, `system.rs`) | EFISTUB and Limine UEFI registration |
| `limine` in pacstrap when index 3 (done, `system.rs`) | Limine install path |

**Related backlog:** [TODO_SUMMARY.md](../../TODO_SUMMARY.md) — Workstream 2 (core items done; multi-kernel + Secure Boot follow-ups).

## Architecture (implemented)

### Data flow

```
AppState (bootloader_index, uki_enabled, selected_kernels, …)
   │
   ├─► SystemService::build_pacstrap_plan()
   │     → efibootmgr (UEFI), grub / limine, systemd-ukify when uki_requested
   ├─► SysConfigService::build_plan()
   │     → locale, hooks…; if UKI: /etc/kernel/cmdline, linux.preset, /boot/EFI/Linux
   │     → mkinitcpio -P when encrypted OR uki_requested
   └─► BootloaderService::build_plan(state, device, storage_plan)
         ├─► 0 systemd-boot: bootctl, loader.conf, entries (linux+initrd+options or efi UKI paths)
         ├─► 1 GRUB: grub-install, GRUB_CMDLINE_LINUX when encrypted, grub-mkconfig
         ├─► 2 EFISTUB: efibootmgr (vmlinuz+unicode or UKI .efi paths)
         └─► 3 Limine: limine.conf, UEFI: BOOTX64.EFI + hook + efibootmgr; BIOS: limine-bios.sys + bios-install

uki_requested ≡ uki_enabled && bootloader_index != 1   (BootloaderService::uki_requested)
```

### Install section ordering (`flow.rs`, `build_install_sections`)

```
Locales → Mirrors → Pre-cleanup → Partitioning → Volume stack → Mounting
 → System pre-install → Pacstrap → fstab → System configuration (sysconfig)
 → Network → Bootloader setup → User setup
```

UKI: `/etc/kernel/cmdline` and `linux.preset` are configured inside **sysconfig** before `mkinitcpio -P`, still before the bootloader section.

### Bootloader index mapping

| Index | ID | `BootloaderService` status | Packages in pacstrap (`system.rs`) | Notes |
|------|-----|---------------------------|------------------------------------|--------|
| 0 | systemd-boot | Complete | *(implicit systemd)* | UEFI-only validation in `flow.rs` |
| 1 | GRUB | Complete | `grub` | UEFI + BIOS |
| 2 | EFISTUB | Complete | *(efibootmgr for all UEFI)* | UEFI-only validation in `flow.rs` |
| 3 | Limine | Complete | `limine` | UEFI + BIOS; pre-mounted BIOS needs whole disk (with GRUB) |

### Shared kernel cmdline (`boot_options_script`)

`BootloaderService::boot_options_script(encrypted: bool) -> String` builds the bash fragment used for `OPTS=$(…)` in chroot. It resolves `/` via `findmnt`, strips btrfs `[@subvol]` suffixes, and for LUKS emits `rd.luks.name=…` or `cryptdevice=…` based on `/etc/mkinitcpio.conf` hooks. Used by systemd-boot, GRUB (when encrypted), EFISTUB (non-UKI), Limine (non-UKI), and sysconfig writes the same logic into `/etc/kernel/cmdline` when UKI is enabled.

### UKI surface area

| Area | Implementation |
|------|----------------|
| State / config / TUI | `uki_enabled`, persistence, summary, UKI screen hidden for GRUB |
| `system.rs` | `systemd-ukify` in pacstrap when `uki_requested` |
| `sysconfig.rs` | `/etc/kernel/cmdline`, `/boot/EFI/Linux`, patch `linux.preset`, `mkinitcpio -P` if encrypted or UKI |
| `bootloader.rs` | `uki_requested()`; systemd-boot `efi` entries; EFISTUB UKI `efibootmgr`; Limine `protocol: efi` + UKI paths |
| `unified_kernel_images.rs` | Label "UKI"; `init_unified_kernel_images` delegates to visibility sync |

### mkinitcpio behavior

- **Encrypted:** inject sd-encrypt / encrypt hooks, then `mkinitcpio -P` (with warning guard).
- **UKI (non-GRUB):** preset switches to `default_uki` / `fallback_uki`, comments `default_image` / `fallback_image`, then `mkinitcpio -P` even without encryption.
- **Neither:** no extra `mkinitcpio -P` in sysconfig; pacstrap hooks produce standard initramfs images.

### Test-only helpers

- `AppState::firmware_uefi_override: Option<bool>` — forces UEFI vs BIOS in tests (`is_uefi()` in `flow.rs`).
- Integration tests in `tests/logic.rs` use `bootloader_*`, `uki_*`, `pre_mounted_*` prefixes.

## Implementation Phases

### Phase 1: Extract shared cmdline builder + EFISTUB implementation — **done**

**Goal:** Extract `boot_options_script` into a reusable function. Replace the EFISTUB placeholder with real `efibootmgr` commands that create a UEFI boot entry pointing at the kernel, with options identical to systemd-boot.

#### 1a. Extract cmdline builder

- [x] Create `boot_options_script(encrypted: bool) -> String` on `BootloaderService`.
- [x] Replace inline constructions in `build_plan()` with calls to the new function.
- [x] Verify existing tests still pass.

#### 1b. EFISTUB install commands

- [x] Match arm `2`: `findmnt` + `lsblk` on `/boot`, `efibootmgr --create` for primary and fallback initramfs, `efibootmgr --verbose`.

#### 1c. EFISTUB UEFI-only validation

- [x] `validate_install_requirements()` guard for `bootloader_index == 2 && !is_uefi()`.
- [x] Disks info panel covers EFISTUB on BIOS (`bl != "GRUB" && bl != "Limine"`).

#### 1d. EFISTUB packages

- [x] `efibootmgr` for all UEFI installs; no extra package for EFISTUB.

#### 1e. Tests

- [x] `bootloader_efistub_creates_efibootmgr_entry`, `bootloader_efistub_luks_uses_shared_cmdline`.

**Acceptance criteria:** [x] all met.

---

### Phase 2: Limine — UEFI + BIOS — **done**

**Reference:** `Documents/arch_manual.md` (Limine section) and Arch Wiki Limine.

**Constraint:** Limine reads kernels/initramfs from FAT (etc.); standard ESP-as-`/boot` satisfies this.

#### 2a–2g

- [x] UEFI: `BOOTX64.EFI` → `/boot/EFI/limine/`, `efibootmgr` for `\EFI\limine\BOOTX64.EFI`.
- [x] BIOS: `limine-bios.sys`, `limine bios-install <disk>`.
- [x] `/boot/limine.conf` with shared `boot_options_script`, primary + fallback entries.
- [x] `/etc/pacman.d/hooks/99-limine.hook` (UEFI).
- [x] `effective_bios_grub_disk` extended for Limine (index 3); pre-mounted BIOS validation with GRUB.
- [x] TUI labels "Limine" / "Efistub".
- [x] Tests: `bootloader_limine_uefi_*`, `bootloader_limine_bios_*`, `bootloader_limine_luks_cmdline`.

**Acceptance criteria:** [x] all met.

---

### Phase 3: Unified Kernel Images — **done**

**Approach:** mkinitcpio UKI outputs; `systemd-ukify` in pacstrap; cmdline from `/etc/kernel/cmdline`.

#### 3a–3h

- [x] `systemd-ukify` when `uki_enabled && bootloader_index != 1`.
- [x] `/etc/kernel/cmdline` before `mkinitcpio -P`.
- [x] Preset `default_uki` / `fallback_uki`, comment `default_image` / `fallback_image`, `mkdir /boot/EFI/Linux`.
- [x] `mkinitcpio -P` when encrypted **or** UKI.
- [x] systemd-boot `efi` lines; EFISTUB UKI loaders; Limine `protocol: efi` + UKI paths; GRUB unchanged / UKI hidden.
- [x] UI "UKI"; `init_unified_kernel_images` → `update_unified_kernel_images_visibility`.
- [x] Tests: `uki_systemd_boot_uses_efi_path`, `uki_efistub_points_to_uki_loader`, `uki_sysconfig_*`, `uki_pacstrap_*`, `uki_limine_uses_efi_protocol`.

**Acceptance criteria:** [x] all met.

---

### Phase 4: Hardening and documentation

- [ ] **Multi-kernel support** — if `selected_kernels` contains more than `linux` (e.g. `linux-lts`), generate additional bootloader entries and UKI presets per kernel. Currently paths hardcode `linux` / `arch-linux.efi`. **Open follow-up.**
- [x] **Pre-mounted mode** — `tests/logic.rs`: `pre_mounted_efistub_empty_target_still_generates_efibootmgr`, `pre_mounted_limine_uefi_empty_target_still_installs` (empty partition target, `findmnt /boot`).
- [x] **Docs** — `Documents/STATUS.md`, `USAGE.md`, `ROADMAP.md`, `TODO_SUMMARY.md`.
- [x] **Secure Boot** — signing out of scope; P1/P2 follow-up in `TODO_SUMMARY.md`.
- [x] **Info panel** — EFISTUB firmware/cmdline note in `src/render/sections/info/bootloader.rs`.

## Recommended Delivery Order

| Order | Phase | Status |
|-------|-------|--------|
| 1 | Phase 1: Shared cmdline + EFISTUB | Done |
| 2 | Phase 2: Limine | Done |
| 3 | Phase 3: UKI | Done |
| 4 | Phase 4: Hardening (multi-kernel TBD) | Partial |

## File Touchpoints (reference)

| Phase | Files |
|-------|--------|
| 1 | `bootloader.rs`, `flow.rs`, `tests/logic.rs` |
| 2 | `bootloader.rs`, `flow.rs`, `bootloader.rs` (app), `info/bootloader.rs`, `state.rs` (`firmware_uefi_override`), `tests/logic.rs` |
| 3 | `sysconfig.rs`, `bootloader.rs`, `system.rs`, `unified_kernel_images.rs`, `tests/logic.rs` |
| 4 | `bootloader.rs` (multi-kernel), docs |

## Test Baseline

Run: `cargo test -- --test-threads=1`.

- **Library:** ~70 tests × 2 binaries (`lib` + `main`).
- **Integration** (`tests/logic.rs`): 28+ tests including bootloader, UKI, pre-mounted, sysconfig, etc.

**Bootloader / UKI–related integration tests (non-exhaustive):**

- `bootloader_systemd_boot_writes_loader_and_entries`, `bootloader_systemd_boot_luks_adds_rd_luks_name`
- `bootloader_grub_luks_injects_cmdline`
- `bootloader_efistub_creates_efibootmgr_entry`, `bootloader_efistub_luks_uses_shared_cmdline`
- `bootloader_limine_uefi_creates_conf_and_efibootmgr`, `bootloader_limine_bios_installs_and_creates_conf`, `bootloader_limine_luks_cmdline`
- `uki_*`, `pre_mounted_efistub_empty_target_still_generates_efibootmgr`, `pre_mounted_limine_uefi_empty_target_still_installs`

**Unit tests** (`bootloader.rs`): `effective_bios_grub_disk_*`, Limine disk resolution.

## Definition of Done

- [x] `bootloader_index == 2` (EFISTUB): real `efibootmgr` + LUKS cmdline when encrypted.
- [x] `bootloader_index == 3` (Limine): UEFI + BIOS paths + `limine.conf`.
- [x] `uki_enabled == true`: mkinitcpio UKI preset, `/etc/kernel/cmdline`, bootloader UKI paths (non-GRUB).
- [x] No `echo 'TODO: ...'` in `BootloaderService::build_plan`.
- [x] No "not implemented yet" for EFISTUB, Limine, UKI in those screens.
- [x] Tests for EFISTUB, Limine (UEFI/BIOS), UKI; Workstream 2 marked done in `TODO_SUMMARY.md` with follow-ups noted.
- [ ] **Optional completion:** multi-kernel entries + UKI presets (Phase 4).

## Open Questions (resolution notes)

1. **Limine config location** — **Resolved:** `/boot/limine.conf` (root of ESP when `/boot` is ESP), matching wiki-style layout.
2. **Limine BIOS** — **Resolved:** same phase; `limine bios-install` + `limine-bios.sys`.
3. **UKI output path** — **`/boot/EFI/Linux/arch-linux.efi`** used for systemd-boot auto-discovery, EFISTUB, and Limine `protocol: efi`.
4. **Multi-kernel UKI** — **Deferred:** Phase 4; extend presets and entries over `selected_kernels`.
5. **EFISTUB firmware quirks** — **Mitigation:** info panel mentions UKI when firmware ignores cmdline from direct kernel boot.
