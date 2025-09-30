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

    // The script must define defaults for EFI_DIR/EFI_DIR_TARGET before any mkdir
    assert!(blob.contains("EFI_DIR=/mnt/boot/EFI/limine; EFI_DIR_TARGET=/boot/EFI/limine;"),
        "EFI_DIR defaults missing; script blob:\n{}", blob);

    // It must attempt to create the directory using the variable (not empty path)
    assert!(blob.contains("mkdir -p \"$EFI_DIR\""),
        "mkdir does not target $EFI_DIR; script blob:\n{}", blob);

    // The plan should include the message for skipping efibootmgr in case of unknown parent
    assert!(blob.contains("Skipping efibootmgr (USB install or unknown parent device)"),
        "skip message missing; script blob:\n{}", blob);

    // Config file must be written under CONFIG_DIR which resolves to EFI/limine by default
    assert!(blob.contains("cat > \"$CONFIG_DIR/limine.conf\""),
        "limine.conf heredoc not present; script blob:\n{}", blob);
}
