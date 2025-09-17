use crate::ui::app::{AppState, Focus, Screen};

pub(crate) fn move_menu_up(app: &mut AppState) {
    if app.selected_index > 0 {
        app.selected_index -= 1;
    } else if !app.menu_entries.is_empty() {
        app.selected_index = app.menu_entries.len() - 1;
    }
    app.list_state.select(Some(app.selected_index));
}

pub(crate) fn move_menu_down(app: &mut AppState) {
    if app.selected_index + 1 < app.menu_entries.len() {
        app.selected_index += 1;
    } else if !app.menu_entries.is_empty() {
        app.selected_index = 0;
    }
    app.list_state.select(Some(app.selected_index));
}

pub(crate) fn move_locales_focus(app: &mut AppState, forward: bool) {
    if app.current_screen() != Screen::Locales {
        return;
    }
    let max = 4; // 3 fields + Continue
    if forward {
        app.locales_focus_index = (app.locales_focus_index + 1) % max;
    } else {
        app.locales_focus_index = (app.locales_focus_index + max - 1) % max;
    }
}

pub(crate) fn change_locales_value(app: &mut AppState, next: bool) {
    if app.current_screen() != Screen::Locales || app.focus != Focus::Content {
        return;
    }
    match app.locales_focus_index {
        0 => cycle_index(
            &mut app.draft_keyboard_layout_index,
            app.keyboard_layout_options.len(),
            next,
        ),
        1 => cycle_index(
            &mut app.draft_locale_language_index,
            app.locale_language_options.len(),
            next,
        ),
        2 => cycle_index(
            &mut app.draft_locale_encoding_index,
            app.locale_encoding_options.len(),
            next,
        ),
        _ => {}
    }
}

pub(crate) fn cycle_index(index: &mut usize, len: usize, next: bool) {
    if len == 0 {
        return;
    }
    if next {
        *index = (*index + 1) % len;
    } else {
        *index = (*index + len - 1) % len;
    }
}

pub(crate) fn move_locales_up(app: &mut AppState) {
    if app.current_screen() != Screen::Locales || app.focus != Focus::Content {
        return;
    }
    if app.locales_focus_index == 0 {
        app.locales_focus_index = 3;
    } else {
        app.locales_focus_index -= 1;
    }
}

pub(crate) fn move_locales_down(app: &mut AppState) {
    if app.current_screen() != Screen::Locales || app.focus != Focus::Content {
        return;
    }
    app.locales_focus_index = (app.locales_focus_index + 1) % 4;
}

pub(crate) fn move_mirrors_repos_up(app: &mut AppState) {
    if app.current_screen() != Screen::MirrorsRepos || app.focus != Focus::Content {
        return;
    }
    if app.mirrors_focus_index == 0 {
        app.mirrors_focus_index = 4;
    } else {
        app.mirrors_focus_index -= 1;
    }
}

pub(crate) fn move_mirrors_repos_down(app: &mut AppState) {
    if app.current_screen() != Screen::MirrorsRepos || app.focus != Focus::Content {
        return;
    }
    app.mirrors_focus_index = (app.mirrors_focus_index + 1) % 5;
}

pub(crate) fn move_screen_up(app: &mut AppState) {
    match app.current_screen() {
        Screen::Locales => move_locales_up(app),
        Screen::MirrorsRepos => move_mirrors_repos_up(app),
        Screen::Disks => move_disks_up(app),
        Screen::DiskEncryption => move_diskenc_up(app),
        Screen::Bootloader => move_bootloader_up(app),
        Screen::Audio => move_audio_up(app),
        Screen::Kernels => move_kernels_up(app),
        Screen::NetworkConfiguration => move_network_up(app),
        Screen::Hostname => move_hostname_up(app),
        Screen::RootPassword => move_rootpass_up(app),
        Screen::UserAccount => move_user_up(app),
        Screen::SwapPartition => move_swap_up(app),
        Screen::UnifiedKernelImages => move_uki_up(app),
        Screen::AutomaticTimeSync => move_ats_up(app),
        Screen::ExperienceMode => move_experience_up(app),
        Screen::SaveConfiguration => move_config_up(app),
        Screen::AdditionalPackages => move_addpkgs_up(app),
        Screen::Timezone => move_timezone_up(app),
        _ => {}
    }
}

