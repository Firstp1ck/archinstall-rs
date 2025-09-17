use crate::ui::app::{AppState, Screen};

// Shared helper used by many screen handlers
pub(crate) fn advance(app: &mut AppState) {
    if app.selected_index + 1 < app.menu_entries.len() {
        let screen = app.current_screen();
        app.processed_sections.insert(screen);
        app.selected_index += 1;
        app.list_state.select(Some(app.selected_index));
        app.focus = crate::ui::app::Focus::Content;
        if app.current_screen() == Screen::Locales {
            app.start_locales_edit();
        }
    } else {
        let screen = app.current_screen();
        app.processed_sections.insert(screen);
        app.focus = crate::ui::app::Focus::Menu;
    }
}


