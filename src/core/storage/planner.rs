use crate::core::state::AppState;
use crate::core::storage::*;

pub struct StoragePlanner;

impl StoragePlanner {
    pub fn compile(state: &AppState) -> Result<StoragePlan, Vec<ValidationError>> {
        let plan = match state.disks_mode_index {
            0 => Self::compile_automatic(state)?,
            1 => Self::compile_manual(state)?,
            _ => Self::compile_pre_mounted(state)?,
        };

        let errors = plan.validate();
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(plan)
    }

    fn compile_automatic(state: &AppState) -> Result<StoragePlan, Vec<ValidationError>> {
        let device_path = match &state.disks_selected_device {
            Some(p) => p.clone(),
            None => {
                return Err(vec![ValidationError {
                    message: "No target disk selected".into(),
                }]);
            }
        };

        let label = match state.disks_label.as_deref().unwrap_or("gpt") {
            "msdos" => DiskLabel::Msdos,
            _ => DiskLabel::Gpt,
        };
        let align = state.disks_align.clone().unwrap_or_else(|| "1MiB".into());
        let is_uefi = state.is_uefi();
        let luks = state.disk_encryption_type_index == 1;

        let mut partitions = Vec::new();
        let mut mounts = Vec::new();
        let mut part_num: u32 = 1;

        if is_uefi {
            partitions.push(PlannedPartition {
                number: part_num,
                role: PartitionRole::Esp,
                start: align.clone(),
                end: "1025MiB".into(),
                filesystem: FilesystemSpec {
                    fstype: "fat32".into(),
                    mkfs_options: vec![],
                },
                flags: vec![PartitionFlag::Esp],
                encryption: None,
                subvolumes: vec![],
            });
            let esp_path = StoragePlan::partition_path(&device_path, part_num);
            mounts.push(PlannedMount {
                source: esp_path,
                target: "/mnt/boot".into(),
                fstype: "vfat".into(),
                options: vec![],
                is_swap: false,
                subvolume: None,
            });
            part_num += 1;
        } else {
            partitions.push(PlannedPartition {
                number: part_num,
                role: PartitionRole::BiosBoot,
                start: align.clone(),
                end: "2MiB".into(),
                filesystem: FilesystemSpec {
                    fstype: "biosboot".into(),
                    mkfs_options: vec![],
                },
                flags: vec![PartitionFlag::BiosGrub],
                encryption: None,
                subvolumes: vec![],
            });
            part_num += 1;
        }

        let swap_start = if is_uefi { "1025MiB" } else { "2MiB" };

        if state.swap_enabled {
            let swap_end = if is_uefi { "5121MiB" } else { "4098MiB" };
            partitions.push(PlannedPartition {
                number: part_num,
                role: PartitionRole::Swap,
                start: swap_start.into(),
                end: swap_end.into(),
                filesystem: FilesystemSpec {
                    fstype: "linux-swap".into(),
                    mkfs_options: vec![],
                },
                flags: vec![],
                encryption: None,
                subvolumes: vec![],
            });
            let swap_path = StoragePlan::partition_path(&device_path, part_num);
            mounts.push(PlannedMount {
                source: swap_path,
                target: String::new(),
                fstype: "swap".into(),
                options: vec![],
                is_swap: true,
                subvolume: None,
            });
            part_num += 1;
        }

        let root_start = if state.swap_enabled {
            if is_uefi { "5121MiB" } else { "4098MiB" }
        } else {
            swap_start
        };

        let encryption = if luks {
            let pw = if state.disk_encryption_password.is_empty() {
                None
            } else {
                Some(state.disk_encryption_password.clone())
            };
            Some(EncryptionSpec {
                method: EncryptionMethod::Luks2,
                mapper_name: "cryptroot".into(),
                passphrase: pw,
            })
        } else {
            None
        };

        let preset = match state.btrfs_subvolume_preset {
            1 => BtrfsSubvolumePreset::Standard,
            2 => BtrfsSubvolumePreset::Extended,
            _ => BtrfsSubvolumePreset::Flat,
        };
        let subvolumes = preset.subvolumes();

        partitions.push(PlannedPartition {
            number: part_num,
            role: PartitionRole::Root,
            start: root_start.into(),
            end: "100%".into(),
            filesystem: FilesystemSpec {
                fstype: "btrfs".into(),
                mkfs_options: vec![],
            },
            flags: vec![],
            encryption: encryption.clone(),
            subvolumes: subvolumes.clone(),
        });

        let root_source = if luks {
            "/dev/mapper/cryptroot".into()
        } else {
            StoragePlan::partition_path(&device_path, part_num)
        };

        if subvolumes.is_empty() {
            // Flat: single root mount, no subvolumes
            mounts.insert(
                0,
                PlannedMount {
                    source: root_source,
                    target: "/mnt".into(),
                    fstype: "btrfs".into(),
                    options: vec![],
                    is_swap: false,
                    subvolume: None,
                },
            );
        } else {
            // Subvolume mounts — root (@) first, then others sorted by mountpoint depth
            let mut subvol_mounts: Vec<PlannedMount> = subvolumes
                .iter()
                .map(|sv| {
                    let target = if sv.mountpoint == "/" {
                        "/mnt".into()
                    } else {
                        format!("/mnt{}", sv.mountpoint)
                    };
                    PlannedMount {
                        source: root_source.clone(),
                        target,
                        fstype: "btrfs".into(),
                        options: sv.mount_options.clone(),
                        is_swap: false,
                        subvolume: Some(sv.name.clone()),
                    }
                })
                .collect();
            subvol_mounts.sort_by(|a, b| {
                if a.target == "/mnt" {
                    return std::cmp::Ordering::Less;
                }
                if b.target == "/mnt" {
                    return std::cmp::Ordering::Greater;
                }
                a.target.cmp(&b.target)
            });
            // Insert subvolume mounts at the beginning (before ESP/swap mounts)
            for (i, m) in subvol_mounts.into_iter().enumerate() {
                mounts.insert(i, m);
            }
        }

        let device = PlannedDevice {
            path: device_path,
            label,
            wipe: state.disks_wipe,
            partitions,
        };

        Ok(StoragePlan {
            devices: vec![device],
            mounts,
            mode: StorageMode::Automatic,
            stacks: vec![],
        })
    }

