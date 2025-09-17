use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::ui::app::{
    self, AppState, Focus, INFOBOX_HEIGHT, KEYBINDS_WIDTH, LEFT_MENU_WIDTH, RepoSignOption,
    RepoSignature, Screen,
};

pub fn draw_sections(frame: &mut Frame, app: &mut AppState) {
    let size = frame.area();

    // left (menu) | right (info + content) | keybinds (rightmost)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(LEFT_MENU_WIDTH),
            Constraint::Min(10),
            Constraint::Length(KEYBINDS_WIDTH),
        ])
        .split(size);

    let left_menu_rect = chunks[0];
    let right_rect = chunks[1];
    let keybinds_rect = chunks[2];

    // On ExperienceMode, split Info and Decision content evenly
    let right_constraints = if app.current_screen() == Screen::ExperienceMode {
        [Constraint::Percentage(50), Constraint::Percentage(50)]
    } else {
        [Constraint::Length(INFOBOX_HEIGHT), Constraint::Min(5)]
    };
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(right_constraints)
        .split(right_rect);

    let infobox_rect = right_chunks[0];
    let content_rect = right_chunks[1];

    app.last_menu_rect = left_menu_rect;
    app.last_infobox_rect = infobox_rect;
    app.last_content_rect = content_rect;

    // Menu
    let items: Vec<ListItem> = app
        .menu_entries
        .iter()
        .map(|entry| ListItem::new(Line::from(entry.label.clone())))
        .collect();
    let menu_title = match app.focus {
        Focus::Menu => " Main Menu (focused) ",
        _ => " Main Menu ",
    };
    let highlight_color = match app.focus {
        Focus::Menu => Color::Yellow,
        _ => Color::White,
    };
    let menu = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(menu_title))
        .highlight_style(
            Style::default()
                .fg(highlight_color)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(menu, left_menu_rect, &mut app.list_state);

    // Info
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
        // Summaries for mirrors
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
            0 => "Best-effort default partition layout",
            1 => "Manual Partitioning",
            _ => "Pre-mounted configuration",
        };
        info_lines.push(Line::from(format!("Disk mode: {}", mode)));
        if let Some(dev) = &app.disks_selected_device {
            info_lines.push(Line::from(format!("Selected drive: {}", dev)));
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
            0 => "Systemd-boot (Default)",
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
    } else if app.current_screen() == Screen::UnifiedKernelImages {
        let uki = if app.uki_enabled {
            "Enabled"
        } else {
            "Disabled"
        };
        info_lines.push(Line::from(format!("UKI: {}", uki)));
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
            // Server mode: show selected server types
            if !app.selected_server_types.is_empty() {
                let mut names: Vec<&str> = app
                    .selected_server_types
                    .iter()
                    .map(|s| s.as_str())
                    .collect();
                names.sort_unstable();
                info_lines.push(Line::from(format!("Selected: {}", names.join(", "))));

                // Show server packages for each selected server
                info_lines.push(Line::from("Installed packages:"));
                for &server in names.iter() {
                    if let Some(set) = app.selected_server_packages.get(server) {
                        let mut pkgs: Vec<&str> = set.iter().map(|s| s.as_str()).collect();
                        pkgs.sort_unstable();
                        // Multi-column compact layout
                        let available_width = infobox_rect.width.saturating_sub(4) as usize;
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
            // Xorg mode: show selected Xorg type and packages
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
                        let available_width = infobox_rect.width.saturating_sub(4) as usize;
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

            // Effective Login Manager (global): user choice if set, else default from first selected env
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

            // Packages summary per selected environment (all packages, multi-column, compact)
            info_lines.push(Line::from("Packages:"));
            for &env in names.iter() {
                // Collect packages: either user-selected set or defaults for env
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

                // Multi-column compact layout within infobox width
                let available_width = infobox_rect.width.saturating_sub(4) as usize; // indent
                let max_name_len = pkgs.iter().map(|s| s.len()).max().unwrap_or(0).min(32); // clamp
                let col_width = (max_name_len + 2).max(6); // at least 6 chars per column
                let mut num_cols = if col_width == 0 {
                    1
                } else {
                    available_width / col_width
                };
                if num_cols == 0 {
                    num_cols = 1;
                }
                num_cols = num_cols.min(4); // cap columns to keep readability
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
                            // pad except last column
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
    frame.render_widget(info, infobox_rect);

    // Keybindings window (vertical right)
    let key_lines = vec![
        Line::from(Span::styled(
            "Keybindings",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Global",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "q, ESC",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — back / close"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Main Menu",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — select section"),
        ]),
        Line::from(vec![
            Span::styled(
                "j/k, ↑/↓",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — move selection"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Decision Menu",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "Tab / Shift-Tab",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — switch field"),
        ]),
        Line::from(vec![
            Span::styled(
                "h/l, ←/→",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — change value"),
        ]),
        Line::from(vec![
            Span::styled(
                ":",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — open command line"),
        ]),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — activate item / Continue"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Popups",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "j/k, ↑/↓",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — move selection"),
        ]),
        Line::from(vec![
            Span::styled(
                "/",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — start search"),
        ]),
        Line::from(vec![
            Span::styled(
                "Backspace",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — delete character"),
        ]),
        Line::from(vec![
            Span::styled(
                "Space",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — toggle (multi-select)"),
        ]),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — select / close"),
        ]),
    ];
    let keybinds = Paragraph::new(key_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Keybindings "),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(keybinds, keybinds_rect);

    // Content
    match app.current_screen() {
        Screen::Locales => app::locales::draw_locales(frame, app, content_rect),
        Screen::MirrorsRepos => app::mirrors::draw_mirrors_repos(frame, app, content_rect),
        Screen::Disks => app::disks::draw_disks(frame, app, content_rect),
        Screen::DiskEncryption => {
            app::disk_encryption::draw_disk_encryption(frame, app, content_rect)
        }
        Screen::Bootloader => app::bootloader::draw_bootloader(frame, app, content_rect),
        Screen::Hostname => app::hostname::draw_hostname(frame, app, content_rect),
        Screen::RootPassword => app::root_password::draw_root_password(frame, app, content_rect),
        Screen::UserAccount => app::user_account::draw_user_account(frame, app, content_rect),
        Screen::ExperienceMode => {
            app::experience_mode::draw_experience_mode(frame, app, content_rect)
        }
        Screen::SwapPartition => app::swap_partition::draw_swap_partition(frame, app, content_rect),
        Screen::UnifiedKernelImages => {
            app::unified_kernel_images::draw_unified_kernel_images(frame, app, content_rect)
        }
        Screen::SaveConfiguration => app::config::draw_configuration(frame, app, content_rect),
        _ => {
            let content_lines = vec![
                Line::from(Span::styled(
                    app.menu_entries[app.selected_index].label.to_string(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(app.menu_entries[app.selected_index].content.clone()),
                Line::from(""),
                Line::from(Span::styled(
                    "[ Continue ]",
                    match app.focus {
                        Focus::Content => Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                        _ => Style::default(),
                    },
                )),
            ];
            let content = Paragraph::new(content_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Desicion Menu "),
                )
                .wrap(Wrap { trim: false });
            frame.render_widget(content, content_rect);
        }
    }
}
