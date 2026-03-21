use archinstall_rs as ai;

fn make_state() -> ai::app::AppState {
    ai::app::AppState::new(true)
}

#[test]
fn utils_redact_chpasswd_and_cryptsetup() {
    // chpasswd preserves username, redacts password
    let cmd = "echo \"alice:s3cr3t\" | chpasswd";
    let red = ai::common::utils::redact_command_for_logging(cmd);
    assert!(red.contains("alice:<REDACTED>"), "{red}");
    assert!(!red.contains("s3cr3t"), "{red}");

    // single quotes variant
    let cmd2 = "echo 'bob:pw' | chpasswd";
    let red2 = ai::common::utils::redact_command_for_logging(cmd2);
    assert!(red2.contains("bob:<REDACTED>"), "{red2}");
    assert!(!red2.contains("pw"), "{red2}");

    // cryptsetup pipeline redacts whole echo payload
    let cmd3 = "echo \"topsecret\" | cryptsetup luksFormat /dev/sda3";
    let red3 = ai::common::utils::redact_command_for_logging(cmd3);
    assert!(red3.contains("echo \"<REDACTED>\" | cryptsetup"), "{red3}");
    assert!(!red3.contains("topsecret"), "{red3}");

    // printf '%s' pipeline (legacy shell strings)
    let cmd4 =
        "printf '%s' 'hunter2' | cryptsetup open --type luks --key-file=- /dev/sda3 cryptroot";
    let red4 = ai::common::utils::redact_command_for_logging(cmd4);
    assert!(red4.contains("printf '%s' '<REDACTED>'"), "{red4}");
    assert!(!red4.contains("hunter2"), "{red4}");
}

#[test]
fn utils_sanitize_terminal_output_line() {
    let raw = "Downloading...\rDownloading...\x1b[31mERR\x1b[0m\tDone\x07";
    let clean = ai::common::utils::sanitize_terminal_output_line(raw);
    assert!(clean.contains("Downloading...\nDownloading..."), "{clean}");
    assert!(!clean.contains("\x1b"), "{clean}");
    assert!(!clean.contains("\x07"), "{clean}");
    assert!(!clean.contains("\t"), "{clean}");
}

#[test]
fn network_manual_plan_contains_expected_config() {
    let mut state = make_state();
    state.network_mode_index = 1; // Manual
    state
        .network_configs
        .push(ai::core::types::NetworkInterfaceConfig {
            interface: "eth0".into(),
            mode: ai::core::types::NetworkConfigMode::Static,
            ip_cidr: Some("192.0.2.10/24".into()),
            gateway: Some("192.0.2.1".into()),
            dns: Some("1.1.1.1,8.8.8.8".into()),
        });

    let plan = ai::core::services::network::NetworkService::build_plan(&state);
    let joined = plan.commands.join("\n");

    // Should enable services and create config file lines
    assert!(
        joined.contains("systemctl --root=/mnt enable systemd-networkd"),
        "{joined}"
    );
    assert!(
        joined.contains("systemctl --root=/mnt enable systemd-resolved"),
        "{joined}"
    );
    assert!(joined.contains("/mnt/etc/systemd/network"), "{joined}");
    assert!(joined.contains("Name=eth0"), "{joined}");
    assert!(joined.contains("Address=192.0.2.10/24"), "{joined}");
    assert!(joined.contains("Gateway=192.0.2.1"), "{joined}");
    assert!(joined.contains("DNS=1.1.1.1"), "{joined}");
    assert!(joined.contains("DNS=8.8.8.8"), "{joined}");
}

#[test]
fn usersetup_redacts_passwords_in_dry_run() {
    let mut state = make_state();
    state.users.push(ai::core::types::UserAccount {
        username: "eve".into(),
        password: "verysecret".into(),
        password_hash: None,
        is_sudo: true,
    });

    let plan = ai::core::services::usersetup::UserSetupService::build_plan(&state);
    let joined = plan.commands.join("\n");
    assert!(
        joined.contains("useradd -m -s /bin/bash -G wheel eve"),
        "{joined}"
    );
    assert!(
        joined.contains("echo \"eve:<REDACTED>\" | chpasswd"),
        "{joined}"
    );
    assert!(!joined.contains("verysecret"), "{joined}");
}

#[test]
fn sysconfig_enables_networkmanager_when_selected() {
    let mut state = make_state();
    state.disks_selected_device = Some("/dev/sda".into());
    state.network_mode_index = 2; // NetworkManager
    let storage_plan = ai::core::storage::planner::StoragePlanner::compile(&state)
        .expect("auto plan should compile");
    let plan = ai::core::services::sysconfig::SysConfigService::build_plan(&state, &storage_plan);
    let joined = plan.commands.join("\n");
    assert!(
        joined.contains("systemctl --root=/mnt enable NetworkManager"),
        "{joined}"
    );
}

#[test]
fn partitioning_auto_includes_root_and_format() {
    let state = make_state();
    let device = "/dev/sda";
    let plan = ai::core::services::partitioning::PartitioningService::build_plan(&state, device);
    let joined = plan.commands.join("\n");
    assert!(
        joined.contains(&format!("parted -s {device} mklabel")),
        "{joined}"
    );
    assert!(
        joined.contains(&format!("parted -s {device} mkpart root btrfs")),
        "{joined}"
    );
    // No encryption by default -> mkfs.btrfs on {device}3
    assert!(
        joined.contains(&format!("mkfs.btrfs -f {device}3")),
        "{joined}"
    );
}

