use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

use super::app::{AppState, Focus, Screen};

// Returns true if the app should quit
pub fn handle_event(app: &mut AppState, ev: Event) -> bool {
    match ev {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            // Quit only with Ctrl-C; ESC never quits
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Esc | KeyCode::Char('q') => {
                if app.popup_open {
                    app.close_popup();
                } else if app.focus == Focus::Content {
                    if app.current_screen() == Screen::Locales { app.discard_locales_edit(); }
                    app.focus = Focus::Menu;
                }
                return false;
            }
            // When popup is open, handle popup keys only
            _ if app.popup_open => return handle_popup_keys(app, key.code),
            // When command line is open, handle only command keys
            _ if app.cmdline_open => return handle_cmdline_keys(app, key.code),
            // Vim motions for menu and decision navigation
            KeyCode::Up | KeyCode::Char('k') => {
                if app.focus == Focus::Menu { move_menu_up(app); }
                else { move_screen_up(app); }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.focus == Focus::Menu { move_menu_down(app); }
                else { move_screen_down(app); }
            }
            
            KeyCode::Enter => {
                if app.focus == Focus::Menu {
                    app.info_message = format!("Opened '{}'", app.menu_entries[app.selected_index].label);
                    app.focus = Focus::Content;
                    if app.current_screen() == Screen::Locales { app.start_locales_edit(); }
                } else if app.current_screen() == Screen::Locales {
                    if app.locales_focus_index <= 2 {
                        app.open_locales_popup();
                    } else if app.locales_focus_index == 3 {
                        // Continue: apply edits and jump to next section (keep decision focus)
                        app.apply_locales_edit();
                        if app.selected_index + 1 < app.menu_entries.len() {
                            app.selected_index += 1;
                            app.list_state.select(Some(app.selected_index));
                            app.focus = Focus::Content;
                            if app.current_screen() == Screen::Locales { app.start_locales_edit(); }
                        } else {
                            app.focus = Focus::Menu;
                        }
                    }
                } else if app.current_screen() == Screen::MirrorsRepos {
                    // Activate items; open popup for Select Regions, Continue advances
                    match app.mirrors_focus_index {
                        0 => { // Select Regions
                            app.open_mirrors_regions_popup();
                        }
                        4 => { // Continue
                            if app.selected_index + 1 < app.menu_entries.len() {
                                app.selected_index += 1;
                                app.list_state.select(Some(app.selected_index));
                                app.focus = Focus::Content;
                                if app.current_screen() == Screen::Locales { app.start_locales_edit(); }
                            } else {
                                app.focus = Focus::Menu;
                            }
                        }
                        2 => { // Optional repositories
                            app.open_optional_repos_popup();
                        }
                        1 => { // Add custom servers
                            app.open_mirrors_custom_server_input();
                        }
                        3 => { // Add custom repository
                            app.open_mirrors_custom_repo_flow();
                        }
                        _ => {}
                    }
                } else if app.current_screen() == Screen::Disks {
                    // Enter selects current option or Continue
                    if app.disks_focus_index <= 2 {
                        app.disks_mode_index = app.disks_focus_index;
                        // Open device list for Best-effort or Manual
                        if app.disks_mode_index == 0 || app.disks_mode_index == 1 {
                            app.open_disks_device_list();
                        }
                    } else if app.disks_focus_index == 3 {
                        if app.selected_index + 1 < app.menu_entries.len() {
                            app.selected_index += 1;
                            app.list_state.select(Some(app.selected_index));
                            app.focus = Focus::Content;
                            if app.current_screen() == Screen::Locales { app.start_locales_edit(); }
                        } else {
                            app.focus = Focus::Menu;
                        }
                    }
                } else if app.current_screen() == Screen::DiskEncryption {
                    let continue_index = if app.disk_encryption_type_index == 1 { 4 } else { 1 };
                    if app.diskenc_focus_index == 0 {
                        app.open_disk_encryption_type_popup();
                    } else if app.disk_encryption_type_index == 1 {
                        match app.diskenc_focus_index {
                            1 => app.open_disk_encryption_password_input(),
                            2 => app.open_disk_encryption_password_confirm_input(),
                            3 => app.open_disk_encryption_partition_list(),
                            idx if idx == continue_index => {
                                if app.selected_index + 1 < app.menu_entries.len() {
                                    app.selected_index += 1;
                                    app.list_state.select(Some(app.selected_index));
                                    app.focus = Focus::Content;
                                    if app.current_screen() == Screen::Locales { app.start_locales_edit(); }
                                } else { app.focus = Focus::Menu; }
                            }
                            _ => {}
                        }
                    } else if app.diskenc_focus_index == continue_index {
                        if app.selected_index + 1 < app.menu_entries.len() {
                            app.selected_index += 1;
                            app.list_state.select(Some(app.selected_index));
                            app.focus = Focus::Content;
                            if app.current_screen() == Screen::Locales { app.start_locales_edit(); }
                        } else { app.focus = Focus::Menu; }
                    }
                } else if app.current_screen() == Screen::Abort {
                    app.open_abort_confirm_popup();
                } else if app.current_screen() == Screen::SwapPartition {
                    if app.swap_focus_index == 0 {
                        app.swap_enabled = !app.swap_enabled;
                    } else if app.swap_focus_index == 1 {
                        if app.selected_index + 1 < app.menu_entries.len() {
                            app.selected_index += 1;
                            app.list_state.select(Some(app.selected_index));
                            app.focus = Focus::Content;
                            if app.current_screen() == Screen::Locales { app.start_locales_edit(); }
                        } else { app.focus = Focus::Menu; }
                    }
                } else if app.current_screen() == Screen::Bootloader {
                    if app.bootloader_focus_index < 4 {
                        app.bootloader_index = app.bootloader_focus_index;
                        // Update visibility of UKI in menu after selection
                        app.update_unified_kernel_images_visibility();
                    } else {
                        if app.selected_index + 1 < app.menu_entries.len() {
                            app.selected_index += 1;
                            app.list_state.select(Some(app.selected_index));
                            app.focus = Focus::Content;
                            if app.current_screen() == Screen::Locales { app.start_locales_edit(); }
                        } else { app.focus = Focus::Menu; }
                    }
                } else if app.current_screen() == Screen::Hostname {
                    if app.hostname_focus_index == 0 {
                        app.open_hostname_input();
                    } else if app.hostname_focus_index == 1 {
                        if app.selected_index + 1 < app.menu_entries.len() {
                            app.selected_index += 1;
                            app.list_state.select(Some(app.selected_index));
                            app.focus = Focus::Content;
                            if app.current_screen() == Screen::Locales { app.start_locales_edit(); }
                        } else { app.focus = Focus::Menu; }
                    }
                } else if app.current_screen() == Screen::RootPassword {
                    match app.rootpass_focus_index {
                        0 => app.open_root_password_input(),
                        1 => app.open_root_password_confirm_input(),
                        2 => {
                            if app.selected_index + 1 < app.menu_entries.len() {
                                app.selected_index += 1;
                                app.list_state.select(Some(app.selected_index));
                                app.focus = Focus::Content;
                                if app.current_screen() == Screen::Locales { app.start_locales_edit(); }
                            } else { app.focus = Focus::Menu; }
                        }
                        _ => {}
                    }
                } else if app.current_screen() == Screen::UserAccount {
                    match app.user_focus_index {
                        0 => app.start_add_user_flow(),
                        1 => {
                            if app.selected_index + 1 < app.menu_entries.len() {
                                app.selected_index += 1;
                                app.list_state.select(Some(app.selected_index));
                                app.focus = Focus::Content;
                                if app.current_screen() == Screen::Locales { app.start_locales_edit(); }
                            } else { app.focus = Focus::Menu; }
                        }
                        _ => {}
                    }
                } else if app.current_screen() == Screen::SaveConfiguration {
                    match app.config_focus_index {
                        0 => { app.save_config(); }
                        1 => {
                            match app.load_config() {
                                Ok(()) => {
                                    let mut msg = String::from("Configuration loaded. Note: re-enter Disk encryption, Root, and User passwords.");
                                    if !app.last_load_missing_sections.is_empty() {
                                        msg.push_str(" Missing: ");
                                        msg.push_str(&app.last_load_missing_sections.join(", "));
                                    }
                                    app.open_info_popup(msg);
                                }
                                Err(()) => app.info_message = "Failed to load configuration".into(),
                            }
                        }
                        2 => {
                            if app.selected_index + 1 < app.menu_entries.len() {
                                app.selected_index += 1;
                                app.list_state.select(Some(app.selected_index));
                                app.focus = Focus::Content;
                                if app.current_screen() == Screen::Locales { app.start_locales_edit(); }
                            } else { app.focus = Focus::Menu; }
                        }
                        _ => {}
                    }
                } else {
                    // Other decision screens: Continue advances to next section
                    if app.selected_index + 1 < app.menu_entries.len() {
                        app.selected_index += 1;
                        app.list_state.select(Some(app.selected_index));
                        app.focus = Focus::Content;
                        if app.current_screen() == Screen::Locales { app.start_locales_edit(); }
                    } else {
                        app.focus = Focus::Menu;
                    }
                }
            }
            KeyCode::Tab => move_locales_focus(app, true),
            KeyCode::BackTab => move_locales_focus(app, false),
            // Vim motions inside decision menu
            KeyCode::Left | KeyCode::Char('h') => change_value(app, false),
            KeyCode::Right | KeyCode::Char('l') => change_value(app, true),
            // Open command line (Locales)
            KeyCode::Char(':') => { if app.focus == Focus::Content { app.cmdline_open = true; app.cmdline_buffer.clear(); } },
            _ => {}
        },
        // No mouse support in TTY
        Event::Resize(_, _) => {}
        _ => {}
    }
    false
}

