use crate::app::{AppState, Focus, Screen};
use crossterm::event::KeyCode;

pub(crate) fn handle_cmdline_keys(app: &mut AppState, code: KeyCode) -> bool {
    match code {
        KeyCode::Esc => {
            app.debug_log("cmdline: close (ESC)");
            app.cmdline_open = false;
            app.cmdline_buffer.clear();
            return false;
        }
        KeyCode::Enter => {
            let cmd_string = app.cmdline_buffer.clone();
            let cmd = cmd_string.trim().to_string();
            if cmd == "w" || cmd == "wq" {
                app.debug_log(&format!("cmdline: Enter '{cmd}'"));
                if app.current_screen() == Screen::Locales {
                    app.apply_locales_edit();
                }
                app.cmdline_open = false;
                app.cmdline_buffer.clear();
                if cmd == "wq" {
                    app.focus = Focus::Menu;
                }
            } else if cmd == "q" {
                app.debug_log("cmdline: Enter 'q' -> discard");
                // Quit decision menu without saving
                if app.current_screen() == Screen::Locales {
                    app.discard_locales_edit();
                }
                app.cmdline_open = false;
                app.cmdline_buffer.clear();
                app.focus = Focus::Menu;
            } else {
                app.debug_log(&format!("cmdline: Enter unrecognized (len={})", cmd.len()));
                app.cmdline_open = false;
                app.cmdline_buffer.clear();
            }
            return false;
        }
        KeyCode::Backspace => {
            app.cmdline_buffer.pop();
            app.debug_log(&format!(
                "cmdline: backspace new_len={}",
                app.cmdline_buffer.len()
            ));
            return false;
        }
        KeyCode::Char(c) => {
            app.cmdline_buffer.push(c);
            app.debug_log(&format!(
                "cmdline: char input new_len={}",
                app.cmdline_buffer.len()
            ));
            return false;
        }
        _ => {}
    }
    false
}
