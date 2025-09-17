use crate::ui::app::{AppState, PopupKind};
use crossterm::event::KeyCode;

pub(crate) fn handle_popup_keys(app: &mut AppState, code: KeyCode) -> bool {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.close_popup();
            return false;
        }
        KeyCode::Enter => {
            match app.popup_kind {
                Some(PopupKind::Info) => {
                    app.close_popup();
                    // If the info was triggered by add-package validation, reopen input
                    if app.addpkgs_reopen_after_info {
                        app.addpkgs_reopen_after_info = false;
                        app.open_additional_package_input();
                    } else if app.hostname_reopen_after_info {
                        app.hostname_reopen_after_info = false;
                        app.open_hostname_input();
                    } else if app.username_reopen_after_info {
                        app.username_reopen_after_info = false;
                        app.open_user_username_input();
                    } else if app.useredit_reopen_after_info {
                        app.useredit_reopen_after_info = false;
                        app.popup_kind = Some(PopupKind::UserEditUsername);
                        app.popup_open = true;
                        app.popup_items.clear();
                        app.popup_visible_indices.clear();
                        app.popup_selected_visible = 0;
                        app.popup_in_search = false;
                        app.popup_search_query.clear();
                    } else if app.network_reopen_after_info_ip {
                        app.network_reopen_after_info_ip = false;
                        app.open_network_ip_input();
                    } else if app.network_reopen_after_info_gateway {
                        app.network_reopen_after_info_gateway = false;
                        app.open_network_gateway_input();
                    } else if app.network_reopen_after_info_dns {
                        app.network_reopen_after_info_dns = false;
                        app.open_network_dns_input();
                    } else if app.userpass_reopen_after_info {
                        app.userpass_reopen_after_info = false;
                        app.open_user_password_confirm_input();
                    } else if app.rootpass_reopen_after_info {
                        app.rootpass_reopen_after_info = false;
                        app.open_root_password_confirm_input();
                    } else if app.diskenc_reopen_after_info {
                        app.diskenc_reopen_after_info = false;
                        app.open_disk_encryption_password_confirm_input();
                    }
                    return false;
                }
                Some(PopupKind::KeyboardLayout)
                | Some(PopupKind::LocaleLanguage)
                | Some(PopupKind::LocaleEncoding) => {
                    app.apply_popup_selection();
                }
                Some(PopupKind::NetworkInterfaces) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(name) = app.popup_items.get(global_idx)
                    {
                        app.network_selected_interface = Some(name.clone());
                        app.close_popup();
                        app.open_network_mode_popup();
                    }
                }
                Some(PopupKind::NetworkMode) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        app.network_draft_mode_static = global_idx == 1; // 0 DHCP, 1 Static
                        app.close_popup();
                        if app.network_draft_mode_static {
                            app.open_network_ip_input();
                        } else if let Some(iface) = app.network_selected_interface.clone() {
                            app.network_configs.push(crate::ui::app::NetworkInterfaceConfig {
                                interface: iface,
                                mode: crate::ui::app::NetworkConfigMode::Dhcp,
                                ip_cidr: None,
                                gateway: None,
                                dns: None,
                            });
                            // Ensure DHCP client package is added for installation
                            let has_dhcp = app
                                .additional_packages
                                .iter()
                                .any(|p| p.name.eq_ignore_ascii_case("dhcp"));
                            if !has_dhcp {
                                if let Some((repo, pkg_name, version, description)) =
                                    app.validate_package("dhcp")
                                {
                                    app.additional_packages.push(
                                        crate::ui::app::AdditionalPackage {
                                            name: pkg_name,
                                            repo,
                                            version,
                                            description,
                                        },
                                    );
                                } else {
                                    app.additional_packages.push(
                                        crate::ui::app::AdditionalPackage {
                                            name: "dhcp".into(),
                                            repo: String::new(),
                                            version: String::new(),
                                            description: String::from("DHCP client"),
                                        },
                                    );
                                }
                            }
                            app.open_info_popup("Interface added with DHCP".into());
                        }
                    }
                }
                Some(PopupKind::NetworkIP) => {
                    let val = app.custom_input_buffer.trim().to_string();
                    app.custom_input_buffer.clear();
                    let valid = {
                        // Accept IPv4 1-255.1-255.1-255.1-255 with optional /0-32
                        let parts: Vec<&str> = val.split('/').collect();
                        let ip = parts.first().map(|s| s.trim()).unwrap_or("");
                        let cidr_ok = if parts.len() > 1 {
                            if let Ok(n) = parts[1].trim().parse::<u8>() {
                                n <= 32
                            } else {
                                false
                            }
                        } else {
                            true
                        };
                        let octets: Vec<&str> = ip.split('.').collect();
                        let ip_ok = octets.len() == 4
                            && octets.iter().all(|o| {
                                if o.is_empty() { return false; }
                                // forbid leading plus/minus, only digits
                                if !o.chars().all(|c| c.is_ascii_digit()) { return false; }
                                if let Ok(n) = o.parse::<u8>() {
                                    n >= 1
                                } else {
                                    false
                                }
                            });
                        !val.is_empty() && cidr_ok && ip_ok
                    };
                    if !valid {
                        app.network_reopen_after_info_ip = true;
                        app.open_info_popup(
                            "Invalid IP. Use IPv4 like 192.168.1.1 or 192.168.1.1/24".into(),
                        );
                    } else {
                        app.network_draft_ip_cidr = val;
                        app.close_popup();
                        app.open_network_gateway_input();
                    }
                }
                Some(PopupKind::NetworkGateway) => {
                    let val = app.custom_input_buffer.trim().to_string();
                    app.custom_input_buffer.clear();
                    let valid = if val.is_empty() {
                        true
                    } else {
                        let octets: Vec<&str> = val.split('.').collect();
                        octets.len() == 4
                            && octets.iter().all(|o| {
                                if o.is_empty() { return false; }
                                if !o.chars().all(|c| c.is_ascii_digit()) { return false; }
                                if let Ok(n) = o.parse::<u8>() { n >= 1 } else { false }
                            })
                    };
                    if !valid {
                        app.network_reopen_after_info_gateway = true;
                        app.open_info_popup(
                            "Invalid Gateway/Router. Use IPv4 like 192.168.1.1".into(),
                        );
                    } else {
                        app.network_draft_gateway = val;
                        app.close_popup();
                        app.open_network_dns_input();
                    }
                }
                Some(PopupKind::NetworkDNS) => {
                    let val = app.custom_input_buffer.trim().to_string();
                    app.custom_input_buffer.clear();
                    let valid = if val.is_empty() {
                        true
                    } else {
                        let octets: Vec<&str> = val.split('.').collect();
                        octets.len() == 4
                            && octets.iter().all(|o| {
                                if o.is_empty() { return false; }
                                if !o.chars().all(|c| c.is_ascii_digit()) { return false; }
                                if let Ok(n) = o.parse::<u8>() { n >= 1 } else { false }
                            })
                    };
                    if !valid {
                        app.network_reopen_after_info_dns = true;
                        app.open_info_popup("Invalid DNS. Use IPv4 like 1.1.1.1".into());
                    } else {
                        app.network_draft_dns = val;
                        app.close_popup();
                        if let Some(iface) = app.network_selected_interface.clone() {
                            app.network_configs.push(crate::ui::app::NetworkInterfaceConfig {
                                interface: iface,
                                mode: crate::ui::app::NetworkConfigMode::Static,
                                ip_cidr: Some(app.network_draft_ip_cidr.clone()),
                                gateway: if app.network_draft_gateway.is_empty() {
                                    None
                                } else {
                                    Some(app.network_draft_gateway.clone())
                                },
                                dns: if app.network_draft_dns.is_empty() {
                                    None
                                } else {
                                    Some(app.network_draft_dns.clone())
                                },
                            });
                            app.open_info_popup("Interface added with Static IP".into());
                        }
                    }
                }
                Some(PopupKind::UserSelectEdit) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        app.selected_user_index = global_idx;
                        app.close_popup();
                        // Open username editor prefilled
                        app.popup_kind = Some(PopupKind::UserEditUsername);
                        app.custom_input_buffer = app
                            .users
                            .get(app.selected_user_index)
                            .map(|u| u.username.clone())
                            .unwrap_or_default();
                        app.popup_open = true;
                        app.popup_items.clear();
                        app.popup_visible_indices.clear();
                        app.popup_selected_visible = 0;
                        app.popup_in_search = false;
                        app.popup_search_query.clear();
                    }
                }
                Some(PopupKind::UserSelectDelete) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        let username = app
                            .users
                            .get(global_idx)
                            .map(|u| u.username.clone())
                            .unwrap_or_default();
                        if global_idx < app.users.len() {
                            app.users.remove(global_idx);
                            app.info_message = format!("Deleted user '{}'.", username);
                        }
                        app.close_popup();
                    }
                }
                Some(PopupKind::UserEditUsername) => {
                    let value = app.custom_input_buffer.trim().to_string();
                    if !value.is_empty()
                        && !crate::ui::app::AppState::is_ascii_lowercase_only(&value)
                    {
                        app.useredit_reopen_after_info = true;
                        app.open_info_popup(
                            "Username must contain only lowercase ASCII characters".into(),
                        );
                    } else if app.selected_user_index < app.users.len() {
                        if let Some(u) = app.users.get_mut(app.selected_user_index) {
                            u.username = value;
                        }
                        app.custom_input_buffer.clear();
                        app.close_popup();
                    } else {
                        app.close_popup();
                    }
                }
                Some(PopupKind::KernelSelect) => {
                    // Ensure at least one selection; default to linux if none
                    if app.selected_kernels.is_empty() {
                        app.selected_kernels.insert("linux".into());
                    }
                    app.close_popup();
                }
                Some(PopupKind::TimezoneSelect) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(name) = app.popup_items.get(global_idx)
                    {
                        app.timezone_value = name.clone();
                    }
                    app.close_popup();
                }
                Some(PopupKind::HostnameInput) => {
                    let value = app.custom_input_buffer.trim().to_string();
                    if !value.is_empty() && !crate::ui::app::AppState::is_ascii_only(&value) {
                        app.hostname_reopen_after_info = true;
                        app.open_info_popup("Hostname must contain only ASCII characters".into());
                    } else {
                        app.hostname_value = value;
                        app.custom_input_buffer.clear();
                        app.close_popup();
                    }
                }
                Some(PopupKind::AdditionalPackageInput) => {
                    let name = app.custom_input_buffer.trim().to_string();
                    if !name.is_empty() {
                        if let Some((repo, pkg_name, version, description)) =
                            app.validate_package(&name)
                        {
                            // Check conflicts with existing selections
                            if let Some(reason) = app.check_additional_pkg_conflicts(&pkg_name) {
                                app.addpkgs_reopen_after_info = true;
                                app.open_info_popup(format!(
                                    "Package '{}' not added: {}.",
                                    pkg_name, reason
                                ));
                                app.custom_input_buffer.clear();
                                return false;
                            }
                            app.additional_packages
                                .push(crate::ui::app::AdditionalPackage {
                                    name: pkg_name,
                                    repo,
                                    version,
                                    description,
                                });
                            app.info_message.clear();
                        } else {
                            // Show info popup but remember to return to input afterwards
                            app.addpkgs_reopen_after_info = true;
                            app.open_info_popup("Package does not exist".into());
                        }
                    }
                    // keep popup open for next entry
                    app.custom_input_buffer.clear();
                }
                Some(PopupKind::MirrorsRegions) => {
                    app.close_popup();
                }
                Some(PopupKind::RootPassword) => {
                    app.root_password = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    app.close_popup();
                }
                Some(PopupKind::RootPasswordConfirm) => {
                    app.root_password_confirm = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    if app.root_password != app.root_password_confirm {
                        app.rootpass_reopen_after_info = true;
                        app.open_info_popup("Root passwords do not match".into());
                    } else {
                        app.info_message.clear();
                        app.close_popup();
                    }
                }
                Some(PopupKind::UserAddUsername) => {
                    let value = app.custom_input_buffer.trim().to_string();
                    if !value.is_empty()
                        && !crate::ui::app::AppState::is_ascii_lowercase_only(&value)
                    {
                        app.username_reopen_after_info = true;
                        app.open_info_popup(
                            "Username must contain only lowercase ASCII characters".into(),
                        );
                    } else {
                        app.draft_user_username = value;
                        app.custom_input_buffer.clear();
                        app.close_popup();
                        if !app.draft_user_username.is_empty() {
                            app.open_user_password_input();
                        }
                    }
                }
                Some(PopupKind::UserAddPassword) => {
                    app.draft_user_password = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    app.close_popup();
                    app.open_user_password_confirm_input();
                }
                Some(PopupKind::UserAddPasswordConfirm) => {
                    app.draft_user_password_confirm = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    if app.draft_user_password != app.draft_user_password_confirm {
                        app.userpass_reopen_after_info = true;
                        app.open_info_popup("User passwords do not match".into());
                    } else {
                        app.info_message.clear();
                        app.close_popup();
                        app.open_user_sudo_select();
                    }
                }
                Some(PopupKind::UserAddSudo) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        app.draft_user_is_sudo = global_idx == 0; // Yes
                        app.users.push(crate::ui::app::UserAccount {
                            username: app.draft_user_username.clone(),
                            password: app.draft_user_password.clone(),
                            password_hash: None,
                            is_sudo: app.draft_user_is_sudo,
                        });
                    }
                    app.close_popup();
                }
                Some(PopupKind::OptionalRepos) => {
                    app.close_popup();
                }
                Some(PopupKind::MirrorsCustomServerInput) => {
                    if !app.custom_input_buffer.trim().is_empty() {
                        app.mirrors_custom_servers
                            .push(app.custom_input_buffer.trim().to_string());
                    }
                    app.custom_input_buffer.clear();
                }
                Some(PopupKind::MirrorsCustomRepoName) => {
                    if !app.custom_input_buffer.trim().is_empty() {
                        app.draft_repo_name = app.custom_input_buffer.trim().to_string();
                        app.open_mirrors_custom_repo_url();
                    }
                }
                Some(PopupKind::MirrorsCustomRepoUrl) => {
                    if !app.custom_input_buffer.trim().is_empty() {
                        app.draft_repo_url = app.custom_input_buffer.trim().to_string();
                        app.open_mirrors_custom_repo_sig();
                    }
                }
                Some(PopupKind::MirrorsCustomRepoSig) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        app.draft_repo_sig_index = global_idx;
                        if app.draft_repo_sig_index == 0 {
                            app.finalize_custom_repo();
                        } else {
                            app.open_mirrors_custom_repo_signopt();
                        }
                    }
                }
                Some(PopupKind::MirrorsCustomRepoSignOpt) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        app.draft_repo_signopt_index = global_idx;
                        app.finalize_custom_repo();
                    }
                }
                Some(PopupKind::DisksDeviceList) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(line) = app.popup_items.get(global_idx)
                    {
                        let parts: Vec<&str> = line.split("|").map(|s| s.trim()).collect();
                        if parts.len() >= 7 {
                            let model_col = parts[0];
                            let path_col = parts[1];
                            let devtype_col = parts[2];
                            let size_col = parts[3];
                            let freespace_col = parts[4];
                            let sector_size_col = parts[5];
                            let read_only_col = parts[6];
                            if !path_col.is_empty() {
                                app.disks_selected_device = Some(path_col.to_string());
                                app.disks_selected_device_model = Some(model_col.to_string());
                                app.disks_selected_device_devtype = Some(devtype_col.to_string());
                                app.disks_selected_device_size = Some(size_col.to_string());
                                app.disks_selected_device_freespace = Some(freespace_col.to_string());
                                app.disks_selected_device_sector_size = Some(sector_size_col.to_string());
                                app.disks_selected_device_read_only = Some(read_only_col.eq_ignore_ascii_case("true"));
                            }
                        }
                    }
                    app.close_popup();
                }
                Some(PopupKind::DiskEncryptionType) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        app.disk_encryption_type_index = if global_idx == 0 { 0 } else { 1 };
                    }
                    app.close_popup();
                }
                Some(PopupKind::DiskEncryptionPassword) => {
                    app.disk_encryption_password = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    app.close_popup();
                }
                Some(PopupKind::DiskEncryptionPasswordConfirm) => {
                    app.disk_encryption_password_confirm = app.custom_input_buffer.clone();
                    app.custom_input_buffer.clear();
                    if app.disk_encryption_password != app.disk_encryption_password_confirm {
                        app.diskenc_reopen_after_info = true;
                        app.open_info_popup("Disk encryption passwords do not match".into());
                    } else {
                        app.info_message.clear();
                        app.close_popup();
                    }
                }
                Some(PopupKind::DiskEncryptionPartitionList) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(line) = app.popup_items.get(global_idx)
                    {
                        let parts: Vec<&str> = line.split("|").map(|s| s.trim()).collect();
                        if parts.len() >= 2 {
                            let path_col = parts[1];
                            if !path_col.is_empty() {
                                app.disk_encryption_selected_partition = Some(path_col.to_string());
                            }
                        }
                    }
                    app.close_popup();
                }
                Some(PopupKind::AbortConfirm) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        if global_idx == 0 {
                            // Yes: save then exit
                            app.save_config();
                        }
                        app.close_popup();
                        return true;
                    }
                    app.close_popup();
                    return true;
                }
                Some(PopupKind::MinimalClearConfirm) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && global_idx == 0
                    {
                        // Yes: set Minimal and clear selections
                        app.experience_mode_index = 1;
                        app.selected_desktop_envs.clear();
                        app.selected_env_packages.clear();
                        app.selected_login_manager = None;
                        app.login_manager_user_set = false;
                    }
                    app.close_popup();
                }
                Some(PopupKind::DesktopEnvSelect) => {
                    // For DE/WM popup, Enter just closes; selection is toggled with Space
                    app.close_popup();
                }
                Some(PopupKind::ServerTypeSelect) => {
                    // For Server popup, Enter just closes; selection is toggled with Space
                    app.close_popup();
                }
                Some(PopupKind::XorgTypeSelect) => {
                    // For Xorg popup, Enter just closes; selection is toggled with Space
                    app.close_popup();
                }
                None => app.apply_popup_selection(),
            }
            return false;
        }
        // Custom text input editing (handled before search)
        KeyCode::Backspace
            if matches!(
                app.popup_kind,
                Some(PopupKind::MirrorsCustomServerInput)
                    | Some(PopupKind::MirrorsCustomRepoName)
                    | Some(PopupKind::MirrorsCustomRepoUrl)
                    | Some(PopupKind::NetworkIP)
                    | Some(PopupKind::NetworkGateway)
                    | Some(PopupKind::NetworkDNS)
                    | Some(PopupKind::DiskEncryptionPassword)
                    | Some(PopupKind::DiskEncryptionPasswordConfirm)
                    | Some(PopupKind::HostnameInput)
                    | Some(PopupKind::AdditionalPackageInput)
                    | Some(PopupKind::RootPassword)
                    | Some(PopupKind::RootPasswordConfirm)
                    | Some(PopupKind::UserAddUsername)
                    | Some(PopupKind::UserEditUsername)
                    | Some(PopupKind::UserAddPassword)
                    | Some(PopupKind::UserAddPasswordConfirm)
            ) =>
        {
            app.custom_input_buffer.pop();
            return false;
        }
        KeyCode::Char(c)
            if matches!(
                app.popup_kind,
                Some(PopupKind::MirrorsCustomServerInput)
                    | Some(PopupKind::MirrorsCustomRepoName)
                    | Some(PopupKind::MirrorsCustomRepoUrl)
                    | Some(PopupKind::NetworkIP)
                    | Some(PopupKind::NetworkGateway)
                    | Some(PopupKind::NetworkDNS)
                    | Some(PopupKind::DiskEncryptionPassword)
                    | Some(PopupKind::DiskEncryptionPasswordConfirm)
                    | Some(PopupKind::HostnameInput)
                    | Some(PopupKind::AdditionalPackageInput)
                    | Some(PopupKind::RootPassword)
                    | Some(PopupKind::RootPasswordConfirm)
                    | Some(PopupKind::UserAddUsername)
                    | Some(PopupKind::UserEditUsername)
                    | Some(PopupKind::UserAddPassword)
                    | Some(PopupKind::UserAddPasswordConfirm)
            ) =>
        {
            app.custom_input_buffer.push(c);
            return false;
        }
        // Search controls
        KeyCode::Char('/') => {
            app.popup_in_search = true;
            return false;
        }
        KeyCode::Backspace => {
            if app.popup_in_search {
                app.popup_search_query.pop();
                app.filter_popup();
            }
            return false;
        }
        KeyCode::Char(' ') => {
            // Toggle selection in multi-select popup
            match app.popup_kind {
                Some(PopupKind::MirrorsRegions) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        if app.mirrors_regions_selected.contains(&global_idx) {
                            app.mirrors_regions_selected.remove(&global_idx);
                        } else {
                            app.mirrors_regions_selected.insert(global_idx);
                        }
                    }
                }
                Some(PopupKind::OptionalRepos) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                    {
                        if app.optional_repos_selected.contains(&global_idx) {
                            app.optional_repos_selected.remove(&global_idx);
                        } else {
                            app.optional_repos_selected.insert(global_idx);
                        }
                    }
                }
                Some(PopupKind::KernelSelect) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(name) = app.popup_items.get(global_idx)
                    {
                        if app.selected_kernels.contains(name) {
                            app.selected_kernels.remove(name);
                        } else {
                            app.selected_kernels.insert(name.clone());
                        }
                    }
                    return false;
                }
                Some(PopupKind::DesktopEnvSelect) => {
                    // If right-pane (packages) focused: toggle package for current env
                    if app.popup_packages_focus {
                        if let Some(&global_idx) =
                            app.popup_visible_indices.get(app.popup_selected_visible)
                            && let Some(env_name) = app.popup_items.get(global_idx)
                        {
                            let packages: Vec<&str> = match env_name.as_str() {
                                "Awesome" => vec![
                                    "alacritty",
                                    "awesome",
                                    "feh",
                                    "gnu-free-fonts",
                                    "slock",
                                    "terminus-font",
                                    "ttf-liberation",
                                    "xorg-server",
                                    "xorg-xinit",
                                    "xorg-xrandr",
                                    "xsel",
                                    "xterm",
                                    "htop",
                                    "iwd",
                                    "nano",
                                    "openssh",
                                    "smartmontools",
                                    "vim",
                                    "wget",
                                    "wireless_tools",
                                    "wpa_supplicant",
                                    "xdg-utils",
                                ],
                                "Bspwm" => vec![
                                    "bspwm",
                                    "dmenu",
                                    "rxvt-unicode",
                                    "sxhkd",
                                    "xdo",
                                    "htop",
                                    "iwd",
                                    "nano",
                                    "openssh",
                                    "smartmontools",
                                    "vim",
                                    "wget",
                                    "wireless_tools",
                                    "wpa_supplicant",
                                    "xdg-utils",
                                ],
                                "Budgie" => vec![
                                    "arc-gtk-theme",
                                    "budgie",
                                    "mate-terminal",
                                    "nemo",
                                    "papirus-icon-theme",
                                    "htop",
                                    "iwd",
                                    "nano",
                                    "openssh",
                                    "smartmontools",
                                    "vim",
                                    "wget",
                                    "wireless_tools",
                                    "wpa_supplicant",
                                    "xdg-utils",
                                ],
                                "Cinnamon" => vec![
                                    "blueman",
                                    "blue-utils",
                                    "cinnamon",
                                    "engrampa",
                                    "gnome-keyring",
                                    "gnome-screenshot",
                                    "gnome-terminal",
                                    "gvfs-smb",
                                    "system-config-printer",
                                    "xdg-user-dirs-gtk",
                                    "xed",
                                    "htop",
                                    "iwd",
                                    "nano",
                                    "openssh",
                                    "smartmontools",
                                    "vim",
                                    "wget",
                                    "wireless_tools",
                                    "wpa_supplicant",
                                    "xdg-utils",
                                ],
                                "Cutefish" => vec![
                                    "cutefish",
                                    "noto-fonts",
                                    "htop",
                                    "iwd",
                                    "nano",
                                    "openssh",
                                    "smartmontools",
                                    "vim",
                                    "wget",
                                    "wireless_tools",
                                    "wpa_supplicant",
                                    "xdg-utils",
                                ],
                                "Deepin" => vec![
                                    "deepin",
                                    "deepin-editor",
                                    "deepin-terminal",
                                    "htop",
                                    "iwd",
                                    "nano",
                                    "openssh",
                                    "smartmontools",
                                    "vim",
                                    "wget",
                                    "wireless_tools",
                                    "wpa_supplicant",
                                    "xdg-utils",
                                ],
                                "Enlightenment" => vec![
                                    "enlightenment",
                                    "terminology",
                                    "htop",
                                    "iwd",
                                    "nano",
                                    "openssh",
                                    "smartmontools",
                                    "vim",
                                    "wget",
                                    "wireless_tools",
                                    "wpa_supplicant",
                                    "xdg-utils",
                                ],
                                "GNOME" => vec![
                                    "gnome",
                                    "gnome-tweaks",
                                    "htop",
                                    "iwd",
                                    "nano",
                                    "openssh",
                                    "smartmontools",
                                    "vim",
                                    "wget",
                                    "wireless_tools",
                                    "wpa_supplicant",
                                    "xdg-utils",
                                ],
                                "Hyprland" => vec![
                                    "dolphin",
                                    "dunst",
                                    "grim",
                                    "hyprland",
                                    "kitty",
                                    "polkit-kde-agent",
                                    "qt5-wayland",
                                    "qt6-wayland",
                                    "slurp",
                                    "polkit",
                                    "wofi",
                                    "xdg-desktop-portal-hyprland",
                                    "htop",
                                    "iwd",
                                    "nano",
                                    "openssh",
                                    "smartmontools",
                                    "vim",
                                    "wget",
                                    "wireless_tools",
                                    "wpa_supplicant",
                                    "xdg-utils",
                                ],
                                "KDE Plasma" => vec![
                                    "ark",
                                    "dolphin",
                                    "kate",
                                    "konsole",
                                    "plasma-meta",
                                    "plasma-workspace",
                                    "htop",
                                    "iwd",
                                    "nano",
                                    "openssh",
                                    "smartmontools",
                                    "vim",
                                    "wget",
                                    "wireless_tools",
                                    "wpa_supplicant",
                                    "xdg-utils",
                                ],
                                "Lxqt" => vec![
                                    "breeze-icons",
                                    "leafpad",
                                    "oxygen-icons",
                                    "slock",
                                    "ttf-freefont",
                                    "xdg-utils",
                                    "htop",
                                    "iwd",
                                    "nano",
                                    "openssh",
                                    "smartmontools",
                                    "vim",
                                    "wget",
                                    "wireless_tools",
                                    "wpa_supplicant",
                                ],
                                "Mate" => vec![
                                    "mate",
                                    "mate-extra",
                                    "htop",
                                    "iwd",
                                    "nano",
                                    "openssh",
                                    "smartmontools",
                                    "vim",
                                    "wget",
                                    "wireless_tools",
                                    "wpa_supplicant",
                                    "xdg-utils",
                                ],
                                "Qtile" => vec![
                                    "alacritty",
                                    "qtile",
                                    "htop",
                                    "iwd",
                                    "nano",
                                    "openssh",
                                    "smartmontools",
                                    "vim",
                                    "wget",
                                    "wireless_tools",
                                    "wpa_supplicant",
                                    "xdg-utils",
                                ],
                                "Sway" => vec![
                                    "brightnessctl",
                                    "dmenu",
                                    "foot",
                                    "grim",
                                    "pavucontrol",
                                    "slurp",
                                    "sway",
                                    "swaybg",
                                    "swayidle",
                                    "swaylock",
                                    "waybar",
                                    "xorg-xwayland",
                                ],
                                "Xfce4" => vec![
                                    "gvfs",
                                    "pavucontrol",
                                    "xarchiver",
                                    "xfce4",
                                    "xfce4-goodies",
                                ],
                                "i3-wm" => vec![
                                    "dmenu",
                                    "i3-wm",
                                    "i3blocks",
                                    "i3lock",
                                    "i3status",
                                    "lightdm",
                                    "lightdm-gtk-greeter",
                                    "xss-lock",
                                    "xterm",
                                ],
                                _ => vec![],
                            };
                            if !packages.is_empty() {
                                let set = app
                                    .selected_env_packages
                                    .entry(env_name.clone())
                                    .or_insert_with(|| {
                                        packages.iter().map(|s| s.to_string()).collect()
                                    });
                                if let Some(pkg_name) =
                                    packages.get(app.popup_packages_selected_index)
                                {
                                    if set.contains(*pkg_name) {
                                        set.remove(*pkg_name);
                                    } else {
                                        set.insert((*pkg_name).to_string());
                                    }
                                }
                            }
                        }
                    } else if app.popup_login_focus {
                        if let Some(&global_idx) =
                            app.popup_visible_indices.get(app.popup_selected_visible)
                            && app.popup_items.get(global_idx).is_some()
                        {
                            // Available login managers list
                            let login_managers = [
                                "none",
                                "gdm",
                                "lightdm-gtk-greeter",
                                "lightdm-slick-greeter",
                                "ly",
                                "sddm",
                            ];
                            let choice = login_managers
                                .get(app.popup_login_selected_index)
                                .unwrap_or(&"none");
                            let value = if *choice == "none" {
                                None
                            } else {
                                Some(choice.to_string())
                            };
                            app.selected_login_manager = value;
                            app.login_manager_user_set = true;
                        }
                    } else if app.popup_drivers_focus {
                        // Toggle Graphic Drivers selection
                        let drivers_all: Vec<(&str, bool)> = vec![
                            (" Open Source Drivers ", false),
                            ("intel-media-driver", true),
                            ("libva-intel-driver", true),
                            ("mesa", true),
                            ("vulkan-intel", true),
                            ("vulkan-nouveau", true),
                            ("vulkan-radeon", true),
                            ("xf86-video-amdgpu", true),
                            ("xf86-video-ati", true),
                            ("xf86-video-nouveau", true),
                            ("xf86-video-vmware", true),
                            ("xorg-server", true),
                            ("xorg-xinit", true),
                            (" Nvidia Drivers ", false),
                            ("dkms", true),
                            ("libva-nvidia-driver", true),
                            (" Choose one ", false),
                            ("nvidia-open-dkms", true),
                            ("nvidia-dkms", true),
                        ];
                        let idx = app
                            .popup_drivers_selected_index
                            .min(drivers_all.len().saturating_sub(1));
                        if let Some((name, selectable)) = drivers_all.get(idx) {
                            if *selectable {
                                let key = name.to_string();
                                if key == "nvidia-open-dkms" {
                                    if app.selected_graphic_drivers.contains("nvidia-open-dkms") {
                                        app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                    } else {
                                        app.selected_graphic_drivers.remove("nvidia-dkms");
                                        app.selected_graphic_drivers.insert(key);
                                        // Ensure Nvidia base drivers are selected
                                        app.selected_graphic_drivers.insert("dkms".into());
                                        app.selected_graphic_drivers
                                            .insert("libva-nvidia-driver".into());
                                        // Remove conflicting nouveau pieces
                                        app.selected_graphic_drivers.remove("xf86-video-nouveau");
                                        app.selected_graphic_drivers.remove("vulkan-nouveau");
                                    }
                                } else if key == "nvidia-dkms" {
                                    if app.selected_graphic_drivers.contains("nvidia-dkms") {
                                        app.selected_graphic_drivers.remove("nvidia-dkms");
                                    } else {
                                        app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                        app.selected_graphic_drivers.insert(key);
                                        // Ensure Nvidia base drivers are selected
                                        app.selected_graphic_drivers.insert("dkms".into());
                                        app.selected_graphic_drivers
                                            .insert("libva-nvidia-driver".into());
                                        // Remove conflicting nouveau pieces
                                        app.selected_graphic_drivers.remove("xf86-video-nouveau");
                                        app.selected_graphic_drivers.remove("vulkan-nouveau");
                                    }
                                } else if app.selected_graphic_drivers.contains(&key) {
                                    app.selected_graphic_drivers.remove(&key);
                                } else {
                                    app.selected_graphic_drivers.insert(key.clone());
                                    if key == "xf86-video-nouveau" || key == "vulkan-nouveau" {
                                        app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                        app.selected_graphic_drivers.remove("nvidia-dkms");
                                        app.selected_graphic_drivers.remove("dkms");
                                        app.selected_graphic_drivers.remove("libva-nvidia-driver");
                                    }
                                }
                            } else {
                                let title = name.trim();
                                if title == "Open Source Drivers" {
                                    let oss = vec![
                                        "intel-media-driver",
                                        "libva-intel-driver",
                                        "libva-mesa-driver",
                                        "mesa",
                                        "vulkan-intel",
                                        "vulkan-nouveau",
                                        "vulkan-radeon",
                                        "xf86-video-amdgpu",
                                        "xf86-video-ati",
                                        "xf86-video-nouveau",
                                        "xf86-video-vmware",
                                        "xorg-server",
                                        "xorg-xinit",
                                    ];
                                    let all_selected = oss
                                        .iter()
                                        .all(|k| app.selected_graphic_drivers.contains(*k));
                                    if all_selected {
                                        for k in oss {
                                            app.selected_graphic_drivers.remove(k);
                                        }
                                    } else {
                                        for k in oss {
                                            app.selected_graphic_drivers.insert(k.to_string());
                                        }
                                        // Remove all Nvidia selections when enabling OSS block
                                        app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                        app.selected_graphic_drivers.remove("nvidia-dkms");
                                        app.selected_graphic_drivers.remove("dkms");
                                        app.selected_graphic_drivers.remove("libva-nvidia-driver");
                                    }
                                } else if title == "Nvidia Drivers" {
                                    let need = vec!["dkms", "libva-nvidia-driver"];
                                    let all_selected = need
                                        .iter()
                                        .all(|k| app.selected_graphic_drivers.contains(*k));
                                    if all_selected {
                                        for k in need {
                                            app.selected_graphic_drivers.remove(k);
                                        }
                                        // also clear choose-one if prerequisites cleared
                                        app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                        app.selected_graphic_drivers.remove("nvidia-dkms");
                                    } else {
                                        for k in need {
                                            app.selected_graphic_drivers.insert(k.to_string());
                                        }
                                    }
                                } else if title == "Choose One" {
                                    let any_selected =
                                        app.selected_graphic_drivers.contains("nvidia-open-dkms")
                                            || app.selected_graphic_drivers.contains("nvidia-dkms");
                                    if any_selected {
                                        app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                        app.selected_graphic_drivers.remove("nvidia-dkms");
                                    } else {
                                        // default to open driver, ensure prerequisites
                                        app.selected_graphic_drivers
                                            .insert("nvidia-open-dkms".into());
                                        app.selected_graphic_drivers.insert("dkms".into());
                                        app.selected_graphic_drivers
                                            .insert("libva-nvidia-driver".into());
                                    }
                                }
                            }
                        }
                        return false;
                    } else if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(name) = app.popup_items.get(global_idx)
                    {
                        // Left-pane: toggle environment selection
                        if app.selected_desktop_envs.contains(name) {
                            app.selected_desktop_envs.remove(name);
                            // If user hasn't manually set LM, recompute from remaining selection
                            if !app.login_manager_user_set {
                                if app.selected_desktop_envs.is_empty() {
                                    app.selected_login_manager = None;
                                } else if let Some(first_env) =
                                    app.selected_desktop_envs.iter().next().cloned()
                                {
                                    let default_lm: Option<&str> = match first_env.as_str() {
                                        "GNOME" => Some("gdm"),
                                        "KDE Plasma" | "Hyprland" | "Cutefish" | "Lxqt" => {
                                            Some("sddm")
                                        }
                                        "Budgie" => Some("lightdm-slick-greeter"),
                                        "Bspwm" | "Cinnamon" | "Deepin" | "Enlightenment"
                                        | "Mate" | "Qtile" | "Sway" | "Xfce4" | "i3-wm" => {
                                            Some("lightdm-gtk-greeter")
                                        }
                                        _ => None,
                                    };
                                    app.selected_login_manager = default_lm.map(|s| s.to_string());
                                }
                            }
                        } else {
                            app.selected_desktop_envs.insert(name.clone());
                            // On first environment selection, set global login manager if not user-set
                            if !app.login_manager_user_set && app.selected_login_manager.is_none() {
                                // compute default for this env
                                let default_lm: Option<&str> = match name.as_str() {
                                    "GNOME" => Some("gdm"),
                                    "KDE Plasma" | "Hyprland" | "Cutefish" | "Lxqt" => Some("sddm"),
                                    "Budgie" => Some("lightdm-slick-greeter"),
                                    "Bspwm" | "Cinnamon" | "Deepin" | "Enlightenment" | "Mate"
                                    | "Qtile" | "Sway" | "Xfce4" | "i3-wm" => {
                                        Some("lightdm-gtk-greeter")
                                    }
                                    _ => None,
                                };
                                app.selected_login_manager = default_lm.map(|s| s.to_string());
                            }
                        }
                    }
                }
                Some(PopupKind::ServerTypeSelect) => {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(name) = app.popup_items.get(global_idx)
                    {
                        // Defaults for this server type
                        let defaults: Vec<&str> = match name.as_str() {
                            "Cockpit" => vec!["cockpit", "packagekit", "udisk2"],
                            "Docker" => vec!["docker"],
                            "Lighttpd" => vec!["lighttpd"],
                            "Mariadb" => vec!["mariadb"],
                            "Nginx" => vec!["nginx"],
                            "Postgresql" => vec!["postgresql"],
                            "Tomcat" => vec!["tomcat10"],
                            "httpd" => vec!["apache"],
                            "sshd" => vec!["openssh"],
                            _ => Vec::new(),
                        };
                        if app.popup_packages_focus {
                            // Toggle package selection in right pane
                            let set = app
                                .selected_server_packages
                                .entry(name.clone())
                                .or_insert_with(|| {
                                    defaults.iter().map(|s| s.to_string()).collect()
                                });
                            let list: Vec<String> =
                                defaults.iter().map(|s| s.to_string()).collect();
                            if !list.is_empty() {
                                let idx = app
                                    .popup_packages_selected_index
                                    .min(list.len().saturating_sub(1));
                                if let Some(pkg) = list.get(idx) {
                                    if set.contains(pkg) {
                                        set.remove(pkg);
                                    } else {
                                        set.insert(pkg.clone());
                                    }
                                }
                            }
                        } else {
                            // Toggle server type in left pane
                            if app.selected_server_types.contains(name) {
                                app.selected_server_types.remove(name);
                                app.selected_server_packages.remove(name);
                            } else {
                                app.selected_server_types.insert(name.clone());
                                app.selected_server_packages.insert(
                                    name.clone(),
                                    defaults.iter().map(|s| s.to_string()).collect(),
                                );
                            }
                        }
                    }
                    return false;
                }
                Some(PopupKind::XorgTypeSelect) => {
                    if app.popup_drivers_focus {
                        // Toggle Graphic Drivers selection in Xorg popup
                        let drivers_all: Vec<(&str, bool)> = vec![
                            (" Open Source Drivers ", false),
                            ("intel-media-driver", true),
                            ("libva-intel-driver", true),
                            ("mesa", true),
                            ("vulkan-intel", true),
                            ("vulkan-nouveau", true),
                            ("vulkan-radeon", true),
                            ("xf86-video-amdgpu", true),
                            ("xf86-video-ati", true),
                            ("xf86-video-nouveau", true),
                            ("xf86-video-vmware", true),
                            ("xorg-server", true),
                            ("xorg-xinit", true),
                            (" Nvidia Drivers ", false),
                            ("dkms", true),
                            ("libva-nvidia-driver", true),
                            (" Choose one ", false),
                            ("nvidia-open-dkms", true),
                            ("nvidia-dkms", true),
                        ];
                        let idx = app
                            .popup_drivers_selected_index
                            .min(drivers_all.len().saturating_sub(1));
                        if let Some((name, selectable)) = drivers_all.get(idx) {
                            if *selectable {
                                let key = name.to_string();
                                if key == "nvidia-open-dkms" {
                                    if app.selected_graphic_drivers.contains("nvidia-open-dkms") {
                                        app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                    } else {
                                        app.selected_graphic_drivers.remove("nvidia-dkms");
                                        app.selected_graphic_drivers.insert(key);
                                        // Ensure Nvidia base drivers are selected
                                        app.selected_graphic_drivers.insert("dkms".into());
                                        app.selected_graphic_drivers
                                            .insert("libva-nvidia-driver".into());
                                        // Remove conflicting nouveau pieces
                                        app.selected_graphic_drivers.remove("xf86-video-nouveau");
                                        app.selected_graphic_drivers.remove("vulkan-nouveau");
                                    }
                                } else if key == "nvidia-dkms" {
                                    if app.selected_graphic_drivers.contains("nvidia-dkms") {
                                        app.selected_graphic_drivers.remove("nvidia-dkms");
                                    } else {
                                        app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                        app.selected_graphic_drivers.insert(key);
                                        // Ensure Nvidia base drivers are selected
                                        app.selected_graphic_drivers.insert("dkms".into());
                                        app.selected_graphic_drivers
                                            .insert("libva-nvidia-driver".into());
                                        // Remove conflicting nouveau pieces
                                        app.selected_graphic_drivers.remove("xf86-video-nouveau");
                                        app.selected_graphic_drivers.remove("vulkan-nouveau");
                                    }
                                } else if app.selected_graphic_drivers.contains(&key) {
                                    app.selected_graphic_drivers.remove(&key);
                                } else {
                                    app.selected_graphic_drivers.insert(key.clone());
                                    if key == "xf86-video-nouveau" || key == "vulkan-nouveau" {
                                        app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                        app.selected_graphic_drivers.remove("nvidia-dkms");
                                        app.selected_graphic_drivers.remove("dkms");
                                        app.selected_graphic_drivers.remove("libva-nvidia-driver");
                                    }
                                }
                            } else {
                                let title = name.trim();
                                if title == "Open Source Drivers" {
                                    let oss = vec![
                                        "intel-media-driver",
                                        "libva-intel-driver",
                                        "libva-mesa-driver",
                                        "mesa",
                                        "vulkan-intel",
                                        "vulkan-nouveau",
                                        "vulkan-radeon",
                                        "xf86-video-amdgpu",
                                        "xf86-video-ati",
                                        "xf86-video-nouveau",
                                        "xf86-video-vmware",
                                        "xorg-server",
                                        "xorg-xinit",
                                    ];
                                    let all_selected = oss
                                        .iter()
                                        .all(|k| app.selected_graphic_drivers.contains(*k));
                                    if all_selected {
                                        for k in oss {
                                            app.selected_graphic_drivers.remove(k);
                                        }
                                    } else {
                                        for k in oss {
                                            app.selected_graphic_drivers.insert(k.to_string());
                                        }
                                        // Remove all Nvidia selections when enabling OSS block
                                        app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                        app.selected_graphic_drivers.remove("nvidia-dkms");
                                        app.selected_graphic_drivers.remove("dkms");
                                        app.selected_graphic_drivers.remove("libva-nvidia-driver");
                                    }
                                } else if title == "Nvidia Drivers" {
                                    let need = vec!["dkms", "libva-nvidia-driver"];
                                    let all_selected = need
                                        .iter()
                                        .all(|k| app.selected_graphic_drivers.contains(*k));
                                    if all_selected {
                                        for k in need {
                                            app.selected_graphic_drivers.remove(k);
                                        }
                                        // also clear choose-one if prerequisites cleared
                                        app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                        app.selected_graphic_drivers.remove("nvidia-dkms");
                                    } else {
                                        for k in need {
                                            app.selected_graphic_drivers.insert(k.to_string());
                                        }
                                    }
                                } else if title == "Choose One" {
                                    let any_selected =
                                        app.selected_graphic_drivers.contains("nvidia-open-dkms")
                                            || app.selected_graphic_drivers.contains("nvidia-dkms");
                                    if any_selected {
                                        app.selected_graphic_drivers.remove("nvidia-open-dkms");
                                        app.selected_graphic_drivers.remove("nvidia-dkms");
                                    } else {
                                        // default to open driver, ensure prerequisites
                                        app.selected_graphic_drivers
                                            .insert("nvidia-open-dkms".into());
                                        app.selected_graphic_drivers.insert("dkms".into());
                                        app.selected_graphic_drivers
                                            .insert("libva-nvidia-driver".into());
                                    }
                                }
                            }
                        }
                        return false;
                    } else if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(name) = app.popup_items.get(global_idx)
                    {
                        // Defaults for Xorg type
                        let defaults: Vec<&str> = match name.as_str() {
                            "Xorg" => vec!["xorg-server"],
                            _ => Vec::new(),
                        };
                        if app.popup_packages_focus {
                            // Toggle package selection in right pane
                            let set = app
                                .selected_xorg_packages
                                .entry(name.clone())
                                .or_insert_with(|| {
                                    defaults.iter().map(|s| s.to_string()).collect()
                                });
                            let list: Vec<String> =
                                defaults.iter().map(|s| s.to_string()).collect();
                            if !list.is_empty() {
                                let idx = app
                                    .popup_packages_selected_index
                                    .min(list.len().saturating_sub(1));
                                if let Some(pkg) = list.get(idx) {
                                    if set.contains(pkg) {
                                        set.remove(pkg);
                                    } else {
                                        set.insert(pkg.clone());
                                    }
                                }
                            }
                        } else {
                            // Toggle Xorg type in left pane
                            if app.selected_xorg_types.contains(name) {
                                app.selected_xorg_types.remove(name);
                                app.selected_xorg_packages.remove(name);
                            } else {
                                app.selected_xorg_types.insert(name.clone());
                                app.selected_xorg_packages.insert(
                                    name.clone(),
                                    defaults.iter().map(|s| s.to_string()).collect(),
                                );
                            }
                        }
                    }
                    return false;
                }
                _ => {}
            }
            return false;
        }
        KeyCode::Char(c) if app.popup_in_search => {
            app.popup_search_query.push(c);
            app.filter_popup();
            return false;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if matches!(app.popup_kind, Some(PopupKind::DesktopEnvSelect)) {
                if app.popup_login_focus {
                    let len = 6; // none, gdm, lightdm-gtk-greeter, lightdm-slick-greeter, ly, sddm
                    app.popup_login_selected_index =
                        (app.popup_login_selected_index + len - 1) % len;
                } else if app.popup_drivers_focus {
                    // Drivers list length (including headers)
                    let drv_len = {
                        let drivers_all: Vec<(&str, bool)> = vec![
                            (" Open Source Drivers ", false),
                            ("intel-media-driver", true),
                            ("libva-intel-driver", true),
                            ("mesa", true),
                            ("vulkan-intel", true),
                            ("vulkan-nouveau", true),
                            ("vulkan-radeon", true),
                            ("xf86-video-amdgpu", true),
                            ("xf86-video-ati", true),
                            ("xf86-video-nouveau", true),
                            ("xf86-video-vmware", true),
                            ("xorg-server", true),
                            ("xorg-xinit", true),
                            (" Nvidia Drivers ", false),
                            ("dkms", true),
                            ("libva-nvidia-driver", true),
                            (" Choose one ", false),
                            ("nvidia-open-dkms", true),
                            ("nvidia-dkms", true),
                        ];
                        drivers_all.len()
                    };
                    if drv_len > 0 {
                        app.popup_drivers_selected_index =
                            (app.popup_drivers_selected_index + drv_len - 1) % drv_len;
                    }
                } else if app.popup_packages_focus {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(env_name) = app.popup_items.get(global_idx)
                    {
                        let packages: Vec<&str> = match env_name.as_str() {
                            "Awesome" => vec![
                                "alacritty",
                                "awesome",
                                "feh",
                                "gnu-free-fonts",
                                "slock",
                                "terminus-font",
                                "ttf-liberation",
                                "xorg-server",
                                "xorg-xinit",
                                "xorg-xrandr",
                                "xsel",
                                "xterm",
                            ],
                            "Bspwm" => vec!["bspwm", "dmenu", "rxvt-unicode", "sxhkd", "xdo"],
                            "Budgie" => vec![
                                "arc-gtk-theme",
                                "budgie",
                                "mate-terminal",
                                "nemo",
                                "papirus-icon-theme",
                            ],
                            "Cinnamon" => vec![
                                "blueman",
                                "blue-utils",
                                "cinnamon",
                                "engrampa",
                                "gnome-keyring",
                                "gnome-screenshot",
                                "gnome-terminal",
                                "gvfs-smb",
                                "system-config-printer",
                                "xdg-user-dirs-gtk",
                                "xed",
                            ],
                            "Cutefish" => vec!["cutefish", "noto-fonts"],
                            "Deepin" => vec!["deepin", "deepin-editor", "deepin-terminal"],
                            "Enlightenment" => vec!["enlightenment", "terminology"],
                            "GNOME" => vec!["gnome", "gnome-tweaks"],
                            "Hyprland" => vec![
                                "dolphin",
                                "dunst",
                                "grim",
                                "hyprland",
                                "kitty",
                                "polkit-kde-agent",
                                "qt5-wayland",
                                "qt6-wayland",
                                "slurp",
                                "wofi",
                                "xdg-desktop-portal-hyprland",
                            ],
                            "KDE Plasma" => vec![
                                "ark",
                                "dolphin",
                                "kate",
                                "konsole",
                                "plasma-meta",
                                "plasma-workspace",
                            ],
                            "Lxqt" => vec![
                                "breeze-icons",
                                "leafpad",
                                "oxygen-icons",
                                "slock",
                                "ttf-freefont",
                                "xdg-utils",
                            ],
                            "Mate" => vec!["mate", "mate-extra"],
                            "Qtile" => vec!["alacritty", "qtile"],
                            "Sway" => vec![
                                "brightnessctl",
                                "dmenu",
                                "foot",
                                "grim",
                                "pavucontrol",
                                "slurp",
                                "sway",
                                "swaybg",
                                "swayidle",
                                "swaylock",
                                "polkit",
                                "waybar",
                                "xorg-xwayland",
                                "htop",
                                "iwd",
                                "nano",
                                "openssh",
                                "smartmontools",
                                "vim",
                                "wget",
                                "wireless_tools",
                                "wpa_supplicant",
                                "xdg-utils",
                            ],
                            "Xfce4" => vec![
                                "gvfs",
                                "pavucontrol",
                                "xarchiver",
                                "xfce4",
                                "xfce4-goodies",
                                "htop",
                                "iwd",
                                "nano",
                                "openssh",
                                "smartmontools",
                                "vim",
                                "wget",
                                "wireless_tools",
                                "wpa_supplicant",
                                "xdg-utils",
                            ],
                            "i3-wm" => vec![
                                "dmenu",
                                "i3-wm",
                                "i3blocks",
                                "i3lock",
                                "i3status",
                                "lightdm",
                                "lightdm-gtk-greeter",
                                "xss-lock",
                                "xterm",
                                "htop",
                                "iwd",
                                "nano",
                                "openssh",
                                "smartmontools",
                                "vim",
                                "wget",
                                "wireless_tools",
                                "wpa_supplicant",
                                "xdg-utils",
                            ],
                            _ => vec![],
                        };
                        let len = packages.len();
                        if len > 0 {
                            app.popup_packages_selected_index =
                                (app.popup_packages_selected_index + len - 1) % len;
                        }
                    }
                } else {
                    super::screens::popup_move_up(app);
                }
            } else if matches!(app.popup_kind, Some(PopupKind::ServerTypeSelect)) {
                if app.popup_packages_focus {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(_server) = app.popup_items.get(global_idx)
                    {
                        // Compute current server's package defaults length for wrapping
                        let selected_label = _server.as_str();
                        let pkgs_len = match selected_label {
                            "Cockpit" => 3,
                            "Docker" => 1,
                            "Lighttpd" => 1,
                            "Mariadb" => 1,
                            "Nginx" => 1,
                            "Postgresql" => 1,
                            "Tomcat" => 1,
                            "httpd" => 1,
                            "sshd" => 1,
                            _ => 0,
                        };
                        if pkgs_len > 0 {
                            app.popup_packages_selected_index =
                                (app.popup_packages_selected_index + pkgs_len - 1) % pkgs_len;
                        }
                    }
                } else {
                    super::screens::popup_move_up(app);
                }
            } else if matches!(app.popup_kind, Some(PopupKind::XorgTypeSelect)) {
                if app.popup_drivers_focus {
                    let drv_len = {
                        let drivers_all: Vec<(&str, bool)> = vec![
                            (" Open Source Drivers ", false),
                            ("intel-media-driver", true),
                            ("libva-intel-driver", true),
                            ("mesa", true),
                            ("vulkan-intel", true),
                            ("vulkan-nouveau", true),
                            ("vulkan-radeon", true),
                            ("xf86-video-amdgpu", true),
                            ("xf86-video-ati", true),
                            ("xf86-video-nouveau", true),
                            ("xf86-video-vmware", true),
                            ("xorg-server", true),
                            ("xorg-xinit", true),
                            (" Nvidia Drivers ", false),
                            ("dkms", true),
                            ("libva-nvidia-driver", true),
                            (" Choose one ", false),
                            ("nvidia-open-dkms", true),
                            ("nvidia-dkms", true),
                        ];
                        drivers_all.len()
                    };
                    if drv_len > 0 {
                        app.popup_drivers_selected_index =
                            (app.popup_drivers_selected_index + drv_len - 1) % drv_len;
                    }
                } else if app.popup_packages_focus {
                    // Only one package currently; wrap safely
                    let pkgs_len = 1;
                    if pkgs_len > 0 {
                        app.popup_packages_selected_index =
                            (app.popup_packages_selected_index + pkgs_len - 1) % pkgs_len;
                    }
                } else {
                    super::screens::popup_move_up(app);
                }
            } else {
                super::screens::popup_move_up(app);
            }
            return false;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if matches!(app.popup_kind, Some(PopupKind::DesktopEnvSelect)) {
                if app.popup_login_focus {
                    let len = 6;
                    app.popup_login_selected_index = (app.popup_login_selected_index + 1) % len;
                } else if app.popup_drivers_focus {
                    let drv_len = {
                        let drivers_all: Vec<(&str, bool)> = vec![
                            (" Open Source Drivers ", false),
                            ("intel-media-driver", true),
                            ("libva-intel-driver", true),
                            ("libva-mesa-driver", true),
                            ("mesa", true),
                            ("vulkan-intel", true),
                            ("vulkan-nouveau", true),
                            ("vulkan-radeon", true),
                            ("xf86-video-amdgpu", true),
                            ("xf86-video-ati", true),
                            ("xf86-video-nouveau", true),
                            ("xf86-video-vmware", true),
                            ("xorg-server", true),
                            ("xorg-xinit", true),
                            (" Nvidia Drivers ", false),
                            ("dkms", true),
                            ("libva-nvidia-driver", true),
                            (" Choose one ", false),
                            ("nvidia-open-dkms", true),
                            ("nvidia-dkms", true),
                        ];
                        drivers_all.len()
                    };
                    if drv_len > 0 {
                        app.popup_drivers_selected_index =
                            (app.popup_drivers_selected_index + 1) % drv_len;
                    }
                } else if app.popup_packages_focus {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(env_name) = app.popup_items.get(global_idx)
                    {
                        let packages: Vec<&str> = match env_name.as_str() {
                            "Awesome" => vec![
                                "alacritty",
                                "awesome",
                                "feh",
                                "gnu-free-fonts",
                                "slock",
                                "terminus-font",
                                "ttf-liberation",
                                "xorg-server",
                                "xorg-xinit",
                                "xorg-xrandr",
                                "xsel",
                                "xterm",
                            ],
                            "Bspwm" => vec!["bspwm", "dmenu", "rxvt-unicode", "sxhkd", "xdo"],
                            "Budgie" => vec![
                                "arc-gtk-theme",
                                "budgie",
                                "mate-terminal",
                                "nemo",
                                "papirus-icon-theme",
                            ],
                            "Cinnamon" => vec![
                                "blueman",
                                "blue-utils",
                                "cinnamon",
                                "engrampa",
                                "gnome-keyring",
                                "gnome-screenshot",
                                "gnome-terminal",
                                "gvfs-smb",
                                "system-config-printer",
                                "xdg-user-dirs-gtk",
                                "xed",
                            ],
                            "Cutefish" => vec!["cutefish", "noto-fonts"],
                            "Deepin" => vec!["deepin", "deepin-editor", "deepin-terminal"],
                            "Enlightenment" => vec!["enlightenment", "terminology"],
                            "GNOME" => vec!["gnome", "gnome-tweaks"],
                            "Hyprland" => vec![
                                "dolphin",
                                "dunst",
                                "grim",
                                "hyprland",
                                "kitty",
                                "polkit-kde-agent",
                                "qt5-wayland",
                                "qt6-wayland",
                                "slurp",
                                "wofi",
                                "xdg-desktop-portal-hyprland",
                            ],
                            "KDE Plasma" => vec![
                                "ark",
                                "dolphin",
                                "kate",
                                "konsole",
                                "plasma-meta",
                                "plasma-workspace",
                            ],
                            "Lxqt" => vec![
                                "breeze-icons",
                                "leafpad",
                                "oxygen-icons",
                                "slock",
                                "ttf-freefont",
                                "xdg-utils",
                            ],
                            "Mate" => vec!["mate", "mate-extra"],
                            "Qtile" => vec!["alacritty", "qtile"],
                            "Sway" => vec![
                                "brightnessctl",
                                "dmenu",
                                "foot",
                                "grim",
                                "pavucontrol",
                                "slurp",
                                "sway",
                                "swaybg",
                                "swayidle",
                                "swaylock",
                                "waybar",
                                "xorg-xwayland",
                            ],
                            "Xfce4" => {
                                vec!["gvfs", "pavucontrol", "xarchiver", "xfce4", "xfce4-goodies"]
                            }
                            "i3-wm" => vec![
                                "dmenu",
                                "i3-wm",
                                "i3blocks",
                                "i3lock",
                                "i3status",
                                "lightdm",
                                "lightdm-gtk-greeter",
                                "xss-lock",
                                "xterm",
                            ],
                            _ => vec![],
                        };
                        let len = packages.len();
                        if len > 0 {
                            app.popup_packages_selected_index =
                                (app.popup_packages_selected_index + 1) % len;
                        }
                    }
                } else {
                    super::screens::popup_move_down(app);
                }
            } else if matches!(app.popup_kind, Some(PopupKind::ServerTypeSelect)) {
                if app.popup_packages_focus {
                    if let Some(&global_idx) =
                        app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(_server) = app.popup_items.get(global_idx)
                    {
                        let selected_label = _server.as_str();
                        let pkgs_len = match selected_label {
                            "Cockpit" => 3,
                            "Docker" => 1,
                            "Lighttpd" => 1,
                            "Mariadb" => 1,
                            "Nginx" => 1,
                            "Postgresql" => 1,
                            "Tomcat" => 1,
                            "httpd" => 1,
                            "sshd" => 1,
                            _ => 0,
                        };
                        if pkgs_len > 0 {
                            app.popup_packages_selected_index =
                                (app.popup_packages_selected_index + 1) % pkgs_len;
                        }
                    }
                } else {
                    super::screens::popup_move_down(app);
                }
            } else if matches!(app.popup_kind, Some(PopupKind::XorgTypeSelect)) {
                if app.popup_drivers_focus {
                    let drv_len = {
                        let drivers_all: Vec<(&str, bool)> = vec![
                            ("intel-media-driver", true),
                            ("libva-intel-driver", true),
                            ("libva-mesa-driver", true),
                            ("mesa", true),
                            ("vulkan-intel", true),
                            ("vulkan-nouveau", true),
                            ("vulkan-radeon", true),
                            ("xf86-video-amdgpu", true),
                            ("xf86-video-ati", true),
                            ("xf86-video-nouveau", true),
                            ("xf86-video-vmware", true),
                            ("xorg-server", true),
                            ("xorg-xinit", true),
                            (" Nvidia Drivers ", false),
                            ("dkms", true),
                            ("libva-nvidia-driver", true),
                            (" Choose one ", false),
                            ("nvidia-open-dkms", true),
                            ("nvidia-dkms", true),
                        ];
                        drivers_all.len()
                    };
                    if drv_len > 0 {
                        app.popup_drivers_selected_index =
                            (app.popup_drivers_selected_index + 1) % drv_len;
                    }
                } else if app.popup_packages_focus {
                    let pkgs_len = 1;
                    if pkgs_len > 0 {
                        app.popup_packages_selected_index =
                            (app.popup_packages_selected_index + 1) % pkgs_len;
                    }
                } else {
                    super::screens::popup_move_down(app);
                }
            } else {
                super::screens::popup_move_down(app);
            }
            return false;
        }
        KeyCode::Left | KeyCode::Char('h')
            if matches!(app.popup_kind, Some(PopupKind::ServerTypeSelect)) =>
        {
            if app.popup_packages_focus {
                app.popup_packages_focus = false;
            }
            return false;
        }
        KeyCode::Right | KeyCode::Char('l')
            if matches!(app.popup_kind, Some(PopupKind::ServerTypeSelect)) =>
        {
            if !app.popup_packages_focus {
                app.popup_packages_focus = true;
                app.popup_packages_selected_index = 0;
            }
            return false;
        }
        // Xorg left/right focus switching: left list -> packages -> drivers -> right stays
        KeyCode::Left | KeyCode::Char('h')
            if matches!(app.popup_kind, Some(PopupKind::XorgTypeSelect)) =>
        {
            if app.popup_drivers_focus {
                app.popup_drivers_focus = false;
                app.popup_packages_focus = true;
            } else if app.popup_packages_focus {
                app.popup_packages_focus = false;
            }
            return false;
        }
        KeyCode::Right | KeyCode::Char('l')
            if matches!(app.popup_kind, Some(PopupKind::XorgTypeSelect)) =>
        {
            if !app.popup_packages_focus && !app.popup_drivers_focus {
                app.popup_packages_focus = true;
                app.popup_packages_selected_index = 0;
            } else if app.popup_packages_focus {
                app.popup_packages_focus = false;
                app.popup_drivers_focus = true;
                app.popup_drivers_selected_index = 0;
            } else if app.popup_drivers_focus {
                // stay on drivers
            }
            return false;
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if matches!(app.popup_kind, Some(PopupKind::DesktopEnvSelect)) {
                if app.popup_login_focus {
                    app.popup_login_focus = false;
                    app.popup_drivers_focus = true;
                } else if app.popup_drivers_focus {
                    app.popup_drivers_focus = false;
                    app.popup_packages_focus = true;
                } else if app.popup_packages_focus {
                    app.popup_packages_focus = false;
                }
                return false;
            }
            return false;
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if matches!(app.popup_kind, Some(PopupKind::DesktopEnvSelect)) {
                if !app.popup_packages_focus && !app.popup_drivers_focus && !app.popup_login_focus {
                    // move into packages first
                    app.popup_packages_focus = true;
                    app.popup_packages_selected_index = 0;
                } else if app.popup_packages_focus {
                    app.popup_packages_focus = false;
                    app.popup_drivers_focus = true;
                    app.popup_drivers_selected_index = 0;
                } else if app.popup_drivers_focus {
                    app.popup_drivers_focus = false;
                    app.popup_login_focus = true;
                    app.popup_login_selected_index = 0;
                } else if app.popup_login_focus {
                    // stay on login
                }
                return false;
            }
            return false;
        }
        _ => {}
    }
    false
}
