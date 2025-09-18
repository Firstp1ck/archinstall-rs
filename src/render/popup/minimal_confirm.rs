use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::app::AppState;

pub fn draw(frame: &mut Frame, app: &mut AppState, popup_rect: Rect, title_text: &str) {
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
}
