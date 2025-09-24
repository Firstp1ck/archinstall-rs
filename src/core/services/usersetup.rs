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

        // Debug summary
        let sudo_users: Vec<String> = state
            .users
            .iter()
            .filter(|u| u.is_sudo)
            .map(|u| u.username.clone())
            .collect();
        state.debug_log(&format!(
            "usersetup: users={} sudo=[{}] login_manager={}",
            state.users.len(),
            sudo_users.join(", "),
            state
                .selected_login_manager
                .clone()
                .unwrap_or_else(|| "none".into())
        ));

        UserSetupPlan::new(cmds)
    }
}
