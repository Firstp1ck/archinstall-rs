use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{AppState, PopupKind};

pub fn draw(frame: &mut Frame, app: &mut AppState, popup_rect: Rect, title_text: &str) {
    frame.render_widget(ratatui::widgets::Clear, popup_rect);
    let popup_block = Block::default()
        .borders(Borders::ALL)
        .title(title_text)
        .border_style(Style::default().fg(Color::Yellow));
    let inner_area = popup_block.inner(popup_rect);
    frame.render_widget(popup_block, popup_rect);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner_area);

    let prompt_text = match app.popup_kind {
        Some(PopupKind::HostnameInput) => "Enter hostname:",
        Some(PopupKind::AdditionalPackageInput) => "Enter package name:",
        Some(PopupKind::RootPassword) => "Type root password:",
        Some(PopupKind::RootPasswordConfirm) => "Re-type root password:",
        Some(PopupKind::UserAddUsername) => "Enter username:",
        Some(PopupKind::UserEditUsername) => "Edit username:",
        Some(PopupKind::UserAddPassword) => "Type user password:",
        Some(PopupKind::UserAddPasswordConfirm) => "Re-type user password:",
        Some(PopupKind::DiskEncryptionPassword) => "Type encryption password:",
        Some(PopupKind::DiskEncryptionPasswordConfirm) => "Re-type encryption password:",
        Some(PopupKind::NetworkIP) => "Enter IPv4 optional), e.g., 192.168.1.1 or 192.168.1.1/24:",
        Some(PopupKind::NetworkGateway) => "Enter gateway (optional):",
        Some(PopupKind::NetworkDNS) => "Enter DNS (optional):",
        _ => "Enter value:",
    };
    let prompt = Paragraph::new(Line::from(prompt_text))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
    frame.render_widget(prompt, inner[0]);

    let masked = if matches!(
        app.popup_kind,
        Some(
            PopupKind::RootPassword
                | PopupKind::RootPasswordConfirm
                | PopupKind::UserAddPassword
                | PopupKind::UserAddPasswordConfirm
                | PopupKind::DiskEncryptionPassword
                | PopupKind::DiskEncryptionPasswordConfirm
        )
    ) {
        "*".repeat(app.custom_input_buffer.chars().count())
    } else {
        app.custom_input_buffer.clone()
    };
    let input_line = Paragraph::new(Line::from(vec![
        Span::styled(
            "> ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(masked),
    ]))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: false });
    frame.render_widget(input_line, inner[1]);

    let hint = Paragraph::new(Line::from("ESC to cancel"))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
    frame.render_widget(hint, inner[2]);

    if matches!(
        app.popup_kind,
        Some(PopupKind::NetworkIP | PopupKind::NetworkGateway | PopupKind::NetworkDNS)
    ) {
        let example_text = match app.popup_kind {
            Some(PopupKind::NetworkIP) => "E.g. 192.168.1.1/24",
            Some(PopupKind::NetworkGateway) => "E.g. 192.168.1.1",
            Some(PopupKind::NetworkDNS) => "E.g. 1.1.1.1",
            _ => "",
        };
        let example = Paragraph::new(Line::from(example_text))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        frame.render_widget(example, inner[3]);
    }
}
