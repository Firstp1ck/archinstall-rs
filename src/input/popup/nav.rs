use crate::app::{AppState, PopupKind};

// Re-export screen movement helpers from the parent input module
use super::super::screens;

pub(crate) fn handle_nav_up(app: &mut AppState) -> bool {
    if matches!(
        app.popup_kind,
        Some(PopupKind::AdditionalPackageGroupSelect)
    ) && app.popup_packages_focus
    {
        handle_group_packages_up(app);
        return false;
    }
    if matches!(app.popup_kind, Some(PopupKind::DesktopEnvSelect)) {
        if app.popup_login_focus {
            let len = 6;
            app.popup_login_selected_index = (app.popup_login_selected_index + len - 1) % len;
        } else if app.popup_drivers_focus {
            let drv_len = {
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
                    ("xorg-server", true),
                    ("xorg-xinit", true),
                    (" Nvidia Drivers ", false),
                    ("dkms", true),
                    ("libva-nvidia-driver", true),
                    (" Choose one ", false),
                    ("nvidia-open-dkms", true),
                    ("nvidia-dkms", true),
                ];
                drivers_all.len()
            };
            if drv_len > 0 {
                app.popup_drivers_selected_index =
                    (app.popup_drivers_selected_index + drv_len - 1) % drv_len;
            }
        } else if app.popup_packages_focus {
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
                        "polkit",
                        "waybar",
                        "xorg-xwayland",
                    ],
                    "Xfce4" => vec!["gvfs", "pavucontrol", "xarchiver", "xfce4", "xfce4-goodies"],
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
            screens::popup_move_up(app);
        }
    } else if matches!(app.popup_kind, Some(PopupKind::ServerTypeSelect)) {
        if app.popup_packages_focus {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible)
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
                        (app.popup_packages_selected_index + pkgs_len - 1) % pkgs_len;
                }
            }
        } else {
            screens::popup_move_up(app);
        }
    } else if matches!(app.popup_kind, Some(PopupKind::XorgTypeSelect)) {
        if app.popup_drivers_focus {
            let drv_len = {
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
                    ("xorg-server", true),
                    ("xorg-xinit", true),
                    (" Nvidia Drivers ", false),
                    ("dkms", true),
                    ("libva-nvidia-driver", true),
                    (" Choose one ", false),
                    ("nvidia-open-dkms", true),
                    ("nvidia-dkms", true),
                ];
                drivers_all.len()
            };
            if drv_len > 0 {
                app.popup_drivers_selected_index =
                    (app.popup_drivers_selected_index + drv_len - 1) % drv_len;
            }
        } else if app.popup_packages_focus {
            let pkgs_len = 1;
            if pkgs_len > 0 {
                app.popup_packages_selected_index =
                    (app.popup_packages_selected_index + pkgs_len - 1) % pkgs_len;
            }
        } else {
            screens::popup_move_up(app);
        }
    } else {
        screens::popup_move_up(app);
    }
    false
}

pub(crate) fn handle_nav_down(app: &mut AppState) -> bool {
    if matches!(
        app.popup_kind,
        Some(PopupKind::AdditionalPackageGroupSelect)
    ) && app.popup_packages_focus
    {
        handle_group_packages_down(app);
        return false;
    }
    if matches!(app.popup_kind, Some(PopupKind::DesktopEnvSelect)) {
        if app.popup_login_focus {
            let len = 6;
            app.popup_login_selected_index = (app.popup_login_selected_index + 1) % len;
        } else if app.popup_drivers_focus {
            let drv_len = {
                let drivers_all: Vec<(&str, bool)> = vec![
                    (" Open Source Drivers ", false),
                    ("intel-media-driver", true),
                    ("libva-intel-driver", true),
                    ("libva-mesa-driver", true),
                    ("mesa", true),
                    ("vulkan-intel", true),
                    ("vulkan-nouveau", true),
                    ("vulkan-radeon", true),
                    ("xf86-video-amdgpu", true),
                    ("xf86-video-ati", true),
                    ("xf86-video-nouveau", true),
                    ("xorg-server", true),
                    ("xorg-xinit", true),
                    (" Nvidia Drivers ", false),
                    ("dkms", true),
                    ("libva-nvidia-driver", true),
                    (" Choose one ", false),
                    ("nvidia-open-dkms", true),
                    ("nvidia-dkms", true),
                ];
                drivers_all.len()
            };
            if drv_len > 0 {
                app.popup_drivers_selected_index = (app.popup_drivers_selected_index + 1) % drv_len;
            }
        } else if app.popup_packages_focus {
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
                    "Xfce4" => vec!["gvfs", "pavucontrol", "xarchiver", "xfce4", "xfce4-goodies"],
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
            screens::popup_move_down(app);
        }
    } else if matches!(app.popup_kind, Some(PopupKind::ServerTypeSelect)) {
        if app.popup_packages_focus {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible)
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
            screens::popup_move_down(app);
        }
    } else if matches!(app.popup_kind, Some(PopupKind::XorgTypeSelect)) {
        if app.popup_drivers_focus {
            let drv_len = {
                let drivers_all: Vec<(&str, bool)> = vec![
                    ("intel-media-driver", true),
                    ("libva-intel-driver", true),
                    ("libva-mesa-driver", true),
                    ("mesa", true),
                    ("vulkan-intel", true),
                    ("vulkan-nouveau", true),
                    ("vulkan-radeon", true),
                    ("xf86-video-amdgpu", true),
                    ("xf86-video-ati", true),
                    ("xf86-video-nouveau", true),
                    ("xorg-server", true),
                    ("xorg-xinit", true),
                    (" Nvidia Drivers ", false),
                    ("dkms", true),
                    ("libva-nvidia-driver", true),
                    (" Choose one ", false),
                    ("nvidia-open-dkms", true),
                    ("nvidia-dkms", true),
                ];
                drivers_all.len()
            };
            if drv_len > 0 {
                app.popup_drivers_selected_index = (app.popup_drivers_selected_index + 1) % drv_len;
            }
        } else if app.popup_packages_focus {
            let pkgs_len = 1;
            if pkgs_len > 0 {
                app.popup_packages_selected_index =
                    (app.popup_packages_selected_index + 1) % pkgs_len;
            }
        } else {
            screens::popup_move_down(app);
        }
    } else {
        screens::popup_move_down(app);
    }
    false
}

// Extend navigation for AdditionalPackageGroupSelect packages list
fn handle_group_packages_up(app: &mut AppState) {
    if app.popup_packages_focus
        && let Some(&gi) = app.popup_visible_indices.get(app.popup_selected_visible)
        && let Some(group) = app.popup_items.get(gi)
    {
        let pkgs = crate::app::AppState::group_packages_for(group);
        let len = pkgs.len();
        if len > 0 {
            app.addpkgs_group_pkg_index = (app.addpkgs_group_pkg_index + len - 1) % len;
        }
    }
}

fn handle_group_packages_down(app: &mut AppState) {
    if app.popup_packages_focus
        && let Some(&gi) = app.popup_visible_indices.get(app.popup_selected_visible)
        && let Some(group) = app.popup_items.get(gi)
    {
        let pkgs = crate::app::AppState::group_packages_for(group);
        let len = pkgs.len();
        if len > 0 {
            app.addpkgs_group_pkg_index = (app.addpkgs_group_pkg_index + 1) % len;
        }
    }
}
