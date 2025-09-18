#![allow(dead_code)]
use super::{
    addpkgs, ats, audio, bootloader, disk_encryption as de, disks, experience, hostname, kernels,
    locales, mirrors, network, rootpass, save_config, swap, timezone, uki, user,
};
use crate::app::{AppState, Focus, Screen};

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
    locales::move_locales_focus(app, forward);
}

// Specific screen wrappers required by top.rs are defined later in this file

pub(crate) fn move_screen_up(app: &mut AppState) {
    match app.current_screen() {
        Screen::Locales => locales::move_locales_up(app),
        Screen::MirrorsRepos => mirrors::move_mirrors_repos_up(app),
        Screen::Disks => disks::move_disks_up(app),
        Screen::DiskEncryption => de::move_diskenc_up(app),
        Screen::Bootloader => bootloader::move_bootloader_up(app),
        Screen::Audio => audio::move_audio_up(app),
        Screen::Kernels => kernels::move_kernels_up(app),
        Screen::NetworkConfiguration => network::move_network_up(app),
        Screen::Hostname => hostname::move_hostname_up(app),
        Screen::RootPassword => rootpass::move_rootpass_up(app),
        Screen::UserAccount => user::move_user_up(app),
        Screen::SwapPartition => swap::move_swap_up(app),
        Screen::UnifiedKernelImages => uki::move_uki_up(app),
        Screen::AutomaticTimeSync => ats::move_ats_up(app),
        Screen::ExperienceMode => experience::move_experience_up(app),
        Screen::SaveConfiguration => save_config::move_config_up(app),
        Screen::AdditionalPackages => addpkgs::move_addpkgs_up(app),
        Screen::Timezone => timezone::move_timezone_up(app),
        _ => {}
    }
}

pub(crate) fn move_screen_down(app: &mut AppState) {
    match app.current_screen() {
        Screen::Locales => locales::move_locales_down(app),
        Screen::MirrorsRepos => mirrors::move_mirrors_repos_down(app),
        Screen::Disks => disks::move_disks_down(app),
        Screen::DiskEncryption => de::move_diskenc_down(app),
        Screen::Bootloader => bootloader::move_bootloader_down(app),
        Screen::Audio => audio::move_audio_down(app),
        Screen::Kernels => kernels::move_kernels_down(app),
        Screen::NetworkConfiguration => network::move_network_down(app),
        Screen::Hostname => hostname::move_hostname_down(app),
        Screen::RootPassword => rootpass::move_rootpass_down(app),
        Screen::UserAccount => user::move_user_down(app),
        Screen::SwapPartition => swap::move_swap_down(app),
        Screen::UnifiedKernelImages => uki::move_uki_down(app),
        Screen::AutomaticTimeSync => ats::move_ats_down(app),
        Screen::ExperienceMode => experience::move_experience_down(app),
        Screen::SaveConfiguration => save_config::move_config_down(app),
        Screen::AdditionalPackages => addpkgs::move_addpkgs_down(app),
        Screen::Timezone => timezone::move_timezone_down(app),
        _ => {}
    }
}

