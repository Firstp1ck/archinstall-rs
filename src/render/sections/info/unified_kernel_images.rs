use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::AppState;

pub(super) fn render(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let uki = if app.uki_enabled {
        "Enabled"
    } else {
        "Disabled"
    };
    let info_lines = vec![
        Line::from(Span::styled(
            "Info",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("UKI: {uki}")),
        Line::from(format!("Secure Boot: {}", app.secure_boot_status_text())),
    ];

    let mut desc_lines = vec![Line::from(Span::styled(
        "Description",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];
    desc_lines.push(Line::from("A Unified Kernel Image (UKI) is a single UEFI-compatible file containing the Linux kernel, initramfs, boot stub, and extra resources bundled together. It can be booted directly by UEFI firmware or by a bootloader, simplifies configuration and signing for Secure Boot, and ensures all essential boot components are packaged, enabling secure and streamlined Linux startup."));
    if app.bootloader_index == 2 {
        let note = if app.is_secure_boot_enabled() {
            "EFISTUB selected: UKI is required because Secure Boot is enabled."
        } else {
            "EFISTUB selected: UKI is enabled by default and recommended for reliability."
        };
        desc_lines.push(Line::from(Span::styled(
            note,
            Style::default().fg(Color::Yellow),
        )));
    }

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
