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

    // Special two-pane layout for AdditionalPackageGroupSelect to show packages side-by-side
    if matches!(app.popup_kind, Some(PopupKind::AdditionalPackageGroupSelect)) {
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
        let selected_group = if let Some(&gi) = app.popup_visible_indices.get(app.popup_selected_visible)
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
        let is_last_group = if let Some(&gi) = app.popup_visible_indices.get(app.popup_selected_visible) {
            gi + 1 == app.popup_items.len()
        } else {
            false
        };
        let footer_title = if is_last_group { " Confirm (Enter)  " } else { " Continue (Enter) " };

        let list_right = List::new(pkg_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(right_title),
            )
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
