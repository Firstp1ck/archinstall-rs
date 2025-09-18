use crate::ui::app::{AppState, Focus, Screen};

pub(crate) fn move_ats_up(app: &mut AppState) {
    if app.current_screen() != Screen::AutomaticTimeSync || app.focus != Focus::Content {
        return;
    }
    if app.ats_focus_index == 0 {
        app.ats_focus_index = 2;
    } else {
        app.ats_focus_index -= 1;
    }
}
pub(crate) fn move_ats_down(app: &mut AppState) {
    if app.current_screen() != Screen::AutomaticTimeSync || app.focus != Focus::Content {
        return;
    }
    app.ats_focus_index = (app.ats_focus_index + 1) % 3;
}
pub(crate) fn change_ats_value(app: &mut AppState, next: bool) {
    if app.current_screen() != Screen::AutomaticTimeSync || app.focus != Focus::Content {
        return;
    }
    if app.ats_focus_index <= 1 {
        if next {
            app.ats_focus_index = (app.ats_focus_index + 1) % 2;
        } else {
            app.ats_focus_index = (app.ats_focus_index + 2 - 1) % 2;
        }
        app.ats_enabled = app.ats_focus_index == 0;
    }
}

pub(crate) fn handle_enter_ats(app: &mut AppState) {
    match app.ats_focus_index {
        0 => app.ats_enabled = true,
        1 => app.ats_enabled = false,
        2 => super::common::advance(app),
        _ => {}
    }
}
