use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::ui::app::{AppState, RepoSignOption, RepoSignature, Screen};

pub fn draw_info(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let mut info_lines = vec![Line::from(Span::styled(
        "Info",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];

    if app.current_screen() == Screen::Locales {
        info_lines.push(Line::from(format!(
            "Keyboard: {}",
            app.current_keyboard_layout()
        )));
        info_lines.push(Line::from(format!(
            "Language: {}",
            app.current_locale_language()
        )));
        info_lines.push(Line::from(format!(
            "Encoding: {}",
            app.current_locale_encoding()
        )));
    } else if app.current_screen() == Screen::MirrorsRepos {
        let regions_count = app.mirrors_regions_selected.len();
        let opt_selected: Vec<&str> = app
            .optional_repos_selected
            .iter()
            .filter_map(|&i| app.optional_repos_options.get(i).map(|s| s.as_str()))
            .collect();
        let servers_count = app.mirrors_custom_servers.len();
        let repos_count = app.custom_repos.len();
        info_lines.push(Line::from(format!("Regions selected: {}", regions_count)));
        if regions_count > 0 {
            for &idx in app.mirrors_regions_selected.iter().take(5) {
                if let Some(name) = app.mirrors_regions_options.get(idx) {
                    info_lines.push(Line::from(format!("- {}", name)));
                }
            }
            if regions_count > 5 {
                info_lines.push(Line::from("…"));
            }
        }
        info_lines.push(Line::from(format!(
            "Optional repos: {}",
            if opt_selected.is_empty() {
                "none".into()
            } else {
                opt_selected.join(", ")
            }
        )));
        if servers_count > 0 {
            info_lines.push(Line::from(format!("Custom servers ({}):", servers_count)));
            for s in app.mirrors_custom_servers.iter().take(3) {
                info_lines.push(Line::from(format!("- {}", s)));
            }
            if servers_count > 3 {
                info_lines.push(Line::from("…"));
            }
        } else {
            info_lines.push(Line::from("Custom servers: none"));
        }
        if repos_count > 0 {
            info_lines.push(Line::from("Custom repos:"));
            for repo in app.custom_repos.iter().take(3) {
                let sig = match repo.signature {
                    RepoSignature::Never => "Never",
                    RepoSignature::Optional => "Optional",
                    RepoSignature::Required => "Required",
                };
                let signopt = match repo.sign_option {
                    Some(RepoSignOption::TrustedOnly) => "TrustedOnly",
                    Some(RepoSignOption::TrustedAll) => "TrustedAll",
                    None => "-",
                };
                info_lines.push(Line::from(format!(
                    "{} | {} | {} | {}",
                    repo.name, repo.url, sig, signopt
                )));
            }
            if repos_count > 3 {
                info_lines.push(Line::from("…"));
            }
        } else {
            info_lines.push(Line::from("Custom repos: none"));
        }
    } else if app.current_screen() == Screen::Disks {
        let mode = match app.disks_mode_index {
            0 => "Best-effort partition layout",
            1 => "Manual Partitioning",
            _ => "Pre-mounted configuration",
        };
        info_lines.push(Line::from(format!("Disk mode: {}", mode)));
        if let Some(dev) = &app.disks_selected_device {
            info_lines.push(Line::from(format!("Selected drive: {}", dev)));
        }
        if app.disks_mode_index == 1 && !app.disks_partitions.is_empty() {
            if let Some(label) = &app.disks_label {
                info_lines.push(Line::from(format!("Label: {}", label)));
            }
            info_lines.push(Line::from(format!(
                "Wipe: {}",
                if app.disks_wipe { "Yes" } else { "No" }
            )));
            if let Some(align) = &app.disks_align {
                info_lines.push(Line::from(format!("Align: {}", align)));
            }
            info_lines.push(Line::from("Partitions:"));
            for p in &app.disks_partitions {
                let name = p.name.clone().unwrap_or_default();
                let role = p.role.clone().unwrap_or_default();
                let fs = p.fs.clone().unwrap_or_default();
                let start = p.start.clone().unwrap_or_default();
                let size = p.size.clone().unwrap_or_default();
                let flags = if p.flags.is_empty() {
                    String::new()
                } else {
                    p.flags.join(",")
                };
                let mp = p.mountpoint.clone().unwrap_or_default();
                let enc = if p.encrypt.unwrap_or(false) {
                    " enc"
                } else {
                    ""
                };

                let mut line = String::new();
                if !name.is_empty() {
                    line.push_str(&format!("{} ", name));
                }
                if !role.is_empty() {
                    line.push_str(&format!("({}) ", role));
                }
                if !fs.is_empty() {
                    line.push_str(&format!("{} ", fs));
                }
                if !start.is_empty() || !size.is_empty() {
                    line.push_str(&format!("[{}..{}] ", start, size));
                }
                if !flags.is_empty() {
                    line.push_str(&format!("flags:{} ", flags));
                }
                if !mp.is_empty() {
                    line.push_str(&format!("-> {} ", mp));
                }
                line.push_str(enc);
                if line.is_empty() {
                    line = "(empty)".into();
                }
                info_lines.push(Line::from(format!("- {}", line.trim())));
            }
        } else if app.disks_mode_index == 0 {
            let bl = match app.bootloader_index {
                1 => "GRUB",
                0 => "systemd-boot",
                2 => "efistub",
                _ => "other",
            };
            info_lines.push(Line::from(format!("Bootloader: {}", bl)));
            info_lines.push(Line::from("Planned layout:"));
            if bl == "GRUB" {
                info_lines.push(Line::from("- gpt: 1MiB bios_boot [bios_grub]"));
                if app.swap_enabled {
                    info_lines.push(Line::from("- swap: 4GiB"));
                }
                let enc = if app.disk_encryption_type_index == 1 {
                    " (LUKS)"
                } else {
                    ""
                };
                info_lines.push(Line::from(format!("- root: btrfs{} (rest)", enc)));
            } else {
                info_lines.push(Line::from("- gpt: 512MiB EFI (vfat, esp) -> /boot"));
                if app.swap_enabled {
                    info_lines.push(Line::from("- swap: 4GiB"));
                }
                let enc = if app.disk_encryption_type_index == 1 {
                    " (LUKS)"
                } else {
                    ""
                };
                info_lines.push(Line::from(format!("- root: btrfs{} (rest)", enc)));
            }
        }
    } else if app.current_screen() == Screen::SwapPartition {
        let swap = if app.swap_enabled {
            "Enabled"
        } else {
            "Disabled"
        };
        info_lines.push(Line::from(format!("Swap: {}", swap)));
    } else if app.current_screen() == Screen::Bootloader {
        let bl = match app.bootloader_index {
            0 => "Systemd-boot",
            1 => "Grub",
            2 => "Efistub",
            _ => "Limine",
        };
        info_lines.push(Line::from(format!("Bootloader: {}", bl)));
    } else if app.current_screen() == Screen::Hostname {
        if app.hostname_value.is_empty() {
            info_lines.push(Line::from("Hostname: (not set)"));
        } else {
            info_lines.push(Line::from(format!("Hostname: {}", app.hostname_value)));
        }
    } else if app.current_screen() == Screen::Timezone {
        if app.timezone_value.is_empty() {
            info_lines.push(Line::from("Timezone: (not set)"));
        } else {
            info_lines.push(Line::from(format!("Timezone: {}", app.timezone_value)));
        }
    } else if app.current_screen() == Screen::UnifiedKernelImages {
        let uki = if app.uki_enabled {
            "Enabled"
        } else {
            "Disabled"
        };
        info_lines.push(Line::from(format!("UKI: {}", uki)));
    } else if app.current_screen() == Screen::AutomaticTimeSync {
        info_lines.push(Line::from(format!(
            "Automatic Time Sync: {}",
            if app.ats_enabled { "Yes" } else { "No" }
        )));
    } else if app.current_screen() == Screen::Kernels {
        if app.selected_kernels.is_empty() {
            info_lines.push(Line::from("Kernels: none"));
        } else {
            let mut v: Vec<&str> = app.selected_kernels.iter().map(|s| s.as_str()).collect();
            v.sort_unstable();
            info_lines.push(Line::from(format!("Kernels: {}", v.join(", "))));
        }
    } else if app.current_screen() == Screen::AdditionalPackages {
        if app.additional_packages.is_empty() {
            info_lines.push(Line::from("Additional packages: none"));
        } else {
            let mut entries: Vec<(String, String)> = app
                .additional_packages
                .iter()
                .map(|p| (p.name.clone(), p.description.clone()))
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            info_lines.push(Line::from(format!("Packages ({}):", entries.len())));
            for (name, desc) in entries.into_iter().take(8) {
                let max_line = (area.width.saturating_sub(4)) as usize;
                let mut line = format!("{} — {}", name, desc);
                if line.len() > max_line {
                    line.truncate(max_line);
                }
                info_lines.push(Line::from(format!("  {}", line)));
            }
            if app.additional_packages.len() > 8 {
                info_lines.push(Line::from("  …"));
            }
        }
    } else if app.current_screen() == Screen::Audio {
        info_lines.push(Line::from(format!("Audio: {}", app.current_audio_label())));
    } else if app.current_screen() == Screen::NetworkConfiguration {
        info_lines.push(Line::from(format!(
            "Network: {}",
            app.current_network_label()
        )));
        if !app.network_configs.is_empty() {
            info_lines.push(Line::from("Interfaces:"));
            for cfg in &app.network_configs {
                let mode = match cfg.mode {
                    crate::ui::app::NetworkConfigMode::Dhcp => "DHCP",
                    crate::ui::app::NetworkConfigMode::Static => "Static",
                };
                let mut line = format!("- {} ({})", cfg.interface, mode);
                if let Some(ip) = &cfg.ip_cidr {
                    line.push_str(&format!(" IP={} ", ip));
                }
                if let Some(gw) = &cfg.gateway {
                    line.push_str(&format!(" GW={} ", gw));
                }
                if let Some(dns) = &cfg.dns {
                    line.push_str(&format!(" DNS={} ", dns));
                }
                info_lines.push(Line::from(line));
            }
        }
    } else if app.current_screen() == Screen::UserAccount {
        if app.users.is_empty() {
            info_lines.push(Line::from("No users added yet."));
        } else {
            info_lines.push(Line::from("Users:"));
            for u in app.users.iter().take(5) {
                let sudo = if u.is_sudo { "sudo" } else { "user" };
                info_lines.push(Line::from(format!("- {} ({})", u.username, sudo)));
            }
            if app.users.len() > 5 {
                info_lines.push(Line::from("…"));
            }
        }
    } else if app.current_screen() == Screen::ExperienceMode {
        info_lines.push(Line::from(format!(
            "Experience: {}",
            app.current_experience_mode_label()
        )));
        if app.experience_mode_index == 2 {
            if !app.selected_server_types.is_empty() {
                let mut names: Vec<&str> = app
                    .selected_server_types
                    .iter()
                    .map(|s| s.as_str())
                    .collect();
                names.sort_unstable();
                info_lines.push(Line::from(format!("Selected: {}", names.join(", "))));
                info_lines.push(Line::from("Installed packages:"));
                for &server in names.iter() {
                    if let Some(set) = app.selected_server_packages.get(server) {
                        let mut pkgs: Vec<&str> = set.iter().map(|s| s.as_str()).collect();
                        pkgs.sort_unstable();
                        let available_width = area.width.saturating_sub(4) as usize;
                        let max_name_len = pkgs.iter().map(|s| s.len()).max().unwrap_or(0).min(32);
                        let col_width = (max_name_len + 2).max(6);
                        let mut num_cols = if col_width == 0 {
                            1
                        } else {
                            available_width / col_width
                        };
                        if num_cols == 0 {
                            num_cols = 1;
                        }
                        num_cols = num_cols.min(4);
                        let count = pkgs.len();
                        info_lines.push(Line::from(format!("- {} ({}):", server, count)));
                        let rows = count.div_ceil(num_cols);
                        for r in 0..rows {
                            let mut line = String::new();
                            for c in 0..num_cols {
                                let idx = r + c * rows;
                                if idx < count {
                                    let name = pkgs[idx];
                                    let shown = if name.len() > max_name_len {
                                        &name[..max_name_len]
                                    } else {
                                        name
                                    };
                                    line.push_str(shown);
                                    if c + 1 < num_cols {
                                        let pad = col_width.saturating_sub(shown.len());
                                        line.push_str(&" ".repeat(pad));
                                    }
                                }
                            }
                            info_lines.push(Line::from(format!("  {}", line)));
                        }
                    } else {
                        info_lines.push(Line::from(format!("- {}: default packages", server)));
                    }
                }
            } else {
                info_lines.push(Line::from("Selected: none"));
            }
        } else if app.experience_mode_index == 3 {
            if !app.selected_xorg_types.is_empty() {
                let mut names: Vec<&str> =
                    app.selected_xorg_types.iter().map(|s| s.as_str()).collect();
                names.sort_unstable();
                info_lines.push(Line::from(format!("Selected: {}", names.join(", "))));
                info_lines.push(Line::from("Installed packages:"));
                for &xorg in names.iter() {
                    if let Some(set) = app.selected_xorg_packages.get(xorg) {
                        let mut pkgs: Vec<&str> = set.iter().map(|s| s.as_str()).collect();
                        pkgs.sort_unstable();
                        let available_width = area.width.saturating_sub(4) as usize;
                        let max_name_len = pkgs.iter().map(|s| s.len()).max().unwrap_or(0).min(32);
                        let col_width = (max_name_len + 2).max(6);
                        let mut num_cols = if col_width == 0 {
                            1
                        } else {
                            available_width / col_width
                        };
                        if num_cols == 0 {
                            num_cols = 1;
                        }
                        num_cols = num_cols.min(4);
                        let count = pkgs.len();
                        info_lines.push(Line::from(format!("- {} ({}):", xorg, count)));
                        let rows = count.div_ceil(num_cols);
                        for r in 0..rows {
                            let mut line = String::new();
                            for c in 0..num_cols {
                                let idx = r + c * rows;
                                if idx < count {
                                    let name = pkgs[idx];
                                    let shown = if name.len() > max_name_len {
                                        &name[..max_name_len]
                                    } else {
                                        name
                                    };
                                    line.push_str(shown);
                                    if c + 1 < num_cols {
                                        let pad = col_width.saturating_sub(shown.len());
                                        line.push_str(&" ".repeat(pad));
                                    }
                                }
                            }
                            info_lines.push(Line::from(format!("  {}", line)));
                        }
                    } else {
                        info_lines.push(Line::from(format!("- {}: default packages", xorg)));
                    }
                }
            } else {
                info_lines.push(Line::from("Selected: none"));
            }
        } else if !app.selected_desktop_envs.is_empty() {
            let mut names: Vec<&str> = app
                .selected_desktop_envs
                .iter()
                .map(|s| s.as_str())
                .collect();
            names.sort_unstable();
            info_lines.push(Line::from(format!("Selected: {}", names.join(", "))));

            let default_for_env = |env: &str| -> Option<&'static str> {
                match env {
                    "GNOME" => Some("gdm"),
                    "KDE Plasma" | "Hyprland" | "Cutefish" | "Lxqt" => Some("sddm"),
                    "Budgie" => Some("lightdm-slick-greeter"),
                    "Bspwm" | "Cinnamon" | "Deepin" | "Enlightenment" | "Mate" | "Qtile"
                    | "Sway" | "Xfce4" | "i3-wm" => Some("lightdm-gtk-greeter"),
                    _ => None,
                }
            };
            let effective_lm: String = if app.login_manager_user_set {
                app.selected_login_manager
                    .clone()
                    .unwrap_or_else(|| "none".into())
            } else if let Some(man) = app.selected_login_manager.clone() {
                man
            } else if let Some(first_env) = names.first() {
                default_for_env(first_env).unwrap_or("none").into()
            } else {
                "none".into()
            };
            info_lines.push(Line::from(format!("Login Manager: {}", effective_lm)));

            info_lines.push(Line::from("Packages:"));
            for &env in names.iter() {
                let mut pkgs: Vec<String> = if let Some(set) = app.selected_env_packages.get(env) {
                    set.iter().cloned().collect()
                } else {
                    match env {
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
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                        "Bspwm" => vec!["bspwm", "dmenu", "rxvt-unicode", "sxhkd", "xdo"]
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "Budgie" => vec![
                            "arc-gtk-theme",
                            "budgie",
                            "mate-terminal",
                            "nemo",
                            "papirus-icon-theme",
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
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
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                        "Cutefish" => vec!["cutefish", "noto-fonts"]
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "Deepin" => vec!["deepin", "deepin-editor", "deepin-terminal"]
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "Enlightenment" => vec!["enlightenment", "terminology"]
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "GNOME" => vec!["gnome", "gnome-tweaks"]
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
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
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                        "KDE Plasma" => vec![
                            "ark",
                            "dolphin",
                            "kate",
                            "konsole",
                            "plasma-meta",
                            "plasma-workspace",
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                        "Lxqt" => vec![
                            "breeze-icons",
                            "leafpad",
                            "oxygen-icons",
                            "slock",
                            "ttf-freefont",
                            "xdg-utils",
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                        "Mate" => vec!["mate", "mate-extra"]
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "Qtile" => vec!["alacritty", "qtile"]
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
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
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                        "Xfce4" => {
                            vec!["gvfs", "pavucontrol", "xarchiver", "xfce4", "xfce4-goodies"]
                                .into_iter()
                                .map(|s| s.to_string())
                                .collect()
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
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                        _ => Vec::new(),
                    }
                };
                pkgs.sort_unstable();
                let count = pkgs.len();
                info_lines.push(Line::from(format!("- {} ({}):", env, count)));

                let available_width = area.width.saturating_sub(4) as usize;
                let max_name_len = pkgs.iter().map(|s| s.len()).max().unwrap_or(0).min(32);
                let col_width = (max_name_len + 2).max(6);
                let mut num_cols = if col_width == 0 {
                    1
                } else {
                    available_width / col_width
                };
                if num_cols == 0 {
                    num_cols = 1;
                }
                num_cols = num_cols.min(4);
                let rows = count.div_ceil(num_cols);
                for r in 0..rows {
                    let mut line = String::new();
                    for c in 0..num_cols {
                        let idx = r + c * rows;
                        if idx < count {
                            let name = &pkgs[idx];
                            let shown = if name.len() > max_name_len {
                                &name[..max_name_len]
                            } else {
                                name
                            };
                            line.push_str(shown);
                            if c + 1 < num_cols {
                                let pad = col_width.saturating_sub(shown.len());
                                line.push_str(&" ".repeat(pad));
                            }
                        }
                    }
                    info_lines.push(Line::from(format!("  {}", line)));
                }
            }
        } else {
            info_lines.push(Line::from("Selected: none"));
            info_lines.push(Line::from("Login Manager: none"));
        }
    } else {
        info_lines.push(Line::from(format!(
            "Selected: {}",
            app.menu_entries[app.selected_index].label
        )));
    }

    let info = Paragraph::new(info_lines)
        .block(Block::default().borders(Borders::ALL).title(" Info "))
        .wrap(Wrap { trim: true });
    frame.render_widget(info, area);
}
