use crate::ui::app::AppState;

pub(crate) fn move_network_up(app: &mut AppState) {
    if app.current_screen() != crate::ui::app::Screen::NetworkConfiguration
        || app.focus != crate::ui::app::Focus::Content
    {
        return;
    }
    if app.network_focus_index == 0 {
        app.network_focus_index = 4;
    } else {
        app.network_focus_index -= 1;
    }
}

pub(crate) fn move_network_down(app: &mut AppState) {
    if app.current_screen() != crate::ui::app::Screen::NetworkConfiguration
        || app.focus != crate::ui::app::Focus::Content
    {
        return;
    }
    app.network_focus_index = (app.network_focus_index + 1) % 5;
}

pub(crate) fn change_network_value(app: &mut AppState, _next: bool) {
    if app.current_screen() != crate::ui::app::Screen::NetworkConfiguration
        || app.focus != crate::ui::app::Focus::Content
    {
        return;
    }
    if app.network_focus_index < 3 {
        let previous_mode = app.network_mode_index;
        app.network_mode_index = app.network_focus_index;
        if previous_mode == 1 && app.network_mode_index != 1 {
            app.network_configs.clear();
        }
    }
}

pub(crate) fn handle_enter_network(app: &mut AppState) {
    if app.network_focus_index < 3 {
        let previous_mode = app.network_mode_index;
        app.network_mode_index = app.network_focus_index;
        if previous_mode == 1 && app.network_mode_index != 1 {
            app.network_configs.clear();
        }
    } else if app.network_focus_index == 3 {
        if app.network_mode_index != 1 {
            app.open_info_popup("Switch to Manual configuration to add interfaces".into());
            return;
        }
        app.open_network_interfaces_popup();
    } else {
        super::common::advance(app);
    }
}