pub(crate) fn move_screen_down(app: &mut AppState) {
    match app.current_screen() {
        Screen::Locales => move_locales_down(app),
        Screen::MirrorsRepos => move_mirrors_repos_down(app),
        Screen::Disks => move_disks_down(app),
        Screen::DiskEncryption => move_diskenc_down(app),
        Screen::Bootloader => move_bootloader_down(app),
        Screen::Audio => move_audio_down(app),
        Screen::Kernels => move_kernels_down(app),
        Screen::NetworkConfiguration => move_network_down(app),
        Screen::Hostname => move_hostname_down(app),
        Screen::RootPassword => move_rootpass_down(app),
        Screen::UserAccount => move_user_down(app),
        Screen::SwapPartition => move_swap_down(app),
        Screen::UnifiedKernelImages => move_uki_down(app),
        Screen::AutomaticTimeSync => move_ats_down(app),
        Screen::ExperienceMode => move_experience_down(app),
        Screen::SaveConfiguration => move_config_down(app),
        Screen::AdditionalPackages => move_addpkgs_down(app),
        Screen::Timezone => move_timezone_down(app),
        _ => {}
    }
}

pub(crate) fn change_value(app: &mut AppState, next: bool) {
    match app.current_screen() {
        Screen::Locales => change_locales_value(app, next),
        Screen::Disks => change_disks_value(app, next),
        Screen::DiskEncryption => change_diskenc_value(app, next),
        Screen::Bootloader => change_bootloader_value(app, next),
        Screen::Audio => change_audio_value(app, next),
        Screen::Kernels => change_kernels_value(app, next),
        Screen::NetworkConfiguration => change_network_value(app, next),
        Screen::Hostname => change_hostname_value(app, next),
        Screen::RootPassword => change_rootpass_value(app, next),
        Screen::UserAccount => change_user_value(app, next),
        Screen::SwapPartition => change_swap_value(app, next),
        Screen::UnifiedKernelImages => change_uki_value(app, next),
        Screen::AutomaticTimeSync => change_ats_value(app, next),
        Screen::ExperienceMode => change_experience_value(app, next),
        Screen::SaveConfiguration => change_config_value(app, next),
        Screen::AdditionalPackages => change_addpkgs_value(app, next),
        Screen::Timezone => change_timezone_value(app, next),
        _ => {}
    }
}

pub(crate) fn handle_enter(app: &mut AppState) {
    if app.focus == Focus::Menu {
        app.info_message = format!("Opened '{}'", app.menu_entries[app.selected_index].label);
        app.focus = Focus::Content;
        if app.current_screen() == Screen::Locales {
            app.start_locales_edit();
        }
        return;
    }

    match app.current_screen() {
        Screen::Locales => handle_enter_locales(app),
        Screen::MirrorsRepos => handle_enter_mirrors(app),
        Screen::Disks => handle_enter_disks(app),
        Screen::DiskEncryption => handle_enter_diskenc(app),
        Screen::Abort => app.open_abort_confirm_popup(),
        Screen::SwapPartition => handle_enter_swap(app),
        Screen::Bootloader => handle_enter_bootloader(app),
        Screen::UnifiedKernelImages => handle_enter_uki(app),
        Screen::Audio => handle_enter_audio(app),
        Screen::Kernels => handle_enter_kernels(app),
        Screen::NetworkConfiguration => handle_enter_network(app),
        Screen::Hostname => handle_enter_hostname(app),
        Screen::RootPassword => handle_enter_rootpass(app),
        Screen::UserAccount => handle_enter_user(app),
        Screen::ExperienceMode => handle_enter_experience(app),
        Screen::AutomaticTimeSync => handle_enter_ats(app),
        Screen::SaveConfiguration => handle_enter_config(app),
        Screen::AdditionalPackages => handle_enter_addpkgs(app),
        Screen::Timezone => handle_enter_timezone(app),
        _ => {
            // Other decision screens: Continue advances to next section
            if app.selected_index + 1 < app.menu_entries.len() {
                app.selected_index += 1;
                app.list_state.select(Some(app.selected_index));
                app.focus = Focus::Content;
                if app.current_screen() == Screen::Locales {
                    app.start_locales_edit();
                }
            } else {
                app.focus = Focus::Menu;
            }
        }
    }
}

