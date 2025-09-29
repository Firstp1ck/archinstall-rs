#!/bin/sh
# install-limine.sh
# Helper script used by the installer to deploy Limine files.
# This script does NOT contain the Limine binaries. It expects the Limine package
# to be installed on the target system (files available in /usr/share/limine).
#
# Usage:
#   install-limine.sh TARGET_MOUNT BOOT_MOUNT [EFI_MOUNT] PARENT_DEV PARTNUM IS_USB CHROOT
#
# Where:
#  - TARGET_MOUNT: root mount of the installed system (e.g. /mnt)
#  - BOOT_MOUNT: mountpoint of the boot partition inside TARGET_MOUNT (e.g. /mnt/boot)
#  - EFI_MOUNT: (optional) mountpoint of the ESP (e.g. /mnt/efi). Empty/string "" for none.
#  - PARENT_DEV: parent device path (e.g. /dev/sda). Required for efibootmgr or bios-install.
#  - PARTNUM: partition number of the ESP (numeric). Required for efibootmgr.
#  - IS_USB: "1" if target device is a USB stick (write to EFI/BOOT), else "0".
#  - CHROOT: "1" to run limine bios-install inside arch-chroot TARGET_MOUNT, else "0".
#
# The script tries to reproduce the behaviour used in archinstall's Python installer.

set -eu

TARGET=${1:-}
BOOT_MOUNT=${2:-}
EFI_MOUNT=${3:-}
PARENT_DEV=${4:-}
PARTNUM=${5:-}
IS_USB=${6:-0}
CHROOT=${7:-0}

if [ -z "$TARGET" ] || [ -z "$BOOT_MOUNT" ]; then
  echo "Usage: $0 TARGET_MOUNT BOOT_MOUNT [EFI_MOUNT] PARENT_DEV PARTNUM IS_USB CHROOT"
  exit 2
fi

# Check for limine package files
if [ ! -e /usr/share/limine/BOOTX64.EFI ] && [ ! -e /usr/share/limine/limine-bios.sys ]; then
  echo "Limine package files not found in /usr/share/limine. Please install the 'limine' package in the target first."
  exit 3
fi

# Detect firmware type (UEFI vs BIOS)
if [ -d /sys/firmware/efi ]; then
  # UEFI path
  if [ -z "$EFI_MOUNT" ]; then
    echo "UEFI detected but no EFI_MOUNT provided"
    exit 4
  fi

  if [ "$IS_USB" = "1" ]; then
    EFI_DIR="$EFI_MOUNT/EFI/BOOT"
  else
    EFI_DIR="$EFI_MOUNT/EFI/limine"
  fi

  mkdir -p "$EFI_DIR"

  cp -v /usr/share/limine/BOOTIA32.EFI "$EFI_DIR" || true
  cp -v /usr/share/limine/BOOTX64.EFI "$EFI_DIR" || true

  echo "Copied EFI binaries to: $EFI_DIR"

  # Optionally create an NVRAM entry (only when not USB and parent device + partnum provided)
  if [ "$IS_USB" != "1" ] && [ -n "$PARENT_DEV" ] && [ -n "$PARTNUM" ]; then
    if [ -r /sys/firmware/efi/fw_platform_size ]; then
      efi_bitness=$(cat /sys/firmware/efi/fw_platform_size | tr -d '\n') || efi_bitness=64
    else
      efi_bitness=64
    fi

    if [ "$efi_bitness" = "64" ]; then
      loader_path="/EFI/limine/BOOTX64.EFI"
    else
      loader_path="/EFI/limine/BOOTIA32.EFI"
    fi

    if command -v efibootmgr >/dev/null 2>&1; then
      echo "Creating efibootmgr entry: loader=$loader_path disk=$PARENT_DEV part=$PARTNUM"
      efibootmgr --create --disk "$PARENT_DEV" --part "$PARTNUM" --label "Arch Linux Limine Bootloader" --loader "$loader_path" --unicode --verbose || true
    else
      echo "efibootmgr not present; skipping NVRAM entry creation"
    fi
  fi
else
  # BIOS path
  echo "Installing Limine for BIOS (legacy)"
  BOOT_LIMINE_DIR="$TARGET/boot/limine"
  mkdir -p "$BOOT_LIMINE_DIR"

  if [ -e /usr/share/limine/limine-bios.sys ]; then
    cp -v /usr/share/limine/limine-bios.sys "$BOOT_LIMINE_DIR/" || true
  fi

  if [ -z "$PARENT_DEV" ]; then
    echo "PARENT_DEV required for bios-install"
    exit 5
  fi

  if [ "$CHROOT" = "1" ]; then
    if command -v arch-chroot >/dev/null 2>&1; then
      echo "Running limine bios-install inside chroot"
      arch-chroot "$TARGET" limine bios-install "$PARENT_DEV" || true
    else
      echo "arch-chroot not found; cannot run bios-install in chroot"
      exit 6
    fi
  else
    if command -v limine >/dev/null 2>&1; then
      limine bios-install "$PARENT_DEV" || true
    else
      echo "limine tool not available on host. To deploy BIOS stage run: arch-chroot $TARGET limine bios-install $PARENT_DEV"
    fi
  fi

  # Ensure limine-bios.sys is present on target /boot/limine
  if [ -e /usr/share/limine/limine-bios.sys ]; then
    cp -v /usr/share/limine/limine-bios.sys "$BOOT_LIMINE_DIR/" || true
  fi
fi

echo "Limine deployment helper finished"
