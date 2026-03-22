use crate::core::state::AppState;
use crate::core::storage::StoragePlan;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct BootloaderPlan {
    pub commands: Vec<String>,
}

impl BootloaderPlan {
    pub fn new(commands: Vec<String>) -> Self {
        Self { commands }
    }
}

pub struct BootloaderService;

impl BootloaderService {
    /// What: Builds a bash snippet that prints kernel cmdline options for `/` (root) inside the chroot.
    ///
    /// Inputs:
    /// - `encrypted`: When true, resolves LUKS mapper + UUID and emits `rd.luks.name=…` or
    ///   `cryptdevice=UUID=…` based on `/etc/mkinitcpio.conf` hooks; otherwise emits `root=UUID=… rw`.
    ///
    /// Output:
    /// - A shell script body (no surrounding quotes) suitable for `OPTS=$(…)` substitution.
    ///
    /// Details:
    /// - Strips btrfs `[@subvol]` suffixes from `findmnt` output before `blkid`/`cryptsetup`.
    pub(crate) fn boot_options_script(encrypted: bool) -> String {
        let strip_subvol = "rootdev=$(findmnt -n -o SOURCE /); rootdev=\"${rootdev%%\\[*}\"";
        if encrypted {
            format!(
                "{strip_subvol}; \
                 mapper=$(basename \"$rootdev\"); \
                 if cryptsetup status \"$mapper\" >/dev/null 2>&1; then \
                   underlying=$(cryptsetup status \"$mapper\" | awk '/device:/{{print $2}}'); \
                   luksuuid=$(blkid -s UUID -o value \"$underlying\" || true); \
                   if grep -qP '^HOOKS=.*\\bsystemd\\b' /etc/mkinitcpio.conf; then \
                     echo \"rd.luks.name=$luksuuid=$mapper root=$rootdev rw\"; \
                   else \
                     echo \"cryptdevice=UUID=$luksuuid:$mapper root=$rootdev rw\"; \
                   fi; \
                 else \
                   rootuuid=$(blkid -s UUID -o value \"$rootdev\" || true); \
                   echo \"root=UUID=$rootuuid rw\"; \
                 fi"
            )
        } else {
            format!(
                "{strip_subvol}; \
                 rootuuid=$(blkid -s UUID -o value \"$rootdev\" || true); \
                 echo \"root=UUID=$rootuuid rw\""
            )
        }
    }