fn handle_enter_config(app: &mut AppState) {
    match app.config_focus_index {
        0 => {
            app.save_config();
            app.open_info_popup("Configuration saved.".into());
        }
        1 => match app.load_config() {
            Ok(()) => {
                let mut msg = String::from(
                    "Configuration loaded. Note: re-enter Disk encryption, Root, and User passwords.",
                );
                if !app.last_load_missing_sections.is_empty() {
                    msg.push_str(" Missing: ");
                    msg.push_str(&app.last_load_missing_sections.join(", "));
                }
                app.open_info_popup(msg);
            }
            Err(()) => app.info_message = "Failed to load configuration".into(),
        },
        2 => advance(app),
        _ => {}
    }
}

fn handle_enter_locales(app: &mut AppState) {
    if app.locales_focus_index <= 2 {
        app.open_locales_popup();
    } else if app.locales_focus_index == 3 {
        app.apply_locales_edit();
        advance(app);
    }
}

fn handle_enter_mirrors(app: &mut AppState) {
    match app.mirrors_focus_index {
        0 => app.open_mirrors_regions_popup(),
        4 => advance(app),
        2 => app.open_optional_repos_popup(),
        1 => app.open_mirrors_custom_server_input(),
        3 => app.open_mirrors_custom_repo_flow(),
        _ => {}
    }
}

fn advance(app: &mut AppState) {
    if app.selected_index + 1 < app.menu_entries.len() {
        // Mark current section as processed before advancing
        let screen = app.current_screen();
        app.processed_sections.insert(screen);
        app.selected_index += 1;
        app.list_state.select(Some(app.selected_index));
        app.focus = Focus::Content;
        if app.current_screen() == Screen::Locales {
            app.start_locales_edit();
        }
    } else {
        let screen = app.current_screen();
        app.processed_sections.insert(screen);
        app.focus = Focus::Menu;
    }
}

fn handle_enter_disks(app: &mut AppState) {
    if app.disks_focus_index <= 2 {
        app.disks_mode_index = app.disks_focus_index;
        if app.disks_mode_index == 0 || app.disks_mode_index == 1 {
            app.open_disks_device_list();
        }
    } else if app.disks_focus_index == 3 {
        advance(app);
    }
}

fn handle_enter_diskenc(app: &mut AppState) {
    let continue_index = if app.disk_encryption_type_index == 1 {
        4
    } else {
        1
    };
    if app.diskenc_focus_index == 0 {
        app.open_disk_encryption_type_popup();
    } else if app.disk_encryption_type_index == 1 {
        match app.diskenc_focus_index {
            1 => app.open_disk_encryption_password_input(),
            2 => app.open_disk_encryption_password_confirm_input(),
            3 => app.open_disk_encryption_partition_list(),
            idx if idx == continue_index => advance(app),
            _ => {}
        }
    } else if app.diskenc_focus_index == continue_index {
        advance(app);
    }
}

fn handle_enter_swap(app: &mut AppState) {
    if app.swap_focus_index == 0 {
        app.swap_enabled = !app.swap_enabled;
    } else if app.swap_focus_index == 1 {
        advance(app);
    }
}

fn handle_enter_bootloader(app: &mut AppState) {
    if app.bootloader_focus_index < 4 {
        app.bootloader_index = app.bootloader_focus_index;
        app.update_unified_kernel_images_visibility();
    } else {
        advance(app);
    }
}

fn handle_enter_uki(app: &mut AppState) {
    if app.uki_focus_index == 0 {
        app.uki_enabled = !app.uki_enabled;
    } else if app.uki_focus_index == 1 {
        advance(app);
    }
}
fn handle_enter_ats(app: &mut AppState) {
    match app.ats_focus_index {
        0 => app.ats_enabled = true,
        1 => app.ats_enabled = false,
        2 => advance(app),
        _ => {}
    }
}

fn handle_enter_kernels(app: &mut AppState) {
    match app.kernels_focus_index {
        0 => app.open_kernels_popup(),
        1 => advance(app),
        _ => {}
    }
}

fn handle_enter_hostname(app: &mut AppState) {
    if app.hostname_focus_index == 0 {
        app.open_hostname_input();
    } else if app.hostname_focus_index == 1 {
        advance(app);
    }
}

fn handle_enter_timezone(app: &mut AppState) {
    if app.timezone_focus_index == 0 {
        app.open_timezone_popup();
    } else if app.timezone_focus_index == 1 {
        advance(app);
    }
}
fn handle_enter_addpkgs(app: &mut AppState) {
    if app.addpkgs_focus_index == 0 {
        app.open_additional_package_input();
    } else if app.addpkgs_focus_index == 1 {
        advance(app);
    }
}

