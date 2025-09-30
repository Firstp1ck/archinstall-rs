// No filesystem interaction needed; we just inspect the plan text

#[test]
fn limine_plan_targets_efi_limine_directory() {
    // Force UEFI mode in code paths that check it
    archinstall_rs::app::install::set_uefi_override_for_tests(Some(true));

    // Prepare a minimal pseudo-state mimicking selection of Limine and kernel 'linux'
    let mut state = archinstall_rs::core::state::AppState::new(true);
    state.bootloader_index = 3; // Limine
    state.selected_kernels.clear();
    state.selected_kernels.insert("linux".into());

    // Build plan with a fake device
    let plan = archinstall_rs::core::services::bootloader::BootloaderService::build_plan(&state, "/dev/fake");

    // Join commands into one blob for inspection
    let blob = plan.commands.join("\n");

    // It must create or reference the limine config in EFI/limine (regular non-USB flow)
    assert!(blob.contains("/mnt/boot/EFI/limine"), "plan does not reference /mnt/boot/EFI/limine:\n{}", blob);

    // Must write limine.conf (or limine.cfg) via heredoc or cp into that directory
    assert!(blob.contains("limine.conf"), "plan does not write limine.conf: \n{}", blob);

    // It must not write to root-level /limine.conf due to empty CONFIG_DIR
    assert!(!blob.contains("Created limine.conf at /limine.conf"), "plan would create config at root /limine.conf (CONFIG_DIR empty)\n{}", blob);
}
