use crate::app::{AppState, PopupKind};

pub(crate) fn handle_enter(app: &mut AppState) -> bool {
    match app.popup_kind {
        Some(PopupKind::Info) => {
            app.close_popup();
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
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible)
                && let Some(name) = app.popup_items.get(global_idx)
            {
                app.network_selected_interface = Some(name.clone());
                app.close_popup();
                app.open_network_mode_popup();
            }
        }
        Some(PopupKind::NetworkMode) => {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                app.network_draft_mode_static = global_idx == 1; // 0 DHCP, 1 Static
                app.close_popup();
                if app.network_draft_mode_static {
                    app.open_network_ip_input();
                } else if let Some(iface) = app.network_selected_interface.clone() {
                    app.network_configs
                        .push(crate::app::NetworkInterfaceConfig {
                            interface: iface,
                            mode: crate::app::NetworkConfigMode::Dhcp,
                            ip_cidr: None,
                            gateway: None,
                            dns: None,
                        });
                    let has_dhcp = app
                        .additional_packages
                        .iter()
                        .any(|p| p.name.eq_ignore_ascii_case("dhcp"));
                    if !has_dhcp {
                        if let Some((repo, pkg_name, version, description)) =
                            app.validate_package("dhcp")
                        {
                            app.additional_packages
                                .push(crate::app::AdditionalPackage {
                                    name: pkg_name,
                                    repo,
                                    version,
                                    description,
                                });
                        } else {
                            app.additional_packages
                                .push(crate::app::AdditionalPackage {
                                    name: "dhcp".into(),
                                    repo: String::new(),
                                    version: String::new(),
                                    description: String::from("DHCP client"),
                                });
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
                        if o.is_empty() {
                            return false;
                        }
                        if !o.chars().all(|c| c.is_ascii_digit()) {
                            return false;
                        }
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
                        if o.is_empty() {
                            return false;
                        }
                        if !o.chars().all(|c| c.is_ascii_digit()) {
                            return false;
                        }
                        if let Ok(n) = o.parse::<u8>() {
                            n >= 1
                        } else {
                            false
                        }
                    })
            };
            if !valid {
                app.network_reopen_after_info_gateway = true;
                app.open_info_popup("Invalid Gateway/Router. Use IPv4 like 192.168.1.1".into());
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
                        if o.is_empty() {
                            return false;
                        }
                        if !o.chars().all(|c| c.is_ascii_digit()) {
                            return false;
                        }
                        if let Ok(n) = o.parse::<u8>() {
                            n >= 1
                        } else {
                            false
                        }
                    })
            };
            if !valid {
                app.network_reopen_after_info_dns = true;
                app.open_info_popup("Invalid DNS. Use IPv4 like 1.1.1.1".into());
            } else {
                app.network_draft_dns = val;
                app.close_popup();
                if let Some(iface) = app.network_selected_interface.clone() {
                    app.network_configs
                        .push(crate::app::NetworkInterfaceConfig {
                            interface: iface,
                            mode: crate::app::NetworkConfigMode::Static,
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
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                app.selected_user_index = global_idx;
                app.close_popup();
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
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
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
            if !value.is_empty() && !crate::app::AppState::is_ascii_lowercase_only(&value) {
                app.useredit_reopen_after_info = true;
                app.open_info_popup("Username must contain only lowercase ASCII characters".into());
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
            if app.selected_kernels.is_empty() {
                app.selected_kernels.insert("linux".into());
            }
            app.close_popup();
        }
        Some(PopupKind::TimezoneSelect) => {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible)
                && let Some(name) = app.popup_items.get(global_idx)
            {
                app.timezone_value = name.clone();
            }
            app.close_popup();
        }
        Some(PopupKind::HostnameInput) => {
            let value = app.custom_input_buffer.trim().to_string();
            if !value.is_empty() && !crate::app::AppState::is_ascii_only(&value) {
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
                if let Some((repo, pkg_name, version, description)) = app.validate_package(&name) {
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
                        .push(crate::app::AdditionalPackage {
                            name: pkg_name,
                            repo,
                            version,
                            description,
                        });
                    app.info_message.clear();
                } else {
                    app.addpkgs_reopen_after_info = true;
                    app.open_info_popup("Package does not exist".into());
                }
            }
            app.custom_input_buffer.clear();
        }
        Some(PopupKind::AdditionalPackageGroupSelect) => {
            if app.popup_packages_focus {
                // Continue or Confirm depending on last group
                let is_last_group = if let Some(&gi) = app.popup_visible_indices.get(app.popup_selected_visible) {
                    gi + 1 == app.popup_items.len()
                } else { false };
                if is_last_group {
                    // Move set into accumulated and apply once
                    app.addpkgs_group_accum_selected
                        .extend(app.addpkgs_group_pkg_selected.iter().cloned());
                    app.addpkgs_group_pkg_selected.clear();
                    // Apply accumulated
                    let to_apply: Vec<String> = app
                        .addpkgs_group_accum_selected
                        .iter()
                        .cloned()
                        .collect();
                    app.addpkgs_group_pkg_selected = to_apply.iter().cloned().collect();
                    app.apply_additional_package_group_selection(true);
                    app.addpkgs_group_accum_selected.clear();
                    // Close after confirm
                    app.close_popup();
                } else {
                    // Accumulate current selections and move to next group
                    app.addpkgs_group_accum_selected
                        .extend(app.addpkgs_group_pkg_selected.iter().cloned());
                    // Apply immediately so Info box reflects intermediate selections
                    let to_apply: Vec<String> = app
                        .addpkgs_group_pkg_selected
                        .iter()
                        .cloned()
                        .collect();
                    app.addpkgs_group_pkg_selected = to_apply.iter().cloned().collect();
                    app.apply_additional_package_group_selection(false);
                    if let Some(&gi) = app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(_name) = app.popup_items.get(gi)
                    {
                        // Move selection to next visible index
                        let next_vis = (app.popup_selected_visible + 1)
                            % app.popup_visible_indices.len();
                        app.popup_selected_visible = next_vis;
                        app.addpkgs_group_pkg_selected.clear();
                        app.addpkgs_group_pkg_index = 0;
                    }
                }
            } else {
                // Enter from left pane goes into the right pane selection
                app.popup_packages_focus = true;
                app.addpkgs_group_pkg_index = 0;
            }
        }
        Some(PopupKind::AdditionalPackageGroupPackages) => {}
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
            if !value.is_empty() && !crate::app::AppState::is_ascii_lowercase_only(&value) {
                app.username_reopen_after_info = true;
                app.open_info_popup("Username must contain only lowercase ASCII characters".into());
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
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                app.draft_user_is_sudo = global_idx == 0;
                app.users.push(crate::app::UserAccount {
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
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                app.draft_repo_sig_index = global_idx;
                if app.draft_repo_sig_index == 0 {
                    app.finalize_custom_repo();
                } else {
                    app.open_mirrors_custom_repo_signopt();
                }
            }
        }
        Some(PopupKind::MirrorsCustomRepoSignOpt) => {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                app.draft_repo_signopt_index = global_idx;
                app.finalize_custom_repo();
            }
        }
        Some(PopupKind::DisksDeviceList) => {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible)
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
                        app.disks_selected_device_read_only =
                            Some(read_only_col.eq_ignore_ascii_case("true"));
                    }
                }
            }
            app.close_popup();
        }
        Some(PopupKind::DiskEncryptionType) => {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
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
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible)
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
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                if global_idx == 0 {
                    app.save_config();
                }
                app.close_popup();
                return true;
            }
            app.close_popup();
            return true;
        }
        Some(PopupKind::MinimalClearConfirm) => {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                if global_idx == 0 {
                    app.experience_mode_index = 1;
                    app.selected_desktop_envs.clear();
                    app.selected_env_packages.clear();
                    app.selected_login_manager = None;
                    app.login_manager_user_set = false;
                }
                app.close_popup();
            }
        }
        Some(PopupKind::WipeConfirm) => {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                let yes = global_idx == 0;
                app.close_popup();
                app.pending_wipe_confirm = Some(yes);
            }
        }
        Some(PopupKind::DesktopEnvSelect)
        | Some(PopupKind::ServerTypeSelect)
        | Some(PopupKind::XorgTypeSelect) => {
            app.close_popup();
        }
        None => app.apply_popup_selection(),
    }
    false
}