// No mouse helpers

fn move_menu_up(app: &mut AppState) {
    if app.selected_index > 0 {
        app.selected_index -= 1;
    } else if !app.menu_entries.is_empty() {
        app.selected_index = app.menu_entries.len() - 1;
    }
    app.list_state.select(Some(app.selected_index));
}

fn move_menu_down(app: &mut AppState) {
    if app.selected_index + 1 < app.menu_entries.len() {
        app.selected_index += 1;
    } else if !app.menu_entries.is_empty() {
        app.selected_index = 0;
    }
    app.list_state.select(Some(app.selected_index));
}

fn move_locales_focus(app: &mut AppState, forward: bool) {
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

fn change_locales_value(app: &mut AppState, next: bool) {
    if app.current_screen() != Screen::Locales || app.focus != Focus::Content {
        return;
    }
    match app.locales_focus_index {
        0 => cycle_index(&mut app.draft_keyboard_layout_index, app.keyboard_layout_options.len(), next),
        1 => cycle_index(&mut app.draft_locale_language_index, app.locale_language_options.len(), next),
        2 => cycle_index(&mut app.draft_locale_encoding_index, app.locale_encoding_options.len(), next),
        _ => {}
    }
}

fn cycle_index(index: &mut usize, len: usize, next: bool) {
    if len == 0 { return; }
    if next {
        *index = (*index + 1) % len;
    } else {
        *index = (*index + len - 1) % len;
    }
}

fn move_locales_up(app: &mut AppState) {
    if app.current_screen() != Screen::Locales || app.focus != Focus::Content { return; }
    if app.locales_focus_index == 0 { app.locales_focus_index = 3; } else { app.locales_focus_index -= 1; }
}

fn move_locales_down(app: &mut AppState) {
    if app.current_screen() != Screen::Locales || app.focus != Focus::Content { return; }
    app.locales_focus_index = (app.locales_focus_index + 1) % 4;
}

fn move_mirrors_repos_up(app: &mut AppState) {
    if app.current_screen() != Screen::MirrorsRepos || app.focus != Focus::Content { return; }
    if app.mirrors_focus_index == 0 { app.mirrors_focus_index = 4; } else { app.mirrors_focus_index -= 1; }
}

fn move_mirrors_repos_down(app: &mut AppState) {
    if app.current_screen() != Screen::MirrorsRepos || app.focus != Focus::Content { return; }
    app.mirrors_focus_index = (app.mirrors_focus_index + 1) % 5;
}

fn move_screen_up(app: &mut AppState) {
    match app.current_screen() {
        Screen::Locales => move_locales_up(app),
        Screen::MirrorsRepos => move_mirrors_repos_up(app),
        Screen::Disks => move_disks_up(app),
        Screen::DiskEncryption => move_diskenc_up(app),
        Screen::Bootloader => move_bootloader_up(app),
        Screen::Hostname => move_hostname_up(app),
        Screen::RootPassword => move_rootpass_up(app),
        Screen::UserAccount => move_user_up(app),
        Screen::SwapPartition => move_swap_up(app),
        Screen::SaveConfiguration => move_config_up(app),
        _ => {}
    }
}

fn move_screen_down(app: &mut AppState) {
    match app.current_screen() {
        Screen::Locales => move_locales_down(app),
        Screen::MirrorsRepos => move_mirrors_repos_down(app),
        Screen::Disks => move_disks_down(app),
        Screen::DiskEncryption => move_diskenc_down(app),
        Screen::Bootloader => move_bootloader_down(app),
        Screen::Hostname => move_hostname_down(app),
        Screen::RootPassword => move_rootpass_down(app),
        Screen::UserAccount => move_user_down(app),
        Screen::SwapPartition => move_swap_down(app),
        Screen::SaveConfiguration => move_config_down(app),
        _ => {}
    }
}

fn change_value(app: &mut AppState, next: bool) {
    match app.current_screen() {
        Screen::Locales => change_locales_value(app, next),
        Screen::Disks => change_disks_value(app, next),
        Screen::DiskEncryption => change_diskenc_value(app, next),
        Screen::Bootloader => change_bootloader_value(app, next),
        Screen::Hostname => change_hostname_value(app, next),
        Screen::RootPassword => change_rootpass_value(app, next),
        Screen::UserAccount => change_user_value(app, next),
        Screen::SwapPartition => change_swap_value(app, next),
        Screen::SaveConfiguration => change_config_value(app, next),
        _ => {}
    }
}

fn handle_popup_keys(app: &mut AppState, code: KeyCode) -> bool {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => { app.close_popup(); return false; }
        KeyCode::Enter => {
            match app.popup_kind {
                Some(super::app::PopupKind::Info) => {
                    app.close_popup();
                    return false;
                }
                Some(super::app::PopupKind::HostnameInput) => {
                    app.hostname_value = app.custom_input_buffer.trim().to_string();
                    app.custom_input_buffer.clear();
                    app.close_popup();
                }
                Some(super::app::PopupKind::MirrorsRegions) => {
                    // nothing to apply on Enter; treat Enter as close
                    app.close_popup();
                }
                Some(super::app::PopupKind::RootPassword) => {
                    app.root_password = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    app.close_popup();
                }
                Some(super::app::PopupKind::RootPasswordConfirm) => {
                    app.root_password_confirm = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    if app.root_password != app.root_password_confirm { app.info_message = "Root passwords do not match".into(); } else { app.info_message.clear(); }
                    app.close_popup();
                }
                Some(super::app::PopupKind::UserAddUsername) => {
                    app.draft_user_username = app.custom_input_buffer.trim().to_string();
                    app.custom_input_buffer.clear();
                    app.close_popup();
                    if !app.draft_user_username.is_empty() { app.open_user_password_input(); }
                }
                Some(super::app::PopupKind::UserAddPassword) => {
                    app.draft_user_password = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    app.close_popup();
                    app.open_user_password_confirm_input();
                }
                Some(super::app::PopupKind::UserAddPasswordConfirm) => {
                    app.draft_user_password_confirm = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    if app.draft_user_password != app.draft_user_password_confirm {
                        app.info_message = "User passwords do not match".into();
                        app.close_popup();
                    } else {
                        app.info_message.clear();
                        app.close_popup();
                        app.open_user_sudo_select();
                    }
                }
                Some(super::app::PopupKind::UserAddSudo) => {
                    if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                        app.draft_user_is_sudo = global_idx == 0; // Yes
                        // finalize add user
                        app.users.push(super::app::UserAccount {
                            username: app.draft_user_username.clone(),
                            password: app.draft_user_password.clone(),
                            is_sudo: app.draft_user_is_sudo,
                        });
                        app.save_config();
                    }
                    app.close_popup();
                }
                Some(super::app::PopupKind::OptionalRepos) => {
                    app.close_popup();
                }
                Some(super::app::PopupKind::MirrorsCustomServerInput) => {
                    if !app.custom_input_buffer.trim().is_empty() {
                        app.mirrors_custom_servers.push(app.custom_input_buffer.trim().to_string());
                    }
                    app.custom_input_buffer.clear();
                    // stay open for multiple inputs
                }
                Some(super::app::PopupKind::MirrorsCustomRepoName) => {
                    if !app.custom_input_buffer.trim().is_empty() {
                        app.draft_repo_name = app.custom_input_buffer.trim().to_string();
                        app.open_mirrors_custom_repo_url();
                    }
                }
                Some(super::app::PopupKind::MirrorsCustomRepoUrl) => {
                    if !app.custom_input_buffer.trim().is_empty() {
                        app.draft_repo_url = app.custom_input_buffer.trim().to_string();
                        app.open_mirrors_custom_repo_sig();
                    }
                }
                Some(super::app::PopupKind::MirrorsCustomRepoSig) => {
                    if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                        app.draft_repo_sig_index = global_idx;
                        if app.draft_repo_sig_index == 0 {
                            app.finalize_custom_repo();
                        } else {
                            app.open_mirrors_custom_repo_signopt();
                        }
                    }
                }
                Some(super::app::PopupKind::MirrorsCustomRepoSignOpt) => {
                    if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                        app.draft_repo_signopt_index = global_idx;
                        app.finalize_custom_repo();
                    }
                }
                Some(super::app::PopupKind::DisksDeviceList) => {
                    // Save selection: first line is header, so selected index >=1
                    if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                        // In device list, index corresponds to a device row directly (header rendered separately)
                        if global_idx < app.popup_items.len() {
                            // parse path column from rendered row using the known format
                            if let Some(line) = app.popup_items.get(global_idx) {
                                // columns are fixed-width separated by " | ", path is column 2
                                let parts: Vec<&str> = line.split("|").map(|s| s.trim()).collect();
                                if parts.len() >= 2 {
                                    let path_col = parts[1];
                                    if !path_col.is_empty() {
                                        app.disks_selected_device = Some(path_col.to_string());
                                    }
                                }
                            }
                        }
                    }
                    app.close_popup();
                }
                Some(super::app::PopupKind::DiskEncryptionType) => {
                    if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                        app.disk_encryption_type_index = if global_idx == 0 { 0 } else { 1 };
                    }
                    app.close_popup();
                }
                Some(super::app::PopupKind::DiskEncryptionPassword) => {
                    app.disk_encryption_password = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    app.close_popup();
                }
                Some(super::app::PopupKind::DiskEncryptionPasswordConfirm) => {
                    app.disk_encryption_password_confirm = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    if app.disk_encryption_password != app.disk_encryption_password_confirm {
                        app.info_message = "Passwords do not match".into();
                    } else { app.info_message.clear(); }
                    app.close_popup();
                }
                Some(super::app::PopupKind::DiskEncryptionPartitionList) => {
                    if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                        if let Some(line) = app.popup_items.get(global_idx) {
                            let parts: Vec<&str> = line.split("|").map(|s| s.trim()).collect();
                            if parts.len() >= 2 {
                                let path_col = parts[1];
                                if !path_col.is_empty() { app.disk_encryption_selected_partition = Some(path_col.to_string()); }
                            }
                        }
                    }
                    app.close_popup();
                }
                Some(super::app::PopupKind::AbortConfirm) => {
                    if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                        if global_idx == 0 {
                            // Yes: exit without saving (explicit save is in Configuration)
                        }
                        app.close_popup();
                        return true; // exit app
                    }
                    app.close_popup();
                    return true;
                }
                _ => { app.apply_popup_selection(); }
            }
            return false;
        }
        // Custom text input editing (handled before search)
        KeyCode::Backspace if matches!(app.popup_kind,
            Some(super::app::PopupKind::MirrorsCustomServerInput) |
            Some(super::app::PopupKind::MirrorsCustomRepoName) |
            Some(super::app::PopupKind::MirrorsCustomRepoUrl) |
            Some(super::app::PopupKind::DiskEncryptionPassword) |
            Some(super::app::PopupKind::DiskEncryptionPasswordConfirm) |
            Some(super::app::PopupKind::HostnameInput) |
            Some(super::app::PopupKind::RootPassword) |
            Some(super::app::PopupKind::RootPasswordConfirm) |
            Some(super::app::PopupKind::UserAddUsername) |
            Some(super::app::PopupKind::UserAddPassword) |
            Some(super::app::PopupKind::UserAddPasswordConfirm)
        ) => {
            app.custom_input_buffer.pop();
            return false;
        }
        KeyCode::Char(c) if matches!(app.popup_kind,
            Some(super::app::PopupKind::MirrorsCustomServerInput) |
            Some(super::app::PopupKind::MirrorsCustomRepoName) |
            Some(super::app::PopupKind::MirrorsCustomRepoUrl) |
            Some(super::app::PopupKind::DiskEncryptionPassword) |
            Some(super::app::PopupKind::DiskEncryptionPasswordConfirm) |
            Some(super::app::PopupKind::HostnameInput) |
            Some(super::app::PopupKind::RootPassword) |
            Some(super::app::PopupKind::RootPasswordConfirm) |
            Some(super::app::PopupKind::UserAddUsername) |
            Some(super::app::PopupKind::UserAddPassword) |
            Some(super::app::PopupKind::UserAddPasswordConfirm)
        ) => {
            app.custom_input_buffer.push(c);
            return false;
        }
        // Search controls
        KeyCode::Char('/') => { app.popup_in_search = true; return false; }
        KeyCode::Backspace => { if app.popup_in_search { app.popup_search_query.pop(); app.filter_popup(); } return false; }
        KeyCode::Char(' ') => {
            // Toggle selection in multi-select popup
            match app.popup_kind {
                Some(super::app::PopupKind::MirrorsRegions) => {
                    if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                        if app.mirrors_regions_selected.contains(&global_idx) {
                            app.mirrors_regions_selected.remove(&global_idx);
                        } else {
                            app.mirrors_regions_selected.insert(global_idx);
                        }
                    }
                }
                Some(super::app::PopupKind::OptionalRepos) => {
                    if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                        if app.optional_repos_selected.contains(&global_idx) {
                            app.optional_repos_selected.remove(&global_idx);
                        } else {
                            app.optional_repos_selected.insert(global_idx);
                        }
                    }
                }
                _ => {}
            }
            return false;
        }
        KeyCode::Char(c) if app.popup_in_search => { app.popup_search_query.push(c); app.filter_popup(); return false; }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.popup_selected_visible == 0 {
                if !app.popup_visible_indices.is_empty() {
                    app.popup_selected_visible = app.popup_visible_indices.len() - 1;
                }
            } else { app.popup_selected_visible = app.popup_selected_visible.saturating_sub(1); }
            return false;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !app.popup_visible_indices.is_empty() {
                app.popup_selected_visible = (app.popup_selected_visible + 1) % app.popup_visible_indices.len();
            }
            return false;
        }
        _ => {}
    }
    false
}

