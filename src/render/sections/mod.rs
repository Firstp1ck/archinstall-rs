/// Draws the install progress (left) and command output (right) panes during installation.
fn draw_install_split(
    frame: &mut Frame,
    app: &mut AppState,
    left: ratatui::layout::Rect,
    right: ratatui::layout::Rect,
) {
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
    // Left: Overall Progress
    let mut left_lines: Vec<Line> = Vec::new();
    left_lines.push(Line::from(Span::styled(
        "Overall Progress",
        Style::default()
            .fg(Color::Cyan)
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
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if is_done {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };
        left_lines.push(Line::from(vec![
            Span::styled(format!("{} ", marker), style),
            Span::styled(title.clone(), style),
        ]));
    }
    let left_block = Block::default()
        .borders(Borders::ALL)
        .title(" Installation in progress ");
    let left_par = Paragraph::new(left_lines)
        .block(left_block)
        .wrap(Wrap { trim: false });
    frame.render_widget(left_par, left);

    // Right: Command Output
    let right_block = Block::default()
        .borders(Borders::ALL)
        .title(" Command output ");
    let inner_right = right_block.inner(right);
    frame.render_widget(right_block, right);
    let max_visible = inner_right.height.saturating_sub(2) as usize; // Reserve 1 line for indicator
    let start = app.install_log.len().saturating_sub(max_visible);
    let visible_lines = &app.install_log[start..];
    let mut output = visible_lines.join("\n");
    // Visual indicator: show only if process ended
    if !app.install_running {
        output.push_str("\n[Install process ended]");
    }
    let right_par = Paragraph::new(output).wrap(Wrap { trim: false });
    frame.render_widget(right_par, inner_right);
}
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::{AppState, INFOBOX_HEIGHT, KEYBINDS_WIDTH, LEFT_MENU_WIDTH, Screen};

pub mod content;
pub mod info;
pub mod keybinds;
pub mod menu;

pub fn draw_sections(frame: &mut Frame, app: &mut AppState) {
    let size = frame.area();

    // left (menu) | right (info + content) | keybinds (rightmost)
    // On Install screen, show the Main Menu as well
    if app.install_running {
        // During installation, use a two-column layout: left (progress), right (command output)
        let cols = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Length(40),
                ratatui::layout::Constraint::Min(10),
            ])
            .split(size);
        app.last_menu_rect = cols[0];
        app.last_content_rect = cols[1];
        draw_install_split(frame, app, cols[0], cols[1]);
    } else if app.reboot_prompt_open {
        // Draw reboot prompt popup
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
        let area = ratatui::layout::Rect {
            x: size.width / 4,
            y: size.height / 3,
            width: size.width / 2,
            height: 7,
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Reboot now? ");
        let text = vec![
            Line::from(Span::styled(
                "Installation completed.",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Do you want to reboot now? [Y/n]"),
            Line::from(""),
            Line::from(Span::styled(
                "Press Y/Enter to reboot, N/Esc to cancel.",
                Style::default().fg(Color::Yellow),
            )),
        ];
        let par = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
        frame.render_widget(par, area);
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

        let right_constraints = if app.current_screen() == Screen::ExperienceMode {
            [Constraint::Percentage(50), Constraint::Percentage(50)]
        } else if app.current_screen() == Screen::Install {
            [Constraint::Length(0), Constraint::Percentage(100)]
        } else {
            [Constraint::Length(INFOBOX_HEIGHT), Constraint::Min(5)]
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
            menu::draw_menu(frame, app, left_menu_rect);
        }
        if app.current_screen() != Screen::Install {
            info::draw_info(frame, app, infobox_rect);
        }
        if !hide_keybinds {
            keybinds::draw_keybinds(frame, keybinds_rect);
        }
        content::draw_content(frame, app, content_rect);
    }
}
