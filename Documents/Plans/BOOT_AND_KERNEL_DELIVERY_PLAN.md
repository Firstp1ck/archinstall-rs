# Boot and Kernel Delivery Implementation Plan

## Goal

Complete the bootloader matrix (EFISTUB, Limine) and a modern kernel boot pipeline, including optional Unified Kernel Images (UKI) integrated with mkinitcpio and the chosen UEFI boot path (systemd-boot / EFISTUB / Limine).

## Why This Comes Next

Workstream 1 ([STORAGE_LAYOUT_PLAN.md](STORAGE_LAYOUT_PLAN.md)) delivered a normalized `StoragePlan`, LUKS-aware cmdline logic in systemd-boot and GRUB, and reliable mount/fstab behavior. Boot and kernel delivery is the next P0 layer because:

- Users who pick **EFISTUB** (index 2) get `echo 'TODO: EFISTUB configuration not yet implemented'`.
- Users who pick **Limine** (index 3) get `echo 'TODO: Limine bootloader setup not yet implemented'`.
- **UKI** (`uki_enabled`) is togglable in the TUI, persisted in config, shown in the install summary — but produces zero install commands.

## Dependencies and Unblocks

| Depends on | Unblocks |
|------------|----------|
| Stable `StoragePlan` with ESP at `/mnt/boot` on UEFI (done) | Secure Boot signing pipeline (future P1/P2) |
| LUKS hook alignment in `SysConfigService` + `boot_options_script` pattern (done) | All four bootloaders producing consistent encrypted-root cmdlines |
| `efibootmgr` in pacstrap for all UEFI installs (done, `system.rs` line 142) | EFISTUB and Limine UEFI registration |
| `limine` in pacstrap when index 3 (done, `system.rs` line 149) | Limine install path |

**Related backlog:** [TODO_SUMMARY.md](../../TODO_SUMMARY.md) — Workstream 2, Boot and Kernel Delivery (P0).

## Current Architecture (as-is)

### Data flow today

```
AppState (bootloader_index, uki_enabled, selected_kernels, …)
   │
   ├─► SystemService::build_pacstrap_plan()     → packages: efibootmgr (UEFI), grub/limine
   ├─► SysConfigService::build_plan()           → mkinitcpio hooks + mkinitcpio -P (encrypted only)
   └─► BootloaderService::build_plan(state, device, storage_plan)
         ├─► 0 systemd-boot: bootctl install, loader.conf, arch.conf entries, efibootmgr fallback
         ├─► 1 GRUB: grub-install, GRUB_CMDLINE_LINUX injection when encrypted, grub-mkconfig
         ├─► 2 EFISTUB: echo 'TODO ...'    ← placeholder
         └─► 3 Limine: echo 'TODO ...'     ← placeholder

AppState.uki_enabled
   └─► Config IO + TUI toggle + install summary only — zero chroot commands generated
```

### Install section ordering (flow.rs, build_install_sections)

```
Locales → Mirrors → Pre-cleanup → Partitioning → Volume stack → Mounting
 → System pre-install → Pacstrap → fstab → System configuration (sysconfig)
 → Network → Bootloader setup → User setup
```

Key ordering fact: `SysConfigService::build_plan()` runs mkinitcpio **before** the bootloader section. This is correct for systemd-boot/GRUB (bootloader entries reference already-built initramfs). For UKI, mkinitcpio must build the UKI itself, so `/etc/kernel/cmdline` and the mkinitcpio UKI preset must be configured **before** mkinitcpio runs — all within the sysconfig section.

### Bootloader index mapping

| Index | ID | `BootloaderService` status | Packages in pacstrap (`system.rs`) | BIOS validation (`flow.rs`) |
|------|-----|---------------------------|------------------------------------|-----------------------------|
| 0 | systemd-boot | Complete | *(implicit systemd)* | Yes — blocks with "requires UEFI" (line 487) |
| 1 | GRUB | Complete | `grub` | N/A — supports UEFI + BIOS |
| 2 | EFISTUB | **Placeholder** | *(none; `efibootmgr` added for all UEFI)* | **Missing** — no UEFI-only guard |
| 3 | Limine | **Placeholder** | `limine` | N/A — supports UEFI + BIOS |

