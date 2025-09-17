use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

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
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: false });
    frame.render_widget(content, inner[0]);
}
