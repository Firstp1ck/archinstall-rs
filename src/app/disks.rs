use serde_json;
use std::process::Command;

use super::AppState;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

#[derive(Clone)]
pub struct DiskDevice {
    pub model: String,
    pub path: String,
    pub devtype: String,
    pub size: String,
    pub freespace: String,
    pub sector_size: String,
    pub read_only: bool,
}

pub fn draw_disks(frame: &mut ratatui::Frame, app: &mut AppState, area: Rect) {
    let title = Span::styled(
        "Disk Partitioning",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let options = vec![
        ("Best-effort partition layout", 0),
        ("Manual Partitioning", 1),
        ("Pre-mounted configuration", 2),
    ];

    let mut lines: Vec<Line> = vec![Line::from(title), Line::from("")];
    // TODO(v0.2.0): Implement Manual Partitioning editor and Pre-mounted flow.
    for (label, idx) in options {
        let is_focused_line = app.disks_focus_index == idx;
        let is_active_line = is_focused_line && matches!(app.focus, super::Focus::Content);
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
        let line = Line::from(vec![
            Span::styled(format!("{} ", bullet), bullet_style),
            Span::styled(label.to_string(), label_style),
        ]);
        lines.push(line);
    }

    let continue_style = if app.disks_focus_index == 3 && matches!(app.focus, super::Focus::Content)
    {
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
                    super::Focus::Content => " Desicion Menu (focused) ",
                    _ => " Desicion Menu ",
                }),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(content, area);
}

impl AppState {
    pub(crate) fn human_bytes(n: u64) -> String {
        const UNITS: [&str; 7] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
        let mut value = n as f64;
        let mut unit = 0;
        while value >= 1024.0 && unit + 1 < UNITS.len() {
            value /= 1024.0;
            unit += 1;
        }
        if unit == 0 {
            format!("{} {}", n, UNITS[unit])
        } else {
            format!("{:.1} {}", value, UNITS[unit])
        }
    }

    pub fn load_disks_devices(&mut self) {
        if let Ok(output) = Command::new("lsblk")
            .args([
                "-J",
                "-b",
                "-o",
                "NAME,PATH,TYPE,SIZE,RO,MODEL,LOG-SEC,PHY-SEC",
            ])
            .output()
            && output.status.success()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                let mut devices: Vec<DiskDevice> = Vec::new();
                if let Some(blockdevices) = json.get("blockdevices").and_then(|v| v.as_array()) {
                    for dev in blockdevices {
                        let devtype = dev
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if devtype != "disk" {
                            continue;
                        }
                        let mut model = dev
                            .get("model")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        let name = dev
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if model.is_empty() {
                            model = name.clone();
                        }
                        let path = dev
                            .get("path")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| format!("/dev/{}", name));
                        let size_b = dev.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                        let ro = dev.get("ro").and_then(|v| v.as_u64()).unwrap_or(0) != 0;
                        let log_sec = dev.get("log-sec").and_then(|v| v.as_u64()).unwrap_or(0);
                        let phy_sec = dev.get("phy-sec").and_then(|v| v.as_u64()).unwrap_or(0);
                        let mut used_b: u64 = 0;
                        if let Some(children) = dev.get("children").and_then(|v| v.as_array()) {
                            for ch in children {
                                if let Some(sz) = ch.get("size").and_then(|v| v.as_u64()) {
                                    used_b = used_b.saturating_add(sz);
                                }
                            }
                        }
                        let free_b = size_b.saturating_sub(used_b);
                        let size = Self::human_bytes(size_b);
                        let freespace = if size_b == 0 {
                            "-".into()
                        } else {
                            Self::human_bytes(free_b)
                        };
                        let sector_size = if log_sec == 0 && phy_sec == 0 {
                            String::new()
                        } else if log_sec != 0 && phy_sec != 0 && log_sec != phy_sec {
                            format!("{}/{} B", log_sec, phy_sec)
                        } else {
                            let s = if phy_sec != 0 { phy_sec } else { log_sec };
                            format!("{} B", s)
                        };
                        devices.push(DiskDevice {
                            model,
                            path,
                            devtype,
                            size,
                            freespace,
                            sector_size,
                            read_only: ro,
                        });
                    }
                }
                self.disks_devices = devices;
                // Default disk selection preference: /dev/vda, fallback /dev/vdb
                if self.disks_selected_device.is_none() {
                    if self.disks_devices.iter().any(|d| d.path == "/dev/vda") {
                        self.disks_selected_device = Some("/dev/vda".into());
                    } else if self.disks_devices.iter().any(|d| d.path == "/dev/vdb") {
                        self.disks_selected_device = Some("/dev/vdb".into());
                    }
                }
            }
        }
    }

    pub fn open_disks_device_list(&mut self) {
        self.load_disks_devices();
        self.popup_kind = Some(super::PopupKind::DisksDeviceList);
        let mut lines: Vec<String> = Vec::new();
        for d in &self.disks_devices {
            let ro = if d.read_only { "True" } else { "False" };
            lines.push(format!(
                "{:<30} | {:<20} | {:<6} | {:<10} | {:<12} | {:<15} | {:<10}",
                d.model, d.path, d.devtype, d.size, d.freespace, d.sector_size, ro
            ));
        }
        self.popup_items = lines;
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_open = true;
    }
}
