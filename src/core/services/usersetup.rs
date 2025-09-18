use crate::core::state::AppState;

#[derive(Clone, Debug)]
pub struct UserSetupPlan {
    pub commands: Vec<String>,
}

impl UserSetupPlan {
    pub fn new(commands: Vec<String>) -> Self {
        Self { commands }
    }
}

pub struct UserSetupService;

impl UserSetupService {
    pub fn build_plan(state: &AppState) -> UserSetupPlan {
        let mut cmds: Vec<String> = Vec::new();
        // TODO: Support zsh/fish shells and user groups selection in UI (v0.3.0).

        fn chroot_cmd(inner: &str) -> String {
            let escaped = inner.replace("'", "'\\''");
            format!("arch-chroot /mnt bash -lc '{}'", escaped)
        }

        // Create users and set passwords
        for user in state.users.iter() {
            if user.username.trim().is_empty() {
                continue;
            }
            let mut add_args = String::from("-m -s /bin/bash");
            if user.is_sudo {
                add_args.push_str(" -G wheel");
            }
            cmds.push(chroot_cmd(&format!(
                "id -u {0} >/dev/null 2>&1 || useradd {1} {0}",
                user.username, add_args
            )));

            // Set password (redact in dry-run)
            if !user.password.is_empty() {
                if state.dry_run {
                    cmds.push(chroot_cmd(&format!(
                        "echo \"{0}:<REDACTED>\" | chpasswd",
                        user.username
                    )));
                } else {
                    let pw_escaped = user.password.replace('"', "\\\"");
                    cmds.push(chroot_cmd(&format!(
                        "echo \"{0}:{1}\" | chpasswd",
                        user.username, pw_escaped
                    )));
                }
            }
        }

        // Configure sudoers: uncomment wheel and sudo groups
        cmds.push(chroot_cmd(
            r"sed -i 's/^#\s*%wheel ALL=(ALL:ALL) ALL/%wheel ALL=(ALL:ALL) ALL/' /etc/sudoers",
        ));
        cmds.push(chroot_cmd(
            r"sed -i 's/^#\s*%sudo ALL=(ALL:ALL) ALL/%sudo ALL=(ALL:ALL) ALL/' /etc/sudoers",
        ));

        // Enable selected login manager if set
        if let Some(lm) = state.selected_login_manager.clone()
            && !lm.is_empty()
            && lm != "none"
        {
            cmds.push(format!("systemctl --root=/mnt enable {}", lm));
        }

        // Hyprland keyboard layout configuration for each created user
        if state.selected_desktop_envs.iter().any(|e| e == "Hyprland") {
            let keymap_src = state
                .keyboard_layout_options
                .get(state.keyboard_layout_index)
                .cloned()
                .unwrap_or_else(|| "us".to_string());
            let hypr_kb = if keymap_src.starts_with("de_CH") {
                "ch".to_string()
            } else {
                keymap_src
                    .chars()
                    .filter(|c| c.is_ascii_alphabetic())
                    .take(4)
                    .collect()
            };
            for user in state.users.iter() {
                if user.username.trim().is_empty() {
                    continue;
                }
                cmds.push(chroot_cmd(&format!(
                    "install -d -m 0755 -o {0} -g {0} /home/{0}/.config/hypr",
                    user.username
                )));
                // If file exists, replace kb_layout; else create with kb_layout
                cmds.push(chroot_cmd(&format!(
                    "if [ -f /home/{0}/.config/hypr/hyprland.conf ]; then \
                       sed -i 's/^\\s*kb_layout\\s*=.*/kb_layout = {1}/' /home/{0}/.config/hypr/hyprland.conf; \
                     else \
                       printf 'kb_layout = {1}\\n' > /home/{0}/.config/hypr/hyprland.conf; \
                     fi",
                    user.username, hypr_kb
                )));
                cmds.push(chroot_cmd(&format!(
                    "chown -R {0}:{0} /home/{0}/.config",
                    user.username
                )));
            }
        }

        UserSetupPlan::new(cmds)
    }
}
