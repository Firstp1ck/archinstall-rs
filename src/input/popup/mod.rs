use crate::ui::app::AppState;
use crossterm::event::KeyCode;

mod enter;
mod left_right;
mod nav;
mod space;
mod text;

pub(crate) fn handle_popup_keys(app: &mut AppState, code: KeyCode) -> bool {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.close_popup();
            return false;
        }
        KeyCode::Enter => {
            return enter::handle_enter(app);
        }
        // Text input editing for specific popups
        KeyCode::Backspace => {
            if text::handle_text_backspace(app) {
                return false;
            }
            if text::handle_search_backspace(app) {
                return false;
            }
        }
        KeyCode::Char('/') => {
            if text::handle_search_slash(app) {
                return false;
            }
        }
        KeyCode::Char(' ') => {
            return space::handle_space(app);
        }
        KeyCode::Char(c) => {
            if text::handle_text_char(app, c) {
                return false;
            }
            if text::handle_search_char(app, c) {
                return false;
            }
            match c {
                'k' => return nav::handle_nav_up(app),
                'j' => return nav::handle_nav_down(app),
                'h' => return left_right::handle_left(app),
                'l' => return left_right::handle_right(app),
                _ => {}
            }
        }
        KeyCode::Up => {
            return nav::handle_nav_up(app);
        }
        KeyCode::Down => {
            return nav::handle_nav_down(app);
        }
        KeyCode::Left => {
            return left_right::handle_left(app);
        }
        KeyCode::Right => {
            return left_right::handle_right(app);
        }
        _ => {}
    }
    false
}
