use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::AppState;

pub fn draw_cmdline(frame: &mut Frame, app: &mut AppState, area: Rect) {
    // Render a one-line command area at the very bottom inside the decision panel
    let cmd_area = Rect {
        x: area.x + 1,
        y: area.y + area.height.saturating_sub(3),
        width: area.width.saturating_sub(2),
        height: 3,
    };
    let content = Paragraph::new(Line::from(vec![
        Span::styled(
            ":",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(app.cmdline_buffer.clone()),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Command "))
    .wrap(Wrap { trim: false });
    frame.render_widget(content, cmd_area);
}
