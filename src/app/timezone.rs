use super::AppState;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::process::Command;

pub fn draw_timezone(frame: &mut ratatui::Frame, app: &mut AppState, area: Rect) {
    let title = Span::styled(
        "Timezone",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let mut lines: Vec<Line> = vec![Line::from(title), Line::from("")];

    // Field 0: Select Timezone
    let is_focus_0 = app.timezone_focus_index == 0;
    let is_active_0 = is_focus_0 && matches!(app.focus, super::Focus::Content);
    let bullet_style_0 = if is_active_0 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let label_style_0 = if is_active_0 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    lines.push(Line::from(vec![
        Span::styled(
            format!("{} ", if is_focus_0 { "▶" } else { " " }),
            bullet_style_0,
        ),
        Span::styled("Select Timezone".to_string(), label_style_0),
    ]));

    // Continue
    let is_focus_1 = app.timezone_focus_index == 1 && matches!(app.focus, super::Focus::Content);
    let continue_style = if is_focus_1 {
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
                    super::Focus::Content => " Desicion Menu (focused) ",
                    _ => " Desicion Menu ",
                }),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(content, area);
}

impl AppState {
    pub fn open_timezone_popup(&mut self) {
        self.popup_kind = Some(super::PopupKind::TimezoneSelect);
        self.popup_open = true;
        self.popup_items = Self::list_timezones();
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    fn list_timezones() -> Vec<String> {
        if let Ok(output) = Command::new("timedatectl").arg("list-timezones").output()
            && output.status.success()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            return text
                .lines()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
        }
        Vec::new()
    }
}
