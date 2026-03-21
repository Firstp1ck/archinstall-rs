# Storage Layout Implementation Plan

## Goal

Build a reliable advanced storage layout foundation that works across automatic installs, manual partitioning, pre-mounted installs, and future encrypted / UKI / LVM / RAID flows.

## Why This Comes First

The current storage pipeline is split across three services — `PartitioningService`, `MountingService`, and `FstabService` — but each makes independent, hardcoded assumptions about a fixed three-partition layout. This blocks every downstream feature.

## Current Architecture (as-is)

### Data flow today

```
AppState (disks_mode_index, swap_enabled, disk_encryption_type_index, disks_partitions)
   │
   ├─► PartitioningService::build_plan()   → branches on disks_mode_index (0=auto, 1=manual)
   ├─► MountingService::build_plan()       → ALWAYS assumes auto layout (p1/p2/p3)
   ├─► FstabService::build_checks_and_fstab() → ALWAYS assumes auto layout (p1/p2/p3)
   └─► BootloaderService::build_plan()     → uses findmnt dynamically (partially decoupled)
```

### Hardcoded partition number assumptions

| Service | Assumption | Location |
|---------|-----------|----------|
| `MountingService` | root = partition 3 | `partition_path(device, 3)` in `build_plan` |
| `MountingService` | ESP = partition 1 | `partition_path(device, 1)` in `build_plan` |
| `MountingService` | swap = partition 2 | `partition_path(device, 2)` in `build_plan` |
| `MountingService` | LUKS root = `/dev/mapper/cryptroot` | hardcoded string literal |
| `FstabService` | p1/p2/p3 via `format!("{device}1")` etc. | `build_checks_and_fstab` |
| `PartitioningService` (auto) | ESP=p1, swap=p2, root=p3 | `build_automatic_partition_plan` |
| `PartitioningService` (auto) | LUKS on `{device}3` → `cryptroot` | same function |

### Manual mode gap

`PartitioningService::build_plan` correctly branches on `disks_mode_index == 1` and iterates `disks_partitions`. However:

- `MountingService::build_plan` does **not** check `disks_mode_index` at all.
- `FstabService::build_checks_and_fstab` does **not** check `disks_mode_index` at all.
- Result: manual installs generate **wrong** mount and fstab commands.

### Pre-mounted mode gap

`disks_mode_index == 2` ("Pre-mounted configuration") has:

- A menu label in `src/app/disks.rs` line 33
- A config string in `src/app/config/io.rs` line 79
- Zero implementation: no detection, no validation, no install flow support.

### Encryption gap

- Encryption is checked via a single global flag (`disk_encryption_type_index == 1`).
- The passphrase is not piped non-interactively in the partition plan (`cryptsetup luksFormat` requires stdin passphrase for unattended use).
- Per-partition encryption (`DiskPartitionSpec.encrypt`) is stored but never consumed by any service.

### btrfs gap

- Automatic mode always creates a single btrfs filesystem on root.
- No subvolume creation, no subvolume-specific mount options, no subvolume-aware fstab entries.
- `DiskPartitionSpec.mount_options` exists but is never consumed.

## Existing Types to Preserve and Extend

### `DiskPartitionSpec` (in `src/core/types.rs`)

```rust
pub struct DiskPartitionSpec {
    pub name: Option<String>,         // device path
    pub role: Option<String>,         // BOOT, SWAP, ROOT, OTHER
    pub fs: Option<String>,           // btrfs, ext4, fat32, ...
    pub start: Option<String>,        // byte offset
    pub size: Option<String>,         // byte size
    pub flags: Vec<String>,           // esp, bios_grub, ...
    pub mountpoint: Option<String>,   // /, /boot, /home, ...
    pub mount_options: Option<String>,// compress=zstd, ...
    pub encrypt: Option<bool>,        // per-partition encryption
}
```

This is a useful UI-level spec. It remains the UI data model. The normalized `StoragePlan` is derived from it for execution.

### `ConfigPartition` (in `src/app/config/types.rs`)

Mirrors `DiskPartitionSpec` for TOML serialization. Keep in sync.

## Implementation Phases

### Phase 1: Introduce the storage model and planner — COMPLETED

**Status:** Done. Merged and passing all tests.

**What was built:**

New module `src/core/storage/` with two files:

