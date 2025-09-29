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

                // Focus more on actual root mount instead of guessing partition 3
                state.debug_log("limine: starting root partition UUID detection");
                state.debug_log("limine: will use multiple methods to find the root filesystem UUID");
                
                // Multi-stage UUID detection with debugging and various fallback methods
                // Main improvement: Focus first on finding the actual root mount, then determine UUID
                cmds.push(chroot_cmd(
                    "echo 'Debug: Starting UUID detection for root partition' > /tmp/limine-debug.log && \
                    # Step 1: Directly find the root filesystem from mount point \
                    rootdev=$(findmnt -n -o SOURCE / 2>/dev/null) && \
                    echo \"Debug: Found root device from findmnt: $rootdev\" >> /tmp/limine-debug.log && \
                    if [ -z \"$rootdev\" ]; then \
                        # Fallback method if findmnt fails \
                        rootdev=$(mount | grep ' on / ' | cut -d' ' -f1) && \
                        echo \"Debug: Found root device from mount command: $rootdev\" >> /tmp/limine-debug.log; \
                    fi && \
                    # Step 2: Get UUID from the detected root device \
                    if [ -n \"$rootdev\" ]; then \
                        uuid=$(blkid -s UUID -o value \"$rootdev\" 2>/dev/null) && \
                        echo \"Debug: UUID from root device ($rootdev): $uuid\" >> /tmp/limine-debug.log; \
                    else \
                        echo \"Debug: Could not determine root device\" >> /tmp/limine-debug.log; \
                        uuid=''; \
                    fi && \
                    # Step 3: If UUID is still empty, try using partition scan \
                    if [ -z \"$uuid\" ]; then \
                        echo \"Debug: Failed to get UUID from root device, trying partition scan\" >> /tmp/limine-debug.log; \
                        # Try to identify likely root partitions based on size and type \
                        for part in $(lsblk -lnp -o NAME,FSTYPE | grep -E 'ext4|btrfs|xfs' | cut -d' ' -f1); do \
                            echo \"Debug: Checking partition $part\" >> /tmp/limine-debug.log; \
                            test_uuid=$(blkid -s UUID -o value \"$part\" 2>/dev/null); \
                            if [ -n \"$test_uuid\" ]; then \
                                echo \"Debug: Found possible root partition: $part with UUID: $test_uuid\" >> /tmp/limine-debug.log; \
                                uuid=\"$test_uuid\"; \
                                rootdev=\"$part\"; \
                                break; \
                            fi; \
                        done; \
                    fi && \
                    # Store results and report outcome \
                    if [ -z \"$uuid\" ]; then \
                        echo \"Warning: Failed to determine root filesystem UUID\" | tee -a /tmp/limine-debug.log; \
                        # Last resort: try direct device \
                        echo \"Debug: Attempting to find root by mounting /mnt\" >> /tmp/limine-debug.log; \
                        root_source=$(findmnt -n -o SOURCE /mnt 2>/dev/null || true); \
                        if [ -n \"$root_source\" ]; then \
                            echo \"Debug: Root source from /mnt: $root_source\" >> /tmp/limine-debug.log; \
                            uuid=$(blkid -s UUID -o value \"$root_source\" 2>/dev/null || true); \
                            echo \"Debug: UUID from /mnt source: $uuid\" >> /tmp/limine-debug.log; \
                        fi; \
                    else \
                        echo \"Success: Found root partition $rootdev with UUID: $uuid\" | tee -a /tmp/limine-debug.log; \
                    fi && \
                    # Store UUID for next steps \
                    printf '%s' \"$uuid\" > /tmp/limine-root-uuid && \
                    # Report identified root device \
                    echo \"Root device: $rootdev\" >> /tmp/limine-debug.log && \
                    # Report UUID \
                    echo \"Root UUID: $uuid\" >> /tmp/limine-debug.log && \
                    # Output all debug info \
                    cat /tmp/limine-debug.log"
                ));

                // Write Limine config file using the detected root UUID with additional fallbacks
                state.debug_log("limine: write /boot/EFI/limine/limine.conf with root UUID");
                cmds.push(chroot_cmd(&format!(
                    "uuid=$(cat /tmp/limine-root-uuid 2>/dev/null) && \
                    echo \"Debug: Reading UUID from temp file: uuid=$uuid\" >> /tmp/limine-debug.log && \
                    # Retry UUID detection if not found in temp file \
                    if [ -z \"$uuid\" ]; then \
                      echo \"Debug: UUID not found in temp file, trying alternative methods\" >> /tmp/limine-debug.log; \
                      # Try root mount first \
                      rootdev=$(findmnt -n -o SOURCE / 2>/dev/null || true); \
                      if [ -n \"$rootdev\" ]; then \
                        uuid=$(blkid -s UUID -o value \"$rootdev\" 2>/dev/null || true); \
                        echo \"Debug: Alternative method (findmnt + blkid): rootdev=$rootdev, uuid=$uuid\" >> /tmp/limine-debug.log; \
                      fi; \
                      # If still not found, try direct partition \
                      if [ -z \"$uuid\" ]; then \
                        uuid=$(blkid -s UUID -o value /dev/{part3} 2>/dev/null || true); \
                        echo \"Debug: Alternative method (direct blkid): uuid=$uuid\" >> /tmp/limine-debug.log; \
                      fi; \
                      # Try lsblk as another fallback \
                      if [ -z \"$uuid\" ]; then \
                        uuid=$(lsblk -no UUID /dev/{part3} 2>/dev/null || true); \
                        echo \"Debug: Alternative method (lsblk): uuid=$uuid\" >> /tmp/limine-debug.log; \
                      fi; \
                    fi && \
                    # Final debug before writing config \
                    echo \"Debug: Using UUID=$uuid for limine.conf\" >> /tmp/limine-debug.log && \
                    # Try to create the config file with heredoc first \
                    cat > /boot/EFI/limine/limine.conf <<EOF \
timeout: 5

/Arch Linux
    protocol: linux
    path: boot():/vmlinuz-linux
    cmdline: root=UUID=$uuid rw
    module_path: boot():/initramfs-linux.img
EOF"
                )));

                // Verify limine.conf contains the UUID; if not, rewrite using printf|tee as a fallback
                state.debug_log("limine: verify limine.conf contents and use fallback if needed");
                cmds.push(chroot_cmd(&format!(
                    "if [ ! -f /boot/EFI/limine/limine.conf ]; then \
                      echo \"Debug: limine.conf not created, using fallback method\" >> /tmp/limine-debug.log; \
                      uuid=$(cat /tmp/limine-root-uuid 2>/dev/null); \
                      printf \"%s\\n\" \"timeout: 5\" \"\" \"/Arch Linux\" \"    protocol: linux\" \"    path: boot():/vmlinuz-linux\" \"    cmdline: root=UUID=$uuid rw\" \"    module_path: boot():/initramfs-linux.img\" | tee /boot/EFI/limine/limine.conf >/dev/null; \
                    else \
                      if ! grep -q \"root=UUID=\" /boot/EFI/limine/limine.conf; then \
                        echo \"Debug: limine.conf missing UUID, using fallback method\" >> /tmp/limine-debug.log; \
                        uuid=$(cat /tmp/limine-root-uuid 2>/dev/null); \
                        printf \"%s\\n\" \"timeout: 5\" \"\" \"/Arch Linux\" \"    protocol: linux\" \"    path: boot():/vmlinuz-linux\" \"    cmdline: root=UUID=$uuid rw\" \"    module_path: boot():/initramfs-linux.img\" | tee /boot/EFI/limine/limine.conf >/dev/null; \
                      else \
                        echo \"Debug: limine.conf successfully created and contains UUID\" >> /tmp/limine-debug.log; \
                      fi; \
                    fi && \
                    # Output file content for verification \
                    echo \"Debug: Final limine.conf content:\" >> /tmp/limine-debug.log && \
                    cat /boot/EFI/limine/limine.conf >> /tmp/limine-debug.log"
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
                    "echo \"Debug: Verifying pacman hook file\" >> /tmp/limine-debug.log && \
                     if [ ! -f /etc/pacman.d/hooks/99-limine.hook ] || \
                        ! grep -q '^Target = limine' /etc/pacman.d/hooks/99-limine.hook || \
                        ! grep -q '^Exec = /usr/bin/cp ' /etc/pacman.d/hooks/99-limine.hook; then \
                       echo \"Debug: Pacman hook file missing or invalid, creating it\" >> /tmp/limine-debug.log; \
                       printf '%s\\n' '[Trigger]' 'Operation = Install' 'Operation = Upgrade' 'Type = Package' 'Target = limine' '' '[Action]' 'Description = Deploying Limine after upgrade...' 'When = PostTransaction' 'Exec = /usr/bin/cp /usr/share/limine/BOOTX64.EFI /boot/EFI/limine/' | tee /etc/pacman.d/hooks/99-limine.hook >/dev/null; \
                       echo \"Debug: Created pacman hook file\" >> /tmp/limine-debug.log; \
                     else \
                       echo \"Debug: Pacman hook file exists and is valid\" >> /tmp/limine-debug.log; \
                     fi && \
                     cat /etc/pacman.d/hooks/99-limine.hook >> /tmp/limine-debug.log"
                ));
                
                // Copy debug logs to host system for easier review when --debug is used
                state.debug_log("limine: copy debug logs to host filesystem for easier access");
                cmds.push(chroot_cmd(
                    "cp /tmp/limine-debug.log /mnt/limine-debug.log 2>/dev/null || true"
                ));
                
                // Final verification of limine.conf UUID
                state.debug_log("limine: final verification of limine.conf contents");
                cmds.push(chroot_cmd(
                    "echo \"===== FINAL VERIFICATION =====\" >> /tmp/limine-debug.log && \
                     if [ -f /boot/EFI/limine/limine.conf ]; then \
                       echo \"limine.conf found, checking content:\" >> /tmp/limine-debug.log && \
                       cat /boot/EFI/limine/limine.conf >> /tmp/limine-debug.log && \
                       if grep -q \"root=UUID=\" /boot/EFI/limine/limine.conf; then \
                         echo \"SUCCESS: UUID found in limine.conf\" >> /tmp/limine-debug.log; \
                       else \
                         echo \"WARNING: UUID NOT found in limine.conf\" >> /tmp/limine-debug.log; \
                         # Last attempt to fix using fallback method with /dev path instead of UUID \
                         rootdev=$(findmnt -n -o SOURCE / 2>/dev/null || echo \"/dev/{part3}\"); \
                         echo \"Creating fallback config with direct device path: $rootdev\" >> /tmp/limine-debug.log && \
                         printf \"%s\\n\" \"timeout: 5\" \"\" \"/Arch Linux\" \"    protocol: linux\" \"    path: boot():/vmlinuz-linux\" \"    cmdline: root=$rootdev rw\" \"    module_path: boot():/initramfs-linux.img\" | tee /boot/EFI/limine/limine.conf >/dev/null; \
                       fi; \
                     else \
                       echo \"ERROR: limine.conf not found\" >> /tmp/limine-debug.log; \
                     fi && \
                     cp /tmp/limine-debug.log /mnt/limine-debug.log 2>/dev/null || true"
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