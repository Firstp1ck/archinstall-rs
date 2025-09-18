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
    // On Install screen, collapse the left menu to give full width to the overview
    let left_constraint = if app.current_screen() == Screen::Install {
        Constraint::Length(0)
    } else {
        Constraint::Length(LEFT_MENU_WIDTH)
    };
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            left_constraint,
            Constraint::Min(10),
            Constraint::Length(KEYBINDS_WIDTH),
        ])
        .split(size);

    let left_menu_rect = chunks[0];
    let right_rect = chunks[1];
    let keybinds_rect = chunks[2];

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

    // Render sections
    if app.current_screen() != Screen::Install {
        menu::draw_menu(frame, app, left_menu_rect);
    }
    if app.current_screen() != Screen::Install {
        info::draw_info(frame, app, infobox_rect);
    }
    keybinds::draw_keybinds(frame, keybinds_rect);
    content::draw_content(frame, app, content_rect);
}
