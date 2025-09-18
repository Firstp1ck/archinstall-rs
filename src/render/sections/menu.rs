use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem};

use crate::app::{AppState, Focus};

pub fn draw_menu(frame: &mut Frame, app: &mut AppState, area: Rect) {
    // Ensure the entire menu area is cleared so the background is fully filled
    // even when the list has fewer items than the available height.
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = app
        .menu_entries
        .iter()
        .map(|entry| {
            let mut label = entry.label.clone();
            if app.processed_sections.contains(&entry.screen) {
                label = format!("✔ {}", label);
            }
            ListItem::new(Line::from(label))
        })
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

    frame.render_stateful_widget(menu, area, &mut app.list_state);
}