pub(crate) fn change_value(app: &mut AppState, next: bool) {
    match app.current_screen() {
        Screen::Locales => locales::change_locales_value(app, next),
        Screen::Disks => disks::change_disks_value(app, next),
        Screen::DiskEncryption => de::change_diskenc_value(app, next),
        Screen::Bootloader => bootloader::change_bootloader_value(app, next),
        Screen::Audio => audio::change_audio_value(app, next),
        Screen::Kernels => kernels::change_kernels_value(app, next),
        Screen::NetworkConfiguration => network::change_network_value(app, next),
        Screen::Hostname => hostname::change_hostname_value(app, next),
        Screen::RootPassword => rootpass::change_rootpass_value(app, next),
        Screen::UserAccount => user::change_user_value(app, next),
        Screen::SwapPartition => swap::change_swap_value(app, next),
        Screen::UnifiedKernelImages => uki::change_uki_value(app, next),
        Screen::AutomaticTimeSync => ats::change_ats_value(app, next),
        Screen::ExperienceMode => experience::change_experience_value(app, next),
        Screen::SaveConfiguration => save_config::change_config_value(app, next),
        Screen::AdditionalPackages => addpkgs::change_addpkgs_value(app, next),
        Screen::Timezone => timezone::change_timezone_value(app, next),
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
        Screen::Locales => locales::handle_enter_locales(app),
        Screen::MirrorsRepos => mirrors::handle_enter_mirrors(app),
        Screen::Disks => disks::handle_enter_disks(app),
        Screen::DiskEncryption => de::handle_enter_diskenc(app),
        Screen::Install => {
            app.start_install();
        }
        Screen::Abort => app.open_abort_confirm_popup(),
        Screen::SwapPartition => swap::handle_enter_swap(app),
        Screen::Bootloader => bootloader::handle_enter_bootloader(app),
        Screen::UnifiedKernelImages => uki::handle_enter_uki(app),
        Screen::Audio => audio::handle_enter_audio(app),
        Screen::Kernels => kernels::handle_enter_kernels(app),
        Screen::NetworkConfiguration => network::handle_enter_network(app),
        Screen::Hostname => hostname::handle_enter_hostname(app),
        Screen::RootPassword => rootpass::handle_enter_rootpass(app),
        Screen::UserAccount => user::handle_enter_user(app),
        Screen::ExperienceMode => experience::handle_enter_experience(app),
        Screen::AutomaticTimeSync => ats::handle_enter_ats(app),
        Screen::SaveConfiguration => save_config::handle_enter_config(app),
        Screen::AdditionalPackages => addpkgs::handle_enter_addpkgs(app),
        Screen::Timezone => timezone::handle_enter_timezone(app),
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

// Deprecated local handlers now delegated to modules; keep only wrappers if needed
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
    locales::handle_enter_locales(app);
}

fn handle_enter_mirrors(app: &mut AppState) {
    mirrors::handle_enter_mirrors(app);
}

fn advance(app: &mut AppState) {
    super::common::advance(app);
}

fn handle_enter_disks(app: &mut AppState) {
    disks::handle_enter_disks(app);
}

fn handle_enter_diskenc(app: &mut AppState) {
    de::handle_enter_diskenc(app);
}

fn handle_enter_swap(app: &mut AppState) {
    swap::handle_enter_swap(app);
}

fn handle_enter_bootloader(app: &mut AppState) {
    bootloader::handle_enter_bootloader(app);
}

fn handle_enter_uki(app: &mut AppState) {
    uki::handle_enter_uki(app);
}
fn handle_enter_ats(app: &mut AppState) {
    ats::handle_enter_ats(app);
}

fn handle_enter_kernels(app: &mut AppState) {
    kernels::handle_enter_kernels(app);
}

fn handle_enter_hostname(app: &mut AppState) {
    hostname::handle_enter_hostname(app);
}

fn handle_enter_timezone(app: &mut AppState) {
    timezone::handle_enter_timezone(app);
}
fn handle_enter_addpkgs(app: &mut AppState) {
    addpkgs::handle_enter_addpkgs(app);
}

fn handle_enter_rootpass(app: &mut AppState) {
    rootpass::handle_enter_rootpass(app);
}

fn handle_enter_user(app: &mut AppState) {
    user::handle_enter_user(app);
}

fn handle_enter_experience(app: &mut AppState) {
    experience::handle_enter_experience(app);
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
    addpkgs::move_addpkgs_up(app);
}
pub(crate) fn move_addpkgs_down(app: &mut AppState) {
    addpkgs::move_addpkgs_down(app);
}
pub(crate) fn change_addpkgs_value(app: &mut AppState, next: bool) {
    addpkgs::change_addpkgs_value(app, next);
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
