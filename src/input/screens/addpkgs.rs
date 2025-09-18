use crate::app::{AppState, Focus, Screen};

pub(crate) fn move_addpkgs_up(app: &mut AppState) {
    if app.current_screen() != Screen::AdditionalPackages || app.focus != Focus::Content {
        return;
    }
    if !app.additional_packages.is_empty() {
        if app.addpkgs_selected_index == 0 {
            app.addpkgs_selected_index = app.additional_packages.len() - 1;
        } else {
            app.addpkgs_selected_index -= 1;
        }
    }
}
pub(crate) fn move_addpkgs_down(app: &mut AppState) {
    if app.current_screen() != Screen::AdditionalPackages || app.focus != Focus::Content {
        return;
    }
    if !app.additional_packages.is_empty() {
        app.addpkgs_selected_index =
            (app.addpkgs_selected_index + 1) % app.additional_packages.len();
    }
}
pub(crate) fn change_addpkgs_value(app: &mut AppState, next: bool) {
    if app.current_screen() != Screen::AdditionalPackages || app.focus != Focus::Content {
        return;
    }
    let fields = 3; // Add package, Select groups, Continue
    if next {
        app.addpkgs_focus_index = (app.addpkgs_focus_index + 1) % fields;
    } else {
        app.addpkgs_focus_index = (app.addpkgs_focus_index + fields - 1) % fields;
    }
}

pub(crate) fn handle_enter_addpkgs(app: &mut AppState) {
    if app.addpkgs_focus_index == 0 {
        app.open_additional_package_input();
    } else if app.addpkgs_focus_index == 1 {
        app.open_additional_package_group_select();
    } else if app.addpkgs_focus_index == 2 {
        super::common::advance(app);
    }
}
