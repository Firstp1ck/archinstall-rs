use crate::app::{AppState, Focus, Screen};

pub(crate) fn move_rootpass_up(app: &mut AppState) {
    if app.current_screen() != Screen::RootPassword || app.focus != Focus::Content {
        return;
    }
    if app.rootpass_focus_index == 0 {
        app.rootpass_focus_index = 2;
    } else {
        app.rootpass_focus_index -= 1;
    }
}
pub(crate) fn move_rootpass_down(app: &mut AppState) {
    if app.current_screen() != Screen::RootPassword || app.focus != Focus::Content {
        return;
    }
    app.rootpass_focus_index = (app.rootpass_focus_index + 1) % 3;
}
pub(crate) fn change_rootpass_value(_app: &mut AppState, _next: bool) {}

pub(crate) fn handle_enter_rootpass(app: &mut AppState) {
    match app.rootpass_focus_index {
        0 => app.open_root_password_input(),
        1 => app.open_root_password_confirm_input(),
        2 => super::common::advance(app),
        _ => {}
    }
}
