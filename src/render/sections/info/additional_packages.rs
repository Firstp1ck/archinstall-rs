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

    if app.additional_packages.is_empty() {
        info_lines.push(Line::from("Additional packages: none"));
    } else {
        let mut entries: Vec<(String, String)> = app
            .additional_packages
            .iter()
            .map(|p| (p.name.clone(), p.description.clone()))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        info_lines.push(Line::from(format!("Packages ({}):", entries.len())));
        for (name, desc) in entries.into_iter().take(8) {
            let max_line = (area.width.saturating_sub(4)) as usize;
            let mut line = format!("{name} — {desc}");
            if line.len() > max_line {
                line.truncate(max_line);
            }
            info_lines.push(Line::from(format!("  {line}")));
        }
        if app.additional_packages.len() > 8 {
            info_lines.push(Line::from("  …"));
        }
    }

    let mut desc_lines = vec![Line::from(Span::styled(
        "Description",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];
    desc_lines.push(Line::from("Additional packages let users customize their system by selecting individual software or groups during installation. You can add specific packages, like terminals or text editors, or choose from predefined groups for easier setup. This allows tailoring the installation with preferred tools and utilities beyond the default selection, supporting various use cases and workflows."));

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
