use crate::app::{AppState, PopupKind};
use crate::core::services::partitioning::PartitioningService;
use crate::core::services::partitioning::PartitionPlan;
use crate::core::services::mounting::MountingService;
use crate::core::services::system::SystemService;
use crate::core::services::fstab::FstabService;
use crate::core::services::sysconfig::SysConfigService;

impl AppState {
    pub fn start_install(&mut self) {
        self.start_install_flow();
    }

    pub fn start_install_flow(&mut self) {
        let Some(target) = self.select_target_and_run_prechecks() else {
            return;
        };
        let plan = self.build_install_plan(&target);
        self.run_install_plan(plan);
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

    fn build_install_plan(&self, target: &str) -> Vec<String> {
        let mut plan: Vec<String> = Vec::new();
        plan.extend(self.build_locales_plan());
        plan.extend(self.build_mirrors_plan());
        // Partitioning step
        let part_cmds = PartitioningService::build_plan(self, target).commands;
        plan.extend(part_cmds);
        // Mounting step
        let mount_cmds = MountingService::build_plan(self, target).commands;
        plan.extend(mount_cmds);
        // System pre-install step: adjust pacman.conf for optional repos
        let sys_pre = SystemService::build_pre_install_plan(self).commands;
        plan.extend(sys_pre);
        // System installation: pacstrap base and selected packages
        let pacstrap = SystemService::build_pacstrap_plan(self).commands;
        plan.extend(pacstrap);
        // fstab and checks
        let fstab_cmds = FstabService::build_checks_and_fstab(self, target).commands;
        plan.extend(fstab_cmds);
        // System configuration inside chroot
        let syscfg_cmds = SysConfigService::build_plan(self).commands;
        plan.extend(syscfg_cmds);
        // Bootloader setup
        let boot_cmds = crate::core::services::bootloader::BootloaderService::build_plan(self, target).commands;
        plan.extend(boot_cmds);
        // User setup: create users, passwords, sudoers, DM, hyprland config
        let user_cmds = crate::core::services::usersetup::UserSetupService::build_plan(self).commands;
        plan.extend(user_cmds);
        plan
    }

    fn run_install_plan(&mut self, cmds: Vec<String>) {
        if self.dry_run {
            let body = cmds.join("\n");
            self.open_info_popup(body);
            return;
        }
        match PartitioningService::execute_plan(PartitionPlan::new(cmds)) {
            Ok(()) => self.open_info_popup("Partitioning completed.".into()),
            Err(msg) => self.open_info_popup(msg),
        }
    }

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

        // Ensure target mirrorlist directory exists
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
                    let name = line
                        .split("  ")
                        .next()
                        .unwrap_or(line)
                        .trim()
                        .to_string();
                    if !name.is_empty() {
                        country_args.push(format!("-c \"{}\"", name.replace('"', "\\\"")));
                    }
                }
            }
        }

        let has_regions = !country_args.is_empty();
        let has_custom_servers = !self.mirrors_custom_servers.is_empty();

        // If the user added custom servers, place them at the top of mirrorlist
        if has_custom_servers {
            // Write custom servers header
            let mut printf_cmd = String::from("printf '%s\\n' ");
            let mut first = true;
            for url in &self.mirrors_custom_servers {
                let mut line = String::from("Server = ");
                line.push_str(url);
                // Simple quote escape for rare cases
                let safe = line.replace('\'', "'\\''");
                if !first { printf_cmd.push(' '); }
                first = false;
                printf_cmd.push_str(&format!("'{}'", safe));
            }
            printf_cmd.push_str(" > /mnt/etc/pacman.d/mirrorlist");
            cmds.push(printf_cmd);
        }

        if has_regions {
            // Use reflector to fetch and sort HTTPS mirrors for selected regions
            let mut reflector_cmd = String::from(
                "reflector --protocol https --sort rate ",
            );
            reflector_cmd.push_str(&country_args.join(" "));
            if has_custom_servers {
                // Save to tmp then append after custom servers
                reflector_cmd.push_str(" --save /mnt/etc/pacman.d/mirrorlist.ai.tmp");
                cmds.push(reflector_cmd);
                cmds.push("cat /mnt/etc/pacman.d/mirrorlist.ai.tmp >> /mnt/etc/pacman.d/mirrorlist".into());
                cmds.push("rm -f /mnt/etc/pacman.d/mirrorlist.ai.tmp".into());
            } else {
                // Save directly
                reflector_cmd.push_str(" --save /mnt/etc/pacman.d/mirrorlist");
                cmds.push(reflector_cmd);
            }
        } else if !has_custom_servers {
            // Neither regions nor custom servers selected: best effort copy from ISO
            cmds.push("test -f /mnt/etc/pacman.d/mirrorlist || install -Dm644 /etc/pacman.d/mirrorlist /mnt/etc/pacman.d/mirrorlist".into());
        }

        cmds
    }
}