    /// Whole-disk path for BIOS `grub-install --target=i386-pc` or `limine bios-install` when the
    /// partitioning flow has no explicit target (pre-mounted mode). Prefers the disk behind `/mnt`
    /// (findmnt + lsblk), then `disks_selected_device`.
    pub(crate) fn effective_bios_grub_disk(state: &AppState, partition_target: &str) -> String {
        if !partition_target.is_empty() {
            return partition_target.to_string();
        }
        if state.is_uefi() {
            return String::new();
        }
        if state.bootloader_index != 1 && state.bootloader_index != 3 {
            return String::new();
        }
        Self::disk_for_block_device_behind_mnt_root()
            .or_else(|| {
                state
                    .disks_selected_device
                    .clone()
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or_default()
    }

    /// Walks from the `/mnt` root block source up to a `TYPE=disk` node (handles partitions, LUKS, LVM).
    pub fn disk_for_block_device_behind_mnt_root() -> Option<String> {
        let out = Command::new("findmnt")
            .args(["-n", "-o", "SOURCE", "--target", "/mnt"])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let mut src = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if src.is_empty() {
            return None;
        }
        if let Some(idx) = src.find('[') {
            src.truncate(idx);
            src = src.trim().to_string();
        }
        if !src.starts_with("/dev/") {
            return None;
        }

        let mut dev = src;
        for _ in 0..32 {
            let out = Command::new("lsblk")
                .args(["-ndo", "TYPE,PKNAME", &dev])
                .output()
                .ok()?;
            if !out.status.success() {
                return None;
            }
            let line = String::from_utf8_lossy(&out.stdout);
            let mut parts = line.split_whitespace();
            let typ = parts.next()?;
            let pkname = parts.next().filter(|s| !s.is_empty());
            if typ == "disk" {
                return Some(dev);
            }
            let parent = pkname?;
            dev = if parent.starts_with("/dev/") {
                parent.to_string()
            } else {
                format!("/dev/{parent}")
            };
        }
        None
    }

    /// What: Builds `arch-chroot` shell commands for Limine on UEFI and/or BIOS.
    ///
    /// Inputs:
    /// - `state`: Firmware mode from `state.is_uefi()` selects EFI vs BIOS steps.
    /// - `device`: Whole disk for `limine bios-install` when not UEFI (ignored on UEFI).
    /// - `boot_options_script`: Bash snippet that prints kernel cmdline options (`boot_options_script` output).
    ///
    /// Output:
    /// - Ordered `arch-chroot … bash -lc '…'` command strings.
    ///
    /// Details:
    /// - Writes `/boot/limine.conf` on the ESP/boot volume; copies `BOOTX64.EFI` and registers NVRAM on UEFI;
    ///   copies `limine-bios.sys` and runs `limine bios-install` on BIOS when `device` is non-empty.
    fn limine_install_chroot_commands(
        state: &AppState,
        device: &str,
        boot_options_script: &str,
    ) -> Vec<String> {
        fn chroot_cmd(inner: &str) -> String {
            let escaped = inner.replace("'", "'\\''");
            format!("arch-chroot /mnt bash -lc '{escaped}'")
        }
        let mut out: Vec<String> = Vec::new();
        out.push(chroot_cmd(&format!(
            "OPTS=$({boot_options_script}); cat > /boot/limine.conf <<LIMINEOF\n\
timeout: 5\n\
\n\
/Arch Linux\n\
    protocol: linux\n\
    path: boot():/vmlinuz-linux\n\
    cmdline: $OPTS\n\
    module_path: boot():/initramfs-linux.img\n\
\n\
/Arch Linux (fallback initramfs)\n\
    protocol: linux\n\
    path: boot():/vmlinuz-linux\n\
    cmdline: $OPTS\n\
    module_path: boot():/initramfs-linux-fallback.img\n\
LIMINEOF"
        )));
        if state.is_uefi() {
            out.push(chroot_cmd(
                "install -d -m 0755 /boot/EFI/limine && install -m 0644 /usr/share/limine/BOOTX64.EFI /boot/EFI/limine/BOOTX64.EFI",
            ));
            out.push(chroot_cmd(
                "install -d -m 0755 /etc/pacman.d/hooks && cat > /etc/pacman.d/hooks/99-limine.hook <<HOOK_EOF\n\
[Trigger]\n\
Operation = Install\n\
Operation = Upgrade\n\
Type = Package\n\
Target = limine\n\
\n\
[Action]\n\
Description = Sync Limine EFI binary after package upgrade\n\
When = PostTransaction\n\
Exec = /usr/bin/install -m 0644 /usr/share/limine/BOOTX64.EFI /boot/EFI/limine/BOOTX64.EFI\n\
HOOK_EOF",
            ));
            out.push(chroot_cmd(
                "if mountpoint -q /sys/firmware/efi/efivars || mount -t efivarfs efivarfs /sys/firmware/efi/efivars 2>/dev/null; then \
                 BOOTSRC=$(findmnt -n -o SOURCE /boot); \
                 DISK=$(lsblk -no pkname \"$BOOTSRC\"); \
                 PART=$(lsblk -no PARTNUM \"$BOOTSRC\"); \
                 efibootmgr --create --disk \"/dev/$DISK\" --part \"$PART\" --label 'Arch Linux Limine' --loader '\\\\EFI\\\\limine\\\\BOOTX64.EFI' --unicode || true; \
                 efibootmgr --verbose || true; \
                 fi",
            ));
        } else {
            out.push(chroot_cmd(
                "install -d -m 0755 /boot/limine && install -m 0644 /usr/share/limine/limine-bios.sys /boot/limine/limine-bios.sys",
            ));
            let disk = device.trim();
            if !disk.is_empty() {
                out.push(chroot_cmd(&format!("limine bios-install {disk}")));
            }
        }
        out
    }

    pub fn build_plan(
        state: &AppState,
        device: &str,
        storage_plan: &StoragePlan,
    ) -> BootloaderPlan {
        let mut cmds: Vec<String> = Vec::new();
        let encrypted = storage_plan.has_encryption();

        fn chroot_cmd(inner: &str) -> String {
            let escaped = inner.replace("'", "'\\''");
            format!("arch-chroot /mnt bash -lc '{escaped}'")
        }

        state.debug_log(&format!(
            "bootloader: build_plan start (uefi={}, bootloader_index={}, device={}, encrypted={})",
            state.is_uefi(),
            state.bootloader_index,
            device,
            encrypted
        ));

        let boot_options_script = Self::boot_options_script(encrypted);

        match state.bootloader_index {
            // 0: systemd-boot
            0 => {
                cmds.push(chroot_cmd(
                    "env SYSTEMD_PAGER=cat SYSTEMD_COLORS=0 timeout 30s bootctl --no-pager install --no-variables --esp-path=/boot --boot-path=/boot",
                ));

                cmds.push(chroot_cmd(
                    "install -d -m 0755 /boot/loader && install -d -m 0755 /boot/loader/entries",
                ));
                cmds.push(chroot_cmd(
                    "bash -lc 'cat > /boot/loader/loader.conf <<EOF\ndefault  arch.conf\ntimeout  4\nconsole-mode auto\neditor   no\nEOF'",
                ));

                // Build arch.conf and fallback using the dynamic options script
                cmds.push(chroot_cmd(&format!(
                    "OPTS=$({boot_options_script}); cat > /boot/loader/entries/arch.conf <<EOF\ntitle   Arch Linux\nlinux   /vmlinuz-linux\ninitrd  /initramfs-linux.img\noptions $OPTS\nEOF"
                )));
                cmds.push(chroot_cmd(&format!(
                    "OPTS=$({boot_options_script}); cat > /boot/loader/entries/arch-fallback.conf <<EOF\ntitle   Arch Linux (fallback initramfs)\nlinux   /vmlinuz-linux\ninitrd  /initramfs-linux-fallback.img\noptions $OPTS\nEOF"
                )));

                cmds.push(chroot_cmd("env SYSTEMD_PAGER=cat SYSTEMD_COLORS=0 timeout 5s bootctl --no-pager list || true"));

                cmds.push(chroot_cmd(
                    "env SYSTEMD_PAGER=cat SYSTEMD_COLORS=0 timeout 5s bootctl --no-pager status >/dev/null 2>&1 || { if mountpoint -q /sys/firmware/efi/efivars || mount -t efivarfs efivarfs /sys/firmware/efi/efivars 2>/dev/null; then timeout 5 efibootmgr --create --disk $(lsblk -no pkname $(findmnt -n -o SOURCE /boot)) --part $(lsblk -no PARTNUM $(findmnt -n -o SOURCE /boot)) --loader '\\EFI\\systemd\\systemd-bootx64.efi' --label 'Linux Boot Manager' --unicode || true; fi; }",
                ));
            }
            // 1: grub
            1 => {
                if state.is_uefi() {
                    cmds.push(chroot_cmd(
                        "grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=GRUB",
                    ));
                } else {
                    cmds.push(chroot_cmd(&format!(
                        "grub-install --target=i386-pc {device}"
                    )));
                }

                // For LUKS, inject rd.luks.name / root= into GRUB_CMDLINE_LINUX before grub-mkconfig
                if encrypted {
                    cmds.push(chroot_cmd(
                        &format!("OPTS=$({boot_options_script}); \
                         sed -i \"s|^GRUB_CMDLINE_LINUX=.*|GRUB_CMDLINE_LINUX=\\\"$OPTS\\\"|\" /etc/default/grub")
                    ));
                }
                cmds.push(chroot_cmd("grub-mkconfig -o /boot/grub/grub.cfg"));
            }
            2 => {
                // Direct kernel boot via firmware: efibootmgr entries with vmlinuz + embedded cmdline/initrd path.
                if state.is_uefi() {
                    cmds.push(chroot_cmd(&format!(
                        "if mountpoint -q /sys/firmware/efi/efivars || mount -t efivarfs efivarfs /sys/firmware/efi/efivars 2>/dev/null; then \
                         OPTS=$({boot_options_script}); \
                         BOOTSRC=$(findmnt -n -o SOURCE /boot); \
                         DISK=$(lsblk -no pkname \"$BOOTSRC\"); \
                         PART=$(lsblk -no PARTNUM \"$BOOTSRC\"); \
                         efibootmgr --create --disk \"/dev/$DISK\" --part \"$PART\" --label 'Arch Linux' --loader '\\\\vmlinuz-linux' --unicode \"$OPTS initrd=\\\\initramfs-linux.img\" || true; \
                         efibootmgr --create --disk \"/dev/$DISK\" --part \"$PART\" --label 'Arch Linux (fallback initramfs)' --loader '\\\\vmlinuz-linux' --unicode \"$OPTS initrd=\\\\initramfs-linux-fallback.img\" || true; \
                         efibootmgr --verbose || true; \
                         fi"
                    )));
                }
            }
            3 => {
                cmds.extend(Self::limine_install_chroot_commands(
                    state,
                    device,
                    boot_options_script.as_str(),
                ));
            }
            _ => {}
        }

        state.debug_log(&format!(
            "bootloader: choice={} mode={} (uefi={}, encrypted={})",
            match state.bootloader_index {
                0 => "systemd-boot",
                1 => "grub",
                2 => "efistub",
                3 => "limine",
                _ => "unknown",
            },
            if state.is_uefi() { "UEFI" } else { "BIOS" },
            state.is_uefi(),
            encrypted
        ));

        BootloaderPlan::new(cmds)
    }
}

#[cfg(test)]
mod tests {
    use super::BootloaderService;
    use crate::app::AppState;

    #[test]
    fn effective_bios_grub_disk_passes_through_partition_target() {
        let mut state = AppState::new(true);
        state.bootloader_index = 1;
        assert_eq!(
            BootloaderService::effective_bios_grub_disk(&state, "/dev/zda").as_str(),
            "/dev/zda"
        );
    }

    #[test]
    fn effective_bios_grub_disk_non_grub_returns_empty_for_empty_target() {
        let state = AppState::new(true);
        assert!(BootloaderService::effective_bios_grub_disk(&state, "").is_empty());
    }

    #[test]
    fn effective_bios_grub_disk_limine_passes_through_partition_target() {
        let mut state = AppState::new(true);
        state.bootloader_index = 3;
        state.firmware_uefi_override = Some(false);
        assert_eq!(
            BootloaderService::effective_bios_grub_disk(&state, "/dev/zda").as_str(),
            "/dev/zda"
        );
    }

    #[test]
    fn effective_bios_grub_disk_limine_empty_on_uefi_override() {
        let mut state = AppState::new(true);
        state.bootloader_index = 3;
        state.firmware_uefi_override = Some(true);
        assert!(BootloaderService::effective_bios_grub_disk(&state, "").is_empty());
    }
}