fn handle_enter_rootpass(app: &mut AppState) {
    match app.rootpass_focus_index {
        0 => app.open_root_password_input(),
        1 => app.open_root_password_confirm_input(),
        2 => advance(app),
        _ => {}
    }
}

fn handle_enter_user(app: &mut AppState) {
    match app.user_focus_index {
        0 => app.start_add_user_flow(),
        1 => {
            // Edit user: open selection popup if users present, else info
            if app.users.is_empty() {
                app.open_info_popup("No users to edit".into());
            } else {
                app.popup_kind = Some(crate::ui::app::PopupKind::UserSelectEdit);
                app.popup_open = true;
                app.popup_items = app.users.iter().map(|u| u.username.clone()).collect();
                app.popup_visible_indices = (0..app.popup_items.len()).collect();
                app.popup_selected_visible = 0;
                app.popup_in_search = false;
                app.popup_search_query.clear();
            }
        }
        2 => {
            // Delete user: open selection popup if users present, else info
            if app.users.is_empty() {
                app.open_info_popup("No users to delete".into());
            } else {
                app.popup_kind = Some(crate::ui::app::PopupKind::UserSelectDelete);
                app.popup_open = true;
                app.popup_items = app.users.iter().map(|u| u.username.clone()).collect();
                app.popup_visible_indices = (0..app.popup_items.len()).collect();
                app.popup_selected_visible = 0;
                app.popup_in_search = false;
                app.popup_search_query.clear();
            }
        }
        3 => advance(app),
        _ => {}
    }
}

fn handle_enter_experience(app: &mut AppState) {
    if app.experience_focus_index == 0 {
        // Ensure selecting Desktop updates the experience first, then open the DE popup
        app.experience_mode_index = 0;
        app.open_desktop_environment_popup();
    } else if app.experience_focus_index <= 3 {
        // Special handling for Minimal (index 1): don't set until user confirms (if needed)
        if app.experience_focus_index == 1 {
            if !app.selected_desktop_envs.is_empty()
                || !app.selected_env_packages.is_empty()
                || app.selected_login_manager.is_some()
                || app.login_manager_user_set
            {
                // Ask for confirmation first; experience_mode_index remains unchanged until Yes
                app.open_minimal_clear_confirm();
            } else {
                // No selections to clear: set Minimal immediately
                app.experience_mode_index = 1;
                app.selected_desktop_envs.clear();
                app.selected_env_packages.clear();
                app.selected_login_manager = None;
                app.login_manager_user_set = false;
            }
        } else if app.experience_focus_index == 2 {
            // Server: open server type selection popup and set mode
            app.experience_mode_index = 2;
            app.open_server_type_popup();
        } else if app.experience_focus_index == 3 {
            // Xorg: open xorg type selection (single Type + Installed packages)
            app.experience_mode_index = 3;
            app.open_xorg_type_popup();
        } else {
            // Other modes can be set immediately
            app.experience_mode_index = app.experience_focus_index;
        }
    } else if app.experience_focus_index == 4 {
        advance(app);
    }
}

