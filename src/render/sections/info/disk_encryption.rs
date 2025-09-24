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

    let enc_type = if app.disk_encryption_type_index == 1 {
        "LUKS"
    } else {
        "None"
    };
    info_lines.push(Line::from(format!("Type: {}", enc_type)));
    if app.disk_encryption_type_index == 1 {
        let pwd_set = if app.disk_encryption_password.is_empty() {
            "(not set)"
        } else {
            "(set)"
        };
        let pwd_conf = if app.disk_encryption_password_confirm.is_empty() {
            "(not set)"
        } else {
            "(set)"
        };
        info_lines.push(Line::from(format!("Password: {}", pwd_set)));
        info_lines.push(Line::from(format!("Confirm: {}", pwd_conf)));
        let part = app
            .disk_encryption_selected_partition
            .clone()
            .unwrap_or_else(|| "(none)".into());
        info_lines.push(Line::from(format!("Partition: {}", part)));
    }

    let mut desc_lines = vec![Line::from(Span::styled(
        "Description",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];
    desc_lines.push(Line::from("Disk encryption protects data by converting the contents of a drive into unreadable code, accessible only with a key or password. This ensures sensitive information remains secure even if the device is lost or stolen, safeguarding system and user data against unauthorized access."));

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