- **`mod.rs`** — All plan types and command generation:
  - `StoragePlan`, `PlannedDevice`, `PlannedPartition`, `PlannedMount` — normalized storage model
  - `StorageMode` (Automatic, Manual, PreMounted), `DiskLabel` (Gpt, Msdos)
  - `PartitionRole` (Esp, BiosBoot, Swap, Root, Home, Var, Other)
  - `PartitionFlag` (Esp, BiosGrub) with `as_parted_str()` helpers
  - `FilesystemSpec`, `EncryptionSpec` (Luks2), `SubvolumeSpec`
  - `ValidationError` with Display impl
  - `StoragePlan::partition_commands()` — generates partitioning shell commands
  - `StoragePlan::mount_commands()` — generates mount shell commands
  - `StoragePlan::fstab_check_commands()` — generates fstab validation + genfstab
  - `StoragePlan::validate()` — checks root exists, ESP/BiosBoot present, no dup mountpoints, no overlap
  - `StoragePlan::root_device_path()`, `esp_device_path()`, `has_encryption()` — accessors for downstream consumers
  - `StoragePlan::partition_path()` — nvme-aware device+number path builder

- **`planner.rs`** — Compilation from AppState to StoragePlan:
  - `StoragePlanner::compile(state) -> Result<StoragePlan, Vec<ValidationError>>` — dispatcher
  - `compile_automatic()` — produces the exact layout currently generated by the three old services
  - `compile_manual()` — reads `disks_partitions`, resolves numbers/roles/mountpoints/encryption from specs
  - `compile_pre_mounted()` — stub returning empty plan (Phase 5)
  - `bytes_to_parted_unit()` — utility matching existing conversion logic

**Key design decisions made during implementation:**

- `PlannedPartition.start` and `end` use `String` (parted unit format like `"1025MiB"`, `"100%"`) rather than raw `u64` bytes. This avoids double conversion and keeps command generation simple.
- biosboot partitions skip both the filesystem hint in `parted mkpart` and the format command, matching existing behavior.
- Root mount is always inserted at position 0 in the mounts list to ensure correct mount ordering.
- Mount commands for `/mnt/boot` inject `modprobe vfat/fat` before mounting, matching existing behavior.

**Test coverage (15 tests, all passing):**

- Partition commands: UEFI×swap, UEFI×swap×LUKS, UEFI×no-swap, BIOS×swap, BIOS×swap×LUKS
- Mount commands: UEFI×swap, UEFI×swap×LUKS, BIOS×swap, UEFI×no-swap
- Fstab commands: UEFI×swap, UEFI×swap×LUKS
- Validation: missing root detection, duplicate mountpoint detection
- Utilities: nvme partition path, plan accessors (root/ESP/encryption)

**Verification:** `cargo check` clean, `cargo clippy` clean, `cargo test` 41/41 passing (15 new × 2 targets + 11 existing integration tests).

### Phase 2: Wire the planner into the install flow — COMPLETED

**Status:** Done. All tests passing (41/41), clippy clean.

**What was done:**

In `src/app/install/flow.rs`:

- [x] Replaced imports: removed `FstabService`, `MountingService`, `PartitioningService`; added `StoragePlanner`.
- [x] `build_install_sections()` now calls `StoragePlanner::compile(self)` at the top. On error, returns an "Error" section with the validation messages instead of proceeding.
- [x] Replaced `PartitioningService::build_plan(self, target).commands` with `storage_plan.partition_commands()`.
- [x] Replaced `MountingService::build_plan(self, target).commands` with `storage_plan.mount_commands()`.
- [x] Replaced `FstabService::build_checks_and_fstab(self, target).commands` with `storage_plan.fstab_check_commands()`.
- [x] `validate_install_requirements()` now runs `StoragePlanner::compile(self)` as a pre-flight check; any validation errors are appended to the issues list.
- [x] `start_install_flow()` skips `select_target_and_run_prechecks()` when `disks_mode_index == 2` (pre-mounted mode), passing an empty target string instead.

Old services deprecated:

- [x] `src/core/services/partitioning.rs` — added deprecation header, replaced TODO with NOTE referencing StoragePlanner.
- [x] `src/core/services/mounting.rs` — added deprecation header, replaced TODO with NOTE.
- [x] `src/core/services/fstab.rs` — added deprecation header, replaced TODO with NOTE.

