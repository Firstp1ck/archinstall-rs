use crate::app::{AppState, PopupKind};
use crate::common::utils::redact_command_for_logging;
use crate::core::services::fstab::FstabService;
use crate::core::services::mounting::MountingService;
use crate::core::services::partitioning::PartitioningService;
use crate::core::services::sysconfig::SysConfigService;
use crate::core::services::system::SystemService;

impl AppState {
    pub fn start_install(&mut self) {
        self.start_install_flow();
    }

    pub fn start_install_flow(&mut self) {
        let Some(target) = self.select_target_and_run_prechecks() else { return; };
        let sections = self.build_install_sections(&target);
        if self.dry_run {
            let mut body_lines: Vec<String> = Vec::new();
            for (title, cmds) in sections {
                body_lines.push(format!("=== {} ===", title));
                for c in cmds {
                    body_lines.push(redact_command_for_logging(&c));
                }
                body_lines.push(String::new());
            }
            self.open_info_popup(body_lines.join("\n"));
            return;
        }
        // Request to exit TUI and run install in stdout mode
        self.exit_tui_after_install = true;
        self.pending_install_sections = Some(sections);
    }

    fn select_target_and_run_prechecks(&mut self) -> Option<String> {
        let target = match &self.disks_selected_device {
            Some(p) => p.clone(),
            None => {
                self.open_info_popup("No target disk selected.".into());
                return None;
            }
        };

        if self.disk_has_mounted_partitions(&target) {
            self.open_info_popup(format!(
                "Device {} has mounted partitions. Unmount before proceeding.",
                target
            ));
            return None;
        }

        if !self.disks_wipe && self.disk_freespace_low(&target) {
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

    fn build_install_sections(&self, target: &str) -> Vec<(String, Vec<String>)> {
        let mut sections: Vec<(String, Vec<String>)> = Vec::new();
        let locales = self.build_locales_plan();
        if !locales.is_empty() { sections.push(("Locales".into(), locales)); }
        let mirrors = self.build_mirrors_plan();
        if !mirrors.is_empty() { sections.push(("Mirrors & Repos".into(), mirrors)); }
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
            "System installation (pacstrap)".into(),
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
        sections
    }

    // No background thread when exiting TUI; runner will execute pending_install_sections

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
                let safe = line.replace('\'', "'\\''");
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
            let mut reflector_cmd = String::from("reflector --protocol https --latest 20 --sort rate ");
            reflector_cmd.push_str(&country_args.join(" "));
            if has_custom_servers {
                // Save to tmp then append after custom servers
                // Generate for host, append to both host and target
                let mut refl_host = reflector_cmd.clone();
                refl_host.push_str(" --save /etc/pacman.d/mirrorlist.ai.tmp");
                cmds.push(refl_host);
                cmds.push("cat /etc/pacman.d/mirrorlist.ai.tmp >> /etc/pacman.d/mirrorlist".into());
                cmds.push("cat /etc/pacman.d/mirrorlist.ai.tmp >> /mnt/etc/pacman.d/mirrorlist".into());
                cmds.push("rm -f /etc/pacman.d/mirrorlist.ai.tmp".into());
            } else {
                // Save directly on host, then copy to target
                reflector_cmd.push_str(" --save /etc/pacman.d/mirrorlist");
                cmds.push(reflector_cmd);
                cmds.push("install -Dm644 /etc/pacman.d/mirrorlist /mnt/etc/pacman.d/mirrorlist".into());
            }
        } else if !has_custom_servers {
            // Neither regions nor custom servers selected: best effort copy from ISO host
            cmds.push("test -f /mnt/etc/pacman.d/mirrorlist || install -Dm644 /etc/pacman.d/mirrorlist /mnt/etc/pacman.d/mirrorlist".into());
        }

        cmds
    }
}
