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
    let pwd_set = if app.root_password.is_empty() {
        "(not set)"
    } else {
        "(set)"
    };
    let conf_set = if app.root_password_confirm.is_empty() {
        "(not set)"
    } else {
        "(set)"
    };
    info_lines.push(Line::from(format!("Password: {}", pwd_set)));
    info_lines.push(Line::from(format!("Confirm: {}", conf_set)));

    let mut desc_lines = vec![Line::from(Span::styled(
        "Description",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];
    desc_lines.push(Line::from("The root password is the secret phrase set for the root (superuser) account on Unix or Linux systems. It grants full administrative control, allowing changes to system files, configurations, and user management. Protecting the root password is crucial because anyone with it has unrestricted access and can impact system security and stability."));

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
