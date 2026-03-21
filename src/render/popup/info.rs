use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::AppState;

pub fn draw(frame: &mut Frame, app: &mut AppState, popup_rect: Rect, title_text: &str) {
    let t = crate::render::theme::catppuccin_mocha();

    frame.render_widget(ratatui::widgets::Clear, popup_rect);

    let popup_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            title_text,
            Style::default()
                .fg(t.highlight)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(t.accent));
    frame.render_widget(popup_block, popup_rect);

    let inner = popup_rect.inner(Margin {
        vertical: 1,
        horizontal: 2,
    });

    let body = if app.popup_items.is_empty() {
        String::new()
    } else {
        app.popup_items[0].clone()
    };

    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(inner);

    let mut lines: Vec<Line> = Vec::new();
    for raw_line in body.split('\n') {
        if raw_line.is_empty() {
            lines.push(Line::from(""));
            continue;
        }

        let style = if raw_line.starts_with("Note:") {
            Style::default().fg(t.warning)
        } else if raw_line.starts_with("Missing") {
            Style::default()
                .fg(t.warning)
                .add_modifier(Modifier::BOLD)
        } else if raw_line.starts_with("  - ") {
            Style::default().fg(t.subtext)
        } else {
            Style::default().fg(t.text)
        };

        lines.push(Line::from(Span::styled(raw_line.to_string(), style)));
    }

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(content, body_chunks[0]);

    let hint = Line::from(vec![
        Span::styled("Press ", Style::default().fg(t.subtext)),
        Span::styled(
            "Enter",
            Style::default()
                .fg(t.highlight)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" or ", Style::default().fg(t.subtext)),
        Span::styled(
            "ESC",
            Style::default()
                .fg(t.highlight)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to close", Style::default().fg(t.subtext)),
    ]);
    let hint_para = Paragraph::new(vec![Line::from(""), hint]);
    frame.render_widget(hint_para, body_chunks[1]);
}
