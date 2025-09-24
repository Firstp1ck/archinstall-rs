use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::AppState;

pub(super) fn render(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let info = Paragraph::new(vec![Line::from(format!(
        "Selected: {}",
        app.menu_entries[app.selected_index].label
    ))])
    .block(Block::default().borders(Borders::ALL).title(" Info "))
    .wrap(Wrap { trim: true });
    frame.render_widget(info, area);
}
