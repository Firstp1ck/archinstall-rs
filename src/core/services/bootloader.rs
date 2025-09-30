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
            // Limine bootloader (index 3)
            3 => {
                state.debug_log("bootloader: installing Limine");
                
                // Install limine package
                cmds.push(chroot_cmd("pacman -S --needed --noconfirm limine"));
                
                if state.is_uefi() {
                    // UEFI mode - install efibootmgr
                    cmds.push(chroot_cmd("pacman -S --needed --noconfirm efibootmgr"));
                    
                    // Copy install script to target and run it
                    cmds.push(
                        "cp ./assets/limine/install-limine.sh /mnt/tmp/install-limine.sh && \
                        chmod +x /mnt/tmp/install-limine.sh".to_string()
                    );
                    
                    // Run the install script
                    cmds.push(
                        "PARENT_DEV=$(lsblk -no pkname $(findmnt -n -o SOURCE /mnt/boot)); \
                        IS_USB=$(if [ \"$(udevadm info --no-pager --query=property --property=ID_BUS --value --name=/dev/$PARENT_DEV 2>/dev/null)\" = \"usb\" ]; then echo 1; else echo 0; fi); \
                        PART_NUM=$(lsblk -no PARTNUM $(findmnt -n -o SOURCE /mnt/boot)); \
                        echo \"Running install-limine.sh with PARENT_DEV=$PARENT_DEV IS_USB=$IS_USB PART_NUM=$PART_NUM\"; \
                        /mnt/tmp/install-limine.sh /mnt /mnt/boot /mnt/boot /dev/$PARENT_DEV $PART_NUM $IS_USB 1".to_string()
                    );
                    
                    // Create pacman hook for UEFI
                    cmds.push(
                        "IS_USB=$(if [ \"$(udevadm info --no-pager --query=property --property=ID_BUS --value --name=/dev/$(lsblk -no pkname $(findmnt -n -o SOURCE /mnt/boot)) 2>/dev/null)\" = \"usb\" ]; then echo 1; else echo 0; fi); \
                        if [ \"$IS_USB\" = \"1\" ]; then \
                            EFI_DIR_TARGET=/boot/EFI/BOOT; \
                        else \
                            EFI_DIR_TARGET=/boot/EFI/limine; \
                        fi; \
                        mkdir -p /mnt/etc/pacman.d/hooks; \
                        cat > /mnt/etc/pacman.d/hooks/99-limine.hook <<HOOK_EOF
[Trigger]
Operation = Install
Operation = Upgrade
Type = Package
Target = limine

[Action]
Description = Deploying Limine after upgrade...
When = PostTransaction
Exec = /bin/sh -c \"/usr/bin/cp /usr/share/limine/BOOTIA32.EFI $EFI_DIR_TARGET/ 2>/dev/null || true && /usr/bin/cp /usr/share/limine/BOOTX64.EFI $EFI_DIR_TARGET/\"
HOOK_EOF
".to_string()
                    );
                } else {
                    // BIOS mode
                    state.debug_log("bootloader: Limine BIOS mode");
                    
                    // Copy install script to target and run it
                    cmds.push(
                        "cp ./assets/limine/install-limine.sh /mnt/tmp/install-limine.sh && \
                        chmod +x /mnt/tmp/install-limine.sh".to_string()
                    );
                    
                    // Run the install script for BIOS
                    cmds.push(
                        "PARENT_DEV=$(lsblk -no pkname $(findmnt -n -o SOURCE /mnt/boot)); \
                        echo \"Running install-limine.sh for BIOS with PARENT_DEV=$PARENT_DEV\"; \
                        /mnt/tmp/install-limine.sh /mnt /mnt/boot \"\" /dev/$PARENT_DEV \"\" 0 1".to_string()
                    );
                    
                    // Create pacman hook for BIOS
                    cmds.push(
                        "PARENT_DEV=/dev/$(lsblk -no pkname $(findmnt -n -o SOURCE /mnt/boot)); \
                        mkdir -p /mnt/etc/pacman.d/hooks; \
                        cat > /mnt/etc/pacman.d/hooks/99-limine.hook <<HOOK_EOF
[Trigger]
Operation = Install
Operation = Upgrade
Type = Package
Target = limine

[Action]
Description = Deploying Limine after upgrade...
When = PostTransaction
Exec = /bin/sh -c \"/usr/bin/limine bios-install $PARENT_DEV && /usr/bin/cp /usr/share/limine/limine-bios.sys /boot/limine/\"
HOOK_EOF
".to_string()
                    );
                }
                
                // Generate limine.conf
                state.debug_log("bootloader: generating limine.conf");
                
                // Generate limine.conf with all selected kernels
                let mut kernels: Vec<String> = state.selected_kernels.iter().cloned().collect();
                kernels.sort();
                
                if kernels.is_empty() {
                    kernels.push("linux".to_string());
                }
                
                let uki_enabled = state.uki_enabled;
                
                // Build kernel parameters
                let kernel_params_setup = if state.disk_encryption_type_index == 1 {
                    // LUKS encryption enabled
                    format!("ROOT_UUID=$$(findmnt -n -o UUID /mnt); \
                    CRYPT_UUID=$$(blkid -s UUID -o value $$(cryptsetup status cryptroot 2>/dev/null | grep 'device:' | awk '{{print $$2}}') 2>/dev/null || echo ''); \
                    if [ -n \"$$CRYPT_UUID\" ]; then \
                        KERNEL_PARAMS=\"root=/dev/mapper/cryptroot cryptdevice=UUID=$$CRYPT_UUID:cryptroot rw\"; \
                    else \
                        KERNEL_PARAMS=\"root=UUID=$$ROOT_UUID rw\"; \
                    fi")
                } else {
                    format!("ROOT_UUID=$$(findmnt -n -o UUID /mnt); \
                    KERNEL_PARAMS=\"root=UUID=$$ROOT_UUID rw\"")
                };
                
                // Determine path_root and config directory based on UEFI/BIOS
                let (path_root_setup, config_dir) = if state.is_uefi() {
                    (format!("BOOT_UUID=$$(findmnt -n -o UUID /mnt/boot); path_root=\"uuid(${{BOOT_UUID}})\""), 
                     "/mnt/boot/EFI/limine".to_string())
                } else {
                    ("path_root=\"boot()\"".to_string(), "/mnt/boot/limine".to_string())
                };
                
                // Copy and customize the limine.conf template
                let mut config_script = format!(
                    "{}; {}; \
                    echo \"UUIDs: ROOT=$$ROOT_UUID BOOT=$$BOOT_UUID\"; \
                    echo \"Kernel params: $$KERNEL_PARAMS\"; \
                    echo \"Path root: $$path_root\"; \
                    mkdir -p {}; \
                    cp ./assets/limine/limine.conf.example {}/limine.conf; ",
                    kernel_params_setup, path_root_setup, config_dir, config_dir
                );
                
                // Replace placeholders in the template
                if uki_enabled {
                    // For UKI, generate EFI protocol entries
                    config_script.push_str(&format!(
                        "cat > {}/limine.conf <<LIMINE_CONF_EOF\ntimeout: 5\n",
                        config_dir
                    ));
                    
                    for kernel in &kernels {
                        for variant in &["", "-fallback"] {
                            config_script.push_str(&format!(
                                "\n/Arch Linux ({}{})\n    protocol: efi\n    path: boot():/EFI/Linux/arch-{}.efi\n    cmdline: $$KERNEL_PARAMS\n",
                                kernel, variant, kernel
                            ));
                        }
                    }
                    config_script.push_str("LIMINE_CONF_EOF\n");
                } else {
                    // Use sed to replace placeholders in template for standard boot
                    config_script.push_str(&format!(
                        "sed -i 's|uuid(BOOT_UUID)|'\"$$path_root\"'|g' {}/limine.conf; \
                        sed -i 's|root=/dev/disk/by-uuid/ROOT_UUID|'\"$$KERNEL_PARAMS\"'|g' {}/limine.conf; \
                        sed -i 's| rw quiet||g' {}/limine.conf; ",
                        config_dir, config_dir, config_dir
                    ));
                    
                    // Add entries for additional kernels if needed
                    if kernels.len() > 1 || !kernels.contains(&"linux".to_string()) {
                        config_script.push_str(&format!(
                            "cat >> {}/limine.conf <<LIMINE_CONF_EOF\n",
                            config_dir
                        ));
                        
                        for kernel in &kernels {
                            if kernel != "linux" {
                                for variant in &["", "-fallback"] {
                                    config_script.push_str(&format!(
                                        "\n/Arch Linux ({}{})\n    protocol: linux\n    path: $$path_root:/vmlinuz-{}\n    cmdline: $$KERNEL_PARAMS\n    module_path: $$path_root:/initramfs-{}{}.img\n",
                                        kernel, variant, kernel, kernel, variant
                                    ));
                                }
                            }
                        }
                        config_script.push_str("LIMINE_CONF_EOF\n");
                    }
                }
                
                config_script.push_str(&format!("echo \"Created limine.conf at {}/limine.conf\"", config_dir));
                
                cmds.push(config_script);
                
                state.debug_log("bootloader: Limine setup complete");
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