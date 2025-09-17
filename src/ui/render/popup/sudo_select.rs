use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem};

use crate::ui::app::AppState;

pub fn draw(frame: &mut Frame, app: &mut AppState, popup_rect: Rect, title_text: &str) {
    frame.render_widget(ratatui::widgets::Clear, popup_rect);
    let popup_block = Block::default()
        .borders(Borders::ALL)
        .title(title_text)
        .border_style(Style::default().fg(Color::Yellow));
    frame.render_widget(popup_block, popup_rect);

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
}
