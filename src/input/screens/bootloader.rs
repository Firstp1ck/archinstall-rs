use crate::ui::app::{AppState, Focus, Screen};

pub(crate) fn move_bootloader_up(app: &mut AppState) {
    if app.current_screen() != Screen::Bootloader || app.focus != Focus::Content {
        return;
    }
    if app.bootloader_focus_index == 0 {
        app.bootloader_focus_index = 4;
    } else {
        app.bootloader_focus_index -= 1;
    }
}
pub(crate) fn move_bootloader_down(app: &mut AppState) {
    if app.current_screen() != Screen::Bootloader || app.focus != Focus::Content {
        return;
    }
    app.bootloader_focus_index = (app.bootloader_focus_index + 1) % 5;
}
pub(crate) fn change_bootloader_value(app: &mut AppState, _next: bool) {
    if app.current_screen() != Screen::Bootloader || app.focus != Focus::Content {
        return;
    }
    if app.bootloader_focus_index < 4 {
        app.bootloader_index = app.bootloader_focus_index;
    }
}

pub(crate) fn handle_enter_bootloader(app: &mut AppState) {
    if app.bootloader_focus_index < 4 {
        app.bootloader_index = app.bootloader_focus_index;
        app.update_unified_kernel_images_visibility();
    } else {
        super::common::advance(app);
    }
}
