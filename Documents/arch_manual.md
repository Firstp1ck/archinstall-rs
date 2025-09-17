# Arch Linux Installation Guide - Basic Hyprland Setup

## Initial Setup
```bash
# List Keymaps
localectl list-keymaps

# Set keyboard layout to Swiss German
loadkeys de_CH-latin1

# Verify system time
timedatectl

# Verify Bootmode
cat /sys/firmware/efi/fw_platform_size

# Check Internet Connection
ping archlinux.org

# List available disks
fdisk -l
```

## Disk Partitioning
```bash
# Start partitioning
fdisk /dev/vda
    g       # Create new GPT partition table
    n       # Create new partition (repeat 3 times)
        +1g  # 1GB for EFI boot partition
        +4g  # 4GB for swap
        default # Remaining space for root filesystem
    t       # Change Partition Types
        4   # Boot Partition - EFI System
        19  # Swap Partition - Linux Swap
        20  # Root Partition - Linux Filesystem
    w # Write to disk

# Verify partition layout
lsblk
```

## Filesystem Setup
```bash
# Create filesystems
mkfs.btrfs /dev/vda3    # Root partition
mkfs.fat -F 32 /dev/vda1  # Boot partition
mkswap /dev/vda2        # Swap partition

# Mount partitions
mount /dev/vda3 /mnt                    # Root partition
mount --mkdir /dev/vda1 /mnt/boot       # Boot partition
swapon /dev/vda2                        # Enable swap
```

## System Installation
```bash
# Install base system and packages

# Add Multilib Repository
vim /etc/pacman.conf
    [multilib]
    Include = /etc/pacman.d/mirrorlist

# Update Mirrors for Multilib
pacman -Sy

pacstrap -K /mnt
    # Essential Terminal Packages
    base linux linux-firmware base-devel
    sudo pam grub efibootmgr
    curl git fish neovim
    man-db man-pages terminus-font
    networkmanager qemu-guest-agent xdg-user-dirs
    pipewire pipewire-pulse wireplumber

    # Graphical Environment Packages
    sddm hyprland xdg-desktop-portal-hyprland
    intel-ucode mesa vulkan-icd-loader sof-firmware
    lib32-mesa lib32-vulkan-icd-loader
    kitty wofi firefox dolphin polkit
```

### Generate fstab
```bash
genfstab -U /mnt >> /mnt/etc/fstab
```

## System Configuration
```bash
# Enter chroot environment
arch-chroot /mnt

# Set timezone
ln -sf /usr/share/zoneinfo/Europe/Zurich /etc/localtime
hwclock --systohc

# Configure locale
nvim /etc/locale.gen
    de_CH.UTF-8 UTF-8
    de_CH ISO-8859-1

# Generate Locale
locale-gen

#Create Local Configuration
nvim /etc/locale.conf
    LANG=de_CH.UTF-8

# Set keyboard layout
nvim /etc/vconsole.conf
    KEYMAP=de_CH-latin1

# Set hostname
nvim /etc/hostname
    usernameVM

# Enable NetworkManager
systemctl enable NetworkManager

# Enable NTP for Timesync
timedatectl set-ntp true

# Set root password
passwd
```

## Bootloader Setup
```bash
# Install and configure GRUB
grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=GRUB
grub-mkconfig -o /boot/grub/grub.cfg
```

### Reboot System 
```bash
reboot
```

## User Setup
```bash
# Create user
useradd -m -G wheel -s /bin/bash username
passwd username

# Configure sudo
export EDITOR=nvim
visudo  # Uncomment wheel and sudo group lines
    %wheel ALL=(ALL:ALL) ALL
    %sudo ALL=(ALL:ALL) ALL

# Enable display manager
systemctl enable --now sddm
```

## Hyprland Configuration
```bash
# Configure keyboard layout for Hyprland
nvim .config/hypr/hyprland.conf
    kb_layout = ch
```

## Final Steps
1. Reboot the system
2. Log in with your user account
3. Start configuring your desktop environment