use crate::app::AppState;
use crossterm::event::KeyCode;

mod enter;
mod left_right;
mod nav;
mod space;
mod text;

pub(crate) fn handle_popup_keys(app: &mut AppState, code: KeyCode) -> bool {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.debug_log("popup: close (ESC/q)");
            app.close_popup();
            return false;
        }
        KeyCode::Enter => {
            return enter::handle_enter(app);
        }
        // Text input editing for specific popups
        KeyCode::Backspace => {
            if text::handle_text_backspace(app) {
                app.debug_log("popup: text backspace");
                return false;
            }
            if text::handle_search_backspace(app) {
                app.debug_log("popup: search backspace");
                return false;
            }
        }
        KeyCode::Char('/') => {
            // During Manual Partitioning popups, '/' should be typed into fields (e.g., mountpoint)
            if matches!(
                app.popup_kind,
                Some(crate::app::PopupKind::ManualPartitionTable)
                    | Some(crate::app::PopupKind::ManualPartitionCreate)
                    | Some(crate::app::PopupKind::ManualPartitionKindSelect)
                    | Some(crate::app::PopupKind::ManualPartitionFilesystem)
                    | Some(crate::app::PopupKind::ManualPartitionMountpoint)
                    | Some(crate::app::PopupKind::ManualPartitionEdit)
            ) {
                if text::handle_text_char(app, '/') {
                    return false;
                }
            } else if text::handle_search_slash(app) {
                app.debug_log("popup: enter search mode");
                return false;
            }
        }
        KeyCode::Char(' ') => {
            return space::handle_space(app);
        }
        KeyCode::Char(c) => {
            if text::handle_text_char(app, c) {
                // do not log text content
                return false;
            }
            if text::handle_search_char(app, c) {
                app.debug_log(&format!(
                    "popup: search char (query_len={})",
                    app.popup_search_query.len()
                ));
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
