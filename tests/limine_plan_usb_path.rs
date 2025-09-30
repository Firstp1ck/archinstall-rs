// Validate that the plan contains the USB (EFI/BOOT) path handling
// and writes configuration under /mnt/boot/EFI/BOOT when USB is detected at runtime.

#[test]
fn limine_plan_includes_usb_efi_boot_path_handling() {
    // Force UEFI path
    archinstall_rs::app::install::set_uefi_override_for_tests(Some(true));

    // Minimal AppState with Limine selected
    let mut state = archinstall_rs::core::state::AppState::new(true);
    state.bootloader_index = 3; // Limine
    state.selected_kernels.clear();
    state.selected_kernels.insert("linux".into());

    let plan = archinstall_rs::core::services::bootloader::BootloaderService::build_plan(&state, "/dev/fake");
    let blob = plan.commands.join("\n");

    // Ensure we explicitly create EFI/BOOT directory
    assert!(blob.contains("Creating directory: /mnt/boot/EFI/BOOT"),
        "missing creation of /mnt/boot/EFI/BOOT; blob:\n{}", blob);
    assert!(blob.contains("install -d -m 0755 \"/mnt/boot/EFI/BOOT\""),
        "missing install -d for /mnt/boot/EFI/BOOT; blob:\n{}", blob);

    // Pacman hook should copy loader binaries to EFI/BOOT as well
    assert!(blob.contains("/usr/share/limine/BOOTX64.EFI /boot/EFI/BOOT/"),
        "hook missing copy to /boot/EFI/BOOT; blob:\n{}", blob);

    // Config generation stage: ensure CONFIG_DIR may switch to EFI/BOOT (presence of condition)
    assert!(blob.contains("if [ -d /mnt/boot/EFI/BOOT ]; then CONFIG_DIR=/mnt/boot/EFI/BOOT; else CONFIG_DIR=/mnt/boot/EFI/limine; fi;"),
        "CONFIG_DIR USB condition missing; blob:\n{}", blob);

    // Ensure config heredoc writes using $CONFIG_DIR/limine.conf
    assert!(blob.contains("cat > \"$CONFIG_DIR/limine.conf\""),
        "limine.conf heredoc not present; blob:\n{}", blob);
}
