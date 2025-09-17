use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::ui::app::{AppState, PopupKind};

pub fn draw_popup(frame: &mut Frame, app: &mut AppState) {
    let area = frame.area();
    // Smaller popup for simple inputs and yes/no
    let (width, height) = if matches!(
        app.popup_kind,
        Some(PopupKind::AbortConfirm)
            | Some(PopupKind::Info)
            | Some(PopupKind::HostnameInput)
            | Some(PopupKind::RootPassword)
            | Some(PopupKind::RootPasswordConfirm)
            | Some(PopupKind::UserAddUsername)
            | Some(PopupKind::UserEditUsername)
            | Some(PopupKind::UserAddPassword)
            | Some(PopupKind::UserAddPasswordConfirm)
            | Some(PopupKind::AdditionalPackageInput)
            | Some(PopupKind::UserAddSudo)
            | Some(PopupKind::MinimalClearConfirm)
            | Some(PopupKind::DiskEncryptionPassword)
            | Some(PopupKind::DiskEncryptionPasswordConfirm)
            | Some(PopupKind::NetworkIP)
            | Some(PopupKind::NetworkGateway)
            | Some(PopupKind::NetworkDNS)
    ) {
        let w = area.width.clamp(20, 34);
        let h = area.height.clamp(7, 9);
        (w, h)
    } else {
        (
            area.width.saturating_mul(2) / 3,
            area.height.saturating_mul(2) / 3,
        )
    };
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    let popup_rect = Rect {
        x,
        y,
        width,
        height,
    };

    // Title based on kind
    let title_text = match app.popup_kind {
        Some(PopupKind::KeyboardLayout) => " Select Keyboard layout ",
        Some(PopupKind::LocaleLanguage) => " Select Locale language ",
        Some(PopupKind::LocaleEncoding) => " Select Locale encoding ",
        Some(PopupKind::MirrorsRegions) => " Select Regions (space to toggle) ",
        Some(PopupKind::MirrorsCustomServerInput) => " Add custom server URL (Enter to add) ",
        Some(PopupKind::MirrorsCustomRepoName) => " Custom repo: name ",
        Some(PopupKind::MirrorsCustomRepoUrl) => " Custom repo: URL ",
        Some(PopupKind::MirrorsCustomRepoSig) => " Custom repo: signature (Enter) ",
        Some(PopupKind::MirrorsCustomRepoSignOpt) => " Custom repo: sign option (Enter) ",
        Some(PopupKind::OptionalRepos) => " Optional repositories (space to toggle) ",
        Some(PopupKind::HostnameInput) => " Enter Hostname ",
        Some(PopupKind::AdditionalPackageInput) => " Add package (Enter to add) ",
        Some(PopupKind::RootPassword) => " Enter Root Password ",
        Some(PopupKind::RootPasswordConfirm) => " Confirm Root Password ",
        Some(PopupKind::NetworkInterfaces) => " Select interface ",
        Some(PopupKind::NetworkMode) => " Select mode ",
        Some(PopupKind::NetworkIP) => " Enter IP address/CIDR ",
        Some(PopupKind::NetworkGateway) => " Enter Gateway/Router (optional) ",
        Some(PopupKind::NetworkDNS) => " Enter DNS (optional, e.g., 1.1.1.1) ",
        Some(PopupKind::UserAddUsername) => " Enter Username ",
        Some(PopupKind::UserAddPassword) => " Enter User Password ",
        Some(PopupKind::UserAddPasswordConfirm) => " Confirm User Password ",
        Some(PopupKind::UserAddSudo) => " Should User be Superuser (sudo)? ",
        Some(PopupKind::MinimalClearConfirm) => " Clear selections for Minimal? ",
        Some(PopupKind::ServerTypeSelect) => " Select Server Type (space to toggle) ",
        Some(PopupKind::XorgTypeSelect) => " Select Xorg (space to toggle)",
        Some(PopupKind::DisksDeviceList) => " Available Drives ",
        Some(PopupKind::DiskEncryptionType) => " Encryption Type ",
        Some(PopupKind::DiskEncryptionPassword) => " Enter encryption password ",
        Some(PopupKind::DiskEncryptionPasswordConfirm) => " Confirm encryption password ",
        Some(PopupKind::DiskEncryptionPartitionList) => " Select Partition to encrypt ",
        Some(PopupKind::AbortConfirm) => " Confirm Abort (Exit) ",
        Some(PopupKind::DesktopEnvSelect) => {
            " Select Desktop/WM (space to toggle, Enter to close) "
        }
        Some(PopupKind::Info) => " Info ",
        Some(PopupKind::KernelSelect) => " Select Kernels (space to toggle) ",
        Some(PopupKind::UserSelectEdit) => " Select user to edit ",
        Some(PopupKind::UserSelectDelete) => " Select user to delete ",
        Some(PopupKind::UserEditUsername) => " Edit username ",
        Some(PopupKind::TimezoneSelect) => " Select Timezone ",
        None => " Select ",
    };

    // Build visible items list (with checkbox for multi-select kinds)
    let is_multi = matches!(
        app.popup_kind,
        Some(PopupKind::MirrorsRegions)
            | Some(PopupKind::OptionalRepos)
            | Some(PopupKind::DesktopEnvSelect)
            | Some(PopupKind::XorgTypeSelect)
            | Some(PopupKind::KernelSelect)
    );
    let is_text_input = matches!(
        app.popup_kind,
        Some(
            PopupKind::MirrorsCustomServerInput
                | PopupKind::MirrorsCustomRepoName
                | PopupKind::MirrorsCustomRepoUrl
                | PopupKind::DiskEncryptionPassword
                | PopupKind::DiskEncryptionPasswordConfirm
                | PopupKind::AdditionalPackageInput
        )
    );
    let is_info = matches!(app.popup_kind, Some(PopupKind::Info));

    if is_info {
        // Clear area to avoid underlying text leaking through
        frame.render_widget(ratatui::widgets::Clear, popup_rect);
        // Draw popup frame
        let popup_block = Block::default()
            .borders(Borders::ALL)
            .title(title_text)
            .border_style(Style::default().fg(Color::Yellow));
        frame.render_widget(popup_block, popup_rect);

        // Content area (use inner layout, but no search/list chrome)
        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3)])
            .split(popup_rect);

        let body = if app.popup_items.is_empty() {
            String::new()
        } else {
            app.popup_items[0].clone()
        };
        let lines = vec![
            Line::from(body),
            Line::from(""),
            Line::from("Press Enter or ESC to close."),
        ];
        let content = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        frame.render_widget(content, inner[0]);
        return;
    }

    // Dedicated, small, single-line input popup (no search/options)
    if matches!(
        app.popup_kind,
        Some(PopupKind::HostnameInput)
            | Some(PopupKind::RootPassword)
            | Some(PopupKind::RootPasswordConfirm)
            | Some(PopupKind::UserAddUsername)
            | Some(PopupKind::UserEditUsername)
            | Some(PopupKind::UserAddPassword)
            | Some(PopupKind::UserAddPasswordConfirm)
            | Some(PopupKind::DiskEncryptionPassword)
            | Some(PopupKind::DiskEncryptionPasswordConfirm)
            | Some(PopupKind::AdditionalPackageInput)
            | Some(PopupKind::NetworkIP)
            | Some(PopupKind::NetworkGateway)
            | Some(PopupKind::NetworkDNS)
    ) {
        frame.render_widget(ratatui::widgets::Clear, popup_rect);
        let popup_block = Block::default()
            .borders(Borders::ALL)
            .title(title_text)
            .border_style(Style::default().fg(Color::Yellow));
        // Compute inner area before moving the block into render_widget
        let inner_area = popup_block.inner(popup_rect);
        frame.render_widget(popup_block, popup_rect);

        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(inner_area);

        let prompt_text = match app.popup_kind {
            Some(PopupKind::HostnameInput) => "Enter hostname:",
            Some(PopupKind::AdditionalPackageInput) => "Enter package name:",
            Some(PopupKind::RootPassword) => "Type root password:",
            Some(PopupKind::RootPasswordConfirm) => "Re-type root password:",
            Some(PopupKind::UserAddUsername) => "Enter username:",
            Some(PopupKind::UserEditUsername) => "Edit username:",
            Some(PopupKind::UserAddPassword) => "Type user password:",
            Some(PopupKind::UserAddPasswordConfirm) => "Re-type user password:",
            Some(PopupKind::DiskEncryptionPassword) => "Type encryption password:",
            Some(PopupKind::DiskEncryptionPasswordConfirm) => "Re-type encryption password:",
            Some(PopupKind::NetworkIP) => "Enter IPv4 optional), e.g., 192.168.1.1 or 192.168.1.1/24:",
            Some(PopupKind::NetworkGateway) => "Enter gateway (optional):",
            Some(PopupKind::NetworkDNS) => "Enter DNS (optional):",
            _ => "Enter value:",
        };
        let prompt = Paragraph::new(Line::from(prompt_text))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        frame.render_widget(prompt, inner[0]);

        let masked = if matches!(
            app.popup_kind,
            Some(
                PopupKind::RootPassword
                    | PopupKind::RootPasswordConfirm
                    | PopupKind::UserAddPassword
                    | PopupKind::UserAddPasswordConfirm
                    | PopupKind::DiskEncryptionPassword
                    | PopupKind::DiskEncryptionPasswordConfirm
            )
        ) {
            "*".repeat(app.custom_input_buffer.chars().count())
        } else {
            app.custom_input_buffer.clone()
        };
        let input_line = Paragraph::new(Line::from(vec![
            Span::styled(
                "> ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(masked),
        ]))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
        frame.render_widget(input_line, inner[1]);

        let hint = Paragraph::new(Line::from("ESC to cancel"))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        frame.render_widget(hint, inner[2]);
        // Bottom example line for network inputs
        if matches!(
            app.popup_kind,
            Some(
                PopupKind::NetworkIP | PopupKind::NetworkGateway | PopupKind::NetworkDNS
            )
        ) {
            let example_text = match app.popup_kind {
                Some(PopupKind::NetworkIP) =>
                    "E.g. 192.168.1.1/24",
                Some(PopupKind::NetworkGateway) => "E.g. 192.168.1.1",
                Some(PopupKind::NetworkDNS) => "E.g. 1.1.1.1",
                _ => "",
            };
            let example = Paragraph::new(Line::from(example_text))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false });
            // inner[3] is the bottom area (Min(1))
            frame.render_widget(example, inner[3]);
        }
        return;
    }

    // Minimal yes/no popup without search for sudo selection
    if matches!(app.popup_kind, Some(PopupKind::UserAddSudo)) {
        // Clear area
        frame.render_widget(ratatui::widgets::Clear, popup_rect);
        // Frame
        let popup_block = Block::default()
            .borders(Borders::ALL)
            .title(title_text)
            .border_style(Style::default().fg(Color::Yellow));
        frame.render_widget(popup_block, popup_rect);

        // Single list area inside
        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3)])
            .split(popup_rect);

        let items: Vec<ListItem> = app
            .popup_visible_indices
            .iter()
            .map(|&i| ListItem::new(app.popup_items[i].clone()))
            .collect();
        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(app.popup_selected_visible));
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Should User be Superuser (sudo)? "),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(list, inner[0], &mut state);
        return;
    }

    // Minimal clear selections confirmation (compact with description + yes/no)
    if matches!(app.popup_kind, Some(PopupKind::MinimalClearConfirm)) {
        frame.render_widget(ratatui::widgets::Clear, popup_rect);
        let popup_block = Block::default()
            .borders(Borders::ALL)
            .title(title_text)
            .border_style(Style::default().fg(Color::Yellow));
        let inner_area = popup_block.inner(popup_rect);
        frame.render_widget(popup_block, popup_rect);

        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Min(3),
            ])
            .split(inner_area);

        let desc = vec![
            Line::from("Selecting 'Minimal' will:"),
            Line::from("- Clear Desktop/WM, packages, and Login Manager"),
        ];
        let desc_widget = Paragraph::new(desc)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });
        frame.render_widget(desc_widget, inner[0]);

        let hint = Paragraph::new(Line::from("Confirm? (Enter=Yes, ESC=No)"))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        frame.render_widget(hint, inner[1]);

        let items: Vec<ListItem> = vec![ListItem::new("Yes"), ListItem::new("No")];
        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(app.popup_selected_visible.min(1)));
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Options "))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(list, inner[2], &mut state);
        return;
    }

    let items: Vec<ListItem> = if is_text_input {
        let prompt = match app.popup_kind {
            Some(PopupKind::MirrorsCustomServerInput) => {
                "Enter a server URL and press Enter to add."
            }
            Some(PopupKind::MirrorsCustomRepoName) => "Enter repository name and press Enter.",
            Some(PopupKind::MirrorsCustomRepoUrl) => "Enter repository URL and press Enter.",
            Some(PopupKind::DiskEncryptionPassword) => "Type password and press Enter.",
            Some(PopupKind::DiskEncryptionPasswordConfirm) => "Re-type password and press Enter.",
            Some(PopupKind::AdditionalPackageInput) => "Enter package name and press Enter.",
            _ => "",
        };
        let display_buffer = if matches!(
            app.popup_kind,
            Some(PopupKind::DiskEncryptionPassword | PopupKind::DiskEncryptionPasswordConfirm)
        ) {
            "*".repeat(app.custom_input_buffer.chars().count())
        } else {
            app.custom_input_buffer.clone()
        };
        vec![ListItem::new(vec![
            Line::from(prompt),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "> ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(display_buffer),
            ]),
            Line::from(""),
            Line::from("Press ESC to close when finished."),
        ])]
    } else {
        app.popup_visible_indices
            .iter()
            .map(|&i| {
                if is_multi {
                    let checked = match app.popup_kind {
                        Some(PopupKind::MirrorsRegions) => {
                            app.mirrors_regions_selected.contains(&i)
                        }
                        Some(PopupKind::OptionalRepos) => app.optional_repos_selected.contains(&i),
                        Some(PopupKind::KernelSelect) => {
                            if let Some(name) = app.popup_items.get(i) {
                                app.selected_kernels.contains(name)
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };
                    let marker = if checked { "[x]" } else { "[ ]" };
                    ListItem::new(format!("{} {}", marker, app.popup_items[i]))
                } else {
                    ListItem::new(app.popup_items[i].clone())
                }
            })
            .collect()
    };

    let mut state = ratatui::widgets::ListState::default();
    state.select(if is_text_input {
        None
    } else {
        Some(app.popup_selected_visible)
    });

    // Search line
    let search_label = if app.popup_in_search { "/" } else { "" };
    let search = Paragraph::new(Line::from(vec![
        Span::styled(
            search_label,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(app.popup_search_query.clone()),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Search "))
    .wrap(Wrap { trim: false });

    // Layout within popup: search on top, two columns below for Desktop select
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(popup_rect);

    // Clear area to avoid underlying text leaking through
    frame.render_widget(ratatui::widgets::Clear, popup_rect);

    // Draw popup frame
    let popup_block = Block::default()
        .borders(Borders::ALL)
        .title(title_text)
        .border_style(Style::default().fg(Color::Yellow));
    frame.render_widget(popup_block, popup_rect);

    // Draw search and list inside
    frame.render_widget(search, inner[0]);

    // Optional header for device list
    if matches!(app.popup_kind, Some(PopupKind::DisksDeviceList)) {
        let header = Paragraph::new(Line::from(
            "Model/Name                      | Path                 | Type   | Size       | Free Space   | Sector size     | Read only"
        ))
        .block(Block::default().borders(Borders::ALL).title(" Columns "))
        .wrap(Wrap { trim: false });
        // Split inner[1] to place header above list
        let inner_list = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(3)])
            .split(inner[1]);
        frame.render_widget(header, inner_list[0]);
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Options "))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(list, inner_list[1], &mut state);
    } else if matches!(app.popup_kind, Some(PopupKind::DesktopEnvSelect)) {
        // Two columns: left selection (split by type), right info panel
        // Give minimal fixed width to Desktop Environments/Window Managers
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(28), Constraint::Min(1)])
            .split(inner[1]);

        // Split visible indices into Desktop Environments and Window Managers
        let is_wm = |name: &str| -> bool {
            matches!(
                name,
                "Awesome" | "Bspwm" | "Enlightenment" | "Hyprland" | "Qtile" | "Sway" | "i3-wm"
            )
        };

        let mut de_visible_positions: Vec<usize> = Vec::new();
        let mut wm_visible_positions: Vec<usize> = Vec::new();
        for (vis_pos, &global_idx) in app.popup_visible_indices.iter().enumerate() {
            if let Some(name) = app.popup_items.get(global_idx) {
                if is_wm(name) {
                    wm_visible_positions.push(vis_pos);
                } else {
                    de_visible_positions.push(vis_pos);
                }
            }
        }

        // Build list items for each group
        let de_items: Vec<ListItem> = de_visible_positions
            .iter()
            .filter_map(|&vis_pos| app.popup_visible_indices.get(vis_pos).copied())
            .map(|global_idx| {
                let name = app.popup_items[global_idx].clone();
                let marker = if app.selected_desktop_envs.contains(&name) {
                    "[x]"
                } else {
                    "[ ]"
                };
                ListItem::new(format!("{} {}", marker, name))
            })
            .collect();
        let wm_items: Vec<ListItem> = wm_visible_positions
            .iter()
            .filter_map(|&vis_pos| app.popup_visible_indices.get(vis_pos).copied())
            .map(|global_idx| {
                let name = app.popup_items[global_idx].clone();
                let marker = if app.selected_desktop_envs.contains(&name) {
                    "[x]"
                } else {
                    "[ ]"
                };
                ListItem::new(format!("{} {}", marker, name))
            })
            .collect();

        // Determine which group the current selection is in
        let selected_vis = app.popup_selected_visible;
        let de_selected_local = de_visible_positions.iter().position(|&p| p == selected_vis);
        let wm_selected_local = wm_visible_positions.iter().position(|&p| p == selected_vis);

        // Left column splits vertically: Desktop Environments (top), Window Managers (bottom)
        let left_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(cols[0]);

        // Desktop Environments list
        let mut de_state = ratatui::widgets::ListState::default();
        de_state.select(de_selected_local);
        let de_focused =
            !app.popup_packages_focus && !app.popup_login_focus && de_selected_local.is_some();
        let de_title = if de_focused {
            " Desktop Environments (focused) "
        } else {
            " Desktop Environments "
        };
        let de_highlight = if de_focused {
            Color::Yellow
        } else {
            Color::White
        };
        let de_list = List::new(de_items)
            .block(Block::default().borders(Borders::ALL).title(de_title))
            .highlight_style(
                Style::default()
                    .fg(de_highlight)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(de_list, left_split[0], &mut de_state);

        // Window Managers list
        let mut wm_state = ratatui::widgets::ListState::default();
        wm_state.select(wm_selected_local);
        let wm_focused =
            !app.popup_packages_focus && !app.popup_login_focus && wm_selected_local.is_some();
        let wm_title = if wm_focused {
            " Window Managers (focused) "
        } else {
            " Window Managers "
        };
        let wm_highlight = if wm_focused {
            Color::Yellow
        } else {
            Color::White
        };
        let wm_list = List::new(wm_items)
            .block(Block::default().borders(Borders::ALL).title(wm_title))
            .highlight_style(
                Style::default()
                    .fg(wm_highlight)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(wm_list, left_split[1], &mut wm_state);

        // Right info column
        let selected_label = if let Some(&global_idx) =
            app.popup_visible_indices.get(app.popup_selected_visible)
            && let Some(name) = app.popup_items.get(global_idx)
        {
            name.clone()
        } else {
            String::new()
        };
        let env_type = match selected_label.as_str() {
            "Awesome" | "Bspwm" | "Enlightenment" | "Hyprland" | "Qtile" | "Sway" | "i3-wm" => {
                "Window Manager"
            }
            _ => "Desktop Environment",
        };
        // Build package list for selected env
        let packages: Vec<&str> = match selected_label.as_str() {
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
                // Common tools
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
                // Common tools
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
                "polkit",
                "waybar",
                "xorg-xwayland",
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
            "Xfce4" => vec![
                "gvfs",
                "pavucontrol",
                "xarchiver",
                "xfce4",
                "xfce4-goodies",
                // Common tools
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
            _ => vec![],
        };

        // Default to all packages selected for this env
        let selected_set = app
            .selected_env_packages
            .entry(selected_label.clone())
            .or_insert_with(|| packages.iter().map(|s| s.to_string()).collect());

        // Build checkbox list items for packages
        let pkg_items: Vec<ListItem> = packages
            .iter()
            .map(|name| {
                let marker = if selected_set.contains(*name) {
                    "[x]"
                } else {
                    "[ ]"
                };
                ListItem::new(format!("{} {}", marker, name))
            })
            .collect();

        // Right side: two columns (packages+drivers in left; login manager in right)
        let right_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(cols[1]);

        // Split the left info column horizontally: Installed packages (left) | Graphic Drivers (right)
        let pkg_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(right_cols[0]);

        // Packages list with selection highlight
        let mut pkg_state = ratatui::widgets::ListState::default();
        let pkg_len = pkg_items.len();
        if pkg_len > 0 {
            if app.popup_packages_selected_index >= pkg_len {
                app.popup_packages_selected_index = pkg_len - 1;
            }
            pkg_state.select(Some(app.popup_packages_selected_index));
        } else {
            pkg_state.select(None);
        }
        let pkg_title = if app.popup_packages_focus {
            format!(" Installed packages — {} (focused) ", env_type)
        } else {
            format!(" Installed packages — {} ", env_type)
        };
        let pkg_highlight = if app.popup_packages_focus {
            Color::Yellow
        } else {
            Color::White
        };
        let pkg_list = List::new(pkg_items)
            .block(Block::default().borders(Borders::ALL).title(pkg_title))
            .highlight_style(
                Style::default()
                    .fg(pkg_highlight)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(pkg_list, pkg_cols[0], &mut pkg_state);

        // Graphic Drivers list (multi-select)
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
        // Build display items with checkboxes; non-selectable headers have no checkbox
        let driver_items: Vec<ListItem> = drivers_all
            .iter()
            .map(|(name, selectable)| {
                if *selectable {
                    let marker = if app.selected_graphic_drivers.contains(*name) {
                        "[x]"
                    } else {
                        "[ ]"
                    };
                    ListItem::new(format!("{} {}", marker, name))
                } else {
                    let title_span = Span::styled(
                        name.trim(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    );
                    ListItem::new(Line::from(title_span))
                }
            })
            .collect();
        let mut drv_state = ratatui::widgets::ListState::default();
        // clamp index to selectable items range length
        let drv_len = driver_items.len();
        if drv_len == 0 {
            drv_state.select(None);
        } else {
            if app.popup_drivers_selected_index >= drv_len {
                app.popup_drivers_selected_index = drv_len - 1;
            }
            drv_state.select(Some(app.popup_drivers_selected_index));
        }
        let drv_title = if app.popup_drivers_focus {
            " Graphic Drivers (focused) "
        } else {
            " Graphic Drivers "
        };
        let drv_highlight = if app.popup_drivers_focus {
            Color::Yellow
        } else {
            Color::White
        };
        let drv_list = List::new(driver_items)
            .block(Block::default().borders(Borders::ALL).title(drv_title))
            .highlight_style(
                Style::default()
                    .fg(drv_highlight)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(drv_list, pkg_cols[1], &mut drv_state);

        // Login manager list (selectable) with default per environment
        let login_managers = [
            "none".to_string(),
            "gdm".to_string(),
            "lightdm-gtk-greeter".to_string(),
            "lightdm-slick-greeter".to_string(),
            "ly".to_string(),
            "sddm".to_string(),
        ];
        // Determine default based on env
        let default_lm: Option<&str> = match selected_label.as_str() {
            "GNOME" => Some("gdm"),
            "KDE Plasma" | "Hyprland" | "Cutefish" | "Lxqt" => Some("sddm"),
            "Budgie" => Some("lightdm-slick-greeter"),
            "Bspwm" | "Cinnamon" | "Deepin" | "Enlightenment" | "Mate" | "Qtile" | "Sway"
            | "Xfce4" | "i3-wm" => Some("lightdm-gtk-greeter"),
            _ => None,
        };
        let selected_lm = if app.login_manager_user_set {
            app.selected_login_manager.clone()
        } else {
            app.selected_login_manager
                .clone()
                .or_else(|| default_lm.map(|s| s.to_string()))
        };

        let lm_items: Vec<ListItem> = login_managers
            .iter()
            .map(|name| {
                let is_selected = match &selected_lm {
                    Some(s) => s == name,
                    None => name == "none",
                };
                let marker = if is_selected { "[x]" } else { "[ ]" };
                ListItem::new(format!("{} {}", marker, name))
            })
            .collect();
        let mut lm_state = ratatui::widgets::ListState::default();
        lm_state.select(Some(
            app.popup_login_selected_index
                .min(login_managers.len().saturating_sub(1)),
        ));
        let lm_title = if app.popup_login_focus {
            " Login Manager (focused) "
        } else {
            " Login Manager "
        };
        let lm_highlight = if app.popup_login_focus {
            Color::Yellow
        } else {
            Color::White
        };
        let lm_list = List::new(lm_items)
            .block(Block::default().borders(Borders::ALL).title(lm_title))
            .highlight_style(
                Style::default()
                    .fg(lm_highlight)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(lm_list, right_cols[1], &mut lm_state);
    } else if matches!(app.popup_kind, Some(PopupKind::ServerTypeSelect)) {
        // Server Type single column (left) and Installed packages (right)
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner[1]);

        // Build server items with checkboxes
        let server_items: Vec<ListItem> = app
            .popup_visible_indices
            .iter()
            .filter_map(|&vis| app.popup_items.get(vis).cloned())
            .map(|name| {
                let marker = if app.selected_server_types.contains(&name) {
                    "[x]"
                } else {
                    "[ ]"
                };
                ListItem::new(format!("{} {}", marker, name))
            })
            .collect();
        let mut server_state = ratatui::widgets::ListState::default();
        server_state.select(Some(app.popup_selected_visible));
        let server_focused = !app.popup_packages_focus;
        let server_title = if server_focused {
            " Server Types (focused) "
        } else {
            " Server Types "
        };
        let server_highlight = if server_focused {
            Color::Yellow
        } else {
            Color::White
        };
        let server_list = List::new(server_items)
            .block(Block::default().borders(Borders::ALL).title(server_title))
            .highlight_style(
                Style::default()
                    .fg(server_highlight)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(server_list, cols[0], &mut server_state);

        // Packages selector for currently selected server type (multi-select, default all)
        let selected_label = if let Some(&global_idx) =
            app.popup_visible_indices.get(app.popup_selected_visible)
            && let Some(name) = app.popup_items.get(global_idx)
        {
            name.clone()
        } else {
            String::new()
        };
        let pkgs: Vec<&str> = match selected_label.as_str() {
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
        // ensure entry exists with defaults
        if !selected_label.is_empty() {
            let set = app
                .selected_server_packages
                .entry(selected_label.clone())
                .or_insert_with(|| pkgs.iter().map(|s| s.to_string()).collect());
            let pkg_items: Vec<ListItem> = pkgs
                .iter()
                .map(|name| {
                    let marker = if set.contains(*name) {
                        "[x]"
                    } else {
                        "[ ]"
                    };
                    ListItem::new(format!("{} {}", marker, name))
                })
                .collect();
            let mut pkg_state = ratatui::widgets::ListState::default();
            // reuse popup_packages_selected_index for consistency across popups
            let mut idx = app.popup_packages_selected_index;
            if idx >= pkg_items.len() {
                idx = pkg_items.len().saturating_sub(1);
            }
            pkg_state.select(Some(idx));
            let pkg_title = if app.popup_packages_focus {
                " Installed packages (focused) "
            } else {
                " Installed packages "
            };
            let pkg_highlight = if app.popup_packages_focus {
                Color::Yellow
            } else {
                Color::White
            };
            let pkg_list = List::new(pkg_items)
                .block(Block::default().borders(Borders::ALL).title(pkg_title))
                .highlight_style(
                    Style::default()
                        .fg(pkg_highlight)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");
            frame.render_stateful_widget(pkg_list, cols[1], &mut pkg_state);
        }
    } else if matches!(app.popup_kind, Some(PopupKind::XorgTypeSelect)) {
        // Xorg Type single column (left) and Installed packages + Graphic Drivers (right)
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner[1]);

        // Build Xorg items with checkboxes
        let xorg_items: Vec<ListItem> = app
            .popup_visible_indices
            .iter()
            .filter_map(|&vis| app.popup_items.get(vis).cloned())
            .map(|name| {
                let marker = if app.selected_xorg_types.contains(&name) {
                    "[x]"
                } else {
                    "[ ]"
                };
                ListItem::new(format!("{} {}", marker, name))
            })
            .collect();
        let mut xorg_state = ratatui::widgets::ListState::default();
        xorg_state.select(Some(app.popup_selected_visible));
        let xorg_focused = !app.popup_packages_focus;
        let xorg_title = if xorg_focused {
            " Xorg (focused) "
        } else {
            " Xorg "
        };
        let xorg_highlight = if xorg_focused {
            Color::Yellow
        } else {
            Color::White
        };
        let xorg_list = List::new(xorg_items)
            .block(Block::default().borders(Borders::ALL).title(xorg_title))
            .highlight_style(
                Style::default()
                    .fg(xorg_highlight)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(xorg_list, cols[0], &mut xorg_state);

        // Packages for Xorg (multi-select, default xorg-server)
        let selected_label = if let Some(&global_idx) =
            app.popup_visible_indices.get(app.popup_selected_visible)
            && let Some(name) = app.popup_items.get(global_idx)
        {
            name.clone()
        } else {
            String::new()
        };
        let pkgs: Vec<&str> = match selected_label.as_str() {
            "Xorg" => vec!["xorg-server"],
            _ => Vec::new(),
        };
        if !selected_label.is_empty() {
            let set = app
                .selected_xorg_packages
                .entry(selected_label.clone())
                .or_insert_with(|| pkgs.iter().map(|s| s.to_string()).collect());
            let pkg_items: Vec<ListItem> = pkgs
                .iter()
                .map(|name| {
                    let marker = if set.contains(*name) {
                        "[x]"
                    } else {
                        "[ ]"
                    };
                    ListItem::new(format!("{} {}", marker, name))
                })
                .collect();
            // Split right column horizontally: Installed packages | Graphic Drivers
            let right_split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(cols[1]);

            let mut pkg_state = ratatui::widgets::ListState::default();
            let mut idx = app.popup_packages_selected_index;
            if idx >= pkg_items.len() {
                idx = pkg_items.len().saturating_sub(1);
            }
            pkg_state.select(Some(idx));
            let pkg_title = if app.popup_packages_focus {
                " Installed packages (focused) "
            } else {
                " Installed packages "
            };
            let pkg_highlight = if app.popup_packages_focus {
                Color::Yellow
            } else {
                Color::White
            };
            let pkg_list = List::new(pkg_items)
                .block(Block::default().borders(Borders::ALL).title(pkg_title))
                .highlight_style(
                    Style::default()
                        .fg(pkg_highlight)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");
            frame.render_stateful_widget(pkg_list, right_split[0], &mut pkg_state);

            // Graphic Drivers list
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
            let driver_items: Vec<ListItem> = drivers_all
                .iter()
                .map(|(name, selectable)| {
                    if *selectable {
                        let marker = if app.selected_graphic_drivers.contains(*name) {
                            "[x]"
                        } else {
                            "[ ]"
                        };
                        ListItem::new(format!("{} {}", marker, name))
                    } else {
                        let title_span = Span::styled(
                            name.trim(),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        );
                        ListItem::new(Line::from(title_span))
                    }
                })
                .collect();
            let mut drv_state = ratatui::widgets::ListState::default();
            let drv_len = driver_items.len();
            if drv_len == 0 {
                drv_state.select(None);
            } else {
                if app.popup_drivers_selected_index >= drv_len {
                    app.popup_drivers_selected_index = drv_len - 1;
                }
                drv_state.select(Some(app.popup_drivers_selected_index));
            }
            let drv_title = if app.popup_drivers_focus {
                " Graphic Drivers (focused) "
            } else {
                " Graphic Drivers "
            };
            let drv_highlight = if app.popup_drivers_focus {
                Color::Yellow
            } else {
                Color::White
            };
            let drv_list = List::new(driver_items)
                .block(Block::default().borders(Borders::ALL).title(drv_title))
                .highlight_style(
                    Style::default()
                        .fg(drv_highlight)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");
            frame.render_stateful_widget(drv_list, right_split[1], &mut drv_state);
        }
    } else {
        let list_title = match app.popup_kind {
            Some(PopupKind::AbortConfirm) => " Save choices before exit? ",
            _ => " Options ",
        };
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(list_title))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(list, inner[1], &mut state);
    }
}