### Shared kernel cmdline generation (`boot_options_script`)

A local `String` variable inside `BootloaderService::build_plan()`. It resolves the root device via `findmnt`, strips btrfs `[/@]` suffixes, and for encrypted roots probes `cryptsetup status` + `blkid` to emit either `rd.luks.name=<uuid>=<mapper>` (systemd hooks) or `cryptdevice=UUID=<uuid>:<mapper>` (udev hooks). Both systemd-boot and GRUB branches consume this same variable.

EFISTUB and Limine need identical cmdline semantics (including LUKS awareness). The variable should be extracted to a shared builder so all four branches reuse it without copy-paste drift.

### UKI surface area (today)

| File | What exists | What's missing |
|------|-------------|----------------|
| `src/core/state.rs` | `uki_enabled: bool`, `uki_focus_index: usize` | — |
| `src/app/unified_kernel_images.rs` | TUI toggle, visibility (hidden for GRUB), `init_unified_kernel_images()` with TODO body | Install-side effects |
| `src/app/config/io.rs` | Persists `unified_kernel_images.enabled` | — |
| `src/app/install/ui.rs` | Shows "Enabled"/"Disabled" in summary | — |
| `src/input/screens/uki.rs` | Up/down/enter handlers | — |
| `src/render/sections/info/unified_kernel_images.rs` | Description panel | — |
| `src/core/services/sysconfig.rs` | Runs `mkinitcpio -P` only when encrypted | UKI preset, `/etc/kernel/cmdline`, conditional UKI build |
| `src/core/services/bootloader.rs` | No branch on `uki_enabled` | UKI-aware entry paths for each bootloader |
| `src/core/services/system.rs` | No UKI-related packages | `systemd-ukify` in pacstrap when UKI enabled |

### mkinitcpio behavior today

- `SysConfigService::build_plan()` calls `mkinitcpio -P` **only when encryption is enabled** (to inject sd-encrypt/encrypt hooks).
- For non-encrypted installs, mkinitcpio runs implicitly via pacstrap's post-install hooks, producing standard `initramfs-linux.img` and `initramfs-linux-fallback.img`.
- No mkinitcpio preset is configured for UKI output. The default preset outputs initramfs only.

## Implementation Phases

### Phase 1: Extract shared cmdline builder + EFISTUB implementation

**Goal:** Extract `boot_options_script` into a reusable function. Replace the EFISTUB placeholder with real `efibootmgr` commands that create a UEFI boot entry pointing at the kernel, with options identical to systemd-boot.

#### 1a. Extract cmdline builder

- [ ] Create `fn boot_options_script(encrypted: bool) -> String` as a method on `BootloaderService` (or a free function in `bootloader.rs`), returning the shell script string currently inlined in `build_plan()`.
- [ ] Replace the two inline constructions (encrypted and unencrypted) in `build_plan()` with calls to the new function.
- [ ] Verify existing tests still pass — this is a pure refactor.

#### 1b. EFISTUB install commands

- [ ] In `build_plan()` match arm `2`, generate chroot commands:
  - Resolve ESP disk and partition number via `findmnt` + `lsblk` on `/boot` (same pattern as systemd-boot fallback block, line 163).
  - Create `efibootmgr --create --disk <disk> --part <partnum> --label "Arch Linux" --loader '\vmlinuz-linux' --unicode '<cmdline> initrd=\initramfs-linux.img'` using the shared `boot_options_script` to fill the cmdline.
  - Create a fallback entry with `initrd=\initramfs-linux-fallback.img`.
  - Run `efibootmgr --verbose` to verify.

#### 1c. EFISTUB UEFI-only validation

- [ ] In `validate_install_requirements()` (flow.rs), add a guard for `bootloader_index == 2 && !is_uefi()` mirroring the existing systemd-boot guard (line 487).
- [ ] Ensure the disks info panel warning (line 238, `bl != "GRUB" && bl != "Limine"`) already covers EFISTUB on BIOS — it does.

#### 1d. EFISTUB packages

- [ ] Verify `efibootmgr` is already included for all UEFI (confirmed: `system.rs` line 142). No additional packages needed for EFISTUB.
- [ ] No changes to `additional_packages.rs` — already shows `efibootmgr` as implied for EFISTUB (line 268).

