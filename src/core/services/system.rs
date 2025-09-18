use crate::core::state::AppState;

#[derive(Clone, Debug)]
pub struct SystemPlan {
    pub commands: Vec<String>,
}

impl SystemPlan {
    pub fn new(commands: Vec<String>) -> Self {
        Self { commands }
    }
}

pub struct SystemService;

impl SystemService {
    pub fn build_pre_install_plan(state: &AppState) -> SystemPlan {
        let mut cmds: Vec<String> = Vec::new();
        // Ensure pacman.conf exists in target
        cmds.push(
            "test -f /mnt/etc/pacman.conf || install -Dm644 /etc/pacman.conf /mnt/etc/pacman.conf"
                .into(),
        );

        // Optional repos from UI state
        let enable_multilib = state.optional_repos_selected.contains(&0); // index 0 used earlier
        let enable_testing = state.optional_repos_selected.contains(&1); // index 1 used earlier

        if enable_multilib {
            // Uncomment [multilib] block
            cmds.push(
                r"sed -i '/^#\s*\[multilib\]/,/^#\s*Include/s/^#\s*//' /mnt/etc/pacman.conf".into(),
            );
        }
        if enable_testing {
            // Uncomment [testing] and [community-testing] if present
            cmds.push(
                r"sed -i '/^#\s*\[testing\]/,/^#\s*Include/s/^#\s*//' /mnt/etc/pacman.conf".into(),
            );
            cmds.push(r"sed -i '/^#\s*\[community-testing\]/,/^#\s*Include/s/^#\s*//' /mnt/etc/pacman.conf".into());
        }

        // Append any custom repositories specified by the user
        if !state.custom_repos.is_empty() {
            // Ensure final newline
            cmds.push("printf '\n' >> /mnt/etc/pacman.conf".into());
            for repo in state.custom_repos.iter() {
                let name_safe = repo.name.replace('\n', " ").replace('\'', "'\\''");
                let url_safe = repo.url.replace('\n', " ").replace('\'', "'\\''");
                cmds.push(format!(
                    "printf '%s\\n' '[{}]' >> /mnt/etc/pacman.conf",
                    name_safe
                ));
                cmds.push(format!(
                    "printf '%s\\n' 'Server = {}' >> /mnt/etc/pacman.conf",
                    url_safe
                ));
                match repo.signature {
                    crate::app::RepoSignature::Never => {
                        cmds.push(
                            "printf '%s\\n' 'SigLevel = Never' >> /mnt/etc/pacman.conf".into(),
                        );
                    }
                    crate::app::RepoSignature::Optional => {
                        // Default optional; include sign option if provided
                        if let Some(opt) = repo.sign_option {
                            match opt {
                                crate::app::RepoSignOption::TrustedOnly => cmds.push("printf '%s\\n' 'SigLevel = PackageRequired DatabaseOptional TrustedOnly' >> /mnt/etc/pacman.conf".into()),
                                crate::app::RepoSignOption::TrustedAll => cmds.push("printf '%s\\n' 'SigLevel = PackageRequired DatabaseOptional TrustedAll' >> /mnt/etc/pacman.conf".into()),
                            }
                        } else {
                            cmds.push("printf '%s\\n' 'SigLevel = PackageRequired DatabaseOptional' >> /mnt/etc/pacman.conf".into());
                        }
                    }
                    crate::app::RepoSignature::Required => {
                        if let Some(opt) = repo.sign_option {
                            match opt {
                                crate::app::RepoSignOption::TrustedOnly => cmds.push("printf '%s\\n' 'SigLevel = Required TrustedOnly' >> /mnt/etc/pacman.conf".into()),
                                crate::app::RepoSignOption::TrustedAll => cmds.push("printf '%s\\n' 'SigLevel = Required TrustedAll' >> /mnt/etc/pacman.conf".into()),
                            }
                        } else {
                            cmds.push(
                                "printf '%s\\n' 'SigLevel = Required' >> /mnt/etc/pacman.conf"
                                    .into(),
                            );
                        }
                    }
                }
                // Add a blank line after each repo block
                cmds.push("printf '\n' >> /mnt/etc/pacman.conf".into());
            }
        }

        SystemPlan::new(cmds)
    }

