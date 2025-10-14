pub use ratatui::Frame;
use ratatui::widgets::Clear;

use crate::app::AppState;

mod cmdline;
mod popup;
mod sections;

pub fn draw(frame: &mut Frame, app: &mut AppState) {
    // Paint full-screen background
    let full_area = frame.area();
    frame.render_widget(Clear, full_area);
    // Respect terminal theme/background: do not force a solid background color.
    // Render sections
    sections::draw_sections(frame, app);

    if app.popup_open {
        popup::draw_popup(frame, app);
    }

    if app.cmdline_open && app.focus == crate::app::Focus::Content {
        let area = app.last_content_rect;
        cmdline::draw_cmdline(frame, app, area);
    }
}
