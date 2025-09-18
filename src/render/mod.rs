use ratatui::Frame;

use crate::ui::app::AppState;

mod cmdline;
mod popup;
mod sections;

pub fn draw(frame: &mut Frame, app: &mut AppState) {
    sections::draw_sections(frame, app);

    if app.popup_open {
        popup::draw_popup(frame, app);
    }

    if app.cmdline_open && app.focus == crate::ui::app::Focus::Content {
        let area = app.last_content_rect;
        cmdline::draw_cmdline(frame, app, area);
    }
}
