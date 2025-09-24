use crate::app::{AppState, PopupKind};
use crate::core::services::fstab::FstabService;
use crate::core::services::mounting::MountingService;
use crate::core::services::partitioning::PartitioningService;
use crate::core::services::sysconfig::SysConfigService;
use crate::core::services::system::SystemService;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;

impl AppState {
    pub fn start_install(&mut self) {
        // Prevent starting a second install while one is already running
        if self.install_running {
            self.debug_log("start_install: rejected, install already running");
            self.open_info_popup("Installation already running. Please wait...".into());
            return;
        }
        // Validate required sections before proceeding
        if let Some(msg) = self.validate_install_requirements() {
            self.debug_log("start_install: rejected, requirements not met");
            self.open_info_popup(msg);
            return;
        }
        self.debug_log("start_install: starting install flow");
        self.start_install_flow();
    }

    pub fn start_install_flow(&mut self) {
        let Some(target) = self.select_target_and_run_prechecks() else {
            self.debug_log("start_install_flow: target selection/prechecks returned None");
            return;
        };
        let sections = self.build_install_sections(&target);
        self.debug_log(&format!(
            "start_install_flow: target='{}' sections={}",
            target,
            sections.len()
        ));
        if self.dry_run {
            // Simulate the same install UI, but do not execute commands.
            // Stream the plan into the live log with a Dry-Run label.
            self.install_running = true;
            self.install_section_titles = sections.iter().map(|(t, _)| t.clone()).collect();
            self.install_section_done = vec![false; self.install_section_titles.len()];
            self.install_current_section = None;

            let (tx, rx) = std::sync::mpsc::channel::<String>();
            self.install_log_rx = Some(rx);

            // Resolve absolute log path up-front
            let log_path_buf = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("dry-run.log");
            let log_path_display = log_path_buf.to_string_lossy().to_string();
            self.debug_log(&format!(
                "dry-run: will write log to {} and stream {} sections",
                log_path_display,
                self.install_section_titles.len()
            ));

            let debug_enabled = self.debug_enabled;
            thread::spawn(move || {
                let debug_tag = "dry-run thread";
                let mut log_file: Option<std::fs::File> = match std::fs::File::create(&log_path_buf)
                {
                    Ok(f) => Some(f),
                    Err(e) => {
                        let _ = tx.send(format!(
                            "WARN: Could not create log file at {}: {}",
                            log_path_display, e
                        ));
                        None
                    }
                };
                let send = |tx: &std::sync::mpsc::Sender<String>, s: String| {
                    let _ = tx.send(s);
                };
                let write_log = |file: &mut Option<std::fs::File>, line: &str| {
                    if let Some(f) = file.as_mut() {
                        use std::io::Write as _;
                        let _ = writeln!(f, "{}", line);
                    }
                };
                let dbg = |msg: &str| {
                    if debug_enabled {
                        let now = chrono::Local::now();
                        let ts = now.format("%Y-%m-%d %H:%M:%S");
                        let _ = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("debug.log")
                            .and_then(|mut f| {
                                use std::io::Write;
                                writeln!(f, "[DEBUG {}] {}: {}", ts, debug_tag, msg)
                            });
                    }
                };

                dbg(&format!(
                    "log path created={}, sections to stream={}",
                    log_path_display,
                    /* sections len not moved here, will infer from loop */ "unknown"
                ));

                let start_msg = "Starting dry-run (no commands will be executed)...".to_string();
                write_log(&mut log_file, &start_msg);
                send(&tx, start_msg);
                std::thread::sleep(std::time::Duration::from_millis(15));
                let path_msg = format!("Logging dry-run output to: {}", log_path_display);
                write_log(&mut log_file, &path_msg);
                send(&tx, path_msg);
                std::thread::sleep(std::time::Duration::from_millis(15));
                for (title, cmds) in sections.into_iter() {
                    dbg(&format!("section_start: '{}' ({} cmds)", title, cmds.len()));
                    let marker = format!("::section_start::{}", title);
                    write_log(&mut log_file, &marker);
                    send(&tx, marker);
                    std::thread::sleep(std::time::Duration::from_millis(25));
                    let header = format!("=== {} ===", title);
                    write_log(&mut log_file, &header);
                    send(&tx, header);
                    std::thread::sleep(std::time::Duration::from_millis(25));
                    for c in cmds {
                        let line = format!(
                            "[Dry-Run] $ {}",
                            crate::common::utils::redact_command_for_logging(&c)
                        );
                        write_log(&mut log_file, &line);
                        send(&tx, line);
                        std::thread::sleep(std::time::Duration::from_millis(8));
                    }
                    let done = format!("::section_done::{}", title);
                    dbg(&format!("section_done: '{}'", title));
                    write_log(&mut log_file, &done);
                    send(&tx, done);
                    write_log(&mut log_file, "");
                    send(&tx, String::new());
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                let complete = "Dry-run completed.".to_string();
                write_log(&mut log_file, &complete);
                send(&tx, complete);
                dbg("thread exiting normally");
            });
            return;
        }
        // Launch background installer and keep TUI running with live logs
        self.install_running = true;
        self.install_section_titles = sections.iter().map(|(t, _)| t.clone()).collect();
        self.install_section_done = vec![false; self.install_section_titles.len()];
        self.install_current_section = None;

        let (tx, rx) = std::sync::mpsc::channel::<String>();
        self.install_log_rx = Some(rx);

        let debug_enabled = self.debug_enabled;
        self.debug_log(&format!(
            "install thread: spawning (sections={})",
            self.install_section_titles.len()
        ));
        thread::spawn(move || {
            let debug_tag = "install thread";
            let mut any_error: Option<String> = None;
            let dbg = |msg: &str| {
                if debug_enabled {
                    let now = chrono::Local::now();
                    let ts = now.format("%Y-%m-%d %H:%M:%S");
                    let _ = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("debug.log")
                        .and_then(|mut f| {
                            use std::io::Write;
                            writeln!(f, "[DEBUG {}] {}: {}", ts, debug_tag, msg)
                        });
                }
            };
            let thread_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let send = |tx: &std::sync::mpsc::Sender<String>, s: String| {
                    let _ = tx.send(s);
                };

                send(&tx, "Starting installation...".to_string());
                dbg(
                    "env: TERM=dumb NO_COLOR=1 PACMAN_COLOR=never SYSTEMD_PAGER=cat SYSTEMD_COLORS=0 PAGER=cat LESS=FRX",
                );
                let thread_panicked = false;
                'outer: for (title, cmds) in sections.into_iter() {
                    dbg(&format!("section_start: '{}' ({} cmds)", title, cmds.len()));
                    send(&tx, format!("::section_start::{}", title));
                    send(&tx, format!("=== {} ===", title));
                    for c in cmds {
                        let red = crate::common::utils::redact_command_for_logging(&c);
                        send(&tx, format!("$ {}", red));
                        dbg(&format!("spawn: '{}'", red));
                        let pipeline = format!("stdbuf -oL -eL {} 2>&1", c);
                        let mut child = match Command::new("bash")
                            .arg("-lc")
                            .arg(&pipeline)
                            .env("TERM", "dumb")
                            .env("NO_COLOR", "1")
                            .env("PACMAN_COLOR", "never")
                            .env("SYSTEMD_PAGER", "cat")
                            .env("SYSTEMD_COLORS", "0")
                            .env("PAGER", "cat")
                            .env("LESS", "FRX")
                            .env("PACMAN", "pacman --noconfirm --noprogressbar --color never")
                            .stdin(Stdio::null())
                            .stdout(Stdio::piped())
                            .spawn()
                        {
                            Ok(ch) => ch,
                            Err(e) => {
                                any_error = Some(format!("Failed to spawn: {} ({})", red, e));
                                send(&tx, any_error.as_ref().unwrap().clone());
                                dbg(&format!("failed to spawn: {} ({})", red, e));
                                break 'outer;
                            }
                        };
                        if let Some(stdout) = child.stdout.take() {
                            let reader = BufReader::new(stdout);
                            for line in reader.lines() {
                                match line {
                                    Ok(l) => {
                                        let clean =
                                            crate::common::utils::sanitize_terminal_output_line(&l);
                                        if !clean.is_empty() {
                                            send(&tx, clean);
                                        }
                                    }
                                    Err(e) => {
                                        dbg(&format!("error reading child stdout: {}", e));
                                        break;
                                    }
                                }
                            }
                        } else {
                            dbg("stdout piping unavailable (child.stdout None)");
                        }
                        match child.wait() {
                            Ok(st) if st.success() => {}
                            Ok(st) => {
                                let code = st.code().unwrap_or(-1);
                                any_error =
                                    Some(format!("Command failed (exit {}): {}", code, red));
                                send(&tx, any_error.as_ref().unwrap().clone());
                                dbg(&format!("command failed (exit {}): {}", code, red));
                                break 'outer;
                            }
                            Err(e) => {
                                any_error = Some(format!("Failed to wait: {} ({})", red, e));
                                send(&tx, any_error.as_ref().unwrap().clone());
                                dbg(&format!("failed to wait: {} ({})", red, e));
                                break 'outer;
                            }
                        }
                    }
                    dbg(&format!("section_done: '{}'", title));
                    send(&tx, format!("::section_done::{}", title));
                    send(&tx, String::new());
                }
                if any_error.is_none() {
                    send(&tx, "Installation completed.".to_string());
                }
                thread_panicked
            }));
            match thread_result {
                Ok(_) => dbg("thread exiting normally"),
                Err(e) => dbg(&format!("thread panicked: {:?}", e)),
            }
        });
    }

    fn select_target_and_run_prechecks(&mut self) -> Option<String> {
        let target = match &self.disks_selected_device {
            Some(p) => p.clone(),
            None => {
                self.debug_log("select_target_and_run_prechecks: no target selected");
                self.open_info_popup("No target disk selected.".into());
                return None;
            }
        };
        self.debug_log(&format!(
            "select_target_and_run_prechecks: target='{}'",
            target
        ));

        if self.disk_has_mounted_partitions(&target) {
            self.debug_log(&format!(
                "select_target_and_run_prechecks: mounted partitions detected on {}",
                target
            ));
            self.open_info_popup(format!(
                "Device {} has mounted partitions. Unmount before proceeding.",
                target
            ));
            return None;
        }

        if !self.disks_wipe && self.disk_freespace_low(&target) {
            self.debug_log(
                "select_target_and_run_prechecks: low free space and wipe disabled -> opening WipeConfirm",
            );
            self.popup_kind = Some(PopupKind::WipeConfirm);
            self.popup_open = true;
            self.popup_items = vec!["Yes, wipe the device".into(), "No, cancel".into()];
            self.popup_visible_indices = (0..self.popup_items.len()).collect();
            self.popup_selected_visible = 1; // default to No
            self.popup_in_search = false;
            self.popup_search_query.clear();
            return None;
        }

        Some(target)
    }

    fn validate_install_requirements(&self) -> Option<String> {
        let mut issues: Vec<String> = Vec::new();

        // Locales: keyboard, language, encoding must not be <none>
        let kb = self.current_keyboard_layout();
        let lang = self.current_locale_language();
        let enc = self.current_locale_encoding();
        if kb == "<none>" || lang == "<none>" || enc == "<none>" {
            issues.push("Locales are not fully set (keyboard, language, encoding).".into());
        }

        // Region (mirrors) must be selected
        if self.mirrors_regions_selected.is_empty() {
            issues.push("Region is not selected (Mirrors & Repositories).".into());
        }

        // Disk partitioning target device must be selected
        if self.disks_selected_device.is_none() {
            issues.push("Disk partitioning: target device is not selected.".into());
        }

        // Hostname must be non-empty
        if self.hostname_value.trim().is_empty() {
            issues.push("Hostname is not set.".into());
        }

        // Root password must be provided and confirmed
        if self.root_password.trim().is_empty() || self.root_password_confirm.trim().is_empty() {
            issues.push("Root password is not set.".into());
        } else if self.root_password != self.root_password_confirm {
            issues.push("Root passwords do not match.".into());
        }

        // At least one user must be configured
        if self.users.is_empty() {
            issues.push("At least one user account must be added.".into());
        }

        // Timezone must be non-empty
        if self.timezone_value.trim().is_empty() {
            issues.push("Timezone is not set.".into());
        }

        if issues.is_empty() {
            self.debug_log("validate_install_requirements: ok (no issues)");
            None
        } else {
            self.debug_log(&format!(
                "validate_install_requirements: issues_found={}",
                issues.len()
            ));
            Some(format!(
                "Please complete the following before installing:\n- {}",
                issues.join("\n- ")
            ))
        }
    }

    fn build_install_sections(&self, target: &str) -> Vec<(String, Vec<String>)> {
        let mut sections: Vec<(String, Vec<String>)> = Vec::new();
        let locales = self.build_locales_plan();
        if !locales.is_empty() {
            sections.push(("Locales".into(), locales));
        }
        let mirrors = self.build_mirrors_plan();
        if !mirrors.is_empty() {
            sections.push(("Mirrors & Repos".into(), mirrors));
        }
        // Pre-cleanup to avoid device busy if re-running installer or previous mounts exist
        sections.push((
            "Pre-cleanup".into(),
            vec![
                // Try to disable any swap using the target disk
                format!("swapoff -a || true"),
                // Unmount any mounts under /mnt from a previous attempt
                "umount -R /mnt 2>/dev/null || true".into(),
                // Settle devices
                "udevadm settle || true".into(),
            ],
        ));
        sections.push((
            "Partitioning".into(),
            PartitioningService::build_plan(self, target).commands,
        ));
        sections.push((
            "Mounting".into(),
            MountingService::build_plan(self, target).commands,
        ));
        sections.push((
            "System pre-install".into(),
            SystemService::build_pre_install_plan(self).commands,
        ));
        sections.push((
            "System package installations".into(),
            SystemService::build_pacstrap_plan(self).commands,
        ));
        sections.push((
            "fstab and checks".into(),
            FstabService::build_checks_and_fstab(self, target).commands,
        ));
        sections.push((
            "System configuration".into(),
            SysConfigService::build_plan(self).commands,
        ));
        sections.push((
            "Bootloader setup".into(),
            crate::core::services::bootloader::BootloaderService::build_plan(self, target).commands,
        ));
        sections.push((
            "User setup".into(),
            crate::core::services::usersetup::UserSetupService::build_plan(self).commands,
        ));
        // Log assembled sections summary
        let summary: String = sections
            .iter()
            .map(|(name, cmds)| format!("{}({})", name, cmds.len()))
            .collect::<Vec<_>>()
            .join(", ");
        self.debug_log(&format!(
            "build_install_sections: count={} [{}]",
            sections.len(),
            summary
        ));
        sections
    }

    // Background thread streams logs into AppState via mpsc

    fn disk_has_mounted_partitions(&self, dev: &str) -> bool {
        let output = std::process::Command::new("lsblk")
            .args(["-J", "-o", "PATH,MOUNTPOINT"])
            .output();
        if let Ok(out) = output
            && out.status.success()
            && let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout)
            && let Some(arr) = json.get("blockdevices").and_then(|v| v.as_array())
        {
            for d in arr {
                let path = d.get("path").and_then(|v| v.as_str()).unwrap_or("");
                if !path.starts_with(dev) {
                    continue;
                }
                if let Some(children) = d.get("children").and_then(|v| v.as_array()) {
                    for ch in children {
                        let p = ch.get("path").and_then(|v| v.as_str()).unwrap_or("");
                        let mp = ch.get("mountpoint").and_then(|v| v.as_str()).unwrap_or("");
                        if p.starts_with(dev) && !mp.is_empty() {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn disk_freespace_low(&self, dev: &str) -> bool {
        if let Some(found) = self.disks_devices.iter().find(|d| d.path == dev)
            && !found.freespace.is_empty()
            && found.freespace != "-"
        {
            let s = found.freespace.to_lowercase();
            let is_low = s.ends_with(" mib") || s.ends_with(" kib") || s.ends_with(" b");
            return is_low;
        }
        false
    }

    pub(crate) fn is_uefi(&self) -> bool {
        std::path::Path::new("/sys/firmware/efi").exists()
    }

    fn build_locales_plan(&self) -> Vec<String> {
        // TODO(v0.2.0): Implement locales pre-install steps as needed.
        Vec::new()
    }
    fn build_mirrors_plan(&self) -> Vec<String> {
        let mut cmds: Vec<String> = Vec::new();

        // Ensure host and target mirrorlist directories exist
        cmds.push("install -d /etc/pacman.d".into());
        cmds.push("install -d /mnt/etc/pacman.d".into());

        // Collect selected regions (as shown to user) and try to extract country codes (2-letter)
        let mut country_args: Vec<String> = Vec::new();
        for &idx in self.mirrors_regions_selected.iter() {
            if let Some(line) = self.mirrors_regions_options.get(idx) {
                // Try to find a 2-letter uppercase token (country code)
                let mut code: Option<String> = None;
                for tok in line.split_whitespace() {
                    if tok.len() == 2 && tok.chars().all(|c| c.is_ascii_uppercase()) {
                        code = Some(tok.to_string());
                        break;
                    }
                }
                if let Some(c) = code {
                    country_args.push(format!("-c \"{}\"", c));
                } else {
                    // Fallback: use the line up to the first double space as the country name
                    let name = line.split("  ").next().unwrap_or(line).trim().to_string();
                    if !name.is_empty() {
                        country_args.push(format!("-c \"{}\"", name.replace('"', "\\\"")));
                    }
                }
            }
        }

        let has_regions = !country_args.is_empty();
        let has_custom_servers = !self.mirrors_custom_servers.is_empty();

        // If the user added custom servers, place them at the top of mirrorlist (host and target)
        if has_custom_servers {
            // Write custom servers header
            let mut printf_cmd_host = String::from("printf '%s\\n' ");
            let mut printf_cmd_target = String::from("printf '%s\\n' ");
            let mut first = true;
            for url in &self.mirrors_custom_servers {
                let mut line = String::from("Server = ");
                line.push_str(url);
                // Simple quote escape for rare cases
                let safe = line.replace('\'', "'\\\''");
                if !first {
                    printf_cmd_host.push(' ');
                    printf_cmd_target.push(' ');
                }
                first = false;
                printf_cmd_host.push_str(&format!("'{}'", safe));
                printf_cmd_target.push_str(&format!("'{}'", safe));
            }
            printf_cmd_host.push_str(" > /etc/pacman.d/mirrorlist");
            printf_cmd_target.push_str(" > /mnt/etc/pacman.d/mirrorlist");
            cmds.push(printf_cmd_host);
            cmds.push(printf_cmd_target);
        }

        if has_regions {
            // Use reflector to fetch and sort HTTPS mirrors for selected regions
            // Prefer the latest mirrors and sort by rate
            // Add --verbose so users can see detailed output during installation
            let mut reflector_cmd =
                String::from("reflector --verbose --protocol https --latest 20 --sort rate ");
            reflector_cmd.push_str(&country_args.join(" "));
            if has_custom_servers {
                // Save to tmp then append after custom servers
                // Generate for host, append to both host and target
                let mut refl_host = reflector_cmd.clone();
                refl_host.push_str(" --save /etc/pacman.d/mirrorlist.ai.tmp");
                cmds.push(refl_host);
                cmds.push("cat /etc/pacman.d/mirrorlist.ai.tmp >> /etc/pacman.d/mirrorlist".into());
                cmds.push(
                    "cat /etc/pacman.d/mirrorlist.ai.tmp >> /mnt/etc/pacman.d/mirrorlist".into(),
                );
                cmds.push("rm -f /etc/pacman.d/mirrorlist.ai.tmp".into());
            } else {
                // Save directly on host, then copy to target
                reflector_cmd.push_str(" --save /etc/pacman.d/mirrorlist");
                cmds.push(reflector_cmd);
                cmds.push(
                    "install -Dm644 /etc/pacman.d/mirrorlist /mnt/etc/pacman.d/mirrorlist".into(),
                );
            }
        } else if !has_custom_servers {
            // Neither regions nor custom servers selected: best effort copy from ISO host
            cmds.push("test -f /mnt/etc/pacman.d/mirrorlist || install -Dm644 /etc/pacman.d/mirrorlist /mnt/etc/pacman.d/mirrorlist".into());
        }

        cmds
    }
}
