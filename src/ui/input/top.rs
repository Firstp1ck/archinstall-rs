use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

use super::screens::{
    change_value, move_menu_down, move_menu_up, move_screen_down, move_screen_up,
};
use super::{cmdline::handle_cmdline_keys, popup::handle_popup_keys};
use crate::ui::app::{AppState, Focus, Screen};

// Returns true if the app should quit
pub fn handle_event(app: &mut AppState, ev: Event) -> bool {
    match ev {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            // Quit only with Ctrl-C; ESC never quits
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Esc | KeyCode::Char('q') => {
                if app.popup_open {
                    app.close_popup();
                } else if app.focus == Focus::Content {
                    if app.current_screen() == Screen::Locales {
                        app.discard_locales_edit();
                    }
                    app.focus = Focus::Menu;
                }
                return false;
            }
            // When popup is open, handle popup keys only
            _ if app.popup_open => return handle_popup_keys(app, key.code),
            // When command line is open, handle only command keys
            _ if app.cmdline_open => return handle_cmdline_keys(app, key.code),
            // Vim motions for menu and decision navigation
            KeyCode::Up | KeyCode::Char('k') => {
                if app.focus == Focus::Menu {
                    move_menu_up(app);
                } else {
                    move_screen_up(app);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.focus == Focus::Menu {
                    move_menu_down(app);
                } else {
                    move_screen_down(app);
                }
            }

            KeyCode::Enter => {
                super::screens::handle_enter(app);
            }
            KeyCode::Tab => super::screens::move_locales_focus(app, true),
            KeyCode::BackTab => super::screens::move_locales_focus(app, false),
            // Vim motions inside decision menu
            KeyCode::Left | KeyCode::Char('h') => change_value(app, false),
            KeyCode::Right | KeyCode::Char('l') => change_value(app, true),
            // Open command line (Locales)
            KeyCode::Char(':') => {
                if app.focus == Focus::Content {
                    app.cmdline_open = true;
                    app.cmdline_buffer.clear();
                }
            }
            _ => {}
        },
        // No mouse support in TTY
        Event::Resize(_, _) => {}
        _ => {}
    }
    false
}
