pub use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Clear};

use crate::app::AppState;

mod cmdline;
mod popup;
mod sections;

fn effective_bg_color() -> Color {
    let colorterm = std::env::var("COLORTERM").unwrap_or_default().to_lowercase();
    let term = std::env::var("TERM").unwrap_or_default().to_lowercase();

    if colorterm.contains("truecolor") || colorterm.contains("24bit") {
        // Exact Dracula background
        Color::Rgb(0x28, 0x2A, 0x36)
    } else if term.contains("256color") {
        // Approximate #282A36 in xterm-256 palette (close to 236/237)
        Color::Indexed(236)
    } else {
        // Plain linux TTY: limited colors → pick a dark fallback
        Color::Black
    }
}

pub fn draw(frame: &mut Frame, app: &mut AppState) {
    // Paint full-screen background
    let full_area = frame.area();
    frame.render_widget(Clear, full_area);
    let bg_color = effective_bg_color();
    let bg = Block::default().style(Style::default().bg(bg_color));
    frame.render_widget(bg, full_area);
    sections::draw_sections(frame, app);

    if app.popup_open {
        popup::draw_popup(frame, app);
    }

    if app.cmdline_open && app.focus == crate::app::Focus::Content {
        let area = app.last_content_rect;
        cmdline::draw_cmdline(frame, app, area);
    }
}
