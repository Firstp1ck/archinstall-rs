use crate::ui::app::AppState;

pub(crate) fn move_config_up(app: &mut AppState) {
    if app.current_screen() != crate::ui::app::Screen::SaveConfiguration
        || app.focus != crate::ui::app::Focus::Content
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
    if app.current_screen() != crate::ui::app::Screen::SaveConfiguration
        || app.focus != crate::ui::app::Focus::Content
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
        1 => match app.load_config() {
            Ok(()) => {
                let mut msg = String::from(
                    "Configuration loaded. Note: re-enter Disk encryption, Root, and User passwords.",
                );
                if !app.last_load_missing_sections.is_empty() {
                    msg.push_str(" Missing: ");
                    msg.push_str(&app.last_load_missing_sections.join(", "));
                }
                app.open_info_popup(msg);
            }
            Err(()) => app.info_message = "Failed to load configuration".into(),
        },
        2 => super::common::advance(app),
        _ => {}
    }
}
