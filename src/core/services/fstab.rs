use crate::core::state::AppState;

#[derive(Clone, Debug)]
pub struct FstabPlan {
    pub commands: Vec<String>,
}

impl FstabPlan {
    pub fn new(commands: Vec<String>) -> Self {
        Self { commands }
    }
}

pub struct FstabService;

impl FstabService {
    pub fn build_checks_and_fstab(state: &AppState, device: &str) -> FstabPlan {
        let mut cmds: Vec<String> = Vec::new();
        // TODO: Add fstab tuning for btrfs subvolumes and mount options (v0.2.0+).

        // Basic device partition paths
        let p1 = format!("{device}1");
        let p2 = format!("{device}2");
        let p3 = format!("{device}3");

        // Check filesystems created
        if state.is_uefi() {
            // Check that partition 1 is an EFI System Partition by PARTLABEL/TYPE GUID, not by fs type
            cmds.push(format!(
                "blkid -o export {p1} | grep -Eq 'PARTLABEL=ESP|PARTLABEL=EFI System Partition|PARTTYPE=EF00|PARTUUID=' || echo 'WARN: ESP not detected on {p1}'"
            ));
        }
        if state.swap_enabled {
            cmds.push(format!(
                "blkid {p2} | grep -q 'TYPE=\"swap\"' || echo 'WARN: swap not found on {p2}'"
            ));
        }
        if state.disk_encryption_type_index == 1 {
            cmds.push("blkid /dev/mapper/cryptroot | grep -q 'TYPE=\"btrfs\"' || echo 'WARN: btrfs not found on cryptroot'".into());
        } else {
            cmds.push(format!(
                "blkid {p3} | grep -q 'TYPE=\"btrfs\"' || echo 'WARN: btrfs not found on {p3}'"
            ));
        }

        // Check mounts
        cmds.push("mountpoint -q /mnt || echo 'ERROR: /mnt is not mounted'".into());
        if state.is_uefi() {
            cmds.push("mountpoint -q /mnt/boot || echo 'ERROR: /mnt/boot is not mounted'".into());
        }
        if state.swap_enabled {
            cmds.push(
                "swapon --noheadings --raw | grep -q '^' || echo 'ERROR: swap not active'".into(),
            );
        }

        // Generate fstab
        cmds.push("genfstab -U /mnt >> /mnt/etc/fstab".into());

        FstabPlan::new(cmds)
    }
}
