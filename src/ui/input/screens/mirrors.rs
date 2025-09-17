use crate::ui::app::AppState;

pub(crate) fn handle_enter_mirrors(app: &mut AppState) {
    match app.mirrors_focus_index {
        0 => app.open_mirrors_regions_popup(),
        4 => super::common::advance(app),
        2 => app.open_optional_repos_popup(),
        1 => app.open_mirrors_custom_server_input(),
        3 => app.open_mirrors_custom_repo_flow(),
        _ => {}
    }
}

pub(crate) fn move_mirrors_repos_up(app: &mut AppState) {
    if app.current_screen() != crate::ui::app::Screen::MirrorsRepos
        || app.focus != crate::ui::app::Focus::Content
    {
        return;
    }
    if app.mirrors_focus_index == 0 {
        app.mirrors_focus_index = 4;
    } else {
        app.mirrors_focus_index -= 1;
    }
}

pub(crate) fn move_mirrors_repos_down(app: &mut AppState) {
    if app.current_screen() != crate::ui::app::Screen::MirrorsRepos
        || app.focus != crate::ui::app::Focus::Content
    {
        return;
    }
    app.mirrors_focus_index = (app.mirrors_focus_index + 1) % 5;
}


