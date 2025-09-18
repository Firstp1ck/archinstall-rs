use super::{AppState, Focus};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

impl AppState {
    #[allow(dead_code)]
    pub fn init_root_password(&mut self) {
        // placeholder
    }
}

pub fn draw_root_password(frame: &mut ratatui::Frame, app: &mut AppState, area: Rect) {
    let title = Span::styled(
        "Root Password",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let mut lines: Vec<Line> = vec![Line::from(title), Line::from("")];

    // Set password
    let is_focused_line = app.rootpass_focus_index == 0;
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
    let value_mask = "*".repeat(app.root_password.chars().count());
    lines.push(Line::from(vec![
        Span::styled(format!("{} ", bullet), bullet_style),
        Span::styled("Set password: ", label_style),
        Span::styled(value_mask, Style::default().fg(Color::White)),
    ]));

    // Confirm password
    let is_focused_line = app.rootpass_focus_index == 1;
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
    let value_mask = "*".repeat(app.root_password_confirm.chars().count());
    lines.push(Line::from(vec![
        Span::styled(format!("{} ", bullet), bullet_style),
        Span::styled("Confirm password: ", label_style),
        Span::styled(value_mask, Style::default().fg(Color::White)),
    ]));

    // Continue
    let continue_style = if app.rootpass_focus_index == 2 && matches!(app.focus, Focus::Content) {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("[ Continue ]", continue_style)));

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
