use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use super::app::{
    self, AppState, Focus, INFOBOX_HEIGHT, KEYBINDS_WIDTH, LEFT_MENU_WIDTH, PopupKind,
    RepoSignOption, RepoSignature, Screen,
};

pub fn draw(frame: &mut Frame, app: &mut AppState) {
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

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(INFOBOX_HEIGHT), Constraint::Min(5)])
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
        Screen::SwapPartition => app::swap_partition::draw_swap_partition(frame, app, content_rect),
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

    // popup overlay (if any) drawn next
    if app.popup_open {
        draw_popup(frame, app);
    }

    // command line at bottom of decision area (all sections)
    if app.cmdline_open && app.focus == Focus::Content {
        draw_cmdline(frame, app, content_rect);
    }
}

fn draw_popup(frame: &mut ratatui::Frame, app: &mut AppState) {
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
            | Some(PopupKind::UserAddPassword)
            | Some(PopupKind::UserAddPasswordConfirm)
            | Some(PopupKind::UserAddSudo)
    ) {
        let w = area.width.clamp(20, 34);
        let h = area.height.clamp(5, 7);
        (w, h)
    } else {
        (
            area.width.saturating_mul(2) / 3,
            area.height.saturating_mul(2) / 3,
        )
    };
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    let popup_rect = ratatui::layout::Rect {
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
        Some(PopupKind::RootPassword) => " Enter Root Password ",
        Some(PopupKind::RootPasswordConfirm) => " Confirm Root Password ",
        Some(PopupKind::UserAddUsername) => " Enter Username ",
        Some(PopupKind::UserAddPassword) => " Enter User Password ",
        Some(PopupKind::UserAddPasswordConfirm) => " Confirm User Password ",
        Some(PopupKind::UserAddSudo) => " Should User be Superuser (sudo)? ",
        Some(PopupKind::DisksDeviceList) => " Available Drives ",
        Some(PopupKind::DiskEncryptionType) => " Encryption Type ",
        Some(PopupKind::DiskEncryptionPassword) => " Enter encryption password ",
        Some(PopupKind::DiskEncryptionPasswordConfirm) => " Confirm encryption password ",
        Some(PopupKind::DiskEncryptionPartitionList) => " Select Partition to encrypt ",
        Some(PopupKind::AbortConfirm) => " Confirm Abort (Exit) ",
        Some(PopupKind::Info) => " Info ",
        None => " Select ",
    };

    // Build visible items list (with checkbox for multi-select kinds)
    let is_multi = matches!(
        app.popup_kind,
        Some(PopupKind::MirrorsRegions) | Some(PopupKind::OptionalRepos)
    );
    let is_text_input = matches!(
        app.popup_kind,
        Some(
            PopupKind::MirrorsCustomServerInput
                | PopupKind::MirrorsCustomRepoName
                | PopupKind::MirrorsCustomRepoUrl
                | PopupKind::DiskEncryptionPassword
                | PopupKind::DiskEncryptionPasswordConfirm
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
            | Some(PopupKind::UserAddPassword)
            | Some(PopupKind::UserAddPasswordConfirm)
    ) {
        frame.render_widget(ratatui::widgets::Clear, popup_rect);
        let popup_block = Block::default()
            .borders(Borders::ALL)
            .title(title_text)
            .border_style(Style::default().fg(Color::Yellow));
        frame.render_widget(popup_block, popup_rect);

        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(popup_rect);

        let prompt_text = match app.popup_kind {
            Some(PopupKind::HostnameInput) => "Enter hostname and press Enter:",
            Some(PopupKind::RootPassword) => "Type root password and press Enter:",
            Some(PopupKind::RootPasswordConfirm) => "Re-type root password and press Enter:",
            Some(PopupKind::UserAddUsername) => "Enter username and press Enter:",
            Some(PopupKind::UserAddPassword) => "Type user password and press Enter:",
            Some(PopupKind::UserAddPasswordConfirm) => "Re-type user password and press Enter:",
            _ => "Enter value and press Enter:",
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
                    .title(" Select Yes/No — Should User be Superuser (sudo)? "),
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

    let items: Vec<ListItem> = if is_text_input {
        let prompt = match app.popup_kind {
            Some(PopupKind::MirrorsCustomServerInput) => {
                "Enter a server URL and press Enter to add."
            }
            Some(PopupKind::MirrorsCustomRepoName) => "Enter repository name and press Enter.",
            Some(PopupKind::MirrorsCustomRepoUrl) => "Enter repository URL and press Enter.",
            Some(PopupKind::DiskEncryptionPassword) => "Type password and press Enter.",
            Some(PopupKind::DiskEncryptionPasswordConfirm) => "Re-type password and press Enter.",
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

    // Layout within popup: search on top, list below
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

fn draw_cmdline(frame: &mut ratatui::Frame, app: &mut AppState, area: ratatui::layout::Rect) {
    // Render a one-line command area at the very bottom inside the decision panel
    let cmd_area = ratatui::layout::Rect {
        x: area.x + 1,
        y: area.y + area.height.saturating_sub(3),
        width: area.width.saturating_sub(2),
        height: 3,
    };
    let content = Paragraph::new(Line::from(vec![
        Span::styled(
            ":",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(app.cmdline_buffer.clone()),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Command "))
    .wrap(Wrap { trim: false });
    frame.render_widget(content, cmd_area);
}

// ... existing code ...
