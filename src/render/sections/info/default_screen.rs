use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::AppState;
use crate::core::types::Screen;

pub(super) fn render(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let mut lines = vec![Line::from(format!(
        "Selected: {}",
        app.menu_entries[app.selected_index].label
    ))];
    if app.current_screen() == Screen::Overview {
        lines.push(Line::from(format!(
            "Secure Boot: {}",
            app.secure_boot_status_text()
        )));
    }
    let info = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Info "))
        .wrap(Wrap { trim: true });
    frame.render_widget(info, area);
}
