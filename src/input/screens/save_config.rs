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
        1 => match app.load_config() {
            Ok(()) => {
                let mut msg = String::from(
                    "Configuration loaded successfully.\n\nNote: Please re-enter the following sensitive fields:\n  - Disk encryption password\n  - Root password\n  - User passwords",
                );
                if !app.last_load_missing_sections.is_empty() {
                    msg.push_str("\n\nMissing sections (not found in file):\n");
                    for (i, section) in app.last_load_missing_sections.iter().enumerate() {
                        if i > 0 {
                            msg.push('\n');
                        }
                        msg.push_str(&format!("  - {section}"));
                    }
                }
                app.open_info_popup(msg);
            }
            Err(_e) => app.info_message = "Failed to load configuration".into(),
        },
        2 => super::common::advance(app),
        _ => {}
    }
}