// The following are screen- and popup-specific helpers moved from input.rs
pub(crate) fn move_disks_up(app: &mut AppState) {
    if app.current_screen() != Screen::Disks || app.focus != Focus::Content {
        return;
    }
    if app.disks_focus_index == 0 {
        app.disks_focus_index = 3;
    } else {
        app.disks_focus_index -= 1;
    }
}
pub(crate) fn move_disks_down(app: &mut AppState) {
    if app.current_screen() != Screen::Disks || app.focus != Focus::Content {
        return;
    }
    app.disks_focus_index = (app.disks_focus_index + 1) % 4;
}
pub(crate) fn change_disks_value(app: &mut AppState, next: bool) {
    if app.current_screen() != Screen::Disks || app.focus != Focus::Content {
        return;
    }
    if app.disks_focus_index <= 2 {
        let mut idx = app.disks_mode_index;
        if next {
            idx = (idx + 1) % 3;
        } else {
            idx = (idx + 3 - 1) % 3;
        }
        app.disks_mode_index = idx;
    }
}
pub(crate) fn move_diskenc_up(app: &mut AppState) {
    if app.current_screen() != Screen::DiskEncryption || app.focus != Focus::Content {
        return;
    }
    let max = if app.disk_encryption_type_index == 1 {
        5
    } else {
        2
    };
    if app.diskenc_focus_index == 0 {
        app.diskenc_focus_index = max - 1;
    } else {
        app.diskenc_focus_index -= 1;
    }
}
pub(crate) fn move_diskenc_down(app: &mut AppState) {
    if app.current_screen() != Screen::DiskEncryption || app.focus != Focus::Content {
        return;
    }
    let max = if app.disk_encryption_type_index == 1 {
        5
    } else {
        2
    };
    app.diskenc_focus_index = (app.diskenc_focus_index + 1) % max;
}
pub(crate) fn change_diskenc_value(app: &mut AppState, next: bool) {
    if app.current_screen() != Screen::DiskEncryption || app.focus != Focus::Content {
        return;
    }
    if app.diskenc_focus_index == 0 {
        let mut idx = app.disk_encryption_type_index;
        if next {
            idx = (idx + 1) % 2;
        } else {
            idx = (idx + 2 - 1) % 2;
        }
        app.disk_encryption_type_index = idx;
    }
}
pub(crate) fn move_swap_up(app: &mut AppState) {
    if app.current_screen() != Screen::SwapPartition || app.focus != Focus::Content {
        return;
    }
    if app.swap_focus_index == 0 {
        app.swap_focus_index = 1;
    } else {
        app.swap_focus_index -= 1;
    }
}
pub(crate) fn move_swap_down(app: &mut AppState) {
    if app.current_screen() != Screen::SwapPartition || app.focus != Focus::Content {
        return;
    }
    app.swap_focus_index = (app.swap_focus_index + 1) % 2;
}
pub(crate) fn change_swap_value(app: &mut AppState, _next: bool) {
    if app.current_screen() != Screen::SwapPartition || app.focus != Focus::Content {
        return;
    }
    if app.swap_focus_index == 0 {
        app.swap_enabled = !app.swap_enabled;
    }
}
pub(crate) fn move_uki_up(app: &mut AppState) {
    if app.current_screen() != Screen::UnifiedKernelImages || app.focus != Focus::Content {
        return;
    }
    if app.uki_focus_index == 0 {
        app.uki_focus_index = 1;
    } else {
        app.uki_focus_index -= 1;
    }
}
pub(crate) fn move_uki_down(app: &mut AppState) {
    if app.current_screen() != Screen::UnifiedKernelImages || app.focus != Focus::Content {
        return;
    }
    app.uki_focus_index = (app.uki_focus_index + 1) % 2;
}
pub(crate) fn change_uki_value(app: &mut AppState, _next: bool) {
    if app.current_screen() != Screen::UnifiedKernelImages || app.focus != Focus::Content {
        return;
    }
    if app.uki_focus_index == 0 {
        app.uki_enabled = !app.uki_enabled;
    }
}
pub(crate) fn move_ats_up(app: &mut AppState) {
    if app.current_screen() != Screen::AutomaticTimeSync || app.focus != Focus::Content {
        return;
    }
    if app.ats_focus_index == 0 {
        app.ats_focus_index = 2;
    } else {
        app.ats_focus_index -= 1;
    }
}
pub(crate) fn move_ats_down(app: &mut AppState) {
    if app.current_screen() != Screen::AutomaticTimeSync || app.focus != Focus::Content {
        return;
    }
    app.ats_focus_index = (app.ats_focus_index + 1) % 3;
}
pub(crate) fn change_ats_value(app: &mut AppState, next: bool) {
    if app.current_screen() != Screen::AutomaticTimeSync || app.focus != Focus::Content {
        return;
    }
    if app.ats_focus_index <= 1 {
        if next {
            app.ats_focus_index = (app.ats_focus_index + 1) % 2;
        } else {
            app.ats_focus_index = (app.ats_focus_index + 2 - 1) % 2;
        }
        app.ats_enabled = app.ats_focus_index == 0;
    }
}
pub(crate) fn move_experience_up(app: &mut AppState) {
    if app.current_screen() != Screen::ExperienceMode || app.focus != Focus::Content {
        return;
    }
    if app.experience_focus_index == 0 {
        app.experience_focus_index = 4;
    } else {
        app.experience_focus_index -= 1;
    }
}
pub(crate) fn move_experience_down(app: &mut AppState) {
    if app.current_screen() != Screen::ExperienceMode || app.focus != Focus::Content {
        return;
    }
    app.experience_focus_index = (app.experience_focus_index + 1) % 5;
}
pub(crate) fn change_experience_value(app: &mut AppState, _next: bool) {
    if app.current_screen() != Screen::ExperienceMode || app.focus != Focus::Content {
        return;
    }
    if app.experience_focus_index <= 3 {
        app.experience_mode_index = app.experience_focus_index;
    }
}
pub(crate) fn move_bootloader_up(app: &mut AppState) {
    if app.current_screen() != Screen::Bootloader || app.focus != Focus::Content {
        return;
    }
    if app.bootloader_focus_index == 0 {
        app.bootloader_focus_index = 4;
    } else {
        app.bootloader_focus_index -= 1;
    }
}
pub(crate) fn move_bootloader_down(app: &mut AppState) {
    if app.current_screen() != Screen::Bootloader || app.focus != Focus::Content {
        return;
    }
    app.bootloader_focus_index = (app.bootloader_focus_index + 1) % 5;
}
pub(crate) fn change_bootloader_value(app: &mut AppState, _next: bool) {
    if app.current_screen() != Screen::Bootloader || app.focus != Focus::Content {
        return;
    }
    if app.bootloader_focus_index < 4 {
        app.bootloader_index = app.bootloader_focus_index;
    }
}