#### 1e. Tests

- [ ] Add `test_efistub_creates_efibootmgr_entry` — assert `efibootmgr --create` and `vmlinuz-linux` and `initramfs-linux.img` in commands.
- [ ] Add `test_efistub_luks_uses_shared_cmdline` — assert encrypted EFISTUB includes `rd.luks.name=` or `cryptdevice=` pattern.
- [ ] Verify existing systemd-boot and GRUB tests unchanged.

**Files touched:**

- `src/core/services/bootloader.rs` — extract helper, EFISTUB branch
- `src/app/install/flow.rs` — UEFI-only validation for index 2
- `tests/logic.rs` — 2+ new tests

**Acceptance criteria:**

- [ ] `bootloader_index == 2` on UEFI produces `efibootmgr` commands with correct kernel path and cmdline.
- [ ] Encrypted EFISTUB produces same LUKS cmdline semantics as systemd-boot.
- [ ] `bootloader_index == 2` on BIOS is blocked by validation.
- [ ] All existing tests pass unchanged.

---

### Phase 2: Limine — UEFI + BIOS install path, config generation, pacman hook

**Goal:** Replace the Limine placeholder with a complete install path covering both UEFI and BIOS, generating `limine.conf` and registering the boot entry.

**Reference:** `Documents/arch_manual.md` lines 177–216 and Arch Wiki Limine page.

**Important constraint:** Limine can only read files from FAT12/16/32 or ISO9660 partitions. On standard installs the kernel and initramfs are on the ESP (FAT32, `/boot`), which satisfies this requirement.

#### 2a. Limine UEFI path

- [ ] Copy `BOOTX64.EFI` from `/usr/share/limine/` to ESP at `/boot/EFI/limine/`.
- [ ] Register via `efibootmgr --create --disk <disk> --part <partnum> --label "Arch Linux Limine" --loader '\EFI\limine\BOOTX64.EFI' --unicode` (resolve disk/partnum from `findmnt` on `/boot` like EFISTUB).

#### 2b. Limine BIOS path

- [ ] Copy `/usr/share/limine/limine-bios.sys` to `/boot/limine/` inside chroot.
- [ ] Run `limine bios-install <whole-disk>` using the same disk resolution as `effective_bios_grub_disk()` (which currently early-returns empty for non-GRUB; needs extending or a similar helper for Limine).

#### 2c. limine.conf generation

- [ ] Generate `/boot/limine.conf` (or `/boot/EFI/limine/limine.conf`) inside chroot with:
  ```
  timeout: 5

  /Arch Linux
      protocol: linux
      path: boot():/vmlinuz-linux
      cmdline: <resolved cmdline>
      module_path: boot():/initramfs-linux.img
  ```
- [ ] Add a fallback entry with `initramfs-linux-fallback.img`.
- [ ] Cmdline via shared `boot_options_script` from Phase 1a (eval in chroot and embed result).

#### 2d. Pacman hook (optional but recommended)

- [ ] Install `99-limine.hook` at `/etc/pacman.d/hooks/` to auto-copy `BOOTX64.EFI` after limine package upgrades (matches `Documents/arch_manual.md` pattern).

#### 2e. BIOS disk resolution for Limine

- [ ] Extend or generalize `effective_bios_grub_disk()` to also handle `bootloader_index == 3`, or create a similar function. Currently it early-returns empty for non-GRUB.
- [ ] Update `validate_install_requirements()` if pre-mounted BIOS Limine needs a resolvable disk (parallel to the existing GRUB check at flow.rs line 472).

#### 2f. UI label cleanup

- [ ] Update label in `src/app/bootloader.rs` from `"Limine (not implemented yet)"` to `"Limine"`.
- [ ] Update label in `src/render/sections/info/bootloader.rs` from `"Limine (not implemented yet)"` to `"Limine"`.

#### 2g. Tests

- [ ] Add `test_limine_uefi_creates_conf_and_efibootmgr` — assert `limine.conf` generation, `BOOTX64.EFI` copy, and `efibootmgr` entry.
- [ ] Add `test_limine_bios_installs_and_creates_conf` — assert `limine bios-install` and `limine-bios.sys` copy in commands (for non-UEFI state).
- [ ] Add `test_limine_luks_cmdline` — assert encrypted Limine includes LUKS cmdline.

