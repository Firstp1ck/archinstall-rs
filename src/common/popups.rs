use crate::app::{AppState, PopupKind};

impl AppState {
    pub fn apply_popup_selection(&mut self) {
        if !self.popup_open {
            return;
        }
        if let Some(idx) = self
            .popup_visible_indices
            .get(self.popup_selected_visible)
            .copied()
        {
            match self.popup_kind {
                Some(PopupKind::KeyboardLayout) => {
                    if self.editing_locales {
                        self.draft_keyboard_layout_index = idx;
                    } else {
                        self.keyboard_layout_index = idx;
                    }
                    // Apply immediately to live environment
                    if let Some(layout) = self
                        .keyboard_layout_options
                        .get(if self.editing_locales {
                            self.draft_keyboard_layout_index
                        } else {
                            self.keyboard_layout_index
                        })
                        .cloned()
                    {
                        self.apply_live_keyboard_layout(&layout);
                    }
                }
                Some(PopupKind::LocaleLanguage) => {
                    if self.editing_locales {
                        self.draft_locale_language_index = idx;
                    } else {
                        self.locale_language_index = idx;
                    }
                    let selected_lang = self
                        .locale_language_options
                        .get(if self.editing_locales {
                            self.draft_locale_language_index
                        } else {
                            self.locale_language_index
                        })
                        .cloned();
                    if let Some(lang) = selected_lang {
                        let desired = self
                            .locale_language_to_encoding
                            .get(&lang)
                            .cloned()
                            .or_else(|| {
                                if lang.to_uppercase().contains("UTF-8") {
                                    Some("UTF-8".to_string())
                                } else {
                                    None
                                }
                            });
                        if let Some(enc_name) = desired {
                            if let Some(eidx) = self
                                .locale_encoding_options
                                .iter()
                                .position(|e| e == &enc_name)
                            {
                                if self.editing_locales {
                                    self.draft_locale_encoding_index = eidx;
                                } else {
                                    self.locale_encoding_index = eidx;
                                }
                            } else {
                                let upper = enc_name.to_uppercase();
                                if upper.starts_with("ISO") {
                                    if let Some(eidx) = self
                                        .locale_encoding_options
                                        .iter()
                                        .position(|e| e.to_uppercase().starts_with("ISO"))
                                    {
                                        if self.editing_locales {
                                            self.draft_locale_encoding_index = eidx;
                                        } else {
                                            self.locale_encoding_index = eidx;
                                        }
                                    }
                                } else if upper.contains("UTF-8")
                                    && let Some(eidx) = self
                                        .locale_encoding_options
                                        .iter()
                                        .position(|e| e.to_uppercase() == "UTF-8")
                                {
                                    if self.editing_locales {
                                        self.draft_locale_encoding_index = eidx;
                                    } else {
                                        self.locale_encoding_index = eidx;
                                    }
                                }
                            }
                        }
                    }
                }
                Some(PopupKind::LocaleEncoding) => {
                    if self.editing_locales {
                        self.draft_locale_encoding_index = idx;
                    } else {
                        self.locale_encoding_index = idx;
                    }
                }
                Some(PopupKind::MirrorsRegions) => {}
                Some(PopupKind::OptionalRepos) => {}
                Some(PopupKind::KernelSelect) => {}
                Some(PopupKind::AdditionalPackageGroupSelect)
                | Some(PopupKind::AdditionalPackageGroupPackages) => {}
                Some(PopupKind::MirrorsCustomServerInput) => {}
                Some(PopupKind::MirrorsCustomRepoName)
                | Some(PopupKind::MirrorsCustomRepoUrl)
                | Some(PopupKind::MirrorsCustomRepoSig)
                | Some(PopupKind::MirrorsCustomRepoSignOpt) => {}
                Some(PopupKind::DisksDeviceList) => {}
                Some(PopupKind::DiskEncryptionType)
                | Some(PopupKind::DiskEncryptionPassword)
                | Some(PopupKind::DiskEncryptionPasswordConfirm)
                | Some(PopupKind::DiskEncryptionPartitionList)
                | Some(PopupKind::AbortConfirm)
                | Some(PopupKind::Info)
                | Some(PopupKind::HostnameInput)
                | Some(PopupKind::RootPassword)
                | Some(PopupKind::RootPasswordConfirm)
                | Some(PopupKind::UserAddUsername)
                | Some(PopupKind::UserAddPassword)
                | Some(PopupKind::UserAddPasswordConfirm)
                | Some(PopupKind::UserAddSudo)
                | Some(PopupKind::DesktopEnvSelect)
                | Some(PopupKind::ServerTypeSelect)
                | Some(PopupKind::XorgTypeSelect)
                | Some(PopupKind::MinimalClearConfirm) => {}
                Some(_) => {}
                None => {}
            }
        }
        self.close_popup();
    }

