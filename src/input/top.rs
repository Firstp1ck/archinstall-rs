use crossterm::event::{
    Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use super::screens::{
    change_value, move_menu_down, move_menu_up, move_screen_down, move_screen_up,
};
use super::{cmdline::handle_cmdline_keys, popup::handle_popup_keys};
use crate::app::{AppState, Focus, Screen};

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
            KeyCode::Up => {
                if app.focus == Focus::Menu {
                    move_menu_up(app);
                } else {
                    // In Additional Packages screen, Up moves selection within packages
                    if app.current_screen() == Screen::AdditionalPackages {
                        super::screens::move_addpkgs_up(app);
                    } else {
                        move_screen_up(app);
                    }
                }
            }
            KeyCode::Down => {
                if app.focus == Focus::Menu {
                    move_menu_down(app);
                } else if app.current_screen() == Screen::AdditionalPackages {
                    super::screens::move_addpkgs_down(app);
                } else {
                    move_screen_down(app);
                }
            }
            KeyCode::Char('k') => {
                if app.focus == Focus::Menu {
                    move_menu_up(app);
                } else if app.current_screen() == Screen::AdditionalPackages {
                    // In Additional Packages screen, j/k switch between Add package and Continue
                    super::screens::change_value(app, false);
                } else {
                    move_screen_up(app);
                }
            }
            KeyCode::Char('j') => {
                if app.focus == Focus::Menu {
                    move_menu_down(app);
                } else if app.current_screen() == Screen::AdditionalPackages {
                    super::screens::change_value(app, true);
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
            KeyCode::Char(' ') => {
                if app.focus == Focus::Content
                    && app.current_screen() == Screen::AdditionalPackages
                    && !app.additional_packages.is_empty()
                {
                    let i = app
                        .addpkgs_selected_index
                        .min(app.additional_packages.len() - 1);
                    if app.addpkgs_selected.contains(&i) {
                        app.addpkgs_selected.remove(&i);
                    } else {
                        app.addpkgs_selected.insert(i);
                    }
                }
            }
            KeyCode::Backspace | KeyCode::Delete => {
                if app.focus == Focus::Content
                    && app.current_screen() == Screen::AdditionalPackages
                    && !app.additional_packages.is_empty()
                {
                    if app.addpkgs_selected.is_empty() {
                        // delete current
                        if app.addpkgs_selected_index < app.additional_packages.len() {
                            app.additional_packages.remove(app.addpkgs_selected_index);
                        }
                    } else {
                        // delete all checked (highest index first)
                        let mut to_delete: Vec<usize> =
                            app.addpkgs_selected.iter().copied().collect();
                        to_delete.sort_by(|a, b| b.cmp(a));
                        for idx in to_delete {
                            if idx < app.additional_packages.len() {
                                app.additional_packages.remove(idx);
                            }
                        }
                        app.addpkgs_selected.clear();
                    }
                    if app.addpkgs_selected_index >= app.additional_packages.len() {
                        app.addpkgs_selected_index =
                            app.additional_packages.len().saturating_sub(1);
                    }
                }
            }
            // Open command line (Locales)
            KeyCode::Char(':') => {
                if app.focus == Focus::Content {
                    app.cmdline_open = true;
                    app.cmdline_buffer.clear();
                }
            }
            _ => {}
        },
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            ..
        }) => {
            // Only handle clicks in Install screen decision menu
            if app.current_screen() == Screen::Install && !app.popup_open {
                let x = column;
                let y = row;
                // Check click against computed targets
                for (i, (rect, target)) in app.install_click_targets.clone().into_iter().enumerate()
                {
                    if x >= rect.x
                        && x < rect.x + rect.width
                        && y >= rect.y
                        && y < rect.y + rect.height
                    {
                        app.install_focus_index = i;
                        match target {
                            crate::core::types::InstallClickTarget::Section(screen) => {
                                if let Some(idx) =
                                    app.menu_entries.iter().position(|m| m.screen == screen)
                                {
                                    app.selected_index = idx;
                                    app.list_state.select(Some(idx));
                                    app.focus = Focus::Content;
                                }
                            }
                            crate::core::types::InstallClickTarget::InstallButton => {
                                super::screens::dispatcher::handle_enter(app);
                            }
                        }
                        break;
                    }
                }
            }
        }
        // No mouse support in TTY for other events
        Event::Resize(_, _) => {}
        _ => {}
    }
    false
}