    pub fn build_pacstrap_plan(state: &AppState) -> SystemPlan {
        let mut cmds: Vec<String> = Vec::new();
        // TODO(v0.3.0+): Add AUR helper support.
        // Update DB to use new mirrors (force refresh)
        cmds.push("pacman -Syy".into());

        use std::collections::BTreeSet;
        let mut package_set: BTreeSet<String> = BTreeSet::new();
        let mut missing: Vec<String> = Vec::new();

        // Essentials
        for p in [
            "base",
            "linux-firmware",
            "base-devel",
            "sudo",
            "pam",
            "curl",
            "git",
            "man-db",
            "man-pages",
            "xdg-user-dirs",
        ] {
            package_set.insert(p.into());
        }

        // Kernels
        for k in state.selected_kernels.iter() {
            package_set.insert(k.clone());
        }

        // Bootloader and EFI tools
        if state.is_uefi() {
            package_set.insert("efibootmgr".into());
        }
        match state.bootloader_index {
            1 => {
                package_set.insert("grub".into());
            }
            3 => {
                package_set.insert("limine".into());
            }
            _ => {}
        }

        // Network stack
        if state.network_mode_index == 2 {
            package_set.insert("networkmanager".into());
        }

        // Audio
        match state.audio_index {
            1 => {
                package_set.insert("pipewire".into());
                package_set.insert("pipewire-pulse".into());
                package_set.insert("wireplumber".into());
            }
            2 => {
                package_set.insert("pulseaudio".into());
                package_set.insert("pulseaudio-alsa".into());
            }
            _ => {}
        }

        // Polkit for Hyprland or Sway
        if state
            .selected_desktop_envs
            .iter()
            .any(|e| e == "Hyprland" || e == "Sway")
        {
            package_set.insert("polkit".into());
        }

        // Login manager (if set and not none)
        if let Some(lm) = state.selected_login_manager.clone()
            && !lm.is_empty()
            && lm != "none"
        {
            package_set.insert(lm);
        }

        // Desktop environment packages (only for selected environments)
        for env in state.selected_desktop_envs.iter() {
            if let Some(set) = state.selected_env_packages.get(env) {
                for p in set.iter() {
                    package_set.insert(p.clone());
                }
            }
        }

        // Server packages (only for selected server types)
        for srv in state.selected_server_types.iter() {
            if let Some(set) = state.selected_server_packages.get(srv) {
                for p in set.iter() {
                    package_set.insert(p.clone());
                }
            }
        }

        // Xorg packages (only for selected xorg types)
        for xorg in state.selected_xorg_types.iter() {
            if let Some(set) = state.selected_xorg_packages.get(xorg) {
                for p in set.iter() {
                    package_set.insert(p.clone());
                }
            }
        }

        // Graphics drivers
        for d in state.selected_graphic_drivers.iter() {
            package_set.insert(d.clone());
        }

        // CPU microcode (detect via /proc/cpuinfo)
        // Prefer exact vendor markers when available; fall back to substring match
        if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
            let lower = cpuinfo.to_lowercase();
            let is_intel = cpuinfo.contains("GenuineIntel") || lower.contains("intel");
            let is_amd = cpuinfo.contains("AuthenticAMD") || lower.contains("amd");
            if is_intel {
                package_set.insert("intel-ucode".into());
            } else if is_amd {
                package_set.insert("amd-ucode".into());
            }
        }

        // User Additional Packages
        for ap in state.additional_packages.iter() {
            package_set.insert(ap.name.clone());
        }

        // Convert to vec for validation
        let packages: Vec<String> = package_set.into_iter().collect();

        if state.dry_run {
            // Validate packages in dry-run to show accurate preview and missing ones
            let mut final_pkgs: Vec<String> = Vec::new();
            for p in packages {
                if let Some((_repo, name, _ver, _desc)) = state.validate_package(&p) {
                    final_pkgs.push(name);
                } else {
                    missing.push(p);
                }
            }

            if !missing.is_empty() {
                let mut echo = String::from("echo 'Missing packages (skipped):");
                for m in &missing {
                    echo.push(' ');
                    echo.push('"');
                    echo.push_str(m);
                    echo.push('"');
                }
                echo.push('\'');
                cmds.push(echo);
            }

            if !final_pkgs.is_empty() {
                let joined = final_pkgs.join(" ");
                // Retry pacstrap up to 2 times on transient fetch errors using different mirrors
                // Reduce noisy output/progress bars by injecting pacman opts via PACMAN env
                cmds.push(format!(
                    "PACMAN=\"pacman --noconfirm --noprogressbar\" pacstrap -K /mnt {} || (pacman -Syy --noconfirm --noprogressbar && PACMAN=\"pacman --noconfirm --noprogressbar\" pacstrap -K /mnt {} )",
                    joined, joined
                ));
            }
        } else {
            // In non-dry-run mode, skip pre-validation to reduce TUI lag; let pacstrap handle errors
            if !packages.is_empty() {
                let joined = packages.join(" ");
                cmds.push(format!(
                    "PACMAN=\"pacman --noconfirm --noprogressbar\" pacstrap -K /mnt {} || (pacman -Syy --noconfirm --noprogressbar && PACMAN=\"pacman --noconfirm --noprogressbar\" pacstrap -K /mnt {} )",
                    joined, joined
                ));
            }
        }

        SystemPlan::new(cmds)
    }
}
