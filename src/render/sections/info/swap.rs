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

    let swap = if app.swap_enabled {
        "Enabled"
    } else {
        "Disabled"
    };
    info_lines.push(Line::from(format!("Swapon: {}", swap)));
    info_lines.push(Line::from(
        "Swapon can be used to activate the swap partition.",
    ));
    info_lines.push(Line::from(
        "If disabled here, the system will be configured with swapoff.",
    ));
    info_lines.push(Line::from(
        "You can always run 'swapon' later to activate the swap partition.",
    ));

    let mut desc_lines = vec![Line::from(Span::styled(
        "Description",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];
    desc_lines.push(Line::from("A swap partition in Linux acts as an extension of physical memory (RAM), using disk space to store inactive memory pages when RAM is full. It helps prevent system crashes under heavy load, enables hibernation by saving the RAM state to disk, and allows the operating system to run more applications than would otherwise fit in physical memory."));

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
