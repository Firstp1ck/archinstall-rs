use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::app::{AppState, PopupKind};

pub fn draw(frame: &mut Frame, app: &mut AppState, area: Rect) {
    // Optional header specifically for device list
    if matches!(app.popup_kind, Some(PopupKind::DisksDeviceList)) {
        let header = Paragraph::new(Line::from(
            "Model/Name                      | Path                 | Type   | Size       | Free Space   | Sector size     | Read only",
        ))
        .block(Block::default().borders(Borders::ALL).title(" Columns "))
        .wrap(Wrap { trim: false });
        let inner_list = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Min(3),
            ])
            .split(area);
        frame.render_widget(header, inner_list[0]);
        let list = List::new(build_items(app))
            .block(Block::default().borders(Borders::ALL).title(" Options "))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(app.popup_selected_visible));
        frame.render_stateful_widget(list, inner_list[1], &mut state);
        return;
    }

    if matches!(app.popup_kind, Some(PopupKind::ManualPartitionTable)) {
        let header = Paragraph::new(Line::from(
            "Status   | Path                 | Type               | Start        | End          | Size       | Filesystem Type | Mountpoint   | Mount options",
        ))
        .block(Block::default().borders(Borders::ALL).title(" Columns "))
        .wrap(Wrap { trim: false });
        let inner_list = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Min(3),
            ])
            .split(area);
        frame.render_widget(header, inner_list[0]);
        let list = List::new(build_items(app))
            .block(Block::default().borders(Borders::ALL).title(" Options "))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(app.popup_selected_visible));
        frame.render_stateful_widget(list, inner_list[1], &mut state);
        return;
    }

    if matches!(app.popup_kind, Some(PopupKind::ManualPartitionCreate)) {
        // Info table + input + unit selector
        let layout = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(8),
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Min(3),
            ])
            .split(area);

        let start = format!("{}", app.manual_create_free_start_bytes);
        let end = format!("{}", app.manual_create_free_end_bytes);
        let remaining = app
            .manual_create_free_end_bytes
            .saturating_sub(app.manual_create_free_start_bytes);
        let (unit_label, unit_divisor): (&str, u64) = match app.manual_create_units_index {
            0 => ("B", 1),
            1 => ("KiB", 1024),
            2 => ("MiB", 1024 * 1024),
            3 => ("GiB", 1024 * 1024 * 1024),
            _ => ("GiB", 1024 * 1024 * 1024),
        };
        let size_live = if app.custom_input_buffer.trim().is_empty() {
            format!("{} {}", remaining / unit_divisor, unit_label)
        } else {
            format!("{} {}", app.custom_input_buffer.clone(), unit_label)
        };
        let info = Paragraph::new(Line::from(format!(
            "Start | {}    \nEnd   | {}    \nSize  | {}",
            start, end, size_live
        )))
        .block(Block::default().borders(Borders::ALL).title(" Info "))
        .wrap(Wrap { trim: false });
        frame.render_widget(info, layout[0]);

        // Input field (reuse style)
        let input = Paragraph::new(Line::from(vec![
            ratatui::text::Span::styled(
                "> ",
                Style::default()
                    .fg(if app.manual_create_focus_units {
                        Color::White
                    } else {
                        Color::Yellow
                    })
                    .add_modifier(Modifier::BOLD),
            ),
            ratatui::text::Span::raw(app.custom_input_buffer.clone()),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if app.manual_create_focus_units {
                    format!(" Size ({}) ", unit_label)
                } else {
                    format!(" Size ({}) (focused) ", unit_label)
                }),
        )
        .wrap(Wrap { trim: false });
        frame.render_widget(input, layout[1]);

        // Units selector
        let units = ["B", "KiB / KB", "MiB / MB", "GiB / GB"];
        let mut items: Vec<ListItem> = Vec::new();
        for (i, u) in units.iter().enumerate() {
            let marker = if i == app.manual_create_units_index {
                "[x]"
            } else {
                "[ ]"
            };
            items.push(ListItem::new(format!("{} {}", marker, u)));
        }
        let list =
            List::new(items)
                .block(Block::default().borders(Borders::ALL).title(
                    if app.manual_create_focus_units {
                        " Units (focused) "
                    } else {
                        " Units (Enter to select) "
                    },
                ))
                .highlight_style(
                    Style::default()
                        .fg(if app.manual_create_focus_units {
                            Color::Yellow
                        } else {
                            Color::White
                        })
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");
        let mut state = ratatui::widgets::ListState::default();
        if app.manual_create_focus_units {
            state.select(Some(app.manual_create_units_index));
        } else {
            state.select(None);
        }
        frame.render_stateful_widget(list, layout[2], &mut state);
        return;
    }

    if matches!(app.popup_kind, Some(PopupKind::ManualPartitionKindSelect)) {
        let list = List::new(build_items(app))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Select Type "),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(app.popup_selected_visible));
        frame.render_stateful_widget(list, area, &mut state);
        return;
    }

    if matches!(app.popup_kind, Some(PopupKind::ManualPartitionFilesystem)) {
        let list = List::new(build_items(app))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Select Filesystem "),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(app.popup_selected_visible));
        frame.render_stateful_widget(list, area, &mut state);
        return;
    }

    if matches!(app.popup_kind, Some(PopupKind::ManualPartitionMountpoint)) {
        let input = Paragraph::new(Line::from(vec![
            ratatui::text::Span::styled(
                "> ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            ratatui::text::Span::raw(app.custom_input_buffer.clone()),
        ]))
        .block(Block::default().borders(Borders::ALL).title(" Mountpoint "))
        .wrap(Wrap { trim: false });
        frame.render_widget(input, area);
        return;
    }

    // Special two-pane layout for AdditionalPackageGroupSelect to show packages side-by-side
    if matches!(
        app.popup_kind,
        Some(PopupKind::AdditionalPackageGroupSelect)
    ) {
        let cols = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Length(26),
                ratatui::layout::Constraint::Min(10),
            ])
            .split(area);

        // Left: groups
        let mut state_left = ratatui::widgets::ListState::default();
        state_left.select(Some(app.popup_selected_visible));
        let left_title = if !app.popup_packages_focus {
            " Groups (focused) "
        } else {
            " Groups "
        };
        let left_highlight = if !app.popup_packages_focus {
            Color::Yellow
        } else {
            Color::White
        };
        let list_left = List::new(build_items(app))
            .block(Block::default().borders(Borders::ALL).title(left_title))
            .highlight_style(
                Style::default()
                    .fg(left_highlight)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(list_left, cols[0], &mut state_left);

        // Right: packages for hovered group
        let selected_group = if let Some(&gi) =
            app.popup_visible_indices.get(app.popup_selected_visible)
            && let Some(name) = app.popup_items.get(gi)
        {
            name.clone()
        } else {
            String::new()
        };
        let pkgs: Vec<String> = crate::app::AppState::group_packages_for(&selected_group)
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        if !pkgs.is_empty() && app.addpkgs_group_pkg_index >= pkgs.len() {
            app.addpkgs_group_pkg_index = pkgs.len() - 1;
        }
        let pkg_items: Vec<ListItem> = pkgs
            .iter()
            .map(|p| {
                let preselected = app.addpkgs_group_pkg_selected.contains(p)
                    || app
                        .additional_packages
                        .iter()
                        .any(|ap| ap.name.eq_ignore_ascii_case(p))
                    || app.check_additional_pkg_conflicts(p).is_some();
                let marker = if preselected { "[x]" } else { "[ ]" };
                let suffix = if app.check_additional_pkg_conflicts(p).is_some() {
                    " — Already selected in another section"
                } else {
                    ""
                };
                ListItem::new(format!("{} {}{}", marker, p, suffix))
            })
            .collect();
        let mut state_right = ratatui::widgets::ListState::default();
        if app.popup_packages_focus && !pkgs.is_empty() {
            state_right.select(Some(app.addpkgs_group_pkg_index));
        } else {
            state_right.select(None);
        }
        let right_title = if app.popup_packages_focus {
            " Packages in group (focused) "
        } else {
            " Packages in group "
        };
        let right_highlight = if app.popup_packages_focus {
            Color::Yellow
        } else {
            Color::White
        };
        // Add footer line with Continue/Confirm info
        let is_last_group =
            if let Some(&gi) = app.popup_visible_indices.get(app.popup_selected_visible) {
                gi + 1 == app.popup_items.len()
            } else {
                false
            };
        let footer_title = if is_last_group {
            " Confirm (Enter)  "
        } else {
            " Continue (Enter) "
        };

        let list_right = List::new(pkg_items)
            .block(Block::default().borders(Borders::ALL).title(right_title))
            .highlight_style(
                Style::default()
                    .fg(right_highlight)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(list_right, cols[1], &mut state_right);

        // Render footer hint
        let hint = Paragraph::new(Line::from(footer_title))
            .block(Block::default().borders(Borders::ALL).title(" Action "))
            .wrap(Wrap { trim: false });
        let rows = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Min(1),
                ratatui::layout::Constraint::Length(3),
            ])
            .split(cols[1]);
        frame.render_widget(hint, rows[1]);
        return;
    }

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.popup_selected_visible));
    let list_title = match app.popup_kind {
        Some(PopupKind::AbortConfirm) => " Save choices before exit? ",
        Some(PopupKind::AurHelperSelect) => " Choose AUR helper ",
        _ => " Options ",
    };
    let list = List::new(build_items(app))
        .block(Block::default().borders(Borders::ALL).title(list_title))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, area, &mut state);
}