    pub fn close_popup(&mut self) {
        self.popup_open = false;
        self.popup_kind = None;
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn filter_popup(&mut self) {
        if !self.popup_open {
            return;
        }
        if self.popup_search_query.is_empty() {
            self.popup_visible_indices = (0..self.popup_items.len()).collect();
            self.popup_selected_visible = 0;
            return;
        }
        let q = self.popup_search_query.to_lowercase();
        self.popup_visible_indices = self
            .popup_items
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                if s.to_lowercase().contains(&q) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();
        if self.popup_visible_indices.is_empty() {
            self.popup_selected_visible = 0;
        } else if self.popup_selected_visible >= self.popup_visible_indices.len() {
            self.popup_selected_visible = self.popup_visible_indices.len() - 1;
        }
    }

    pub fn open_info_popup(&mut self, message: String) {
        self.popup_kind = Some(PopupKind::Info);
        self.popup_items = vec![message];
        self.popup_visible_indices = vec![0];
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_open = true;
    }

    pub fn open_hostname_input(&mut self) {
        self.popup_kind = Some(PopupKind::HostnameInput);
        self.custom_input_buffer.clear();
        self.popup_open = true;
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_user_username_input(&mut self) {
        self.popup_kind = Some(PopupKind::UserAddUsername);
        self.custom_input_buffer.clear();
        self.popup_open = true;
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_user_password_input(&mut self) {
        self.popup_kind = Some(PopupKind::UserAddPassword);
        self.custom_input_buffer.clear();
        self.popup_open = true;
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_user_password_confirm_input(&mut self) {
        self.popup_kind = Some(PopupKind::UserAddPasswordConfirm);
        self.custom_input_buffer.clear();
        self.popup_open = true;
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_kernels_popup(&mut self) {
        self.popup_kind = Some(PopupKind::KernelSelect);
        self.popup_open = true;
        self.popup_items = vec![
            "linux".into(),
            "linux-hardened".into(),
            "linux-lts".into(),
            "linux-zen".into(),
        ];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_user_sudo_select(&mut self) {
        self.popup_kind = Some(PopupKind::UserAddSudo);
        self.popup_open = true;
        self.popup_items = vec!["Yes".into(), "No".into()];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_minimal_clear_confirm(&mut self) {
        self.popup_kind = Some(PopupKind::MinimalClearConfirm);
        self.popup_open = true;
        self.popup_items = vec!["Yes".into(), "No".into()];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_desktop_environment_popup(&mut self) {
        self.popup_kind = Some(PopupKind::DesktopEnvSelect);
        self.popup_open = true;
        self.popup_packages_focus = false;
        self.popup_packages_selected_index = 0;
        self.popup_drivers_focus = false;
        self.popup_drivers_selected_index = 0;
        self.popup_login_focus = false;
        self.popup_login_selected_index = 0;
        self.popup_items = vec![
            "Awesome".into(),
            "Bspwm".into(),
            "Budgie".into(),
            "Cinnamon".into(),
            "Cutefish".into(),
            "Deepin".into(),
            "Enlightenment".into(),
            "GNOME".into(),
            "Hyprland".into(),
            "KDE Plasma".into(),
            "Lxqt".into(),
            "Mate".into(),
            "Qtile".into(),
            "Sway".into(),
            "Xfce4".into(),
            "i3-wm".into(),
        ];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_server_type_popup(&mut self) {
        self.popup_kind = Some(PopupKind::ServerTypeSelect);
        self.popup_open = true;
        self.popup_items = vec![
            "Cockpit".into(),
            "Docker".into(),
            "Lighttpd".into(),
            "Mariadb".into(),
            "Nginx".into(),
            "Postgresql".into(),
            "Tomcat".into(),
            "httpd".into(),
            "sshd".into(),
        ];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_xorg_type_popup(&mut self) {
        self.popup_kind = Some(PopupKind::XorgTypeSelect);
        self.popup_open = true;
        self.popup_items = vec!["Xorg".into()];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_packages_focus = false;
        self.popup_packages_selected_index = 0;
        self.popup_drivers_focus = false;
        self.popup_drivers_selected_index = 0;
    }

    pub fn start_add_user_flow(&mut self) {
        self.draft_user_username.clear();
        self.draft_user_password.clear();
        self.draft_user_password_confirm.clear();
        self.draft_user_is_sudo = false;
        self.open_user_username_input();
    }

    pub fn open_root_password_input(&mut self) {
        self.popup_kind = Some(PopupKind::RootPassword);
        self.custom_input_buffer.clear();
        self.popup_open = true;
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_root_password_confirm_input(&mut self) {
        self.popup_kind = Some(PopupKind::RootPasswordConfirm);
        self.custom_input_buffer.clear();
        self.popup_open = true;
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }
}