pub(crate) fn move_kernels_up(app: &mut AppState) {
    if app.current_screen() != Screen::Kernels || app.focus != Focus::Content {
        return;
    }
    if app.kernels_focus_index == 0 {
        app.kernels_focus_index = 1;
    } else {
        app.kernels_focus_index -= 1;
    }
}

pub(crate) fn move_kernels_down(app: &mut AppState) {
    if app.current_screen() != Screen::Kernels || app.focus != Focus::Content {
        return;
    }
    app.kernels_focus_index = (app.kernels_focus_index + 1) % 2;
}

pub(crate) fn change_kernels_value(_app: &mut AppState, _next: bool) {}

pub(crate) fn move_audio_up(app: &mut AppState) {
    if app.current_screen() != Screen::Audio || app.focus != Focus::Content {
        return;
    }
    if app.audio_focus_index == 0 {
        app.audio_focus_index = 3;
    } else {
        app.audio_focus_index -= 1;
    }
}

pub(crate) fn move_audio_down(app: &mut AppState) {
    if app.current_screen() != Screen::Audio || app.focus != Focus::Content {
        return;
    }
    app.audio_focus_index = (app.audio_focus_index + 1) % 4;
}

pub(crate) fn change_audio_value(app: &mut AppState, _next: bool) {
    if app.current_screen() != Screen::Audio || app.focus != Focus::Content {
        return;
    }
    if app.audio_focus_index < 3 {
        app.audio_index = app.audio_focus_index;
    }
}

fn handle_enter_audio(app: &mut AppState) {
    if app.audio_focus_index < 3 {
        app.audio_index = app.audio_focus_index;
    } else {
        advance(app);
    }
}

pub(crate) fn move_network_up(app: &mut AppState) {
    if app.current_screen() != Screen::NetworkConfiguration || app.focus != Focus::Content {
        return;
    }
    if app.network_focus_index == 0 {
        app.network_focus_index = 4;
    } else {
        app.network_focus_index -= 1;
    }
}

pub(crate) fn move_network_down(app: &mut AppState) {
    if app.current_screen() != Screen::NetworkConfiguration || app.focus != Focus::Content {
        return;
    }
    app.network_focus_index = (app.network_focus_index + 1) % 5;
}

pub(crate) fn change_network_value(app: &mut AppState, _next: bool) {
    if app.current_screen() != Screen::NetworkConfiguration || app.focus != Focus::Content {
        return;
    }
    if app.network_focus_index < 3 {
        let previous_mode = app.network_mode_index;
        app.network_mode_index = app.network_focus_index;
        // If leaving Manual mode (1) to Copy ISO (0) or NetworkManager (2), clear manual interfaces
        if previous_mode == 1 && app.network_mode_index != 1 {
            app.network_configs.clear();
        }
    }
}

