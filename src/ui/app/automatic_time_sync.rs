use super::{AppState, Focus};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

impl AppState {
    #[allow(dead_code)]
    pub fn init_automatic_time_sync(&mut self) {
        // placeholder
    }
}

pub fn draw_automatic_time_sync(frame: &mut ratatui::Frame, app: &mut AppState, area: Rect) {
    let title = Span::styled(
        "Automatic Time Sync",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let mut lines: Vec<Line> = vec![Line::from(title), Line::from("")];

    // Info text
    lines.push(Line::from(
        "Would you like to use automatic time synchronization (NTP) with default time servers?",
    ));
    lines.push(Line::from(
        "Hardware time and other post-configuration steps might be required in order for NTP to work.",
    ));
    lines.push(Line::from(""));

    // Yes / No options
    let options = vec![
        ("Yes", app.ats_focus_index == 0, app.ats_enabled),
        ("No", app.ats_focus_index == 1, !app.ats_enabled),
    ];
    for (label, is_focused_line, is_selected) in options {
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
        let value = if is_selected { "[x]" } else { "[ ]" };
        let value_style = if is_active_line {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let line = Line::from(vec![
            Span::styled(format!("{} ", bullet), bullet_style),
            Span::styled(format!("{} ", value), value_style),
            Span::styled(label.to_string(), label_style),
        ]);
        lines.push(line);
    }

    // Continue
    let continue_style = if app.ats_focus_index == 2 && matches!(app.focus, Focus::Content) {
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
