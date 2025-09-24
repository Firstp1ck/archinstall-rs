use crate::app::AppState;

pub(crate) fn move_experience_up(app: &mut AppState) {
    if app.current_screen() != crate::app::Screen::ExperienceMode
        || app.focus != crate::app::Focus::Content
    {
        return;
    }
    if app.experience_focus_index == 0 {
        app.experience_focus_index = 4;
    } else {
        app.experience_focus_index -= 1;
    }
}

pub(crate) fn move_experience_down(app: &mut AppState) {
    if app.current_screen() != crate::app::Screen::ExperienceMode
        || app.focus != crate::app::Focus::Content
    {
        return;
    }
    app.experience_focus_index = (app.experience_focus_index + 1) % 5;
}

pub(crate) fn change_experience_value(app: &mut AppState, _next: bool) {
    if app.current_screen() != crate::app::Screen::ExperienceMode
        || app.focus != crate::app::Focus::Content
    {
        return;
    }
    if app.experience_focus_index <= 3 {
        app.experience_mode_index = app.experience_focus_index;
    }
}

pub(crate) fn handle_enter_experience(app: &mut AppState) {
    if app.experience_focus_index == 0 {
        app.experience_mode_index = 0;
        // If graphics drivers were cleared (e.g., after Minimal), seed defaults for Desktop
        if app.selected_graphic_drivers.is_empty() {
            for k in [
                "intel-media-driver",
                "libva-intel-driver",
                "mesa",
                "vulkan-intel",
                "vulkan-nouveau",
                "vulkan-radeon",
                "xf86-video-amdgpu",
                "xf86-video-ati",
                "xf86-video-nouveau",
                "xorg-server",
                "xorg-xinit",
            ] {
                app.selected_graphic_drivers.insert(k.into());
            }
        }
        app.open_desktop_environment_popup();
    } else if app.experience_focus_index <= 3 {
        if app.experience_focus_index == 1 {
            if !app.selected_desktop_envs.is_empty()
                || !app.selected_env_packages.is_empty()
                || app.selected_login_manager.is_some()
                || app.login_manager_user_set
            {
                app.open_minimal_clear_confirm();
            } else {
                app.experience_mode_index = 1;
                app.selected_desktop_envs.clear();
                app.selected_env_packages.clear();
                app.selected_login_manager = None;
                app.login_manager_user_set = false;
                // Also clear any previously selected graphics drivers for Minimal
                app.selected_graphic_drivers.clear();
            }
        } else if app.experience_focus_index == 2 {
            app.experience_mode_index = 2;
            app.open_server_type_popup();
        } else if app.experience_focus_index == 3 {
            app.experience_mode_index = 3;
            // If graphics drivers were cleared (e.g., after Minimal), seed defaults for Xorg
            if app.selected_graphic_drivers.is_empty() {
                for k in [
                    "intel-media-driver",
                    "libva-intel-driver",
                    "mesa",
                    "vulkan-intel",
                    "vulkan-nouveau",
                    "vulkan-radeon",
                    "xf86-video-amdgpu",
                    "xf86-video-ati",
                    "xf86-video-nouveau",
                    "xorg-server",
                    "xorg-xinit",
                ] {
                    app.selected_graphic_drivers.insert(k.into());
                }
            }
            app.open_xorg_type_popup();
        } else {
            app.experience_mode_index = app.experience_focus_index;
        }
    } else if app.experience_focus_index == 4 {
        super::common::advance(app);
    }
}