fn handle_enter_network(app: &mut AppState) {
    if app.network_focus_index < 3 {
        let previous_mode = app.network_mode_index;
        app.network_mode_index = app.network_focus_index;
        // If leaving Manual mode (1) to Copy ISO (0) or NetworkManager (2), clear manual interfaces
        if previous_mode == 1 && app.network_mode_index != 1 {
            app.network_configs.clear();
        }
    } else if app.network_focus_index == 3 {
        // Add interface (only in Manual)
        if app.network_mode_index != 1 {
            app.open_info_popup("Switch to Manual configuration to add interfaces".into());
            return;
        }
        app.open_network_interfaces_popup();
    } else {
        advance(app);
    }
}
pub(crate) fn move_hostname_up(app: &mut AppState) {
    if app.current_screen() != Screen::Hostname || app.focus != Focus::Content {
        return;
    }
    if app.hostname_focus_index == 0 {
        app.hostname_focus_index = 1;
    } else {
        app.hostname_focus_index -= 1;
    }
}
pub(crate) fn move_hostname_down(app: &mut AppState) {
    if app.current_screen() != Screen::Hostname || app.focus != Focus::Content {
        return;
    }
    app.hostname_focus_index = (app.hostname_focus_index + 1) % 2;
}
pub(crate) fn change_hostname_value(_app: &mut AppState, _next: bool) {}
pub(crate) fn move_timezone_up(app: &mut AppState) {
    if app.current_screen() != Screen::Timezone || app.focus != Focus::Content {
        return;
    }
    if app.timezone_focus_index == 0 {
        app.timezone_focus_index = 1;
    } else {
        app.timezone_focus_index -= 1;
    }
}
pub(crate) fn move_timezone_down(app: &mut AppState) {
    if app.current_screen() != Screen::Timezone || app.focus != Focus::Content {
        return;
    }
    app.timezone_focus_index = (app.timezone_focus_index + 1) % 2;
}
pub(crate) fn change_timezone_value(_app: &mut AppState, _next: bool) {}
pub(crate) fn move_addpkgs_up(app: &mut AppState) {
    if app.current_screen() != Screen::AdditionalPackages || app.focus != Focus::Content {
        return;
    }
    // Move selection up within packages list (do not change focus index here)
    if !app.additional_packages.is_empty() {
        if app.addpkgs_selected_index == 0 {
            app.addpkgs_selected_index = app.additional_packages.len() - 1;
        } else {
            app.addpkgs_selected_index -= 1;
        }
    }
}
pub(crate) fn move_addpkgs_down(app: &mut AppState) {
    if app.current_screen() != Screen::AdditionalPackages || app.focus != Focus::Content {
        return;
    }
    // Move selection down within packages list (do not change focus index here)
    if !app.additional_packages.is_empty() {
        app.addpkgs_selected_index =
            (app.addpkgs_selected_index + 1) % app.additional_packages.len();
    }
}
pub(crate) fn change_addpkgs_value(app: &mut AppState, next: bool) {
    if app.current_screen() != Screen::AdditionalPackages || app.focus != Focus::Content {
        return;
    }
    // Use left/right (h/l) to switch between Add package and Continue
    if next {
        app.addpkgs_focus_index = (app.addpkgs_focus_index + 1) % 2;
    } else {
        app.addpkgs_focus_index = (app.addpkgs_focus_index + 2 - 1) % 2;
    }
}
pub(crate) fn move_rootpass_up(app: &mut AppState) {
    if app.current_screen() != Screen::RootPassword || app.focus != Focus::Content {
        return;
    }
    if app.rootpass_focus_index == 0 {
        app.rootpass_focus_index = 2;
    } else {
        app.rootpass_focus_index -= 1;
    }
}
pub(crate) fn move_rootpass_down(app: &mut AppState) {
    if app.current_screen() != Screen::RootPassword || app.focus != Focus::Content {
        return;
    }
    app.rootpass_focus_index = (app.rootpass_focus_index + 1) % 3;
}
pub(crate) fn change_rootpass_value(_app: &mut AppState, _next: bool) {}
pub(crate) fn move_user_up(app: &mut AppState) {
    if app.current_screen() != Screen::UserAccount || app.focus != Focus::Content {
        return;
    }
    if app.user_focus_index == 0 {
        app.user_focus_index = 3;
    } else {
        app.user_focus_index -= 1;
    }
}
pub(crate) fn move_user_down(app: &mut AppState) {
    if app.current_screen() != Screen::UserAccount || app.focus != Focus::Content {
        return;
    }
    app.user_focus_index = (app.user_focus_index + 1) % 4;
}
pub(crate) fn change_user_value(_app: &mut AppState, _next: bool) {}
pub(crate) fn move_config_up(app: &mut AppState) {
    if app.current_screen() != Screen::SaveConfiguration || app.focus != Focus::Content {
        return;
    }
    if app.config_focus_index == 0 {
        app.config_focus_index = 2;
    } else {
        app.config_focus_index -= 1;
    }
}
pub(crate) fn move_config_down(app: &mut AppState) {
    if app.current_screen() != Screen::SaveConfiguration || app.focus != Focus::Content {
        return;
    }
    app.config_focus_index = (app.config_focus_index + 1) % 3;
}
pub(crate) fn change_config_value(_app: &mut AppState, _next: bool) {}