**Files touched:**

- `src/core/services/bootloader.rs` — Limine branch (UEFI + BIOS), disk resolution
- `src/app/install/flow.rs` — validation for pre-mounted BIOS Limine
- `src/app/bootloader.rs` — label update
- `src/render/sections/info/bootloader.rs` — label update
- `tests/logic.rs` — 3+ new tests

**Acceptance criteria:**

- [ ] Limine on UEFI produces: EFI copy, efibootmgr entry, limine.conf with correct cmdline.
- [ ] Limine on BIOS produces: bios-install command, limine-bios.sys copy, limine.conf.
- [ ] Encrypted Limine uses same LUKS cmdline as other bootloaders.
- [ ] Labels no longer say "not implemented yet".

---

### Phase 3: Unified Kernel Images — mkinitcpio preset, cmdline, boot integration

**Goal:** When `uki_enabled == true` (and bootloader is not GRUB), configure mkinitcpio to build a UKI and point the bootloader/firmware at it.

**Canonical Arch approach (2025+):** mkinitcpio builds the UKI when its preset is configured with a `uki` output path. If `systemd-ukify` is installed, mkinitcpio delegates UKI assembly to ukify. The kernel cmdline is read from `/etc/kernel/cmdline`.

#### 3a. Package requirements

- [ ] Add `systemd-ukify` to pacstrap package set when `uki_enabled && bootloader_index != 1` (in `SystemService::build_pacstrap_plan()`).

#### 3b. Kernel cmdline file

- [ ] In `SysConfigService::build_plan()`, when `uki_enabled`, write `/etc/kernel/cmdline` inside chroot using the same resolution logic as `boot_options_script` (shared helper from Phase 1a).
- [ ] This must happen **before** mkinitcpio runs, so insert it early in the sysconfig command sequence.

#### 3c. mkinitcpio UKI preset

- [ ] When `uki_enabled`, modify the mkinitcpio preset (e.g. `/etc/mkinitcpio.d/linux.preset`) to set:
  - `default_uki="/boot/EFI/Linux/arch-linux.efi"`
  - `fallback_uki="/boot/EFI/Linux/arch-linux-fallback.efi"`
  - Unset or comment out `default_image` and `fallback_image` (initramfs paths), since the UKI embeds the initramfs.
- [ ] Create directory `/boot/EFI/Linux/` inside chroot.

#### 3d. mkinitcpio invocation

- [ ] Currently `mkinitcpio -P` runs only when encrypted (`sysconfig.rs` line 158–170). When `uki_enabled`, it must **always** run (to build the UKI), regardless of encryption.
- [ ] Restructure the mkinitcpio block: if `encrypted` → inject hooks first, then if `uki_enabled` → configure preset, then always run `mkinitcpio -P` if either condition is true. For non-encrypted + non-UKI installs, pacstrap's implicit mkinitcpio run suffices.

#### 3e. Bootloader entry updates

When `uki_enabled`, bootloader entries reference the UKI binary instead of separate vmlinuz + initramfs:

- [ ] **systemd-boot (index 0):** Replace the `arch.conf` entry's `linux`, `initrd`, and `options` lines with a single `efi /EFI/Linux/arch-linux.efi` line. Similarly for fallback.
- [ ] **EFISTUB (index 2):** `efibootmgr --loader '\EFI\Linux\arch-linux.efi'` — no `--unicode` cmdline needed since it's embedded in the UKI.
- [ ] **Limine (index 3):** Entry uses UKI path per Limine's protocol support (`path: boot():/EFI/Linux/arch-linux.efi`).
- [ ] **GRUB (index 1):** No UKI support — `uki_enabled` is already hidden for GRUB in the TUI (`update_unified_kernel_images_visibility()`, line 15). No changes needed.

#### 3f. UI cleanup

- [ ] Update label in `src/app/unified_kernel_images.rs` from `"UKI (not implemented yet)"` to `"UKI"`.
- [ ] Implement `init_unified_kernel_images()` body or remove the TODO (it may not need runtime init beyond what exists).