**Deferred:** Extending `BootloaderService::build_plan` to accept `&StoragePlan`. The current `findmnt`-based approach inside chroot continues to work. Can be done in a later pass.

**Verification:** `cargo check` clean, `cargo clippy` clean, `cargo test` 41/41 passing.

### Phase 3: Add manual mode to the planner — COMPLETED

**Status:** Done. All tests passing (61/61), clippy clean.

**What was done:**

- [x] Added manual-specific pre-validation in `compile_manual()`:
  - Rejects empty partition lists with a clear error.
  - Every spec must have: role, filesystem (except BiosBoot), start position, size.
  - Non-swap/non-biosboot partitions require a mountpoint.
  - All errors collected before conversion, so the user sees all issues at once.
  - Structural validation (root exists, no duplicate mountpoints, no overlaps) runs via `plan.validate()` after conversion.

- [x] In `src/input/screens/disks.rs`, when advancing past the Disks screen in manual mode (`disks_mode_index == 1`), `StoragePlanner::compile()` is called. Validation errors are shown as an info popup, blocking navigation until the layout is valid.

- [x] In `src/input/popup/enter.rs`, `finalize_manual_partition` now rejects `OTHER` partitions with an empty mountpoint — shows an info popup and returns early without creating the spec.

- [x] Simplified the duplicate `if` arms for mode 0/1 in `handle_enter_disks` (clippy `if_same_then_else` fix).

**Test coverage (10 new tests, all passing):**

- `test_manual_esp_root` — ESP+root layout compiles, correct mount order, correct mkfs commands.
- `test_manual_esp_swap_root` — ESP+swap+root: swap mount, swapon in commands.
- `test_manual_bios_root` — BiosBoot+root: no format for biosboot, single mount.
- `test_manual_root_with_encryption` — LUKS on root: cryptsetup commands, mapper path in mounts.
- `test_manual_rejects_empty_partitions` — Empty `disks_partitions` returns error.
- `test_manual_rejects_missing_role` — Spec without role returns error.
- `test_manual_rejects_missing_start` — Spec without start returns error.
- `test_manual_rejects_missing_mountpoint` — Non-swap spec without mountpoint returns error.
- `test_manual_swap_no_mountpoint_ok` — Swap without mountpoint is allowed.
- `test_manual_mount_options_propagate` — `mount_options` string flows into `PlannedMount.options` and appears in mount commands.

**Deferred:** Mixed existing + created partitions (skip `partition_commands` for existing entries) — this requires runtime partition detection and is better suited as a follow-up within Phase 3 or as part of a future "keep existing partitions" feature.

**Verification:** `cargo check` clean, `cargo clippy` clean, `cargo test` 61/61 passing (25 new × 2 targets + 11 integration).

### Phase 4: Add btrfs subvolume support — COMPLETED

**Status:** Done. All tests passing (71/71), clippy clean.

**What was done:**

- [x] Added `BtrfsSubvolumePreset` enum to `src/core/storage/mod.rs` with three variants:
  - `Flat` — no subvolumes (current behavior, default)
  - `Standard` — `@`, `@home`, `@snapshots` with `compress=zstd,noatime`
  - `Extended` — `@`, `@home`, `@var_log`, `@snapshots` with `compress=zstd,noatime`
  - Each preset has a `subvolumes()` method returning `Vec<SubvolumeSpec>` and a `label()` for UI display.

- [x] Added `btrfs_subvolume_preset` field (usize: 0/1/2) to `AppState` in `src/core/state.rs`, defaulting to 0 (Flat).

- [x] Updated `compile_automatic()` in `src/core/storage/planner.rs`:
  - Resolves the preset from `state.btrfs_subvolume_preset`.
  - Populates `subvolumes` on the root `PlannedPartition` when btrfs.
  - When subvolumes are present, replaces the single root `PlannedMount` with per-subvolume mounts (each with `subvolume: Some(name)` and preset mount options).
  - Root subvolume (`@`) mount is sorted first.

- [x] Extended `partition_commands()` in `src/core/storage/mod.rs`:
  - After `mkfs.btrfs`, if the partition has subvolumes: temp mount to `/mnt`, `btrfs subvolume create /mnt/<name>` for each, then `umount /mnt`.

- [x] `mount_commands()` already supported `subvolume` field — now populated by the planner. Generates `mount -o subvol=@,compress=zstd,noatime ...` for each subvolume.

