use crate::ui::app::{AppState, Focus, Screen};

pub(crate) fn move_locales_focus(app: &mut AppState, forward: bool) {
    if app.current_screen() != Screen::Locales {
        return;
    }
    let max = 4; // 3 fields + Continue
    if forward {
        app.locales_focus_index = (app.locales_focus_index + 1) % max;
    } else {
        app.locales_focus_index = (app.locales_focus_index + max - 1) % max;
    }
}

pub(crate) fn change_locales_value(app: &mut AppState, next: bool) {
    if app.current_screen() != Screen::Locales || app.focus != Focus::Content {
        return;
    }
    match app.locales_focus_index {
        0 => cycle_index(
            &mut app.draft_keyboard_layout_index,
            app.keyboard_layout_options.len(),
            next,
        ),
        1 => cycle_index(
            &mut app.draft_locale_language_index,
            app.locale_language_options.len(),
            next,
        ),
        2 => cycle_index(
            &mut app.draft_locale_encoding_index,
            app.locale_encoding_options.len(),
            next,
        ),
        _ => {}
    }
}

pub(crate) fn cycle_index(index: &mut usize, len: usize, next: bool) {
    if len == 0 {
        return;
    }
    if next {
        *index = (*index + 1) % len;
    } else {
        *index = (*index + len - 1) % len;
    }
}

pub(crate) fn move_locales_up(app: &mut AppState) {
    if app.current_screen() != Screen::Locales || app.focus != Focus::Content {
        return;
    }
    if app.locales_focus_index == 0 {
        app.locales_focus_index = 3;
    } else {
        app.locales_focus_index -= 1;
    }
}

pub(crate) fn move_locales_down(app: &mut AppState) {
    if app.current_screen() != Screen::Locales || app.focus != Focus::Content {
        return;
    }
    app.locales_focus_index = (app.locales_focus_index + 1) % 4;
}

pub(crate) fn handle_enter_locales(app: &mut AppState) {
    if app.locales_focus_index <= 2 {
        app.open_locales_popup();
    } else if app.locales_focus_index == 3 {
        app.apply_locales_edit();
        super::common::advance(app);
    }
}
