use crate::app::{AppState, Focus, Screen};

pub(crate) fn move_disks_up(app: &mut AppState) {
    if app.current_screen() != Screen::Disks || app.focus != Focus::Content {
        return;
    }
    if app.disks_focus_index == 0 {
        app.disks_focus_index = 3;
    } else {
        app.disks_focus_index -= 1;
    }
}
pub(crate) fn move_disks_down(app: &mut AppState) {
    if app.current_screen() != Screen::Disks || app.focus != Focus::Content {
        return;
    }
    app.disks_focus_index = (app.disks_focus_index + 1) % 4;
}
pub(crate) fn change_disks_value(app: &mut AppState, _next: bool) {
    if app.current_screen() != Screen::Disks || app.focus != Focus::Content {
        return;
    }
    // Selecting a partitioning mode should happen explicitly on Enter.
    // Left/Right on this screen do not implicitly change the selected mode.
}

pub(crate) fn handle_enter_disks(app: &mut AppState) {
    if app.disks_focus_index <= 2 {
        app.disks_mode_index = app.disks_focus_index;
        if app.disks_mode_index == 0 {
            // Best-effort selected: keep current bootloader selection; just open device list
            app.open_disks_device_list();
        } else if app.disks_mode_index == 1 {
            app.open_disks_device_list();
        }
    } else if app.disks_focus_index == 3 {
        super::common::advance(app);
    }
}
