use crate::core::services::bootloader::{BootloaderService, kernel_artifacts};
use crate::core::state::AppState;
use crate::core::storage::StoragePlan;

#[derive(Clone, Debug)]
pub struct SysConfigPlan {
    pub commands: Vec<String>,
}

impl SysConfigPlan {
    pub fn new(commands: Vec<String>) -> Self {
        Self { commands }
    }
}

pub struct SysConfigService;

impl SysConfigService {
    pub fn build_plan(state: &AppState, storage_plan: &StoragePlan) -> SysConfigPlan {
        let mut cmds: Vec<String> = Vec::new();
        let encrypted = storage_plan.has_encryption();
        let uki = BootloaderService::uki_requested(state);
        let boot_options_script = BootloaderService::boot_options_script(encrypted);

        const MKINITCPIO_P: &str = "out=$(mkinitcpio -P 2>&1); rc=$?; printf '%s\\n' \"$out\"; if [ \"$rc\" -ne 0 ]; then if printf '%s\\n' \"$out\" | grep -q '^==> ERROR:'; then exit \"$rc\"; fi; if printf '%s\\n' \"$out\" | grep -q 'WARNING: errors were encountered during the build'; then echo 'mkinitcpio returned warnings-only non-zero exit; continuing install' >&2; else exit \"$rc\"; fi; fi";

        // Helper: wrap a command to run inside the target system via arch-chroot
        fn chroot_cmd(inner: &str) -> String {
            // Escape single quotes for safe embedding within single-quoted bash -lc
            let escaped = inner.replace("'", "'\\''");
            format!("arch-chroot /mnt bash -lc '{escaped}'")
        }

        // Timezone and hardware clock
        let timezone = if state.timezone_value.is_empty() {
            "UTC".to_string()
        } else {
            state.timezone_value.clone()
        };
        cmds.push(chroot_cmd(&format!(
            "ln -sf /usr/share/zoneinfo/{timezone} /etc/localtime"
        )));
        cmds.push(chroot_cmd("hwclock --systohc"));

        // Locale configuration
        let language = state
            .locale_language_options
            .get(state.locale_language_index)
            .cloned()
            .unwrap_or_else(|| "en_US.UTF-8".to_string());
        let encoding = state
            .locale_language_to_encoding
            .get(&language)
            .cloned()
            .unwrap_or_else(|| "UTF-8".to_string());
        let locale_gen_line = format!("{language} {encoding}");
        // Ensure desired locale line is uncommented or appended in /etc/locale.gen
        cmds.push(chroot_cmd(&format!(
            "sed -i 's/^#\\s*{locale_gen_line}/{locale_gen_line}/' /etc/locale.gen"
        )));
        cmds.push(chroot_cmd(&format!(
            "grep -q '^{locale_gen_line}$' /etc/locale.gen || echo '{locale_gen_line}' >> /etc/locale.gen"
        )));
        cmds.push(chroot_cmd("locale-gen"));

        // /etc/locale.conf
        cmds.push(chroot_cmd(&format!(
            "printf 'LANG=%s\\n' '{language}' > /etc/locale.conf"
        )));

        // /etc/vconsole.conf (keyboard layout)
        let keymap = state
            .keyboard_layout_options
            .get(state.keyboard_layout_index)
            .cloned()
            .unwrap_or_else(|| "us".to_string());
        cmds.push(chroot_cmd(&format!(
            "printf 'KEYMAP=%s\\n' '{keymap}' > /etc/vconsole.conf"
        )));

        // Hostname and hosts
        let hostname = if state.hostname_value.is_empty() {
            "archlinux".to_string()
        } else {
            state.hostname_value.clone()
        };
        cmds.push(chroot_cmd(&format!(
            "printf '%s\\n' '{hostname}' > /etc/hostname"
        )));
        cmds.push(chroot_cmd(&format!(
            "printf '%s\\n%s\\n%s\\n' '127.0.0.1   localhost' '::1         localhost' '127.0.1.1   {hostname}.localdomain {hostname}' > /etc/hosts"
        )));

        // Enable NetworkManager if chosen
        if state.network_mode_index == 2 {
            cmds.push("systemctl --root=/mnt enable NetworkManager".into());
        }

        // Enable NTP if chosen (avoid timedatectl in chroot)
        if state.ats_enabled {
            cmds.push("systemctl --root=/mnt enable systemd-timesyncd".into());
        }

        // Enable SSH daemon when the SSH server type is selected
        // (openssh installs the `sshd.service` unit on Arch).
        let wants_sshd = state.selected_server_types.contains("sshd");
        let openssh_in_server_pkgs = state
            .selected_server_packages
            .get("sshd")
            .map_or(false, |set| set.contains("openssh"));
        let openssh_in_additional_pkgs = state
            .additional_packages
            .iter()
            .any(|p| p.name == "openssh");
        if wants_sshd && (openssh_in_server_pkgs || openssh_in_additional_pkgs) {
            cmds.push("systemctl --root=/mnt enable sshd".into());
        }

        // Root password (set only if provided and confirmed)
        if !state.root_password.is_empty() && state.root_password == state.root_password_confirm {
            if state.dry_run {
                let cmd = "echo \"root:<REDACTED>\" | chpasswd";
                cmds.push(chroot_cmd(cmd));
            } else {
                let pw_escaped = state.root_password.replace('"', "\\\"");
                let cmd = format!("echo \"root:{pw_escaped}\" | chpasswd");
                cmds.push(chroot_cmd(&cmd));
            }
        }

        // AUR setup (optional)
        if state.aur_selected {
            // Create an unprivileged build user and allow passwordless pacman for dependency install
            cmds.push(chroot_cmd(
                "id -u aurbuild >/dev/null 2>&1 || useradd -m -s /bin/bash aurbuild",
            ));
            cmds.push(chroot_cmd(
                "install -d -m0755 /etc/sudoers.d && printf '%s\n' 'aurbuild ALL=(ALL) NOPASSWD: /usr/bin/pacman' > /etc/sudoers.d/aurbuild && chmod 0440 /etc/sudoers.d/aurbuild",
            ));

            match state.aur_helper_index {
                Some(1) => {
                    // paru (Rust toolchain needed)
                    cmds.push(chroot_cmd(
                        "pacman -Syu --noconfirm --needed base-devel git rust",
                    ));
                    cmds.push(chroot_cmd(
                        "sudo -u aurbuild bash -lc 'cd /tmp && rm -rf paru && git clone https://aur.archlinux.org/paru.git && cd paru && makepkg -si --noconfirm'",
                    ));
                    cmds.push(chroot_cmd(
                        "sudo -u aurbuild bash -lc 'paru -Syu --noconfirm'",
                    ));
                }
                _ => {
                    // yay (Go toolchain needed)
                    cmds.push(chroot_cmd(
                        "pacman -Syu --noconfirm --needed base-devel git go",
                    ));
                    cmds.push(chroot_cmd(
                        "sudo -u aurbuild bash -lc 'cd /tmp && rm -rf yay && git clone https://aur.archlinux.org/yay.git && cd yay && makepkg -si --noconfirm'",
                    ));
                    cmds.push(chroot_cmd(
                        "sudo -u aurbuild bash -lc 'yay -Syu --noconfirm'",
                    ));
                }
            }
            // Cleanup temporary build user and artifacts
            cmds.push(chroot_cmd("rm -rf /tmp/yay /tmp/paru || true"));
            cmds.push(chroot_cmd("rm -f /etc/sudoers.d/aurbuild || true"));
            cmds.push(chroot_cmd("userdel -r aurbuild || true"));
        }

        // UKI: kernel cmdline file, output dir, and preset per kernel must exist before mkinitcpio -P.
        let esp = storage_plan.esp_chroot_mountpoint();
        if uki {
            cmds.push(chroot_cmd(&format!(
                "install -d -m 0755 /etc/kernel && OPTS=$({boot_options_script}) && printf '%s\\n' \"$OPTS\" > /etc/kernel/cmdline"
            )));
            cmds.push(chroot_cmd(&format!("install -d -m 0755 {esp}/EFI/Linux")));
            for kernel in state.selected_kernels.iter() {
                let ka = kernel_artifacts(kernel);
                cmds.push(chroot_cmd(&format!(
                    "PRESET=/etc/mkinitcpio.d/{preset}; \
                     if [ -f \"$PRESET\" ]; then \
                       sed -i -E 's/^default_image=/#default_image=/' \"$PRESET\"; \
                       sed -i -E 's/^fallback_image=/#fallback_image=/' \"$PRESET\"; \
                       sed -i -E 's|^#?default_uki=.*|default_uki=\"{esp}/EFI/Linux/{uki_default}\"|' \"$PRESET\"; \
                       grep -q '^default_uki=' \"$PRESET\" || printf '%s\\n' 'default_uki=\"{esp}/EFI/Linux/{uki_default}\"' >> \"$PRESET\"; \
                       sed -i -E 's|^#?fallback_uki=.*|fallback_uki=\"{esp}/EFI/Linux/{uki_fallback}\"|' \"$PRESET\"; \
                       grep -q '^fallback_uki=' \"$PRESET\" || printf '%s\\n' 'fallback_uki=\"{esp}/EFI/Linux/{uki_fallback}\"' >> \"$PRESET\"; \
                     fi",
                    preset = ka.preset,
                    uki_default = ka.uki_default,
                    uki_fallback = ka.uki_fallback,
                )));
            }
        }

        // mkinitcpio: for LUKS, ensure the correct encrypt hook is present.
        // Modern Arch (mkinitcpio >=37) defaults to `systemd` hooks → use `sd-encrypt`.
        // Older ISOs still ship `udev` hooks → need `encrypt` instead.  Detect which is
        // active and insert the matching hook after `block` (idempotent: skip if already there).
        // Also ensure `systemd` users have `sd-vconsole` (not `keymap consolefont`).
        if encrypted {
            cmds.push(chroot_cmd(
                "if grep -qP '^HOOKS=.*\\bsystemd\\b' /etc/mkinitcpio.conf; then \
                   grep -qP '^HOOKS=.*\\bsd-encrypt\\b' /etc/mkinitcpio.conf || \
                     sed -i '/^HOOKS=/s/\\bblock\\b/block sd-encrypt/' /etc/mkinitcpio.conf; \
                 else \
                   grep -qP '^HOOKS=.*\\bencrypt\\b' /etc/mkinitcpio.conf || \
                     sed -i '/^HOOKS=/s/\\bblock\\b/block encrypt/' /etc/mkinitcpio.conf; \
                 fi",
            ));
        }
        if encrypted || uki {
            cmds.push(chroot_cmd(MKINITCPIO_P));
        }

        // Debug summary (log only, do not add to command list)
        state.debug_log(&format!(
            "sysconfig: hostname={} timezone={} ats={} kernels={} addpkgs={} sudoers_edits={} aur_selected={} aur_helper={} uki={}",
            hostname,
            timezone,
            state.ats_enabled,
            state.selected_kernels.len(),
            state.additional_packages.len(),
            if state.aur_selected { 1 } else { 0 },
            state.aur_selected,
            state
                .aur_helper_index
                .map(|i| if i == 1 { "paru" } else { "yay" })
                .unwrap_or("none"),
            uki
        ));

        SysConfigPlan::new(cmds)
    }
}
