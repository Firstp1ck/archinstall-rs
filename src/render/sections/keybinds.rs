use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn draw_keybinds_with_theme(frame: &mut Frame, area: Rect, theme: crate::render::theme::Theme) {
    let key_lines = vec![
        Line::from(Span::styled(
            "Keybindings",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Global",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "q, ESC",
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — back / close"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Main Menu",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — select section"),
        ]),
        Line::from(vec![
            Span::styled(
                "j/k, ↑/↓",
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — move selection"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Decision Menu",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "Tab / Shift-Tab",
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — switch field"),
        ]),
        Line::from(vec![
            Span::styled(
                "h/l, ←/→",
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — change value"),
        ]),
        Line::from(vec![
            Span::styled(
                ":",
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — open command line"),
        ]),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — activate item / Continue"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Popups",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "j/k, ↑/↓",
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — move selection"),
        ]),
        Line::from(vec![
            Span::styled(
                "/",
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — start search"),
        ]),
        Line::from(vec![
            Span::styled(
                "Backspace",
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — delete character"),
        ]),
        Line::from(vec![
            Span::styled(
                "Space",
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — toggle (multi-select)"),
        ]),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — select / close"),
        ]),
    ];

    let keybinds = Paragraph::new(key_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Keybindings "),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(keybinds, area);
}
