use crate::ui::app::{AppState, Focus, Screen};

pub(crate) fn move_timezone_up(app: &mut AppState) {
    if app.current_screen() != Screen::Timezone || app.focus != Focus::Content {
        return;
    }
    if app.timezone_focus_index == 0 {
        app.timezone_focus_index = 1;
    } else {
        app.timezone_focus_index -= 1;
    }
}
pub(crate) fn move_timezone_down(app: &mut AppState) {
    if app.current_screen() != Screen::Timezone || app.focus != Focus::Content {
        return;
    }
    app.timezone_focus_index = (app.timezone_focus_index + 1) % 2;
}
pub(crate) fn change_timezone_value(_app: &mut AppState, _next: bool) {}

pub(crate) fn handle_enter_timezone(app: &mut AppState) {
    if app.timezone_focus_index == 0 {
        app.open_timezone_popup();
    } else if app.timezone_focus_index == 1 {
        super::common::advance(app);
    }
}
