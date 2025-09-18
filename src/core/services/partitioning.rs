use crate::ui::core::state::AppState;

#[derive(Clone, Debug)]
pub struct PartitionPlan {
    pub commands: Vec<String>,
}

impl PartitionPlan {
    pub fn new(commands: Vec<String>) -> Self {
        Self { commands }
    }
}

pub struct PartitioningService;

impl PartitioningService {
    pub fn build_plan(state: &AppState, device: &str) -> PartitionPlan {
        let mut cmds: Vec<String> = Vec::new();
        let label = state.disks_label.clone().unwrap_or_else(|| "gpt".into());
        if state.disks_wipe {
            cmds.push(format!("wipefs -a {}", device));
        }

        let align = state.disks_align.clone().unwrap_or_else(|| "1MiB".into());
        cmds.push(format!("parted -s {} mklabel {}", device, label));

        let mut next_start = align.clone();
        if state.is_uefi() && state.bootloader_index != 1 {
            cmds.push(format!(
                "parted -s {} mkpart ESP fat32 {} 513MiB",
                device, next_start
            ));
            cmds.push(format!("parted -s {} set 1 esp on", device));
            cmds.push(format!("mkfs.vfat -F32 {}1", device));
            next_start = "513MiB".into();
        } else {
            cmds.push(format!(
                "parted -s {} mkpart biosboot {} 2MiB",
                device, next_start
            ));
            cmds.push(format!("parted -s {} set 1 bios_grub on", device));
            next_start = "2MiB".into();
        }

        if state.swap_enabled {
            cmds.push(format!(
                "parted -s {} mkpart swap linux-swap {} 4098MiB",
                device, next_start
            ));
            cmds.push(format!("mkswap {}2", device));
            next_start = "4098MiB".into();
        }

        cmds.push(format!(
            "parted -s {} mkpart root btrfs {} 100%",
            device, next_start
        ));
        let luks = state.disk_encryption_type_index == 1;
        if luks {
            cmds.push(format!("cryptsetup luksFormat {}3", device));
            cmds.push(format!("cryptsetup open {}3 cryptroot", device));
            cmds.push("mkfs.btrfs -f /dev/mapper/cryptroot".into());
        } else {
            cmds.push(format!("mkfs.btrfs -f {}3", device));
        }

        PartitionPlan::new(cmds)
    }

    pub fn execute_plan(plan: PartitionPlan) -> Result<(), String> {
        for c in plan.commands {
            let mut parts = c.split_whitespace();
            let bin = parts.next().unwrap_or("");
            let args: Vec<&str> = parts.collect();
            let status = std::process::Command::new(bin).args(args).status();
            match status {
                Ok(st) if st.success() => {}
                Ok(_) => return Err(format!("Command failed: {}", c)),
                Err(_) => return Err(format!("Failed to run: {}", c)),
            }
        }
        Ok(())
    }
}