fn move_disks_up(app: &mut AppState) {
    if app.current_screen() != Screen::Disks || app.focus != Focus::Content { return; }
    if app.disks_focus_index == 0 { app.disks_focus_index = 3; } else { app.disks_focus_index -= 1; }
}

fn move_disks_down(app: &mut AppState) {
    if app.current_screen() != Screen::Disks || app.focus != Focus::Content { return; }
    app.disks_focus_index = (app.disks_focus_index + 1) % 4;
}

fn change_disks_value(app: &mut AppState, next: bool) {
    if app.current_screen() != Screen::Disks || app.focus != Focus::Content { return; }
    if app.disks_focus_index <= 2 {
        // Cycle between the three modes
        let mut idx = app.disks_mode_index;
        if next { idx = (idx + 1) % 3; } else { idx = (idx + 3 - 1) % 3; }
        app.disks_mode_index = idx;
    }
}

fn move_diskenc_up(app: &mut AppState) {
    if app.current_screen() != Screen::DiskEncryption || app.focus != Focus::Content { return; }
    let max = if app.disk_encryption_type_index == 1 { 5 } else { 2 }; // fields + Continue
    if app.diskenc_focus_index == 0 { app.diskenc_focus_index = max - 1; } else { app.diskenc_focus_index -= 1; }
}

