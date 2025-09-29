use crate::app::{AppState, PopupKind};

fn finalize_manual_partition(app: &mut AppState) {
    // Compute size in bytes from stored selection (preferred) or current input
    let computed_size = {
        let qty = app.custom_input_buffer.trim().parse::<u64>().unwrap_or(0);
        let unit_multiplier: u64 = match app.manual_create_units_index {
            0 => 1,
            1 => 1024,               // KiB/KB
            2 => 1024 * 1024,        // MiB/MB
            3 => 1024 * 1024 * 1024, // GiB/GB
            _ => 1024 * 1024 * 1024,
        };
        qty.saturating_mul(unit_multiplier)
    };
    let size_bytes = if app.manual_create_selected_size_bytes != 0 {
        app.manual_create_selected_size_bytes
    } else {
        computed_size
    };
    let start_b = app.manual_create_free_start_bytes;
    let end_b = app.manual_create_free_end_bytes;
    let max_bytes = end_b.saturating_sub(start_b);
    let final_size = size_bytes.min(max_bytes);

    // Map kind index back to role string (strip any (created) suffix)
    let role = match app.manual_create_kind_index {
        0 => "BOOT",
        1 => "SWAP",
        2 => "ROOT",
        3 => "OTHER",
        _ => "OTHER",
    };

    // Filesystem selection
    let fs = if app.manual_create_kind_index == 1 {
        Some("linux-swap".to_string())
    } else {
        app.manual_create_fs_options
            .get(app.manual_create_fs_index)
            .cloned()
    };

    // Mountpoint
    let mountpoint = if app.manual_create_kind_index == 1 {
        None
    } else {
        let mp = if app.manual_create_kind_index == 0 {
            "/boot"
        } else if app.manual_create_kind_index == 2 {
            "/"
        } else {
            app.manual_create_mountpoint.as_str()
        };
        Some(mp.to_string())
    };

    // Save partition spec (update if editing, else append)
    let spec = crate::core::types::DiskPartitionSpec {
        name: app.disks_selected_device.clone(),
        role: Some(role.to_string()),
        fs,
        start: Some(format!("{start_b}")),
        size: Some(format!("{final_size}")),
        mountpoint,
        ..Default::default()
    };
    if let Some(edit_idx) = app.manual_edit_index.take() {
        if let Some(existing) = app.disks_partitions.get_mut(edit_idx) {
            *existing = spec;
        } else {
            app.disks_partitions.push(spec);
        }
    } else {
        app.disks_partitions.push(spec);
    }

    // After creation, reset selection and go back to the ManualPartitionTable for the selected device
    app.manual_create_selected_size_bytes = 0;
    app.open_manual_partition_table_for_selected();
}

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
        Some(PopupKind::OptionalRepos) => {
            // Only close; do not open AUR helper popup on Enter
            app.close_popup();
        }
        Some(PopupKind::AurHelperSelect) => {
            if let Some(&gi) = app.popup_visible_indices.get(app.popup_selected_visible) {
                app.aur_helper_index = Some(gi);
            }
            app.close_popup();
            // Return to Optional repositories
            app.open_optional_repos_popup();
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
                            app.additional_packages.push(crate::app::AdditionalPackage {
                                name: pkg_name,
                                repo,
                                version,
                                description,
                            });
                        } else {
                            app.additional_packages.push(crate::app::AdditionalPackage {
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
                    app.info_message = format!("Deleted user '{username}'.");
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
                            "Package '{pkg_name}' not added: {reason}."
                        ));
                        app.custom_input_buffer.clear();
                        return false;
                    }
                    app.additional_packages.push(crate::app::AdditionalPackage {
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
                let is_last_group =
                    if let Some(&gi) = app.popup_visible_indices.get(app.popup_selected_visible) {
                        gi + 1 == app.popup_items.len()
                    } else {
                        false
                    };
                if is_last_group {
                    // Move set into accumulated and apply once
                    app.addpkgs_group_accum_selected
                        .extend(app.addpkgs_group_pkg_selected.iter().cloned());
                    app.addpkgs_group_pkg_selected.clear();
                    // Apply accumulated
                    let to_apply: Vec<String> =
                        app.addpkgs_group_accum_selected.iter().cloned().collect();
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
                    let to_apply: Vec<String> =
                        app.addpkgs_group_pkg_selected.iter().cloned().collect();
                    app.addpkgs_group_pkg_selected = to_apply.iter().cloned().collect();
                    app.apply_additional_package_group_selection(false);
                    if let Some(&gi) = app.popup_visible_indices.get(app.popup_selected_visible)
                        && let Some(_name) = app.popup_items.get(gi)
                    {
                        // Move selection to next visible index
                        let next_vis =
                            (app.popup_selected_visible + 1) % app.popup_visible_indices.len();
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
            app.open_root_password_confirm_input();
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
            // If Manual mode is active, open the follow-up manual partition table
            let manual_mode = app.disks_mode_index == 1;
            app.close_popup();
            if manual_mode {
                app.open_manual_partition_table_for_selected();
            }
        }
        Some(PopupKind::ManualPartitionTable) => {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible)
                && let Some(line) = app.popup_items.get(global_idx)
            {
                let _cols: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
                if global_idx < app.manual_partition_row_meta.len() {
                    let m = &app.manual_partition_row_meta[global_idx];
                    if m.kind.eq_ignore_ascii_case("free") {
                        let used_b = m.free_start.unwrap_or(0);
                        let total_b = m.free_end.unwrap_or(0);
                        app.close_popup();
                        app.open_manual_partition_create(used_b, total_b);
                        return false;
                    } else if m.kind.eq_ignore_ascii_case("created") {
                        let idx = m.spec_index.unwrap_or(0);
                        app.close_popup();
                        app.popup_kind = Some(PopupKind::ManualPartitionEdit);
                        app.popup_items = vec!["Modify".into(), "Delete".into()];
                        app.popup_visible_indices = vec![0, 1];
                        app.popup_selected_visible = 0;
                        app.popup_in_search = false;
                        app.popup_search_query.clear();
                        app.info_message = format!("__EDIT_INDEX__{idx}");
                        app.popup_open = true;
                        return false;
                    }
                }
            }
            app.close_popup();
        }
        Some(PopupKind::ManualPartitionEdit) => {
            // Retrieve the stashed index from info_message
            let idx = app
                .info_message
                .strip_prefix("__EDIT_INDEX__")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                if global_idx == 0 {
                    // Modify: preload current spec into creation flow
                    if let Some(spec) = app.disks_partitions.get(idx).cloned() {
                        // Mark which partition we are editing so finalize updates instead of appending
                        app.manual_edit_index = Some(idx);
                        app.close_popup();
                        // Map role back to kind index
                        app.manual_create_kind_index = match spec.role.as_deref().unwrap_or("") {
                            "BOOT" => 0,
                            "SWAP" => 1,
                            "ROOT" => 2,
                            _ => 3,
                        };
                        // Preload size in GiB
                        if let (Some(start), Some(size)) = (spec.start, spec.size)
                            && let (Ok(st), Ok(sz)) = (start.parse::<u64>(), size.parse::<u64>())
                        {
                            app.manual_create_free_start_bytes = st;
                            app.manual_create_free_end_bytes = st.saturating_add(sz);
                            app.manual_create_selected_size_bytes = sz;
                            app.custom_input_buffer = format!("{}", sz / (1024 * 1024 * 1024));
                        }
                        // Filesystem and mountpoint
                        if let Some(fs) = spec.fs {
                            app.manual_create_fs_options = vec![fs.clone()];
                            app.manual_create_fs_index = 0;
                        }
                        if let Some(mp) = spec.mountpoint {
                            app.manual_create_mountpoint = mp;
                        }
                        app.open_manual_partition_size_units();
                    }
                } else {
                    // Delete
                    if idx < app.disks_partitions.len() {
                        app.disks_partitions.remove(idx);
                    }
                    app.close_popup();
                    app.open_manual_partition_table_for_selected();
                }
            }
        }
        Some(PopupKind::ManualPartitionKindSelect) => {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                app.manual_create_kind_index = global_idx;
                // For SWAP, no mountpoint; for BOOT/ROOT/OTHER, ask FS then mountpoint (except SWAP)
                app.close_popup();
                // Size + units first for all kinds
                app.open_manual_partition_size_units();
            }
        }
        Some(PopupKind::ManualPartitionCreate) => {
            // First Enter switches focus from size to units; next Enter proceeds
            if !app.manual_create_focus_units {
                app.manual_create_focus_units = true;
                return false;
            }
            // Persist the chosen size in bytes for subsequent steps
            let qty = app.custom_input_buffer.trim().parse::<u64>().unwrap_or(0);
            let unit_multiplier: u64 = match app.manual_create_units_index {
                0 => 1,
                1 => 1024,
                2 => 1024 * 1024,
                3 => 1024 * 1024 * 1024,
                _ => 1024 * 1024 * 1024,
            };
            let size_bytes = qty.saturating_mul(unit_multiplier);
            let start_b = app.manual_create_free_start_bytes;
            let end_b = app.manual_create_free_end_bytes;
            let max_bytes = end_b.saturating_sub(start_b);
            app.manual_create_selected_size_bytes = size_bytes.min(max_bytes);
            // After size+unit, go to FS selection if needed, then mountpoint
            let needs_fs = app.manual_create_kind_index != 1; // not for SWAP
            app.close_popup();
            if needs_fs {
                app.open_manual_partition_fs_select();
            } else {
                // SWAP done (no fs mountpoint questions) -> finalize and reopen table
                finalize_manual_partition(app);
            }
        }
        Some(PopupKind::ManualPartitionFilesystem) => {
            if let Some(&global_idx) = app.popup_visible_indices.get(app.popup_selected_visible) {
                app.manual_create_fs_index = global_idx;
                app.close_popup();
                // BOOT/ROOT/OTHER mountpoints
                if app.manual_create_kind_index == 0
                    || app.manual_create_kind_index == 2
                    || app.manual_create_kind_index == 3
                {
                    app.open_manual_partition_mountpoint();
                }
                // If SWAP or BOOT without mountpoint step, finalize
                if app.manual_create_kind_index == 1 {
                    // finalize directly
                    finalize_manual_partition(app);
                }
            }
        }
        Some(PopupKind::ManualPartitionMountpoint) => {
            app.manual_create_mountpoint = app.custom_input_buffer.clone();
            app.custom_input_buffer.clear();
            app.close_popup();
            finalize_manual_partition(app);
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
            app.open_disk_encryption_password_confirm_input();
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
                app.open_disk_encryption_partition_list();
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
        None => {
            // Special handling: pressing Enter on Install screen should ask wipe confirm
            if app.current_screen() == crate::core::types::Screen::Install {
                let dev = app
                    .disks_selected_device
                    .clone()
                    .unwrap_or_else(|| "the selected drive".into());
                app.popup_kind = Some(PopupKind::WipeConfirm);
                app.popup_items = vec![
                    format!("Yes — wipe {} and continue", dev),
                    "No — cancel".into(),
                ];
                app.popup_visible_indices = vec![0, 1];
                app.popup_selected_visible = 0; // Yes by default
                app.popup_in_search = false;
                app.popup_search_query.clear();
                app.popup_open = true;
                return false;
            } else {
                app.apply_popup_selection();
            }
        }
    }
    false
}
