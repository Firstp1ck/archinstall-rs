use super::{AppState, Focus};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

impl AppState {
    #[allow(dead_code)]
    pub fn init_user_account(&mut self) {}
}

pub fn draw_user_account(frame: &mut ratatui::Frame, app: &mut AppState, area: Rect) {
    let title = Span::styled(
        "User Account",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let mut lines: Vec<Line> = vec![Line::from(title), Line::from("")];

    // Options: Add user, Edit user, Delete user
    let options = vec![("Add a user", 0), ("Edit user", 1), ("Delete user", 2)];

    for (label, idx) in options {
        let is_focused_line = app.user_focus_index == idx;
        let is_active_line = is_focused_line && matches!(app.focus, Focus::Content);
        let bullet = if is_focused_line { "▶" } else { " " };
        let bullet_style = if is_active_line {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let label_style = if is_active_line {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let line = Line::from(vec![
            Span::styled(format!("{bullet} "), bullet_style),
            Span::styled(label.to_string(), label_style),
        ]);
        lines.push(line);
    }

    let continue_style = if app.user_focus_index == 3 && matches!(app.focus, Focus::Content) {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("[ Continue ]", continue_style)));

    // Users summary moved to the Info box

    let content = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(match app.focus {
                    Focus::Content => " Desicion Menu (focused) ",
                    _ => " Desicion Menu ",
                }),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(content, area);
}