fn move_diskenc_down(app: &mut AppState) {
    if app.current_screen() != Screen::DiskEncryption || app.focus != Focus::Content { return; }
    let max = if app.disk_encryption_type_index == 1 { 5 } else { 2 };
    app.diskenc_focus_index = (app.diskenc_focus_index + 1) % max;
}

fn change_diskenc_value(app: &mut AppState, next: bool) {
    if app.current_screen() != Screen::DiskEncryption || app.focus != Focus::Content { return; }
    if app.diskenc_focus_index == 0 {
        // cycle encryption type
        let mut idx = app.disk_encryption_type_index;
        if next { idx = (idx + 1) % 2; } else { idx = (idx + 2 - 1) % 2; }
        app.disk_encryption_type_index = idx;
        app.save_config();
    }
}

fn move_swap_up(app: &mut AppState) {
    if app.current_screen() != Screen::SwapPartition || app.focus != Focus::Content { return; }
    if app.swap_focus_index == 0 { app.swap_focus_index = 1; } else { app.swap_focus_index -= 1; }
}

fn move_swap_down(app: &mut AppState) {
    if app.current_screen() != Screen::SwapPartition || app.focus != Focus::Content { return; }
    app.swap_focus_index = (app.swap_focus_index + 1) % 2; // toggle + Continue
}

