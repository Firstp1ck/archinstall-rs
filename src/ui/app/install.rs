use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::{AppState, Focus};

impl AppState {
    #[allow(dead_code)]
    pub fn init_install(&mut self) {
        // placeholder
    }
}

pub fn draw_install(frame: &mut ratatui::Frame, app: &mut AppState, area: Rect) {
    let title_span = Span::styled(
        "Install",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    // Build content as sections so we can split columns only at section boundaries
    let mut sections: Vec<Vec<Line>> = Vec::new();
    sections.push(vec![Line::from(title_span), Line::from("")]);

    // Locales
    let locales_items = vec![
        format!("Keyboard: {}", app.current_keyboard_layout()),
        format!("Language: {}", app.current_locale_language()),
        format!("Encoding: {}", app.current_locale_encoding()),
    ];
    push_section_lines(&mut sections, "Locales", &locales_items);

    // Mirrors & Repositories
    {
        let mut items: Vec<String> = Vec::new();
        if !app.mirrors_regions_selected.is_empty() {
            let mut names: Vec<String> = app
                .mirrors_regions_selected
                .iter()
                .filter_map(|&i| app.mirrors_regions_options.get(i).cloned())
                .collect();
            names.sort();
            items.push(format!("Regions: {}", names.join(", ")));
        }
        if !app.optional_repos_selected.is_empty() {
            let mut names: Vec<String> = app
                .optional_repos_selected
                .iter()
                .filter_map(|&i| app.optional_repos_options.get(i).cloned())
                .collect();
            names.sort();
            items.push(format!("Optional repos: {}", names.join(", ")));
        }
        if !app.mirrors_custom_servers.is_empty() {
            items.push(format!("Custom servers: {}", app.mirrors_custom_servers.len()));
        }
        if !app.custom_repos.is_empty() {
            items.push(format!("Custom repos: {}", app.custom_repos.len()));
        }
        let mirrors_items = items;
        push_section_lines(&mut sections, "Mirrors & Repositories", &mirrors_items);
    }

    // Disk Partitioning
    {
        let mut items: Vec<String> = Vec::new();
        let mode = match app.disks_mode_index {
            0 => "Best-effort partition layout",
            1 => "Manual Partitioning",
            _ => "Pre-mounted configuration",
        };
        items.push(format!("Mode: {}", mode));
        if let Some(dev) = &app.disks_selected_device {
            items.push(format!("Device: {}", dev));
        }
        let disks_items = items;
        push_section_lines(&mut sections, "Disks", &disks_items);
    }

    // Disk Encryption
    {
        let mut items: Vec<String> = Vec::new();
        let enc = if app.disk_encryption_type_index == 1 { "LUKS" } else { "None" };
        items.push(format!("Type: {}", enc));
        if let Some(p) = &app.disk_encryption_selected_partition {
            items.push(format!("Partition: {}", p));
        }
        let diskenc_items = items;
        push_section_lines(&mut sections, "Disk Encryption", &diskenc_items);
    }

    // Swap
    let swap_items = vec![format!(
        "{}",
        if app.swap_enabled { "Enabled" } else { "Disabled" }
    )];
    push_section_lines(&mut sections, "Swap", &swap_items);

    // Bootloader
    let boot_items = vec![format!(
        "{}",
        match app.bootloader_index {
            0 => "systemd-boot",
            1 => "grub",
            2 => "efistub",
            _ => "limine",
        }
    )];
    push_section_lines(&mut sections, "Bootloader", &boot_items);

    // Unified Kernel Images
    if app.bootloader_index != 1 {
        let uki_items = vec![format!(
            "{}",
            if app.uki_enabled { "Enabled" } else { "Disabled" }
        )];
        push_section_lines(&mut sections, "Unified Kernel Images", &uki_items);
    }

    // System
    {
        let mut items: Vec<String> = Vec::new();
        if !app.hostname_value.is_empty() {
            items.push(format!("Hostname: {}", app.hostname_value));
        }
        items.push(format!(
            "Automatic Time Sync: {}",
            if app.ats_enabled { "Yes" } else { "No" }
        ));
        if !app.timezone_value.is_empty() {
            items.push(format!("Timezone: {}", app.timezone_value));
        }
        let system_items = items;
        push_section_lines(&mut sections, "System", &system_items);
    }

    // Users
    if !app.users.is_empty() {
        let ua_items = vec![format!("Users: {}", app.users.len())];
        push_section_lines(&mut sections, "User Accounts", &ua_items);
    }

    // Experience Mode
    {
        let mut items: Vec<String> = Vec::new();
        let mode = app.current_experience_mode_label();
        items.push(format!("Mode: {}", mode));
        match app.experience_mode_index {
            2 => {
                if !app.selected_server_types.is_empty() {
                    let mut names: Vec<&str> =
                        app.selected_server_types.iter().map(|s| s.as_str()).collect();
                    names.sort_unstable();
                    items.push(format!("Servers: {}", names.join(", ")));
                }
            }
            3 => {
                if !app.selected_xorg_types.is_empty() {
                    let mut names: Vec<&str> =
                        app.selected_xorg_types.iter().map(|s| s.as_str()).collect();
                    names.sort_unstable();
                    items.push(format!("Xorg: {}", names.join(", ")));
                }
            }
            _ => {
                if !app.selected_desktop_envs.is_empty() {
                    let mut names: Vec<&str> =
                        app.selected_desktop_envs.iter().map(|s| s.as_str()).collect();
                    names.sort_unstable();
                    items.push(format!("Desktops: {}", names.join(", ")));
                }
            }
        }
        // Login Manager (effective)
        let lm = if app.login_manager_user_set {
            app.selected_login_manager
                .clone()
                .unwrap_or_else(|| "none".into())
        } else {
            app.selected_login_manager.clone().unwrap_or_else(|| "none".into())
        };
        items.push(format!("Login Manager: {}", lm));
        let exp_items = items;
        push_section_lines(&mut sections, "Experience", &exp_items);

        // List all packages contributed by Experience Mode selections
        let mut exp_pkg_sec: Vec<Line> = Vec::new();
        exp_pkg_sec.push(Line::from(Span::styled(
            "Experience Packages",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));

        // Desktop environments
        if !app.selected_desktop_envs.is_empty() {
            let mut envs: Vec<&str> = app
                .selected_desktop_envs
                .iter()
                .map(|s| s.as_str())
                .collect();
            envs.sort_unstable();
            for env in envs {
                let mut pkgs: Vec<String> = if let Some(set) = app.selected_env_packages.get(env) {
                    set.iter().cloned().collect()
                } else {
                    env_default_packages(env)
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect()
                };
                pkgs.sort_unstable();
                let joined = pkgs.join(", ");
                exp_pkg_sec.push(Line::from(format!("- {} ({}): {}", env, pkgs.len(), joined)));
            }
            exp_pkg_sec.push(Line::from(""));
        }

        // Server types
        if !app.selected_server_types.is_empty() {
            let mut servers: Vec<&str> = app
                .selected_server_types
                .iter()
                .map(|s| s.as_str())
                .collect();
            servers.sort_unstable();
            for server in servers {
                let mut pkgs: Vec<String> = if let Some(set) = app.selected_server_packages.get(server) {
                    set.iter().cloned().collect()
                } else {
                    server_default_packages(server)
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect()
                };
                pkgs.sort_unstable();
                let joined = pkgs.join(", ");
                exp_pkg_sec.push(Line::from(format!("- {} ({}): {}", server, pkgs.len(), joined)));
            }
            exp_pkg_sec.push(Line::from(""));
        }

        // Xorg types
        if !app.selected_xorg_types.is_empty() {
            let mut xorgs: Vec<&str> = app
                .selected_xorg_types
                .iter()
                .map(|s| s.as_str())
                .collect();
            xorgs.sort_unstable();
            for xorg in xorgs {
                let mut pkgs: Vec<String> = if let Some(set) = app.selected_xorg_packages.get(xorg) {
                    set.iter().cloned().collect()
                } else {
                    xorg_default_packages(xorg)
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect()
                };
                pkgs.sort_unstable();
                let joined = pkgs.join(", ");
                exp_pkg_sec.push(Line::from(format!("- {} ({}): {}", xorg, pkgs.len(), joined)));
            }
            exp_pkg_sec.push(Line::from(""));
        }
        if !exp_pkg_sec.is_empty() { sections.push(exp_pkg_sec); }
    }

    // Graphic Drivers
    if !app.selected_graphic_drivers.is_empty() {
        let mut names: Vec<&str> = app
            .selected_graphic_drivers
            .iter()
            .map(|s| s.as_str())
            .collect();
        names.sort_unstable();
        let gdr_items = vec![names.join(", ")];
        push_section_lines(&mut sections, "Graphic Drivers", &gdr_items);
    }

    // Kernels
    if !app.selected_kernels.is_empty() {
        let mut names: Vec<&str> = app.selected_kernels.iter().map(|s| s.as_str()).collect();
        names.sort_unstable();
        let kern_items = vec![names.join(", ")];
        push_section_lines(&mut sections, "Kernels", &kern_items);
    }

    // Network Configuration
    {
        let mut items: Vec<String> = Vec::new();
        items.push(app.current_network_label().to_string());
        if !app.network_configs.is_empty() {
            items.push(format!("Interfaces: {}", app.network_configs.len()));
        }
        let net_items = items;
        push_section_lines(&mut sections, "Network", &net_items);
    }

    // Additional Packages: show name and description
    if !app.additional_packages.is_empty() {
        let mut apkg_sec: Vec<Line> = Vec::new();
        apkg_sec.push(Line::from(Span::styled(
            "Additional Packages",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        let count = app.additional_packages.len();
        apkg_sec.push(Line::from(format!("Packages ({}):", count)));
        // Sort by name for stable display
        let mut entries: Vec<(String, String)> = app
            .additional_packages
            .iter()
            .map(|p| (p.name.clone(), p.description.clone()))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let max_line = area.width.saturating_sub(4) as usize; // account for indent
        let show_limit = 12usize;
        for (name, desc) in entries.into_iter().take(show_limit) {
            let mut line = format!("{} — {}", name, desc);
            if line.len() > max_line { line.truncate(max_line); }
            apkg_sec.push(Line::from(format!("  - {}", line)));
        }
        if count > show_limit {
            apkg_sec.push(Line::from("  …"));
        }
        apkg_sec.push(Line::from(""));
        sections.push(apkg_sec);
    }

    // Install button
    let button_style = match app.focus {
        Focus::Content => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        _ => Style::default(),
    };
    sections.push(vec![Line::from(Span::styled("[ Install ]", button_style))]);

    // Render outer block and split into two columns inside
    let title = match app.focus {
        Focus::Content => " Desicion Menu (focused) ",
        _ => " Desicion Menu ",
    };
    let outer = Block::default().borders(Borders::ALL).title(title);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // Compute section-aware split close to half of total lines
    let section_heights: Vec<usize> = sections.iter().map(|s| s.len()).collect();
    let total_lines: usize = section_heights.iter().sum();
    let target: usize = total_lines / 2;
    let mut acc: usize = 0;
    let mut split_index: usize = sections.len();
    for (i, h) in section_heights.iter().enumerate() {
        if acc + h > target {
            split_index = i;
            break;
        }
        acc += h;
    }
    // Flatten sections into columns
    let mut left_lines: Vec<Line> = Vec::new();
    let mut right_lines: Vec<Line> = Vec::new();
    for (i, sec) in sections.into_iter().enumerate() {
        if i < split_index {
            left_lines.extend(sec);
        } else {
            right_lines.extend(sec);
        }
    }

    let left_par = Paragraph::new(left_lines).wrap(Wrap { trim: false });
    let right_par = Paragraph::new(right_lines).wrap(Wrap { trim: false });
    frame.render_widget(left_par, cols[0]);
    frame.render_widget(right_par, cols[1]);
}

fn push_section_lines(sections: &mut Vec<Vec<Line>>, name: &str, items: &Vec<String>) {
    if items.is_empty() {
        return;
    }
    let mut sec: Vec<Line> = Vec::new();
    sec.push(Line::from(Span::styled(
        name.to_string(),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    for item in items {
        sec.push(Line::from(format!("- {}", item)));
    }
    sec.push(Line::from(""));
    sections.push(sec);
}

// Default packages for Experience Mode when selection sets are not present
fn env_default_packages(env: &str) -> Vec<&'static str> {
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
        _ => Vec::new(),
    }
}

fn server_default_packages(server: &str) -> Vec<&'static str> {
    match server {
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
    }
}

fn xorg_default_packages(xorg: &str) -> Vec<&'static str> {
    match xorg {
        "Xorg" => vec!["xorg-server"],
        _ => Vec::new(),
    }
}
