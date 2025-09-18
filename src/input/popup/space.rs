use crate::app::{AppState, PopupKind};

pub(crate) fn handle_space(app: &mut AppState) -> bool {
    match app.popup_kind {
        Some(PopupKind::MirrorsRegions) => {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                if app.mirrors_regions_selected.contains(&global_idx) {
                    app.mirrors_regions_selected.remove(&global_idx);
                } else {
                    app.mirrors_regions_selected.insert(global_idx);
                }
            }
        }
        Some(PopupKind::OptionalRepos) => {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                if app.optional_repos_selected.contains(&global_idx) {
                    app.optional_repos_selected.remove(&global_idx);
                } else {
                    app.optional_repos_selected.insert(global_idx);
                }
            }
        }
        Some(PopupKind::KernelSelect) => {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible)
                && let Some(name) = app.popup_items.get(global_idx)
            {
                if app.selected_kernels.contains(name) {
                    app.selected_kernels.remove(name);
                } else {
                    app.selected_kernels.insert(name.clone());
                }
            }
            return false;
        }
        Some(PopupKind::DesktopEnvSelect) => {
            if app.popup_packages_focus {
                if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible)
                    && let Some(env_name) = app.popup_items.get(global_idx)
                {
                    let packages: Vec<&str> = match env_name.as_str() {
                        "Awesome" => vec![
                            "alacritty",
                            "awesome",
                            "feh",
                            "gnu-free-fonts",
                            "slock",
                            "terminus-font",
                            "ttf-liberation",
                            "xorg-server",
                            "xorg-xinit",
                            "xorg-xrandr",
                            "xsel",
                            "xterm",
                            "htop",
                            "iwd",
                            "nano",
                            "openssh",
                            "smartmontools",
                            "vim",
                            "wget",
                            "wireless_tools",
                            "wpa_supplicant",
                            "xdg-utils",
                        ],
                        "Bspwm" => vec![
                            "bspwm",
                            "dmenu",
                            "rxvt-unicode",
                            "sxhkd",
                            "xdo",
                            "htop",
                            "iwd",
                            "nano",
                            "openssh",
                            "smartmontools",
                            "vim",
                            "wget",
                            "wireless_tools",
                            "wpa_supplicant",
                            "xdg-utils",
                        ],
                        "Budgie" => vec![
                            "arc-gtk-theme",
                            "budgie",
                            "mate-terminal",
                            "nemo",
                            "papirus-icon-theme",
                            "htop",
                            "iwd",
                            "nano",
                            "openssh",
                            "smartmontools",
                            "vim",
                            "wget",
                            "wireless_tools",
                            "wpa_supplicant",
                            "xdg-utils",
                        ],
                        "Cinnamon" => vec![
                            "blueman",
                            "blue-utils",
                            "cinnamon",
                            "engrampa",
                            "gnome-keyring",
                            "gnome-screenshot",
                            "gnome-terminal",
                            "gvfs-smb",
                            "system-config-printer",
                            "xdg-user-dirs-gtk",
                            "xed",
                            "htop",
                            "iwd",
                            "nano",
                            "openssh",
                            "smartmontools",
                            "vim",
                            "wget",
                            "wireless_tools",
                            "wpa_supplicant",
                            "xdg-utils",
                        ],
                        "Cutefish" => vec![
                            "cutefish",
                            "noto-fonts",
                            "htop",
                            "iwd",
                            "nano",
                            "openssh",
                            "smartmontools",
                            "vim",
                            "wget",
                            "wireless_tools",
                            "wpa_supplicant",
                            "xdg-utils",
                        ],
                        "Deepin" => vec![
                            "deepin",
                            "deepin-editor",
                            "deepin-terminal",
                            "htop",
                            "iwd",
                            "nano",
                            "openssh",
                            "smartmontools",
                            "vim",
                            "wget",
                            "wireless_tools",
                            "wpa_supplicant",
                            "xdg-utils",
                        ],
                        "Enlightenment" => vec![
                            "enlightenment",
                            "terminology",
                            "htop",
                            "iwd",
                            "nano",
                            "openssh",
                            "smartmontools",
                            "vim",
                            "wget",
                            "wireless_tools",
                            "wpa_supplicant",
                            "xdg-utils",
                        ],
                        "GNOME" => vec![
                            "gnome",
                            "gnome-tweaks",
                            "htop",
                            "iwd",
                            "nano",
                            "openssh",
                            "smartmontools",
                            "vim",
                            "wget",
                            "wireless_tools",
                            "wpa_supplicant",
                            "xdg-utils",
                        ],
                        "Hyprland" => vec![
                            "dolphin",
                            "dunst",
                            "grim",
                            "hyprland",
                            "kitty",
                            "polkit-kde-agent",
                            "qt5-wayland",
                            "qt6-wayland",
                            "slurp",
                            "polkit",
                            "wofi",
                            "xdg-desktop-portal-hyprland",
                            "htop",
                            "iwd",
                            "nano",
                            "openssh",
                            "smartmontools",
                            "vim",
                            "wget",
                            "wireless_tools",
                            "wpa_supplicant",
                            "xdg-utils",
                        ],
                        "KDE Plasma" => vec![
                            "ark",
                            "dolphin",
                            "kate",
                            "konsole",
                            "plasma-meta",
                            "plasma-workspace",
                            "htop",
                            "iwd",
                            "nano",
                            "openssh",
                            "smartmontools",
                            "vim",
                            "wget",
                            "wireless_tools",
                            "wpa_supplicant",
                            "xdg-utils",
                        ],
                        "Lxqt" => vec![
                            "breeze-icons",
                            "leafpad",
                            "oxygen-icons",
                            "slock",
                            "ttf-freefont",
                            "xdg-utils",
                            "htop",
                            "iwd",
                            "nano",
                            "openssh",
                            "smartmontools",
                            "vim",
                            "wget",
                            "wireless_tools",
                            "wpa_supplicant",
                        ],
                        "Mate" => vec![
                            "mate",
                            "mate-extra",
                            "htop",
                            "iwd",
                            "nano",
                            "openssh",
                            "smartmontools",
                            "vim",
                            "wget",
                            "wireless_tools",
                            "wpa_supplicant",
                            "xdg-utils",
                        ],
                        "Qtile" => vec![
                            "alacritty",
                            "qtile",
                            "htop",
                            "iwd",
                            "nano",
                            "openssh",
                            "smartmontools",
                            "vim",
                            "wget",
                            "wireless_tools",
                            "wpa_supplicant",
                            "xdg-utils",
                        ],
                        "Sway" => vec![
                            "brightnessctl",
                            "dmenu",
                            "foot",
                            "grim",
                            "pavucontrol",
                            "slurp",
                            "sway",
                            "swaybg",
                            "swayidle",
                            "swaylock",
                            "waybar",
                            "xorg-xwayland",
                        ],
                        "Xfce4" => {
                            vec!["gvfs", "pavucontrol", "xarchiver", "xfce4", "xfce4-goodies"]
                        }
                        "i3-wm" => vec![
                            "dmenu",
                            "i3-wm",
                            "i3blocks",
                            "i3lock",
                            "i3status",
                            "lightdm",
                            "lightdm-gtk-greeter",
                            "xss-lock",
                            "xterm",
                        ],
                        _ => vec![],
                    };
                    if !packages.is_empty() {
                        let set = app
                            .selected_env_packages
                            .entry(env_name.clone())
                            .or_insert_with(|| packages.iter().map(|s| s.to_string()).collect());
                        if let Some(pkg_name) = packages.get(app.popup_packages_selected_index) {
                            if set.contains(*pkg_name) {
                                set.remove(*pkg_name);
                            } else {
                                set.insert((*pkg_name).to_string());
                            }
                        }
                    }
                }
            } else if app.popup_login_focus {
                if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible)
                    && app.popup_items.get(global_idx).is_some()
                {
                    let login_managers = [
                        "none",
                        "gdm",
                        "lightdm-gtk-greeter",
                        "lightdm-slick-greeter",
                        "ly",
                        "sddm",
                    ];
                    let choice = login_managers
                        .get(app.popup_login_selected_index)
                        .unwrap_or(&"none");
                    let value = if *choice == "none" {
                        None
                    } else {
                        Some(choice.to_string())
                    };
                    app.selected_login_manager = value;
                    app.login_manager_user_set = true;
                }
            } else if app.popup_drivers_focus {
                let drivers_all: Vec<(&str, bool)> = vec![
                    (" Open Source Drivers ", false),
                    ("intel-media-driver", true),
                    ("libva-intel-driver", true),
                    ("mesa", true),
                    ("vulkan-intel", true),
                    ("vulkan-nouveau", true),
                    ("vulkan-radeon", true),
                    ("xf86-video-amdgpu", true),
                    ("xf86-video-ati", true),
                    ("xf86-video-nouveau", true),
                    ("xf86-video-vmware", true),
                    ("xorg-server", true),
                    ("xorg-xinit", true),
                    (" Nvidia Drivers ", false),
                    ("dkms", true),
                    ("libva-nvidia-driver", true),
                    (" Choose one ", false),
                    ("nvidia-open-dkms", true),
                    ("nvidia-dkms", true),
                ];
                let idx = app
                    .popup_drivers_selected_index
                    .min(drivers_all.len().saturating_sub(1));
                if let Some((name, selectable)) = drivers_all.get(idx) {
                    if *selectable {
                        let key = name.to_string();
                        if key == "nvidia-open-dkms" {
                            if app.selected_graphic_drivers.contains("nvidia-open-dkms") {
                                app.selected_graphic_drivers.remove("nvidia-open-dkms");
                            } else {
                                app.selected_graphic_drivers.remove("nvidia-dkms");
                                app.selected_graphic_drivers.insert(key);
                                app.selected_graphic_drivers.insert("dkms".into());
                                app.selected_graphic_drivers
                                    .insert("libva-nvidia-driver".into());
                                app.selected_graphic_drivers.remove("xf86-video-nouveau");
                                app.selected_graphic_drivers.remove("vulkan-nouveau");
                            }
                        } else if key == "nvidia-dkms" {
                            if app.selected_graphic_drivers.contains("nvidia-dkms") {
                                app.selected_graphic_drivers.remove("nvidia-dkms");
                            } else {
                                app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                app.selected_graphic_drivers.insert(key);
                                app.selected_graphic_drivers.insert("dkms".into());
                                app.selected_graphic_drivers
                                    .insert("libva-nvidia-driver".into());
                                app.selected_graphic_drivers.remove("xf86-video-nouveau");
                                app.selected_graphic_drivers.remove("vulkan-nouveau");
                            }
                        } else if app.selected_graphic_drivers.contains(&key) {
                            app.selected_graphic_drivers.remove(&key);
                        } else {
                            app.selected_graphic_drivers.insert(key.clone());
                            if key == "xf86-video-nouveau" || key == "vulkan-nouveau" {
                                app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                app.selected_graphic_drivers.remove("nvidia-dkms");
                                app.selected_graphic_drivers.remove("dkms");
                                app.selected_graphic_drivers.remove("libva-nvidia-driver");
                            }
                        }
                    } else {
                        let title = name.trim();
                        if title == "Open Source Drivers" {
                            let oss = vec![
                                "intel-media-driver",
                                "libva-intel-driver",
                                "libva-mesa-driver",
                                "mesa",
                                "vulkan-intel",
                                "vulkan-nouveau",
                                "vulkan-radeon",
                                "xf86-video-amdgpu",
                                "xf86-video-ati",
                                "xf86-video-nouveau",
                                "xf86-video-vmware",
                                "xorg-server",
                                "xorg-xinit",
                            ];
                            let all_selected = oss
                                .iter()
                                .all(|k| app.selected_graphic_drivers.contains(*k));
                            if all_selected {
                                for k in oss {
                                    app.selected_graphic_drivers.remove(k);
                                }
                            } else {
                                for k in oss {
                                    app.selected_graphic_drivers.insert(k.to_string());
                                }
                                app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                app.selected_graphic_drivers.remove("nvidia-dkms");
                                app.selected_graphic_drivers.remove("dkms");
                                app.selected_graphic_drivers.remove("libva-nvidia-driver");
                            }
                        } else if title == "Nvidia Drivers" {
                            let need = vec!["dkms", "libva-nvidia-driver"];
                            let all_selected = need
                                .iter()
                                .all(|k| app.selected_graphic_drivers.contains(*k));
                            if all_selected {
                                for k in need {
                                    app.selected_graphic_drivers.remove(k);
                                }
                                app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                app.selected_graphic_drivers.remove("nvidia-dkms");
                            } else {
                                for k in need {
                                    app.selected_graphic_drivers.insert(k.to_string());
                                }
                            }
                        } else if title == "Choose One" {
                            let any_selected =
                                app.selected_graphic_drivers.contains("nvidia-open-dkms")
                                    || app.selected_graphic_drivers.contains("nvidia-dkms");
                            if any_selected {
                                app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                app.selected_graphic_drivers.remove("nvidia-dkms");
                            } else {
                                app.selected_graphic_drivers
                                    .insert("nvidia-open-dkms".into());
                                app.selected_graphic_drivers.insert("dkms".into());
                                app.selected_graphic_drivers
                                    .insert("libva-nvidia-driver".into());
                            }
                        }
                    }
                }
                return false;
            } else if let Some(&global_idx) =
                app.popup_visible_indices.get(app.popup_selected_visible)
                && let Some(name) = app.popup_items.get(global_idx)
            {
                if app.selected_desktop_envs.contains(name) {
                    app.selected_desktop_envs.remove(name);
                    if !app.login_manager_user_set {
                        if app.selected_desktop_envs.is_empty() {
                            app.selected_login_manager = None;
                        } else if let Some(first_env) =
                            app.selected_desktop_envs.iter().next().cloned()
                        {
                            let default_lm: Option<&str> = match first_env.as_str() {
                                "GNOME" => Some("gdm"),
                                "KDE Plasma" | "Hyprland" | "Cutefish" | "Lxqt" => Some("sddm"),
                                "Budgie" => Some("lightdm-slick-greeter"),
                                "Bspwm" | "Cinnamon" | "Deepin" | "Enlightenment" | "Mate"
                                | "Qtile" | "Sway" | "Xfce4" | "i3-wm" => {
                                    Some("lightdm-gtk-greeter")
                                }
                                _ => None,
                            };
                            app.selected_login_manager = default_lm.map(|s| s.to_string());
                        }
                    }
                } else {
                    app.selected_desktop_envs.insert(name.clone());
                    if !app.login_manager_user_set && app.selected_login_manager.is_none() {
                        let default_lm: Option<&str> = match name.as_str() {
                            "GNOME" => Some("gdm"),
                            "KDE Plasma" | "Hyprland" | "Cutefish" | "Lxqt" => Some("sddm"),
                            "Budgie" => Some("lightdm-slick-greeter"),
                            "Bspwm" | "Cinnamon" | "Deepin" | "Enlightenment" | "Mate"
                            | "Qtile" | "Sway" | "Xfce4" | "i3-wm" => Some("lightdm-gtk-greeter"),
                            _ => None,
                        };
                        app.selected_login_manager = default_lm.map(|s| s.to_string());
                    }
                }
            }
        }
        Some(PopupKind::AdditionalPackageGroupPackages) => {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible)
                && let Some(name) = app.popup_items.get(global_idx)
            {
                if app.addpkgs_group_pkg_selected.contains(name) {
                    app.addpkgs_group_pkg_selected.remove(name);
                } else {
                    app.addpkgs_group_pkg_selected.insert(name.clone());
                }
            }
            return false;
        }
        Some(PopupKind::AdditionalPackageGroupSelect) => {
            if !app.popup_packages_focus {
                return false;
            }
            if let Some(&gi) = app.popup_visible_indices.get(app.popup_selected_visible)
                && let Some(group) = app.popup_items.get(gi)
            {
                let pkgs = crate::app::AppState::group_packages_for(group);
                let idx = app
                    .addpkgs_group_pkg_index
                    .min(pkgs.len().saturating_sub(1));
                if let Some(name) = pkgs.get(idx) {
                    let name = name.to_string();
                    // If package is already in Additional packages list, toggle removal immediately
                    if let Some(pos) = app
                        .additional_packages
                        .iter()
                        .position(|p| p.name.eq_ignore_ascii_case(&name))
                    {
                        app.additional_packages.remove(pos);
                        // Also clear any pending selection flags for this package
                        app.addpkgs_group_pkg_selected.remove(&name);
                        app.addpkgs_group_accum_selected.remove(&name);
                        return false;
                    }
                    // If package conflicts with another section, treat it as pre-selected
                    // and allow deselect to remove from install list AND other sections
                    if app.check_additional_pkg_conflicts(&name).is_some() {
                        // Determine current selected state (either in additional or via sections or in this group state)
                        let mut currently_selected = app
                            .additional_packages
                            .iter()
                            .any(|p| p.name.eq_ignore_ascii_case(&name));
                        if !currently_selected {
                            for env in app.selected_desktop_envs.iter() {
                                if let Some(set) = app.selected_env_packages.get(env)
                                    && set.contains(&name)
                                {
                                    currently_selected = true;
                                    break;
                                }
                            }
                        }
                        if !currently_selected {
                            for srv in app.selected_server_types.iter() {
                                if let Some(set) = app.selected_server_packages.get(srv)
                                    && set.contains(&name)
                                {
                                    currently_selected = true;
                                    break;
                                }
                            }
                        }
                        if !currently_selected {
                            for xorg in app.selected_xorg_types.iter() {
                                if let Some(set) = app.selected_xorg_packages.get(xorg)
                                    && set.contains(&name)
                                {
                                    currently_selected = true;
                                    break;
                                }
                            }
                        }

                        if currently_selected {
                            // Deselect: remove from additional list and from all active section sets
                            if let Some(pos) = app
                                .additional_packages
                                .iter()
                                .position(|p| p.name.eq_ignore_ascii_case(&name))
                            {
                                app.additional_packages.remove(pos);
                            }
                            for env in app.selected_desktop_envs.clone().iter() {
                                if let Some(set) = app.selected_env_packages.get_mut(env) {
                                    set.remove(&name);
                                }
                            }
                            for srv in app.selected_server_types.clone().iter() {
                                if let Some(set) = app.selected_server_packages.get_mut(srv) {
                                    set.remove(&name);
                                }
                            }
                            for xorg in app.selected_xorg_types.clone().iter() {
                                if let Some(set) = app.selected_xorg_packages.get_mut(xorg) {
                                    set.remove(&name);
                                }
                            }
                            app.addpkgs_group_pkg_selected.remove(&name);
                            app.addpkgs_group_accum_selected.remove(&name);
                        } else {
                            // Select: add to additional list minimally and mark selected here
                            app.additional_packages.push(crate::app::AdditionalPackage {
                                name: name.clone(),
                                repo: String::new(),
                                version: String::new(),
                                description: String::from(
                                    "Already selected in another section (group override)",
                                ),
                            });
                            app.addpkgs_group_pkg_selected.insert(name.clone());
                        }
                        return false;
                    }
                    // Otherwise toggle selection for adding
                    if app.addpkgs_group_pkg_selected.contains(&name) {
                        app.addpkgs_group_pkg_selected.remove(&name);
                        app.addpkgs_group_accum_selected.remove(&name);
                    } else {
                        app.addpkgs_group_pkg_selected.insert(name);
                        app.addpkgs_group_accum_selected
                            .extend(app.addpkgs_group_pkg_selected.iter().cloned());
                    }
                    // persist per group live as user toggles
                    if let Some(&gi) = app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(group) = app.popup_items.get(gi)
                    {
                        let entry = app.addpkgs_group_selected.entry(group.clone()).or_default();
                        *entry = app.addpkgs_group_pkg_selected.clone();
                    }
                }
            }
            return false;
        }
        Some(PopupKind::ServerTypeSelect) => {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible)
                && let Some(name) = app.popup_items.get(global_idx)
            {
                let defaults: Vec<&str> = match name.as_str() {
                    "Cockpit" => vec!["cockpit", "packagekit", "udisk2"],
                    "Docker" => vec!["docker"],
                    "Lighttpd" => vec!["lighttpd"],
                    "Mariadb" => vec!["mariadb"],
                    "Nginx" => vec!["nginx"],
                    "Postgresql" => vec!["postgresql"],
                    "Tomcat" => vec!["tomcat10"],
                    "httpd" => vec!["apache"],
                    "sshd" => vec!["openssh"],
                    _ => Vec::new(),
                };
                if app.popup_packages_focus {
                    let set = app
                        .selected_server_packages
                        .entry(name.clone())
                        .or_insert_with(|| defaults.iter().map(|s| s.to_string()).collect());
                    let list: Vec<String> = defaults.iter().map(|s| s.to_string()).collect();
                    if !list.is_empty() {
                        let idx = app
                            .popup_packages_selected_index
                            .min(list.len().saturating_sub(1));
                        if let Some(pkg) = list.get(idx) {
                            if set.contains(pkg) {
                                set.remove(pkg);
                            } else {
                                set.insert(pkg.clone());
                            }
                        }
                    }
                } else if app.selected_server_types.contains(name) {
                    app.selected_server_types.remove(name);
                    app.selected_server_packages.remove(name);
                } else {
                    app.selected_server_types.insert(name.clone());
                    app.selected_server_packages.insert(
                        name.clone(),
                        defaults.iter().map(|s| s.to_string()).collect(),
                    );
                }
            }
            return false;
        }
        Some(PopupKind::XorgTypeSelect) => {
            if app.popup_drivers_focus {
                let drivers_all: Vec<(&str, bool)> = vec![
                    (" Open Source Drivers ", false),
                    ("intel-media-driver", true),
                    ("libva-intel-driver", true),
                    ("mesa", true),
                    ("vulkan-intel", true),
                    ("vulkan-nouveau", true),
                    ("vulkan-radeon", true),
                    ("xf86-video-amdgpu", true),
                    ("xf86-video-ati", true),
                    ("xf86-video-nouveau", true),
                    ("xf86-video-vmware", true),
                    ("xorg-server", true),
                    ("xorg-xinit", true),
                    (" Nvidia Drivers ", false),
                    ("dkms", true),
                    ("libva-nvidia-driver", true),
                    (" Choose one ", false),
                    ("nvidia-open-dkms", true),
                    ("nvidia-dkms", true),
                ];
                let idx = app
                    .popup_drivers_selected_index
                    .min(drivers_all.len().saturating_sub(1));
                if let Some((name, selectable)) = drivers_all.get(idx) {
                    if *selectable {
                        let key = name.to_string();
                        if key == "nvidia-open-dkms" {
                            if app.selected_graphic_drivers.contains("nvidia-open-dkms") {
                                app.selected_graphic_drivers.remove("nvidia-open-dkms");
                            } else {
                                app.selected_graphic_drivers.remove("nvidia-dkms");
                                app.selected_graphic_drivers.insert(key);
                                app.selected_graphic_drivers.insert("dkms".into());
                                app.selected_graphic_drivers
                                    .insert("libva-nvidia-driver".into());
                                app.selected_graphic_drivers.remove("xf86-video-nouveau");
                                app.selected_graphic_drivers.remove("vulkan-nouveau");
                            }
                        } else if key == "nvidia-dkms" {
                            if app.selected_graphic_drivers.contains("nvidia-dkms") {
                                app.selected_graphic_drivers.remove("nvidia-dkms");
                            } else {
                                app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                app.selected_graphic_drivers.insert(key);
                                app.selected_graphic_drivers.insert("dkms".into());
                                app.selected_graphic_drivers
                                    .insert("libva-nvidia-driver".into());
                                app.selected_graphic_drivers.remove("xf86-video-nouveau");
                                app.selected_graphic_drivers.remove("vulkan-nouveau");
                            }
                        } else if app.selected_graphic_drivers.contains(&key) {
                            app.selected_graphic_drivers.remove(&key);
                        } else {
                            app.selected_graphic_drivers.insert(key.clone());
                            if key == "xf86-video-nouveau" || key == "vulkan-nouveau" {
                                app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                app.selected_graphic_drivers.remove("nvidia-dkms");
                                app.selected_graphic_drivers.remove("dkms");
                                app.selected_graphic_drivers.remove("libva-nvidia-driver");
                            }
                        }
                    } else {
                        let title = name.trim();
                        if title == "Open Source Drivers" {
                            let oss = vec![
                                "intel-media-driver",
                                "libva-intel-driver",
                                "libva-mesa-driver",
                                "mesa",
                                "vulkan-intel",
                                "vulkan-nouveau",
                                "vulkan-radeon",
                                "xf86-video-amdgpu",
                                "xf86-video-ati",
                                "xf86-video-nouveau",
                                "xf86-video-vmware",
                                "xorg-server",
                                "xorg-xinit",
                            ];
                            let all_selected = oss
                                .iter()
                                .all(|k| app.selected_graphic_drivers.contains(*k));
                            if all_selected {
                                for k in oss {
                                    app.selected_graphic_drivers.remove(k);
                                }
                            } else {
                                for k in oss {
                                    app.selected_graphic_drivers.insert(k.to_string());
                                }
                                app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                app.selected_graphic_drivers.remove("nvidia-dkms");
                                app.selected_graphic_drivers.remove("dkms");
                                app.selected_graphic_drivers.remove("libva-nvidia-driver");
                            }
                        } else if title == "Nvidia Drivers" {
                            let need = vec!["dkms", "libva-nvidia-driver"];
                            let all_selected = need
                                .iter()
                                .all(|k| app.selected_graphic_drivers.contains(*k));
                            if all_selected {
                                for k in need {
                                    app.selected_graphic_drivers.remove(k);
                                }
                                app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                app.selected_graphic_drivers.remove("nvidia-dkms");
                            } else {
                                for k in need {
                                    app.selected_graphic_drivers.insert(k.to_string());
                                }
                            }
                        } else if title == "Choose One" {
                            let any_selected =
                                app.selected_graphic_drivers.contains("nvidia-open-dkms")
                                    || app.selected_graphic_drivers.contains("nvidia-dkms");
                            if any_selected {
                                app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                app.selected_graphic_drivers.remove("nvidia-dkms");
                            } else {
                                app.selected_graphic_drivers
                                    .insert("nvidia-open-dkms".into());
                                app.selected_graphic_drivers.insert("dkms".into());
                                app.selected_graphic_drivers
                                    .insert("libva-nvidia-driver".into());
                            }
                        }
                    }
                }
                return false;
            } else if let Some(&global_idx) =
                app.popup_visible_indices.get(app.popup_selected_visible)
                && let Some(name) = app.popup_items.get(global_idx)
            {
                let defaults: Vec<&str> = match name.as_str() {
                    "Xorg" => vec!["xorg-server"],
                    _ => Vec::new(),
                };
                if app.popup_packages_focus {
                    let set = app
                        .selected_xorg_packages
                        .entry(name.clone())
                        .or_insert_with(|| defaults.iter().map(|s| s.to_string()).collect());
                    let list: Vec<String> = defaults.iter().map(|s| s.to_string()).collect();
                    if !list.is_empty() {
                        let idx = app
                            .popup_packages_selected_index
                            .min(list.len().saturating_sub(1));
                        if let Some(pkg) = list.get(idx) {
                            if set.contains(pkg) {
                                set.remove(pkg);
                            } else {
                                set.insert(pkg.clone());
                            }
                        }
                    }
                } else if app.selected_xorg_types.contains(name) {
                    app.selected_xorg_types.remove(name);
                    app.selected_xorg_packages.remove(name);
                } else {
                    app.selected_xorg_types.insert(name.clone());
                    app.selected_xorg_packages.insert(
                        name.clone(),
                        defaults.iter().map(|s| s.to_string()).collect(),
                    );
                }
            }
            return false;
        }
        _ => {}
    }
    false
}
