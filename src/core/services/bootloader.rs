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
            // 2: efistub TODO
            2 => {
                cmds.push("echo 'TODO: EFISTUB configuration not yet implemented'".into());
            }
            // 3: Limine
            3 => {
                state.debug_log("bootloader: installing Limine");

                // Mount sanity checks (ensure these lines appear in the log)
                cmds.push(
                    "echo '[check] Verifying mounts before bootloader'; \
                     echo '[check] findmnt /mnt'; findmnt /mnt || { echo 'ERROR: /mnt is not mounted'; exit 1; }; \
                     echo '[check] findmnt /mnt/boot'; findmnt /mnt/boot || { echo 'ERROR: /mnt/boot is not mounted'; exit 1; }; \
                     echo '[check] ls -ld /mnt/boot'; ls -ld /mnt/boot || { echo 'ERROR: /mnt/boot directory missing'; exit 1; }"
                        .to_string(),
                );

                // Install limine package
                cmds.push(chroot_cmd("pacman -S --needed --noconfirm limine"));

                if state.is_uefi() {
                    // UEFI mode - install efibootmgr
                    cmds.push(chroot_cmd("pacman -S --needed --noconfirm efibootmgr"));

                    // Consolidated UEFI setup to keep variables in scope (raw string for correct escaping)
                    cmds.push(r#"
# Resolve ESP parent disk robustly (handle /dev/disk/by-uuid/* and btrfs-style sources)
BOOT_SRC=$(findmnt -n -o SOURCE /mnt/boot 2>/dev/null | sed 's/\[.*$//');
BOOT_DEV=$(readlink -f "$BOOT_SRC" 2>/dev/null || echo "$BOOT_SRC");
PARENT_DEV=$(lsblk -no pkname "$BOOT_DEV" 2>/dev/null);
if [ -z "$PARENT_DEV" ]; then echo "WARN: PARENT_DEV empty; defaulting to internal disk assumption"; fi;
IS_USB=$(if [ -n "$PARENT_DEV" ] && [ "$(udevadm info --no-pager --query=property --property=ID_BUS --value --name=/dev/$PARENT_DEV 2>/dev/null)" = "usb" ]; then echo 1; else echo 0; fi);
echo "USB detection: IS_USB=$IS_USB PARENT_DEV=$PARENT_DEV";

# Create both potential target directories to avoid variable reliance
echo "Creating directory: /mnt/boot/EFI/limine";
install -d -m 0755 "/mnt/boot/EFI/limine" || { echo "ERROR: Failed to create /mnt/boot/EFI/limine"; exit 1; };
echo "Creating directory: /mnt/boot/EFI/BOOT";
install -d -m 0755 "/mnt/boot/EFI/BOOT" 2>/dev/null || true;

# Copy Limine EFI binaries to both locations (best-effort for EFI/BOOT)
cp /mnt/usr/share/limine/BOOTIA32.EFI "/mnt/boot/EFI/limine/" 2>/dev/null || true;
cp /mnt/usr/share/limine/BOOTX64.EFI "/mnt/boot/EFI/limine/" 2>/dev/null || true;
cp /mnt/usr/share/limine/BOOTIA32.EFI "/mnt/boot/EFI/BOOT/" 2>/dev/null || true;
cp /mnt/usr/share/limine/BOOTX64.EFI "/mnt/boot/EFI/BOOT/" 2>/dev/null || true;
echo "Copied Limine EFI binaries to /mnt/boot/EFI/limine and /mnt/boot/EFI/BOOT";

if [ "$IS_USB" != "1" ] && [ -n "$PARENT_DEV" ]; then
  PART_NUM=$(lsblk -no PARTNUM "$BOOT_DEV" 2>/dev/null);
  EFI_BITNESS=$(cat /sys/firmware/efi/fw_platform_size 2>/dev/null || echo 64);
  if [ "$EFI_BITNESS" = "64" ]; then
    LOADER_PATH="/EFI/limine/BOOTX64.EFI";
  else
    LOADER_PATH="/EFI/limine/BOOTIA32.EFI";
  fi;
  echo "Creating NVRAM entry: disk=/dev/$PARENT_DEV part=$PART_NUM loader=$LOADER_PATH";
  efibootmgr --create --disk "/dev/$PARENT_DEV" --part "$PART_NUM" --label "Arch Linux Limine Bootloader" --loader "$LOADER_PATH" --unicode --verbose || echo "efibootmgr failed, continuing...";
else
  echo "Skipping efibootmgr (USB install or unknown parent device)";
fi;

mkdir -p /mnt/etc/pacman.d/hooks;
cat > /mnt/etc/pacman.d/hooks/99-limine.hook <<HOOK_EOF
[Trigger]
Operation = Install
Operation = Upgrade
Type = Package
Target = limine

[Action]
Description = Deploying Limine after upgrade...
When = PostTransaction
Exec = /bin/sh -c "/usr/bin/cp /usr/share/limine/BOOTIA32.EFI /boot/EFI/limine/ 2>/dev/null || true; /usr/bin/cp /usr/share/limine/BOOTX64.EFI /boot/EFI/limine/ 2>/dev/null || true; /usr/bin/cp /usr/share/limine/BOOTIA32.EFI /boot/EFI/BOOT/ 2>/dev/null || true; /usr/bin/cp /usr/share/limine/BOOTX64.EFI /boot/EFI/BOOT/ 2>/dev/null || true"
HOOK_EOF
"#.to_string());
                } else {
                    // BIOS mode
                    state.debug_log("bootloader: Limine BIOS mode");

                    // Create /boot/limine directory
                    cmds.push(chroot_cmd("mkdir -p /boot/limine"));

                    // Copy limine-bios.sys
                    cmds.push(chroot_cmd("cp /usr/share/limine/limine-bios.sys /boot/limine/"));

                    // Run limine bios-install
                    cmds.push(format!(
                        "PARENT_DEV=/dev/$(lsblk -no pkname $(findmnt -n -o SOURCE /mnt/boot)); \
                        arch-chroot /mnt limine bios-install $PARENT_DEV || echo \"limine bios-install failed\""
                    ));

                    // Create pacman hook for BIOS
                    cmds.push(format!(
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
"
                    ));
                }

                // Generate limine.conf (and limine.cfg) with all selected kernels
                state.debug_log("bootloader: generating limine.conf");

                let mut kernels: Vec<String> = state.selected_kernels.iter().cloned().collect();
                kernels.sort();
                if kernels.is_empty() {
                    kernels.push("linux".to_string());
                }
                let uki_enabled = state.uki_enabled;

                // Build kernel parameters
                let kernel_params_setup = if state.disk_encryption_type_index == 1 {
                    r#"# Resolve root device and normalize (strip btrfs subvol suffix like [/@])
ROOT_DEV=$(findmnt -n -o SOURCE /mnt 2>/dev/null | sed 's/\[.*$//');
ROOT_UUID=$(blkid -s UUID -o value "$ROOT_DEV" 2>/dev/null || echo '');
# Try to detect underlying device of cryptroot for cryptdevice=UUID
MAP_NAME=$(basename "$(findmnt -n -o SOURCE /mnt 2>/dev/null | sed 's/\[.*$//')" | sed 's@^/dev/mapper/@@');
CRYPT_DEV=$(cryptsetup status "$MAP_NAME" 2>/dev/null | awk '/device:/ {print $2}');
CRYPT_UUID=$(blkid -s UUID -o value "$CRYPT_DEV" 2>/dev/null || echo '');
if [ -n "$CRYPT_UUID" ] && [ -n "$MAP_NAME" ]; then
  KERNEL_PARAMS="root=/dev/mapper/$MAP_NAME cryptdevice=UUID=$CRYPT_UUID:$MAP_NAME rw";
else
  KERNEL_PARAMS="root=UUID=$ROOT_UUID rw";
fi"#.to_string()
                } else {
                    r#"# Resolve root device and normalize (strip btrfs subvol suffix like [/@])
ROOT_DEV=$(findmnt -n -o SOURCE /mnt 2>/dev/null | sed 's/\[.*$//');
ROOT_UUID=$(blkid -s UUID -o value "$ROOT_DEV" 2>/dev/null || echo '');
KERNEL_PARAMS="root=UUID=$ROOT_UUID rw""#.to_string()
                };

                // Determine path_root setup and config dir selection based on UEFI/BIOS
                let path_root_setup = if state.is_uefi() {
                    r#"# Choose config dir similarly to the UEFI deployment target
if [ -d /mnt/boot/EFI/BOOT ]; then CONFIG_DIR=/mnt/boot/EFI/BOOT; else CONFIG_DIR=/mnt/boot/EFI/limine; fi;
BOOT_DEV=$(findmnt -n -o SOURCE /mnt/boot 2>/dev/null | sed 's/\[.*$//');
BOOT_UUID=$(blkid -s UUID -o value "$BOOT_DEV" 2>/dev/null || echo '');
# Fallback to /etc/fstab UUID
[ -z "$BOOT_UUID" ] && BOOT_UUID=$(awk '$2=="/boot"{print $1}' /mnt/etc/fstab 2>/dev/null | sed -n 's/^UUID=//p' | head -n1);
path_root="uuid(${BOOT_UUID})";
# Final guard
[ -z "$CONFIG_DIR" ] && CONFIG_DIR=/mnt/boot/EFI/limine;"#.to_string()
                } else {
                    r#"CONFIG_DIR=/mnt/boot/limine; path_root="boot()";
[ -z "$CONFIG_DIR" ] && CONFIG_DIR=/mnt/boot/limine;"#.to_string()
                };

                // Build the configuration file content in a single command
                // NOTE: Using <<LIMINE_CONF_EOF (without quotes) to allow variable expansion
                let mut config_script = String::new();
                config_script.push_str(&kernel_params_setup);
                config_script.push('\n');
                config_script.push_str(&path_root_setup);
                config_script.push('\n');
                config_script.push_str(r#"ROOT_UUID_FALLBACK=$(awk '$2=="/"{print $1}' /mnt/etc/fstab 2>/dev/null | sed -n 's/^UUID=//p' | head -n1);
if [ -z "$ROOT_UUID" ] && [ -n "$ROOT_UUID_FALLBACK" ]; then ROOT_UUID=$ROOT_UUID_FALLBACK; fi;
echo "Devices: ROOT_DEV=$ROOT_DEV BOOT_DEV=$BOOT_DEV";
echo "UUIDs: ROOT=$ROOT_UUID BOOT=$BOOT_UUID";
echo "Kernel params: $KERNEL_PARAMS";
echo "Path root: $path_root";
mkdir -p "$CONFIG_DIR";
cat > "$CONFIG_DIR/limine.conf" <<LIMINE_CONF_EOF
timeout: 5
"#);

                // Add entries for each kernel
                for kernel in &kernels {
                    for variant in &["", "-fallback"] {
                        if uki_enabled {
                            config_script.push_str(&format!(
                                "\n/Arch Linux ({}{})\n    protocol: efi\n    path: boot():/EFI/Linux/arch-{}.efi\n    cmdline: $KERNEL_PARAMS\n",
                                kernel, variant, kernel
                            ));
                        } else {
                            config_script.push_str(&format!(
                                "\n/Arch Linux ({}{})\n    protocol: linux\n    path: $path_root:/vmlinuz-{}\n    cmdline: $KERNEL_PARAMS\n    module_path: $path_root:/initramfs-{}{}.img\n",
                                kernel, variant, kernel, kernel, variant
                            ));
                        }
                    }
                }

                // Close heredoc, create limine.cfg for compatibility, and echo path
                config_script.push_str("LIMINE_CONF_EOF\n");
                config_script.push_str(
                    "cp -f \"$CONFIG_DIR/limine.conf\" \"$CONFIG_DIR/limine.cfg\" 2>/dev/null || true; "
                );
                // Also try to deploy config to EFI/BOOT best-effort for removable media
                config_script.push_str(
                    "if [ -d /mnt/boot/EFI/BOOT ]; then cp -f \"$CONFIG_DIR/limine.conf\" /mnt/boot/EFI/BOOT/limine.conf 2>/dev/null || true; fi; "
                );
                config_script.push_str(
                    "echo \"Created limine.conf at $CONFIG_DIR/limine.conf\""
                );

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