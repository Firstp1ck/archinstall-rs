use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{AppState, NetworkConfigMode};

pub(super) fn render(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let mut info_lines = vec![Line::from(Span::styled(
        "Info",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];

    info_lines.push(Line::from(format!(
        "Network: {}",
        app.current_network_label()
    )));
    if !app.network_configs.is_empty() {
        info_lines.push(Line::from("Interfaces:"));
        for cfg in &app.network_configs {
            let mode = match cfg.mode {
                NetworkConfigMode::Dhcp => "DHCP",
                NetworkConfigMode::Static => "Static",
            };
            let mut line = format!("- {} ({})", cfg.interface, mode);
            if let Some(ip) = &cfg.ip_cidr {
                line.push_str(&format!(" IP={} ", ip));
            }
            if let Some(gw) = &cfg.gateway {
                line.push_str(&format!(" GW={} ", gw));
            }
            if let Some(dns) = &cfg.dns {
                line.push_str(&format!(" DNS={} ", dns));
            }
            info_lines.push(Line::from(line));
        }
    }

    let mut desc_lines = vec![Line::from(Span::styled(
        "Description",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];
    desc_lines.push(Line::from("Network configuration in Arch Linux sets up wired or wireless connections, assigns IP addresses (dynamic by DHCP or manually), and manages DNS. Tools like systemd-networkd, NetworkManager, or iproute2 can be used, but only one network manager should be active at a time to avoid conflicts."));

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
