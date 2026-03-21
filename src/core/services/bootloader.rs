use crate::core::state::AppState;
use crate::core::storage::StoragePlan;

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

        // Kernel options differ depending on whether LUKS encryption is active.
        // With LUKS we use rd.luks.name to auto-open the container at boot;
        // without it we simply pass root=UUID=<uuid>.
        let boot_options_script = if encrypted {
            // Discover the underlying LUKS partition UUID at install time
            "rootdev=$(findmnt -n -o SOURCE /); \
             if cryptsetup status \"$(basename \"$rootdev\")\" >/dev/null 2>&1; then \
               underlying=$(cryptsetup status \"$(basename \"$rootdev\")\" | awk '/device:/{print $2}'); \
               luksuuid=$(blkid -s UUID -o value \"$underlying\" || true); \
               mapper=$(basename \"$rootdev\"); \
               echo \"rd.luks.name=$luksuuid=$mapper root=$rootdev rw\"; \
             else \
               rootuuid=$(blkid -s UUID -o value \"$rootdev\" || true); \
               echo \"root=UUID=$rootuuid rw\"; \
             fi"
        } else {
            "rootdev=$(findmnt -n -o SOURCE /); \
             rootuuid=$(blkid -s UUID -o value \"$rootdev\" || true); \
             echo \"root=UUID=$rootuuid rw\""
        };

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

                // For LUKS, inject cryptdevice into GRUB_CMDLINE_LINUX before generating config
                if encrypted {
                    cmds.push(chroot_cmd(
                        &format!("OPTS=$({boot_options_script}); \
                         sed -i \"s|^GRUB_CMDLINE_LINUX=.*|GRUB_CMDLINE_LINUX=\\\"$OPTS\\\"|\" /etc/default/grub")
                    ));
                }
                cmds.push(chroot_cmd("grub-mkconfig -o /boot/grub/grub.cfg"));
            }
            2 => {
                cmds.push("echo 'TODO: EFISTUB configuration not yet implemented'".into());
            }
            3 => {
                cmds.push("echo 'TODO: Limine bootloader setup not yet implemented'".into());
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
