use crate::ui::app::{AppState, PopupKind};

pub(crate) fn handle_text_backspace(app: &mut AppState) -> bool {
    if matches!(
        app.popup_kind,
        Some(PopupKind::MirrorsCustomServerInput)
            | Some(PopupKind::MirrorsCustomRepoName)
            | Some(PopupKind::MirrorsCustomRepoUrl)
            | Some(PopupKind::NetworkIP)
            | Some(PopupKind::NetworkGateway)
            | Some(PopupKind::NetworkDNS)
            | Some(PopupKind::DiskEncryptionPassword)
            | Some(PopupKind::DiskEncryptionPasswordConfirm)
            | Some(PopupKind::HostnameInput)
            | Some(PopupKind::AdditionalPackageInput)
            | Some(PopupKind::RootPassword)
            | Some(PopupKind::RootPasswordConfirm)
            | Some(PopupKind::UserAddUsername)
            | Some(PopupKind::UserEditUsername)
            | Some(PopupKind::UserAddPassword)
            | Some(PopupKind::UserAddPasswordConfirm)
    ) {
        app.custom_input_buffer.pop();
        return true;
    }
    false
}

pub(crate) fn handle_text_char(app: &mut AppState, c: char) -> bool {
    if matches!(
        app.popup_kind,
        Some(PopupKind::MirrorsCustomServerInput)
            | Some(PopupKind::MirrorsCustomRepoName)
            | Some(PopupKind::MirrorsCustomRepoUrl)
            | Some(PopupKind::NetworkIP)
            | Some(PopupKind::NetworkGateway)
            | Some(PopupKind::NetworkDNS)
            | Some(PopupKind::DiskEncryptionPassword)
            | Some(PopupKind::DiskEncryptionPasswordConfirm)
            | Some(PopupKind::HostnameInput)
            | Some(PopupKind::AdditionalPackageInput)
            | Some(PopupKind::RootPassword)
            | Some(PopupKind::RootPasswordConfirm)
            | Some(PopupKind::UserAddUsername)
            | Some(PopupKind::UserEditUsername)
            | Some(PopupKind::UserAddPassword)
            | Some(PopupKind::UserAddPasswordConfirm)
    ) {
        app.custom_input_buffer.push(c);
        return true;
    }
    false
}

pub(crate) fn handle_search_slash(app: &mut AppState) -> bool {
    app.popup_in_search = true;
    true
}

pub(crate) fn handle_search_backspace(app: &mut AppState) -> bool {
    if app.popup_in_search {
        app.popup_search_query.pop();
        app.filter_popup();
        return true;
    }
    false
}

pub(crate) fn handle_search_char(app: &mut AppState, c: char) -> bool {
    if app.popup_in_search {
        app.popup_search_query.push(c);
        app.filter_popup();
        return true;
    }
    false
}
