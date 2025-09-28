use crate::core::state::AppState;
use std::fs::{File, create_dir_all};
use std::io::Write;
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
                // Limine (UEFI/BIOS) - Rust-native config generation
                let mut kernels: Vec<String> = if state.selected_kernels.is_empty() {
                    vec!["linux".to_string()]
                } else {
                    state.selected_kernels.iter().cloned().collect()
                };
                kernels.sort();
                if kernels.is_empty() {
                    kernels.push("linux".to_string());
                }
                let cmdline = detect_root_cmdline(state);
                let mut limine_conf = String::from("timeout 4\n");
                for k in &kernels {
                    limine_conf.push_str(&format!(
                        "/Arch Linux ({k})\nprotocol: linux\npath: boot():/vmlinuz-{k}\ncmdline: {cmdline}\nmodule_path: boot():/initramfs-{k}.img\n\n/Arch Linux ({k}) (fallback)\nprotocol: linux\npath: boot():/vmlinuz-{k}\ncmdline: {cmdline}\nmodule_path: boot():/initramfs-{k}-fallback.img\n\n"
                    ));
                }
                // Ensure directory exists and write limine.conf on target root
                if let Err(e) = create_dir_all("/mnt/boot/limine") {
                    state.debug_log(&format!("WARN: create_dir_all(/mnt/boot/limine) failed: {}", e));
                }
                let limine_conf_path = "/mnt/boot/limine/limine.conf";
                match File::create(limine_conf_path).and_then(|mut f| f.write_all(limine_conf.as_bytes())) {
                    Ok(_) => {}
                    Err(e) => {
                        state.debug_log(&format!("WARN: writing {} failed on host: {} — falling back to chroot write", limine_conf_path, e));
                        // Fallback: write inside chroot with a quoted heredoc (no expansion)
                        let heredoc_cmd = format!(
                            "install -d -m0755 /boot/limine; cat > /boot/limine/limine.conf <<'EOF'\n{}\nEOF",
                            limine_conf
                        );
                        cmds.push(chroot_cmd(&heredoc_cmd));
                    }
                }
                // Install Limine binaries as before
                if state.is_uefi() {
                    cmds.push(chroot_cmd("install -d -m0755 /boot/EFI/limine /boot/EFI/BOOT /boot/limine"));
                    cmds.push(chroot_cmd("if [ -f /usr/share/limine/BOOTX64.EFI ]; then cp /usr/share/limine/BOOTX64.EFI /boot/EFI/limine/; else echo 'Warning: /usr/share/limine/BOOTX64.EFI not found' >&2; fi"));
                    cmds.push(chroot_cmd("cp /boot/limine/limine.conf /boot/EFI/BOOT/limine.conf || true; cp /boot/limine/limine.conf /boot/EFI/limine/limine.conf || true"));
                } else {
                    cmds.push(chroot_cmd("install -d -m0755 /boot/limine"));
                    cmds.push(chroot_cmd("if [ -f /usr/share/limine/limine-bios.sys ]; then cp /usr/share/limine/limine-bios.sys /boot/limine/; else echo 'Warning: /usr/share/limine/limine-bios.sys not found' >&2; fi"));
                    cmds.push(chroot_cmd("cp /boot/limine/limine.conf /boot/limine.conf || true; install -d -m0755 /limine || true; cp /boot/limine/limine.conf /limine/limine.conf || true"));
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

fn get_cmd_output(cmd: &mut Command) -> Option<String> {
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn detect_root_cmdline(state: &AppState) -> String {
    // Detect the device mounted at /mnt (target root)
    let root_src = get_cmd_output(Command::new("findmnt").args(["-n", "-o", "SOURCE", "/mnt"]))
        .or_else(|| get_cmd_output(Command::new("findmnt").args(["-n", "-o", "SOURCE", "/"])) )
        .unwrap_or_else(|| "/dev/root".to_string());

    // Prefer PARTUUID, else UUID, else device path
    let partuuid = get_cmd_output(Command::new("blkid").args(["-s", "PARTUUID", "-o", "value", &root_src]));
    let root_uuid = get_cmd_output(Command::new("blkid").args(["-s", "UUID", "-o", "value", &root_src]));

    // If encryption is used, attempt to derive cryptdevice spec from the parent block device
    let mut crypt_cmd: Option<String> = None;
    if state.disk_encryption_type_index == 1 {
        if let Some(pkname) = get_cmd_output(Command::new("lsblk").args(["-no", "pkname", &root_src])) {
            let dev = if pkname.starts_with("/dev/") { pkname } else { format!("/dev/{}", pkname) };
            let typ = get_cmd_output(Command::new("blkid").args(["-s", "TYPE", "-o", "value", &dev]));
            if matches!(typ.as_deref(), Some("crypto_LUKS")) {
                if let Some(luks_uuid) = get_cmd_output(Command::new("blkid").args(["-s", "UUID", "-o", "value", &dev])) {
                    crypt_cmd = Some(format!("cryptdevice=UUID={}:cryptroot ", luks_uuid));
                }
            }
        }
    }

    // Compose final kernel cmdline
    let root_arg = if let Some(pu) = partuuid { format!("root=PARTUUID={} rw", pu) }
                   else if let Some(ru) = root_uuid { format!("root=UUID={} rw", ru) }
                   else { format!("root={} rw", root_src) };

    match crypt_cmd {
        Some(prefix) => format!("{}root=/dev/mapper/cryptroot rw", prefix),
        None => root_arg,
    }
}
