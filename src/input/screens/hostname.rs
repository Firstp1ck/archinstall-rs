use crate::app::{AppState, Focus, Screen};

pub(crate) fn move_hostname_up(app: &mut AppState) {
    if app.current_screen() != Screen::Hostname || app.focus != Focus::Content {
        return;
    }
    if app.hostname_focus_index == 0 {
        app.hostname_focus_index = 1;
    } else {
        app.hostname_focus_index -= 1;
    }
}
pub(crate) fn move_hostname_down(app: &mut AppState) {
    if app.current_screen() != Screen::Hostname || app.focus != Focus::Content {
        return;
    }
    app.hostname_focus_index = (app.hostname_focus_index + 1) % 2;
}
pub(crate) fn change_hostname_value(_app: &mut AppState, _next: bool) {}

pub(crate) fn handle_enter_hostname(app: &mut AppState) {
    if app.hostname_focus_index == 0 {
        app.open_hostname_input();
    } else if app.hostname_focus_index == 1 {
        super::common::advance(app);
    }
}