fn change_swap_value(app: &mut AppState, _next: bool) {
    if app.current_screen() != Screen::SwapPartition || app.focus != Focus::Content { return; }
    if app.swap_focus_index == 0 { app.swap_enabled = !app.swap_enabled; }
}

fn move_bootloader_up(app: &mut AppState) {
    if app.current_screen() != Screen::Bootloader || app.focus != Focus::Content { return; }
    if app.bootloader_focus_index == 0 { app.bootloader_focus_index = 4; } else { app.bootloader_focus_index -= 1; }
}

fn move_bootloader_down(app: &mut AppState) {
    if app.current_screen() != Screen::Bootloader || app.focus != Focus::Content { return; }
    app.bootloader_focus_index = (app.bootloader_focus_index + 1) % 5; // 4 choices + Continue
}

fn change_bootloader_value(app: &mut AppState, next: bool) {
    if app.current_screen() != Screen::Bootloader || app.focus != Focus::Content { return; }
    if app.bootloader_focus_index < 4 {
        app.bootloader_index = app.bootloader_focus_index;
    }
}

fn move_hostname_up(app: &mut AppState) {
    if app.current_screen() != Screen::Hostname || app.focus != Focus::Content { return; }
    if app.hostname_focus_index == 0 { app.hostname_focus_index = 1; } else { app.hostname_focus_index -= 1; }
}

