pub use ratatui::Frame;
use ratatui::widgets::Clear;

use crate::app::AppState;

mod cmdline;
pub mod theme;
mod popup;
mod sections;

pub fn draw(frame: &mut Frame, app: &mut AppState) {
    let theme = theme::catppuccin_mocha();
    // Paint full-screen background
    let full_area = frame.area();
    frame.render_widget(Clear, full_area);
    // Respect terminal theme/background: do not force a solid background color.
    // Render sections
    sections::draw_sections_with_theme(frame, app, theme);

    if app.popup_open {
        popup::draw_popup(frame, app);
    }

    if app.cmdline_open && app.focus == crate::app::Focus::Content {
        let area = app.last_content_rect;
        cmdline::draw_cmdline(frame, app, area);
    }

    // Render ephemeral toast in bottom-right
    if let Some(msg) = app.toast_message.clone() {
        if let Some(deadline) = app.toast_deadline {
            if std::time::Instant::now() >= deadline {
                app.toast_message = None;
                app.toast_deadline = None;
            } else {
                use ratatui::layout::{Alignment, Rect};
                use ratatui::style::{Modifier, Style};
                use ratatui::text::Line;
                use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
                let area = frame.area();
                let width = msg.len().clamp(16, 48) as u16 + 4;
                let height: u16 = 3;
                let x = area.x + area.width.saturating_sub(width + 1);
                let y = area.y + area.height.saturating_sub(height + 1);
                let rect = Rect { x, y, width, height };
                let t = theme::catppuccin_mocha();
                let para = Paragraph::new(Line::from(msg))
                    .block(Block::default().borders(Borders::ALL))
                    .style(Style::default().fg(t.accent).add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });
                frame.render_widget(para, rect);
            }
        } else {
            app.toast_message = None;
        }
    }
}
