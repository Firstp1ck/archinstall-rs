use crate::app::{AppState, Focus, Screen};

pub(crate) fn move_uki_up(app: &mut AppState) {
    if app.current_screen() != Screen::UnifiedKernelImages || app.focus != Focus::Content {
        return;
    }
    if app.uki_focus_index == 0 {
        app.uki_focus_index = 1;
    } else {
        app.uki_focus_index -= 1;
    }
}
pub(crate) fn move_uki_down(app: &mut AppState) {
    if app.current_screen() != Screen::UnifiedKernelImages || app.focus != Focus::Content {
        return;
    }
    app.uki_focus_index = (app.uki_focus_index + 1) % 2;
}
pub(crate) fn change_uki_value(app: &mut AppState, _next: bool) {
    if app.current_screen() != Screen::UnifiedKernelImages || app.focus != Focus::Content {
        return;
    }
    if app.uki_focus_index == 0 {
        app.uki_enabled = !app.uki_enabled;
    }
}

pub(crate) fn handle_enter_uki(app: &mut AppState) {
    if app.uki_focus_index == 0 {
        app.uki_enabled = !app.uki_enabled;
    } else if app.uki_focus_index == 1 {
        super::common::advance(app);
    }
}
