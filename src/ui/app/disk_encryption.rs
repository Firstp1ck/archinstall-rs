use std::process::Command;
use serde_json;

use super::AppState;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

impl AppState {
    pub fn init_disk_encryption(&mut self) {
        // placeholder
    }

    pub fn open_disk_encryption_type_popup(&mut self) {
        self.popup_kind = Some(super::PopupKind::DiskEncryptionType);
        self.popup_items = vec!["None".into(), "LUKS".into()];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = self.disk_encryption_type_index.min(self.popup_items.len().saturating_sub(1));
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_open = true;
    }

    pub fn open_disk_encryption_password_input(&mut self) {
        self.popup_kind = Some(super::PopupKind::DiskEncryptionPassword);
        self.custom_input_buffer.clear();
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_open = true;
    }

    pub fn open_disk_encryption_password_confirm_input(&mut self) {
        self.popup_kind = Some(super::PopupKind::DiskEncryptionPasswordConfirm);
        self.custom_input_buffer.clear();
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_open = true;
    }

    pub fn open_disk_encryption_partition_list(&mut self) {
        self.popup_kind = Some(super::PopupKind::DiskEncryptionPartitionList);
        let selected_device = self.disks_selected_device.clone();
        self.popup_items = Self::collect_partitions(selected_device.as_deref());
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_open = true;
    }

    fn collect_partitions(device_filter: Option<&str>) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        if device_filter.is_none() { return lines; }
        if let Ok(output) = Command::new("lsblk").args([
            "-J",
            "-b",
            "-o",
            "NAME,PATH,TYPE,SIZE",
        ]).output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(blockdevices) = json.get("blockdevices").and_then(|v| v.as_array()) {
                        for dev in blockdevices {
                            if let Some(children) = dev.get("children").and_then(|v| v.as_array()) {
                                for ch in children {
                                    let typ = ch.get("type").and_then(|v| v.as_str()).unwrap_or("");
                                    if typ != "part" { continue; }
                                    let name = ch.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                    let path = ch.get("path").and_then(|v| v.as_str()).unwrap_or("");
                                    if let Some(filter) = device_filter {
                                        if !path.starts_with(filter) { continue; }
                                    }
                                    let size_b = ch.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                                    let size = Self::human_bytes(size_b);
                                    lines.push(format!("{:<15} | {:<20} | {:<10}", name, path, size));
                                }
                            }
                        }
                    }
                }
            }
        }
        lines
    }
}

pub fn draw_disk_encryption(frame: &mut ratatui::Frame, app: &mut AppState, area: Rect) {
    let title = Span::styled(
        "Disk Encryption",
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
    );

    let enc_type = match app.disk_encryption_type_index { 0 => "None", _ => "LUKS" };

    let mut lines: Vec<Line> = vec![Line::from(title), Line::from("")];

    // Field 0: Encryption Type (always visible)
    let is_focused_line = app.diskenc_focus_index == 0 && matches!(app.focus, super::Focus::Content);
    let bullet = if app.diskenc_focus_index == 0 { "▶" } else { " " };
    let bullet_style = if is_focused_line { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
    let label_style = if is_focused_line { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
    let value_style = if is_focused_line { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
    lines.push(Line::from(vec![
        Span::styled(format!("{} ", bullet), bullet_style),
        Span::styled("Encryption Type: ", label_style),
        Span::styled(enc_type.to_string(), value_style),
    ]));

    // If LUKS selected, show password, confirm, and partition selector
    if app.disk_encryption_type_index == 1 {
        // Password
        let idx = 1;
        let is_act = app.diskenc_focus_index == idx && matches!(app.focus, super::Focus::Content);
        let bullet_style = if is_act { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
        let label_style = if is_act { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
        let value_style = if is_act { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
        let masked = "*".repeat(app.disk_encryption_password.chars().count());
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", if app.diskenc_focus_index == idx { "▶" } else { " " }), bullet_style),
            Span::styled("Password: ", label_style),
            Span::styled(masked, value_style),
        ]));

        // Confirm Password
        let idx = 2;
        let is_act = app.diskenc_focus_index == idx && matches!(app.focus, super::Focus::Content);
        let bullet_style = if is_act { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
        let label_style = if is_act { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
        let value_style = if is_act { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
        let masked = "*".repeat(app.disk_encryption_password_confirm.chars().count());
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", if app.diskenc_focus_index == idx { "▶" } else { " " }), bullet_style),
            Span::styled("Confirm Password: ", label_style),
            Span::styled(masked, value_style),
        ]));

        // Partition selector
        let idx = 3;
        let is_act = app.diskenc_focus_index == idx && matches!(app.focus, super::Focus::Content);
        let bullet_style = if is_act { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
        let label_style = if is_act { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
        let value_style = if is_act { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
        let part = app.disk_encryption_selected_partition.clone().unwrap_or_else(|| "(none)".into());
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", if app.diskenc_focus_index == idx { "▶" } else { " " }), bullet_style),
            Span::styled("Partition: ", label_style),
            Span::styled(part, value_style),
        ]));
    }

    // Continue
    let continue_index = if app.disk_encryption_type_index == 1 { 4 } else { 1 };
    let continue_style = if app.diskenc_focus_index == continue_index && matches!(app.focus, super::Focus::Content) {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("[ Continue ]", continue_style)));

    let content = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(match app.focus { super::Focus::Content => " Desicion Menu (focused) ", _ => " Desicion Menu " }))
        .wrap(Wrap { trim: false });
    frame.render_widget(content, area);
}


