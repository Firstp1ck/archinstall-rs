use crate::app::{AppState, PopupKind};

pub(crate) fn handle_left(app: &mut AppState) -> bool {
    if matches!(app.popup_kind, Some(PopupKind::ServerTypeSelect)) {
        if app.popup_packages_focus {
            app.popup_packages_focus = false;
        }
        return false;
    }
    if matches!(app.popup_kind, Some(PopupKind::XorgTypeSelect)) {
        if app.popup_drivers_focus {
            app.popup_drivers_focus = false;
            app.popup_packages_focus = true;
        } else if app.popup_packages_focus {
            app.popup_packages_focus = false;
        }
        return false;
    }
    if matches!(app.popup_kind, Some(PopupKind::DesktopEnvSelect)) {
        if app.popup_login_focus {
            app.popup_login_focus = false;
            app.popup_drivers_focus = true;
        } else if app.popup_drivers_focus {
            app.popup_drivers_focus = false;
            app.popup_packages_focus = true;
        } else if app.popup_packages_focus {
            app.popup_packages_focus = false;
        }
        return false;
    }
    false
}

pub(crate) fn handle_right(app: &mut AppState) -> bool {
    if matches!(app.popup_kind, Some(PopupKind::ServerTypeSelect)) {
        if !app.popup_packages_focus {
            app.popup_packages_focus = true;
            app.popup_packages_selected_index = 0;
        }
        return false;
    }
    if matches!(app.popup_kind, Some(PopupKind::XorgTypeSelect)) {
        if !app.popup_packages_focus && !app.popup_drivers_focus {
            app.popup_packages_focus = true;
            app.popup_packages_selected_index = 0;
        } else if app.popup_packages_focus {
            app.popup_packages_focus = false;
            app.popup_drivers_focus = true;
            app.popup_drivers_selected_index = 0;
        } else if app.popup_drivers_focus {
            // stay on drivers
        }
        return false;
    }
    if matches!(app.popup_kind, Some(PopupKind::DesktopEnvSelect)) {
        if !app.popup_packages_focus && !app.popup_drivers_focus && !app.popup_login_focus {
            app.popup_packages_focus = true;
            app.popup_packages_selected_index = 0;
        } else if app.popup_packages_focus {
            app.popup_packages_focus = false;
            app.popup_drivers_focus = true;
            app.popup_drivers_selected_index = 0;
        } else if app.popup_drivers_focus {
            app.popup_drivers_focus = false;
            app.popup_login_focus = true;
            app.popup_login_selected_index = 0;
        } else if app.popup_login_focus {
            // stay on login
        }
        return false;
    }
    false
}
