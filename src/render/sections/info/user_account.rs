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

    if app.users.is_empty() {
        info_lines.push(Line::from("No users added yet."));
    } else {
        info_lines.push(Line::from("Users:"));
        for u in app.users.iter().take(5) {
            let sudo = if u.is_sudo { "sudo" } else { "user" };
            info_lines.push(Line::from(format!("- {} ({})", u.username, sudo)));
        }
        if app.users.len() > 5 {
            info_lines.push(Line::from("…"));
        }
    }

    let mut desc_lines = vec![Line::from(Span::styled(
        "Description",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];
    desc_lines.push(Line::from("A user account in Arch Linux is an identity used to log in and interact with the system, providing access to files, directories, and system resources based on permissions. Each user has a unique username, a home directory, and configurable group memberships for access control. User information is stored in /etc/passwd, while passwords are managed securely in /etc/shadow."));

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
