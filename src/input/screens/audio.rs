use crate::app::{AppState, Focus, Screen};

pub(crate) fn move_audio_up(app: &mut AppState) {
    if app.current_screen() != Screen::Audio || app.focus != Focus::Content {
        return;
    }
    if app.audio_focus_index == 0 {
        app.audio_focus_index = 3;
    } else {
        app.audio_focus_index -= 1;
    }
}

pub(crate) fn move_audio_down(app: &mut AppState) {
    if app.current_screen() != Screen::Audio || app.focus != Focus::Content {
        return;
    }
    app.audio_focus_index = (app.audio_focus_index + 1) % 4;
}

pub(crate) fn change_audio_value(app: &mut AppState, _next: bool) {
    if app.current_screen() != Screen::Audio || app.focus != Focus::Content {
        return;
    }
    if app.audio_focus_index < 3 {
        app.audio_index = app.audio_focus_index;
    }
}

pub(crate) fn handle_enter_audio(app: &mut AppState) {
    if app.audio_focus_index < 3 {
        app.audio_index = app.audio_focus_index;
    } else {
        super::common::advance(app);
    }
}
