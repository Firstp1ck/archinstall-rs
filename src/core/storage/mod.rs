pub mod planner;

use std::fmt;

#[derive(Clone, Debug)]
pub struct StoragePlan {
    pub devices: Vec<PlannedDevice>,
    pub mounts: Vec<PlannedMount>,
    pub mode: StorageMode,
    /// Layered device stacks for LVM, RAID, or multi-layer encryption setups.
    /// Empty for simple partition-based layouts. Command generation is stubbed (Phase 6).
    pub stacks: Vec<DeviceStack>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageMode {
    Automatic,
    Manual,
    PreMounted,
}

#[derive(Clone, Debug)]
pub struct PlannedDevice {
    pub path: String,
    pub label: DiskLabel,
    pub wipe: bool,
    pub partitions: Vec<PlannedPartition>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiskLabel {
    Gpt,
    Msdos,
}

impl DiskLabel {
    pub fn as_parted_str(&self) -> &str {
        match self {
            DiskLabel::Gpt => "gpt",
            DiskLabel::Msdos => "msdos",
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlannedPartition {
    pub number: u32,
    pub role: PartitionRole,
    pub start: String,
    pub end: String,
    pub filesystem: FilesystemSpec,
    pub flags: Vec<PartitionFlag>,
    pub encryption: Option<EncryptionSpec>,
    pub subvolumes: Vec<SubvolumeSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PartitionRole {
    Esp,
    BiosBoot,
    Swap,
    Root,
    Home,
    Var,
    Other(String),
}

impl PartitionRole {
    pub fn from_str_role(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "BOOT" | "EFI" | "ESP" => PartitionRole::Esp,
            "BIOS_BOOT" => PartitionRole::BiosBoot,
            "SWAP" => PartitionRole::Swap,
            "ROOT" => PartitionRole::Root,
            "HOME" => PartitionRole::Home,
            "VAR" => PartitionRole::Var,
            other => PartitionRole::Other(other.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PartitionFlag {
    Esp,
    BiosGrub,
}

impl PartitionFlag {
    pub fn as_parted_str(&self) -> &str {
        match self {
            PartitionFlag::Esp => "esp",
            PartitionFlag::BiosGrub => "bios_grub",
        }
    }
}

#[derive(Clone, Debug)]
pub struct FilesystemSpec {
    pub fstype: String,
    pub mkfs_options: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct EncryptionSpec {
    pub method: EncryptionMethod,
    pub mapper_name: String,
    /// Passphrase for LUKS encryption (piped to cryptsetup via stdin at install time).
    pub passphrase: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EncryptionMethod {
    Luks2,
}

// ── LVM / RAID / Volume layer abstractions (Phase 6) ──

/// LVM logical volume specification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LvmSpec {
    pub vg_name: String,
    pub lv_name: String,
    /// Size expression: bytes, "100%FREE", or a parted-style unit like "50GiB".
    pub size: String,
}

/// Software RAID specification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RaidSpec {
    /// RAID level: "0", "1", "5", "6", "10".
    pub level: String,
    /// Member device paths (partitions or other stacks).
    pub members: Vec<String>,
    /// md device name (used in /dev/md/<name>).
    pub name: String,
}

/// A single transformation layer in a device stack.
#[derive(Clone, Debug)]
pub enum VolumeLayer {
    Luks(EncryptionSpec),
    Lvm(LvmSpec),
    Raid(RaidSpec),
}

/// An ordered stack of volume transformations from a base device to a final usable device.
///
/// Represents chains like: `/dev/sda3` → LUKS → LVM VG/LV → filesystem → mount.
/// Each layer transforms the previous device path into a new one.
#[derive(Clone, Debug)]
pub struct DeviceStack {
    /// Base device path (e.g., a partition like "/dev/sda3").
    pub base: String,
    /// Ordered transformations applied on top of the base device.
    pub layers: Vec<VolumeLayer>,
    /// Filesystem to create on the final device.
    pub filesystem: Option<FilesystemSpec>,
}

impl DeviceStack {
    /// Resolve the final device path after all layers are applied.
    pub fn final_device_path(&self) -> String {
        let mut current = self.base.clone();
        for layer in &self.layers {
            match layer {
                VolumeLayer::Luks(enc) => {
                    current = format!("/dev/mapper/{}", enc.mapper_name);
                }
                VolumeLayer::Lvm(lvm) => {
                    current = format!("/dev/{}/{}", lvm.vg_name, lvm.lv_name);
                }
                VolumeLayer::Raid(raid) => {
                    current = format!("/dev/md/{}", raid.name);
                }
            }
        }
        current
    }

    /// Generate setup commands for this device stack.
    ///
    /// Walks each layer in order, tracking the current device path and emitting
    /// the commands needed to set up that layer. Finishes with filesystem creation
    /// on the final device when a `FilesystemSpec` is present.
    pub fn setup_commands(&self) -> Vec<String> {
        let mut cmds = Vec::new();
        let mut current = self.base.clone();

        for layer in &self.layers {
            match layer {
                VolumeLayer::Luks(enc) => {
                    cmds.push(
                        "modprobe -q dm_crypt 2>/dev/null || modprobe -q dm-crypt 2>/dev/null || true"
                            .into(),
                    );
                    cmds.push(Self::luks_format_cmd(&current, enc.passphrase.as_deref()));
                    cmds.push("udevadm settle".into());
                    cmds.push(Self::luks_open_cmd(
                        &current,
                        &enc.mapper_name,
                        enc.passphrase.as_deref(),
                    ));
                    current = format!("/dev/mapper/{}", enc.mapper_name);
                }
                VolumeLayer::Lvm(lvm) => {
                    cmds.push(format!("pvcreate {current}"));
                    cmds.push(format!("vgcreate {} {current}", lvm.vg_name));
                    let size_flag = if lvm.size.contains('%') {
                        format!("-l {}", lvm.size)
                    } else {
                        format!("-L {}", lvm.size)
                    };
                    cmds.push(format!(
                        "lvcreate {size_flag} {} -n {}",
                        lvm.vg_name, lvm.lv_name
                    ));
                    current = format!("/dev/{}/{}", lvm.vg_name, lvm.lv_name);
                }
                VolumeLayer::Raid(raid) => {
                    let members_str = raid.members.join(" ");
                    cmds.push(format!(
                        "mdadm --create /dev/md/{} --level={} --raid-devices={} {}",
                        raid.name,
                        raid.level,
                        raid.members.len(),
                        members_str
                    ));
                    current = format!("/dev/md/{}", raid.name);
                }
            }
        }

        if let Some(ref fs) = self.filesystem {
            cmds.push(Self::mkfs_command(&fs.fstype, &current));
        }

        cmds
    }

    /// Build a `cryptsetup luksFormat` command, piping the passphrase via stdin.
    /// Uses `--key-file=-` so cryptsetup reads from stdin (not /dev/tty), and
    /// `-q` to suppress the "Are you sure?" confirmation. Uses `printf '%s'`
    /// instead of `echo -n` to avoid shell-dependent newline behaviour.
    pub(crate) fn luks_format_cmd(device: &str, passphrase: Option<&str>) -> String {
        match passphrase {
            Some(pw) => {
                let escaped = pw.replace('\'', "'\\''");
                format!(
                    "printf '%s' '{escaped}' | cryptsetup luksFormat --type luks2 -q --key-file=- {device}"
                )
            }
            None => format!("cryptsetup luksFormat --type luks2 -q {device}"),
        }
    }

    /// Build a `cryptsetup open` command, piping the passphrase via stdin.
    /// Uses `--key-file=-` so cryptsetup reads from stdin with consistent
    /// passphrase processing (no newline stripping) matching luksFormat.
    pub(crate) fn luks_open_cmd(device: &str, mapper: &str, passphrase: Option<&str>) -> String {
        match passphrase {
            Some(pw) => {
                let escaped = pw.replace('\'', "'\\''");
                format!(
                    "printf '%s' '{escaped}' | cryptsetup open --type luks --key-file=- {device} {mapper}"
                )
            }
            None => format!("cryptsetup open --type luks {device} {mapper}"),
        }
    }

    fn mkfs_command(fstype: &str, device: &str) -> String {
        match fstype {
            "fat32" => format!("mkfs.fat -F 32 {device}"),
            "fat16" => format!("mkfs.fat -F 16 {device}"),
            "fat12" => format!("mkfs.fat -F 12 {device}"),
            "linux-swap" => format!("mkswap {device}"),
            "btrfs" => format!("mkfs.btrfs -f {device}"),
            "ext4" => format!("mkfs.ext4 -F {device}"),
            "ext3" => format!("mkfs.ext3 -F {device}"),
            "ext2" => format!("mkfs.ext2 -F {device}"),
            "xfs" => format!("mkfs.xfs -f {device}"),
            "f2fs" => format!("mkfs.f2fs -f {device}"),
            _ => format!("mkfs.ext4 -F {device}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubvolumeSpec {
    pub name: String,
    pub mountpoint: String,
    pub mount_options: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BtrfsSubvolumePreset {
    Flat,
    Standard,
    Extended,
}

impl BtrfsSubvolumePreset {
    pub fn subvolumes(&self) -> Vec<SubvolumeSpec> {
        let default_opts = vec!["compress=zstd".into(), "noatime".into()];
        match self {
            BtrfsSubvolumePreset::Flat => vec![],
            BtrfsSubvolumePreset::Standard => vec![
                SubvolumeSpec {
                    name: "@".into(),
                    mountpoint: "/".into(),
                    mount_options: default_opts.clone(),
                },
                SubvolumeSpec {
                    name: "@home".into(),
                    mountpoint: "/home".into(),
                    mount_options: default_opts.clone(),
                },
                SubvolumeSpec {
                    name: "@snapshots".into(),
                    mountpoint: "/.snapshots".into(),
                    mount_options: default_opts,
                },
            ],
            BtrfsSubvolumePreset::Extended => vec![
                SubvolumeSpec {
                    name: "@".into(),
                    mountpoint: "/".into(),
                    mount_options: default_opts.clone(),
                },
                SubvolumeSpec {
                    name: "@home".into(),
                    mountpoint: "/home".into(),
                    mount_options: default_opts.clone(),
                },
                SubvolumeSpec {
                    name: "@var_log".into(),
                    mountpoint: "/var/log".into(),
                    mount_options: default_opts.clone(),
                },
                SubvolumeSpec {
                    name: "@snapshots".into(),
                    mountpoint: "/.snapshots".into(),
                    mount_options: default_opts,
                },
            ],
        }
    }

    pub fn label(&self) -> &str {
        match self {
            BtrfsSubvolumePreset::Flat => "Flat (no subvolumes)",
            BtrfsSubvolumePreset::Standard => "Standard (@, @home, @snapshots)",
            BtrfsSubvolumePreset::Extended => "Extended (@, @home, @var_log, @snapshots)",
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlannedMount {
    pub source: String,
    pub target: String,
    pub fstype: String,
    pub options: Vec<String>,
    pub is_swap: bool,
    pub subvolume: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ValidationError {
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StoragePlan {
    /// Collect all LUKS mapper names used by this plan (for pre-cleanup).
    pub fn luks_mapper_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for device in &self.devices {
            for part in &device.partitions {
                if let Some(enc) = &part.encryption {
                    names.push(enc.mapper_name.clone());
                }
            }
        }
        for stack in &self.stacks {
            for layer in &stack.layers {
                if let VolumeLayer::Luks(enc) = layer {
                    names.push(enc.mapper_name.clone());
                }
            }
        }
        names
    }

    /// Generate partition path for a device + partition number.
    /// Handles nvme-style devices (ending in digit) with a `p` separator.
    pub fn partition_path(device: &str, number: u32) -> String {
        if device.chars().last().is_some_and(|c| c.is_ascii_digit()) {
            format!("{device}p{number}")
        } else {
            format!("{device}{number}")
        }
    }

    pub fn partition_commands(&self) -> Vec<String> {
        let mut cmds = Vec::new();

        for device in &self.devices {
            if device.wipe {
                cmds.push(format!("wipefs -a {}", device.path));
            }

            cmds.push(format!(
                "parted -s {} mklabel {}",
                device.path,
                device.label.as_parted_str()
            ));
            cmds.push(format!("partprobe {} || true", device.path));
            cmds.push("udevadm settle".into());

            for part in &device.partitions {
                let parted_type = match part.role {
                    PartitionRole::Esp => "ESP",
                    PartitionRole::BiosBoot => "biosboot",
                    PartitionRole::Swap => "swap",
                    PartitionRole::Root => "root",
                    _ => "primary",
                };

                // biosboot partitions use only a label, no filesystem hint
                if part.role == PartitionRole::BiosBoot {
                    cmds.push(format!(
                        "parted -s {} mkpart {} {} {}",
                        device.path, parted_type, part.start, part.end
                    ));
                } else {
                    cmds.push(format!(
                        "parted -s {} mkpart {} {} {} {}",
                        device.path, parted_type, part.filesystem.fstype, part.start, part.end
                    ));
                }

                for flag in &part.flags {
                    cmds.push(format!(
                        "parted -s {} set {} {} on",
                        device.path,
                        part.number,
                        flag.as_parted_str()
                    ));
                }

                // biosboot partitions are not formatted
                if part.role == PartitionRole::BiosBoot {
                    continue;
                }

                let part_path = Self::partition_path(&device.path, part.number);

                if let Some(enc) = &part.encryption {
                    match enc.method {
                        EncryptionMethod::Luks2 => {
                            cmds.push(
                                "modprobe -q dm_crypt 2>/dev/null || modprobe -q dm-crypt 2>/dev/null || true"
                                    .into(),
                            );
                            cmds.push(DeviceStack::luks_format_cmd(
                                &part_path,
                                enc.passphrase.as_deref(),
                            ));
                            cmds.push("udevadm settle".into());
                            cmds.push(DeviceStack::luks_open_cmd(
                                &part_path,
                                &enc.mapper_name,
                                enc.passphrase.as_deref(),
                            ));
                        }
                    }
                }

                let format_target = if let Some(enc) = &part.encryption {
                    format!("/dev/mapper/{}", enc.mapper_name)
                } else {
                    part_path
                };

                match part.filesystem.fstype.as_str() {
                    "fat32" => cmds.push(format!("mkfs.fat -F 32 {format_target}")),
                    "fat16" => cmds.push(format!("mkfs.fat -F 16 {format_target}")),
                    "fat12" => cmds.push(format!("mkfs.fat -F 12 {format_target}")),
                    "linux-swap" => cmds.push(format!("mkswap {format_target}")),
                    "btrfs" => cmds.push(format!("mkfs.btrfs -f {format_target}")),
                    "ext4" => cmds.push(format!("mkfs.ext4 -F {format_target}")),
                    "ext3" => cmds.push(format!("mkfs.ext3 -F {format_target}")),
                    "ext2" => cmds.push(format!("mkfs.ext2 -F {format_target}")),
                    "xfs" => cmds.push(format!("mkfs.xfs -f {format_target}")),
                    "f2fs" => cmds.push(format!("mkfs.f2fs -f {format_target}")),
                    _ => cmds.push(format!("mkfs.ext4 -F {format_target}")),
                }

                if !part.subvolumes.is_empty() && part.filesystem.fstype == "btrfs" {
                    cmds.push(format!("mount {format_target} /mnt"));
                    for sv in &part.subvolumes {
                        cmds.push(format!("btrfs subvolume create /mnt/{}", sv.name));
                    }
                    cmds.push("umount /mnt".into());
                }
            }

            cmds.push(format!("partprobe {} || true", device.path));
            cmds.push("udevadm settle".into());
        }

        cmds
    }

    pub fn mount_commands(&self) -> Vec<String> {
        if self.mode == StorageMode::PreMounted {
            return vec![];
        }

        let mut cmds = Vec::new();
        cmds.push("mkdir -p /mnt".into());

        for mount in &self.mounts {
            if mount.is_swap {
                cmds.push(format!("swapon {}", mount.source));
            } else {
                let mut opts = mount.options.clone();
                if let Some(ref sv) = mount.subvolume {
                    opts.insert(0, format!("subvol={sv}"));
                }

                // Load FAT filesystem modules and verify support before mounting
                if mount.fstype == "vfat" {
                    // Try modprobe first (handles compressed .ko.zst modules).
                    // If that fails, rebuild module deps and retry — on some ISOs
                    // modules.dep is missing or stale, causing modprobe to silently
                    // fail even though the .ko files exist on disk.
                    cmds.push(
                        "modprobe -q fat 2>/dev/null; modprobe -q vfat 2>/dev/null; modprobe -q msdos 2>/dev/null; modprobe -q nls_cp437 2>/dev/null; modprobe -q nls_iso8859_1 2>/dev/null; modprobe -q nls_ascii 2>/dev/null; true"
                            .into(),
                    );
                    cmds.push(
                        "if ! grep -qE '\\bvfat\\b|\\bfat\\b|\\bmsdos\\b' /proc/filesystems; then depmod -a 2>/dev/null; modprobe fat 2>/dev/null; modprobe vfat 2>/dev/null; modprobe nls_cp437 2>/dev/null; modprobe nls_iso8859_1 2>/dev/null; modprobe nls_ascii 2>/dev/null; true; fi"
                            .into(),
                    );
                    // Verify FAT support is actually available before attempting mount
                    cmds.push(format!(
                        "grep -qE '\\bvfat\\b|\\bfat\\b|\\bmsdos\\b' /proc/filesystems || {{ echo 'ERROR: FAT filesystem support is not available in the running kernel after loading modules.' >&2; echo 'Cannot mount {} -- ensure CONFIG_VFAT_FS is enabled or the vfat module is loadable.' >&2; echo 'Tried: modprobe, depmod -a + modprobe retry.' >&2; echo 'Module directory: /lib/modules/'$(uname -r) >&2; ls /lib/modules/ >&2 2>/dev/null; echo 'Available filesystems:' >&2; cat /proc/filesystems >&2; exit 1; }}",
                        mount.source
                    ));
                }

                let build_mount_cmd = |fstype: Option<&str>| -> String {
                    let type_flag = fstype.map(|t| format!("-t {t} ")).unwrap_or_default();
                    if opts.is_empty() {
                        if mount.target == "/mnt" {
                            format!("mount {type_flag}{} /mnt", mount.source)
                        } else {
                            format!("mount {type_flag}--mkdir {} {}", mount.source, mount.target)
                        }
                    } else {
                        let opt_str = opts.join(",");
                        if mount.target == "/mnt" {
                            format!("mount {type_flag}-o {opt_str} {} /mnt", mount.source)
                        } else {
                            format!(
                                "mount {type_flag}--mkdir -o {opt_str} {} {}",
                                mount.source, mount.target
                            )
                        }
                    }
                };

                if mount.fstype == "vfat" {
                    // FAT support verified above; try vfat first, fall back to fat/msdos
                    // for environments that register the type under a different name.
                    let cmd_vfat = build_mount_cmd(Some("vfat"));
                    let cmd_fat = build_mount_cmd(Some("fat"));
                    let cmd_msdos = build_mount_cmd(Some("msdos"));
                    cmds.push(format!("{cmd_vfat} || {cmd_fat} || {cmd_msdos}"));
                } else {
                    cmds.push(build_mount_cmd(None));
                }
            }
        }

        cmds
    }

    pub fn fstab_check_commands(&self) -> Vec<String> {
        let mut cmds = Vec::new();

        if self.mode == StorageMode::PreMounted {
            // Pre-mounted: skip blkid/format checks — we didn't create these filesystems.
            // Validate that expected mountpoints are still active and generate fstab.
            for mount in &self.mounts {
                if mount.is_swap {
                    continue;
                }
                cmds.push(format!(
                    "mountpoint -q {} || echo 'ERROR: {} is not mounted'",
                    mount.target, mount.target
                ));
            }
            for mount in &self.mounts {
                if mount.is_swap {
                    cmds.push(
                        "swapon --noheadings --raw | grep -q '^' || echo 'ERROR: swap not active'"
                            .into(),
                    );
                }
            }
            cmds.push("genfstab -U /mnt >> /mnt/etc/fstab".into());
            return cmds;
        }

        for mount in &self.mounts {
            if mount.is_swap {
                continue;
            }
            if mount.fstype == "vfat" || mount.fstype == "fat32" {
                cmds.push(format!(
                    "blkid -o export {} | grep -Eq 'PARTLABEL=ESP|PARTLABEL=EFI System Partition|PARTTYPE=EF00|PARTUUID=' || echo 'WARN: ESP not detected on {}'",
                    mount.source, mount.source
                ));
            }
        }

        for mount in &self.mounts {
            if mount.is_swap {
                cmds.push(format!(
                    "blkid {} | grep -q 'TYPE=\"swap\"' || echo 'WARN: swap not found on {}'",
                    mount.source, mount.source
                ));
            }
        }

        for mount in &self.mounts {
            if mount.is_swap {
                continue;
            }
            if mount.fstype == "vfat" || mount.fstype == "fat32" {
                continue;
            }
            cmds.push(format!(
                "blkid {} | grep -q 'TYPE=\"{}\"' || echo 'WARN: {} not found on {}'",
                mount.source, mount.fstype, mount.fstype, mount.source
            ));
        }

        for mount in &self.mounts {
            if mount.is_swap {
                continue;
            }
            cmds.push(format!(
                "mountpoint -q {} || echo 'ERROR: {} is not mounted'",
                mount.target, mount.target
            ));
        }

        for mount in &self.mounts {
            if mount.is_swap {
                cmds.push(
                    "swapon --noheadings --raw | grep -q '^' || echo 'ERROR: swap not active'"
                        .into(),
                );
            }
        }

        cmds.push("genfstab -U /mnt >> /mnt/etc/fstab".into());

        cmds
    }

    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if self.mode == StorageMode::PreMounted {
            let has_root = self.mounts.iter().any(|m| m.target == "/mnt");
            if !has_root {
                errors.push(ValidationError {
                    message: "Pre-mounted mode requires a filesystem mounted at /mnt".into(),
                });
            }
            return errors;
        }

        let has_root = self
            .devices
            .iter()
            .any(|d| d.partitions.iter().any(|p| p.role == PartitionRole::Root));
        if !has_root {
            errors.push(ValidationError {
                message: "No root partition defined".into(),
            });
        }

        let has_esp = self
            .devices
            .iter()
            .any(|d| d.partitions.iter().any(|p| p.role == PartitionRole::Esp));
        let has_biosboot = self.devices.iter().any(|d| {
            d.partitions
                .iter()
                .any(|p| p.role == PartitionRole::BiosBoot)
        });

        let is_uefi = self
            .mounts
            .iter()
            .any(|m| m.target == "/mnt/boot" && (m.fstype == "vfat" || m.fstype == "fat32"))
            || has_esp;

        if is_uefi && !has_esp {
            errors.push(ValidationError {
                message: "UEFI mode requires an ESP partition".into(),
            });
        }
        if !is_uefi && !has_biosboot && !has_esp {
            errors.push(ValidationError {
                message: "BIOS mode requires a BIOS boot partition or ESP".into(),
            });
        }

        let mut mountpoints: Vec<&str> = Vec::new();
        for mount in &self.mounts {
            if mount.is_swap {
                continue;
            }
            if mountpoints.contains(&mount.target.as_str()) {
                errors.push(ValidationError {
                    message: format!("Duplicate mountpoint: {}", mount.target),
                });
            }
            mountpoints.push(&mount.target);
        }

        for device in &self.devices {
            for (i, a) in device.partitions.iter().enumerate() {
                for b in device.partitions.iter().skip(i + 1) {
                    if let (Ok(a_start), Ok(a_end), Ok(b_start), Ok(b_end)) = (
                        parse_parted_to_mib(&a.start),
                        parse_parted_to_mib(&a.end),
                        parse_parted_to_mib(&b.start),
                        parse_parted_to_mib(&b.end),
                    ) && a_start < b_end
                        && b_start < a_end
                    {
                        errors.push(ValidationError {
                            message: format!(
                                "Overlapping partitions: #{} and #{} on {}",
                                a.number, b.number, device.path
                            ),
                        });
                    }
                }
            }
        }

        errors
    }

    pub fn root_device_path(&self) -> Option<String> {
        for mount in &self.mounts {
            if mount.target == "/mnt" && !mount.is_swap {
                return Some(mount.source.clone());
            }
        }
        None
    }

    pub fn esp_device_path(&self) -> Option<String> {
        for mount in &self.mounts {
            if (mount.target == "/mnt/boot" || mount.target == "/mnt/efi") && !mount.is_swap {
                return Some(mount.source.clone());
            }
        }
        None
    }

    /// Collect setup commands for all device stacks (LVM, RAID, multi-layer).
    /// Returns an empty vec for simple partition-based layouts with no stacks.
    pub fn stack_setup_commands(&self) -> Vec<String> {
        let mut cmds = Vec::new();
        for stack in &self.stacks {
            cmds.extend(stack.setup_commands());
        }
        cmds
    }

    pub fn has_encryption(&self) -> bool {
        self.devices
            .iter()
            .any(|d| d.partitions.iter().any(|p| p.encryption.is_some()))
            || self
                .stacks
                .iter()
                .any(|s| s.layers.iter().any(|l| matches!(l, VolumeLayer::Luks(_))))
    }
}

fn parse_parted_to_mib(s: &str) -> Result<f64, ()> {
    let s = s.trim();
    if let Some(v) = s.strip_suffix("MiB") {
        v.trim().parse::<f64>().map_err(|_| ())
    } else if let Some(v) = s.strip_suffix("GiB") {
        v.trim().parse::<f64>().map(|n| n * 1024.0).map_err(|_| ())
    } else if s == "100%" {
        Ok(f64::MAX)
    } else {
        s.parse::<f64>().map_err(|_| ())
    }
}
