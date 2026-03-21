// DEPRECATED: Superseded by `crate::core::storage::StoragePlan::mount_commands()`.
// Kept temporarily so existing integration tests in tests/logic.rs still compile.

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
        // NOTE: Btrfs subvolumes and custom mount layout are handled by StoragePlanner (Phase 4).
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
            // Load FAT-related kernel modules (may already be built-in)
            cmds.push(
                "modprobe -q fat 2>/dev/null; modprobe -q vfat 2>/dev/null; modprobe -q msdos 2>/dev/null; modprobe -q nls_cp437 2>/dev/null; modprobe -q nls_iso8859_1 2>/dev/null; modprobe -q nls_ascii 2>/dev/null; true"
                    .into(),
            );
            let esp_part = Self::partition_path(device, 1);
            // Verify FAT support is available before attempting mount
            cmds.push(format!(
                "grep -qE '\\bvfat\\b|\\bfat\\b|\\bmsdos\\b' /proc/filesystems || {{ echo 'ERROR: FAT filesystem support is not available in the running kernel after loading modules.' >&2; echo 'Cannot mount {esp_part} — ensure CONFIG_VFAT_FS is enabled or the vfat module is loadable.' >&2; echo 'Available filesystems:' >&2; cat /proc/filesystems >&2; exit 1; }}"
            ));
            // Mount ESP with fallback across FAT type names
            cmds.push(format!(
                "mount -t vfat --mkdir {esp_part} /mnt/boot || mount -t fat --mkdir {esp_part} /mnt/boot || mount -t msdos --mkdir {esp_part} /mnt/boot"
            ));
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
