use ratatui::Frame;
use ratatui::layout::{Alignment, Rect, Layout, Direction, Constraint};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{AppState, RepoSignOption, RepoSignature, Screen};

pub fn draw_info(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let mut info_lines = vec![Line::from(Span::styled(
        "Info",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];

    if app.current_screen() == Screen::Locales {
        info_lines.push(Line::from(format!(
            "Keyboard: {}",
            app.current_keyboard_layout()
        )));
        info_lines.push(Line::from(format!(
            "Language: {}",
            app.current_locale_language()
        )));
        info_lines.push(Line::from(format!(
            "Encoding: {}",
            app.current_locale_encoding()
        )));

        // Description box shown above Info for Locales
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("Locales are used by glibc and other locale-aware programs or libraries for rendering text, correctly displaying regional monetary values, time and date formats, alphabetic idiosyncrasies, and other locale-specific standards."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(45),
                Constraint::Percentage(55),
            ])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::MirrorsRepos {
        let regions_count = app.mirrors_regions_selected.len();
        let opt_selected: Vec<&str> = app
            .optional_repos_selected
            .iter()
            .filter_map(|&i| app.optional_repos_options.get(i).map(|s| s.as_str()))
            .collect();
        let servers_count = app.mirrors_custom_servers.len();
        let repos_count = app.custom_repos.len();
        info_lines.push(Line::from(format!("Regions selected: {}", regions_count)));
        if regions_count > 0 {
            for &idx in app.mirrors_regions_selected.iter().take(5) {
                if let Some(name) = app.mirrors_regions_options.get(idx) {
                    info_lines.push(Line::from(format!("- {}", name)));
                }
            }
            if regions_count > 5 {
                info_lines.push(Line::from("…"));
            }
        }
        info_lines.push(Line::from(format!(
            "Optional repos: {}",
            if opt_selected.is_empty() {
                "none".into()
            } else {
                opt_selected.join(", ")
            }
        )));
        if servers_count > 0 {
            info_lines.push(Line::from(format!("Custom servers ({}):", servers_count)));
            for s in app.mirrors_custom_servers.iter().take(3) {
                info_lines.push(Line::from(format!("- {}", s)));
            }
            if servers_count > 3 {
                info_lines.push(Line::from("…"));
            }
        } else {
            info_lines.push(Line::from("Custom servers: none"));
        }
        if repos_count > 0 {
            info_lines.push(Line::from("Custom repos:"));
            for repo in app.custom_repos.iter().take(3) {
                let sig = match repo.signature {
                    RepoSignature::Never => "Never",
                    RepoSignature::Optional => "Optional",
                    RepoSignature::Required => "Required",
                };
                let signopt = match repo.sign_option {
                    Some(RepoSignOption::TrustedOnly) => "TrustedOnly",
                    Some(RepoSignOption::TrustedAll) => "TrustedAll",
                    None => "-",
                };
                info_lines.push(Line::from(format!(
                    "{} | {} | {} | {}",
                    repo.name, repo.url, sig, signopt
                )));
            }
            if repos_count > 3 {
                info_lines.push(Line::from("…"));
            }
        } else {
            info_lines.push(Line::from("Custom repos: none"));
        }

        // Description box shown above Info for Mirrors & Repositories (analog to Locales)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("An Arch Linux mirror is a server that copies official repositories, letting users download packages and updates efficiently. Repositories are organized collections of software for installation. Custom mirrors store user-managed package copies, often for local or organizational use, while custom repositories let users or groups provide their own curated package sets."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(45),
                Constraint::Percentage(55),
            ])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::DiskEncryption {
        // Build Info summary
        let enc_type = if app.disk_encryption_type_index == 1 { "LUKS" } else { "None" };
        info_lines.push(Line::from(format!("Type: {}", enc_type)));
        if app.disk_encryption_type_index == 1 {
            let pwd_set = if app.disk_encryption_password.is_empty() { "(not set)" } else { "(set)" };
            let pwd_conf = if app.disk_encryption_password_confirm.is_empty() { "(not set)" } else { "(set)" };
            info_lines.push(Line::from(format!("Password: {}", pwd_set)));
            info_lines.push(Line::from(format!("Confirm: {}", pwd_conf)));
            let part = app
                .disk_encryption_selected_partition
                .clone()
                .unwrap_or_else(|| "(none)".into());
            info_lines.push(Line::from(format!("Partition: {}", part)));
        }

        // Description box above Info
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("Disk encryption protects data by converting the contents of a drive into unreadable code, accessible only with a key or password. This ensures sensitive information remains secure even if the device is lost or stolen, safeguarding system and user data against unauthorized access."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::Disks {
        let mode = match app.disks_mode_index {
            0 => "Best-effort partition layout",
            1 => "Manual Partitioning",
            _ => "Pre-mounted configuration",
        };

        // Description box (shown above Info/Partitions)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("Disk partitioning divides a storage device into independent sections for system management. The root partition (/) holds essential OS files, home (/home) stores user data, boot (/boot) contains files needed to start the system, and swap ([SWAP]) provides virtual memory to supplement RAM, aiding stability and hibernation."));

        // Manual Partitioning: vertical split (Description over Info/Partitions), then horizontal
        if app.disks_mode_index == 1 {
            let vchunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
                .split(area);

            let description = Paragraph::new(desc_lines)
                .block(Block::default().borders(Borders::ALL).title(" Description "))
                .wrap(Wrap { trim: true });
            frame.render_widget(description, vchunks[0]);

            // Split the lower area horizontally
            let cols = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Percentage(50),
                    ratatui::layout::Constraint::Percentage(50),
                ])
                .split(vchunks[1]);

            // Left pane: general info (no partitions)
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

            // Right pane: partitions (existing on left, created on right)
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
                let enc = if p.encrypt.unwrap_or(false) { " enc" } else { "" };
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

            // Build a titled block for partitions and render a single left-aligned list
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
            let part_p = Paragraph::new(combined.join("\n"))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });
            frame.render_widget(part_p, right_inner);
            return;
        } else if app.disks_mode_index == 0 {
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
                // Show selected target device path for best-effort mode
                info_lines.push(Line::from(format!("Selected drive: {}", dev)));
            }
            info_lines.push(Line::from("Planned layout:"));
            if app.is_uefi() {
                info_lines.push(Line::from("- gpt: 1024MiB EFI (FAT, ESP) -> /boot"));
                if app.swap_enabled {
                    info_lines.push(Line::from("- swap: 4GiB"));
                }
                let enc = if app.disk_encryption_type_index == 1 { " (LUKS)" } else { "" };
                info_lines.push(Line::from(format!("- root: btrfs{} (rest)", enc)));
            } else {
                info_lines.push(Line::from("- gpt: 1MiB bios_boot [bios_grub]"));
                if app.swap_enabled {
                    info_lines.push(Line::from("- swap: 4GiB"));
                }
                let enc = if app.disk_encryption_type_index == 1 { " (LUKS)" } else { "" };
                info_lines.push(Line::from(format!("- root: btrfs{} (rest)", enc)));
                if bl != "GRUB" && bl != "Limine" {
                    info_lines.push(Line::from("Warning: Selected bootloader requires UEFI; choose GRUB or Limine for BIOS."));
                }
            }

            // Render Description (top) and Info (bottom)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
                .split(area);

            let description = Paragraph::new(desc_lines)
                .block(Block::default().borders(Borders::ALL).title(" Description "))
                .wrap(Wrap { trim: true });
            frame.render_widget(description, chunks[0]);

            let info = Paragraph::new(info_lines)
                .block(Block::default().borders(Borders::ALL).title(" Info "))
                .wrap(Wrap { trim: true });
            frame.render_widget(info, chunks[1]);
            return;
        } else {
            // Pre-mounted configuration (render description over an empty/default info panel)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
                .split(area);

            let description = Paragraph::new(desc_lines)
                .block(Block::default().borders(Borders::ALL).title(" Description "))
                .wrap(Wrap { trim: true });
            frame.render_widget(description, chunks[0]);

            let info = Paragraph::new(info_lines.clone())
                .block(Block::default().borders(Borders::ALL).title(" Info "))
                .wrap(Wrap { trim: true });
            frame.render_widget(info, chunks[1]);
            return;
        }
    } else if app.current_screen() == Screen::SwapPartition {
        let swap = if app.swap_enabled {
            "Enabled"
        } else {
            "Disabled"
        };
        info_lines.push(Line::from(format!("Swapon: {}", swap)));
        info_lines.push(Line::from(
            "Swapon can be used to activate the swap partition.",
        ));
        info_lines.push(Line::from(
            "If disabled here, the system will be configured with swapoff.",
        ));
        info_lines.push(Line::from(
            "You can always run 'swapon' later to activate the swap partition.",
        ));

        // Description box above Info (analogous to other sections)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("A swap partition in Linux acts as an extension of physical memory (RAM), using disk space to store inactive memory pages when RAM is full. It helps prevent system crashes under heavy load, enables hibernation by saving the RAM state to disk, and allows the operating system to run more applications than would otherwise fit in physical memory."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::Bootloader {
        let bl = match app.bootloader_index {
            0 => "Systemd-boot",
            1 => "Grub",
            2 => "Efistub",
            _ => "Limine",
        };
        info_lines.push(Line::from(format!("Bootloader: {}", bl)));

        // Description box above Info (analogous to Locales)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("A bootloader is a program that starts at system boot, loading the Operating System kernel and initializing hardware. Systemd-boot is simple, UEFI-only, and uses separate text files for each boot entry, making it easy to maintain, but offers minimal features and customization. GRUB2 is more complex, working on both BIOS and UEFI, supporting advanced features, graphical menus, multi-OS setups, and custom scripts, making it ideal for diverse or complex boot needs."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::Hostname {
        if app.hostname_value.is_empty() {
            info_lines.push(Line::from("Hostname: (not set)"));
        } else {
            info_lines.push(Line::from(format!("Hostname: {}", app.hostname_value)));
        }

        // Description box above Info (analogous to Locales)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("A hostname is a unique, human-readable label assigned to a computer on a network, making it easier to identify and connect to devices. Unlike usernames, which specify individuals accessing the system, or root, the administrative account, a hostname names the machine itself for use in local networks or as part of internet addresses, helping organize and distinguish devices within larger environments."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::RootPassword {
        // Build Info summary
        let pwd_set = if app.root_password.is_empty() { "(not set)" } else { "(set)" };
        let conf_set = if app.root_password_confirm.is_empty() { "(not set)" } else { "(set)" };
        info_lines.push(Line::from(format!("Password: {}", pwd_set)));
        info_lines.push(Line::from(format!("Confirm: {}", conf_set)));

        // Description box above Info (analogous to Locales)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("The root password is the secret phrase set for the root (superuser) account on Unix or Linux systems. It grants full administrative control, allowing changes to system files, configurations, and user management. Protecting the root password is crucial because anyone with it has unrestricted access and can impact system security and stability."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::Timezone {
        if app.timezone_value.is_empty() {
            info_lines.push(Line::from("Timezone: (not set)"));
        } else {
            info_lines.push(Line::from(format!("Timezone: {}", app.timezone_value)));
        }

        // Description box above Info (analogous to other sections)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("Timezone determines the local time setting for your Arch Linux system, ensuring logs, scheduled tasks, and timestamps match your region. Setting the correct timezone keeps your system aligned with your geographic location, helping maintain accuracy for daily operations and network interactions."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::UnifiedKernelImages {
        let uki = if app.uki_enabled {
            "Enabled"
        } else {
            "Disabled"
        };
        info_lines.push(Line::from(format!("UKI: {}", uki)));

        // Description box above Info (analogous to Locales)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("A Unified Kernel Image (UKI) is a single UEFI-compatible file containing the Linux kernel, initramfs, boot stub, and extra resources bundled together. It can be booted directly by UEFI firmware or by a bootloader, simplifies configuration and signing for Secure Boot, and ensures all essential boot components are packaged, enabling secure and streamlined Linux startup."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::AutomaticTimeSync {
        info_lines.push(Line::from(format!(
            "Automatic Time Sync: {}",
            if app.ats_enabled { "Yes" } else { "No" }
        )));

        // Description box above Info (analogous to other sections)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("Automatic Time Sync (NTP) keeps your system clock accurate by synchronizing it with internet time servers. This process ensures logs, scheduled tasks, and time-sensitive functions stay reliable and consistent, helping prevent errors caused by time drift on Arch Linux systems."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::Kernels {
        if app.selected_kernels.is_empty() {
            info_lines.push(Line::from("Kernels: none"));
        } else {
            let mut v: Vec<&str> = app.selected_kernels.iter().map(|s| s.as_str()).collect();
            v.sort_unstable();
            info_lines.push(Line::from(format!("Kernels: {}", v.join(", "))));
        }

        // Description box above Info (analogous to other sections)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("Kernels are the core component of Arch Linux, responsible for managing hardware, system resources, and communication between software and hardware. Arch Linux provides several kernel options, including the latest stable, LTS (Long Term Support), and specialized kernels like zen or hardened, each offering different features and performance characteristics. Users can easily install, switch, or maintain multiple kernels via the package manager. Recommended are at least two Kernels to install."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::AdditionalPackages {
        if app.additional_packages.is_empty() {
            info_lines.push(Line::from("Additional packages: none"));
        } else {
            let mut entries: Vec<(String, String)> = app
                .additional_packages
                .iter()
                .map(|p| (p.name.clone(), p.description.clone()))
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            info_lines.push(Line::from(format!("Packages ({}):", entries.len())));
            for (name, desc) in entries.into_iter().take(8) {
                let max_line = (area.width.saturating_sub(4)) as usize;
                let mut line = format!("{} — {}", name, desc);
                if line.len() > max_line {
                    line.truncate(max_line);
                }
                info_lines.push(Line::from(format!("  {}", line)));
            }
            if app.additional_packages.len() > 8 {
                info_lines.push(Line::from("  …"));
            }
        }

        // Description box above Info (analogous to other sections)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("Additional packages let users customize their system by selecting individual software or groups during installation. You can add specific packages, like terminals or text editors, or choose from predefined groups for easier setup. This allows tailoring the installation with preferred tools and utilities beyond the default selection, supporting various use cases and workflows."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(45),
                Constraint::Percentage(55),
            ])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::Audio {
        info_lines.push(Line::from(format!("Audio: {}", app.current_audio_label())));

        // Description box above Info (analogous to other sections)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("Audio lets you choose between PipeWire, a modern multimedia framework with support for audio, video, and low latency, or PulseAudio, a traditional sound server for managing audio streams and devices. You can also opt for no audio server for minimal systems. PipeWire aims to eventually replace PulseAudio with broader features and compatibility."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::NetworkConfiguration {
        info_lines.push(Line::from(format!(
            "Network: {}",
            app.current_network_label()
        )));
        if !app.network_configs.is_empty() {
            info_lines.push(Line::from("Interfaces:"));
            for cfg in &app.network_configs {
                let mode = match cfg.mode {
                    crate::app::NetworkConfigMode::Dhcp => "DHCP",
                    crate::app::NetworkConfigMode::Static => "Static",
                };
                let mut line = format!("- {} ({})", cfg.interface, mode);
                if let Some(ip) = &cfg.ip_cidr {
                    line.push_str(&format!(" IP={} ", ip));
                }
                if let Some(gw) = &cfg.gateway {
                    line.push_str(&format!(" GW={} ", gw));
                }
                if let Some(dns) = &cfg.dns {
                    line.push_str(&format!(" DNS={} ", dns));
                }
                info_lines.push(Line::from(line));
            }
        }

        // Description box above Info (analogous to other sections)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("Network configuration in Arch Linux sets up wired or wireless connections, assigns IP addresses (dynamic by DHCP or manually), and manages DNS. Tools like systemd-networkd, NetworkManager, or iproute2 can be used, but only one network manager should be active at a time to avoid conflicts."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::UserAccount {
        if app.users.is_empty() {
            info_lines.push(Line::from("No users added yet."));
        } else {
            info_lines.push(Line::from("Users:"));
            for u in app.users.iter().take(5) {
                let sudo = if u.is_sudo { "sudo" } else { "user" };
                info_lines.push(Line::from(format!("- {} ({})", u.username, sudo)));
            }
            if app.users.len() > 5 {
                info_lines.push(Line::from("…"));
            }
        }

        // Description box above Info (analogous to Locales)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("A user account in Arch Linux is an identity used to log in and interact with the system, providing access to files, directories, and system resources based on permissions. Each user has a unique username, a home directory, and configurable group memberships for access control. User information is stored in /etc/passwd, while passwords are managed securely in /etc/shadow."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::ExperienceMode {
        info_lines.push(Line::from(format!(
            "Experience: {}",
            app.current_experience_mode_label()
        )));
        if app.experience_mode_index == 2 {
            if !app.selected_server_types.is_empty() {
                let mut names: Vec<&str> = app
                    .selected_server_types
                    .iter()
                    .map(|s| s.as_str())
                    .collect();
                names.sort_unstable();
                info_lines.push(Line::from(format!("Selected: {}", names.join(", "))));
                info_lines.push(Line::from("Installed packages:"));
                for &server in names.iter() {
                    if let Some(set) = app.selected_server_packages.get(server) {
                        let mut pkgs: Vec<&str> = set.iter().map(|s| s.as_str()).collect();
                        pkgs.sort_unstable();
                        let available_width = area.width.saturating_sub(4) as usize;
                        let max_name_len = pkgs.iter().map(|s| s.len()).max().unwrap_or(0).min(32);
                        let col_width = (max_name_len + 2).max(6);
                        let mut num_cols = if col_width == 0 {
                            1
                        } else {
                            available_width / col_width
                        };
                        if num_cols == 0 {
                            num_cols = 1;
                        }
                        num_cols = num_cols.min(4);
                        let count = pkgs.len();
                        info_lines.push(Line::from(format!("- {} ({}):", server, count)));
                        let rows = count.div_ceil(num_cols);
                        for r in 0..rows {
                            let mut line = String::new();
                            for c in 0..num_cols {
                                let idx = r + c * rows;
                                if idx < count {
                                    let name = pkgs[idx];
                                    let shown = if name.len() > max_name_len {
                                        &name[..max_name_len]
                                    } else {
                                        name
                                    };
                                    line.push_str(shown);
                                    if c + 1 < num_cols {
                                        let pad = col_width.saturating_sub(shown.len());
                                        line.push_str(&" ".repeat(pad));
                                    }
                                }
                            }
                            info_lines.push(Line::from(format!("  {}", line)));
                        }
                    } else {
                        info_lines.push(Line::from(format!("- {}: default packages", server)));
                    }
                }
            } else {
                info_lines.push(Line::from("Selected: none"));
            }
        } else if app.experience_mode_index == 3 {
            if !app.selected_xorg_types.is_empty() {
                let mut names: Vec<&str> =
                    app.selected_xorg_types.iter().map(|s| s.as_str()).collect();
                names.sort_unstable();
                info_lines.push(Line::from(format!("Selected: {}", names.join(", "))));
                info_lines.push(Line::from("Installed packages:"));
                for &xorg in names.iter() {
                    if let Some(set) = app.selected_xorg_packages.get(xorg) {
                        let mut pkgs: Vec<&str> = set.iter().map(|s| s.as_str()).collect();
                        pkgs.sort_unstable();
                        let available_width = area.width.saturating_sub(4) as usize;
                        let max_name_len = pkgs.iter().map(|s| s.len()).max().unwrap_or(0).min(32);
                        let col_width = (max_name_len + 2).max(6);
                        let mut num_cols = if col_width == 0 {
                            1
                        } else {
                            available_width / col_width
                        };
                        if num_cols == 0 {
                            num_cols = 1;
                        }
                        num_cols = num_cols.min(4);
                        let count = pkgs.len();
                        info_lines.push(Line::from(format!("- {} ({}):", xorg, count)));
                        let rows = count.div_ceil(num_cols);
                        for r in 0..rows {
                            let mut line = String::new();
                            for c in 0..num_cols {
                                let idx = r + c * rows;
                                if idx < count {
                                    let name = pkgs[idx];
                                    let shown = if name.len() > max_name_len {
                                        &name[..max_name_len]
                                    } else {
                                        name
                                    };
                                    line.push_str(shown);
                                    if c + 1 < num_cols {
                                        let pad = col_width.saturating_sub(shown.len());
                                        line.push_str(&" ".repeat(pad));
                                    }
                                }
                            }
                            info_lines.push(Line::from(format!("  {}", line)));
                        }
                    } else {
                        info_lines.push(Line::from(format!("- {}: default packages", xorg)));
                    }
                }
            } else {
                info_lines.push(Line::from("Selected: none"));
            }
        } else if !app.selected_desktop_envs.is_empty() {
            let mut names: Vec<&str> = app
                .selected_desktop_envs
                .iter()
                .map(|s| s.as_str())
                .collect();
            names.sort_unstable();
            info_lines.push(Line::from(format!("Selected: {}", names.join(", "))));

            let default_for_env = |env: &str| -> Option<&'static str> {
                match env {
                    "GNOME" => Some("gdm"),
                    "KDE Plasma" | "Hyprland" | "Cutefish" | "Lxqt" => Some("sddm"),
                    "Budgie" => Some("lightdm-slick-greeter"),
                    "Bspwm" | "Cinnamon" | "Deepin" | "Enlightenment" | "Mate" | "Qtile"
                    | "Sway" | "Xfce4" | "i3-wm" => Some("lightdm-gtk-greeter"),
                    _ => None,
                }
            };
            let effective_lm: String = if app.login_manager_user_set {
                app.selected_login_manager
                    .clone()
                    .unwrap_or_else(|| "none".into())
            } else if let Some(man) = app.selected_login_manager.clone() {
                man
            } else if let Some(first_env) = names.first() {
                default_for_env(first_env).unwrap_or("none").into()
            } else {
                "none".into()
            };
            info_lines.push(Line::from(format!("Login Manager: {}", effective_lm)));

            info_lines.push(Line::from("Packages:"));
            for &env in names.iter() {
                let mut pkgs: Vec<String> = if let Some(set) = app.selected_env_packages.get(env) {
                    set.iter().cloned().collect()
                } else {
                    match env {
                        "Awesome" => vec![
                            "alacritty",
                            "awesome",
                            "feh",
                            "gnu-free-fonts",
                            "slock",
                            "terminus-font",
                            "ttf-liberation",
                            "xorg-server",
                            "xorg-xinit",
                            "xorg-xrandr",
                            "xsel",
                            "xterm",
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                        "Bspwm" => vec!["bspwm", "dmenu", "rxvt-unicode", "sxhkd", "xdo"]
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "Budgie" => vec![
                            "arc-gtk-theme",
                            "budgie",
                            "mate-terminal",
                            "nemo",
                            "papirus-icon-theme",
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                        "Cinnamon" => vec![
                            "blueman",
                            "blue-utils",
                            "cinnamon",
                            "engrampa",
                            "gnome-keyring",
                            "gnome-screenshot",
                            "gnome-terminal",
                            "gvfs-smb",
                            "system-config-printer",
                            "xdg-user-dirs-gtk",
                            "xed",
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                        "Cutefish" => vec!["cutefish", "noto-fonts"]
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "Deepin" => vec!["deepin", "deepin-editor", "deepin-terminal"]
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "Enlightenment" => vec!["enlightenment", "terminology"]
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "GNOME" => vec!["gnome", "gnome-tweaks"]
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "Hyprland" => vec![
                            "dolphin",
                            "dunst",
                            "grim",
                            "hyprland",
                            "kitty",
                            "polkit-kde-agent",
                            "qt5-wayland",
                            "qt6-wayland",
                            "slurp",
                            "wofi",
                            "xdg-desktop-portal-hyprland",
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                        "KDE Plasma" => vec![
                            "ark",
                            "dolphin",
                            "kate",
                            "konsole",
                            "plasma-meta",
                            "plasma-workspace",
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                        "Lxqt" => vec![
                            "breeze-icons",
                            "leafpad",
                            "oxygen-icons",
                            "slock",
                            "ttf-freefont",
                            "xdg-utils",
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                        "Mate" => vec!["mate", "mate-extra"]
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "Qtile" => vec!["alacritty", "qtile"]
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "Sway" => vec![
                            "brightnessctl",
                            "dmenu",
                            "foot",
                            "grim",
                            "pavucontrol",
                            "slurp",
                            "sway",
                            "swaybg",
                            "swayidle",
                            "swaylock",
                            "waybar",
                            "xorg-xwayland",
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                        "Xfce4" => {
                            vec!["gvfs", "pavucontrol", "xarchiver", "xfce4", "xfce4-goodies"]
                                .into_iter()
                                .map(|s| s.to_string())
                                .collect()
                        }
                        "i3-wm" => vec![
                            "dmenu",
                            "i3-wm",
                            "i3blocks",
                            "i3lock",
                            "i3status",
                            "lightdm",
                            "lightdm-gtk-greeter",
                            "xss-lock",
                            "xterm",
                        ]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                        _ => Vec::new(),
                    }
                };
                pkgs.sort_unstable();
                let count = pkgs.len();
                info_lines.push(Line::from(format!("- {} ({}):", env, count)));

                let available_width = area.width.saturating_sub(4) as usize;
                let max_name_len = pkgs.iter().map(|s| s.len()).max().unwrap_or(0).min(32);
                let col_width = (max_name_len + 2).max(6);
                let mut num_cols = if col_width == 0 {
                    1
                } else {
                    available_width / col_width
                };
                if num_cols == 0 {
                    num_cols = 1;
                }
                num_cols = num_cols.min(4);
                let rows = count.div_ceil(num_cols);
                for r in 0..rows {
                    let mut line = String::new();
                    for c in 0..num_cols {
                        let idx = r + c * rows;
                        if idx < count {
                            let name = &pkgs[idx];
                            let shown = if name.len() > max_name_len {
                                &name[..max_name_len]
                            } else {
                                name
                            };
                            line.push_str(shown);
                            if c + 1 < num_cols {
                                let pad = col_width.saturating_sub(shown.len());
                                line.push_str(&" ".repeat(pad));
                            }
                        }
                    }
                    info_lines.push(Line::from(format!("  {}", line)));
                }
            }
        } else {
            info_lines.push(Line::from("Selected: none"));
            info_lines.push(Line::from("Login Manager: none"));
        }

        // Description box above Info for Experience Mode (analogous to others)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("Experience Mode allows selecting a predefined system role during installation, such as Desktop, Minimal, Server, or Xorg. Each mode determines which desktop environments, window managers, drivers, and login managers are installed, tailoring the system for general use, lightweight setups, server purposes, or just graphical infrastructure."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else if app.current_screen() == Screen::SaveConfiguration {
        // Build minimal Info summary (can be extended later with config-related details)
        if !app.last_load_missing_sections.is_empty() {
            info_lines.push(Line::from("Last load missing sections:"));
            for s in app.last_load_missing_sections.iter().take(6) {
                info_lines.push(Line::from(format!("- {}", s)));
            }
            if app.last_load_missing_sections.len() > 6 {
                info_lines.push(Line::from("…"));
            }
        } else {
            info_lines.push(Line::from("No previous load warnings."));
        }

        // Description box above Info (analogous to other sections)
        let mut desc_lines = vec![Line::from(Span::styled(
            "Description",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))];
        desc_lines.push(Line::from("Configuration lets you save or load your installation setup for Arch Linux, making it easy to reuse or share selections across multiple installs. When you load a configuration, sensitive data like passwords and certain custom settings need to be re-entered, ensuring security while streamlining system setup and consistency."));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let description = Paragraph::new(desc_lines)
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, chunks[0]);

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
        return;
    } else {
        info_lines.push(Line::from(format!(
            "Selected: {}",
            app.menu_entries[app.selected_index].label
        )));
    }

    let info = Paragraph::new(info_lines)
        .block(Block::default().borders(Borders::ALL).title(" Info "))
        .wrap(Wrap { trim: true });
    frame.render_widget(info, area);
}