    fn compile_manual(state: &AppState) -> Result<StoragePlan, Vec<ValidationError>> {
        let device_path = match &state.disks_selected_device {
            Some(p) => p.clone(),
            None => {
                return Err(vec![ValidationError {
                    message: "No target disk selected".into(),
                }]);
            }
        };

        if state.disks_partitions.is_empty() {
            return Err(vec![ValidationError {
                message: "No partitions defined for manual mode".into(),
            }]);
        }

        // Pre-validate each spec before conversion
        let mut pre_errors = Vec::new();
        for (i, spec) in state.disks_partitions.iter().enumerate() {
            if let Some(spec_device) = &spec.name
                && spec_device != &device_path
            {
                continue;
            }

            let label = format!("Partition {}", i + 1);

            if spec.role.is_none() || spec.role.as_deref() == Some("") {
                pre_errors.push(ValidationError {
                    message: format!("{label}: missing role (BOOT/SWAP/ROOT/OTHER)"),
                });
            }

            let role_str = spec.role.as_deref().unwrap_or("");
            let role = PartitionRole::from_str_role(role_str);

            if (spec.fs.is_none() || spec.fs.as_deref() == Some(""))
                && role != PartitionRole::BiosBoot
            {
                pre_errors.push(ValidationError {
                    message: format!("{label}: missing filesystem type"),
                });
            }

            if spec.start.is_none() || spec.start.as_deref() == Some("") {
                pre_errors.push(ValidationError {
                    message: format!("{label}: missing start position"),
                });
            }

            if spec.size.is_none() || spec.size.as_deref() == Some("") {
                pre_errors.push(ValidationError {
                    message: format!("{label}: missing size"),
                });
            }

            if role != PartitionRole::Swap && role != PartitionRole::BiosBoot {
                let mp = spec.mountpoint.as_deref().unwrap_or("");
                if mp.is_empty() {
                    pre_errors.push(ValidationError {
                        message: format!("{label} ({role_str}): missing mountpoint"),
                    });
                }
            }
        }

        if !pre_errors.is_empty() {
            return Err(pre_errors);
        }

        let label = match state.disks_label.as_deref().unwrap_or("gpt") {
            "msdos" => DiskLabel::Msdos,
            _ => DiskLabel::Gpt,
        };

        let mut sorted_specs = state.disks_partitions.clone();
        sorted_specs.sort_by(|a, b| {
            let sa = a
                .start
                .as_ref()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let sb = b
                .start
                .as_ref()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            sa.cmp(&sb)
        });

        let mut partitions = Vec::new();
        let mut mounts = Vec::new();
        let mut part_num: u32 = 1;

        for spec in &sorted_specs {
            if let Some(spec_device) = &spec.name
                && spec_device != &device_path
            {
                continue;
            }

            let role_str = spec.role.as_deref().unwrap_or("OTHER");
            let role = PartitionRole::from_str_role(role_str);
            let fs = spec.fs.as_deref().unwrap_or("ext4");
            let start = spec.start.as_deref().unwrap_or("0");
            let size = spec.size.as_deref().unwrap_or("100%");

            let start_str = bytes_to_parted_unit(start);
            let end_str = bytes_to_parted_unit(size);

            let mut flags = Vec::new();
            match role {
                PartitionRole::Esp => flags.push(PartitionFlag::Esp),
                PartitionRole::BiosBoot => flags.push(PartitionFlag::BiosGrub),
                _ => {}
            }

            let encryption = if spec.encrypt.unwrap_or(false) {
                let mapper = match role {
                    PartitionRole::Root => "cryptroot",
                    PartitionRole::Home => "crypthome",
                    _ => "cryptdev",
                };
                let pw = if state.disk_encryption_password.is_empty() {
                    None
                } else {
                    Some(state.disk_encryption_password.clone())
                };
                Some(EncryptionSpec {
                    method: EncryptionMethod::Luks2,
                    mapper_name: mapper.into(),
                    passphrase: pw,
                })
            } else {
                None
            };

            let is_btrfs_root = role == PartitionRole::Root && fs == "btrfs";
            let subvolumes = if is_btrfs_root {
                let preset = match state.btrfs_subvolume_preset {
                    1 => BtrfsSubvolumePreset::Standard,
                    2 => BtrfsSubvolumePreset::Extended,
                    _ => BtrfsSubvolumePreset::Flat,
                };
                preset.subvolumes()
            } else {
                vec![]
            };

            partitions.push(PlannedPartition {
                number: part_num,
                role: role.clone(),
                start: start_str,
                end: end_str,
                filesystem: FilesystemSpec {
                    fstype: fs.into(),
                    mkfs_options: vec![],
                },
                flags,
                encryption: encryption.clone(),
                subvolumes: subvolumes.clone(),
            });

            let part_path = StoragePlan::partition_path(&device_path, part_num);
            let source = if let Some(ref enc) = encryption {
                format!("/dev/mapper/{}", enc.mapper_name)
            } else {
                part_path
            };

            match role {
                PartitionRole::Swap => {
                    mounts.push(PlannedMount {
                        source,
                        target: String::new(),
                        fstype: "swap".into(),
                        options: vec![],
                        is_swap: true,
                        subvolume: None,
                    });
                }
                PartitionRole::BiosBoot => {}
                _ if !subvolumes.is_empty() => {
                    let mut subvol_mounts: Vec<PlannedMount> = subvolumes
                        .iter()
                        .map(|sv| {
                            let target = if sv.mountpoint == "/" {
                                "/mnt".into()
                            } else {
                                format!("/mnt{}", sv.mountpoint)
                            };
                            PlannedMount {
                                source: source.clone(),
                                target,
                                fstype: "btrfs".into(),
                                options: sv.mount_options.clone(),
                                is_swap: false,
                                subvolume: Some(sv.name.clone()),
                            }
                        })
                        .collect();
                    subvol_mounts.sort_by(|a, b| {
                        if a.target == "/mnt" {
                            return std::cmp::Ordering::Less;
                        }
                        if b.target == "/mnt" {
                            return std::cmp::Ordering::Greater;
                        }
                        a.target.cmp(&b.target)
                    });
                    mounts.extend(subvol_mounts);
                }
                _ => {
                    let mountpoint = spec.mountpoint.as_deref().unwrap_or(match role {
                        PartitionRole::Esp => "/boot",
                        PartitionRole::Root => "/",
                        PartitionRole::Home => "/home",
                        PartitionRole::Var => "/var",
                        _ => "/mnt/unknown",
                    });

                    let target = if mountpoint == "/" {
                        "/mnt".into()
                    } else if mountpoint.starts_with('/') {
                        format!("/mnt{mountpoint}")
                    } else {
                        format!("/mnt/{mountpoint}")
                    };

                    let mount_fstype = if fs == "fat32" || fs == "fat16" || fs == "fat12" {
                        "vfat".into()
                    } else {
                        fs.to_string()
                    };

                    let options: Vec<String> = spec
                        .mount_options
                        .as_ref()
                        .map(|o| o.split(',').map(|s| s.trim().to_string()).collect())
                        .unwrap_or_default();

                    mounts.push(PlannedMount {
                        source,
                        target,
                        fstype: mount_fstype,
                        options,
                        is_swap: false,
                        subvolume: None,
                    });
                }
            }

            part_num += 1;
        }

        // Sort mounts: root first, then by path depth
        mounts.sort_by(|a, b| {
            if a.is_swap != b.is_swap {
                return a.is_swap.cmp(&b.is_swap);
            }
            if a.target == "/mnt" {
                return std::cmp::Ordering::Less;
            }
            if b.target == "/mnt" {
                return std::cmp::Ordering::Greater;
            }
            a.target.cmp(&b.target)
        });

        let device = PlannedDevice {
            path: device_path,
            label,
            wipe: state.disks_wipe,
            partitions,
        };

        Ok(StoragePlan {
            devices: vec![device],
            mounts,
            mode: StorageMode::Manual,
            stacks: vec![],
        })
    }

    fn compile_pre_mounted(state: &AppState) -> Result<StoragePlan, Vec<ValidationError>> {
        let mut mounts = Vec::new();

        // Detect mounts under /mnt via findmnt -J -R
        let findmnt_result = std::process::Command::new("findmnt")
            .args(["-J", "-R", "--target", "/mnt"])
            .output();

        match findmnt_result {
            Ok(out) if out.status.success() => {
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout)
                    && let Some(filesystems) = json.get("filesystems").and_then(|v| v.as_array())
                {
                    collect_findmnt_mounts(filesystems, &mut mounts);
                }
            }
            _ => {
                return Err(vec![ValidationError {
                    message: "Failed to detect mounts under /mnt (findmnt failed). \
                              Ensure filesystems are mounted at /mnt before selecting pre-mounted mode."
                        .into(),
                }]);
            }
        }

        // Detect active swap devices
        if let Ok(out) = std::process::Command::new("swapon")
            .args(["--raw", "--noheadings"])
            .output()
            && out.status.success()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if let Some(device) = line.split_whitespace().next() {
                    mounts.push(PlannedMount {
                        source: device.to_string(),
                        target: String::new(),
                        fstype: "swap".into(),
                        options: vec![],
                        is_swap: true,
                        subvolume: None,
                    });
                }
            }
        }

        // Pre-return validation with detailed messages
        let mut errors = Vec::new();

        let has_root = mounts.iter().any(|m| m.target == "/mnt");
        if !has_root {
            errors.push(ValidationError {
                message: "Pre-mounted mode requires a filesystem mounted at /mnt (root)".into(),
            });
        }

        let is_uefi = state.is_uefi();
        if is_uefi {
            let has_boot = mounts
                .iter()
                .any(|m| m.target == "/mnt/boot" || m.target == "/mnt/efi");
            if !has_boot {
                errors.push(ValidationError {
                    message: "UEFI system detected but no ESP mounted at /mnt/boot or /mnt/efi"
                        .into(),
                });
            }
        }

        if state.swap_enabled && !mounts.iter().any(|m| m.is_swap) {
            errors.push(ValidationError {
                message: "Swap is enabled but no active swap device detected".into(),
            });
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Sort: non-swap first with root (/mnt) at top, then by path depth
        mounts.sort_by(|a, b| {
            if a.is_swap != b.is_swap {
                return a.is_swap.cmp(&b.is_swap);
            }
            if a.target == "/mnt" {
                return std::cmp::Ordering::Less;
            }
            if b.target == "/mnt" {
                return std::cmp::Ordering::Greater;
            }
            a.target.cmp(&b.target)
        });

        Ok(StoragePlan {
            devices: vec![],
            mounts,
            mode: StorageMode::PreMounted,
            stacks: vec![],
        })
    }
}

/// Recursively collect mount entries from findmnt JSON output into PlannedMount list.
fn collect_findmnt_mounts(fs_array: &[serde_json::Value], mounts: &mut Vec<PlannedMount>) {
    for fs in fs_array {
        let target = fs.get("target").and_then(|v| v.as_str()).unwrap_or("");
        let source = fs.get("source").and_then(|v| v.as_str()).unwrap_or("");
        let fstype = fs.get("fstype").and_then(|v| v.as_str()).unwrap_or("");
        let options_str = fs.get("options").and_then(|v| v.as_str()).unwrap_or("");

        if !target.is_empty() && !source.is_empty() {
            let opts: Vec<String> = if options_str.is_empty() {
                vec![]
            } else {
                options_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            };

            let subvolume = opts
                .iter()
                .find(|o| o.starts_with("subvol="))
                .and_then(|o| o.strip_prefix("subvol="))
                .map(|s| s.to_string());

            mounts.push(PlannedMount {
                source: source.to_string(),
                target: target.to_string(),
                fstype: fstype.to_string(),
                options: opts,
                is_swap: false,
                subvolume,
            });
        }

        if let Some(children) = fs.get("children").and_then(|v| v.as_array()) {
            collect_findmnt_mounts(children, mounts);
        }
    }
}

