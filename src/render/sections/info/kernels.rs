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

    if app.selected_kernels.is_empty() {
        info_lines.push(Line::from("Kernels: none"));
    } else {
        let mut v: Vec<&str> = app.selected_kernels.iter().map(|s| s.as_str()).collect();
        v.sort_unstable();
        info_lines.push(Line::from(format!("Kernels: {}", v.join(", "))));
    }

    let mut desc_lines = vec![Line::from(Span::styled(
        "Description",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];
    desc_lines.push(Line::from("Kernels are the core component of Arch Linux, responsible for managing hardware, system resources, and communication between software and hardware. Arch Linux provides several kernel options, including the latest stable, LTS (Long Term Support), and specialized kernels like zen or hardened, each offering different features and performance characteristics. Users can easily install, switch, or maintain multiple kernels via the package manager. Recommended are at least two Kernels to install."));

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