fn move_hostname_down(app: &mut AppState) {
    if app.current_screen() != Screen::Hostname || app.focus != Focus::Content { return; }
    app.hostname_focus_index = (app.hostname_focus_index + 1) % 2;
}

fn change_hostname_value(_app: &mut AppState, _next: bool) {}

fn move_rootpass_up(app: &mut AppState) {
    if app.current_screen() != Screen::RootPassword || app.focus != Focus::Content { return; }
    if app.rootpass_focus_index == 0 { app.rootpass_focus_index = 2; } else { app.rootpass_focus_index -= 1; }
}

fn move_rootpass_down(app: &mut AppState) {
    if app.current_screen() != Screen::RootPassword || app.focus != Focus::Content { return; }
    app.rootpass_focus_index = (app.rootpass_focus_index + 1) % 3; // set, confirm, Continue
}

fn change_rootpass_value(_app: &mut AppState, _next: bool) {}

fn move_user_up(app: &mut AppState) {
    if app.current_screen() != Screen::UserAccount || app.focus != Focus::Content { return; }
    if app.user_focus_index == 0 { app.user_focus_index = 1; } else { app.user_focus_index -= 1; }
}

fn move_user_down(app: &mut AppState) {
    if app.current_screen() != Screen::UserAccount || app.focus != Focus::Content { return; }
    app.user_focus_index = (app.user_focus_index + 1) % 2; // Add user + Continue
}

