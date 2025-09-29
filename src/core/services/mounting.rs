use crate::core::state::AppState;

#[derive(Clone, Debug)]
pub struct MountPlan {
    pub commands: Vec<String>,
}

impl MountPlan {
    pub fn new(commands: Vec<String>) -> Self {
        Self { commands }
    }
}

pub struct MountingService;

impl MountingService {
    // Helper to get proper partition path (handles devices ending with a digit like /dev/nvme0n1)
    fn partition_path(device: &str, n: u32) -> String {
        if device
            .chars()
            .last()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            format!("{device}p{n}")
        } else {
            format!("{device}{n}")
        }
    }

    pub fn build_plan(state: &AppState, device: &str) -> MountPlan {
        let mut cmds: Vec<String> = Vec::new();
        // TODO: Handle btrfs subvolumes and custom mount layout (v0.2.0+ / README Roadmap).
        let luks = state.disk_encryption_type_index == 1;
        cmds.push("mkdir -p /mnt".into());
        if luks {
            cmds.push("mount /dev/mapper/cryptroot /mnt".into());
        } else {
            // Mount root (partition 3 in automatic layout)
            let root_part = Self::partition_path(device, 3);
            cmds.push(format!("mount {root_part} /mnt"));
        }
        // On UEFI, always mount the ESP at /mnt/boot so both systemd-boot and GRUB can find it
        if state.is_uefi() {
            // Ensure kernel filesystem drivers are available (some ISOs need explicit load)
            cmds.push("modprobe -q vfat || true".into());
            cmds.push("modprobe -q fat || true".into());
            let esp_part = Self::partition_path(device, 1);
            cmds.push(format!("mount --mkdir {esp_part} /mnt/boot"));
        }
        if state.swap_enabled {
            let swap_part = Self::partition_path(device, 2);
            cmds.push(format!("swapon {swap_part}"));
        }
        // Debug summary
        let root_path = if luks {
            "/dev/mapper/cryptroot".to_string()
        } else {
            Self::partition_path(device, 3)
        };
        state.debug_log(&format!(
            "mounting: root={} uefi={} esp=/mnt/boot swap={} luks={}",
            root_path,
            state.is_uefi(),
            state.swap_enabled,
            luks
        ));
        MountPlan::new(cmds)
    }
}
