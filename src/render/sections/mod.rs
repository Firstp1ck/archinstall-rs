use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::{AppState, INFOBOX_HEIGHT, KEYBINDS_WIDTH, LEFT_MENU_WIDTH, Screen};

pub mod content;
pub mod info;
pub mod keybinds;
pub mod menu;

pub fn draw_sections(frame: &mut Frame, app: &mut AppState) {
    let size = frame.area();

    // left (menu) | right (info + content) | keybinds (rightmost)
    // On Install screen, show the Main Menu as well
    let left_constraint = Constraint::Length(LEFT_MENU_WIDTH);
    // During install process, hide keybinds pane to maximize content area
    let hide_keybinds = app.install_running;
    let chunks = if hide_keybinds {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([left_constraint, Constraint::Min(10)])
            .split(size)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                left_constraint,
                Constraint::Min(10),
                Constraint::Length(KEYBINDS_WIDTH),
            ])
            .split(size)
    };

    let left_menu_rect = chunks[0];
    let right_rect = chunks[1];
    let keybinds_rect = if hide_keybinds {
        // dummy rect; will not be rendered
        Rect::new(0, 0, 0, 0)
    } else {
        chunks[2]
    };

    // On ExperienceMode, split Info and Decision content evenly
    // On Install, hide Info and give full height to Decision content
    let right_constraints = if app.current_screen() == Screen::ExperienceMode {
        [Constraint::Percentage(50), Constraint::Percentage(50)]
    } else if app.current_screen() == Screen::Install {
        [Constraint::Length(0), Constraint::Percentage(100)]
    } else {
        [Constraint::Length(INFOBOX_HEIGHT), Constraint::Min(5)]
    };
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(right_constraints)
        .split(right_rect);

    let infobox_rect: Rect = right_chunks[0];
    let content_rect: Rect = right_chunks[1];

    app.last_menu_rect = left_menu_rect;
    app.last_infobox_rect = infobox_rect;
    app.last_content_rect = content_rect;

    // Render sections; keep Main Menu visible, but skip Info on Install
    menu::draw_menu(frame, app, left_menu_rect);
    if app.current_screen() != Screen::Install {
        info::draw_info(frame, app, infobox_rect);
    }
    if !hide_keybinds {
        keybinds::draw_keybinds(frame, keybinds_rect);
    }
    content::draw_content(frame, app, content_rect);
}