fn change_user_value(_app: &mut AppState, _next: bool) {}

fn move_config_up(app: &mut AppState) {
    if app.current_screen() != Screen::SaveConfiguration || app.focus != Focus::Content { return; }
    if app.config_focus_index == 0 { app.config_focus_index = 2; } else { app.config_focus_index -= 1; }
}

fn move_config_down(app: &mut AppState) {
    if app.current_screen() != Screen::SaveConfiguration || app.focus != Focus::Content { return; }
    app.config_focus_index = (app.config_focus_index + 1) % 3; // Save, Load, Continue
}

fn change_config_value(_app: &mut AppState, _next: bool) {
    // No inline cycling for configuration screen
}

fn handle_cmdline_keys(app: &mut AppState, code: KeyCode) -> bool {
    match code {
        KeyCode::Esc => { app.cmdline_open = false; app.cmdline_buffer.clear(); return false; }
        KeyCode::Enter => {
            let cmd_string = app.cmdline_buffer.clone();
            let cmd = cmd_string.trim().to_string();
            if cmd == "w" || cmd == "wq" {
                if app.current_screen() == Screen::Locales { app.apply_locales_edit(); }
                app.cmdline_open = false; app.cmdline_buffer.clear();
                if cmd == "wq" { app.focus = Focus::Menu; }
            } else if cmd == "q" {
                // Quit decision menu without saving
                if app.current_screen() == Screen::Locales { app.discard_locales_edit(); }
                app.cmdline_open = false; app.cmdline_buffer.clear();
                app.focus = Focus::Menu;
            } else {
                app.cmdline_open = false; app.cmdline_buffer.clear();
            }
            return false;
        }
        KeyCode::Backspace => { app.cmdline_buffer.pop(); return false; }
        KeyCode::Char(c) => { app.cmdline_buffer.push(c); return false; }
        _ => {}
    }
    false
}


