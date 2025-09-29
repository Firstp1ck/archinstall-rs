use super::{AppState, Focus};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

impl AppState {
    #[allow(dead_code)]
    pub fn init_bootloader(&mut self) {
        // nothing to init for now
    }

    #[allow(dead_code)]
    pub fn current_bootloader_label(&self) -> &'static str {
        match self.bootloader_index {
            0 => "Systemd-boot",
            1 => "Grub",
            2 => "Efistub (not implemented yet)",
            _ => "Limine (not implemented yet)",
        }
    }
}

pub fn draw_bootloader(frame: &mut ratatui::Frame, app: &mut AppState, area: Rect) {
    let title = Span::styled(
        "Bootloader",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let mut lines: Vec<Line> = vec![Line::from(title), Line::from("")];

    let choices = ["Systemd-boot", "Grub", "Efistub (not implemented yet)", "Limine (not implemented yet)"];

    for (idx, label) in choices.iter().enumerate() {
        let is_focused_line = app.bootloader_focus_index == idx;
        let is_active_line = is_focused_line && matches!(app.focus, Focus::Content);
        let is_selected = app.bootloader_index == idx;
        let bullet = if is_focused_line { "▶" } else { " " };
        let marker = if is_selected { "[x]" } else { "[ ]" };
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
            Span::styled(format!("{marker} {label}"), label_style),
        ]);
        lines.push(line);
    }

    let continue_style = if app.bootloader_focus_index == 4 && matches!(app.focus, Focus::Content) {
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
