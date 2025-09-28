use crate::core::state::AppState;

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
    pub fn build_plan(state: &AppState, _device: &str) -> BootloaderPlan {
        let mut cmds: Vec<String> = Vec::new();

        fn chroot_cmd(inner: &str) -> String {
            let escaped = inner.replace("'", "'\\''");
            format!("arch-chroot /mnt bash -lc '{}'", escaped)
        }

        match state.bootloader_index {
            // 0: systemd-boot
            0 => {
                // Install systemd-boot. Avoid touching EFI variables on some firmwares that hang.
                // Explicitly point to ESP and boot paths; skip writing NVRAM entries (we keep fallback).
                cmds.push(chroot_cmd(
                    "env SYSTEMD_PAGER=cat SYSTEMD_COLORS=0 timeout 30s bootctl --no-pager install --no-variables --esp-path=/boot --boot-path=/boot",
                ));

                // Build loader.conf
                cmds.push(chroot_cmd(
                    "install -d -m 0755 /boot/loader && install -d -m 0755 /boot/loader/entries",
                ));
                cmds.push(chroot_cmd(
                    "bash -lc 'cat > /boot/loader/loader.conf <<EOF\ndefault  arch.conf\ntimeout  4\nconsole-mode auto\neditor   no\nEOF'",
                ));

                // Build arch.conf and fallback with inline UUID discovery
                cmds.push(chroot_cmd(
                    "rootdev=$(findmnt -n -o SOURCE /); rootuuid=$(blkid -s UUID -o value \"$rootdev\" || true); cat > /boot/loader/entries/arch.conf <<EOF\ntitle   Arch Linux\nlinux   /vmlinuz-linux\ninitrd  /initramfs-linux.img\noptions root=UUID=$rootuuid rw\nEOF",
                ));
                cmds.push(chroot_cmd(
                    "rootdev=$(findmnt -n -o SOURCE /); rootuuid=$(blkid -s UUID -o value \"$rootdev\" || true); cat > /boot/loader/entries/arch-fallback.conf <<EOF\ntitle   Arch Linux (fallback initramfs)\nlinux   /vmlinuz-linux\ninitrd  /initramfs-linux-fallback.img\noptions root=UUID=$rootuuid rw\nEOF",
                ));

                // Verify entries
                cmds.push(chroot_cmd("env SYSTEMD_PAGER=cat SYSTEMD_COLORS=0 timeout 5s bootctl --no-pager list || true"));

                // Fallback: if bootctl install failed, attempt efibootmgr
                // Guard on efivarfs presence and add a timeout to avoid hangs on buggy firmware
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
                    // BIOS mode install to disk (not a partition)
                    cmds.push(chroot_cmd(&format!(
                        "grub-install --target=i386-pc {}",
                        _device
                    )));
                }
                cmds.push(chroot_cmd("grub-mkconfig -o /boot/grub/grub.cfg"));
            }
            // TODO(v0.2.0+): Implement EFISTUB boot entry creation and kernel cmdline.
            2 => {
                cmds.push("echo 'TODO: EFISTUB configuration not yet implemented'".into());
            }
            // Limine implementation
            3 => {
                // Determine kernels list (fallback to 'linux' if none)
                let mut kernels: Vec<String> = if state.selected_kernels.is_empty() {
                    vec!["linux".to_string()]
                } else {
                    state.selected_kernels.iter().cloned().collect()
                };
                // BTreeSet iteration is sorted; ensure stable order
                kernels.sort();
                let kernels_str = if kernels.is_empty() {
                    "linux".to_string()
                } else {
                    kernels.join(" ")
                };

                if state.is_uefi() {
                    // Ensure directories for EFI and config
                    cmds.push(chroot_cmd("install -d -m0755 /boot/EFI/limine /boot/EFI/BOOT /boot/limine"));

                    // Copy Limine EFI binary if present
                    cmds.push(chroot_cmd(
                        "if [ -f /usr/share/limine/BOOTX64.EFI ]; then cp /usr/share/limine/BOOTX64.EFI /boot/EFI/limine/; else echo 'Warning: /usr/share/limine/BOOTX64.EFI not found' >&2; fi",
                    ));

                    // Generate limine.conf using only double quotes to avoid escaping pitfalls
                    let gen_conf = format!(
                        "install -d -m0755 /boot/limine; : > /boot/limine/limine.conf; rootdev=$(findmnt -n -o SOURCE / || true); if [ {} -eq 1 ]; then luksdev=$(lsblk -no pkname \"$rootdev\" | sed \"s#^#/dev/#\"); luks_uuid=$(blkid -s UUID -o value \"$luksdev\" || true); cmdline=\"cryptdevice=UUID=$luks_uuid:cryptroot root=/dev/mapper/cryptroot rw\"; else root_uuid=$(blkid -s UUID -o value \"$rootdev\" || true); cmdline=\"root=UUID=$root_uuid rw\"; fi; for k in {}; do {{ printf \"/Arch Linux (%s)\\n\" \"$k\"; printf \"protocol: linux\\n\"; printf \"path: boot():/vmlinuz-%s\\n\" \"$k\"; printf \"cmdline: %s\\n\" \"$cmdline\"; printf \"module_path: boot():/initramfs-%s.img\\n\\n\" \"$k\"; printf \"/Arch Linux (%s) (fallback)\\n\" \"$k\"; printf \"protocol: linux\\n\"; printf \"path: boot():/vmlinuz-%s\\n\" \"$k\"; printf \"cmdline: %s\\n\" \"$cmdline\"; printf \"module_path: boot():/initramfs-%s-fallback.img\\n\\n\" \"$k\"; }} >> /boot/limine/limine.conf; done",
                        if state.disk_encryption_type_index == 1 { 1 } else { 0 },
                        kernels_str
                    );
                    cmds.push(chroot_cmd(&gen_conf));

                    // Try to create an NVRAM entry; always copy fallback EFI path as well
                    cmds.push(chroot_cmd(
                        "dev=$(findmnt -n -o SOURCE /boot); disk=/dev/$(lsblk -no pkname \"$dev\"); part=$(lsblk -no PARTNUM \"$dev\"); if mountpoint -q /sys/firmware/efi/efivars || mount -t efivarfs efivarfs /sys/firmware/efi/efivars 2>/dev/null; then timeout 5 efibootmgr --create --disk \"$disk\" --part \"$part\" --loader '\\EFI\\limine\\BOOTX64.EFI' --label 'Limine' --unicode || true; fi; cp /boot/EFI/limine/BOOTX64.EFI /boot/EFI/BOOT/BOOTX64.EFI || true",
                    ));
                } else {
                    // BIOS install flow
                    cmds.push(chroot_cmd("install -d -m0755 /boot/limine"));
                    cmds.push(chroot_cmd(
                        "if [ -f /usr/share/limine/limine-bios.sys ]; then cp /usr/share/limine/limine-bios.sys /boot/limine/; else echo 'Warning: /usr/share/limine/limine-bios.sys not found' >&2; fi",
                    ));

                    let gen_conf = format!(
                        "install -d -m0755 /boot/limine; : > /boot/limine/limine.conf; rootdev=$(findmnt -n -o SOURCE / || true); if [ {} -eq 1 ]; then luksdev=$(lsblk -no pkname \"$rootdev\" | sed \"s#^#/dev/#\"); luks_uuid=$(blkid -s UUID -o value \"$luksdev\" || true); cmdline=\"cryptdevice=UUID=$luks_uuid:cryptroot root=/dev/mapper/cryptroot rw\"; else root_uuid=$(blkid -s UUID -o value \"$rootdev\" || true); cmdline=\"root=UUID=$root_uuid rw\"; fi; for k in {}; do {{ printf \"/Arch Linux (%s)\\n\" \"$k\"; printf \"protocol: linux\\n\"; printf \"path: boot():/vmlinuz-%s\\n\" \"$k\"; printf \"cmdline: %s\\n\" \"$cmdline\"; printf \"module_path: boot():/initramfs-%s.img\\n\\n\" \"$k\"; printf \"/Arch Linux (%s) (fallback)\\n\" \"$k\"; printf \"protocol: linux\\n\"; printf \"path: boot():/vmlinuz-%s\\n\" \"$k\"; printf \"cmdline: %s\\n\" \"$cmdline\"; printf \"module_path: boot():/initramfs-%s-fallback.img\\n\\n\" \"$k\"; }} >> /boot/limine/limine.conf; done",
                        if state.disk_encryption_type_index == 1 { 1 } else { 0 },
                        kernels_str
                    );
                    cmds.push(chroot_cmd(&gen_conf));

                    // Detect bios_grub partition and run limine bios-install
                    cmds.push(chroot_cmd(&format!(
                        "disk=\"{}\"; pn=$(lsblk -nr -o PARTNUM,PARTFLAGS \"$disk\" | awk '$2 ~ /bios_grub/ {{print $1; exit}}'); if [ -n \"$pn\" ]; then limine bios-install \"$disk\" \"$pn\" || true; else limine bios-install \"$disk\" || true; fi",
                        _device
                    )));
                }
            }
            _ => {}
        }

        // Debug summary
        state.debug_log(&format!(
            "bootloader: choice={} mode={} (uefi={})",
            match state.bootloader_index {
                0 => "systemd-boot",
                1 => "grub",
                2 => "efistub",
                3 => "limine",
                _ => "unknown",
            },
            if state.is_uefi() { "UEFI" } else { "BIOS" },
            state.is_uefi()
        ));

        BootloaderPlan::new(cmds)
    }
}
