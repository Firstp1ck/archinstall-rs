use crate::app::AppState;

pub(crate) fn move_config_up(app: &mut AppState) {
    if app.current_screen() != crate::app::Screen::SaveConfiguration
        || app.focus != crate::app::Focus::Content
    {
        return;
    }
    if app.config_focus_index == 0 {
        app.config_focus_index = 2;
    } else {
        app.config_focus_index -= 1;
    }
}
pub(crate) fn move_config_down(app: &mut AppState) {
    if app.current_screen() != crate::app::Screen::SaveConfiguration
        || app.focus != crate::app::Focus::Content
    {
        return;
    }
    app.config_focus_index = (app.config_focus_index + 1) % 3;
}
pub(crate) fn change_config_value(_app: &mut AppState, _next: bool) {}

pub(crate) fn handle_enter_config(app: &mut AppState) {
    match app.config_focus_index {
        0 => {
            app.save_config();
            app.open_info_popup("Configuration saved.".into());
        }
        1 => {
            app.open_config_load_popup();
        }
        2 => super::common::advance(app),
        _ => {}
    }
}
