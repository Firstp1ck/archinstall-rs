use crate::ui::app::{AppState, Focus, Screen};
use crossterm::event::KeyCode;

pub(crate) fn handle_cmdline_keys(app: &mut AppState, code: KeyCode) -> bool {
    match code {
        KeyCode::Esc => {
            app.cmdline_open = false;
            app.cmdline_buffer.clear();
            return false;
        }
        KeyCode::Enter => {
            let cmd_string = app.cmdline_buffer.clone();
            let cmd = cmd_string.trim().to_string();
            if cmd == "w" || cmd == "wq" {
                if app.current_screen() == Screen::Locales {
                    app.apply_locales_edit();
                }
                app.cmdline_open = false;
                app.cmdline_buffer.clear();
                if cmd == "wq" {
                    app.focus = Focus::Menu;
                }
            } else if cmd == "q" {
                // Quit decision menu without saving
                if app.current_screen() == Screen::Locales {
                    app.discard_locales_edit();
                }
                app.cmdline_open = false;
                app.cmdline_buffer.clear();
                app.focus = Focus::Menu;
            } else {
                app.cmdline_open = false;
                app.cmdline_buffer.clear();
            }
            return false;
        }
        KeyCode::Backspace => {
            app.cmdline_buffer.pop();
            return false;
        }
        KeyCode::Char(c) => {
            app.cmdline_buffer.push(c);
            return false;
        }
        _ => {}
    }
    false
}