- [x] `fstab_check_commands()` validates each subvolume mount via `mountpoint -q` checks.

- [x] Config persistence:
  - Added `btrfs_subvolume_preset` (String: "flat"/"standard"/"extended") to `ConfigDisks` in `src/app/config/types.rs`.
  - Wired save/load in `src/app/config/io.rs`.

**Deferred:** UI popup for selecting the btrfs preset on the Disks screen — the field is persisted and functional, but no TUI selector exists yet. Can be added as a minor UX follow-up.

**Test coverage (5 new tests, all passing):**

- `test_btrfs_flat_unchanged` — Flat preset produces no subvolumes, no subvol commands.
- `test_btrfs_standard_subvolumes` — Standard: 3 subvolumes, correct partition + mount + fstab commands.
- `test_btrfs_extended_subvolumes` — Extended: 4 subvolumes including `@var_log`.
- `test_btrfs_standard_with_luks` — Standard + LUKS: subvol creation uses mapper device.
- `test_btrfs_fstab_checks_subvolume_mounts` — Fstab checks each subvolume mountpoint.

**Verification:** `cargo check` clean, `cargo clippy` clean, `cargo test` 71/71 passing.

### Phase 5: Implement pre-mounted mode — COMPLETED

**Status:** Done. All tests passing (87/87), clippy clean.

**What was done:**

- [x] Implemented `compile_pre_mounted()` in `src/core/storage/planner.rs`:
  - Detects current mounts under `/mnt` via `findmnt -J -R --target /mnt`.
  - Recursively collects mount entries (including nested children) into `PlannedMount` list.
  - Detects subvolume mounts from `subvol=` options in mount flags.
  - Detects active swap devices via `swapon --raw --noheadings`.
  - Returns `StoragePlan` with empty `devices` (no partitioning) and detected `mounts`.

- [x] Pre-return validation in `compile_pre_mounted()`:
  - Checks `/mnt` is mounted (root).
  - Checks `/mnt/boot` or `/mnt/efi` is mounted when UEFI firmware detected.
  - Checks swap is active if `state.swap_enabled`.
  - Returns all errors at once so the user sees all issues.

- [x] `partition_commands()` returns empty for deviceless plans (verified — loops over `self.devices` which is empty).

- [x] `mount_commands()` returns empty vec for `PreMounted` mode (early return guard).

