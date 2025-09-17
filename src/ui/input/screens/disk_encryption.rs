use crate::ui::app::{AppState, Focus, Screen};

pub(crate) fn move_diskenc_up(app: &mut AppState) {
    if app.current_screen() != Screen::DiskEncryption || app.focus != Focus::Content {
        return;
    }
    let max = if app.disk_encryption_type_index == 1 { 5 } else { 2 };
    if app.diskenc_focus_index == 0 {
        app.diskenc_focus_index = max - 1;
    } else {
        app.diskenc_focus_index -= 1;
    }
}
pub(crate) fn move_diskenc_down(app: &mut AppState) {
    if app.current_screen() != Screen::DiskEncryption || app.focus != Focus::Content {
        return;
    }
    let max = if app.disk_encryption_type_index == 1 { 5 } else { 2 };
    app.diskenc_focus_index = (app.diskenc_focus_index + 1) % max;
}
pub(crate) fn change_diskenc_value(app: &mut AppState, next: bool) {
    if app.current_screen() != Screen::DiskEncryption || app.focus != Focus::Content {
        return;
    }
    if app.diskenc_focus_index == 0 {
        let mut idx = app.disk_encryption_type_index;
        if next {
            idx = (idx + 1) % 2;
        } else {
            idx = (idx + 2 - 1) % 2;
        }
        app.disk_encryption_type_index = idx;
    }
}

pub(crate) fn handle_enter_diskenc(app: &mut AppState) {
    let continue_index = if app.disk_encryption_type_index == 1 { 4 } else { 1 };
    if app.diskenc_focus_index == 0 {
        app.open_disk_encryption_type_popup();
    } else if app.disk_encryption_type_index == 1 {
        match app.diskenc_focus_index {
            1 => app.open_disk_encryption_password_input(),
            2 => app.open_disk_encryption_password_confirm_input(),
            3 => app.open_disk_encryption_partition_list(),
            idx if idx == continue_index => super::common::advance(app),
            _ => {}
        }
    } else if app.diskenc_focus_index == continue_index {
        super::common::advance(app);
    }
}


