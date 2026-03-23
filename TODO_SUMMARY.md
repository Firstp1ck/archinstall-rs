# Semantic TODO Backlog

This backlog groups repository TODOs by feature area and dependency, then orders them by implementation priority.

## Priority Legend

- **P0**: Core install correctness and blockers for major roadmap features.
- **P1**: High-value UX/completeness improvements that unlock smooth usage.
- **P2**: Nice-to-have improvements that can follow after core stabilization.

## What To Implement Next (Project List)

1. **Disk & filesystem foundation (P0)**  
   Complete advanced partitioning + mount/fstab consistency so later features (encryption/UKI/boot) are reliable.
2. **Boot chain expansion (P0)** — *core path done*  
   EFISTUB, Limine, and UKI (mkinitcpio + non-GRUB bootloaders) are implemented; follow-ups: multi-kernel UKI/entries, Secure Boot signing (P1/P2).
3. **Encryption UX + system hooks (P0/P1)**  
   Finalize passphrase UX and mkinitcpio/boot integration for encrypted installs, including **LVM on LUKS** and **LUKS on LVM** layout choices.
4. **System identity and essential config hardening (P1)**  
   Finish hostname/root/time-sync/locales behavior and additional system config coverage.
5. **Config automation and unattended mode (P1/P2)**  
   Make save/load + CLI config-driven unattended installs first-class.
6. **User experience polish and ecosystem features (P2)**  
   Shell/groups UI, kernel screen completeness, AUR flow polish, abort flow cleanup.
7. **Extended configuration, localization, and desktop ecosystem (P2)**  
   Multi-language UI, custom mirrors/repos, swap compression, rEFInd/linux-hardened follow-ups, U2F login, extra DE/WM targets, Bluetooth/print services, NetworkManager+iwd, btrfs snapshot stacks (Timeshift/Snapper, grub-btrfs with GRUB).

## Workstreams

### 1) Disk Architecture and Storage Stack (P0)

**Goal:** Reliable advanced storage layout as the base for all install modes.

- [x] Normalized storage model (`StoragePlan`) and planner introduced (Phase 1).
- [x] Install flow wired to use `StoragePlanner` instead of hardcoded services (Phase 2).
- [x] Manual partition editor validation and planner hardening (Phase 3).
- [x] btrfs subvolume-aware mount layout and fstab tuning (Phase 4).
- [x] Pre-mounted flow (Phase 5).
- [x] LVM + RAID extensibility (Phase 6).

**Progress:** All phases (1-6) and all follow-up items complete. See `Documents/Plans/STORAGE_LAYOUT_PLAN.md` for full details.

Follow-ups completed:

- [x] `DeviceStack::setup_commands()` wired for LUKS/LVM/RAID command generation.
- [x] `BootloaderService` accepts `&StoragePlan` with LUKS-aware kernel cmdline.
- [x] Btrfs subvolume preset TUI selector on Disks screen.

**Additional roadmap:**

- [ ] **LVM:** first-class **LVM** in the disks/install flow (VG/LV guided setup, sizing, and integration with encryption stacking in workstream 3) — low-level planner support exists; track UX completeness and common presets.

**Unblocks:** Encryption, UKI, non-default bootloaders, unattended reliability.

### 2) Boot and Kernel Delivery (P0)

**Goal:** Complete bootloader matrix and modern kernel boot pipeline.

**Plan:** See [`Documents/Plans/BOOT_AND_KERNEL_DELIVERY_PLAN.md`](Documents/Plans/BOOT_AND_KERNEL_DELIVERY_PLAN.md) for phased implementation (EFISTUB → Limine → UKI integration).

- [x] EFISTUB boot entry creation + kernel cmdline generation.
- [x] Limine setup path (UEFI + BIOS, `limine.conf`, optional pacman hook).
- [x] Unified Kernel Images: mkinitcpio preset, `/etc/kernel/cmdline`, `systemd-ukify`, entries for systemd-boot / EFISTUB / Limine.

**Follow-ups (not blocking basic installs):**

- [ ] Multi-kernel: extra `linux-lts`, **`linux-hardened`**, and similar variants — bootloader + UKI preset entries (currently default `linux` only).
- [ ] **rEFInd** bootloader option (alongside EFISTUB, Limine, systemd-boot/GRUB paths as applicable).
- [ ] **Secure Boot (P1/P2):** signing pipeline for UKI/EFI binaries — out of scope for initial UKI delivery; track when hardening boot trust.

**Depends on:** Stable partition/mount model and system hook generation.  
**Unblocks:** Secure-boot-friendly flow and advanced boot scenarios (signing still TODO).

### 3) Encryption and Early-Boot Integration (P0/P1)

**Goal:** Make encrypted installs fully usable end-to-end.

- [ ] Encryption password prompts + partition selection UX.
- [ ] **Disk encryption + LVM stacking:** support **LVM on LUKS** and **LUKS on LVM** — selectable layout with correct crypttab/initramfs, unlock order, and kernel cmdline for boot.
- [ ] mkinitcpio/system config hooks for LUKS/UKI and related early-boot settings.

