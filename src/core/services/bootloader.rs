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

/// Derived filenames for a kernel package (e.g. `linux`, `linux-lts`).
pub struct KernelArtifacts {
    pub vmlinuz: String,
    pub initramfs: String,
    pub initramfs_fallback: String,
    pub uki_default: String,
    pub uki_fallback: String,
    pub preset: String,
}

pub fn kernel_artifacts(pkg: &str) -> KernelArtifacts {
    KernelArtifacts {
        vmlinuz: format!("vmlinuz-{pkg}"),
        initramfs: format!("initramfs-{pkg}.img"),
        initramfs_fallback: format!("initramfs-{pkg}-fallback.img"),
        uki_default: format!("arch-{pkg}.efi"),
        uki_fallback: format!("arch-{pkg}-fallback.efi"),
        preset: format!("{pkg}.preset"),
    }
}

/// Detect CPU microcode from `/proc/cpuinfo`.
/// Returns the `.img` filename (e.g. `"intel-ucode.img"`) or `None` when
/// the CPU vendor is unrecognised or `/proc/cpuinfo` is unavailable (dry-run / CI).
pub(crate) fn detect_microcode() -> Option<&'static str> {
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    let lower = cpuinfo.to_lowercase();
    if cpuinfo.contains("GenuineIntel") || lower.contains("intel") {
        Some("intel-ucode.img")
    } else if cpuinfo.contains("AuthenticAMD") || lower.contains("amd") {
        Some("amd-ucode.img")
    } else {
        None
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

    /// Returns true when UKI install integration should run (`uki_enabled` and bootloader is not GRUB).
    ///
    /// Same rule as UKI TUI visibility (hidden for GRUB).
    pub(crate) fn uki_requested(state: &AppState) -> bool {
        state.uki_enabled && state.bootloader_index != 1
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
        esp: &str,
    ) -> Vec<String> {
        fn chroot_cmd(inner: &str) -> String {
            let escaped = inner.replace("'", "'\\''");
            format!("arch-chroot /mnt bash -lc '{escaped}'")
        }
        let mut out: Vec<String> = Vec::new();
        let uki = Self::uki_requested(state)
            || (state.bootloader_index == 2 && state.is_secure_boot_enabled());
        let ucode = detect_microcode();

        if uki {
            // Build UKI limine.conf entries for each selected kernel
            let mut conf = format!("cat > {esp}/limine.conf <<'LIMINEOF'\ntimeout: 5\n");
            for kernel in state.selected_kernels.iter() {
                let ka = kernel_artifacts(kernel);
                let suffix = if kernel == "linux" {
                    String::new()
                } else {
                    format!(" ({kernel})")
                };
                conf.push_str(&format!(
                    "\n/Arch Linux{suffix}\n    protocol: efi\n    path: boot():/EFI/Linux/{uki_default}\n\
                     \n/Arch Linux{suffix} (fallback UKI)\n    protocol: efi\n    path: boot():/EFI/Linux/{uki_fallback}\n",
                    uki_default = ka.uki_default,
                    uki_fallback = ka.uki_fallback,
                ));
            }
            conf.push_str("LIMINEOF");
            out.push(chroot_cmd(&conf));
        } else {
            // Build non-UKI limine.conf entries for each selected kernel
            let mut conf = format!(
                "OPTS=$({boot_options_script}); cat > {esp}/limine.conf <<LIMINEOF\ntimeout: 5\n"
            );
            for kernel in state.selected_kernels.iter() {
                let ka = kernel_artifacts(kernel);
                let suffix = if kernel == "linux" {
                    String::new()
                } else {
                    format!(" ({kernel})")
                };
                let ucode_line = ucode
                    .map(|u| format!("    module_path: boot():/{u}\n"))
                    .unwrap_or_default();
                conf.push_str(&format!(
                    "\n/Arch Linux{suffix}\n    protocol: linux\n    path: boot():/{vmlinuz}\n    cmdline: $OPTS\n{ucode_line}    module_path: boot():/{initramfs}\n\
                     \n/Arch Linux{suffix} (fallback initramfs)\n    protocol: linux\n    path: boot():/{vmlinuz}\n    cmdline: $OPTS\n{ucode_line}    module_path: boot():/{initramfs_fb}\n",
                    vmlinuz = ka.vmlinuz,
                    initramfs = ka.initramfs,
                    initramfs_fb = ka.initramfs_fallback,
                ));
            }
            conf.push_str("LIMINEOF");
            out.push(chroot_cmd(&conf));
        }
        if state.is_uefi() {
            out.push(chroot_cmd(&format!(
                "install -d -m 0755 {esp}/EFI/limine && \
                 install -m 0644 /usr/share/limine/BOOTX64.EFI {esp}/EFI/limine/BOOTX64.EFI && \
                 install -d -m 0755 {esp}/EFI/BOOT && \
                 install -m 0644 /usr/share/limine/BOOTX64.EFI {esp}/EFI/BOOT/BOOTX64.EFI"
            )));
            out.push(chroot_cmd(&format!(
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
Exec = /bin/sh -c \"/usr/bin/install -Dm 0644 /usr/share/limine/BOOTX64.EFI {esp}/EFI/limine/BOOTX64.EFI && /usr/bin/install -Dm 0644 /usr/share/limine/BOOTX64.EFI {esp}/EFI/BOOT/BOOTX64.EFI\"\n\
HOOK_EOF"
            )));
            out.push(chroot_cmd(&format!(
                "if mountpoint -q /sys/firmware/efi/efivars || mount -t efivarfs efivarfs /sys/firmware/efi/efivars 2>/dev/null; then \
                 BOOTSRC=$(findmnt -n -o SOURCE {esp}); \
                 DISK=$(lsblk -no pkname \"$BOOTSRC\"); \
                 PART=$(lsblk -no PARTN \"$BOOTSRC\"); \
                 efibootmgr --create --disk \"/dev/$DISK\" --part \"$PART\" --label 'Arch Linux Limine' --loader '\\\\EFI\\\\limine\\\\BOOTX64.EFI' --unicode || \
                 echo 'WARNING: efibootmgr failed to create NVRAM entry; UEFI fallback path EFI/BOOT/BOOTX64.EFI is available'; \
                 first_arch=$(efibootmgr | awk '/^Boot[0-9A-Fa-f]{{4}}\\*/ && /Arch Linux/{{print substr($1,5,4); exit}}'); \
                 if [ -n \"$first_arch\" ]; then \
                   current=$(efibootmgr | awk -F'BootOrder: ' '/BootOrder:/{{print $2}}' | tr -d ' \\r'); \
                   if [ -n \"$current\" ]; then \
                     rest=$(echo \"$current\" | awk -F, -v id=\"$first_arch\" '{{out=\"\"; for(i=1;i<=NF;i++) if($i!=id) out=out (out?\",\":\"\") $i; print out}}'); \
                     efibootmgr -o \"$first_arch${{rest:+,$rest}}\" || true; \
                     efibootmgr -n \"$first_arch\" || true; \
                   fi; \
                 fi; \
                 efibootmgr --verbose || true; \
                 fi"
            )));
        } else {
            out.push(chroot_cmd(&format!(
                "install -d -m 0755 {esp}/limine && install -m 0644 /usr/share/limine/limine-bios.sys {esp}/limine/limine-bios.sys"
            )));
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

        let esp = storage_plan.esp_chroot_mountpoint();

        state.debug_log(&format!(
            "bootloader: build_plan start (uefi={}, bootloader_index={}, device={}, encrypted={}, esp={})",
            state.is_uefi(),
            state.bootloader_index,
            device,
            encrypted,
            esp
        ));

        let boot_options_script = Self::boot_options_script(encrypted);
        let uki = Self::uki_requested(state)
            || (state.bootloader_index == 2 && state.is_secure_boot_enabled());
        let ucode = detect_microcode();

        // Shell snippet to delete stale "Arch Linux" NVRAM entries before creating new ones
        let nvram_cleanup = "for bootnum in $(efibootmgr 2>/dev/null | grep -i 'Arch Linux' | awk '\"'\"'{print $1}'\"'\"' | sed 's/Boot//;s/\\*//'); do \
                             efibootmgr -b \"$bootnum\" -B 2>/dev/null || true; done";

        match state.bootloader_index {
            // 0: systemd-boot
            0 => {
                cmds.push(chroot_cmd(&format!(
                    "env SYSTEMD_PAGER=cat SYSTEMD_COLORS=0 timeout 30s bootctl --no-pager install --no-variables --esp-path={esp} --boot-path={esp}"
                )));

                cmds.push(chroot_cmd(&format!(
                    "install -d -m 0755 {esp}/loader && install -d -m 0755 {esp}/loader/entries"
                )));

                // First kernel determines the default entry name
                let first_kernel = state
                    .selected_kernels
                    .iter()
                    .next()
                    .cloned()
                    .unwrap_or_else(|| "linux".into());
                let default_conf = if first_kernel == "linux" {
                    "arch.conf".to_string()
                } else {
                    format!("arch-{first_kernel}.conf")
                };

                cmds.push(chroot_cmd(&format!(
                    "cat > {esp}/loader/loader.conf <<EOF\ndefault  {default_conf}\ntimeout  4\nconsole-mode auto\neditor   no\nEOF"
                )));

                for kernel in state.selected_kernels.iter() {
                    let ka = kernel_artifacts(kernel);
                    let conf_name = if kernel == "linux" {
                        "arch".to_string()
                    } else {
                        format!("arch-{kernel}")
                    };
                    let title_suffix = if kernel == "linux" {
                        String::new()
                    } else {
                        format!(" ({kernel})")
                    };

                    if uki {
                        cmds.push(chroot_cmd(&format!(
                            "cat > {esp}/loader/entries/{conf_name}.conf <<'EOF'\ntitle   Arch Linux{title_suffix}\nefi     /EFI/Linux/{uki_default}\nEOF",
                            uki_default = ka.uki_default,
                        )));
                        cmds.push(chroot_cmd(&format!(
                            "cat > {esp}/loader/entries/{conf_name}-fallback.conf <<'EOF'\ntitle   Arch Linux{title_suffix} (fallback UKI)\nefi     /EFI/Linux/{uki_fallback}\nEOF",
                            uki_fallback = ka.uki_fallback,
                        )));
                    } else {
                        let ucode_line = ucode
                            .map(|u| format!("initrd  /{u}\\n"))
                            .unwrap_or_default();
                        cmds.push(chroot_cmd(&format!(
                            "OPTS=$({boot_options_script}); cat > {esp}/loader/entries/{conf_name}.conf <<EOF\ntitle   Arch Linux{title_suffix}\nlinux   /{vmlinuz}\n{ucode_line}initrd  /{initramfs}\noptions $OPTS\nEOF",
                            vmlinuz = ka.vmlinuz,
                            initramfs = ka.initramfs,
                        )));
                        cmds.push(chroot_cmd(&format!(
                            "OPTS=$({boot_options_script}); cat > {esp}/loader/entries/{conf_name}-fallback.conf <<EOF\ntitle   Arch Linux{title_suffix} (fallback initramfs)\nlinux   /{vmlinuz}\n{ucode_line}initrd  /{initramfs_fb}\noptions $OPTS\nEOF",
                            vmlinuz = ka.vmlinuz,
                            initramfs_fb = ka.initramfs_fallback,
                        )));
                    }
                }

                cmds.push(chroot_cmd("env SYSTEMD_PAGER=cat SYSTEMD_COLORS=0 timeout 5s bootctl --no-pager list || true"));

                cmds.push(chroot_cmd(&format!(
                    "env SYSTEMD_PAGER=cat SYSTEMD_COLORS=0 timeout 5s bootctl --no-pager status >/dev/null 2>&1 || {{ if mountpoint -q /sys/firmware/efi/efivars || mount -t efivarfs efivarfs /sys/firmware/efi/efivars 2>/dev/null; then timeout 5 efibootmgr --create --disk $(lsblk -no pkname $(findmnt -n -o SOURCE {esp})) --part $(lsblk -no PARTN $(findmnt -n -o SOURCE {esp})) --loader '\\EFI\\systemd\\systemd-bootx64.efi' --label 'Linux Boot Manager' --unicode || true; fi; }}"
                )));
            }
            // 1: grub
            1 => {
                if state.is_uefi() {
                    cmds.push(chroot_cmd(&format!(
                        "grub-install --target=x86_64-efi --efi-directory={esp} --bootloader-id=GRUB"
                    )));
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
                cmds.push(chroot_cmd(&format!("grub-mkconfig -o {esp}/grub/grub.cfg")));
            }
            // 2: EFISTUB — direct kernel boot via firmware
            2 => {
                if state.is_uefi() {
                    let first_kernel = state
                        .selected_kernels
                        .iter()
                        .next()
                        .cloned()
                        .unwrap_or_else(|| "linux".into());
                    let first_ka = kernel_artifacts(&first_kernel);

                    if uki {
                        // Copy primary UKI to UEFI standard fallback path
                        cmds.push(chroot_cmd(&format!(
                            "install -d -m 0755 {esp}/EFI/BOOT && if [ -f {esp}/EFI/Linux/{uki_default} ]; then \
                             install -m 0644 {esp}/EFI/Linux/{uki_default} {esp}/EFI/BOOT/BOOTX64.EFI; \
                             else echo \"WARNING: {esp}/EFI/Linux/{uki_default} missing; UKI fallback copy skipped\"; fi",
                            uki_default = first_ka.uki_default,
                        )));

                        // Pacman hook: refresh fallback copy on kernel upgrade (any selected kernel)
                        let hook_targets: String = state
                            .selected_kernels
                            .iter()
                            .map(|k| format!("Target = {k}\n"))
                            .collect();
                        cmds.push(chroot_cmd(&format!(
                            "install -d -m 0755 /etc/pacman.d/hooks && cat > /etc/pacman.d/hooks/99-efistub-uki-fallback.hook <<HOOK_EOF\n\
[Trigger]\n\
Operation = Install\n\
Operation = Upgrade\n\
Type = Package\n\
{hook_targets}\
\n\
[Action]\n\
Description = Refresh UEFI fallback UKI copy after kernel upgrade\n\
When = PostTransaction\n\
Exec = /bin/sh -c \"for f in {esp}/EFI/Linux/{uki_default} {esp}/EFI/Linux/{uki_fallback}; do [ -f \\\"$f\\\" ] && /usr/bin/install -Dm 0644 \\\"$f\\\" {esp}/EFI/BOOT/BOOTX64.EFI && break; done\"\n\
HOOK_EOF",
                            uki_default = first_ka.uki_default,
                            uki_fallback = first_ka.uki_fallback,
                        )));

                        // efibootmgr: clean stale entries, then register each kernel's UKI
                        let mut efi_script = format!(
                            "if mountpoint -q /sys/firmware/efi/efivars || mount -t efivarfs efivarfs /sys/firmware/efi/efivars 2>/dev/null; then \
                             {nvram_cleanup}; \
                             BOOTSRC=$(findmnt -n -o SOURCE {esp}); \
                             DISK=$(lsblk -no pkname \"$BOOTSRC\"); \
                             PART=$(lsblk -no PARTN \"$BOOTSRC\"); "
                        );
                        for kernel in state.selected_kernels.iter() {
                            let ka = kernel_artifacts(kernel);
                            let label_suffix = if kernel == "linux" {
                                String::new()
                            } else {
                                format!(" ({kernel})")
                            };
                            efi_script.push_str(&format!(
                                "efibootmgr --create --disk \"/dev/$DISK\" --part \"$PART\" --label 'Arch Linux{label_suffix}' --loader '\\\\EFI\\\\Linux\\\\{uki_default}' || \
                                 echo \"WARNING: efibootmgr failed for {kernel} UKI NVRAM entry\"; \
                                 efibootmgr --create --disk \"/dev/$DISK\" --part \"$PART\" --label 'Arch Linux{label_suffix} (fallback UKI)' --loader '\\\\EFI\\\\Linux\\\\{uki_fallback}' || \
                                 echo \"WARNING: efibootmgr failed for {kernel} fallback UKI NVRAM entry\"; ",
                                uki_default = ka.uki_default,
                                uki_fallback = ka.uki_fallback,
                            ));
                        }
                        efi_script.push_str(
                            "first_arch=$(efibootmgr | awk '/^Boot[0-9A-Fa-f]{4}\\*/ && /Arch Linux/{print substr($1,5,4); exit}'); \
                             if [ -n \"$first_arch\" ]; then \
                               current=$(efibootmgr | awk -F'BootOrder: ' '/BootOrder:/{print $2}' | tr -d ' \\r'); \
                               if [ -n \"$current\" ]; then \
                                 rest=$(echo \"$current\" | awk -F, -v id=\"$first_arch\" '{out=\"\"; for(i=1;i<=NF;i++) if($i!=id) out=out (out?\",\":\"\") $i; print out}'); \
                                 efibootmgr -o \"$first_arch${rest:+,$rest}\" || true; \
                                 efibootmgr -n \"$first_arch\" || true; \
                               fi; \
                             fi; \
                             efibootmgr --verbose || true; fi",
                        );
                        cmds.push(chroot_cmd(&efi_script));
                    } else {
                        // Non-UKI: startup.nsh with FS-scanning loop (primary kernel only)
                        // Ensure firmware-loadable artifacts exist on the ESP for EFISTUB paths.
                        for kernel in state.selected_kernels.iter() {
                            let ka = kernel_artifacts(kernel);
                            cmds.push(chroot_cmd(&format!(
                                "install -d -m 0755 {esp}/EFI/Linux && \
                                 for f in /boot/{vmlinuz} /boot/{initramfs} /boot/{initramfs_fb}; do \
                                   if [ -f \"$f\" ]; then \
                                     install -m 0644 \"$f\" {esp}/EFI/Linux/$(basename \"$f\"); \
                                   else \
                                     echo \"EFISTUB-DIAG: missing artifact $f; skipping ESP copy\"; \
                                   fi; \
                                 done",
                                vmlinuz = ka.vmlinuz,
                                initramfs = ka.initramfs,
                                initramfs_fb = ka.initramfs_fallback,
                            )));
                        }
                        if let Some(u) = ucode {
                            cmds.push(chroot_cmd(&format!(
                                "if [ -f /boot/{u} ]; then install -Dm 0644 /boot/{u} {esp}/EFI/Linux/{u}; fi"
                            )));
                        }

                        let ucode_nsh = ucode
                            .map(|u| format!(" initrd=%d:\\EFI\\Linux\\{u}"))
                            .unwrap_or_default();
                        cmds.push(chroot_cmd(&format!(
                            "OPTS=$({boot_options_script}); cat > {esp}/startup.nsh <<'NSHEOF'\n\
@echo -off\n\
for %d in FS0 FS1 FS2 FS3 FS4 FS5 FS6 FS7 FS8 FS9\n\
  if exist %d:\\EFI\\Linux\\{vmlinuz} then\n\
    %d:\\EFI\\Linux\\{vmlinuz} $OPTS{ucode_nsh} initrd=%d:\\EFI\\Linux\\{initramfs}\n\
  endif\n\
endfor\n\
NSHEOF\nchmod 0644 {esp}/startup.nsh",
                            vmlinuz = first_ka.vmlinuz,
                            initramfs = first_ka.initramfs,
                        )));

                        // Pacman hook: regenerate startup.nsh on kernel upgrade
                        let hook_targets: String = state
                            .selected_kernels
                            .iter()
                            .map(|k| format!("Target = {k}\n"))
                            .collect();
                        cmds.push(chroot_cmd(&format!(
                            "install -d -m 0755 /etc/pacman.d/hooks && cat > /etc/pacman.d/hooks/99-efistub-direct.hook <<HOOK_EOF\n\
[Trigger]\n\
Operation = Install\n\
Operation = Upgrade\n\
Type = Package\n\
{hook_targets}\
\n\
[Action]\n\
Description = Refresh EFISTUB startup.nsh after kernel upgrade\n\
When = PostTransaction\n\
Exec = /bin/sh -c \"install -d -m 0755 {esp}/EFI/Linux; for f in /boot/vmlinuz-* /boot/initramfs-*.img /boot/*-ucode.img; do [ -f \\\"\\$f\\\" ] && /usr/bin/install -Dm 0644 \\\"\\$f\\\" {esp}/EFI/Linux/\\$(basename \\\"\\$f\\\"); done; OPTS=$({boot_options_script}); cat > {esp}/startup.nsh <<NSH_INNER\\n@echo -off\\nfor %%d in FS0 FS1 FS2 FS3 FS4 FS5 FS6 FS7 FS8 FS9\\n  if exist %%d:\\\\EFI\\\\Linux\\\\{vmlinuz} then\\n    %%d:\\\\EFI\\\\Linux\\\\{vmlinuz} \\$OPTS{ucode_hook} initrd=%%d:\\\\EFI\\\\Linux\\\\{initramfs}\\n  endif\\nendfor\\nNSH_INNER\"\n\
HOOK_EOF",
                            vmlinuz = first_ka.vmlinuz,
                            initramfs = first_ka.initramfs,
                            ucode_hook = ucode
                                .map(|u| format!(" initrd=%%d:\\\\EFI\\\\Linux\\\\{u}"))
                                .unwrap_or_default(),
                        )));

                        // efibootmgr: clean stale entries, then register each kernel
                        let mut efi_script = format!(
                            "if mountpoint -q /sys/firmware/efi/efivars || mount -t efivarfs efivarfs /sys/firmware/efi/efivars 2>/dev/null; then \
                             {nvram_cleanup}; \
                             OPTS=$({boot_options_script}); \
                             BOOTSRC=$(findmnt -n -o SOURCE {esp}); \
                             DISK=$(lsblk -no pkname \"$BOOTSRC\"); \
                             PART=$(lsblk -no PARTN \"$BOOTSRC\"); \
                             echo \"EFISTUB-DIAG: esp={esp} bootsrc=$BOOTSRC disk=/dev/$DISK part=$PART\"; \
                             if [ -z \"$BOOTSRC\" ] || [ -z \"$DISK\" ] || [ -z \"$PART\" ]; then \
                               echo 'EFISTUB-DIAG: failed to resolve ESP source/disk/partition from findmnt/lsblk'; \
                               findmnt -no SOURCE {esp} || true; \
                               lsblk -f || true; \
                             fi; \
                             ls -l {esp}/EFI/Linux 2>/dev/null || echo 'EFISTUB-DIAG: EFI/Linux missing or unreadable'; "
                        );
                        let ucode_efi = ucode
                            .map(|u| format!("initrd=\\\\\\\\EFI\\\\\\\\Linux\\\\\\\\{u} "))
                            .unwrap_or_default();
                        for kernel in state.selected_kernels.iter() {
                            let ka = kernel_artifacts(kernel);
                            let label_suffix = if kernel == "linux" {
                                String::new()
                            } else {
                                format!(" ({kernel})")
                            };
                            efi_script.push_str(&format!(
                                "for req in {vmlinuz} {initramfs} {initramfs_fb}; do \
                                   if [ ! -f {esp}/EFI/Linux/$req ]; then \
                                     echo \"EFISTUB-DIAG: missing ESP artifact {esp}/EFI/Linux/$req\"; \
                                   fi; \
                                 done; \
                                 out=$(efibootmgr --create --disk \"/dev/$DISK\" --part \"$PART\" --label 'Arch Linux{label_suffix}' --loader '\\\\EFI\\\\Linux\\\\{vmlinuz}' --unicode \"$OPTS {ucode_efi}initrd=\\\\\\\\EFI\\\\\\\\Linux\\\\\\\\{initramfs}\" 2>&1); rc=$?; \
                                 if [ $rc -ne 0 ]; then \
                                   echo \"WARNING: efibootmgr failed; ESP has {esp}/startup.nsh as fallback\"; \
                                   echo \"EFISTUB-DIAG: primary efibootmgr rc=$rc kernel={kernel}\"; \
                                   echo \"$out\"; \
                                 fi; \
                                 out=$(efibootmgr --create --disk \"/dev/$DISK\" --part \"$PART\" --label 'Arch Linux{label_suffix} (fallback initramfs)' --loader '\\\\EFI\\\\Linux\\\\{vmlinuz}' --unicode \"$OPTS {ucode_efi}initrd=\\\\\\\\EFI\\\\\\\\Linux\\\\\\\\{initramfs_fb}\" 2>&1); rc=$?; \
                                 if [ $rc -ne 0 ]; then \
                                   echo \"WARNING: efibootmgr failed for {kernel} fallback NVRAM entry\"; \
                                   echo \"EFISTUB-DIAG: fallback efibootmgr rc=$rc kernel={kernel}\"; \
                                   echo \"$out\"; \
                                 fi; ",
                                vmlinuz = ka.vmlinuz,
                                initramfs = ka.initramfs,
                                initramfs_fb = ka.initramfs_fallback,
                            ));
                        }
                        efi_script.push_str(
                            "first_arch=$(efibootmgr | awk '/^Boot[0-9A-Fa-f]{4}\\*/ && /Arch Linux/{print substr($1,5,4); exit}'); \
                             if [ -n \"$first_arch\" ]; then \
                               current=$(efibootmgr | awk -F'BootOrder: ' '/BootOrder:/{print $2}' | tr -d ' \\r'); \
                               if [ -n \"$current\" ]; then \
                                 rest=$(echo \"$current\" | awk -F, -v id=\"$first_arch\" '{out=\"\"; for(i=1;i<=NF;i++) if($i!=id) out=out (out?\",\":\"\") $i; print out}'); \
                                 efibootmgr -o \"$first_arch${rest:+,$rest}\" || true; \
                                 efibootmgr -n \"$first_arch\" || true; \
                               fi; \
                             fi; \
                             efibootmgr --verbose || true; \
                             else \
                             echo 'WARNING: efivarfs unavailable; skipping EFISTUB NVRAM entry creation'; \
                             echo 'EFISTUB-DIAG: /sys/firmware/efi state and mounts follow'; \
                             ls -ld /sys/firmware/efi /sys/firmware/efi/efivars 2>/dev/null || true; \
                             mount | grep -i efivarfs || true; \
                             fi",
                        );
                        cmds.push(chroot_cmd(&efi_script));
                    }
                }
            }
            3 => {
                cmds.extend(Self::limine_install_chroot_commands(
                    state,
                    device,
                    boot_options_script.as_str(),
                    esp,
                ));
            }
            _ => {}
        }

        state.debug_log(&format!(
            "bootloader: choice={} mode={} (uefi={}, encrypted={}, uki={}, esp={}, kernels={:?})",
            match state.bootloader_index {
                0 => "systemd-boot",
                1 => "grub",
                2 => "efistub",
                3 => "limine",
                _ => "unknown",
            },
            if state.is_uefi() { "UEFI" } else { "BIOS" },
            state.is_uefi(),
            encrypted,
            uki,
            esp,
            state.selected_kernels,
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
