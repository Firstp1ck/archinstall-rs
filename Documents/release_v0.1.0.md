# Release Notes - archinstall-rs v0.1.0

**Release Date:** 19.09.2025  
**Version:** 0.1.0  
**Status:** MVP (Minimum Viable Product)

## 🎉 What's New

This is the first major release of archinstall-rs, a modern TUI installer for Arch Linux written in Rust. This MVP release provides a solid foundation for Arch Linux installation with an intuitive terminal interface.

## ✨ Key Features

### 🖥️ Modern TUI Interface
- Built with [ratatui](https://github.com/ratatui-org/ratatui) for responsive terminal interface
- Intuitive navigation with arrow keys and keyboard shortcuts
- Real-time configuration preview and validation

### 🔧 Core Installation Features
- **Configuration Management**: Save and load installation configurations in TOML format
- **Disk Management**: Automatic best-effort partitioning with GPT layout
- **Bootloader Support**: systemd-boot (UEFI) and GRUB (UEFI/BIOS)
- **Package Management**: Mirror configuration with reflector integration
- **System Configuration**: Locale, timezone, hostname, and keyboard layout setup
- **User Management**: Root password and user account creation with sudo privileges

### 🛡️ Safety Features
- **Dry-run Mode**: Preview all installation commands before execution
- **Safety Checks**: Abort if target disk has mounted partitions
- **Confirmation Dialogs**: Wipe confirmation for already partitioned devices
- **Installation Progress**: Real-time progress tracking and log viewer

### 📦 Partitioning Strategy
- **GPT Layout**: Modern partitioning with GUID Partition Table
- **EFI System Partition**: 512MiB for UEFI systems
- **BIOS Boot Partition**: 1MiB for GRUB on BIOS systems
- **Swap Partition**: 4GiB automatic sizing (configurable)
- **Root Filesystem**: Btrfs for modern filesystem features
- **Optional LUKS**: Disk encryption support (planned for future releases)

## 📋 System Requirements

### Build Requirements
- Rust 1.70 or later (2024 edition)
- Cargo package manager

### Runtime Requirements
- Arch Linux live environment (USB/ISO)
- UEFI or BIOS system
- Internet connection for package downloads
- Minimum 512MB RAM (1GB+ recommended)
- Minimum 2GB disk space (20GB+ recommended)

## ⚠️ Current Limitations

### What Works
- ✅ 64-bit systems
- ✅ UEFI systems
- ✅ Best-effort partitioning only
- ✅ GRUB and systemd-boot bootloaders
- ✅ NetworkManager for networking
- ✅ Desktop Environment experience mode

### Known Issues
- ⚠️ View may appear stuck at the end, but installation completes successfully
- ⚠️ Installation progress view needs improvement
- ⚠️ Manual partitioning partially implemented. Simple partitioning possible.

### Not Yet Implemented
- ❌ Disk encryption/LUKS (planned for v0.2.0)
- ❌ Custom server/repository configuration
- ❌ Unified Kernel Images (Secure Boot support)
- ❌ RAID and LVM support
- ❌ Advanced bootloader options (EFISTUB, Limine)

## 🎮 Usage

### Basic Installation Flow
1. Boot into Arch Linux live environment
2. Ensure internet connection
3. Download via `curl -LO https://github.com/Firstp1ck/archinstall-rs/releases/download/v0.1.0/archinstall-rs`
4. Check ShaSum: 
- `github_sha_full="<Insert copied sha from the binary>"`
- `echo "${github_sha_full#sha256:} archinstall-rs" | sha256sum -c - >/dev/null && echo "Checksums match!" || echo "Checksums do not match!"`
5. Make executable with `chmod +x archinstall-rs`
6. Run `./archinstall-rs`
7. Navigate through configuration sections
8. Review your setup
9. Start installation

### Navigation
- **Arrow Keys**: Navigate menu and content
- **Enter**: Select/Confirm/Run actions
- **Esc/q**: Close popups or return to menu
- **Ctrl-C**: Quit application

## 🔧 Configuration

The installer supports saving configurations in TOML format for reproducible installations:

```toml
[users]
users = []
additional_packages = []

[locales]
keyboard_layout = "us"
locale_language = "en_US.UTF-8"

[mirrors]
regions = ["United States"]
optional_repos = ["multilib"]

[disks]
mode = "Best-effort partition layout"

[bootloader]
kind = "systemd-boot"

[system]
hostname = "archlinux"
automatic_time_sync = true
timezone = "Europe/London"
```

## 🛠️ Technical Details

### Architecture
- **Language**: Rust (2024 edition)
- **TUI Framework**: ratatui
- **Configuration**: TOML format
- **Partitioning**: parted, mkfs, cryptsetup
- **Package Management**: pacstrap, pacman

### Project Structure
- Modular design with separate modules for each installation section
- Clear separation between UI, logic, and data layers
- Type-safe configuration management
- Comprehensive error handling

## 🐛 Bug Reports

If you encounter any issues, please report them on [GitHub Issues](https://github.com/Firstp1ck/archinstall-rs/issues) with:
- Clear description of the problem
- Steps to reproduce
- System information
- Relevant configuration files
- Error messages or logs

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- The Arch Linux community for excellent documentation
- The [ratatui](https://github.com/ratatui-org/ratatui) team for the TUI framework
- The original [archinstall](https://github.com/archlinux/archinstall) Python project for inspiration
- All contributors who help improve this project

## ⚠️ Disclaimer

This installer modifies disk partitions and system configurations. Always backup important data before using. The developers are not responsible for any data loss or system damage that may occur.

---

**Made with ❤️ and 🦀 for the Arch Linux community**