#### 3g. EFISTUB label cleanup

- [ ] Update label in `src/app/bootloader.rs` from `"Efistub (not implemented yet)"` to `"Efistub"`.
- [ ] Update label in `src/render/sections/info/bootloader.rs` from `"Efistub (not implemented yet)"` to `"Efistub"`.

#### 3h. Tests

- [ ] Add `test_uki_systemd_boot_uses_efi_path` — assert UKI-enabled systemd-boot entry contains `efi /EFI/Linux/arch-linux.efi` instead of separate linux/initrd.
- [ ] Add `test_uki_efistub_points_to_uki` — assert loader path is `\EFI\Linux\arch-linux.efi`.
- [ ] Add `test_uki_sysconfig_writes_cmdline_and_preset` — assert `/etc/kernel/cmdline` and mkinitcpio preset modification appear in sysconfig commands.
- [ ] Add `test_uki_sysconfig_runs_mkinitcpio_without_encryption` — assert `mkinitcpio -P` runs even when not encrypted, if UKI is enabled.
- [ ] Add `test_uki_pacstrap_includes_ukify` — assert `systemd-ukify` is in package set when UKI enabled.

**Files touched:**

- `src/core/services/sysconfig.rs` — `/etc/kernel/cmdline`, mkinitcpio preset, conditional mkinitcpio execution
- `src/core/services/bootloader.rs` — UKI-aware entry paths for indices 0, 2, 3
- `src/core/services/system.rs` — `systemd-ukify` in package set
- `src/app/unified_kernel_images.rs` — label update, `init_unified_kernel_images()` cleanup
- `src/app/bootloader.rs` — EFISTUB label update
- `src/render/sections/info/bootloader.rs` — EFISTUB label update
- `tests/logic.rs` — 5+ new tests

**Acceptance criteria:**

- [ ] `uki_enabled == true` with systemd-boot produces UKI path entries, mkinitcpio preset config, and `/etc/kernel/cmdline`.
- [ ] `uki_enabled == true` with EFISTUB boots the UKI directly via firmware.
- [ ] `uki_enabled == false` preserves today's separate vmlinuz + initramfs behavior for all bootloaders.
- [ ] `mkinitcpio -P` runs for UKI regardless of encryption state.
- [ ] Labels no longer say "not implemented yet".

---

### Phase 4: Hardening and documentation

- [ ] **Multi-kernel support** — if `selected_kernels` contains more than `linux` (e.g. `linux-lts`), generate additional bootloader entries and UKI presets per kernel. Currently systemd-boot entries hardcode `vmlinuz-linux`. This is a separate scope item.
- [ ] **Pre-mounted mode** — verify EFISTUB and Limine work with `effective_bios_grub_disk` / disk resolution when there's no partitioning target.
- [ ] **Docs** — update `Documents/STATUS.md` (remove "GRUB and systemd-boot only"), `Documents/USAGE.md` (bootloader bullets), `Documents/ROADMAP.md` (move EFISTUB/Limine from roadmap to done).
- [ ] **Secure Boot** — signing pipeline is explicitly out of scope for this plan. List as a P1/P2 follow-up in `TODO_SUMMARY.md`.

## Recommended Delivery Order

| Order | Phase | Rationale | Risk | Effort |
|-------|-------|-----------|------|--------|
| 1 | Phase 1: Shared cmdline + EFISTUB | Smallest new surface; validates shared cmdline extraction; UEFI-only | Low–Medium | Low–Medium |
| 2 | Phase 2: Limine | User-visible bootloader; reuses cmdline helper; UEFI+BIOS | Medium | Medium |
| 3 | Phase 3: UKI | Touches mkinitcpio + all boot paths; needs stable EFISTUB/Limine patterns | High (toolchain) | High |
| 4 | Phase 4: Hardening | Polish, multi-kernel, docs | Low | Low |

**Alternative:** If UKI is urgently needed only for systemd-boot, implement Phase 3 steps 3a–3d + 3e (systemd-boot only) right after Phase 1, then extend to EFISTUB/Limine after Phase 2.

## File Touchpoints (complete list)

### Phase 1

