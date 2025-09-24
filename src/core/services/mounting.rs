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
    pub fn build_plan(state: &AppState, device: &str) -> MountPlan {
        let mut cmds: Vec<String> = Vec::new();
        // TODO: Handle btrfs subvolumes and custom mount layout (v0.2.0+ / README Roadmap).
        let luks = state.disk_encryption_type_index == 1;
        cmds.push("mkdir -p /mnt".into());
        if luks {
            cmds.push("mount /dev/mapper/cryptroot /mnt".into());
        } else {
            cmds.push(format!("mount {}3 /mnt", device));
        }
        // On UEFI, always mount the ESP at /mnt/boot so both systemd-boot and GRUB can find it
        if state.is_uefi() {
            // Ensure kernel filesystem drivers are available (some ISOs need explicit load)
            cmds.push("modprobe -q vfat || true".into());
            cmds.push("modprobe -q fat || true".into());
            cmds.push(format!("mount --mkdir {p}1 /mnt/boot", p = device));
        }
        if state.swap_enabled {
            cmds.push(format!("swapon {}2", device));
        }
        // Debug summary
        state.debug_log(&format!(
            "mounting: root={} uefi={} esp=/mnt/boot swap={} luks={}",
            if luks {
                "/dev/mapper/cryptroot"
            } else {
                &format!("{}3", device)
            },
            state.is_uefi(),
            state.swap_enabled,
            luks
        ));
        MountPlan::new(cmds)
    }
}
