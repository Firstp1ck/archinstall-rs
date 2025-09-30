// Ensures the UEFI plan embeds safe defaults when PARENT_DEV is empty
// and still writes config to /mnt/boot/EFI/limine.

#[test]
fn limine_plan_has_safe_defaults_and_writes_config() {
    // Force UEFI path
    archinstall_rs::app::install::set_uefi_override_for_tests(Some(true));

    // Minimal state
    let mut state = archinstall_rs::core::state::AppState::new(true);
    state.bootloader_index = 3; // Limine
    state.selected_kernels.clear();
    state.selected_kernels.insert("linux".into());

    let plan = archinstall_rs::core::services::bootloader::BootloaderService::build_plan(&state, "/dev/fake");
    let blob = plan.commands.join("\n");

    // The script must explicitly create both target directories
    assert!(blob.contains("Creating directory: /mnt/boot/EFI/limine"),
        "missing creation of /mnt/boot/EFI/limine; blob:\n{}", blob);
    assert!(blob.contains("install -d -m 0755 \"/mnt/boot/EFI/limine\""),
        "missing install -d for /mnt/boot/EFI/limine; blob:\n{}", blob);
    assert!(blob.contains("Creating directory: /mnt/boot/EFI/BOOT"),
        "missing creation of /mnt/boot/EFI/BOOT; blob:\n{}", blob);
    assert!(blob.contains("install -d -m 0755 \"/mnt/boot/EFI/BOOT\""),
        "missing install -d for /mnt/boot/EFI/BOOT; blob:\n{}", blob);

    // The plan should include the message for skipping efibootmgr in case of unknown parent
    assert!(blob.contains("Skipping efibootmgr (USB install or unknown parent device)"),
        "skip message missing; script blob:\n{}", blob);

    // Pacman hook should copy loader binaries to both locations
    assert!(blob.contains("/usr/share/limine/BOOTX64.EFI /boot/EFI/limine/"),
        "hook missing copy to /boot/EFI/limine; blob:\n{}", blob);
    assert!(blob.contains("/usr/share/limine/BOOTX64.EFI /boot/EFI/BOOT/"),
        "hook missing copy to /boot/EFI/BOOT; blob:\n{}", blob);

    // Config file must be written under CONFIG_DIR (and optional mirror to EFI/BOOT is fine)
    assert!(blob.contains("cat > \"$CONFIG_DIR/limine.conf\""),
        "limine.conf heredoc not present; script blob:\n{}", blob);
}
