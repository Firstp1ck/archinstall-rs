use crate::ui::app::{AppState, Focus, Screen};

pub(crate) fn move_swap_up(app: &mut AppState) {
    if app.current_screen() != Screen::SwapPartition || app.focus != Focus::Content {
        return;
    }
    if app.swap_focus_index == 0 {
        app.swap_focus_index = 1;
    } else {
        app.swap_focus_index -= 1;
    }
}
pub(crate) fn move_swap_down(app: &mut AppState) {
    if app.current_screen() != Screen::SwapPartition || app.focus != Focus::Content {
        return;
    }
    app.swap_focus_index = (app.swap_focus_index + 1) % 2;
}
pub(crate) fn change_swap_value(app: &mut AppState, _next: bool) {
    if app.current_screen() != Screen::SwapPartition || app.focus != Focus::Content {
        return;
    }
    if app.swap_focus_index == 0 {
        app.swap_enabled = !app.swap_enabled;
    }
}

pub(crate) fn handle_enter_swap(app: &mut AppState) {
    if app.swap_focus_index == 0 {
        app.swap_enabled = !app.swap_enabled;
    } else if app.swap_focus_index == 1 {
        super::common::advance(app);
    }
}
