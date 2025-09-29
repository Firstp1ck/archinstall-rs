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
    pub fn open_additional_package_group_select(&mut self) {
        self.popup_kind = Some(super::PopupKind::AdditionalPackageGroupSelect);
        self.popup_open = true;
        self.popup_items = self.addpkgs_group_names.clone();
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.addpkgs_group_pkg_selected.clear();
        self.popup_packages_focus = false;
        self.addpkgs_group_pkg_index = 0;
    }
    pub fn open_additional_package_group_packages(&mut self, group_name: &str) {
        self.popup_kind = Some(super::PopupKind::AdditionalPackageGroupPackages);
        self.popup_open = true;
        let group_list = Self::group_packages_for(group_name);
        self.popup_items = group_list.into_iter().map(|s| s.to_string()).collect();
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        // restore persisted selection for this group
        self.addpkgs_group_pkg_selected = self
            .addpkgs_group_selected
            .get(group_name)
            .cloned()
            .unwrap_or_default();
        self.popup_packages_focus = true;
        self.addpkgs_group_pkg_index = 0;
        if let Some(idx) = self
            .addpkgs_group_names
            .iter()
            .position(|n| n == group_name)
        {
            self.addpkgs_group_index = idx;
        }
    }
    pub(crate) fn group_packages_for(group_name: &str) -> Vec<&'static str> {
        match group_name {
            "Terminals" => vec![
                "alacritty",
                "kitty",
                "foot",
                "konsole",
                "gnome-terminal",
                "xterm",
                "wezterm",
                "tilix",
            ],
            "Shells" => vec!["fish", "zsh", "nushell", "dash", "tcsh"],
            "Browsers" => vec!["firefox", "chromium", "qutebrowser", "epiphany"],
            "Test Editors" | "Text Editors" => vec![
                "nano", "vim", "neovim", "micro", "helix", "gedit", "kate", "mousepad",
            ],
            "dotfile Management" => vec!["stow", "chezmoi", "yadm"],
            _ => Vec::new(),
        }
    }
    pub fn apply_additional_package_group_selection(&mut self, show_info: bool) {
        let mut added = 0usize;
        let mut added_names: Vec<String> = Vec::new();
        let mut skipped = 0usize;
        let mut already = 0usize;
        for name in self.addpkgs_group_pkg_selected.clone().into_iter() {
            if self
                .additional_packages
                .iter()
                .any(|p| p.name.eq_ignore_ascii_case(&name))
            {
                already += 1;
                continue;
            }
            if let Some(reason) = self.check_additional_pkg_conflicts(&name) {
                let _ = reason; // ignore but count as skipped
                skipped += 1;
                continue;
            }
            if let Some((repo, pkg_name, version, description)) = self.validate_package(&name) {
                self.additional_packages
                    .push(crate::app::AdditionalPackage {
                        name: pkg_name,
                        repo,
                        version,
                        description,
                    });
                added += 1;
                added_names.push(name);
            } else {
                // Fallback: add with minimal metadata so it appears in Info box
                self.additional_packages
                    .push(crate::app::AdditionalPackage {
                        name: name.clone(),
                        repo: String::new(),
                        version: String::new(),
                        description: String::from("Selected from package groups"),
                    });
                added += 1;
                added_names.push(name);
            }
        }
        // persist selection for this group name
        if let Some(name) = self
            .addpkgs_group_names
            .get(self.addpkgs_group_index)
            .cloned()
        {
            self.addpkgs_group_selected
                .insert(name, self.addpkgs_group_pkg_selected.clone());
        }
        self.addpkgs_group_pkg_selected.clear();
        if show_info && (added > 0 || skipped > 0 || already > 0) {
            let mut msg = String::new();
            if added > 0 {
                msg.push_str("Added packages:\n");
                for n in added_names {
                    msg.push_str("- ");
                    msg.push_str(&n);
                    msg.push('\n');
                }
            }
            if skipped > 0 {
                msg.push_str(&format!("Skipped: {skipped}\n"));
            }
            if already > 0 {
                msg.push_str(&format!("Already present: {already}\n"));
            }
            if msg.is_empty() {
                msg.push_str("No changes");
            }
            self.open_info_popup(msg);
        }
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
        // Find the exact package match, not just the first result
        let mut lines = stdout.lines().filter(|l| !l.trim().is_empty());
        let mut header = None;
        let mut desc_line = String::new();

        // Look for the exact package name match
        while let Some(line) = lines.next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let repo_and_name = parts[0];
                if let Some(pkg_name) = repo_and_name.split('/').nth(1)
                    && pkg_name == name.trim()
                {
                    header = Some(line);
                    // Get the description from the next line
                    if let Some(desc) = lines.next() {
                        desc_line = desc.trim().to_string();
                    }
                    break;
                }
            }
        }

        let header = header?;
        // Parse header
        // Split to repo/name and version
        let mut parts = header.split_whitespace();
        let repo_and_name = parts.next()?; // e.g., extra/stow
        let version = parts.next().unwrap_or("").to_string();

        // Remove installation status markers like [Installiert] or [Installed]
        let version = version
            .split('[')
            .next()
            .unwrap_or(&version)
            .trim()
            .to_string();
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
        // Experience Mode: Desktop, Server, Xorg packages (only consider active selections)
        for env in self.selected_desktop_envs.iter() {
            if let Some(set) = self.selected_env_packages.get(env)
                && set.contains(n)
            {
                return Some("already included in Desktop Environment packages".into());
            }
        }
        for srv in self.selected_server_types.iter() {
            if let Some(set) = self.selected_server_packages.get(srv)
                && set.contains(n)
            {
                return Some("already included in Server packages".into());
            }
        }
        for xorg in self.selected_xorg_types.iter() {
            if let Some(set) = self.selected_xorg_packages.get(xorg)
                && set.contains(n)
            {
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

    // Field 1: Select package groups
    let is_focus_1 = app.addpkgs_focus_index == 1 && matches!(app.focus, super::Focus::Content);
    let bullet_style_1 = if is_focus_1 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let label_style_1 = if is_focus_1 {
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
                if app.addpkgs_focus_index == 1 {
                    "▶"
                } else {
                    " "
                }
            ),
            bullet_style_1,
        ),
        Span::styled("Select package groups".to_string(), label_style_1),
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
                Span::styled(format!("{bullet} "), style),
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
    let is_focus_2 = app.addpkgs_focus_index == 2 && matches!(app.focus, super::Focus::Content);
    let continue_style = if is_focus_2 {
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
