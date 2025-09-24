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

        // Check if manual partitioning mode is selected
        let is_manual_mode = state.disks_mode_index == 1;

        if is_manual_mode && !state.disks_partitions.is_empty() {
            // Manual partitioning mode: process the disks_partitions vector
            Self::build_manual_partition_plan(state, device, &mut part_cmds);
        } else {
            // Best-effort automatic partitioning mode
            Self::build_automatic_partition_plan(state, device, &mut part_cmds);
        }

        // Debug summary (no raw command strings)
        let label = state.disks_label.clone().unwrap_or_else(|| "gpt".into());
        let align = state.disks_align.clone().unwrap_or_else(|| "1MiB".into());
        if is_manual_mode {
            let mut specs = Vec::new();
            for p in state.disks_partitions.iter() {
                if let Some(spec_device) = &p.name
                    && spec_device != device
                {
                    continue;
                }
                let role = p.role.clone().unwrap_or_else(|| "OTHER".into());
                let fs = p.fs.clone().unwrap_or_else(|| "ext4".into());
                let size = p.size.clone().unwrap_or_else(|| "?".into());
                specs.push(format!("{}:{}:{}", role, fs, size));
            }
            state.debug_log(&format!(
                "partitioning: manual device={} label={} wipe={} align={} specs_count={} [{}]",
                device,
                label,
                state.disks_wipe,
                align,
                specs.len(),
                specs.join(", ")
            ));
        } else {
            state.debug_log(&format!(
                "partitioning: auto device={} label={} wipe={} align={} uefi={} swap={} luks={}",
                device,
                label,
                state.disks_wipe,
                align,
                state.is_uefi(),
                state.swap_enabled,
                state.disk_encryption_type_index == 1
            ));
        }

        PartitionPlan::new(part_cmds)
    }

    fn build_automatic_partition_plan(state: &AppState, device: &str, part_cmds: &mut Vec<String>) {
        // TODO(v0.4.0): Add LVM/RAID support and advanced btrfs subvolume layouts.
        let label = state.disks_label.clone().unwrap_or_else(|| "gpt".into());
        if state.disks_wipe {
            part_cmds.push(format!("wipefs -a {}", device));
        }

        let align = state.disks_align.clone().unwrap_or_else(|| "1MiB".into());
        part_cmds.push(format!("parted -s {} mklabel {}", device, label));
        part_cmds.push(format!("partprobe {} || true", device));
        part_cmds.push("udevadm settle".into());

        let mut next_start = align.clone();
        // If system boots via UEFI, always create an ESP as partition 1
        if state.is_uefi() {
            part_cmds.push(format!(
                "parted -s {} mkpart ESP fat32 {} 1025MiB",
                device, next_start
            ));
            part_cmds.push(format!("parted -s {} set 1 esp on", device));
            part_cmds.push(format!("mkfs.fat -F 32 {}1", device));
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
    }

    fn build_manual_partition_plan(state: &AppState, device: &str, part_cmds: &mut Vec<String>) {
        let label = state.disks_label.clone().unwrap_or_else(|| "gpt".into());
        if state.disks_wipe {
            part_cmds.push(format!("wipefs -a {}", device));
        }

        part_cmds.push(format!("parted -s {} mklabel {}", device, label));
        part_cmds.push(format!("partprobe {} || true", device));
        part_cmds.push("udevadm settle".into());

        // Sort partitions by start position to ensure correct order
        let mut sorted_partitions = state.disks_partitions.clone();
        sorted_partitions.sort_by(|a, b| {
            let start_a = a
                .start
                .as_ref()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let start_b = b
                .start
                .as_ref()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            start_a.cmp(&start_b)
        });

        let mut partition_number = 1;
        for spec in &sorted_partitions {
            // Skip partitions not for this device
            if let Some(spec_device) = &spec.name
                && spec_device != device
            {
                continue;
            }

            let role = spec.role.as_deref().unwrap_or("OTHER");
            let fs = spec.fs.as_deref().unwrap_or("ext4");
            let start = spec.start.as_deref().unwrap_or("0");
            let size = spec.size.as_deref().unwrap_or("100%");

            // Convert start and size to appropriate units for parted
            let start_str = Self::bytes_to_parted_unit(start);
            let size_str = Self::bytes_to_parted_unit(size);

            // Create partition
            let part_type = match role {
                "BOOT" | "EFI" => "ESP",
                "SWAP" => "linux-swap",
                _ => "primary",
            };

            part_cmds.push(format!(
                "parted -s {} mkpart {} {} {} {}",
                device, part_type, fs, start_str, size_str
            ));

            // Set partition flags based on role
            match role {
                "BOOT" | "EFI" => {
                    part_cmds.push(format!(
                        "parted -s {} set {} esp on",
                        device, partition_number
                    ));
                }
                "BIOS_BOOT" => {
                    part_cmds.push(format!(
                        "parted -s {} set {} bios_grub on",
                        device, partition_number
                    ));
                }
                _ => {}
            }

            // Format the partition
            let partition_path = Self::get_partition_path(device, partition_number);
            match fs {
                "fat32" | "fat16" | "fat12" => {
                    let fat_type = match fs {
                        "fat32" => "32",
                        "fat16" => "16",
                        "fat12" => "12",
                        _ => "32",
                    };
                    part_cmds.push(format!("mkfs.fat -F {} {}", fat_type, partition_path));
                }
                "linux-swap" => {
                    part_cmds.push(format!("mkswap {}", partition_path));
                }
                "btrfs" => {
                    part_cmds.push(format!("mkfs.btrfs -f {}", partition_path));
                }
                "ext4" => {
                    part_cmds.push(format!("mkfs.ext4 -F {}", partition_path));
                }
                "ext3" => {
                    part_cmds.push(format!("mkfs.ext3 -F {}", partition_path));
                }
                "ext2" => {
                    part_cmds.push(format!("mkfs.ext2 -F {}", partition_path));
                }
                "xfs" => {
                    part_cmds.push(format!("mkfs.xfs -f {}", partition_path));
                }
                "f2fs" => {
                    part_cmds.push(format!("mkfs.f2fs -f {}", partition_path));
                }
                _ => {
                    part_cmds.push(format!("mkfs.ext4 -F {}", partition_path));
                }
            }

            partition_number += 1;
        }

        part_cmds.push(format!("partprobe {} || true", device));
        part_cmds.push("udevadm settle".into());
    }

    fn bytes_to_parted_unit(bytes_str: &str) -> String {
        // If it's already in a parted-compatible format, return as-is
        if bytes_str.contains("MiB")
            || bytes_str.contains("GiB")
            || bytes_str.contains("KiB")
            || bytes_str.contains("MB")
            || bytes_str.contains("GB")
            || bytes_str.contains("KB")
            || bytes_str == "100%"
        {
            return bytes_str.to_string();
        }

        // Convert bytes to MiB for parted
        if let Ok(bytes) = bytes_str.parse::<u64>() {
            let mib = bytes / (1024 * 1024);
            if mib == 0 {
                "1MiB".to_string()
            } else {
                format!("{}MiB", mib)
            }
        } else {
            bytes_str.to_string()
        }
    }

    fn get_partition_path(device: &str, partition_number: u32) -> String {
        // Handle devices that end with a digit (like /dev/nvme0n1)
        if device
            .chars()
            .last()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            format!("{}p{}", device, partition_number)
        } else {
            format!("{}{}", device, partition_number)
        }
    }

    pub fn execute_plan(plan: PartitionPlan) -> Result<(), String> {
        for c in plan.commands {
            let status = std::process::Command::new("bash")
                .arg("-lc")
                .arg(&c)
                .status();
            match status {
                Ok(st) if st.success() => {}
                Ok(_) => {
                    return Err(format!(
                        "Command failed: {}",
                        crate::common::utils::redact_command_for_logging(&c)
                    ));
                }
                Err(_) => {
                    return Err(format!(
                        "Failed to run: {}",
                        crate::common::utils::redact_command_for_logging(&c)
                    ));
                }
            }
        }
        Ok(())
    }
}
