use crate::core::state::AppState;

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
    pub fn build_plan(state: &AppState) -> SysConfigPlan {
        let mut cmds: Vec<String> = Vec::new();
        // TODO: Add keyboard layout for Xorg/Wayland DEs beyond Hyprland (v0.3.0 UX).
        // TODO: Add additional system configuration (hosts, mkinitcpio hooks for LUKS/UKI) (v0.2.0+).

        // Helper: wrap a command to run inside the target system via arch-chroot
        fn chroot_cmd(inner: &str) -> String {
            // Escape single quotes for safe embedding within single-quoted bash -lc
            let escaped = inner.replace("'", "'\\''");
            format!("arch-chroot /mnt bash -lc '{}'", escaped)
        }

        // Timezone and hardware clock
        let timezone = if state.timezone_value.is_empty() {
            "UTC".to_string()
        } else {
            state.timezone_value.clone()
        };
        cmds.push(chroot_cmd(&format!(
            "ln -sf /usr/share/zoneinfo/{} /etc/localtime",
            timezone
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
        let locale_gen_line = format!("{} {}", language, encoding);
        // Ensure desired locale line is uncommented or appended in /etc/locale.gen
        cmds.push(chroot_cmd(&format!(
            "sed -i 's/^#\\s*{0}/{0}/' /etc/locale.gen",
            locale_gen_line
        )));
        cmds.push(chroot_cmd(&format!(
            "grep -q '^{}$' /etc/locale.gen || echo '{}' >> /etc/locale.gen",
            locale_gen_line, locale_gen_line
        )));
        cmds.push(chroot_cmd("locale-gen"));

        // /etc/locale.conf
        cmds.push(chroot_cmd(&format!(
            "printf 'LANG=%s\\n' '{}' > /etc/locale.conf",
            language
        )));

        // /etc/vconsole.conf (keyboard layout)
        let keymap = state
            .keyboard_layout_options
            .get(state.keyboard_layout_index)
            .cloned()
            .unwrap_or_else(|| "us".to_string());
        cmds.push(chroot_cmd(&format!(
            "printf 'KEYMAP=%s\\n' '{}' > /etc/vconsole.conf",
            keymap
        )));

        // Hostname and hosts
        let hostname = if state.hostname_value.is_empty() {
            "archlinux".to_string()
        } else {
            state.hostname_value.clone()
        };
        cmds.push(chroot_cmd(&format!(
            "printf '%s\\n' '{}' > /etc/hostname",
            hostname
        )));
        cmds.push(chroot_cmd(&format!(
            "printf '%s\\n%s\\n%s\\n' '127.0.0.1   localhost' '::1         localhost' '127.0.1.1   {0}.localdomain {0}' > /etc/hosts",
            hostname
        )));

        // Enable NetworkManager if chosen
        if state.network_mode_index == 2 {
            cmds.push("systemctl --root=/mnt enable NetworkManager".into());
        }

        // Enable NTP if chosen (avoid timedatectl in chroot)
        if state.ats_enabled {
            cmds.push("systemctl --root=/mnt enable systemd-timesyncd".into());
        }

        // Root password (set only if provided and confirmed)
        if !state.root_password.is_empty() && state.root_password == state.root_password_confirm {
            if state.dry_run {
                let cmd = "echo \"root:<REDACTED>\" | chpasswd";
                cmds.push(chroot_cmd(cmd));
            } else {
                let pw_escaped = state.root_password.replace('"', "\\\"");
                let cmd = format!("echo \"root:{}\" | chpasswd", pw_escaped);
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

        // Debug summary (log only, do not add to command list)
        state.debug_log(&format!(
            "sysconfig: hostname={} timezone={} ats={} kernels={} addpkgs={} sudoers_edits={} aur_selected={} aur_helper={}",
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
                .unwrap_or("none")
        ));

        SysConfigPlan::new(cmds)
    }
}
