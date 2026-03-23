## archinstall-rs v0.2.3

Release date: 2026-03-23

### Highlights
- **Bootloader reliability**: Major hardening for **EFISTUB**, **Limine**, and **systemd-boot**. UEFI fallback paths (`EFI/BOOT/BOOTX64.EFI`) and better `efibootmgr` handling improve first-boot success when NVRAM entries are missing or unreliable.
- **UKI support expanded**: UKI flow is now wired through install planning for supported bootloaders, with matching sysconfig/kernel preset behavior and improved artifact handling.
- **Secure Boot awareness**: More robust Secure Boot detection (efivars + fallbacks), clearer status in the TUI, and safer policy enforcement for EFISTUB + UKI combinations.

### Improvements & fixes
- **Multi-kernel boot entries** are now generated for selected kernels in bootloader plans.
- **EFISTUB diagnostics** improved (ESP paths, artifact checks, `efibootmgr` stderr/return codes) to make VM/firmware issues easier to troubleshoot.
- **UEFI path and partition lookup fixes** (`PARTN`, escaped loader paths, cleaner NVRAM entry matching/reordering) reduce boot-entry misconfiguration risks.
- **UX/docs updates**: EFISTUB is clearly labeled **experimental**, and boot/UKI guidance in TUI + docs is more explicit.

### Risky / review before relying on it
- Boot pipeline changes in this release are concentrated in **bootloader + UKI + Secure Boot** logic. Do at least one **real or VM install test** (not only dry-run) if you depend on EFISTUB, Limine, or Secure Boot.
- For safety, re-check **partitioning**, **encryption**, and **bootloader** selections in the TUI before confirming install.

### Breaking changes
- None intended for normal TUI users. EFISTUB behavior and labels changed significantly and are now explicitly marked experimental.
