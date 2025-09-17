use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::ui::app::{AppState, PopupKind};

mod desktop;
mod general;
mod info;
mod minimal_confirm;
mod server;
mod sudo_select;
mod text_input;
mod xorg;

pub fn draw_popup(frame: &mut Frame, app: &mut AppState) {
    let area = frame.area();
    let (width, height) = if matches!(
        app.popup_kind,
        Some(PopupKind::AbortConfirm)
            | Some(PopupKind::Info)
            | Some(PopupKind::HostnameInput)
            | Some(PopupKind::RootPassword)
            | Some(PopupKind::RootPasswordConfirm)
            | Some(PopupKind::UserAddUsername)
            | Some(PopupKind::UserEditUsername)
            | Some(PopupKind::UserAddPassword)
            | Some(PopupKind::UserAddPasswordConfirm)
            | Some(PopupKind::AdditionalPackageInput)
            | Some(PopupKind::UserAddSudo)
            | Some(PopupKind::MinimalClearConfirm)
            | Some(PopupKind::DiskEncryptionPassword)
            | Some(PopupKind::DiskEncryptionPasswordConfirm)
            | Some(PopupKind::NetworkIP)
            | Some(PopupKind::NetworkGateway)
            | Some(PopupKind::NetworkDNS)
    ) {
        let w = area.width.clamp(20, 34);
        let h = area.height.clamp(7, 9);
        (w, h)
    } else {
        (
            area.width.saturating_mul(2) / 3,
            area.height.saturating_mul(2) / 3,
        )
    };
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    let popup_rect = Rect {
        x,
        y,
        width,
        height,
    };

    let title_text = match app.popup_kind {
        Some(PopupKind::KeyboardLayout) => " Select Keyboard layout ",
        Some(PopupKind::LocaleLanguage) => " Select Locale language ",
        Some(PopupKind::LocaleEncoding) => " Select Locale encoding ",
        Some(PopupKind::MirrorsRegions) => " Select Regions (space to toggle) ",
        Some(PopupKind::MirrorsCustomServerInput) => " Add custom server URL (Enter to add) ",
        Some(PopupKind::MirrorsCustomRepoName) => " Custom repo: name ",
        Some(PopupKind::MirrorsCustomRepoUrl) => " Custom repo: URL ",
        Some(PopupKind::MirrorsCustomRepoSig) => " Custom repo: signature (Enter) ",
        Some(PopupKind::MirrorsCustomRepoSignOpt) => " Custom repo: sign option (Enter) ",
        Some(PopupKind::OptionalRepos) => " Optional repositories (space to toggle) ",
        Some(PopupKind::HostnameInput) => " Enter Hostname ",
        Some(PopupKind::AdditionalPackageInput) => " Add package (Enter to add) ",
        Some(PopupKind::RootPassword) => " Enter Root Password ",
        Some(PopupKind::RootPasswordConfirm) => " Confirm Root Password ",
        Some(PopupKind::NetworkInterfaces) => " Select interface ",
        Some(PopupKind::NetworkMode) => " Select mode ",
        Some(PopupKind::NetworkIP) => " Enter IP address/CIDR ",
        Some(PopupKind::NetworkGateway) => " Enter Gateway/Router (optional) ",
        Some(PopupKind::NetworkDNS) => " Enter DNS (optional, e.g., 1.1.1.1) ",
        Some(PopupKind::UserAddUsername) => " Enter Username ",
        Some(PopupKind::UserAddPassword) => " Enter User Password ",
        Some(PopupKind::UserAddPasswordConfirm) => " Confirm User Password ",
        Some(PopupKind::UserAddSudo) => " Should User be Superuser (sudo)? ",
        Some(PopupKind::MinimalClearConfirm) => " Clear selections for Minimal? ",
        Some(PopupKind::ServerTypeSelect) => " Select Server Type (space to toggle) ",
        Some(PopupKind::XorgTypeSelect) => " Select Xorg (space to toggle)",
        Some(PopupKind::DisksDeviceList) => " Available Drives ",
        Some(PopupKind::DiskEncryptionType) => " Encryption Type ",
        Some(PopupKind::DiskEncryptionPassword) => " Enter encryption password ",
        Some(PopupKind::DiskEncryptionPasswordConfirm) => " Confirm encryption password ",
        Some(PopupKind::DiskEncryptionPartitionList) => " Select Partition to encrypt ",
        Some(PopupKind::AbortConfirm) => " Confirm Abort (Exit) ",
        Some(PopupKind::WipeConfirm) => " Confirm Disk Wipe ",
        Some(PopupKind::DesktopEnvSelect) => {
            " Select Desktop/WM (space to toggle, Enter to close) "
        }
        Some(PopupKind::Info) => " Info ",
        Some(PopupKind::KernelSelect) => " Select Kernels (space to toggle) ",
        Some(PopupKind::UserSelectEdit) => " Select user to edit ",
        Some(PopupKind::UserSelectDelete) => " Select user to delete ",
        Some(PopupKind::UserEditUsername) => " Edit username ",
        Some(PopupKind::TimezoneSelect) => " Select Timezone ",
        None => " Select ",
    };

    // Specialized popups that do not use the common search chrome
    if matches!(app.popup_kind, Some(PopupKind::Info)) {
        info::draw(frame, app, popup_rect, title_text);
        return;
    }

    if matches!(
        app.popup_kind,
        Some(
            PopupKind::HostnameInput
                | PopupKind::RootPassword
                | PopupKind::RootPasswordConfirm
                | PopupKind::UserAddUsername
                | PopupKind::UserEditUsername
                | PopupKind::UserAddPassword
                | PopupKind::UserAddPasswordConfirm
                | PopupKind::DiskEncryptionPassword
                | PopupKind::DiskEncryptionPasswordConfirm
                | PopupKind::AdditionalPackageInput
                | PopupKind::NetworkIP
                | PopupKind::NetworkGateway
                | PopupKind::NetworkDNS
        )
    ) {
        text_input::draw(frame, app, popup_rect, title_text);
        return;
    }

    if matches!(app.popup_kind, Some(PopupKind::UserAddSudo)) {
        sudo_select::draw(frame, app, popup_rect, title_text);
        return;
    }

    if matches!(app.popup_kind, Some(PopupKind::MinimalClearConfirm)) {
        minimal_confirm::draw(frame, app, popup_rect, title_text);
        return;
    }

    // Common chrome for searchable, list-based popups
    frame.render_widget(ratatui::widgets::Clear, popup_rect);
    let popup_block = Block::default()
        .borders(Borders::ALL)
        .title(title_text)
        .border_style(Style::default().fg(Color::Yellow));
    frame.render_widget(popup_block, popup_rect);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(popup_rect);

    let search_label = if app.popup_in_search { "/" } else { "" };
    let search = Paragraph::new(Line::from(vec![
        ratatui::text::Span::styled(
            search_label,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        ratatui::text::Span::raw(app.popup_search_query.clone()),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Search "))
    .wrap(Wrap { trim: false });
    frame.render_widget(search, inner[0]);

    match app.popup_kind {
        Some(PopupKind::DesktopEnvSelect) => desktop::draw(frame, app, inner[1]),
        Some(PopupKind::ServerTypeSelect) => server::draw(frame, app, inner[1]),
        Some(PopupKind::XorgTypeSelect) => xorg::draw(frame, app, inner[1]),
        _ => general::draw(frame, app, inner[1]),
    }
}
