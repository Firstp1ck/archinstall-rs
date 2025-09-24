use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::AppState;

pub(super) fn render(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let mut info_lines = vec![Line::from(Span::styled(
        "Info",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];

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
            let mut names: Vec<&str> = app.selected_xorg_types.iter().map(|s| s.as_str()).collect();
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
                "Bspwm" | "Cinnamon" | "Deepin" | "Enlightenment" | "Mate" | "Qtile" | "Sway"
                | "Xfce4" | "i3-wm" => Some("lightdm-gtk-greeter"),
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
                    // ...existing code...
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
                .into_iter()
                .map(|s| s.to_string())
                .collect()
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

    let mut desc_lines = vec![Line::from(Span::styled(
        "Description",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];
    desc_lines.push(Line::from("Experience Mode allows selecting a predefined system role during installation, such as Desktop, Minimal, Server, or Xorg. Each mode determines which desktop environments, window managers, drivers, and login managers are installed, tailoring the system for general use, lightweight setups, server purposes, or just graphical infrastructure."));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    let description = Paragraph::new(desc_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Description "),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(description, chunks[0]);

    let info = Paragraph::new(info_lines)
        .block(Block::default().borders(Borders::ALL).title(" Info "))
        .wrap(Wrap { trim: true });
    frame.render_widget(info, chunks[1]);
}
