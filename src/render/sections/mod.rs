/// Draws the install progress (left) and command output (right) panes during installation.
fn draw_install_split(
    frame: &mut Frame,
    app: &mut AppState,
    left: ratatui::layout::Rect,
    right: ratatui::layout::Rect,
    theme: crate::render::theme::Theme,
) {
    use ratatui::style::{Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
    // Left: Overall Progress
    let mut left_lines: Vec<Line> = Vec::new();
    left_lines.push(Line::from(Span::styled(
        "Overall Progress",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )));
    left_lines.push(Line::from(""));
    for (idx, title) in app.install_section_titles.iter().enumerate() {
        let is_done = app.install_section_done.get(idx).copied().unwrap_or(false);
        let is_current = app.install_current_section == Some(idx) && !is_done;
        let marker = if is_done {
            "[x]"
        } else if is_current {
            "[>]"
        } else {
            "[ ]"
        };
        let style = if is_current {
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD)
        } else if is_done {
            Style::default().fg(theme.success)
        } else {
            Style::default().fg(theme.subtext)
        };
        left_lines.push(Line::from(vec![
            Span::styled(format!("{marker} "), style),
            Span::styled(title.clone(), style),
        ]));
    }
    let left_block = Block::default()
        .borders(Borders::ALL)
        .title(if app.install_running {
            " Installation in progress "
        } else {
            " Installation status "
        });
    let left_par = Paragraph::new(left_lines)
        .block(left_block)
        .wrap(Wrap { trim: true });
    frame.render_widget(left_par, left);

    // Right: Command Output
    let right_block = Block::default()
        .borders(Borders::ALL)
        .title(" Command output ");
    let inner_right = right_block.inner(right);
    frame.render_widget(right_block, right);
    // Explicit clear to avoid leftover characters
    frame.render_widget(ratatui::widgets::Clear, inner_right);
    let max_visible = inner_right.height.saturating_sub(2) as usize; // Reserve 1 line for indicator
    let start = app.install_log.len().saturating_sub(max_visible);
    let visible_lines = &app.install_log[start..];
    // Truncate each line to the inner width to avoid wrap artifacts
    let max_width = inner_right.width.saturating_sub(1) as usize; // keep 1 col margin
    let mut output = String::new();
    for (i, line) in visible_lines.iter().enumerate() {
        let mut truncated = String::new();
        for ch in line.chars().take(max_width) {
            truncated.push(ch);
        }
        if i > 0 {
            output.push('\n');
        }
        output.push_str(&truncated);
    }
    // Visual indicator: show only if process ended
    if !app.install_running {
        output.push_str("\n[Install process ended]");
    }
    let right_par = Paragraph::new(output).wrap(Wrap { trim: true });
    frame.render_widget(right_par, inner_right);
}
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::{AppState, KEYBINDS_WIDTH, LEFT_MENU_WIDTH, Screen};
use crate::render::theme::Theme;

pub mod content;
pub mod info;
pub mod keybinds;
pub mod menu;

pub fn draw_sections_with_theme(frame: &mut Frame, app: &mut AppState, theme: Theme) {
    let size = frame.area();

    // Draw background content first
    if app.install_running || !app.install_section_titles.is_empty() {
        // During or after installation, use a two-column layout: left (progress), right (command output)
        let cols = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Length(40),
                ratatui::layout::Constraint::Min(10),
            ])
            .split(size);
        app.last_menu_rect = cols[0];
        app.last_content_rect = cols[1];
        draw_install_split(frame, app, cols[0], cols[1], theme);
    } else {
        let left_constraint = Constraint::Length(LEFT_MENU_WIDTH);
        let hide_keybinds = app.install_running;
        let chunks = if hide_keybinds {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([left_constraint, Constraint::Min(10)])
                .split(size)
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    left_constraint,
                    Constraint::Min(10),
                    Constraint::Length(KEYBINDS_WIDTH),
                ])
                .split(size)
        };

        let left_menu_rect = chunks[0];
        let right_rect = chunks[1];
        let keybinds_rect = if hide_keybinds {
            Rect::new(0, 0, 0, 0)
        } else {
            chunks[2]
        };

        // Always split right pane 50/50 between Infobox and Decision Menu for non-Install screens
        let right_constraints = if app.current_screen() == Screen::Install {
            [Constraint::Length(0), Constraint::Percentage(100)]
        } else {
            [Constraint::Percentage(50), Constraint::Percentage(50)]
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

        if !app.install_running {
            menu::draw_menu_with_theme(frame, app, left_menu_rect, theme);
        }
        if app.current_screen() != Screen::Install {
            info::draw_info_with_theme(frame, app, infobox_rect, theme);
        }
        if !hide_keybinds {
            keybinds::draw_keybinds_with_theme(frame, keybinds_rect, theme);
        }
        content::draw_content(frame, app, content_rect);
    }

    // Then overlay the reboot prompt if open (clear the area first so underlying text doesn't bleed through)
    if app.reboot_prompt_open {
        use ratatui::style::{Modifier, Style};
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
        let area = ratatui::layout::Rect {
            x: size.width / 4,
            y: size.height / 3,
            width: size.width / 2,
            height: 7,
        };
        // Clear popup area to avoid background artifacts
        frame.render_widget(ratatui::widgets::Clear, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Reboot now? ");
        let text = vec![
            Line::from(Span::styled(
                "Installation completed.",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Do you want to reboot now? [Y/n]"),
            Line::from(""),
            Line::from(Span::styled(
                "Press Y/Enter to reboot, N/Esc to cancel.",
                Style::default().fg(theme.highlight),
            )),
        ];
        let par = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
        frame.render_widget(par, area);
    }
}
