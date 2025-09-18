pub use ratatui::Frame;

use crate::app::AppState;

mod cmdline;
mod popup;
mod sections;

pub fn draw(frame: &mut Frame, app: &mut AppState) {
    // Clear the full frame to avoid artifacts from underlying tty contents
    let full_area = frame.area();
    frame.render_widget(ratatui::widgets::Clear, full_area);
    sections::draw_sections(frame, app);

    if app.popup_open {
        popup::draw_popup(frame, app);
    }

    if app.cmdline_open && app.focus == crate::app::Focus::Content {
        let area = app.last_content_rect;
        cmdline::draw_cmdline(frame, app, area);
    }
}