**Depends on:** Disk architecture + boot chain updates.  
**Unblocks:** Production-grade encrypted desktop/server installs.

### 4) Core System Configuration Completion (P1)

**Goal:** Eliminate remaining setup gaps in essential system config screens/services.

- [ ] Locales pre-install steps in install flow.
- [ ] Keyboard layout coverage beyond current DE-specific scope.
- [ ] Hostname dialog validation rules.
- [ ] Root password validation/state handling.
- [ ] Automatic time sync UI interactions.

**Depends on:** None (parallelizable).  
**Unblocks:** Better first-boot correctness and fewer post-install manual fixes.

### 5) Configuration Automation and Non-Interactive Installs (P1/P2)

**Goal:** Move toward reproducible, template-driven installs.

- [ ] Save/Load configuration actions UI (TOML IO) completion and consistency checks.
- [ ] Parse CLI config path + unattended flags to run non-interactively.

**Depends on:** Core screens/services behavior being stable.  
**Unblocks:** CI/testing workflows, repeatable installs, future cloud/template features.

### 6) UX and Ecosystem Polish (P2)

**Goal:** Improve usability and completeness after core workflows are stable.

- [ ] User setup enhancements (shell selection + groups UI).
- [ ] Kernel screen initialization/actions completion.
- [ ] AUR helper support alignment/polish in system service flow.
- [ ] Abort flow integration and cleanup handling.

**Depends on:** Core implementation complete.  
**Unblocks:** Better day-to-day user experience.

### 7) Extended Configuration, Localization, and Desktop Ecosystem (P2)

**Goal:** Broader installer coverage for language, mirrors, repositories, swap tuning, boot/auth/desktop choices, common hardware services, and Wi-Fi backend selection.

- [ ] **Localization:** installer **multi-language** support — **English** and **German** (UI strings / locale selection for the TUI or equivalent).
- [ ] **Pacman mirrors:** allow adding a **custom mirror server** (user-defined base URL) in addition to reflector/country presets.
- [ ] **Custom repositories:** add repo flow capturing **Name → URL → signature check → signing options** (mirror `[repo]` semantics: SigLevel, key/trust options as needed).
- [ ] **Swap compression:** selectable algorithm where supported (e.g. zram) — **zstd** (default), **lzo-rle**, **lzo**, **lz4**, **lz4hc**.
- [ ] **U2F login setup:** configure **U2F device**-based login (e.g. PAM / `pam_u2f` and related packages) as an optional post-install security step.
- [ ] **Desktop environment / compositor / WM:** add install targets for **Niri**, **River**, **Xmonad**, **labwc**, **COSMIC** (alongside existing DE/WM options).
- [ ] **Packages / services:** optional **Bluetooth** stack setup; **print service** (e.g. CUPS and sensible defaults or driver guidance).
- [ ] **Btrfs snapshots:** optional install of **Timeshift** or **Snapper** (package + basic configuration aligned with subvolume layout); when **GRUB** is the bootloader, also set up **grub-btrfs** so snapshots appear in the boot menu.
- [ ] **Network manager:** **NetworkManager** with **iwd** backend option (vs wpa_supplicant or other backends).

**Depends on:** Stable core install flow and package/service wiring patterns.  
**Unblocks:** Power-user and regional installs without heavy post-install scripting.

## Suggested Execution Plan

### Now (next sprint)

- [x] Workstream 1: Disk Architecture and Storage Stack (all phases complete)
- [x] Workstream 2: Boot and Kernel Delivery (EFISTUB, Limine, UKI — see plan; multi-kernel + Secure Boot signing remain)
- [ ] Workstream 3: Encryption and Early-Boot Integration (design + plumbing)

### Next

- [ ] Workstream 4: Core System Configuration Completion
- [ ] Workstream 5: Configuration Automation and Non-Interactive Installs

### Later

- [ ] Workstream 6: UX and Ecosystem Polish
- [ ] Workstream 7: Extended Configuration, Localization, and Desktop Ecosystem

## Source Coverage

This semantic backlog is derived from TODO markers currently found in:

- `src/app/abort.rs`
- `src/app/automatic_time_sync.rs`
- `src/app/disk_encryption.rs`
- `src/app/disks.rs`
- `src/app/hostname.rs`
- `src/app/install/flow.rs`
- `src/app/kernels.rs`
- `src/app/root_password.rs`
- `src/app/save_configuration.rs`
- `src/app/unified_kernel_images.rs`
- `src/core/services/bootloader.rs`
- `src/core/services/fstab.rs` *(deprecated — replaced by StoragePlan)*
- `src/core/services/mounting.rs` *(deprecated — replaced by StoragePlan)*
- `src/core/services/partitioning.rs` *(deprecated — replaced by StoragePlan)*
- `src/core/services/sysconfig.rs`
- `src/core/services/system.rs`
- `src/core/services/usersetup.rs`
- `src/main.rs`

Marked **roadmap** bullets (e.g. LVM UX in workstream 1, LVM/LUKS stacking, workstream **7**) are captured here for planning; they are not necessarily reflected as `TODO` markers in the tree yet.