- [x] `fstab_check_commands()` for `PreMounted` mode:
  - Skips blkid filesystem-type checks (we didn't create these filesystems).
  - Keeps mountpoint validation (`mountpoint -q`) for each non-swap mount.
  - Keeps swap-active check.
  - Still runs `genfstab -U /mnt >> /mnt/etc/fstab`.

- [x] `build_install_sections()` skips pre-cleanup (swapoff/umount) for pre-mounted mode.

- [x] `validate_install_requirements()` skips "target device not selected" check for mode 2.

- [x] `start_install_flow()` already skipped `select_target_and_run_prechecks()` for mode 2 (Phase 2).

- [x] Disks screen input (`src/input/screens/disks.rs`):
  - Mode 2 selection runs `StoragePlanner::compile()` and shows detected mount summary in an info popup.
  - Continue button validates pre-mounted plan (same as manual mode) before advancing.

- [x] Info panel (`src/render/sections/info/disks.rs`):
  - Pre-mounted mode shows detected mounts under /mnt via live `findmnt` query.
  - Shows active swap devices via `swapon`.
  - Shows helpful message when no mounts detected.

**Test coverage (8 new tests, all passing):**

- `test_pre_mounted_no_partition_commands` — Empty partition commands for deviceless plan.
- `test_pre_mounted_no_mount_commands` — Empty mount commands (already mounted).
- `test_pre_mounted_fstab_mountpoint_checks` — Mountpoint checks present, no blkid, genfstab runs.
- `test_pre_mounted_fstab_with_swap` — Swap active check included in fstab commands.
- `test_pre_mounted_validation_missing_root` — Missing /mnt mount triggers validation error.
- `test_pre_mounted_validation_root_ok` — Plan with /mnt mount validates cleanly.
- `test_pre_mounted_with_subvolumes` — Subvolume mounts validate and produce correct fstab checks.
- `test_pre_mounted_accessors` — `root_device_path()`, `esp_device_path()`, `has_encryption()` work for pre-mounted plans.

**Verification:** `cargo check` clean, `cargo clippy` clean, `cargo test` 87/87 passing (38 new × 2 targets + 11 integration).

Files touched:

- `src/core/storage/planner.rs` — `compile_pre_mounted()` implementation + `collect_findmnt_mounts()` helper + 8 new tests
- `src/core/storage/mod.rs` — `PreMounted` guards in `mount_commands()` and `fstab_check_commands()`
- `src/app/install/flow.rs` — Skip pre-cleanup and device-check for pre-mounted mode
- `src/input/screens/disks.rs` — Mode 2 enter handler and Continue validation
- `src/render/sections/info/disks.rs` — Live mount detection display for pre-mounted mode

Acceptance criteria:

- [x] Pre-mounted mode detects existing mounts and proceeds through install flow.
- [x] Validation errors clearly explain missing mounts.
- [x] No partitioning or formatting commands are generated.

### Phase 6: Prepare for LVM and RAID extensibility — COMPLETED

**Status:** Done. All tests passing (105/105), clippy clean.

**What was done:**

- [x] Added volume layer types to `src/core/storage/mod.rs`:
  - `LvmSpec` — LVM logical volume (vg_name, lv_name, size expression).
  - `RaidSpec` — software RAID (level, member devices, md device name).
  - `VolumeLayer` enum — `Luks(EncryptionSpec)`, `Lvm(LvmSpec)`, `Raid(RaidSpec)`.
  - `DeviceStack` — ordered transformation chain from base device through layers to final device.
  - `DeviceStack::final_device_path()` — resolves the final device path by walking layers.
  - `DeviceStack::setup_commands()` — stub returning empty vec (LVM/RAID command generation deferred).

- [x] Added `stacks: Vec<DeviceStack>` field to `StoragePlan`. All existing plan constructors updated with `stacks: vec![]` — zero runtime behavior changes.

- [x] Model can express layered device stacks:
  - `disk → partition → LUKS → LVM VG/LV → filesystem → mount` (LVM-on-LUKS)
  - `disk → partition → LVM → LUKS → filesystem → mount` (LUKS-on-LVM)
  - `disk → partition × N → RAID → filesystem → mount` (software RAID mirror)
  - `disk → partition → (no layers) → filesystem → mount` (passthrough, existing behavior)

**Test coverage (9 new tests, all passing):**

- `test_device_stack_luks_final_path` — LUKS layer resolves to `/dev/mapper/<name>`.
- `test_device_stack_lvm_final_path` — LVM layer resolves to `/dev/<vg>/<lv>`.
- `test_device_stack_raid_final_path` — RAID layer resolves to `/dev/md/<name>`.
- `test_device_stack_luks_on_lvm` — Multi-layer: LVM then LUKS.
- `test_device_stack_lvm_on_luks` — Multi-layer: LUKS then LVM (canonical encrypted layout).
- `test_lvm_on_luks_full_plan` — Complete LVM-on-LUKS plan with ESP, root, home, and swap LVs. Verifies plan construction, accessors, encryption detection, and that existing partition_commands still work.
- `test_raid1_plan_model` — RAID-1 mirror across two devices. Verifies multi-device plan and RAID member extraction.
- `test_device_stack_no_layers` — Empty layers returns base path unchanged.
- `test_existing_plans_have_empty_stacks` — Automatic, BIOS, and PreMounted plans have empty stacks (no regression).

Files touched:

- `src/core/storage/mod.rs` — `LvmSpec`, `RaidSpec`, `VolumeLayer`, `DeviceStack` types; `stacks` field on `StoragePlan`
- `src/core/storage/planner.rs` — `stacks: vec![]` in all plan constructors; 9 new tests

Acceptance criteria:

- [x] No refactoring needed later to add LVM or RAID.
- [x] The model can express layered device stacks.
- [x] No runtime behavior changes.

## Recommended Delivery Order

| Order | Phase | Status | Risk | Effort |
|-------|-------|--------|------|--------|
| 1 | Phase 1: Storage model + planner | **DONE** | — | — |
| 2 | Phase 2: Wire into install flow | **DONE** | — | — |
| 3 | Phase 3: Manual mode in planner | **DONE** | — | — |
| 4 | Phase 4: btrfs subvolumes | **DONE** | — | — |
| 5 | Phase 5: Pre-mounted mode | **DONE** | — | — |
| 6 | Phase 6: LVM/RAID extensibility | **DONE** | — | — |

All phases complete. The storage layout foundation is ready for downstream features.

## File Touchpoints (complete list)

### New files (created in Phase 1)

- `src/core/storage/mod.rs` — plan types and command generation
- `src/core/storage/planner.rs` — AppState → StoragePlan compiler + tests

### Modified files (Phase 1)

- `src/core/mod.rs` — added `pub mod storage`

### Modified files (Phase 2)

- `src/app/install/flow.rs` — replaced service calls with plan-driven generation
- `src/core/services/partitioning.rs` — deprecated, added NOTE replacing TODO
- `src/core/services/mounting.rs` — deprecated, added NOTE replacing TODO
- `src/core/services/fstab.rs` — deprecated, added NOTE replacing TODO

### Modified files (Phase 3)

- `src/core/storage/planner.rs` — manual-specific pre-validation + 10 new tests
- `src/input/screens/disks.rs` — validation gate on advance from Disks screen
- `src/input/popup/enter.rs` — mountpoint enforcement for non-swap partitions

### Modified files (Phase 4)

- `src/core/storage/mod.rs` — `BtrfsSubvolumePreset` enum, subvolume creation in `partition_commands()`
- `src/core/storage/planner.rs` — subvolume population in `compile_automatic()`, 5 new tests
- `src/core/state.rs` — `btrfs_subvolume_preset` field
- `src/app/config/types.rs` — `btrfs_subvolume_preset` in `ConfigDisks`
- `src/app/config/io.rs` — save/load preset

### Modified files (Phase 5)

- `src/core/storage/planner.rs` — `compile_pre_mounted()` impl, `collect_findmnt_mounts()` helper, 8 new tests
- `src/core/storage/mod.rs` — `PreMounted` guards in `mount_commands()` and `fstab_check_commands()`
- `src/app/install/flow.rs` — skip pre-cleanup and device-check for pre-mounted mode
- `src/input/screens/disks.rs` — mode 2 enter handler and Continue validation
- `src/render/sections/info/disks.rs` — live mount detection display for pre-mounted mode

### Modified files (Phase 6)

- `src/core/storage/mod.rs` — `LvmSpec`, `RaidSpec`, `VolumeLayer`, `DeviceStack` types; `stacks` field on `StoragePlan`
- `src/core/storage/planner.rs` — `stacks: vec![]` in all plan constructors; 9 new tests

### Modified files (follow-up items)

- `src/core/storage/mod.rs` — `setup_commands()` for LVM/RAID/LUKS, `stack_setup_commands()`, enhanced `has_encryption()`
- `src/core/services/bootloader.rs` — `&StoragePlan` parameter, LUKS-aware boot options
- `src/app/install/flow.rs` — stack setup section, bootloader call update
- `src/core/types.rs` — `BtrfsSubvolumePreset` popup kind
- `src/app/disks.rs` — btrfs preset row + popup opener
- `src/input/screens/disks.rs` — focus range 0..=4, btrfs preset handler
- `src/input/popup/enter.rs` — `BtrfsSubvolumePreset` handler
- `src/render/popup/mod.rs` — popup title/size/search config
- `src/render/sections/info/disks.rs` — btrfs preset in info panel
- `tests/logic.rs` — bootloader test update + 2 new LUKS tests

### Deprecated (Phase 2)

- `src/core/services/partitioning.rs` — replaced by `StoragePlan::partition_commands()`. Kept for integration test compat.
- `src/core/services/mounting.rs` — replaced by `StoragePlan::mount_commands()`. Kept for integration test compat.
- `src/core/services/fstab.rs` — replaced by `StoragePlan::fstab_check_commands()`. Kept for integration test compat.

## Definition of Done

This goal is complete when:

- [x] Normalized storage model exists with full type coverage
- [x] Automatic mode compiles to a validated plan and generates correct commands
- [x] Manual mode compiles to a validated plan (planner path exists)
- [x] Plan validation catches missing root, missing boot partition, duplicate mounts, overlaps
- [x] 15 unit tests cover all UEFI/BIOS × swap × LUKS combinations
- [x] Install flow uses the planner instead of hardcoded services (Phase 2)
- [x] Manual installs produce correct mount and fstab commands end-to-end (Phase 3)
- [x] Pre-mounted mode detects existing mounts and installs without repartitioning (Phase 5)
- [x] btrfs subvolumes and custom mount options are supported (Phase 4)
- [x] The storage model can represent future encryption, UKI, LVM, and RAID work (Phase 6)
- [x] Existing automatic installs are not regressed (command output matches pre-refactor)

## Follow-up Items — COMPLETED

All three deferred follow-ups from Phases 2, 4, and 6 have been implemented:

### 1. DeviceStack command generation — DONE

Implemented `DeviceStack::setup_commands()` in `src/core/storage/mod.rs`:

- **LUKS layers**: `cryptsetup luksFormat <device>` + `cryptsetup open <device> <mapper>`
- **LVM layers**: `pvcreate`, `vgcreate`, `lvcreate` (with `-l` for percentage sizes, `-L` for absolute)
- **RAID layers**: `mdadm --create /dev/md/<name> --level=<N> --raid-devices=<count> <members...>`
- **Filesystem**: `mkfs.*` on the final device when `FilesystemSpec` is present.
- Added `StoragePlan::stack_setup_commands()` to aggregate all stack commands.
- Wired into install flow as a "Volume stack setup (LVM/RAID)" section between partitioning and mounting.
- 10 new tests: LUKS-only, LVM-only, LVM-percent-size, RAID, LVM-on-LUKS chain, no-filesystem, no-layers, swap-via-LVM, aggregation, encryption-via-stacks.

### 2. Bootloader service integration — DONE

Updated `BootloaderService::build_plan()` in `src/core/services/bootloader.rs`:

- Signature changed to accept `&StoragePlan` alongside `&AppState` and device path.
- **systemd-boot**: Generates LUKS-aware kernel options (`rd.luks.name=<UUID>=<mapper> root=<mapper> rw`) when `storage_plan.has_encryption()` is true. Falls back to simple `root=UUID=<uuid> rw` when unencrypted.
- **GRUB**: Injects `GRUB_CMDLINE_LINUX` into `/etc/default/grub` with LUKS parameters before running `grub-mkconfig`.
- Updated call site in `build_install_sections()` to pass `&storage_plan`.
- 2 new integration tests: LUKS systemd-boot (`rd.luks.name` present), LUKS GRUB (`GRUB_CMDLINE_LINUX` injection).

### 3. Btrfs preset UI — DONE

Added TUI selector for btrfs subvolume presets on the Disks screen:

- New `PopupKind::BtrfsSubvolumePreset` variant in `src/core/types.rs`.
- New row "Btrfs Subvolumes: <preset>" at focus index 3 on the Disks screen (between mode options and Continue).
- Row is dimmed (DarkGray) when not in automatic mode (mode 0); interactive only for automatic.
- Pressing Enter opens a popup with three options: Flat, Standard, Extended.
- Selecting a preset updates `state.btrfs_subvolume_preset`.
- Info panel for mode 0 displays current btrfs subvolume preset.
- Focus indices adjusted: mode options (0-2), btrfs preset (3), Continue (4).
- Popup sized as compact (small popup group), search bar hidden.

Files touched:

- `src/core/storage/mod.rs` — `setup_commands()` implementation, `mkfs_command()` helper, `stack_setup_commands()`, enhanced `has_encryption()`
- `src/core/storage/planner.rs` — updated existing test assertions, 10 new tests
- `src/core/services/bootloader.rs` — new `&StoragePlan` parameter, LUKS-aware boot options
- `src/app/install/flow.rs` — stack setup section, updated bootloader call
- `src/core/types.rs` — `BtrfsSubvolumePreset` popup kind
- `src/app/disks.rs` — btrfs preset row in `draw_disks()`, `open_btrfs_subvolume_preset_popup()`
- `src/input/screens/disks.rs` — focus range 0..=4, btrfs preset enter handler
- `src/input/popup/enter.rs` — `BtrfsSubvolumePreset` selection handler
- `src/render/popup/mod.rs` — popup title, size, search-bar hiding
- `src/render/sections/info/disks.rs` — btrfs preset display in info panel
- `tests/logic.rs` — updated bootloader test, 2 new LUKS bootloader tests

Verification: `cargo check` clean, `cargo clippy` clean, `cargo test` 127/127 passing (57 lib × 2 + 13 integration).