#[test]
fn partitioning_manual_converts_bytes_to_mib() {
    let mut state = make_state();
    state.disks_mode_index = 1; // manual
    state
        .disks_partitions
        .push(ai::core::types::DiskPartitionSpec {
            name: Some("/dev/sda".into()),
            role: Some("OTHER".into()),
            fs: Some("ext4".into()),
            start: Some("1048576".into()), // 1 MiB
            size: Some("2097152".into()),  // 2 MiB
            flags: vec![],
            mountpoint: None,
            mount_options: None,
            encrypt: None,
        });
    let device = "/dev/sda";
    let plan = ai::core::services::partitioning::PartitioningService::build_plan(&state, device);
    let joined = plan.commands.join("\n");
    assert!(joined.contains("1MiB"), "{joined}");
    // Size is a byte length: 1 MiB start + 2 MiB size → end at 3 MiB (not 2 MiB as absolute end).
    assert!(joined.contains("3MiB"), "{joined}");
}

#[test]
fn mounting_mounts_root_partition_when_not_encrypted() {
    let mut state = make_state();
    state.disk_encryption_type_index = 0; // None
    let device = "/dev/nvme0n1";
    let plan = ai::core::services::mounting::MountingService::build_plan(&state, device);
    let joined = plan.commands.join("\n");
    // nvme gets p-suffix on partition numbers
    assert!(joined.contains("mkdir -p /mnt"), "{joined}");
    assert!(joined.contains("mount /dev/nvme0n1p3 /mnt"), "{joined}");
}

#[test]
fn fstab_generates_and_checks_mountpoints() {
    let state = make_state();
    let device = "/dev/sda";
    let plan = ai::core::services::fstab::FstabService::build_checks_and_fstab(&state, device);
    let joined = plan.commands.join("\n");
    assert!(joined.contains("mountpoint -q /mnt"), "{joined}");
    assert!(
        joined.contains("genfstab -U /mnt >> /mnt/etc/fstab"),
        "{joined}"
    );
}

#[test]
fn system_pacstrap_plan_includes_pacstrap_when_not_dry_run() {
    let state = ai::app::AppState::new(false); // non-dry-run branch
    // Ensure at least one kernel is present (AppState::new adds 'linux' by default)
    let plan = ai::core::services::system::SystemService::build_pacstrap_plan(&state);
    let joined = plan.commands.join("\n");
    assert!(joined.contains("pacman -Syy"), "{joined}");
    assert!(joined.contains(" pacstrap -K /mnt "), "{joined}");
}

#[test]
fn bootloader_systemd_boot_writes_loader_and_entries() {
    let mut state = make_state(); // default bootloader_index = 0 (systemd-boot)
    state.disks_selected_device = Some("/dev/sda".into());
    let device = "/dev/sda";
    let storage_plan = ai::core::storage::planner::StoragePlanner::compile(&state)
        .expect("auto plan should compile");
    let plan = ai::core::services::bootloader::BootloaderService::build_plan(
        &state,
        device,
        &storage_plan,
    );
    let joined = plan.commands.join("\n");
    assert!(joined.contains("bootctl"), "{joined}");
    assert!(joined.contains("loader.conf"), "{joined}");
    assert!(joined.contains("arch.conf"), "{joined}");
    assert!(joined.contains("arch-fallback.conf"), "{joined}");
    // Non-encrypted: should use simple root=UUID= options
    assert!(!joined.contains("rd.luks.name"), "{joined}");
}

#[test]
fn bootloader_systemd_boot_luks_adds_rd_luks_name() {
    let mut state = make_state();
    state.disks_selected_device = Some("/dev/sda".into());
    state.disk_encryption_type_index = 1; // LUKS
    let device = "/dev/sda";
    let storage_plan = ai::core::storage::planner::StoragePlanner::compile(&state)
        .expect("luks plan should compile");
    let plan = ai::core::services::bootloader::BootloaderService::build_plan(
        &state,
        device,
        &storage_plan,
    );
    let joined = plan.commands.join("\n");
    assert!(joined.contains("bootctl"), "{joined}");
    assert!(joined.contains("rd.luks.name="), "{joined}");
    assert!(joined.contains("arch.conf"), "{joined}");
}

#[test]
fn bootloader_grub_luks_injects_cmdline() {
    let mut state = make_state();
    state.disks_selected_device = Some("/dev/sda".into());
    state.disk_encryption_type_index = 1; // LUKS
    state.bootloader_index = 1; // GRUB
    let device = "/dev/sda";
    let storage_plan = ai::core::storage::planner::StoragePlanner::compile(&state)
        .expect("luks plan should compile");
    let plan = ai::core::services::bootloader::BootloaderService::build_plan(
        &state,
        device,
        &storage_plan,
    );
    let joined = plan.commands.join("\n");
    assert!(joined.contains("grub-install"), "{joined}");
    assert!(joined.contains("GRUB_CMDLINE_LINUX"), "{joined}");
    assert!(joined.contains("grub-mkconfig"), "{joined}");
}