fn build_items(app: &mut AppState) -> Vec<ListItem<'static>> {
    let is_multi = matches!(
        app.popup_kind,
        Some(PopupKind::MirrorsRegions)
            | Some(PopupKind::OptionalRepos)
            | Some(PopupKind::DesktopEnvSelect)
            | Some(PopupKind::XorgTypeSelect)
            | Some(PopupKind::KernelSelect)
            | Some(PopupKind::AdditionalPackageGroupPackages)
    );

    app.popup_visible_indices
        .iter()
        .map(|&i| {
            if is_multi {
                let checked = match app.popup_kind {
                    Some(PopupKind::MirrorsRegions) => app.mirrors_regions_selected.contains(&i),
                    Some(PopupKind::OptionalRepos) => app.optional_repos_selected.contains(&i),
                    Some(PopupKind::KernelSelect) => {
                        if let Some(name) = app.popup_items.get(i) {
                            app.selected_kernels.contains(name)
                        } else {
                            false
                        }
                    }
                    Some(PopupKind::AdditionalPackageGroupPackages) => {
                        if let Some(name) = app.popup_items.get(i) {
                            app.addpkgs_group_pkg_selected.contains(name)
                        } else {
                            false
                        }
                    }
                    _ => false,
                };
                let marker = if checked { "[x]" } else { "[ ]" };
                ListItem::new(format!("{} {}", marker, app.popup_items[i].clone()))
            } else {
                ListItem::new(app.popup_items[i].clone())
            }
        })
        .collect()
}
