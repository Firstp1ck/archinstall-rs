use crate::core::state::AppState;
// Removed unused File/Write imports
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
                // Preview cmdline computed by Rust for logging purposes
                let rust_cmdline_preview = detect_root_cmdline(state);
                state.debug_log(&format!("limine: rust_cmdline_preview={}", rust_cmdline_preview));
                // Build entries using a $cmdline placeholder to be computed inside chroot
                let mut entries_tpl = String::new();
                for k in &kernels {
                    entries_tpl.push_str(&format!(
                        "/Arch Linux ({k})\nprotocol: linux\npath: boot():/vmlinuz-{k}\ncmdline: $cmdline\nmodule_path: boot():/initramfs-{k}.img\n\n/Arch Linux ({k}) (fallback)\nprotocol: linux\npath: boot():/vmlinuz-{k}\ncmdline: $cmdline\nmodule_path: boot():/initramfs-{k}-fallback.img\n\n"
                    ));
                }
                // Compute cmdline inside chroot and write the config with variable expansion
                let script_tpl = r#"install -d -m0755 /boot/EFI/limine /boot/EFI/BOOT /boot/limine; \
root_spec=$(awk '($1 !~ /^#/ && $2=="/"){print $1; exit}' /etc/fstab 2>/dev/null || true); \
root_opts=$(awk '($1 !~ /^#/ && $2=="/"){print $4; exit}' /etc/fstab 2>/dev/null || true); \
cmdline=""; \
if [ -n "$root_spec" ]; then \
  case "$root_spec" in \
    UUID=*|PARTUUID=*|LABEL=*|/dev/*) cmdline="root=$root_spec rw" ;; \
    *) cmdline="root=$root_spec rw" ;; \
  esac; \
fi; \
if [ -z "$cmdline" ]; then \
  root_src=$(findmnt -n -o SOURCE / 2>/dev/null || true); \
  root_uuid=$([ -n "$root_src" ] && blkid -s UUID -o value "$root_src" 2>/dev/null || true); \
  partuuid=$([ -n "$root_src" ] && blkid -s PARTUUID -o value "$root_src" 2>/dev/null || true); \
  if [ -e /dev/mapper/cryptroot ]; then \
    luks_dev=$(lsblk -no pkname "$root_src" 2>/dev/null | head -n1); \
    case "$luks_dev" in /*) ;; *) [ -n "$luks_dev" ] && luks_dev=/dev/"$luks_dev" ;; esac; \
    luks_uuid=""; [ -n "$luks_dev" ] && luks_uuid=$(blkid -s UUID -o value "$luks_dev" 2>/dev/null || true); \
    cmdline="root=/dev/mapper/cryptroot rw"; \
    [ -n "$luks_uuid" ] && cmdline="cryptdevice=UUID=$luks_uuid:cryptroot $cmdline"; \
  else \
    if [ -n "$partuuid" ]; then cmdline="root=PARTUUID=$partuuid rw"; \
    elif [ -n "$root_uuid" ]; then cmdline="root=UUID=$root_uuid rw"; \
    elif [ -n "$root_src" ]; then cmdline="root=$root_src rw"; \
    else cmdline="root=/dev/root rw"; fi; \
  fi; \
fi; \
# Final safety: if still empty, try blkid on / and default to /dev/root \
if [ -z "$cmdline" ]; then \
  tmp=$(blkid -s UUID -o value $(findmnt -n -o SOURCE / 2>/dev/null) 2>/dev/null || true); \
  if [ -n "$tmp" ]; then cmdline="root=UUID=$tmp rw"; else cmdline="root=/dev/root rw"; fi; \
fi; \
subvol=$(printf %s "$root_opts" | tr ',' '\n' | grep -E '^(subvol(=|id=).*)$' -m1 2>/dev/null || true); \
if [ -n "$subvol" ]; then cmdline="$cmdline rootflags=$subvol"; fi; \
cat > /boot/EFI/limine/limine.conf <<EOF\n# resolved cmdline: $cmdline\ntimeout: 5\n__ENTRIES__\nEOF\ncp /boot/EFI/limine/limine.conf /boot/EFI/BOOT/limine.conf || true; \
cp /boot/EFI/limine/limine.conf /boot/limine/limine.conf || true; \
# Also provide limine.cfg for broader compatibility \
cp /boot/EFI/limine/limine.conf /boot/EFI/limine/limine.cfg || true; \
cp /boot/EFI/limine/limine.conf /boot/EFI/BOOT/limine.cfg || true; \
cp /boot/EFI/limine/limine.conf /boot/limine/limine.cfg || true;"#;
                let write_conf_cmd = script_tpl.replace("__ENTRIES__", &entries_tpl);
                cmds.push(chroot_cmd(&write_conf_cmd));
                // Install Limine binaries and copy config
                if state.is_uefi() {
                    // Ensure limine package is installed to provide BOOTX64.EFI
                    cmds.push(chroot_cmd("pacman -Sy --noconfirm limine || true"));
                    cmds.push(chroot_cmd("install -d -m0755 /boot/EFI/limine /boot/EFI/BOOT /boot/limine"));
                    cmds.push(chroot_cmd("if [ -f /usr/share/limine/BOOTX64.EFI ]; then cp /usr/share/limine/BOOTX64.EFI /boot/EFI/limine/; else echo 'Warning: /usr/share/limine/BOOTX64.EFI not found' >&2; fi"));
                    // Fallback BOOT path
                    cmds.push(chroot_cmd("cp /boot/EFI/limine/BOOTX64.EFI /boot/EFI/BOOT/BOOTX64.EFI || true"));
                    cmds.push(chroot_cmd("cp /boot/EFI/limine/limine.conf /boot/EFI/BOOT/limine.conf || true"));
                    // Try to add NVRAM entry
                    cmds.push(chroot_cmd(
                        "dev=$(findmnt -n -o SOURCE /boot) || true; \
if [ -n \"$dev\" ]; then \
  disk=/dev/$(lsblk -no pkname \"$dev\"); \
  part=$(lsblk -no PARTNUM \"$dev\" 2>/dev/null | sed -e \"s/[[:space:]]//g\"); \
  if mountpoint -q /sys/firmware/efi/efivars || mount -t efivarfs efivarfs /sys/firmware/efi/efivars 2>/dev/null; then \
    if echo \"$part\" | grep -qE \"^[0-9]+$\"; then \
      loader=\\\\EFI\\\\limine\\\\BOOTX64.EFI; \
      timeout 5 efibootmgr --create --disk \"$disk\" --part \"$part\" --loader \"$loader\" --label \"Limine\" --unicode || true; \
    fi; \
  fi; \
fi"
                    ));
                    // Pacman hook to redeploy BOOTX64.EFI after limine upgrades
                    cmds.push(chroot_cmd("install -d -m0755 /etc/pacman.d/hooks"));
                    cmds.push(chroot_cmd(
                        "cat > /etc/pacman.d/hooks/99-limine.hook <<EOF\n[Trigger]\nOperation = Install\nOperation = Upgrade\nType = Package\nTarget = limine\n\n[Action]\nDescription = Deploying Limine after upgrade...\nWhen = PostTransaction\nExec = /usr/bin/cp /usr/share/limine/BOOTX64.EFI /boot/EFI/limine/\nEOF"
                    ));
                } else {
                    // BIOS
                    cmds.push(chroot_cmd("pacman -Sy --noconfirm limine || true"));
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
    // Prefer device mounted at /mnt (target root) by parsing proc mounts to avoid picking live ISO root
    fn dev_for_mount(mnt: &str) -> Option<String> {
        for path in ["/proc/self/mounts", "/proc/mounts", "/etc/mtab"] {
            if let Ok(data) = std::fs::read_to_string(path) {
                for line in data.lines() {
                    if line.trim_start().starts_with('#') { continue; }
                    let mut it = line.split_whitespace();
                    if let (Some(dev), Some(mp)) = (it.next(), it.next()) {
                        if mp == mnt { return Some(dev.to_string()); }
                    }
                }
            }
        }
        None
    }

    let root_src = dev_for_mount("/mnt")
        .or_else(|| get_cmd_output(Command::new("findmnt").args(["-n", "-o", "SOURCE", "/mnt"])) )
        .or_else(|| dev_for_mount("/"))
        .unwrap_or_else(|| "/dev/root".to_string());

    let partuuid = get_cmd_output(Command::new("blkid").args(["-s", "PARTUUID", "-o", "value", &root_src]));
    let root_uuid = get_cmd_output(Command::new("blkid").args(["-s", "UUID", "-o", "value", &root_src]));

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

    let root_arg = if let Some(pu) = partuuid { format!("root=PARTUUID={} rw", pu) }
                   else if let Some(ru) = root_uuid { format!("root=UUID={} rw", ru) }
                   else { format!("root={} rw", root_src) };

    match crypt_cmd { Some(prefix) => format!("{}root=/dev/mapper/cryptroot rw", prefix), None => root_arg }
}