- `src/core/services/bootloader.rs` — extract `boot_options_script()` helper, EFISTUB branch
- `src/app/install/flow.rs` — UEFI-only validation for `bootloader_index == 2`
- `tests/logic.rs` — 2+ new EFISTUB tests

### Phase 2

- `src/core/services/bootloader.rs` — Limine branch (UEFI + BIOS paths, limine.conf, pacman hook)
- `src/app/install/flow.rs` — pre-mounted BIOS Limine validation
- `src/app/bootloader.rs` — label cleanup ("Limine")
- `src/render/sections/info/bootloader.rs` — label cleanup ("Limine")
- `tests/logic.rs` — 3+ new Limine tests

### Phase 3

- `src/core/services/sysconfig.rs` — `/etc/kernel/cmdline`, mkinitcpio UKI preset, conditional execution
- `src/core/services/bootloader.rs` — UKI-aware entries for indices 0, 2, 3
- `src/core/services/system.rs` — `systemd-ukify` package
- `src/app/unified_kernel_images.rs` — label cleanup, `init_unified_kernel_images()` body
- `src/app/bootloader.rs` — EFISTUB label cleanup
- `src/render/sections/info/bootloader.rs` — EFISTUB label cleanup
- `tests/logic.rs` — 5+ new UKI tests

### Phase 4

- `Documents/STATUS.md`, `Documents/USAGE.md`, `Documents/ROADMAP.md` — status sync
- Multi-kernel entry generation in `bootloader.rs`

## Test Baseline

Current: 127/127 passing (57 lib tests x 2 targets + 13 integration tests in `tests/logic.rs`).

Existing bootloader-related tests in `tests/logic.rs`:

- `bootloader_systemd_boot_writes_loader_and_entries` — non-encrypted systemd-boot
- `bootloader_systemd_boot_luks_adds_rd_luks_name` — encrypted systemd-boot
- `bootloader_grub_luks_injects_cmdline` — encrypted GRUB

Unit tests in `bootloader.rs`:

- `effective_bios_grub_disk_passes_through_partition_target`
- `effective_bios_grub_disk_non_grub_returns_empty_for_empty_target`

Expected after all phases: ~140+ tests (10+ new across phases).

## Definition of Done

This goal is complete when:

- [ ] `bootloader_index == 2` (EFISTUB) runs real UEFI boot installation (efibootmgr) with correct kernel cmdline including LUKS.
- [ ] `bootloader_index == 3` (Limine) runs real installation on both UEFI (EFI copy + efibootmgr + limine.conf) and BIOS (bios-install + limine.conf).
- [ ] `uki_enabled == true` configures mkinitcpio UKI preset, writes `/etc/kernel/cmdline`, and updates bootloader entries to reference the UKI binary.
- [ ] No remaining `echo 'TODO: ...'` placeholders in `BootloaderService::build_plan`.
- [ ] No remaining "not implemented yet" labels in TUI for EFISTUB, Limine, or UKI.
- [ ] Tests cover encrypted + unencrypted cases for EFISTUB, Limine (UEFI), and UKI with systemd-boot.
- [ ] [TODO_SUMMARY.md](../../TODO_SUMMARY.md) Workstream 2 checkboxes can be marked complete.

## Open Questions (resolve during implementation)

1. **Limine config location** — `/boot/limine.conf` (wiki standard) vs `/boot/EFI/limine/limine.conf` (arch_manual.md pattern). Need to test which Limine actually reads on UEFI. Wiki says `limine.conf` in the root of the partition containing the EFI binary.
2. **Limine BIOS in same phase or split** — Phase 2 covers both, but if BIOS Limine is complex (MBR install, disk table handling), it could be 2a/2b.
3. **UKI output path** — `/boot/EFI/Linux/arch-linux.efi` is the systemd-boot auto-discovery path. Verify this works for Limine and EFISTUB.
4. **Multi-kernel UKI** — one UKI per kernel vs single default. Defer to Phase 4, but design Phase 3 presets so extension is easy (iterate `selected_kernels`).
5. **EFISTUB firmware quirks** — some firmware does not pass cmdline from UEFI variables. The Arch Wiki recommends UKI in those cases. Consider mentioning in the info panel description. Not a blocker for implementation.
