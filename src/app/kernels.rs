use super::{AppState, Focus};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

impl AppState {
    #[allow(dead_code)]
    pub fn init_kernels(&mut self) {
        // placeholder
    }

    pub fn kernels_summary(&self) -> String {
        if self.selected_kernels.is_empty() {
            "none".into()
        } else {
            let mut v: Vec<&str> = self.selected_kernels.iter().map(|s| s.as_str()).collect();
            v.sort_unstable();
            v.join(", ")
        }
    }
}

pub fn draw_kernels(frame: &mut ratatui::Frame, app: &mut AppState, area: Rect) {
    let title = Span::styled(
        "Kernels",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let mut lines: Vec<Line> = vec![Line::from(title), Line::from("")];

    let options = vec![
        (format!("Selected: {}", app.kernels_summary()), 0),
        ("Continue".to_string(), 1),
    ];

    for (label, idx) in options {
        let is_focused_line = app.kernels_focus_index == idx;
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
        let shown = if idx == 1 {
            format!("[ {} ]", label)
        } else {
            label
        };
        let line = Line::from(vec![
            Span::styled(format!("{} ", bullet), bullet_style),
            Span::styled(shown, label_style),
        ]);
        lines.push(line);
    }

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
