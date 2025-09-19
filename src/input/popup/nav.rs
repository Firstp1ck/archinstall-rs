use crate::app::{AppState, PopupKind};

// Re-export screen movement helpers from the parent input module
use super::super::screens;

pub(crate) fn handle_nav_up(app: &mut AppState) -> bool {
    // Manual partition size/units: when units focused, move selection
    if matches!(app.popup_kind, Some(PopupKind::ManualPartitionCreate))
        && app.manual_create_focus_units
    {
        if app.manual_create_units_index == 0 {
            app.manual_create_units_index = 3;
        } else {
            app.manual_create_units_index -= 1;
        }
        return false;
    }
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
                use crate::common::env_packages::get_env_packages;
                let packages = get_env_packages(env_name.as_str());
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
    // Manual partition size/units: when units focused, move selection
    if matches!(app.popup_kind, Some(PopupKind::ManualPartitionCreate))
        && app.manual_create_focus_units
    {
        app.manual_create_units_index = (app.manual_create_units_index + 1) % 4;
        return false;
    }
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
                use crate::common::env_packages::get_env_packages;
                let packages = get_env_packages(env_name.as_str());
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
