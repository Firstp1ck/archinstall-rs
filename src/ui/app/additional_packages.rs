use super::AppState;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::process::Command;

impl AppState {
    #[allow(dead_code)]
    pub fn init_additional_packages(&mut self) {}

    pub fn open_additional_package_input(&mut self) {
        self.popup_kind = Some(super::PopupKind::AdditionalPackageInput);
        self.custom_input_buffer.clear();
        self.popup_open = true;
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }
    /// Validate a package by querying pacman. On success, returns parsed (repo, name, version, description)
    pub fn validate_package(&self, name: &str) -> Option<(String, String, String, String)> {
        if name.trim().is_empty() {
            return None;
        }
        let pattern = format!("^{}$", name.trim());
        let output = Command::new("pacman")
            .args(["-Ss", &pattern])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Expect first non-empty line like: "extra/stow 2.4.1-1"
        // Next line (indented) is description
        let mut lines = stdout.lines().filter(|l| !l.trim().is_empty());
        let header = lines.next()?;
        let desc_line = lines.next().unwrap_or("").trim();
        // Parse header
        // Split to repo/name and version
        let mut parts = header.split_whitespace();
        let repo_and_name = parts.next()?; // e.g., extra/stow
        let version = parts.next().unwrap_or("").to_string();
        let mut rn_iter = repo_and_name.splitn(2, '/');
        let repo = rn_iter.next()?.to_string();
        let pkg_name = rn_iter.next()?.to_string();
        let description = desc_line.trim().to_string();
        if pkg_name != name.trim() {
            // Ensure exact name match after ^...$
            return None;
        }
        Some((repo, pkg_name, version, description))
    }

    /// Returns Some(reason) if the package is already covered by other selections
    pub fn check_additional_pkg_conflicts(&self, name: &str) -> Option<String> {
        let n = name.trim();
        if n.is_empty() {
            return None;
        }
        // Kernels are managed in the Kernels section
        if matches!(n, "linux" | "linux-hardened" | "linux-lts" | "linux-zen") {
            return Some("already covered by Kernels selection".into());
        }
        // Experience Mode: Desktop, Server, Xorg packages
        for (_env, set) in self.selected_env_packages.iter() {
            if set.contains(n) {
                return Some("already included in Desktop Environment packages".into());
            }
        }
        for (_srv, set) in self.selected_server_packages.iter() {
            if set.contains(n) {
                return Some("already included in Server packages".into());
            }
        }
        for (_xorg, set) in self.selected_xorg_packages.iter() {
            if set.contains(n) {
                return Some("already included in Xorg packages".into());
            }
        }
        // Graphic drivers
        if self.selected_graphic_drivers.contains(n) {
            return Some("already included in Graphic Drivers".into());
        }
        // Bootloader implied packages
        let mut boot_pkgs: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        match self.bootloader_index {
            0 => {
                // systemd-boot
                boot_pkgs.insert("systemd");
            }
            1 => {
                // grub
                boot_pkgs.insert("grub");
            }
            2 => {
                // efistub
                boot_pkgs.insert("efibootmgr");
            }
            3 => {
                // limine
                boot_pkgs.insert("limine");
            }
            _ => {}
        }
        if boot_pkgs.contains(&n) {
            return Some("already covered by Bootloader selection".into());
        }
        // Network configuration implied package
        if self.network_mode_index == 2 && n.eq_ignore_ascii_case("networkmanager") {
            return Some("already covered by Network configuration".into());
        }
        None
    }
}

pub fn draw_additional_packages(frame: &mut ratatui::Frame, app: &mut AppState, area: Rect) {
    let title = Span::styled(
        "Additional packages",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let mut lines: Vec<Line> = vec![Line::from(title), Line::from("")];

    // Field 0: Add package (opens input popup)
    let is_focus_0 = app.addpkgs_focus_index == 0 && matches!(app.focus, super::Focus::Content);
    let bullet_style_0 = if is_focus_0 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let label_style_0 = if is_focus_0 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    lines.push(Line::from(vec![
        Span::styled(
            format!(
                "{} ",
                if app.addpkgs_focus_index == 0 {
                    "▶"
                } else {
                    " "
                }
            ),
            bullet_style_0,
        ),
        Span::styled("Add package".to_string(), label_style_0),
        Span::raw("  (press Enter)"),
    ]));

    // Show current list (selectable)
    if app.additional_packages.is_empty() {
        lines.push(Line::from("  Current: none"));
    } else {
        lines.push(Line::from(
            "  Current packages (j/k to select, Del/Backspace to delete):",
        ));
        // clamp selected index
        if app.addpkgs_selected_index >= app.additional_packages.len() {
            app.addpkgs_selected_index = app.additional_packages.len().saturating_sub(1);
        }
        for (i, pkg) in app.additional_packages.iter().enumerate() {
            let selected = i == app.addpkgs_selected_index;
            let checked = app.addpkgs_selected.contains(&i);
            let style = if selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let bullet = if selected { "▶" } else { " " };
            let mark = if checked { "[x]" } else { "[ ]" };
            lines.push(Line::from(vec![
                Span::styled(format!("{} ", bullet), style),
                Span::styled(
                    format!(
                        "{} {} {} — {}",
                        mark, pkg.name, pkg.version, pkg.description
                    ),
                    style,
                ),
            ]));
        }
    }

    // Continue
    let is_focus_1 = app.addpkgs_focus_index == 1 && matches!(app.focus, super::Focus::Content);
    let continue_style = if is_focus_1 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("[ Continue ]", continue_style)));

    let content = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(match app.focus {
                    super::Focus::Content => " Desicion Menu (focused) ",
                    _ => " Desicion Menu ",
                }),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(content, area);
}
