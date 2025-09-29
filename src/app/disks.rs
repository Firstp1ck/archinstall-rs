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
            Span::styled(format!("{bullet} "), bullet_style),
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
                            .unwrap_or_else(|| format!("/dev/{name}"));
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
                            format!("{log_sec}/{phy_sec} B")
                        } else {
                            let s = if phy_sec != 0 { phy_sec } else { log_sec };
                            format!("{s} B")
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

        // Fallback single fake device for Windows/dry-run when lsblk is unavailable
        if (cfg!(windows) || self.dry_run) && self.disks_devices.is_empty() {
            self.disks_devices = vec![DiskDevice {
                model: "Virtual Disk".into(),
                path: "/dev/sda".into(),
                devtype: "disk".into(),
                size: "64.0 GiB".into(),
                freespace: "64.0 GiB".into(),
                sector_size: "512 B".into(),
                read_only: false,
            }];
            if self.disks_selected_device.is_none() {
                self.disks_selected_device = Some("/dev/sda".into());
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

    pub fn open_manual_partition_table_for_selected(&mut self) {
        let Some(device) = self.disks_selected_device.clone() else {
            return;
        };
        self.popup_kind = Some(super::PopupKind::ManualPartitionTable);
        // Build rows: Status | Path | Type | Start | End | Size | FS | Mountpoint | Mount options
        let mut rows: Vec<String> = Vec::new();
        // Build an auxiliary meta list to keep selection behavior without hidden markers
        let mut meta: Vec<crate::core::types::ManualPartitionRowMeta> = Vec::new();
        // Use lsblk JSON for partitions and filesystems
        if let Ok(output) = Command::new("lsblk")
            .args([
                "-J",
                "-b",
                "-o",
                "NAME,PATH,TYPE,SIZE,FSTYPE,MOUNTPOINT,LABEL,PARTFLAGS,START,PHY-SEC,LOG-SEC",
            ])
            .output()
            && output.status.success()
            && let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout)
            && let Some(blockdevices) = json.get("blockdevices").and_then(|v| v.as_array())
        {
            for dev in blockdevices {
                let path = dev.get("path").and_then(|v| v.as_str()).unwrap_or("");
                if !device.is_empty() && !path.starts_with(&device) {
                    continue;
                }
                // Include created specs and free space as rows
                let parent_size = dev.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                // Track used ranges (start,end) across existing and created partitions
                let mut used_ranges: Vec<(u64, u64)> = Vec::new();
                if let Some(children) = dev.get("children").and_then(|v| v.as_array()) {
                    let sector_size = dev
                        .get("phy-sec")
                        .and_then(|v| v.as_u64())
                        .or_else(|| dev.get("log-sec").and_then(|v| v.as_u64()))
                        .unwrap_or(512);
                    for ch in children {
                        let ch_name = ch.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let ch_type = ch.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        if ch_type != "part" {
                            continue;
                        }
                        let ch_size = ch.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                        let ch_fstype = ch.get("fstype").and_then(|v| v.as_str()).unwrap_or("");
                        let ch_mount = ch.get("mountpoint").and_then(|v| v.as_str()).unwrap_or("");
                        // START is in sectors; convert to bytes using detected sector size
                        let start_sectors = ch.get("start").and_then(|v| v.as_u64()).unwrap_or(0);
                        let start_bytes = start_sectors.saturating_mul(sector_size);
                        let end_bytes = start_bytes.saturating_add(ch_size);
                        used_ranges.push((start_bytes, end_bytes));
                        // Flags from PARTFLAGS or LABEL heuristics
                        let _flags = ch.get("partflags").and_then(|v| v.as_str()).unwrap_or("");
                        let status = "existing"; // existing partition
                        let dev_display = if ch_name.is_empty() { path } else { ch_name };
                        let size_h = Self::human_bytes(ch_size);
                        let start_h = if start_bytes == 0 {
                            String::new()
                        } else {
                            format!("{start_bytes}")
                        };
                        let end_h = if end_bytes == 0 {
                            String::new()
                        } else {
                            format!("{end_bytes}")
                        };
                        rows.push(format!(
                                        "{:<8} | {:<20} | {:<18} | {:<12} | {:<12} | {:<10} | {:<14} | {:<12} | {:<14}",
                                        status,
                                        dev_display,
                                        "Primary",
                                        start_h,
                                        end_h,
                                        size_h,
                                        ch_fstype,
                                        ch_mount,
                                        ""
                                    ));
                        meta.push(crate::core::types::ManualPartitionRowMeta {
                            kind: "existing".into(),
                            spec_index: None,
                            free_start: None,
                            free_end: None,
                        });
                    }
                }
                // Always show created specs for this selected device
                let mut next_other_index: u32 = 4;
                let part_path_for = |base: &str, n: u32| -> String {
                    if base
                        .chars()
                        .last()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                    {
                        format!("{base}p{n}")
                    } else {
                        format!("{base}{n}")
                    }
                };
                for (spec_idx, spec) in self.disks_partitions.iter().enumerate() {
                    // If spec.name is set, only show for that device
                    if let Some(dev_name) = &spec.name
                        && dev_name != path
                    {
                        continue;
                    }
                    let role = spec.role.clone().unwrap_or_default();
                    let fs = spec.fs.clone().unwrap_or_default();
                    let mp = spec.mountpoint.clone().unwrap_or_default();
                    let start = spec.start.clone().unwrap_or_default();
                    let size = spec.size.clone().unwrap_or_default();
                    let (end, size_h) =
                        if let (Ok(st), Ok(sz)) = (start.parse::<u64>(), size.parse::<u64>()) {
                            // Track as used range
                            let end_b = st.saturating_add(sz);
                            used_ranges.push((st, end_b));
                            (format!("{end_b}"), Self::human_bytes(sz))
                        } else {
                            (String::new(), size)
                        };
                    // Derive display partition path numbering: 1 BOOT, 2 SWAP, 3 ROOT, >=4 OTHER
                    let display_path = match role.as_str() {
                        "BOOT" => part_path_for(path, 1),
                        "SWAP" => part_path_for(path, 2),
                        "ROOT" => part_path_for(path, 3),
                        _ => {
                            let p = part_path_for(path, next_other_index);
                            next_other_index += 1;
                            p
                        }
                    };
                    rows.push(format!(
                                    "{:<8} | {:<20} | {:<18} | {:<12} | {:<12} | {:<10} | {:<14} | {:<12} | {:<14}",
                                    "created",
                                    display_path,
                                    role,
                                    start,
                                    end,
                                    size_h,
                                    fs,
                                    mp,
                                    ""
                                ));
                    meta.push(crate::core::types::ManualPartitionRowMeta {
                        kind: "created".into(),
                        spec_index: Some(spec_idx),
                        free_start: None,
                        free_end: None,
                    });
                }

                // Merge used ranges and compute free gaps across the device
                if parent_size > 0 {
                    used_ranges.sort_by_key(|(s, _)| *s);
                    let mut merged: Vec<(u64, u64)> = Vec::new();
                    for (s, e) in used_ranges {
                        if merged.is_empty() {
                            merged.push((s, e));
                        } else {
                            let last = merged.last_mut().unwrap();
                            if s <= last.1 {
                                if e > last.1 {
                                    last.1 = e;
                                }
                            } else {
                                merged.push((s, e));
                            }
                        }
                    }
                    let mut cursor: u64 = 0;
                    for (s, e) in &merged {
                        if *s > cursor {
                            let free_start = cursor;
                            let free_end = *s;
                            let free = free_end.saturating_sub(free_start);
                            let size_h = Self::human_bytes(free);
                            rows.push(format!(
                                            "{:<8} | {:<20} | {:<18} | {:<12} | {:<12} | {:<10} | {:<14} | {:<12} | {:<14}",
                                            "free",
                                            path,
                                            "Free space",
                                            format!("{}", free_start),
                                            format!("{}", free_end),
                                            size_h,
                                            "",
                                            "",
                                            ""
                                        ));
                            meta.push(crate::core::types::ManualPartitionRowMeta {
                                kind: "free".into(),
                                spec_index: None,
                                free_start: Some(free_start),
                                free_end: Some(free_end),
                            });
                        }
                        cursor = (*e).max(cursor);
                    }
                    if cursor < parent_size {
                        let free_start = cursor;
                        let free_end = parent_size;
                        let free = free_end.saturating_sub(free_start);
                        let size_h = Self::human_bytes(free);
                        rows.push(format!(
                                        "{:<8} | {:<20} | {:<18} | {:<12} | {:<12} | {:<10} | {:<14} | {:<12} | {:<14}",
                                        "free",
                                        path,
                                        "Free space",
                                        format!("{}", free_start),
                                        format!("{}", free_end),
                                        size_h,
                                        "",
                                        "",
                                        ""
                                    ));
                        meta.push(crate::core::types::ManualPartitionRowMeta {
                            kind: "free".into(),
                            spec_index: None,
                            free_start: Some(free_start),
                            free_end: Some(free_end),
                        });
                    }
                }
            }
        }
        self.popup_items = rows;
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_open = true;
        // Store meta directly in state for robust indexing
        self.manual_partition_row_meta = meta;
    }

    pub fn open_manual_partition_create(&mut self, free_used_bytes: u64, free_total_bytes: u64) {
        // Ensure we are not in edit mode when starting a fresh create flow
        self.manual_edit_index = None;
        self.popup_kind = Some(super::PopupKind::ManualPartitionKindSelect);
        self.manual_create_free_start_bytes = free_used_bytes;
        self.manual_create_free_end_bytes = free_total_bytes;
        self.manual_create_selected_size_bytes = 0;
        self.manual_create_units_index = 3; // default GiB display
        // First step: Kind selection
        let mut kinds = vec![
            "BOOT".to_string(),
            "SWAP".to_string(),
            "ROOT".to_string(),
            "OTHER".to_string(),
        ];
        for k in &mut kinds {
            let created_exists = self
                .disks_partitions
                .iter()
                .any(|p| p.role.as_deref().unwrap_or("").eq_ignore_ascii_case(k));
            if created_exists {
                *k = format!("{k} (created)");
            }
        }
        self.popup_items = kinds;
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        let default_bytes = free_total_bytes.saturating_sub(free_used_bytes);
        self.custom_input_buffer = format!("{}", default_bytes / (1024 * 1024 * 1024));
        self.popup_open = true;
    }

    pub fn open_manual_partition_size_units(&mut self) {
        self.popup_kind = Some(super::PopupKind::ManualPartitionCreate);
        // Choose sensible defaults per kind when not editing
        // BOOT: 1 GiB, SWAP: 4 GiB, otherwise remaining space
        let remaining_bytes = self
            .manual_create_free_end_bytes
            .saturating_sub(self.manual_create_free_start_bytes);
        if self.manual_edit_index.is_none() {
            let desired_gib: u64 = match self.manual_create_kind_index {
                0 => 1, // BOOT
                1 => 4, // SWAP
                _ => remaining_bytes / (1024 * 1024 * 1024),
            };
            let desired_bytes = desired_gib.saturating_mul(1024 * 1024 * 1024);
            let final_bytes = desired_bytes.min(remaining_bytes);
            self.custom_input_buffer = format!("{}", final_bytes / (1024 * 1024 * 1024));
        } else if self.custom_input_buffer.trim().is_empty() {
            // Fallback: for edit flows that somehow have empty buffer, use remaining
            self.custom_input_buffer = format!("{}", remaining_bytes / (1024 * 1024 * 1024));
        }
        // Clear selected size until confirmed
        self.manual_create_selected_size_bytes = 0;
        self.manual_create_focus_units = false;
        // Default unit to GiB; clamp to 0..=3 (B, KiB/KB, MiB/MB, GiB/GB)
        if self.manual_create_units_index > 3 {
            self.manual_create_units_index = 3;
        }
        self.popup_items = vec![
            "Start        | ".to_string() + &format!("{}", self.manual_create_free_start_bytes),
            "End          | ".to_string() + &format!("{}", self.manual_create_free_end_bytes),
            "Size         | ".to_string() + &self.custom_input_buffer,
            String::from("--UNITS--"),
            String::from("B"),
            String::from("KiB / KB"),
            String::from("MiB / MB"),
            String::from("GiB / GB"),
        ];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0; // not used in this view
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_open = true;
    }

    pub fn open_manual_partition_fs_select(&mut self) {
        self.popup_kind = Some(super::PopupKind::ManualPartitionFilesystem);
        // Build FS options based on kind
        let fs = match self.manual_create_kind_index {
            0 => vec!["fat32", "fat16", "fat12"],
            1 => vec!["linux-swap"],
            2 | 3 => vec![
                "btrfs", "ext2", "ext3", "ext4", "f2fs", "fat12", "fat16", "fat32", "ntfs", "xfs",
            ],
            _ => vec!["ext4"],
        };
        self.manual_create_fs_options = fs.iter().map(|s| s.to_string()).collect();
        self.manual_create_fs_index = 0;
        self.popup_items = self.manual_create_fs_options.clone();
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_open = true;
    }

    pub fn open_manual_partition_mountpoint(&mut self) {
        self.popup_kind = Some(super::PopupKind::ManualPartitionMountpoint);
        self.custom_input_buffer.clear();
        // Defaults based on kind
        self.manual_create_mountpoint = match self.manual_create_kind_index {
            0 => "/boot".into(),
            2 => "/".into(),
            3 => "/mnt".into(),
            _ => String::new(),
        };
        self.custom_input_buffer = self.manual_create_mountpoint.clone();
        self.popup_open = true;
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }
}
