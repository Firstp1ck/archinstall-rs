use crate::app::{AppState, PopupKind};

fn should_log_len(len: usize) -> bool {
    // Log for len 0..=2 and then every 5th character to reduce noise
    len <= 2 || len % 5 == 0
}

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
            | Some(PopupKind::ManualPartitionCreate)
            | Some(PopupKind::ManualPartitionMountpoint)
    ) {
        // In ManualPartitionCreate, only handle backspace when size field is focused
        if matches!(app.popup_kind, Some(PopupKind::ManualPartitionCreate))
            && app.manual_create_focus_units
        {
            return false;
        }
        app.custom_input_buffer.pop();
        let len = app.custom_input_buffer.len();
        if should_log_len(len) {
            app.debug_log(&format!(
                "popup:text backspace (len={})",
                len
            ));
        }
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
            | Some(PopupKind::ManualPartitionCreate)
            | Some(PopupKind::ManualPartitionMountpoint)
    ) {
        if matches!(app.popup_kind, Some(PopupKind::ManualPartitionCreate)) {
            // When units focused, don't consume j/k so nav can handle
            if app.manual_create_focus_units {
                return false;
            }
            // Size field: digits only, swallow other chars (including j/k)
            if c.is_ascii_digit() {
                app.custom_input_buffer.push(c);
                let len = app.custom_input_buffer.len();
                if should_log_len(len) {
                    app.debug_log(&format!(
                        "popup:text char digit (len={})",
                        len
                    ));
                }
            }
            return true;
        } else {
            app.custom_input_buffer.push(c);
            let len = app.custom_input_buffer.len();
            if should_log_len(len) {
                app.debug_log(&format!(
                    "popup:text char (len={})",
                    len
                ));
            }
            return true;
        }
    }
    false
}

pub(crate) fn handle_search_slash(app: &mut AppState) -> bool {
    app.popup_in_search = true;
    app.debug_log("popup: search enter");
    true
}

pub(crate) fn handle_search_backspace(app: &mut AppState) -> bool {
    if app.popup_in_search {
        app.popup_search_query.pop();
        app.filter_popup();
        app.debug_log(&format!(
            "popup: search backspace (len={})",
            app.popup_search_query.len()
        ));
        return true;
    }
    false
}

pub(crate) fn handle_search_char(app: &mut AppState, c: char) -> bool {
    if app.popup_in_search {
        app.popup_search_query.push(c);
        app.filter_popup();
        app.debug_log(&format!(
            "popup: search char (len={})",
            app.popup_search_query.len()
        ));
        return true;
    }
    false
}
