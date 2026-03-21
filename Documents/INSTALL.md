# Installation

## Requirements

### Build

- Rust (edition 2024) on a recent stable toolchain (1.84+ recommended)
- Cargo

### Runtime (typical use)

- Arch Linux live environment (USB/ISO)
- UEFI or BIOS firmware
- Network for package downloads
- Minimum about 512 MiB RAM (1 GiB+ recommended)
- Minimum about 2 GiB disk space (20 GiB+ recommended for a comfortable desktop)

## Quick install from the live ISO (recommended)

From the Arch live ISO with networking, the [release install script](https://github.com/Firstp1ck/archinstall-rs/releases/latest) downloads the binary and bundled example configs, verifies them, and runs the installer:

```bash
curl -fsSL https://github.com/Firstp1ck/archinstall-rs/releases/latest/download/install.sh | bash
```

Pass options through to the installer (for example `--help`) with:

```bash
curl -fsSL https://github.com/Firstp1ck/archinstall-rs/releases/latest/download/install.sh | bash -s -- --help
```

To pin a version, replace `latest` with a tag path, for example `releases/download/vX.X.X/install.sh`.

### Optional: keyboard layout (ISO TTY)

```bash
loadkeys de_CH-latin1
```

## `boot.sh` (ISO TTY + minimal GUI)

From a clone of this repository:

```bash
git clone https://github.com/Firstp1ck/archinstall-rs.git
cd archinstall-rs
./boot.sh
```

Notes:

- Prepares a minimal GUI (Wayland cage + foot, or Xorg + xterm), prints progress, and logs to the path shown on start.
- If no prebuilt binary is present, `boot.sh` can download the latest release automatically.
- If a GUI cannot be prepared, the helper exits with an error (no half-started state).

Dry-run (no disk changes) with verbose logs:

```bash
ARCHINSTALL_DRY_RUN_LOG=/tmp/ai-dry.log ./boot.sh INSTALLER_FLAGS="--dry-run --debug"
```

## Pre-built binary (script)

Same as the quick install one-liner above; from the project root you can also run `./boot.sh` after obtaining the tree.

## Build from source

Install toolchain dependencies:

```bash
pacman -Sy rustup git gcc base-devel
rustup install stable
```

Clone and run:

```bash
git clone https://github.com/Firstp1ck/archinstall-rs.git
cd archinstall-rs
cargo build
cargo run
```

## VM boot parameters

In the ISO boot menu, press `e` and adjust the kernel command line if you need more cow space, for example:

```bash
linux /arch/boot/x86_64/vmlinuz-linux archisobasedir=arch cow_spacesize=8G
```
