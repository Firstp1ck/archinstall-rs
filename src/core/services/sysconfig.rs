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
            cmds.push(chroot_cmd("systemctl enable NetworkManager"));
        }

        // Enable NTP if chosen
        if state.ats_enabled {
            cmds.push(chroot_cmd("systemctl enable systemd-timesyncd"));
            cmds.push(chroot_cmd("timedatectl set-ntp true"));
        }

        // Root password (set only if provided and confirmed)
        if !state.root_password.is_empty() && state.root_password == state.root_password_confirm {
            if state.dry_run {
                let cmd = "echo \"root:<REDACTED>\" | chpasswd";
                cmds.push(chroot_cmd(cmd));
            } else {
                let pw_escaped = state.root_password.replace('"', "\\\"");
                let cmd = format!(
                    "echo \"root:{}\" | chpasswd",
                    pw_escaped
                );
                cmds.push(chroot_cmd(&cmd));
            }
        }

        SysConfigPlan::new(cmds)
    }
}


