# Semantic TODO Backlog

This backlog groups repository TODOs by feature area and dependency, then orders them by implementation priority.

## Priority Legend

- **P0**: Core install correctness and blockers for major roadmap features.
- **P1**: High-value UX/completeness improvements that unlock smooth usage.
- **P2**: Nice-to-have improvements that can follow after core stabilization.

## What To Implement Next (Project List)

1. **Disk & filesystem foundation (P0)**  
   Complete advanced partitioning + mount/fstab consistency so later features (encryption/UKI/boot) are reliable.
2. **Boot chain expansion (P0)**  
   Add EFISTUB/Limine and UKI integration after disk layout and hooks are stable.
3. **Encryption UX + system hooks (P0/P1)**  
   Finalize passphrase UX and mkinitcpio/boot integration for encrypted installs.
4. **System identity and essential config hardening (P1)**  
   Finish hostname/root/time-sync/locales behavior and additional system config coverage.
5. **Config automation and unattended mode (P1/P2)**  
   Make save/load + CLI config-driven unattended installs first-class.
6. **User experience polish and ecosystem features (P2)**  
   Shell/groups UI, kernel screen completeness, AUR flow polish, abort flow cleanup.

## Workstreams

### 1) Disk Architecture and Storage Stack (P0)

**Goal:** Reliable advanced storage layout as the base for all install modes.

- [ ] Advanced manual partition editor and pre-mounted flow.
- [ ] btrfs subvolume-aware mount layout.
- [ ] btrfs-aware fstab tuning/mount options.
- [ ] LVM + RAID support and advanced btrfs layout options.

**Depends on:** Existing best-effort partitioning path.  
**Unblocks:** Encryption, UKI, non-default bootloaders, unattended reliability.

### 2) Boot and Kernel Delivery (P0)

**Goal:** Complete bootloader matrix and modern kernel boot pipeline.

- [ ] EFISTUB boot entry creation + kernel cmdline generation.
- [ ] Limine setup path (replace current TODO placeholder behavior).
- [ ] Unified Kernel Images generation integrated with mkinitcpio/systemd-boot.

**Depends on:** Stable partition/mount model and system hook generation.  
**Unblocks:** Secure-boot-friendly flow and advanced boot scenarios.

### 3) Encryption and Early-Boot Integration (P0/P1)

**Goal:** Make encrypted installs fully usable end-to-end.

- [ ] Encryption password prompts + partition selection UX.
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

## Suggested Execution Plan

### Now (next sprint)

- [ ] Workstream 1: Disk Architecture and Storage Stack
- [ ] Workstream 2: Boot and Kernel Delivery (start EFISTUB first)
- [ ] Workstream 3: Encryption and Early-Boot Integration (design + plumbing)

### Next

- [ ] Workstream 4: Core System Configuration Completion
- [ ] Workstream 5: Configuration Automation and Non-Interactive Installs

### Later

- [ ] Workstream 6: UX and Ecosystem Polish

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
- `src/core/services/fstab.rs`
- `src/core/services/mounting.rs`
- `src/core/services/partitioning.rs`
- `src/core/services/sysconfig.rs`
- `src/core/services/system.rs`
- `src/core/services/usersetup.rs`
- `src/main.rs`
