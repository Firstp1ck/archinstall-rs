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

    // UEFI stage: ensure USB branch sets EFI_DIR/EFI_DIR_TARGET to EFI/BOOT
    assert!(blob.contains("EFI_DIR=/mnt/boot/EFI/BOOT"),
        "USB EFI_DIR assignment missing; blob:\n{}", blob);
    assert!(blob.contains("EFI_DIR_TARGET=/boot/EFI/BOOT"),
        "USB EFI_DIR_TARGET assignment missing; blob:\n{}", blob);

    // Config generation stage: ensure CONFIG_DIR may switch to EFI/BOOT
    assert!(blob.contains("CONFIG_DIR=/mnt/boot/EFI/BOOT"),
        "CONFIG_DIR USB path missing; blob:\n{}", blob);

    // Ensure config heredoc writes using $CONFIG_DIR/limine.conf
    assert!(blob.contains("cat > \"$CONFIG_DIR/limine.conf\""),
        "limine.conf heredoc not present; blob:\n{}", blob);
}