fn bytes_to_parted_unit(bytes_str: &str) -> String {
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

    if let Ok(bytes) = bytes_str.parse::<u64>() {
        let mib = bytes / (1024 * 1024);
        if mib == 0 {
            "1MiB".to_string()
        } else {
            format!("{mib}MiB")
        }
    } else {
        bytes_str.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_auto_uefi(swap: bool, luks: bool) -> StoragePlan {
        // Build a plan manually matching what the planner would produce for UEFI
        let device_path = "/dev/sda";
        let mut partitions = Vec::new();
        let mut mounts = Vec::new();
        let mut part_num: u32 = 1;

        // ESP
        partitions.push(PlannedPartition {
            number: part_num,
            role: PartitionRole::Esp,
            start: "1MiB".into(),
            end: "1025MiB".into(),
            filesystem: FilesystemSpec {
                fstype: "fat32".into(),
                mkfs_options: vec![],
            },
            flags: vec![PartitionFlag::Esp],
            encryption: None,
            subvolumes: vec![],
        });
        mounts.push(PlannedMount {
            source: format!("{device_path}{part_num}"),
            target: "/mnt/boot".into(),
            fstype: "vfat".into(),
            options: vec![],
            is_swap: false,
            subvolume: None,
        });
        part_num += 1;

        if swap {
            partitions.push(PlannedPartition {
                number: part_num,
                role: PartitionRole::Swap,
                start: "1025MiB".into(),
                end: "5121MiB".into(),
                filesystem: FilesystemSpec {
                    fstype: "linux-swap".into(),
                    mkfs_options: vec![],
                },
                flags: vec![],
                encryption: None,
                subvolumes: vec![],
            });
            mounts.push(PlannedMount {
                source: format!("{device_path}{part_num}"),
                target: String::new(),
                fstype: "swap".into(),
                options: vec![],
                is_swap: true,
                subvolume: None,
            });
            part_num += 1;
        }

        let root_start = if swap { "5121MiB" } else { "1025MiB" };
        let encryption = if luks {
            Some(EncryptionSpec {
                method: EncryptionMethod::Luks2,
                mapper_name: "cryptroot".into(),
                passphrase: None,
            })
        } else {
            None
        };

        partitions.push(PlannedPartition {
            number: part_num,
            role: PartitionRole::Root,
            start: root_start.into(),
            end: "100%".into(),
            filesystem: FilesystemSpec {
                fstype: "btrfs".into(),
                mkfs_options: vec![],
            },
            flags: vec![],
            encryption: encryption.clone(),
            subvolumes: vec![],
        });

        let root_source = if luks {
            "/dev/mapper/cryptroot".into()
        } else {
            format!("{device_path}{part_num}")
        };

        mounts.insert(
            0,
            PlannedMount {
                source: root_source,
                target: "/mnt".into(),
                fstype: "btrfs".into(),
                options: vec![],
                is_swap: false,
                subvolume: None,
            },
        );

        let device = PlannedDevice {
            path: device_path.into(),
            label: DiskLabel::Gpt,
            wipe: true,
            partitions,
        };

        StoragePlan {
            devices: vec![device],
            mounts,
            mode: StorageMode::Automatic,
            stacks: vec![],
        }
    }

    fn compile_auto_bios(swap: bool, luks: bool) -> StoragePlan {
        let device_path = "/dev/sda";
        let mut partitions = Vec::new();
        let mut mounts = Vec::new();
        let mut part_num: u32 = 1;

        // BIOS boot
        partitions.push(PlannedPartition {
            number: part_num,
            role: PartitionRole::BiosBoot,
            start: "1MiB".into(),
            end: "2MiB".into(),
            filesystem: FilesystemSpec {
                fstype: "biosboot".into(),
                mkfs_options: vec![],
            },
            flags: vec![PartitionFlag::BiosGrub],
            encryption: None,
            subvolumes: vec![],
        });
        part_num += 1;

        if swap {
            partitions.push(PlannedPartition {
                number: part_num,
                role: PartitionRole::Swap,
                start: "2MiB".into(),
                end: "4098MiB".into(),
                filesystem: FilesystemSpec {
                    fstype: "linux-swap".into(),
                    mkfs_options: vec![],
                },
                flags: vec![],
                encryption: None,
                subvolumes: vec![],
            });
            mounts.push(PlannedMount {
                source: format!("{device_path}{part_num}"),
                target: String::new(),
                fstype: "swap".into(),
                options: vec![],
                is_swap: true,
                subvolume: None,
            });
            part_num += 1;
        }

        let root_start = if swap { "4098MiB" } else { "2MiB" };
        let encryption = if luks {
            Some(EncryptionSpec {
                method: EncryptionMethod::Luks2,
                mapper_name: "cryptroot".into(),
                passphrase: None,
            })
        } else {
            None
        };

        partitions.push(PlannedPartition {
            number: part_num,
            role: PartitionRole::Root,
            start: root_start.into(),
            end: "100%".into(),
            filesystem: FilesystemSpec {
                fstype: "btrfs".into(),
                mkfs_options: vec![],
            },
            flags: vec![],
            encryption: encryption.clone(),
            subvolumes: vec![],
        });

        let root_source = if luks {
            "/dev/mapper/cryptroot".into()
        } else {
            format!("{device_path}{part_num}")
        };

        mounts.insert(
            0,
            PlannedMount {
                source: root_source,
                target: "/mnt".into(),
                fstype: "btrfs".into(),
                options: vec![],
                is_swap: false,
                subvolume: None,
            },
        );

        let device = PlannedDevice {
            path: device_path.into(),
            label: DiskLabel::Gpt,
            wipe: true,
            partitions,
        };

        StoragePlan {
            devices: vec![device],
            mounts,
            mode: StorageMode::Automatic,
            stacks: vec![],
        }
    }

    // ── Partition command tests ──

    #[test]
    fn test_partition_cmds_uefi_swap_no_luks() {
        let plan = compile_auto_uefi(true, false);
        let cmds = plan.partition_commands();
        let device = "/dev/sda";

        assert_eq!(cmds[0], format!("wipefs -a {device}"));
        assert_eq!(cmds[1], format!("parted -s {device} mklabel gpt"));
        assert_eq!(cmds[2], format!("partprobe {device} || true"));
        assert_eq!(cmds[3], "udevadm settle");
        assert_eq!(
            cmds[4],
            format!("parted -s {device} mkpart ESP fat32 1MiB 1025MiB")
        );
        assert_eq!(cmds[5], format!("parted -s {device} set 1 esp on"));
        assert_eq!(cmds[6], format!("mkfs.fat -F 32 {device}1"));
        assert_eq!(
            cmds[7],
            format!("parted -s {device} mkpart swap linux-swap 1025MiB 5121MiB")
        );
        assert_eq!(cmds[8], format!("mkswap {device}2"));
        assert_eq!(
            cmds[9],
            format!("parted -s {device} mkpart root btrfs 5121MiB 100%")
        );
        assert_eq!(cmds[10], format!("mkfs.btrfs -f {device}3"));
    }

    #[test]
    fn test_partition_cmds_uefi_swap_luks() {
        let plan = compile_auto_uefi(true, true);
        let cmds = plan.partition_commands();
        let device = "/dev/sda";

        assert_eq!(
            cmds[9],
            format!("parted -s {device} mkpart root btrfs 5121MiB 100%")
        );
        assert_eq!(
            cmds[10],
            "modprobe -q dm_crypt 2>/dev/null || modprobe -q dm-crypt 2>/dev/null || true"
        );
        assert_eq!(
            cmds[11],
            format!("cryptsetup luksFormat --type luks2 -q {device}3")
        );
        assert_eq!(cmds[12], "udevadm settle");
        assert_eq!(
            cmds[13],
            format!("cryptsetup open --type luks {device}3 cryptroot")
        );
        assert_eq!(cmds[14], "mkfs.btrfs -f /dev/mapper/cryptroot");
    }

    #[test]
    fn test_partition_cmds_uefi_no_swap_no_luks() {
        let plan = compile_auto_uefi(false, false);
        let cmds = plan.partition_commands();
        let device = "/dev/sda";

        assert_eq!(
            cmds[4],
            format!("parted -s {device} mkpart ESP fat32 1MiB 1025MiB")
        );
        assert_eq!(cmds[5], format!("parted -s {device} set 1 esp on"));
        assert_eq!(cmds[6], format!("mkfs.fat -F 32 {device}1"));
        assert_eq!(
            cmds[7],
            format!("parted -s {device} mkpart root btrfs 1025MiB 100%")
        );
        assert_eq!(cmds[8], format!("mkfs.btrfs -f {device}2"));
    }

    #[test]
    fn test_partition_cmds_bios_swap_no_luks() {
        let plan = compile_auto_bios(true, false);
        let cmds = plan.partition_commands();
        let device = "/dev/sda";

        assert_eq!(cmds[0], format!("wipefs -a {device}"));
        assert_eq!(cmds[1], format!("parted -s {device} mklabel gpt"));
        assert_eq!(cmds[2], format!("partprobe {device} || true"));
        assert_eq!(cmds[3], "udevadm settle");
        assert_eq!(
            cmds[4],
            format!("parted -s {device} mkpart biosboot 1MiB 2MiB")
        );
        assert_eq!(cmds[5], format!("parted -s {device} set 1 bios_grub on"));
        // no mkfs for biosboot — next is swap
        assert_eq!(
            cmds[6],
            format!("parted -s {device} mkpart swap linux-swap 2MiB 4098MiB")
        );
        assert_eq!(cmds[7], format!("mkswap {device}2"));
        assert_eq!(
            cmds[8],
            format!("parted -s {device} mkpart root btrfs 4098MiB 100%")
        );
        assert_eq!(cmds[9], format!("mkfs.btrfs -f {device}3"));
    }

    #[test]
    fn test_partition_cmds_bios_swap_luks() {
        let plan = compile_auto_bios(true, true);
        let cmds = plan.partition_commands();
        let device = "/dev/sda";

        assert_eq!(
            cmds[8],
            format!("parted -s {device} mkpart root btrfs 4098MiB 100%")
        );
        assert_eq!(
            cmds[9],
            "modprobe -q dm_crypt 2>/dev/null || modprobe -q dm-crypt 2>/dev/null || true"
        );
        assert_eq!(
            cmds[10],
            format!("cryptsetup luksFormat --type luks2 -q {device}3")
        );
        assert_eq!(cmds[11], "udevadm settle");
        assert_eq!(
            cmds[12],
            format!("cryptsetup open --type luks {device}3 cryptroot")
        );
        assert_eq!(cmds[13], "mkfs.btrfs -f /dev/mapper/cryptroot");
    }

    // ── Mount command tests ──

    #[test]
    fn test_mount_cmds_uefi_swap_no_luks() {
        let plan = compile_auto_uefi(true, false);
        let cmds = plan.mount_commands();

        assert_eq!(cmds[0], "mkdir -p /mnt");
        assert_eq!(cmds[1], "mount /dev/sda3 /mnt");
        assert_eq!(cmds[2], "modprobe -q fat || true");
        assert_eq!(cmds[3], "modprobe -q vfat || true");
        assert_eq!(cmds[4], "modprobe -q msdos || true");
        assert_eq!(cmds[5], "modprobe -q nls_cp437 || true");
        assert_eq!(cmds[6], "modprobe -q nls_iso8859_1 || true");
        assert_eq!(cmds[7], "modprobe -q nls_ascii || true");
        assert_eq!(
            cmds[8],
            "{ mount -t vfat --mkdir /dev/sda1 /mnt/boot || mount -t fat --mkdir /dev/sda1 /mnt/boot || mount -t msdos --mkdir /dev/sda1 /mnt/boot || mount --mkdir /dev/sda1 /mnt/boot; } || { echo 'ERROR: Failed to mount /dev/sda1 - ensure FAT/vfat filesystem support is available (check: grep -E \"vfat|fat|msdos\" /proc/filesystems)' >&2; exit 1; }"
        );
        assert_eq!(cmds[9], "swapon /dev/sda2");
    }

    #[test]
    fn test_mount_cmds_uefi_swap_luks() {
        let plan = compile_auto_uefi(true, true);
        let cmds = plan.mount_commands();

        assert_eq!(cmds[0], "mkdir -p /mnt");
        assert_eq!(cmds[1], "mount /dev/mapper/cryptroot /mnt");
        assert_eq!(cmds[2], "modprobe -q fat || true");
        assert_eq!(cmds[3], "modprobe -q vfat || true");
        assert_eq!(cmds[4], "modprobe -q msdos || true");
        assert_eq!(cmds[5], "modprobe -q nls_cp437 || true");
        assert_eq!(cmds[6], "modprobe -q nls_iso8859_1 || true");
        assert_eq!(cmds[7], "modprobe -q nls_ascii || true");
        assert_eq!(
            cmds[8],
            "{ mount -t vfat --mkdir /dev/sda1 /mnt/boot || mount -t fat --mkdir /dev/sda1 /mnt/boot || mount -t msdos --mkdir /dev/sda1 /mnt/boot || mount --mkdir /dev/sda1 /mnt/boot; } || { echo 'ERROR: Failed to mount /dev/sda1 - ensure FAT/vfat filesystem support is available (check: grep -E \"vfat|fat|msdos\" /proc/filesystems)' >&2; exit 1; }"
        );
        assert_eq!(cmds[9], "swapon /dev/sda2");
    }

    #[test]
    fn test_mount_cmds_bios_swap_no_luks() {
        let plan = compile_auto_bios(true, false);
        let cmds = plan.mount_commands();

        assert_eq!(cmds[0], "mkdir -p /mnt");
        assert_eq!(cmds[1], "mount /dev/sda3 /mnt");
        assert_eq!(cmds[2], "swapon /dev/sda2");
        assert_eq!(cmds.len(), 3);
    }

    #[test]
    fn test_mount_cmds_uefi_no_swap() {
        let plan = compile_auto_uefi(false, false);
        let cmds = plan.mount_commands();

        assert_eq!(cmds[0], "mkdir -p /mnt");
        assert_eq!(cmds[1], "mount /dev/sda2 /mnt");
        assert_eq!(cmds[2], "modprobe -q fat || true");
        assert_eq!(cmds[3], "modprobe -q vfat || true");
        assert_eq!(cmds[4], "modprobe -q msdos || true");
        assert_eq!(cmds[5], "modprobe -q nls_cp437 || true");
        assert_eq!(cmds[6], "modprobe -q nls_iso8859_1 || true");
        assert_eq!(cmds[7], "modprobe -q nls_ascii || true");
        assert_eq!(
            cmds[8],
            "{ mount -t vfat --mkdir /dev/sda1 /mnt/boot || mount -t fat --mkdir /dev/sda1 /mnt/boot || mount -t msdos --mkdir /dev/sda1 /mnt/boot || mount --mkdir /dev/sda1 /mnt/boot; } || { echo 'ERROR: Failed to mount /dev/sda1 - ensure FAT/vfat filesystem support is available (check: grep -E \"vfat|fat|msdos\" /proc/filesystems)' >&2; exit 1; }"
        );
        assert_eq!(cmds.len(), 9);
    }

    // ── Fstab command tests ──

    #[test]
    fn test_fstab_cmds_uefi_swap_no_luks() {
        let plan = compile_auto_uefi(true, false);
        let cmds = plan.fstab_check_commands();

        assert!(cmds.iter().any(|c| c.contains("ESP not detected")));
        assert!(cmds.iter().any(|c| c.contains("swap not found")));
        assert!(cmds.iter().any(|c| c.contains("btrfs")));
        assert!(cmds.iter().any(|c| c.contains("mountpoint -q /mnt")));
        assert!(cmds.iter().any(|c| c.contains("mountpoint -q /mnt/boot")));
        assert!(cmds.iter().any(|c| c.contains("swap not active")));
        assert!(cmds.last().unwrap().contains("genfstab"));
    }

    #[test]
    fn test_fstab_cmds_uefi_swap_luks() {
        let plan = compile_auto_uefi(true, true);
        let cmds = plan.fstab_check_commands();

        assert!(
            cmds.iter()
                .any(|c| c.contains("/dev/mapper/cryptroot") && c.contains("btrfs"))
        );
        assert!(cmds.last().unwrap().contains("genfstab"));
    }

    // ── Validation tests ──

    #[test]
    fn test_validation_missing_root() {
        let plan = StoragePlan {
            devices: vec![PlannedDevice {
                path: "/dev/sda".into(),
                label: DiskLabel::Gpt,
                wipe: true,
                partitions: vec![PlannedPartition {
                    number: 1,
                    role: PartitionRole::Esp,
                    start: "1MiB".into(),
                    end: "1025MiB".into(),
                    filesystem: FilesystemSpec {
                        fstype: "fat32".into(),
                        mkfs_options: vec![],
                    },
                    flags: vec![PartitionFlag::Esp],
                    encryption: None,
                    subvolumes: vec![],
                }],
            }],
            mounts: vec![PlannedMount {
                source: "/dev/sda1".into(),
                target: "/mnt/boot".into(),
                fstype: "vfat".into(),
                options: vec![],
                is_swap: false,
                subvolume: None,
            }],
            mode: StorageMode::Automatic,
            stacks: vec![],
        };

        let errors = plan.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("root")));
    }

    #[test]
    fn test_validation_duplicate_mountpoints() {
        let plan = compile_auto_uefi(true, false);
        let mut bad_plan = plan;
        bad_plan.mounts.push(PlannedMount {
            source: "/dev/sda99".into(),
            target: "/mnt".into(),
            fstype: "ext4".into(),
            options: vec![],
            is_swap: false,
            subvolume: None,
        });

        let errors = bad_plan.validate();
        assert!(errors.iter().any(|e| e.message.contains("Duplicate")));
    }

    #[test]
    fn test_nvme_partition_path() {
        assert_eq!(
            StoragePlan::partition_path("/dev/nvme0n1", 1),
            "/dev/nvme0n1p1"
        );
        assert_eq!(StoragePlan::partition_path("/dev/sda", 3), "/dev/sda3");
    }

    #[test]
    fn test_plan_accessors() {
        let plan = compile_auto_uefi(true, false);
        assert_eq!(plan.root_device_path(), Some("/dev/sda3".into()));
        assert_eq!(plan.esp_device_path(), Some("/dev/sda1".into()));
        assert!(!plan.has_encryption());

        let plan_luks = compile_auto_uefi(true, true);
        assert_eq!(
            plan_luks.root_device_path(),
            Some("/dev/mapper/cryptroot".into())
        );
        assert!(plan_luks.has_encryption());
    }

    // ── Manual mode tests ──

    fn make_manual_state() -> crate::core::state::AppState {
        let mut state = crate::core::state::AppState::new(true);
        state.disks_mode_index = 1;
        state.disks_selected_device = Some("/dev/sda".into());
        state
    }

    #[test]
    fn test_manual_esp_root() {
        let mut state = make_manual_state();
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("BOOT".into()),
                fs: Some("fat32".into()),
                start: Some("1048576".into()),   // 1 MiB
                size: Some("1073741824".into()), // 1 GiB
                mountpoint: Some("/boot".into()),
                ..Default::default()
            });
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("ROOT".into()),
                fs: Some("btrfs".into()),
                start: Some("1074790400".into()), // ~1025 MiB
                size: Some("100%".into()),
                mountpoint: Some("/".into()),
                ..Default::default()
            });

        let plan = StoragePlanner::compile(&state).expect("should compile");
        assert_eq!(plan.mode, StorageMode::Manual);
        assert_eq!(plan.devices.len(), 1);
        assert_eq!(plan.devices[0].partitions.len(), 2);
        assert_eq!(plan.devices[0].partitions[0].role, PartitionRole::Esp);
        assert_eq!(plan.devices[0].partitions[1].role, PartitionRole::Root);

        // Mounts: root first, then boot
        assert_eq!(plan.mounts[0].target, "/mnt");
        assert_eq!(plan.mounts[1].target, "/mnt/boot");

        // Commands should include ESP and root formatting
        let cmds = plan.partition_commands();
        let joined = cmds.join("\n");
        assert!(joined.contains("mkfs.fat -F 32"), "{joined}");
        assert!(joined.contains("mkfs.btrfs -f"), "{joined}");
    }

    #[test]
    fn test_manual_esp_swap_root() {
        let mut state = make_manual_state();
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("BOOT".into()),
                fs: Some("fat32".into()),
                start: Some("1048576".into()),
                size: Some("1073741824".into()),
                mountpoint: Some("/boot".into()),
                ..Default::default()
            });
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("SWAP".into()),
                fs: Some("linux-swap".into()),
                start: Some("1074790400".into()),
                size: Some("4294967296".into()), // 4 GiB
                ..Default::default()
            });
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("ROOT".into()),
                fs: Some("ext4".into()),
                start: Some("5369757696".into()),
                size: Some("100%".into()),
                mountpoint: Some("/".into()),
                ..Default::default()
            });

        let plan = StoragePlanner::compile(&state).expect("should compile");
        assert_eq!(plan.devices[0].partitions.len(), 3);
        assert!(plan.mounts.iter().any(|m| m.is_swap));
        assert!(plan.mounts.iter().any(|m| m.target == "/mnt"));
        assert!(plan.mounts.iter().any(|m| m.target == "/mnt/boot"));

        let mount_cmds = plan.mount_commands();
        let joined = mount_cmds.join("\n");
        assert!(joined.contains("mount"), "{joined}");
        assert!(joined.contains("swapon"), "{joined}");
    }

    #[test]
    fn test_manual_bios_root() {
        let mut state = make_manual_state();
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("BIOS_BOOT".into()),
                fs: Some("biosboot".into()),
                start: Some("1048576".into()),
                size: Some("1048576".into()), // 1 MiB
                ..Default::default()
            });
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("ROOT".into()),
                fs: Some("ext4".into()),
                start: Some("2097152".into()),
                size: Some("100%".into()),
                mountpoint: Some("/".into()),
                ..Default::default()
            });

        let plan = StoragePlanner::compile(&state).expect("should compile");
        assert_eq!(plan.devices[0].partitions[0].role, PartitionRole::BiosBoot);
        assert_eq!(plan.devices[0].partitions[1].role, PartitionRole::Root);
        assert_eq!(plan.mounts.len(), 1);
        assert_eq!(plan.mounts[0].target, "/mnt");
    }

    #[test]
    fn test_manual_root_with_encryption() {
        let mut state = make_manual_state();
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("BOOT".into()),
                fs: Some("fat32".into()),
                start: Some("1048576".into()),
                size: Some("1073741824".into()),
                mountpoint: Some("/boot".into()),
                ..Default::default()
            });
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("ROOT".into()),
                fs: Some("btrfs".into()),
                start: Some("1074790400".into()),
                size: Some("100%".into()),
                mountpoint: Some("/".into()),
                encrypt: Some(true),
                ..Default::default()
            });

        let plan = StoragePlanner::compile(&state).expect("should compile");
        assert!(plan.has_encryption());
        assert_eq!(
            plan.root_device_path(),
            Some("/dev/mapper/cryptroot".into())
        );

        let cmds = plan.partition_commands();
        let joined = cmds.join("\n");
        assert!(joined.contains("cryptsetup luksFormat"), "{joined}");
        assert!(joined.contains("cryptsetup open"), "{joined}");
        assert!(
            joined.contains("mkfs.btrfs -f /dev/mapper/cryptroot"),
            "{joined}"
        );
    }

    #[test]
    fn test_manual_rejects_empty_partitions() {
        let state = make_manual_state();
        let err = StoragePlanner::compile(&state).unwrap_err();
        assert!(
            err.iter()
                .any(|e| e.message.contains("No partitions defined"))
        );
    }

    #[test]
    fn test_manual_rejects_missing_role() {
        let mut state = make_manual_state();
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: None,
                fs: Some("ext4".into()),
                start: Some("1048576".into()),
                size: Some("100%".into()),
                mountpoint: Some("/".into()),
                ..Default::default()
            });

        let err = StoragePlanner::compile(&state).unwrap_err();
        assert!(err.iter().any(|e| e.message.contains("missing role")));
    }

    #[test]
    fn test_manual_rejects_missing_start() {
        let mut state = make_manual_state();
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("ROOT".into()),
                fs: Some("ext4".into()),
                start: None,
                size: Some("100%".into()),
                mountpoint: Some("/".into()),
                ..Default::default()
            });

        let err = StoragePlanner::compile(&state).unwrap_err();
        assert!(err.iter().any(|e| e.message.contains("missing start")));
    }

    #[test]
    fn test_manual_rejects_missing_mountpoint() {
        let mut state = make_manual_state();
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("ROOT".into()),
                fs: Some("ext4".into()),
                start: Some("1048576".into()),
                size: Some("100%".into()),
                mountpoint: None,
                ..Default::default()
            });

        let err = StoragePlanner::compile(&state).unwrap_err();
        assert!(err.iter().any(|e| e.message.contains("missing mountpoint")));
    }

    #[test]
    fn test_manual_swap_no_mountpoint_ok() {
        let mut state = make_manual_state();
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("BOOT".into()),
                fs: Some("fat32".into()),
                start: Some("1048576".into()),
                size: Some("1073741824".into()),
                mountpoint: Some("/boot".into()),
                ..Default::default()
            });
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("SWAP".into()),
                fs: Some("linux-swap".into()),
                start: Some("1074790400".into()),
                size: Some("4294967296".into()),
                mountpoint: None,
                ..Default::default()
            });
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("ROOT".into()),
                fs: Some("btrfs".into()),
                start: Some("5369757696".into()),
                size: Some("100%".into()),
                mountpoint: Some("/".into()),
                ..Default::default()
            });

        let plan = StoragePlanner::compile(&state).expect("swap without mountpoint should be fine");
        assert!(plan.mounts.iter().any(|m| m.is_swap));
    }

    // ── Btrfs subvolume tests ──

    #[test]
    fn test_btrfs_flat_unchanged() {
        let mut state = crate::core::state::AppState::new(true);
        state.disks_selected_device = Some("/dev/sda".into());
        state.btrfs_subvolume_preset = 0; // Flat
        state.swap_enabled = true;

        let plan = StoragePlanner::compile(&state).expect("flat should compile");
        let root_part = plan.devices[0]
            .partitions
            .iter()
            .find(|p| p.role == PartitionRole::Root)
            .unwrap();
        assert!(root_part.subvolumes.is_empty());

        let root_mount = plan.mounts.iter().find(|m| m.target == "/mnt").unwrap();
        assert!(root_mount.subvolume.is_none());
        assert!(root_mount.options.is_empty());

        let cmds = plan.partition_commands();
        let joined = cmds.join("\n");
        assert!(!joined.contains("btrfs subvolume create"), "{joined}");
    }

    #[test]
    fn test_btrfs_standard_subvolumes() {
        let mut state = crate::core::state::AppState::new(true);
        state.disks_selected_device = Some("/dev/sda".into());
        state.btrfs_subvolume_preset = 1; // Standard
        state.swap_enabled = true;

        let plan = StoragePlanner::compile(&state).expect("standard should compile");
        let root_part = plan.devices[0]
            .partitions
            .iter()
            .find(|p| p.role == PartitionRole::Root)
            .unwrap();
        assert_eq!(root_part.subvolumes.len(), 3);
        let names: Vec<&str> = root_part
            .subvolumes
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert!(names.contains(&"@"));
        assert!(names.contains(&"@home"));
        assert!(names.contains(&"@snapshots"));

        // Partition commands should contain subvolume creation
        let part_cmds = plan.partition_commands();
        let joined = part_cmds.join("\n");
        assert!(
            joined.contains("btrfs subvolume create /mnt/@\n"),
            "{joined}"
        );
        assert!(
            joined.contains("btrfs subvolume create /mnt/@home"),
            "{joined}"
        );
        assert!(
            joined.contains("btrfs subvolume create /mnt/@snapshots"),
            "{joined}"
        );
        assert!(joined.contains("umount /mnt"), "{joined}");

        // Mount commands should contain subvol= options
        let mount_cmds = plan.mount_commands();
        let mjoined = mount_cmds.join("\n");
        assert!(mjoined.contains("subvol=@"), "{mjoined}");
        assert!(mjoined.contains("subvol=@home"), "{mjoined}");
        assert!(mjoined.contains("subvol=@snapshots"), "{mjoined}");
        assert!(mjoined.contains("compress=zstd"), "{mjoined}");
        assert!(mjoined.contains("noatime"), "{mjoined}");

        // Mount order: root first, then /.snapshots, /home
        let non_swap_mounts: Vec<&PlannedMount> =
            plan.mounts.iter().filter(|m| !m.is_swap).collect();
        assert_eq!(non_swap_mounts[0].target, "/mnt");
        assert_eq!(non_swap_mounts[0].subvolume.as_deref(), Some("@"));
    }

    #[test]
    fn test_btrfs_extended_subvolumes() {
        let mut state = crate::core::state::AppState::new(true);
        state.disks_selected_device = Some("/dev/sda".into());
        state.btrfs_subvolume_preset = 2; // Extended
        state.swap_enabled = false;

        let plan = StoragePlanner::compile(&state).expect("extended should compile");
        let root_part = plan.devices[0]
            .partitions
            .iter()
            .find(|p| p.role == PartitionRole::Root)
            .unwrap();
        assert_eq!(root_part.subvolumes.len(), 4);
        let names: Vec<&str> = root_part
            .subvolumes
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert!(names.contains(&"@var_log"));

        let mount_cmds = plan.mount_commands();
        let mjoined = mount_cmds.join("\n");
        assert!(mjoined.contains("subvol=@var_log"), "{mjoined}");
        assert!(mjoined.contains("/mnt/var/log"), "{mjoined}");
    }

    #[test]
    fn test_btrfs_standard_with_luks() {
        let mut state = crate::core::state::AppState::new(true);
        state.disks_selected_device = Some("/dev/sda".into());
        state.btrfs_subvolume_preset = 1; // Standard
        state.swap_enabled = true;
        state.disk_encryption_type_index = 1; // LUKS

        let plan = StoragePlanner::compile(&state).expect("standard+luks should compile");
        assert!(plan.has_encryption());

        let part_cmds = plan.partition_commands();
        let joined = part_cmds.join("\n");
        // Should mount the mapper device for subvolume creation
        assert!(
            joined.contains("mount /dev/mapper/cryptroot /mnt"),
            "{joined}"
        );
        assert!(joined.contains("btrfs subvolume create /mnt/@"), "{joined}");

        let mount_cmds = plan.mount_commands();
        let mjoined = mount_cmds.join("\n");
        assert!(mjoined.contains("subvol=@"), "{mjoined}");
        // Root mount source is the mapper
        let root_mount = plan.mounts.iter().find(|m| m.target == "/mnt").unwrap();
        assert_eq!(root_mount.source, "/dev/mapper/cryptroot");
    }

    #[test]
    fn test_btrfs_fstab_checks_subvolume_mounts() {
        let mut state = crate::core::state::AppState::new(true);
        state.disks_selected_device = Some("/dev/sda".into());
        state.btrfs_subvolume_preset = 1; // Standard
        state.swap_enabled = false;

        let plan = StoragePlanner::compile(&state).expect("should compile");
        let fstab_cmds = plan.fstab_check_commands();
        let joined = fstab_cmds.join("\n");

        // Should check each subvolume mountpoint
        assert!(joined.contains("mountpoint -q /mnt"), "{joined}");
        assert!(joined.contains("mountpoint -q /mnt/home"), "{joined}");
        assert!(joined.contains("mountpoint -q /mnt/.snapshots"), "{joined}");
        assert!(joined.contains("genfstab"), "{joined}");
    }

    #[test]
    fn test_manual_mount_options_propagate() {
        let mut state = make_manual_state();
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("BOOT".into()),
                fs: Some("fat32".into()),
                start: Some("1048576".into()),
                size: Some("1073741824".into()),
                mountpoint: Some("/boot".into()),
                ..Default::default()
            });
        state
            .disks_partitions
            .push(crate::core::types::DiskPartitionSpec {
                name: Some("/dev/sda".into()),
                role: Some("ROOT".into()),
                fs: Some("btrfs".into()),
                start: Some("1074790400".into()),
                size: Some("100%".into()),
                mountpoint: Some("/".into()),
                mount_options: Some("compress=zstd,noatime".into()),
                ..Default::default()
            });

        let plan = StoragePlanner::compile(&state).expect("should compile");
        let root_mount = plan.mounts.iter().find(|m| m.target == "/mnt").unwrap();
        assert_eq!(root_mount.options, vec!["compress=zstd", "noatime"]);

        let mount_cmds = plan.mount_commands();
        let joined = mount_cmds.join("\n");
        assert!(joined.contains("compress=zstd,noatime"), "{joined}");
    }

    // ── Pre-mounted mode tests ──

    fn make_pre_mounted_plan(mounts: Vec<PlannedMount>) -> StoragePlan {
        StoragePlan {
            devices: vec![],
            mounts,
            mode: StorageMode::PreMounted,
            stacks: vec![],
        }
    }

    #[test]
    fn test_pre_mounted_no_partition_commands() {
        let plan = make_pre_mounted_plan(vec![PlannedMount {
            source: "/dev/sda2".into(),
            target: "/mnt".into(),
            fstype: "btrfs".into(),
            options: vec![],
            is_swap: false,
            subvolume: None,
        }]);
        let cmds = plan.partition_commands();
        assert!(
            cmds.is_empty(),
            "pre-mounted should produce no partition commands"
        );
    }

    #[test]
    fn test_pre_mounted_no_mount_commands() {
        let plan = make_pre_mounted_plan(vec![PlannedMount {
            source: "/dev/sda2".into(),
            target: "/mnt".into(),
            fstype: "btrfs".into(),
            options: vec![],
            is_swap: false,
            subvolume: None,
        }]);
        let cmds = plan.mount_commands();
        assert!(
            cmds.is_empty(),
            "pre-mounted should produce no mount commands"
        );
    }

    #[test]
    fn test_pre_mounted_fstab_mountpoint_checks() {
        let plan = make_pre_mounted_plan(vec![
            PlannedMount {
                source: "/dev/sda2".into(),
                target: "/mnt".into(),
                fstype: "btrfs".into(),
                options: vec![],
                is_swap: false,
                subvolume: None,
            },
            PlannedMount {
                source: "/dev/sda1".into(),
                target: "/mnt/boot".into(),
                fstype: "vfat".into(),
                options: vec![],
                is_swap: false,
                subvolume: None,
            },
        ]);
        let cmds = plan.fstab_check_commands();
        let joined = cmds.join("\n");

        // Should check mountpoints
        assert!(joined.contains("mountpoint -q /mnt"), "{joined}");
        assert!(joined.contains("mountpoint -q /mnt/boot"), "{joined}");
        // Should NOT contain blkid checks (pre-mounted skips format verification)
        assert!(!joined.contains("blkid"), "{joined}");
        // Should still generate fstab
        assert!(joined.contains("genfstab -U /mnt"), "{joined}");
    }

    #[test]
    fn test_pre_mounted_fstab_with_swap() {
        let plan = make_pre_mounted_plan(vec![
            PlannedMount {
                source: "/dev/sda2".into(),
                target: "/mnt".into(),
                fstype: "ext4".into(),
                options: vec![],
                is_swap: false,
                subvolume: None,
            },
            PlannedMount {
                source: "/dev/sda3".into(),
                target: String::new(),
                fstype: "swap".into(),
                options: vec![],
                is_swap: true,
                subvolume: None,
            },
        ]);
        let cmds = plan.fstab_check_commands();
        let joined = cmds.join("\n");

        assert!(joined.contains("mountpoint -q /mnt"), "{joined}");
        assert!(joined.contains("swap not active"), "{joined}");
        assert!(joined.contains("genfstab"), "{joined}");
    }

    #[test]
    fn test_pre_mounted_validation_missing_root() {
        let plan = make_pre_mounted_plan(vec![PlannedMount {
            source: "/dev/sda1".into(),
            target: "/mnt/boot".into(),
            fstype: "vfat".into(),
            options: vec![],
            is_swap: false,
            subvolume: None,
        }]);
        let errors = plan.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("/mnt")));
    }

    #[test]
    fn test_pre_mounted_validation_root_ok() {
        let plan = make_pre_mounted_plan(vec![PlannedMount {
            source: "/dev/sda2".into(),
            target: "/mnt".into(),
            fstype: "ext4".into(),
            options: vec![],
            is_swap: false,
            subvolume: None,
        }]);
        let errors = plan.validate();
        assert!(
            errors.is_empty(),
            "plan with /mnt root should validate: {:?}",
            errors
        );
    }

    #[test]
    fn test_pre_mounted_with_subvolumes() {
        let plan = make_pre_mounted_plan(vec![
            PlannedMount {
                source: "/dev/sda2".into(),
                target: "/mnt".into(),
                fstype: "btrfs".into(),
                options: vec!["subvol=@".into(), "compress=zstd".into()],
                is_swap: false,
                subvolume: Some("@".into()),
            },
            PlannedMount {
                source: "/dev/sda2".into(),
                target: "/mnt/home".into(),
                fstype: "btrfs".into(),
                options: vec!["subvol=@home".into(), "compress=zstd".into()],
                is_swap: false,
                subvolume: Some("@home".into()),
            },
            PlannedMount {
                source: "/dev/sda1".into(),
                target: "/mnt/boot".into(),
                fstype: "vfat".into(),
                options: vec![],
                is_swap: false,
                subvolume: None,
            },
        ]);
        let errors = plan.validate();
        assert!(
            errors.is_empty(),
            "subvolume plan should validate: {:?}",
            errors
        );

        let fstab_cmds = plan.fstab_check_commands();
        let joined = fstab_cmds.join("\n");
        assert!(joined.contains("mountpoint -q /mnt"), "{joined}");
        assert!(joined.contains("mountpoint -q /mnt/home"), "{joined}");
        assert!(joined.contains("mountpoint -q /mnt/boot"), "{joined}");
        assert!(joined.contains("genfstab"), "{joined}");
    }

    #[test]
    fn test_pre_mounted_accessors() {
        let plan = make_pre_mounted_plan(vec![
            PlannedMount {
                source: "/dev/sda2".into(),
                target: "/mnt".into(),
                fstype: "btrfs".into(),
                options: vec![],
                is_swap: false,
                subvolume: None,
            },
            PlannedMount {
                source: "/dev/sda1".into(),
                target: "/mnt/boot".into(),
                fstype: "vfat".into(),
                options: vec![],
                is_swap: false,
                subvolume: None,
            },
        ]);

        assert_eq!(plan.root_device_path(), Some("/dev/sda2".into()));
        assert_eq!(plan.esp_device_path(), Some("/dev/sda1".into()));
        assert!(!plan.has_encryption());
        assert_eq!(plan.mode, StorageMode::PreMounted);
    }

    // ── Device stack / LVM / RAID extensibility tests (Phase 6) ──

    #[test]
    fn test_device_stack_luks_final_path() {
        let stack = DeviceStack {
            base: "/dev/sda3".into(),
            layers: vec![VolumeLayer::Luks(EncryptionSpec {
                method: EncryptionMethod::Luks2,
                mapper_name: "cryptroot".into(),
                passphrase: None,
            })],
            filesystem: Some(FilesystemSpec {
                fstype: "btrfs".into(),
                mkfs_options: vec![],
            }),
        };
        assert_eq!(stack.final_device_path(), "/dev/mapper/cryptroot");
    }

    #[test]
    fn test_device_stack_lvm_final_path() {
        let stack = DeviceStack {
            base: "/dev/sda3".into(),
            layers: vec![VolumeLayer::Lvm(LvmSpec {
                vg_name: "vg0".into(),
                lv_name: "root".into(),
                size: "100%FREE".into(),
            })],
            filesystem: Some(FilesystemSpec {
                fstype: "ext4".into(),
                mkfs_options: vec![],
            }),
        };
        assert_eq!(stack.final_device_path(), "/dev/vg0/root");
    }

    #[test]
    fn test_device_stack_raid_final_path() {
        let stack = DeviceStack {
            base: "/dev/sda1".into(),
            layers: vec![VolumeLayer::Raid(RaidSpec {
                level: "1".into(),
                members: vec!["/dev/sda1".into(), "/dev/sdb1".into()],
                name: "md0".into(),
            })],
            filesystem: Some(FilesystemSpec {
                fstype: "ext4".into(),
                mkfs_options: vec![],
            }),
        };
        assert_eq!(stack.final_device_path(), "/dev/md/md0");
    }

    #[test]
    fn test_device_stack_luks_on_lvm() {
        // partition → LVM → LUKS → filesystem
        let stack = DeviceStack {
            base: "/dev/sda3".into(),
            layers: vec![
                VolumeLayer::Lvm(LvmSpec {
                    vg_name: "vg0".into(),
                    lv_name: "cryptlv".into(),
                    size: "100%FREE".into(),
                }),
                VolumeLayer::Luks(EncryptionSpec {
                    method: EncryptionMethod::Luks2,
                    mapper_name: "cryptroot".into(),
                    passphrase: None,
                }),
            ],
            filesystem: Some(FilesystemSpec {
                fstype: "btrfs".into(),
                mkfs_options: vec![],
            }),
        };
        assert_eq!(stack.final_device_path(), "/dev/mapper/cryptroot");
    }

    #[test]
    fn test_device_stack_lvm_on_luks() {
        // partition → LUKS → LVM → filesystem (the canonical LVM-on-LUKS layout)
        let stack = DeviceStack {
            base: "/dev/nvme0n1p3".into(),
            layers: vec![
                VolumeLayer::Luks(EncryptionSpec {
                    method: EncryptionMethod::Luks2,
                    mapper_name: "cryptlvm".into(),
                    passphrase: None,
                }),
                VolumeLayer::Lvm(LvmSpec {
                    vg_name: "vg_system".into(),
                    lv_name: "lv_root".into(),
                    size: "50GiB".into(),
                }),
            ],
            filesystem: Some(FilesystemSpec {
                fstype: "ext4".into(),
                mkfs_options: vec![],
            }),
        };
        assert_eq!(stack.final_device_path(), "/dev/vg_system/lv_root");
    }

    #[test]
    fn test_lvm_on_luks_full_plan() {
        // Verify the model can represent a complete LVM-on-LUKS install layout:
        //   /dev/sda1 → ESP (fat32) → /boot
        //   /dev/sda2 → LUKS → LVM VG → lv_root (ext4) → /
        //   /dev/sda2 → LUKS → LVM VG → lv_home (ext4) → /home
        //   /dev/sda2 → LUKS → LVM VG → lv_swap (swap)
        let root_stack = DeviceStack {
            base: "/dev/sda2".into(),
            layers: vec![
                VolumeLayer::Luks(EncryptionSpec {
                    method: EncryptionMethod::Luks2,
                    mapper_name: "cryptlvm".into(),
                    passphrase: None,
                }),
                VolumeLayer::Lvm(LvmSpec {
                    vg_name: "vg0".into(),
                    lv_name: "lv_root".into(),
                    size: "50GiB".into(),
                }),
            ],
            filesystem: Some(FilesystemSpec {
                fstype: "ext4".into(),
                mkfs_options: vec![],
            }),
        };
        let home_stack = DeviceStack {
            base: "/dev/sda2".into(),
            layers: vec![
                VolumeLayer::Luks(EncryptionSpec {
                    method: EncryptionMethod::Luks2,
                    mapper_name: "cryptlvm".into(),
                    passphrase: None,
                }),
                VolumeLayer::Lvm(LvmSpec {
                    vg_name: "vg0".into(),
                    lv_name: "lv_home".into(),
                    size: "100%FREE".into(),
                }),
            ],
            filesystem: Some(FilesystemSpec {
                fstype: "ext4".into(),
                mkfs_options: vec![],
            }),
        };
        let swap_stack = DeviceStack {
            base: "/dev/sda2".into(),
            layers: vec![
                VolumeLayer::Luks(EncryptionSpec {
                    method: EncryptionMethod::Luks2,
                    mapper_name: "cryptlvm".into(),
                    passphrase: None,
                }),
                VolumeLayer::Lvm(LvmSpec {
                    vg_name: "vg0".into(),
                    lv_name: "lv_swap".into(),
                    size: "8GiB".into(),
                }),
            ],
            filesystem: Some(FilesystemSpec {
                fstype: "linux-swap".into(),
                mkfs_options: vec![],
            }),
        };

        assert_eq!(root_stack.final_device_path(), "/dev/vg0/lv_root");
        assert_eq!(home_stack.final_device_path(), "/dev/vg0/lv_home");
        assert_eq!(swap_stack.final_device_path(), "/dev/vg0/lv_swap");

        // Build the full plan
        let plan = StoragePlan {
            devices: vec![PlannedDevice {
                path: "/dev/sda".into(),
                label: DiskLabel::Gpt,
                wipe: true,
                partitions: vec![
                    PlannedPartition {
                        number: 1,
                        role: PartitionRole::Esp,
                        start: "1MiB".into(),
                        end: "1025MiB".into(),
                        filesystem: FilesystemSpec {
                            fstype: "fat32".into(),
                            mkfs_options: vec![],
                        },
                        flags: vec![PartitionFlag::Esp],
                        encryption: None,
                        subvolumes: vec![],
                    },
                    PlannedPartition {
                        number: 2,
                        role: PartitionRole::Root,
                        start: "1025MiB".into(),
                        end: "100%".into(),
                        filesystem: FilesystemSpec {
                            fstype: "ext4".into(),
                            mkfs_options: vec![],
                        },
                        flags: vec![],
                        encryption: Some(EncryptionSpec {
                            method: EncryptionMethod::Luks2,
                            mapper_name: "cryptlvm".into(),
                            passphrase: None,
                        }),
                        subvolumes: vec![],
                    },
                ],
            }],
            mounts: vec![
                PlannedMount {
                    source: "/dev/vg0/lv_root".into(),
                    target: "/mnt".into(),
                    fstype: "ext4".into(),
                    options: vec![],
                    is_swap: false,
                    subvolume: None,
                },
                PlannedMount {
                    source: "/dev/sda1".into(),
                    target: "/mnt/boot".into(),
                    fstype: "vfat".into(),
                    options: vec![],
                    is_swap: false,
                    subvolume: None,
                },
                PlannedMount {
                    source: "/dev/vg0/lv_home".into(),
                    target: "/mnt/home".into(),
                    fstype: "ext4".into(),
                    options: vec![],
                    is_swap: false,
                    subvolume: None,
                },
                PlannedMount {
                    source: "/dev/vg0/lv_swap".into(),
                    target: String::new(),
                    fstype: "swap".into(),
                    options: vec![],
                    is_swap: true,
                    subvolume: None,
                },
            ],
            mode: StorageMode::Manual,
            stacks: vec![root_stack, home_stack, swap_stack],
        };

        // The plan should hold together without panics
        assert_eq!(plan.stacks.len(), 3);
        assert_eq!(plan.root_device_path(), Some("/dev/vg0/lv_root".into()));
        assert_eq!(plan.esp_device_path(), Some("/dev/sda1".into()));
        assert!(plan.has_encryption());

        // Stacks now generate real setup commands
        let root_cmds = plan.stacks[0].setup_commands();
        assert!(
            root_cmds
                .iter()
                .any(|c| c.contains("cryptsetup luksFormat"))
        );
        assert!(root_cmds.iter().any(|c| c.contains("pvcreate")));
        assert!(root_cmds.iter().any(|c| c.contains("vgcreate")));
        assert!(root_cmds.iter().any(|c| c.contains("lvcreate")));
        assert!(root_cmds.iter().any(|c| c.contains("mkfs.ext4")));

        // Existing partition_commands still work for the ESP + LUKS partition
        let cmds = plan.partition_commands();
        let joined = cmds.join("\n");
        assert!(joined.contains("mkfs.fat -F 32"), "{joined}");
        assert!(joined.contains("cryptsetup luksFormat"), "{joined}");
        assert!(joined.contains("cryptsetup open"), "{joined}");
    }

    #[test]
    fn test_raid1_plan_model() {
        // Verify the model can represent a RAID-1 mirror:
        //   /dev/sda1 + /dev/sdb1 → md0 (RAID1) → ext4 → /
        let raid_stack = DeviceStack {
            base: "/dev/sda1".into(),
            layers: vec![VolumeLayer::Raid(RaidSpec {
                level: "1".into(),
                members: vec!["/dev/sda1".into(), "/dev/sdb1".into()],
                name: "md0".into(),
            })],
            filesystem: Some(FilesystemSpec {
                fstype: "ext4".into(),
                mkfs_options: vec![],
            }),
        };

        assert_eq!(raid_stack.final_device_path(), "/dev/md/md0");

        let plan = StoragePlan {
            devices: vec![
                PlannedDevice {
                    path: "/dev/sda".into(),
                    label: DiskLabel::Gpt,
                    wipe: true,
                    partitions: vec![PlannedPartition {
                        number: 1,
                        role: PartitionRole::Root,
                        start: "1MiB".into(),
                        end: "100%".into(),
                        filesystem: FilesystemSpec {
                            fstype: "ext4".into(),
                            mkfs_options: vec![],
                        },
                        flags: vec![],
                        encryption: None,
                        subvolumes: vec![],
                    }],
                },
                PlannedDevice {
                    path: "/dev/sdb".into(),
                    label: DiskLabel::Gpt,
                    wipe: true,
                    partitions: vec![PlannedPartition {
                        number: 1,
                        role: PartitionRole::Root,
                        start: "1MiB".into(),
                        end: "100%".into(),
                        filesystem: FilesystemSpec {
                            fstype: "ext4".into(),
                            mkfs_options: vec![],
                        },
                        flags: vec![],
                        encryption: None,
                        subvolumes: vec![],
                    }],
                },
            ],
            mounts: vec![PlannedMount {
                source: "/dev/md/md0".into(),
                target: "/mnt".into(),
                fstype: "ext4".into(),
                options: vec![],
                is_swap: false,
                subvolume: None,
            }],
            mode: StorageMode::Manual,
            stacks: vec![raid_stack],
        };

        assert_eq!(plan.stacks.len(), 1);
        assert_eq!(plan.root_device_path(), Some("/dev/md/md0".into()));

        let raid_cmds = plan.stacks[0].setup_commands();
        assert!(raid_cmds.iter().any(|c| c.contains("mdadm --create")));
        assert!(raid_cmds.iter().any(|c| c.contains("--level=1")));
        assert!(raid_cmds.iter().any(|c| c.contains("--raid-devices=2")));
        assert!(raid_cmds.iter().any(|c| c.contains("mkfs.ext4")));

        if let VolumeLayer::Raid(ref raid) = plan.stacks[0].layers[0] {
            assert_eq!(raid.level, "1");
            assert_eq!(raid.members.len(), 2);
        } else {
            panic!("expected Raid layer");
        }
    }

    #[test]
    fn test_device_stack_no_layers() {
        let stack = DeviceStack {
            base: "/dev/sda1".into(),
            layers: vec![],
            filesystem: Some(FilesystemSpec {
                fstype: "ext4".into(),
                mkfs_options: vec![],
            }),
        };
        assert_eq!(stack.final_device_path(), "/dev/sda1");
    }

    #[test]
    fn test_existing_plans_have_empty_stacks() {
        let plan = compile_auto_uefi(true, false);
        assert!(plan.stacks.is_empty());

        let plan2 = compile_auto_bios(true, true);
        assert!(plan2.stacks.is_empty());

        let pre = make_pre_mounted_plan(vec![PlannedMount {
            source: "/dev/sda2".into(),
            target: "/mnt".into(),
            fstype: "ext4".into(),
            options: vec![],
            is_swap: false,
            subvolume: None,
        }]);
        assert!(pre.stacks.is_empty());
    }

    // ── setup_commands() tests ──

    #[test]
    fn test_setup_commands_luks_only() {
        let stack = DeviceStack {
            base: "/dev/sda3".into(),
            layers: vec![VolumeLayer::Luks(EncryptionSpec {
                method: EncryptionMethod::Luks2,
                mapper_name: "cryptroot".into(),
                passphrase: None,
            })],
            filesystem: Some(FilesystemSpec {
                fstype: "btrfs".into(),
                mkfs_options: vec![],
            }),
        };
        let cmds = stack.setup_commands();
        assert_eq!(
            cmds[0],
            "modprobe -q dm_crypt 2>/dev/null || modprobe -q dm-crypt 2>/dev/null || true"
        );
        assert_eq!(cmds[1], "cryptsetup luksFormat --type luks2 -q /dev/sda3");
        assert_eq!(cmds[2], "udevadm settle");
        assert_eq!(cmds[3], "cryptsetup open --type luks /dev/sda3 cryptroot");
        assert_eq!(cmds[4], "mkfs.btrfs -f /dev/mapper/cryptroot");
        assert_eq!(cmds.len(), 5);
    }

    #[test]
    fn test_setup_commands_lvm_only() {
        let stack = DeviceStack {
            base: "/dev/sda2".into(),
            layers: vec![VolumeLayer::Lvm(LvmSpec {
                vg_name: "vg0".into(),
                lv_name: "lv_root".into(),
                size: "50GiB".into(),
            })],
            filesystem: Some(FilesystemSpec {
                fstype: "ext4".into(),
                mkfs_options: vec![],
            }),
        };
        let cmds = stack.setup_commands();
        assert_eq!(cmds[0], "pvcreate /dev/sda2");
        assert_eq!(cmds[1], "vgcreate vg0 /dev/sda2");
        assert_eq!(cmds[2], "lvcreate -L 50GiB vg0 -n lv_root");
        assert_eq!(cmds[3], "mkfs.ext4 -F /dev/vg0/lv_root");
        assert_eq!(cmds.len(), 4);
    }

    #[test]
    fn test_setup_commands_lvm_percent_size() {
        let stack = DeviceStack {
            base: "/dev/sda2".into(),
            layers: vec![VolumeLayer::Lvm(LvmSpec {
                vg_name: "vg0".into(),
                lv_name: "lv_root".into(),
                size: "100%FREE".into(),
            })],
            filesystem: None,
        };
        let cmds = stack.setup_commands();
        assert!(cmds[2].contains("-l 100%FREE"), "{}", cmds[2]);
    }

    #[test]
    fn test_setup_commands_raid() {
        let stack = DeviceStack {
            base: "/dev/sda1".into(),
            layers: vec![VolumeLayer::Raid(RaidSpec {
                level: "1".into(),
                members: vec!["/dev/sda1".into(), "/dev/sdb1".into()],
                name: "md0".into(),
            })],
            filesystem: Some(FilesystemSpec {
                fstype: "ext4".into(),
                mkfs_options: vec![],
            }),
        };
        let cmds = stack.setup_commands();
        assert_eq!(
            cmds[0],
            "mdadm --create /dev/md/md0 --level=1 --raid-devices=2 /dev/sda1 /dev/sdb1"
        );
        assert_eq!(cmds[1], "mkfs.ext4 -F /dev/md/md0");
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn test_setup_commands_lvm_on_luks() {
        let stack = DeviceStack {
            base: "/dev/nvme0n1p3".into(),
            layers: vec![
                VolumeLayer::Luks(EncryptionSpec {
                    method: EncryptionMethod::Luks2,
                    mapper_name: "cryptlvm".into(),
                    passphrase: None,
                }),
                VolumeLayer::Lvm(LvmSpec {
                    vg_name: "vg_system".into(),
                    lv_name: "lv_root".into(),
                    size: "50GiB".into(),
                }),
            ],
            filesystem: Some(FilesystemSpec {
                fstype: "ext4".into(),
                mkfs_options: vec![],
            }),
        };
        let cmds = stack.setup_commands();
        assert_eq!(
            cmds[0],
            "modprobe -q dm_crypt 2>/dev/null || modprobe -q dm-crypt 2>/dev/null || true"
        );
        assert_eq!(
            cmds[1],
            "cryptsetup luksFormat --type luks2 -q /dev/nvme0n1p3"
        );
        assert_eq!(cmds[2], "udevadm settle");
        assert_eq!(
            cmds[3],
            "cryptsetup open --type luks /dev/nvme0n1p3 cryptlvm"
        );
        assert_eq!(cmds[4], "pvcreate /dev/mapper/cryptlvm");
        assert_eq!(cmds[5], "vgcreate vg_system /dev/mapper/cryptlvm");
        assert_eq!(cmds[6], "lvcreate -L 50GiB vg_system -n lv_root");
        assert_eq!(cmds[7], "mkfs.ext4 -F /dev/vg_system/lv_root");
        assert_eq!(cmds.len(), 8);
    }

    #[test]
    fn test_setup_commands_no_filesystem() {
        let stack = DeviceStack {
            base: "/dev/sda2".into(),
            layers: vec![VolumeLayer::Luks(EncryptionSpec {
                method: EncryptionMethod::Luks2,
                mapper_name: "crypt".into(),
                passphrase: None,
            })],
            filesystem: None,
        };
        let cmds = stack.setup_commands();
        assert_eq!(cmds.len(), 4);
        assert!(!cmds.iter().any(|c| c.contains("mkfs")));
    }

    #[test]
    fn test_setup_commands_no_layers_with_fs() {
        let stack = DeviceStack {
            base: "/dev/sda1".into(),
            layers: vec![],
            filesystem: Some(FilesystemSpec {
                fstype: "xfs".into(),
                mkfs_options: vec![],
            }),
        };
        let cmds = stack.setup_commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], "mkfs.xfs -f /dev/sda1");
    }

    #[test]
    fn test_stack_setup_commands_aggregates() {
        let plan = StoragePlan {
            devices: vec![],
            mounts: vec![PlannedMount {
                source: "/dev/vg0/lv_root".into(),
                target: "/mnt".into(),
                fstype: "ext4".into(),
                options: vec![],
                is_swap: false,
                subvolume: None,
            }],
            mode: StorageMode::Manual,
            stacks: vec![
                DeviceStack {
                    base: "/dev/sda2".into(),
                    layers: vec![VolumeLayer::Luks(EncryptionSpec {
                        method: EncryptionMethod::Luks2,
                        mapper_name: "crypt".into(),
                        passphrase: None,
                    })],
                    filesystem: None,
                },
                DeviceStack {
                    base: "/dev/mapper/crypt".into(),
                    layers: vec![VolumeLayer::Lvm(LvmSpec {
                        vg_name: "vg0".into(),
                        lv_name: "lv_root".into(),
                        size: "100%FREE".into(),
                    })],
                    filesystem: Some(FilesystemSpec {
                        fstype: "ext4".into(),
                        mkfs_options: vec![],
                    }),
                },
            ],
        };
        let cmds = plan.stack_setup_commands();
        assert!(cmds.iter().any(|c| c.contains("cryptsetup luksFormat")));
        assert!(cmds.iter().any(|c| c.contains("pvcreate")));
        assert!(cmds.iter().any(|c| c.contains("lvcreate")));
        assert!(cmds.iter().any(|c| c.contains("mkfs.ext4")));
    }

    #[test]
    fn test_has_encryption_via_stacks() {
        let plan = StoragePlan {
            devices: vec![],
            mounts: vec![PlannedMount {
                source: "/dev/mapper/crypt".into(),
                target: "/mnt".into(),
                fstype: "ext4".into(),
                options: vec![],
                is_swap: false,
                subvolume: None,
            }],
            mode: StorageMode::Manual,
            stacks: vec![DeviceStack {
                base: "/dev/sda2".into(),
                layers: vec![VolumeLayer::Luks(EncryptionSpec {
                    method: EncryptionMethod::Luks2,
                    mapper_name: "crypt".into(),
                    passphrase: None,
                })],
                filesystem: Some(FilesystemSpec {
                    fstype: "ext4".into(),
                    mkfs_options: vec![],
                }),
            }],
        };
        assert!(plan.has_encryption());
    }

    #[test]
    fn test_setup_commands_swap_via_lvm() {
        let stack = DeviceStack {
            base: "/dev/sda2".into(),
            layers: vec![VolumeLayer::Lvm(LvmSpec {
                vg_name: "vg0".into(),
                lv_name: "lv_swap".into(),
                size: "8GiB".into(),
            })],
            filesystem: Some(FilesystemSpec {
                fstype: "linux-swap".into(),
                mkfs_options: vec![],
            }),
        };
        let cmds = stack.setup_commands();
        assert!(cmds.last().unwrap().contains("mkswap"));
    }
}
