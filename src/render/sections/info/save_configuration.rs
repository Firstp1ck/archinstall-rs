use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::AppState;

pub(super) fn render(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let mut info_lines = vec![Line::from(Span::styled(
        "Info",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];

    if !app.last_load_missing_sections.is_empty() {
        info_lines.push(Line::from("Last load missing sections:"));
        for s in app.last_load_missing_sections.iter().take(6) {
            info_lines.push(Line::from(format!("- {s}")));
        }
        if app.last_load_missing_sections.len() > 6 {
            info_lines.push(Line::from("…"));
        }
    } else {
        info_lines.push(Line::from("No previous load warnings."));
    }

    let mut desc_lines = vec![Line::from(Span::styled(
        "Description",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];
    desc_lines.push(Line::from("Configuration lets you save or load your installation setup for Arch Linux, making it easy to reuse or share selections across multiple installs. When you load a configuration, sensitive data like passwords and certain custom settings need to be re-entered, ensuring security while streamlining system setup and consistency."));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    let description = Paragraph::new(desc_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Description "),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(description, chunks[0]);

    let info = Paragraph::new(info_lines)
        .block(Block::default().borders(Borders::ALL).title(" Info "))
        .wrap(Wrap { trim: true });
    frame.render_widget(info, chunks[1]);
}
