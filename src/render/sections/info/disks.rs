use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::AppState;

pub(super) fn render(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let mode = match app.disks_mode_index {
        0 => "Best-effort partition layout",
        1 => "Manual Partitioning",
        _ => "Pre-mounted configuration",
    };

    // Description header
    let mut desc_lines = vec![Line::from(Span::styled(
        "Description",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];
    desc_lines.push(Line::from("Disk partitioning divides a storage device into independent sections for system management. The root partition (/) holds essential OS files, home (/home) stores user data, boot (/boot) contains files needed to start the system, and swap ([SWAP]) provides virtual memory to supplement RAM, aiding stability and hibernation."));

    if app.disks_mode_index == 1 {
        // Manual Partitioning: vertical split then 50/50 columns
        let vchunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Description "),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(description, vchunks[0]);

        // Split the lower area horizontally
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(vchunks[1]);

        // Left pane: general info
        let mut left_info: Vec<Line> = Vec::new();
        left_info.push(Line::from(format!("Disk mode: {}", mode)));
        if let Some(dev) = &app.disks_selected_device {
            left_info.push(Line::from(format!("Selected drive: {}", dev)));
        }
        if let Some(label) = &app.disks_label {
            left_info.push(Line::from(format!("Label: {}", label)));
        }
        left_info.push(Line::from(format!(
            "Wipe: {}",
            if app.disks_wipe { "Yes" } else { "No" }
        )));
        if let Some(align) = &app.disks_align {
            left_info.push(Line::from(format!("Align: {}", align)));
        }
        let left_block = Paragraph::new(left_info)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(left_block, cols[0]);

        // Right pane: partitions
        let mut right_lines: Vec<String> = Vec::new();
        for p in &app.disks_partitions {
            let role = p.role.clone().unwrap_or_default();
            let fs = p.fs.clone().unwrap_or_default();
            let start = p.start.clone().unwrap_or_default();
            let size = p.size.clone().unwrap_or_default();
            let flags = if p.flags.is_empty() {
                String::new()
            } else {
                p.flags.join(",")
            };
            let mp = p.mountpoint.clone().unwrap_or_default();
            let enc = if p.encrypt.unwrap_or(false) {
                " enc"
            } else {
                ""
            };
            let mut line = String::new();
            if !role.is_empty() {
                line.push_str(&format!("({}) ", role));
            }
            if !fs.is_empty() {
                line.push_str(&format!("{} ", fs));
            }
            if !start.is_empty() || !size.is_empty() {
                line.push_str(&format!("[{}..{}] ", start, size));
            }
            if !flags.is_empty() {
                line.push_str(&format!("flags:{} ", flags));
            }
            if !mp.is_empty() {
                line.push_str(&format!("-> {} ", mp));
            }
            line.push_str(enc);
            if line.is_empty() {
                line = "(empty)".into();
            }
            right_lines.push(line.trim().to_string());
        }

        let mut left_lines: Vec<String> = Vec::new();
        if let Some(dev) = &app.disks_selected_device
            && let Ok(output) = std::process::Command::new("lsblk")
                .args([
                    "-J",
                    "-b",
                    "-o",
                    "NAME,PATH,TYPE,SIZE,FSTYPE,START,PHY-SEC,LOG-SEC",
                ])
                .output()
            && output.status.success()
            && let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout)
            && let Some(blockdevices) = json.get("blockdevices").and_then(|v| v.as_array())
        {
            for devnode in blockdevices {
                let path = devnode.get("path").and_then(|v| v.as_str()).unwrap_or("");
                if !path.starts_with(dev) {
                    continue;
                }
                let sector_size = devnode
                    .get("phy-sec")
                    .and_then(|v| v.as_u64())
                    .or_else(|| devnode.get("log-sec").and_then(|v| v.as_u64()))
                    .unwrap_or(512);
                if let Some(children) = devnode.get("children").and_then(|v| v.as_array()) {
                    for ch in children {
                        let ch_type = ch.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        if ch_type != "part" {
                            continue;
                        }
                        let name = ch.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let size_b = ch.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                        let start_sectors = ch.get("start").and_then(|v| v.as_u64()).unwrap_or(0);
                        let start_b = start_sectors.saturating_mul(sector_size);
                        let end_b = start_b.saturating_add(size_b);
                        let fs = ch.get("fstype").and_then(|v| v.as_str()).unwrap_or("");
                        left_lines.push(format!(
                            "{} {} [{}..{}] {}",
                            name,
                            crate::app::AppState::human_bytes(size_b),
                            start_b,
                            end_b,
                            fs
                        ));
                    }
                }
            }
        }

        let right_block = Block::default().borders(Borders::ALL).title(" Partitions ");
        frame.render_widget(right_block.clone(), cols[1]);
        let right_inner = right_block.inner(cols[1]);
        let mut combined: Vec<String> = Vec::new();
        if !left_lines.is_empty() {
            combined.push("Existing:".into());
            combined.extend(left_lines.into_iter().map(|s| format!("- {}", s)));
        }
        if !right_lines.is_empty() {
            combined.push("Created:".into());
            combined.extend(right_lines.into_iter().map(|s| format!("- {}", s)));
        }
        if combined.is_empty() {
            combined.push("(none)".into());
        }
        let part_p = Paragraph::new(combined.join("\n")).wrap(Wrap { trim: true });
        frame.render_widget(part_p, right_inner);
        return;
    }

    if app.disks_mode_index == 0 {
        let mut info_lines = vec![Line::from(Span::styled(
            "Info",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))];
        let bl = match app.bootloader_index {
            0 => "systemd-boot",
            1 => "GRUB",
            2 => "efistub",
            3 => "Limine",
            _ => "other",
        };
        info_lines.push(Line::from(format!("Bootloader: {}", bl)));
        let fw = if app.is_uefi() { "UEFI" } else { "BIOS" };
        info_lines.push(Line::from(format!("Firmware: {}", fw)));
        if let Some(dev) = &app.disks_selected_device {
            info_lines.push(Line::from(format!("Selected drive: {}", dev)));
        }
        info_lines.push(Line::from("Planned layout:"));
        if app.is_uefi() {
            info_lines.push(Line::from("- gpt: 1024MiB EFI (FAT, ESP) -> /boot"));
            if app.swap_enabled {
                info_lines.push(Line::from("- swap: 4GiB"));
            }
            let enc = if app.disk_encryption_type_index == 1 {
                " (LUKS)"
            } else {
                ""
            };
            info_lines.push(Line::from(format!("- root: btrfs{} (rest)", enc)));
        } else {
            info_lines.push(Line::from("- gpt: 1MiB bios_boot [bios_grub]"));
            if app.swap_enabled {
                info_lines.push(Line::from("- swap: 4GiB"));
            }
            let enc = if app.disk_encryption_type_index == 1 {
                " (LUKS)"
            } else {
                ""
            };
            info_lines.push(Line::from(format!("- root: btrfs{} (rest)", enc)));
            if bl != "GRUB" && bl != "Limine" {
                info_lines.push(Line::from(
                    "Warning: Selected bootloader requires UEFI; choose GRUB or Limine for BIOS.",
                ));
            }
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Description "),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Description "),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(vec![Line::from(Span::styled(
            "Info",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))])
        .block(Block::default().borders(Borders::ALL).title(" Info "))
        .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
    }
}
