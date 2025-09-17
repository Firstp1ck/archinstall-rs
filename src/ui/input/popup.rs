use crate::ui::app::{AppState, PopupKind};
use crossterm::event::KeyCode;

pub(crate) fn handle_popup_keys(app: &mut AppState, code: KeyCode) -> bool {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.close_popup();
            return false;
        }
        KeyCode::Enter => {
            match app.popup_kind {
                Some(PopupKind::Info) => {
                    app.close_popup();
                    return false;
                }
                Some(PopupKind::KeyboardLayout)
                | Some(PopupKind::LocaleLanguage)
                | Some(PopupKind::LocaleEncoding) => {
                    app.apply_popup_selection();
                }
                Some(PopupKind::HostnameInput) => {
                    app.hostname_value = app.custom_input_buffer.trim().to_string();
                    app.custom_input_buffer.clear();
                    app.close_popup();
                }
                Some(PopupKind::MirrorsRegions) => {
                    app.close_popup();
                }
                Some(PopupKind::RootPassword) => {
                    app.root_password = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    app.close_popup();
                }
                Some(PopupKind::RootPasswordConfirm) => {
                    app.root_password_confirm = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    if app.root_password != app.root_password_confirm {
                        app.info_message = "Root passwords do not match".into();
                    } else {
                        app.info_message.clear();
                    }
                    app.close_popup();
                }
                Some(PopupKind::UserAddUsername) => {
                    app.draft_user_username = app.custom_input_buffer.trim().to_string();
                    app.custom_input_buffer.clear();
                    app.close_popup();
                    if !app.draft_user_username.is_empty() {
                        app.open_user_password_input();
                    }
                }
                Some(PopupKind::UserAddPassword) => {
                    app.draft_user_password = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    app.close_popup();
                    app.open_user_password_confirm_input();
                }
                Some(PopupKind::UserAddPasswordConfirm) => {
                    app.draft_user_password_confirm = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    if app.draft_user_password != app.draft_user_password_confirm {
                        app.info_message = "User passwords do not match".into();
                        app.close_popup();
                    } else {
                        app.info_message.clear();
                        app.close_popup();
                        app.open_user_sudo_select();
                    }
                }
                Some(PopupKind::UserAddSudo) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        app.draft_user_is_sudo = global_idx == 0; // Yes
                        app.users.push(crate::ui::app::UserAccount {
                            username: app.draft_user_username.clone(),
                            password: app.draft_user_password.clone(),
                            is_sudo: app.draft_user_is_sudo,
                        });
                    }
                    app.close_popup();
                }
                Some(PopupKind::OptionalRepos) => {
                    app.close_popup();
                }
                Some(PopupKind::MirrorsCustomServerInput) => {
                    if !app.custom_input_buffer.trim().is_empty() {
                        app.mirrors_custom_servers
                            .push(app.custom_input_buffer.trim().to_string());
                    }
                    app.custom_input_buffer.clear();
                }
                Some(PopupKind::MirrorsCustomRepoName) => {
                    if !app.custom_input_buffer.trim().is_empty() {
                        app.draft_repo_name = app.custom_input_buffer.trim().to_string();
                        app.open_mirrors_custom_repo_url();
                    }
                }
                Some(PopupKind::MirrorsCustomRepoUrl) => {
                    if !app.custom_input_buffer.trim().is_empty() {
                        app.draft_repo_url = app.custom_input_buffer.trim().to_string();
                        app.open_mirrors_custom_repo_sig();
                    }
                }
                Some(PopupKind::MirrorsCustomRepoSig) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        app.draft_repo_sig_index = global_idx;
                        if app.draft_repo_sig_index == 0 {
                            app.finalize_custom_repo();
                        } else {
                            app.open_mirrors_custom_repo_signopt();
                        }
                    }
                }
                Some(PopupKind::MirrorsCustomRepoSignOpt) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        app.draft_repo_signopt_index = global_idx;
                        app.finalize_custom_repo();
                    }
                }
                Some(PopupKind::DisksDeviceList) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(line) = app.popup_items.get(global_idx)
                    {
                        let parts: Vec<&str> = line.split("|").map(|s| s.trim()).collect();
                        if parts.len() >= 2 {
                            let path_col = parts[1];
                            if !path_col.is_empty() {
                                app.disks_selected_device = Some(path_col.to_string());
                            }
                        }
                    }
                    app.close_popup();
                }
                Some(PopupKind::DiskEncryptionType) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        app.disk_encryption_type_index = if global_idx == 0 { 0 } else { 1 };
                    }
                    app.close_popup();
                }
                Some(PopupKind::DiskEncryptionPassword) => {
                    app.disk_encryption_password = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    app.close_popup();
                }
                Some(PopupKind::DiskEncryptionPasswordConfirm) => {
                    app.disk_encryption_password_confirm = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    if app.disk_encryption_password != app.disk_encryption_password_confirm {
                        app.info_message = "Passwords do not match".into();
                    } else {
                        app.info_message.clear();
                    }
                    app.close_popup();
                }
                Some(PopupKind::DiskEncryptionPartitionList) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(line) = app.popup_items.get(global_idx)
                    {
                        let parts: Vec<&str> = line.split("|").map(|s| s.trim()).collect();
                        if parts.len() >= 2 {
                            let path_col = parts[1];
                            if !path_col.is_empty() {
                                app.disk_encryption_selected_partition = Some(path_col.to_string());
                            }
                        }
                    }
                    app.close_popup();
                }
                Some(PopupKind::AbortConfirm) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        if global_idx == 0 {
                            // Yes: save then exit
                            app.save_config();
                        }
                        app.close_popup();
                        return true;
                    }
                    app.close_popup();
                    return true;
                }
                Some(PopupKind::MinimalClearConfirm) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && global_idx == 0
                    {
                        // Yes: set Minimal and clear selections
                        app.experience_mode_index = 1;
                        app.selected_desktop_envs.clear();
                        app.selected_env_packages.clear();
                        app.selected_login_manager = None;
                        app.login_manager_user_set = false;
                    }
                    app.close_popup();
                }
                Some(PopupKind::DesktopEnvSelect) => {
                    // For DE/WM popup, Enter just closes; selection is toggled with Space
                    app.close_popup();
                }
                Some(PopupKind::ServerTypeSelect) => {
                    // For Server popup, Enter just closes; selection is toggled with Space
                    app.close_popup();
                }
                Some(PopupKind::XorgTypeSelect) => {
                    // For Xorg popup, Enter just closes; selection is toggled with Space
                    app.close_popup();
                }
                None => app.apply_popup_selection(),
            }
            return false;
        }
        // Custom text input editing (handled before search)
        KeyCode::Backspace
            if matches!(
                app.popup_kind,
                Some(PopupKind::MirrorsCustomServerInput)
                    | Some(PopupKind::MirrorsCustomRepoName)
                    | Some(PopupKind::MirrorsCustomRepoUrl)
                    | Some(PopupKind::DiskEncryptionPassword)
                    | Some(PopupKind::DiskEncryptionPasswordConfirm)
                    | Some(PopupKind::HostnameInput)
                    | Some(PopupKind::RootPassword)
                    | Some(PopupKind::RootPasswordConfirm)
                    | Some(PopupKind::UserAddUsername)
                    | Some(PopupKind::UserAddPassword)
                    | Some(PopupKind::UserAddPasswordConfirm)
            ) =>
        {
            app.custom_input_buffer.pop();
            return false;
        }
        KeyCode::Char(c)
            if matches!(
                app.popup_kind,
                Some(PopupKind::MirrorsCustomServerInput)
                    | Some(PopupKind::MirrorsCustomRepoName)
                    | Some(PopupKind::MirrorsCustomRepoUrl)
                    | Some(PopupKind::DiskEncryptionPassword)
                    | Some(PopupKind::DiskEncryptionPasswordConfirm)
                    | Some(PopupKind::HostnameInput)
                    | Some(PopupKind::RootPassword)
                    | Some(PopupKind::RootPasswordConfirm)
                    | Some(PopupKind::UserAddUsername)
                    | Some(PopupKind::UserAddPassword)
                    | Some(PopupKind::UserAddPasswordConfirm)
            ) =>
        {
            app.custom_input_buffer.push(c);
            return false;
        }
        // Search controls
        KeyCode::Char('/') => {
            app.popup_in_search = true;
            return false;
        }
        KeyCode::Backspace => {
            if app.popup_in_search {
                app.popup_search_query.pop();
                app.filter_popup();
            }
            return false;
        }
        KeyCode::Char(' ') => {
            // Toggle selection in multi-select popup
            match app.popup_kind {
                Some(PopupKind::MirrorsRegions) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        if app.mirrors_regions_selected.contains(&global_idx) {
                            app.mirrors_regions_selected.remove(&global_idx);
                        } else {
                            app.mirrors_regions_selected.insert(global_idx);
                        }
                    }
                }
                Some(PopupKind::OptionalRepos) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        if app.optional_repos_selected.contains(&global_idx) {
                            app.optional_repos_selected.remove(&global_idx);
                        } else {
                            app.optional_repos_selected.insert(global_idx);
                        }
                    }
                }
                Some(PopupKind::DesktopEnvSelect) => {
                    // If right-pane (packages) focused: toggle package for current env
                    if app.popup_packages_focus {
                        if let Some(&global_idx) =
                            app.popup_visible_indices.get(app.popup_selected_visible)
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
                                ],
                                "Bspwm" => vec!["bspwm", "dmenu", "rxvt-unicode", "sxhkd", "xdo"],
                                "Budgie" => vec![
                                    "arc-gtk-theme",
                                    "budgie",
                                    "mate-terminal",
                                    "nemo",
                                    "papirus-icon-theme",
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
                                ],
                                "Cutefish" => vec!["cutefish", "noto-fonts"],
                                "Deepin" => vec!["deepin", "deepin-editor", "deepin-terminal"],
                                "Enlightenment" => vec!["enlightenment", "terminology"],
                                "GNOME" => vec!["gnome", "gnome-tweaks"],
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
                                    "wofi",
                                    "xdg-desktop-portal-hyprland",
                                ],
                                "KDE Plasma" => vec![
                                    "ark",
                                    "dolphin",
                                    "kate",
                                    "konsole",
                                    "plasma-meta",
                                    "plasma-workspace",
                                ],
                                "Lxqt" => vec![
                                    "breeze-icons",
                                    "leafpad",
                                    "oxygen-icons",
                                    "slock",
                                    "ttf-freefont",
                                    "xdg-utils",
                                ],
                                "Mate" => vec!["mate", "mate-extra"],
                                "Qtile" => vec!["alacritty", "qtile"],
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
                                "Xfce4" => vec![
                                    "gvfs",
                                    "pavucontrol",
                                    "xarchiver",
                                    "xfce4",
                                    "xfce4-goodies",
                                ],
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
                                    .or_insert_with(|| {
                                        packages.iter().map(|s| s.to_string()).collect()
                                    });
                                if let Some(pkg_name) =
                                    packages.get(app.popup_packages_selected_index)
                                {
                                    if set.contains(*pkg_name) {
                                        set.remove(*pkg_name);
                                    } else {
                                        set.insert((*pkg_name).to_string());
                                    }
                                }
                            }
                        }
                    } else if app.popup_login_focus {
                        if let Some(&global_idx) =
                            app.popup_visible_indices.get(app.popup_selected_visible)
                            && app.popup_items.get(global_idx).is_some()
                        {
                            // Available login managers list
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
                    } else if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(name) = app.popup_items.get(global_idx)
                    {
                        // Left-pane: toggle environment selection
                        if app.selected_desktop_envs.contains(name) {
                            app.selected_desktop_envs.remove(name);
                            // If user hasn't manually set LM, recompute from remaining selection
                            if !app.login_manager_user_set {
                                if app.selected_desktop_envs.is_empty() {
                                    app.selected_login_manager = None;
                                } else if let Some(first_env) =
                                    app.selected_desktop_envs.iter().next().cloned()
                                {
                                    let default_lm: Option<&str> = match first_env.as_str() {
                                        "GNOME" => Some("gdm"),
                                        "KDE Plasma" | "Hyprland" | "Cutefish" | "Lxqt" => {
                                            Some("sddm")
                                        }
                                        "Budgie" => Some("lightdm-slick-greeter"),
                                        "Bspwm" | "Cinnamon" | "Deepin" | "Enlightenment"
                                        | "Mate" | "Qtile" | "Sway" | "Xfce4" | "i3-wm" => {
                                            Some("lightdm-gtk-greeter")
                                        }
                                        _ => None,
                                    };
                                    app.selected_login_manager = default_lm.map(|s| s.to_string());
                                }
                            }
                        } else {
                            app.selected_desktop_envs.insert(name.clone());
                            // On first environment selection, set global login manager if not user-set
                            if !app.login_manager_user_set && app.selected_login_manager.is_none() {
                                // compute default for this env
                                let default_lm: Option<&str> = match name.as_str() {
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
                    }
                }
                Some(PopupKind::ServerTypeSelect) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(name) = app.popup_items.get(global_idx)
                    {
                        // Defaults for this server type
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
                            // Toggle package selection in right pane
                            let set = app
                                .selected_server_packages
                                .entry(name.clone())
                                .or_insert_with(|| {
                                    defaults.iter().map(|s| s.to_string()).collect()
                                });
                            let list: Vec<String> =
                                defaults.iter().map(|s| s.to_string()).collect();
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
                        } else {
                            // Toggle server type in left pane
                            if app.selected_server_types.contains(name) {
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
                    }
                    return false;
                }
                Some(PopupKind::XorgTypeSelect) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(name) = app.popup_items.get(global_idx)
                    {
                        // Defaults for Xorg type
                        let defaults: Vec<&str> = match name.as_str() {
                            "Xorg" => vec!["xorg-server"],
                            _ => Vec::new(),
                        };
                        if app.popup_packages_focus {
                            // Toggle package selection in right pane
                            let set = app
                                .selected_xorg_packages
                                .entry(name.clone())
                                .or_insert_with(|| {
                                    defaults.iter().map(|s| s.to_string()).collect()
                                });
                            let list: Vec<String> =
                                defaults.iter().map(|s| s.to_string()).collect();
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
                        } else {
                            // Toggle Xorg type in left pane
                            if app.selected_xorg_types.contains(name) {
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
                    }
                    return false;
                }
                _ => {}
            }
            return false;
        }
        KeyCode::Char(c) if app.popup_in_search => {
            app.popup_search_query.push(c);
            app.filter_popup();
            return false;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if matches!(app.popup_kind, Some(PopupKind::DesktopEnvSelect)) {
                if app.popup_login_focus {
                    let len = 6; // none, gdm, lightdm-gtk-greeter, lightdm-slick-greeter, ly, sddm
                    app.popup_login_selected_index =
                        (app.popup_login_selected_index + len - 1) % len;
                } else if app.popup_packages_focus {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
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
                            ],
                            "Bspwm" => vec!["bspwm", "dmenu", "rxvt-unicode", "sxhkd", "xdo"],
                            "Budgie" => vec![
                                "arc-gtk-theme",
                                "budgie",
                                "mate-terminal",
                                "nemo",
                                "papirus-icon-theme",
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
                            ],
                            "Cutefish" => vec!["cutefish", "noto-fonts"],
                            "Deepin" => vec!["deepin", "deepin-editor", "deepin-terminal"],
                            "Enlightenment" => vec!["enlightenment", "terminology"],
                            "GNOME" => vec!["gnome", "gnome-tweaks"],
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
                                "wofi",
                                "xdg-desktop-portal-hyprland",
                            ],
                            "KDE Plasma" => vec![
                                "ark",
                                "dolphin",
                                "kate",
                                "konsole",
                                "plasma-meta",
                                "plasma-workspace",
                            ],
                            "Lxqt" => vec![
                                "breeze-icons",
                                "leafpad",
                                "oxygen-icons",
                                "slock",
                                "ttf-freefont",
                                "xdg-utils",
                            ],
                            "Mate" => vec!["mate", "mate-extra"],
                            "Qtile" => vec!["alacritty", "qtile"],
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
                        let len = packages.len();
                        if len > 0 {
                            app.popup_packages_selected_index =
                                (app.popup_packages_selected_index + len - 1) % len;
                        }
                    }
                } else {
                    super::screens::popup_move_up(app);
                }
            } else if matches!(app.popup_kind, Some(PopupKind::ServerTypeSelect)) {
                if app.popup_packages_focus {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(_server) = app.popup_items.get(global_idx)
                    {
                        // Compute current server's package defaults length for wrapping
                        let selected_label = _server.as_str();
                        let pkgs_len = match selected_label {
                            "Cockpit" => 3,
                            "Docker" => 1,
                            "Lighttpd" => 1,
                            "Mariadb" => 1,
                            "Nginx" => 1,
                            "Postgresql" => 1,
                            "Tomcat" => 1,
                            "httpd" => 1,
                            "sshd" => 1,
                            _ => 0,
                        };
                        if pkgs_len > 0 {
                            app.popup_packages_selected_index =
                                (app.popup_packages_selected_index + pkgs_len - 1) % pkgs_len;
                        }
                    }
                } else {
                    super::screens::popup_move_up(app);
                }
            } else if matches!(app.popup_kind, Some(PopupKind::XorgTypeSelect)) {
                if app.popup_packages_focus {
                    // Only one package currently; wrap safely
                    let pkgs_len = 1;
                    if pkgs_len > 0 {
                        app.popup_packages_selected_index =
                            (app.popup_packages_selected_index + pkgs_len - 1) % pkgs_len;
                    }
                } else {
                    super::screens::popup_move_up(app);
                }
            } else {
                super::screens::popup_move_up(app);
            }
            return false;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if matches!(app.popup_kind, Some(PopupKind::DesktopEnvSelect)) {
                if app.popup_login_focus {
                    let len = 6;
                    app.popup_login_selected_index = (app.popup_login_selected_index + 1) % len;
                } else if app.popup_packages_focus {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
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
                            ],
                            "Bspwm" => vec!["bspwm", "dmenu", "rxvt-unicode", "sxhkd", "xdo"],
                            "Budgie" => vec![
                                "arc-gtk-theme",
                                "budgie",
                                "mate-terminal",
                                "nemo",
                                "papirus-icon-theme",
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
                            ],
                            "Cutefish" => vec!["cutefish", "noto-fonts"],
                            "Deepin" => vec!["deepin", "deepin-editor", "deepin-terminal"],
                            "Enlightenment" => vec!["enlightenment", "terminology"],
                            "GNOME" => vec!["gnome", "gnome-tweaks"],
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
                                "wofi",
                                "xdg-desktop-portal-hyprland",
                            ],
                            "KDE Plasma" => vec![
                                "ark",
                                "dolphin",
                                "kate",
                                "konsole",
                                "plasma-meta",
                                "plasma-workspace",
                            ],
                            "Lxqt" => vec![
                                "breeze-icons",
                                "leafpad",
                                "oxygen-icons",
                                "slock",
                                "ttf-freefont",
                                "xdg-utils",
                            ],
                            "Mate" => vec!["mate", "mate-extra"],
                            "Qtile" => vec!["alacritty", "qtile"],
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
                        let len = packages.len();
                        if len > 0 {
                            app.popup_packages_selected_index =
                                (app.popup_packages_selected_index + 1) % len;
                        }
                    }
                } else {
                    super::screens::popup_move_down(app);
                }
            } else if matches!(app.popup_kind, Some(PopupKind::ServerTypeSelect)) {
                if app.popup_packages_focus {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(_server) = app.popup_items.get(global_idx)
                    {
                        let selected_label = _server.as_str();
                        let pkgs_len = match selected_label {
                            "Cockpit" => 3,
                            "Docker" => 1,
                            "Lighttpd" => 1,
                            "Mariadb" => 1,
                            "Nginx" => 1,
                            "Postgresql" => 1,
                            "Tomcat" => 1,
                            "httpd" => 1,
                            "sshd" => 1,
                            _ => 0,
                        };
                        if pkgs_len > 0 {
                            app.popup_packages_selected_index =
                                (app.popup_packages_selected_index + 1) % pkgs_len;
                        }
                    }
                } else {
                    super::screens::popup_move_down(app);
                }
            } else if matches!(app.popup_kind, Some(PopupKind::XorgTypeSelect)) {
                if app.popup_packages_focus {
                    let pkgs_len = 1;
                    if pkgs_len > 0 {
                        app.popup_packages_selected_index =
                            (app.popup_packages_selected_index + 1) % pkgs_len;
                    }
                } else {
                    super::screens::popup_move_down(app);
                }
            } else {
                super::screens::popup_move_down(app);
            }
            return false;
        }
        KeyCode::Left | KeyCode::Char('h')
            if matches!(app.popup_kind, Some(PopupKind::ServerTypeSelect)) =>
        {
            if app.popup_packages_focus {
                app.popup_packages_focus = false;
            }
            return false;
        }
        KeyCode::Right | KeyCode::Char('l')
            if matches!(app.popup_kind, Some(PopupKind::ServerTypeSelect)) =>
        {
            if !app.popup_packages_focus {
                app.popup_packages_focus = true;
                app.popup_packages_selected_index = 0;
            }
            return false;
        }
        // Xorg left/right focus switching mirrors Server behavior
        KeyCode::Left | KeyCode::Char('h')
            if matches!(app.popup_kind, Some(PopupKind::XorgTypeSelect)) =>
        {
            if app.popup_packages_focus {
                app.popup_packages_focus = false;
            }
            return false;
        }
        KeyCode::Right | KeyCode::Char('l')
            if matches!(app.popup_kind, Some(PopupKind::XorgTypeSelect)) =>
        {
            if !app.popup_packages_focus {
                app.popup_packages_focus = true;
                app.popup_packages_selected_index = 0;
            }
            return false;
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if matches!(app.popup_kind, Some(PopupKind::DesktopEnvSelect)) {
                if app.popup_login_focus {
                    app.popup_login_focus = false;
                    app.popup_packages_focus = true;
                } else if app.popup_packages_focus {
                    app.popup_packages_focus = false;
                }
                return false;
            }
            return false;
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if matches!(app.popup_kind, Some(PopupKind::DesktopEnvSelect)) {
                if !app.popup_packages_focus {
                    app.popup_login_focus = false;
                    app.popup_packages_focus = true;
                    app.popup_packages_selected_index = 0;
                } else {
                    app.popup_packages_focus = false;
                    app.popup_login_focus = true;
                    app.popup_login_selected_index = 0;
                }
                return false;
            }
            return false;
        }
        _ => {}
    }
    false
}
