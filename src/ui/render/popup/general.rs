use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::ui::app::{AppState, PopupKind};

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
