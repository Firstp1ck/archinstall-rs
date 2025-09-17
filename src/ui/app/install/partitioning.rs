use crate::ui::app::AppState;

impl AppState {
    pub fn build_partition_plan(&self, dev: &str) -> Vec<String> {
        let mut cmds: Vec<String> = Vec::new();
        let label = self.disks_label.clone().unwrap_or_else(|| "gpt".into());
        if self.disks_wipe {
            cmds.push(format!("wipefs -a {}", dev));
        }

        let align = self.disks_align.clone().unwrap_or_else(|| "1MiB".into());
        cmds.push(format!("parted -s {} mklabel {}", dev, label));

        let mut next_start = align.clone();
        if self.is_uefi() && self.bootloader_index != 1 {
            cmds.push(format!(
                "parted -s {} mkpart ESP fat32 {} 513MiB",
                dev, next_start
            ));
            cmds.push(format!("parted -s {} set 1 esp on", dev));
            cmds.push(format!("mkfs.vfat -F32 {}1", dev));
            next_start = "513MiB".into();
        } else {
            cmds.push(format!(
                "parted -s {} mkpart biosboot {} 2MiB",
                dev, next_start
            ));
            cmds.push(format!("parted -s {} set 1 bios_grub on", dev));
            next_start = "2MiB".into();
        }

        if self.swap_enabled {
            cmds.push(format!(
                "parted -s {} mkpart swap linux-swap {} 4098MiB",
                dev, next_start
            ));
            cmds.push(format!("mkswap {}2", dev));
            next_start = "4098MiB".into();
        }

        cmds.push(format!(
            "parted -s {} mkpart root btrfs {} 100%",
            dev, next_start
        ));
        let luks = self.disk_encryption_type_index == 1;
        if luks {
            cmds.push(format!("cryptsetup luksFormat {}3", dev));
            cmds.push(format!("cryptsetup open {}3 cryptroot", dev));
            cmds.push("mkfs.btrfs -f /dev/mapper/cryptroot".into());
        } else {
            cmds.push(format!("mkfs.btrfs -f {}3", dev));
        }
        cmds
    }

    pub fn execute_plan(&mut self, cmds: Vec<String>) {
        for c in cmds {
            let mut parts = c.split_whitespace();
            let bin = parts.next().unwrap_or("");
            let args: Vec<&str> = parts.collect();
            let status = std::process::Command::new(bin).args(args).status();
            if let Ok(st) = status {
                if !st.success() {
                    self.open_info_popup(format!("Command failed: {}", c));
                    return;
                }
            } else {
                self.open_info_popup(format!("Failed to run: {}", c));
                return;
            }
        }
        self.open_info_popup("Partitioning completed.".into());
    }
}