// Popup vertical navigation (including DE/WM boundary logic)
pub(crate) fn popup_move_up(app: &mut AppState) {
    if matches!(
        app.popup_kind,
        Some(crate::ui::app::PopupKind::DesktopEnvSelect)
    ) && !app.popup_visible_indices.is_empty()
    {
        let is_wm_name = |s: &str| -> bool {
            matches!(
                s,
                "Awesome" | "Bspwm" | "Enlightenment" | "Hyprland" | "Qtile" | "Sway" | "i3-wm"
            )
        };
        let mut de_positions: Vec<usize> = Vec::new();
        let mut wm_positions: Vec<usize> = Vec::new();
        for (vis_pos, &gi) in app.popup_visible_indices.iter().enumerate() {
            if let Some(name) = app.popup_items.get(gi) {
                if is_wm_name(name) {
                    wm_positions.push(vis_pos);
                } else {
                    de_positions.push(vis_pos);
                }
            }
        }
        let current_vis = app.popup_selected_visible;
        let current_is_wm = if let Some(&gi) = app.popup_visible_indices.get(current_vis)
            && let Some(name) = app.popup_items.get(gi)
        {
            is_wm_name(name)
        } else {
            false
        };
        if current_is_wm {
            if let Some(idx) = wm_positions.iter().position(|&p| p == current_vis) {
                if idx > 0 {
                    app.popup_selected_visible = wm_positions[idx - 1];
                } else if let Some(&last_de) = de_positions.last() {
                    app.popup_selected_visible = last_de;
                }
            }
        } else if let Some(idx) = de_positions.iter().position(|&p| p == current_vis) {
            if idx > 0 {
                app.popup_selected_visible = de_positions[idx - 1];
            } else if let Some(&last_wm) = wm_positions.last() {
                app.popup_selected_visible = last_wm;
            }
        }
        return;
    }
    if app.popup_selected_visible == 0 {
        if !app.popup_visible_indices.is_empty() {
            app.popup_selected_visible = app.popup_visible_indices.len() - 1;
        }
    } else {
        app.popup_selected_visible = app.popup_selected_visible.saturating_sub(1);
    }
}

pub(crate) fn popup_move_down(app: &mut AppState) {
    if matches!(
        app.popup_kind,
        Some(crate::ui::app::PopupKind::DesktopEnvSelect)
    ) && !app.popup_visible_indices.is_empty()
    {
        let is_wm_name = |s: &str| -> bool {
            matches!(
                s,
                "Awesome" | "Bspwm" | "Enlightenment" | "Hyprland" | "Qtile" | "Sway" | "i3-wm"
            )
        };
        let mut de_positions: Vec<usize> = Vec::new();
        let mut wm_positions: Vec<usize> = Vec::new();
        for (vis_pos, &gi) in app.popup_visible_indices.iter().enumerate() {
            if let Some(name) = app.popup_items.get(gi) {
                if is_wm_name(name) {
                    wm_positions.push(vis_pos);
                } else {
                    de_positions.push(vis_pos);
                }
            }
        }
        let current_vis = app.popup_selected_visible;
        let current_is_wm = if let Some(&gi) = app.popup_visible_indices.get(current_vis)
            && let Some(name) = app.popup_items.get(gi)
        {
            is_wm_name(name)
        } else {
            false
        };
        if current_is_wm {
            if let Some(idx) = wm_positions.iter().position(|&p| p == current_vis) {
                if idx + 1 < wm_positions.len() {
                    app.popup_selected_visible = wm_positions[idx + 1];
                } else if let Some(&first_de) = de_positions.first() {
                    app.popup_selected_visible = first_de;
                }
            }
        } else if let Some(idx) = de_positions.iter().position(|&p| p == current_vis) {
            if idx + 1 < de_positions.len() {
                app.popup_selected_visible = de_positions[idx + 1];
            } else if let Some(&first_wm) = wm_positions.first() {
                app.popup_selected_visible = first_wm;
            }
        }
        return;
    }
    if !app.popup_visible_indices.is_empty() {
        app.popup_selected_visible =
            (app.popup_selected_visible + 1) % app.popup_visible_indices.len();
    }
}
