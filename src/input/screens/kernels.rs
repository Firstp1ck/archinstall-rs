use crate::ui::app::AppState;

pub(crate) fn move_kernels_up(app: &mut AppState) {
    if app.current_screen() != crate::ui::app::Screen::Kernels
        || app.focus != crate::ui::app::Focus::Content
    {
        return;
    }
    if app.kernels_focus_index == 0 {
        app.kernels_focus_index = 1;
    } else {
        app.kernels_focus_index -= 1;
    }
}

pub(crate) fn move_kernels_down(app: &mut AppState) {
    if app.current_screen() != crate::ui::app::Screen::Kernels
        || app.focus != crate::ui::app::Focus::Content
    {
        return;
    }
    app.kernels_focus_index = (app.kernels_focus_index + 1) % 2;
}

pub(crate) fn change_kernels_value(_app: &mut AppState, _next: bool) {}

pub(crate) fn handle_enter_kernels(app: &mut AppState) {
    match app.kernels_focus_index {
        0 => app.open_kernels_popup(),
        1 => super::common::advance(app),
        _ => {}
    }
}
