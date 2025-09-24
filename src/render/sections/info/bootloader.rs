use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::AppState;

pub(super) fn render(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let bl = match app.bootloader_index {
        0 => "Systemd-boot",
        1 => "Grub",
        2 => "Efistub",
        _ => "Limine",
    };
    let info_lines = vec![
        Line::from(Span::styled(
            "Info",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("Bootloader: {}", bl)),
    ];

    let mut desc_lines = vec![Line::from(Span::styled(
        "Description",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];
    desc_lines.push(Line::from("A bootloader is a program that starts at system boot, loading the Operating System kernel and initializing hardware. Systemd-boot is simple, UEFI-only, and uses separate text files for each boot entry, making it easy to maintain, but offers minimal features and customization. GRUB2 is more complex, working on both BIOS and UEFI, supporting advanced features, graphical menus, multi-OS setups, and custom scripts, making it ideal for diverse or complex boot needs."));

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
