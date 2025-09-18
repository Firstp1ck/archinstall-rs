use crate::ui::app::{AppState, PopupKind};
use crate::ui::core::services::partitioning::PartitioningService;
use crate::ui::core::services::partitioning::PartitionPlan;

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
        plan.extend(PartitioningService::build_plan(self, target).commands);
        plan.extend(self.build_bootloader_plan());
        plan.extend(self.build_uki_plan());
        plan.extend(self.build_system_plan());
        plan.extend(self.build_users_plan());
        plan.extend(self.build_experience_plan());
        plan.extend(self.build_graphics_plan());
        plan.extend(self.build_kernels_plan());
        plan.extend(self.build_network_plan());
        plan.extend(self.build_additional_packages_plan());
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
        Vec::new()
    }
    fn build_mirrors_plan(&self) -> Vec<String> {
        Vec::new()
    }
    fn build_bootloader_plan(&self) -> Vec<String> {
        Vec::new()
    }
    fn build_uki_plan(&self) -> Vec<String> {
        Vec::new()
    }
    fn build_system_plan(&self) -> Vec<String> {
        Vec::new()
    }
    fn build_users_plan(&self) -> Vec<String> {
        Vec::new()
    }
    fn build_experience_plan(&self) -> Vec<String> {
        Vec::new()
    }
    fn build_graphics_plan(&self) -> Vec<String> {
        Vec::new()
    }
    fn build_kernels_plan(&self) -> Vec<String> {
        Vec::new()
    }
    fn build_network_plan(&self) -> Vec<String> {
        Vec::new()
    }
    fn build_additional_packages_plan(&self) -> Vec<String> {
        Vec::new()
    }
}

// (no free-function entry; use AppState::start_install instead)
