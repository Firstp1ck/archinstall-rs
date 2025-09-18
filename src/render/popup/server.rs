use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem};

use crate::app::AppState;

pub fn draw(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

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
    if !selected_label.is_empty() {
        let set = app
            .selected_server_packages
            .entry(selected_label.clone())
            .or_insert_with(|| pkgs.iter().map(|s| s.to_string()).collect());
        let pkg_items: Vec<ListItem> = pkgs
            .iter()
            .map(|name| {
                let marker = if set.contains(*name) { "[x]" } else { "[ ]" };
                ListItem::new(format!("{} {}", marker, name))
            })
            .collect();
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
        frame.render_stateful_widget(pkg_list, cols[1], &mut pkg_state);
    }
}
