# Roadmap

## v0.1.0 — MVP installer (done)

- TUI scaffolding and navigation
- Configuration save/load (TOML)
- Disk selection and partitioning plan preview (info popup)
- Best-effort automatic partitioning (GPT, ESP/BIOS boot, swap, btrfs root, optional LUKS, btrfs subvolume presets)
- Pre-mounted mode when `/mnt` is already prepared; install skips repartitioning
- Partitioning via parted/mkfs/cryptsetup with safety checks
- Abort if target has mounted partitions
- Wipe confirmation when a device already appears partitioned
- Dry-run mode shows the full command plan without applying changes
- Mirrors (reflector regions, custom servers, optional/custom repos) persisted
- Mount filesystems, enable swap, pacstrap base and selected packages
- Generate fstab
- Basic system configuration (locale, timezone, hostname, keymap)
- Networking and time sync (NetworkManager / systemd-timesyncd as selected)
- Root/user setup (passwords, sudoers, optional login manager)
- Bootloaders: systemd-boot (UEFI) and GRUB (UEFI/BIOS), including LUKS-related kernel options when encryption is enabled
- Installation progress view and log viewer
- Manual partitioning partially implemented (simple flows)

## v0.2.0 — Manual partitioning and boot enhancements (done)

- Custom mount points
- Help system with contextual information
- Installation progress visualization
- Profile system (Desktop, Server, Minimal)
- AUR helper integration
- Post-installation script support
- Configuration validation
- Log viewer

## v0.3.0 — Advanced disk and system features

- EFISTUB and Limine bootloaders
- LVM support
- RAID configuration
- Unified kernel image generation
- Multi-language UI
- Advanced partition editor

## v0.4.0 — Advanced features

- Unattended installation mode
- Network installation support
- Configuration templates
- Cloud-init support
- Ansible playbook generation

## v1.0.0 — Production ready

- Comprehensive testing suite
- Complete documentation
- Stable API
- Official Arch Linux repository inclusion (aspirational)
