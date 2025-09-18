use crate::ui::app::AppState;

pub(crate) fn move_user_up(app: &mut AppState) {
    if app.current_screen() != crate::ui::app::Screen::UserAccount
        || app.focus != crate::ui::app::Focus::Content
    {
        return;
    }
    if app.user_focus_index == 0 {
        app.user_focus_index = 3;
    } else {
        app.user_focus_index -= 1;
    }
}
pub(crate) fn move_user_down(app: &mut AppState) {
    if app.current_screen() != crate::ui::app::Screen::UserAccount
        || app.focus != crate::ui::app::Focus::Content
    {
        return;
    }
    app.user_focus_index = (app.user_focus_index + 1) % 4;
}
pub(crate) fn change_user_value(_app: &mut AppState, _next: bool) {}

pub(crate) fn handle_enter_user(app: &mut AppState) {
    match app.user_focus_index {
        0 => app.start_add_user_flow(),
        1 => {
            if app.users.is_empty() {
                app.open_info_popup("No users to edit".into());
            } else {
                app.popup_kind = Some(crate::ui::app::PopupKind::UserSelectEdit);
                app.popup_open = true;
                app.popup_items = app.users.iter().map(|u| u.username.clone()).collect();
                app.popup_visible_indices = (0..app.popup_items.len()).collect();
                app.popup_selected_visible = 0;
                app.popup_in_search = false;
                app.popup_search_query.clear();
            }
        }
        2 => {
            if app.users.is_empty() {
                app.open_info_popup("No users to delete".into());
            } else {
                app.popup_kind = Some(crate::ui::app::PopupKind::UserSelectDelete);
                app.popup_open = true;
                app.popup_items = app.users.iter().map(|u| u.username.clone()).collect();
                app.popup_visible_indices = (0..app.popup_items.len()).collect();
                app.popup_selected_visible = 0;
                app.popup_in_search = false;
                app.popup_search_query.clear();
            }
        }
        3 => super::common::advance(app),
        _ => {}
    }
}
