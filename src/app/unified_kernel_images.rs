use super::{AppState, Focus, MenuEntry, Screen};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

impl AppState {
    #[allow(dead_code)]
    pub fn init_unified_kernel_images(&mut self) {
        // TODO(v0.2.0): Implement UKI generation plan and mkinitcpio/systemd-boot integration.
    }

    pub fn update_unified_kernel_images_visibility(&mut self) {
        // Show UKI for all bootloaders except GRUB (index 1)
        let should_show = self.bootloader_index != 1;
        let uki_pos = self
            .menu_entries
            .iter()
            .position(|e| matches!(e.screen, Screen::UnifiedKernelImages));
        if should_show {
            if uki_pos.is_none()
                && let Some(bl_idx) = self
                    .menu_entries
                    .iter()
                    .position(|e| matches!(e.screen, Screen::Bootloader))
            {
                let insert_at = bl_idx + 1;
                self.menu_entries.insert(
                    insert_at,
                    MenuEntry {
                        label: "Unified Kernel Images".into(),
                        content: String::new(),
                        screen: Screen::UnifiedKernelImages,
                    },
                );
                if self.selected_index >= insert_at {
                    self.selected_index += 1;
                    self.list_state.select(Some(self.selected_index));
                }
            }
        } else if let Some(idx) = uki_pos {
            // If currently selected, move selection away first
            if self.selected_index == idx {
                if idx + 1 < self.menu_entries.len() {
                    self.selected_index += 1;
                } else if idx > 0 {
                    self.selected_index -= 1;
                }
                self.list_state.select(Some(self.selected_index));
            }
            self.menu_entries.remove(idx);
            if self.selected_index > idx {
                self.selected_index = self.selected_index.saturating_sub(1);
                self.list_state.select(Some(self.selected_index));
            }
        }
    }
}

pub fn draw_unified_kernel_images(frame: &mut ratatui::Frame, app: &mut AppState, area: Rect) {
    let title = Span::styled(
        "Unified Kernel Images",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let mut lines: Vec<Line> = vec![Line::from(title), Line::from("")];

    let options = vec![(
        "UKI",
        if app.uki_enabled {
            "Enabled"
        } else {
            "Disabled"
        },
        app.uki_focus_index == 0,
    )];

    for (label, value, is_focused_line) in options {
        let is_active_line = is_focused_line && matches!(app.focus, Focus::Content);
        let bullet = if is_focused_line { "▶" } else { " " };
        let bullet_style = if is_active_line {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let label_style = if is_active_line {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let value_style = if is_active_line {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let line = Line::from(vec![
            Span::styled(format!("{bullet} "), bullet_style),
            Span::styled(format!("{label}: "), label_style),
            Span::styled(value.to_string(), value_style),
        ]);
        lines.push(line);
    }

    // Continue
    let continue_style = if app.uki_focus_index == 1 && matches!(app.focus, Focus::Content) {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("[ Continue ]", continue_style)));

    let content = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(match app.focus {
                    Focus::Content => " Desicion Menu (focused) ",
                    _ => " Desicion Menu ",
                }),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(content, area);
}
