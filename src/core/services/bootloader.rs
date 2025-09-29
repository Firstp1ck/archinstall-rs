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
            format!("arch-chroot /mnt bash -lc '{escaped}'")
        }

        // Debug: entering bootloader plan build
        state.debug_log(&format!(
            "bootloader: build_plan start (uefi={}, bootloader_index={}, device={})",
            state.is_uefi(),
            state.bootloader_index,
            _device
        ));

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
                        "grub-install --target=i386-pc {_device}"
                    )));
                }
                cmds.push(chroot_cmd("grub-mkconfig -o /boot/grub/grub.cfg"));
            }
            // TODO(v0.2.0+): Implement EFISTUB boot entry creation and kernel cmdline.
            2 => {
                cmds.push("echo 'TODO: EFISTUB configuration not yet implemented'".into());
            }
            // Limine bootloader (index 3): ensure package is present first
            3 => {
                state.debug_log("limine: starting plan build");
                // Ensure Limine is installed in the target system (non-interactive, skip if already installed)
                state.debug_log("limine: ensure package 'limine' is installed (pacman --needed)");
                cmds.push(chroot_cmd("pacman -S --needed --noconfirm limine"));

                // Ensure ESP directory exists for Limine and deploy BOOTX64.EFI
                state.debug_log("limine: ensure ESP target directory /boot/EFI/limine exists");
                cmds.push(chroot_cmd("mkdir -p /boot/EFI/limine"));
                state.debug_log("limine: copy BOOTX64.EFI to ESP (source: /usr/share/limine/BOOTX64.EFI)");
                cmds.push(chroot_cmd(
                    "cp -f /usr/share/limine/BOOTX64.EFI /boot/EFI/limine/BOOTX64.EFI",
                ));

                // Minimal pre-check: warn if ESP partition is not 1 (best-effort layout) without using lsblk
                state.debug_log("limine: pre-check ESP partition number using findmnt /boot");
                cmds.push(chroot_cmd(
                    "src=$(findmnt -n -o SOURCE /boot 2>/dev/null || true); part=; if [ -n \"$src\" ]; then case \"$src\" in */*p[0-9]) part=${src##*p};; */*[0-9]) part=${src##*[!0-9]};; esac; fi; if [ -n \"$part\" ] && [ \"$part\" != 1 ]; then echo 'Warning: Expected ESP partition number 1, got' \"$part\"; fi"
                ));

                // Ensure efivarfs is available (UEFI vars); skip if not UEFI
                state.debug_log("limine: ensure efivarfs is mounted when in UEFI mode");
                cmds.push(chroot_cmd(
                    "[ -d /sys/firmware/efi ] || exit 0; mountpoint -q /sys/firmware/efi/efivars || mount -t efivarfs efivarfs /sys/firmware/efi/efivars 2>/dev/null || true"
                ));

                // Create UEFI boot entry for Limine using selected device from state/config; assume best-effort ESP partition 1
                let selected = state.disks_selected_device.as_deref().unwrap_or(_device);
                let device_short = selected.trim_start_matches("/dev/");
                state.debug_log(&format!(
                    "limine: selected_device={}, device_short={}, efibootmgr target=\\EFI\\limine\\BOOTX64.EFI",
                    selected, device_short
                ));
                cmds.push(chroot_cmd(&format!(
                    "[ -d /sys/firmware/efi ] || exit 0; timeout 5 efibootmgr --create --disk /dev/{device_short} --part 1 --label 'Arch Linux Limine Bootloader' --loader '\\EFI\\limine\\BOOTX64.EFI' --unicode || true"
                )));

                // Determine root partition UUID (best-effort: partition 3) and persist for later limine.conf generation (with fallbacks)
                let part3 = if device_short
                    .chars()
                    .last()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                {
                    format!("{device_short}p3")
                } else {
                    format!("{device_short}3")
                };
                state.debug_log(&format!(
                    "limine: root partition candidate=/dev/{} (best-effort p3)",
                    part3
                ));
                state.debug_log("limine: will resolve root UUID (blkid, fallback to findmnt /)");
                cmds.push(chroot_cmd(&format!(
                    "uuid=$(blkid -s UUID -o value /dev/{part3} 2>/dev/null || true); if [ -z \"$uuid\" ]; then rootdev=$(findmnt -n -o SOURCE / 2>/dev/null || true); [ -n \"$rootdev\" ] && uuid=$(blkid -s UUID -o value \"$rootdev\" 2>/dev/null || true); fi; [ -n \"$uuid\" ] || echo Warning: could-not-determine-UUID-for-/dev/{part3}; printf '%s' \"$uuid\" > /tmp/limine-root-uuid"
                )));

                // Write Limine config file using the detected root UUID; fallback to blkid and recheck
                state.debug_log("limine: write /boot/EFI/limine/limine.conf with root UUID");
                cmds.push(chroot_cmd(&format!(
                    "uuid=$(cat /tmp/limine-root-uuid 2>/dev/null); if [ -z \"$uuid\" ]; then uuid=$(blkid -s UUID -o value /dev/{part3} 2>/dev/null || true); fi; if [ -z \"$uuid\" ]; then rootdev=$(findmnt -n -o SOURCE / 2>/dev/null || true); [ -n \"$rootdev\" ] && uuid=$(blkid -s UUID -o value \"$rootdev\" 2>/dev/null || true); fi; cat > /boot/EFI/limine/limine.conf <<EOF\ntimeout: 5\n\n/Arch Linux\n    protocol: linux\n    path: boot():/vmlinuz-linux\n    cmdline: root=UUID=$uuid rw\n    module_path: boot():/initramfs-linux.img\nEOF"
                )));

                // Verify limine.conf contains the UUID; if not, rewrite using printf/tee as a fallback
                state.debug_log("limine: verify limine.conf contains UUID; fallback to printf|tee if missing");
                cmds.push(chroot_cmd(&format!(
                    "uuid=$(cat /tmp/limine-root-uuid 2>/dev/null); if [ -z \"$uuid\" ]; then uuid=$(blkid -s UUID -o value /dev/{part3} 2>/dev/null || true); fi; if [ -z \"$uuid\" ]; then rootdev=$(findmnt -n -o SOURCE / 2>/dev/null || true); [ -n \"$rootdev\" ] && uuid=$(blkid -s UUID -o value \"$rootdev\" 2>/dev/null || true); fi; if [ -z \"$uuid\" ] || ! grep -q \"root=UUID=$uuid\" /boot/EFI/limine/limine.conf; then printf \"%s\\n\" \"timeout: 5\" \"\" \"/Arch Linux\" \"    protocol: linux\" \"    path: boot():/vmlinuz-linux\" \"    cmdline: root=UUID=$uuid rw\" \"    module_path: boot():/initramfs-linux.img\" | tee /boot/EFI/limine/limine.conf >/dev/null; fi"
                )));

                // Ensure pacman hooks directory exists
                state.debug_log("limine: ensure pacman hooks directory /etc/pacman.d/hooks exists");
                cmds.push(chroot_cmd("mkdir -p /etc/pacman.d/hooks"));

                // Write pacman hook for Limine deployment via heredoc
                state.debug_log("limine: write pacman hook 99-limine.hook");
                cmds.push(chroot_cmd(
                    "cat > /etc/pacman.d/hooks/99-limine.hook <<'EOF'\n[Trigger]\nOperation = Install\nOperation = Upgrade\nType = Package\nTarget = limine\n\n[Action]\nDescription = Deploying Limine after upgrade...\nWhen = PostTransaction\nExec = /usr/bin/cp /usr/share/limine/BOOTX64.EFI /boot/EFI/limine/\nEOF"
                ));

                // Verify hook content; if missing, rewrite using printf/tee as fallback
                state.debug_log("limine: verify pacman hook; fallback to printf|tee if needed");
                cmds.push(chroot_cmd(
                    "grep -q '^Target = limine' /etc/pacman.d/hooks/99-limine.hook && grep -q '^Exec = /usr/bin/cp ' /etc/pacman.d/hooks/99-limine.hook || printf '%s\\n' '[Trigger]' 'Operation = Install' 'Operation = Upgrade' 'Type = Package' 'Target = limine' '' '[Action]' 'Description = Deploying Limine after upgrade...' 'When = PostTransaction' 'Exec = /usr/bin/cp /usr/share/limine/BOOTX64.EFI /boot/EFI/limine/' | tee /etc/pacman.d/hooks/99-limine.hook >/dev/null"
                ));
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
