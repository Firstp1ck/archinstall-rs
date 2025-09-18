use crate::core::state::AppState;

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
        let mut part_cmds: Vec<String> = Vec::new();
        // TODO(v0.2.0): Support Manual Partitioning editor and executing explicit partition list.
        // TODO(v0.4.0): Add LVM/RAID support and advanced btrfs subvolume layouts.
        let label = state.disks_label.clone().unwrap_or_else(|| "gpt".into());
        if state.disks_wipe {
            part_cmds.push(format!("wipefs -a {}", device));
        }

        let align = state.disks_align.clone().unwrap_or_else(|| "1MiB".into());
        part_cmds.push(format!("parted -s {} mklabel {}", device, label));

        let mut next_start = align.clone();
        // If system boots via UEFI, always create an ESP as partition 1
        if state.is_uefi() {
            part_cmds.push(format!(
                "parted -s {} mkpart ESP fat32 {} 1025MiB",
                device, next_start
            ));
            part_cmds.push(format!("parted -s {} set 1 esp on", device));
            part_cmds.push(format!("mkfs.vfat -F32 {}1", device));
            next_start = "1025MiB".into();
        } else {
            part_cmds.push(format!(
                "parted -s {} mkpart biosboot {} 2MiB",
                device, next_start
            ));
            part_cmds.push(format!("parted -s {} set 1 bios_grub on", device));
            next_start = "2MiB".into();
        }

        if state.swap_enabled {
            let swap_end = if state.is_uefi() {
                "5121MiB"
            } else {
                "4098MiB"
            };
            part_cmds.push(format!(
                "parted -s {} mkpart swap linux-swap {} {}",
                device, next_start, swap_end
            ));
            part_cmds.push(format!("mkswap {}2", device));
            next_start = swap_end.into();
        }

        part_cmds.push(format!(
            "parted -s {} mkpart root btrfs {} 100%",
            device, next_start
        ));
        let luks = state.disk_encryption_type_index == 1;
        if luks {
            part_cmds.push(format!("cryptsetup luksFormat {}3", device));
            part_cmds.push(format!("cryptsetup open {}3 cryptroot", device));
            part_cmds.push("mkfs.btrfs -f /dev/mapper/cryptroot".into());
        } else {
            part_cmds.push(format!("mkfs.btrfs -f {}3", device));
        }

        PartitionPlan::new(part_cmds)
    }

    pub fn execute_plan(plan: PartitionPlan) -> Result<(), String> {
        for c in plan.commands {
            let status = std::process::Command::new("bash")
                .arg("-lc")
                .arg(&c)
                .status();
            match status {
                Ok(st) if st.success() => {}
                Ok(_) => return Err(format!("Command failed: {}", c)),
                Err(_) => return Err(format!("Failed to run: {}", c)),
            }
        }
        Ok(())
    }
}
