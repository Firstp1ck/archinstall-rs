use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

mod abort;
mod additional_packages;
mod audio;
mod automatic_time_sync;
pub mod bootloader;
pub mod config;
pub mod disk_encryption;
pub mod disks;
pub mod experience_mode;
pub mod hostname;
mod install;
mod kernels;
pub mod locales;
pub mod mirrors;
mod network_configuration;
pub mod root_password;
mod save_configuration;
pub mod swap_partition;
mod timezone;
pub mod unified_kernel_images;
pub mod user_account;

pub const LEFT_MENU_WIDTH: u16 = 30;
pub const INFOBOX_HEIGHT: u16 = 12;
pub const KEYBINDS_WIDTH: u16 = 56;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Menu,
    Content,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Overview,
    Locales,
    MirrorsRepos,
    Disks,
    DiskEncryption,
    SwapPartition,
    Bootloader,
    UnifiedKernelImages,
    Hostname,
    RootPassword,
    UserAccount,
    ExperienceMode,
    Audio,
    Kernels,
    NetworkConfiguration,
    AdditionalPackages,
    Timezone,
    AutomaticTimeSync,
    SaveConfiguration,
    Install,
    Abort,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PopupKind {
    KeyboardLayout,
    LocaleLanguage,
    LocaleEncoding,
    MirrorsRegions,
    OptionalRepos,
    MirrorsCustomServerInput,
    MirrorsCustomRepoName,
    MirrorsCustomRepoUrl,
    MirrorsCustomRepoSig,
    MirrorsCustomRepoSignOpt,
    DisksDeviceList,
    DiskEncryptionType,
    DiskEncryptionPassword,
    DiskEncryptionPasswordConfirm,
    DiskEncryptionPartitionList,
    HostnameInput,
    RootPassword,
    RootPasswordConfirm,
    UserAddUsername,
    UserAddPassword,
    UserAddPasswordConfirm,
    UserAddSudo,
    DesktopEnvSelect,
    ServerTypeSelect,
    XorgTypeSelect,
    Info,
    AbortConfirm,
    MinimalClearConfirm,
}

#[derive(Clone)]
pub struct MenuEntry {
    pub label: String,
    pub content: String,
    pub screen: Screen,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RepoSignature {
    Never,
    Optional,
    Required,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RepoSignOption {
    TrustedOnly,
    TrustedAll,
}

#[derive(Clone)]
pub struct CustomRepo {
    pub name: String,
    pub url: String,
    pub signature: RepoSignature,
    pub sign_option: Option<RepoSignOption>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub username: String,
    pub password: String,
    pub is_sudo: bool,
}

// Config structs moved to app/config.rs

pub struct AppState {
    pub menu_entries: Vec<MenuEntry>,
    pub selected_index: usize,
    pub info_message: String,
    pub focus: Focus,
    pub last_menu_rect: Rect,
    pub last_infobox_rect: Rect,
    pub last_content_rect: Rect,
    pub list_state: ListState,

    // Locales screen state
    pub locales_focus_index: usize, // 0: keyboard, 1: language, 2: encoding
    pub keyboard_layout_options: Vec<String>,
    pub keyboard_layout_index: usize,
    pub locale_language_options: Vec<String>,
    pub locale_language_index: usize,
    pub locale_encoding_options: Vec<String>,
    pub locale_encoding_index: usize,
    pub locales_loaded: bool,

    // Popup state
    pub popup_open: bool,
    pub popup_kind: Option<PopupKind>,
    pub popup_items: Vec<String>,
    pub popup_visible_indices: Vec<usize>,
    pub popup_selected_visible: usize,
    pub popup_in_search: bool,
    pub popup_search_query: String,

    // Edit session (Locales)
    pub editing_locales: bool,
    pub draft_keyboard_layout_index: usize,
    pub draft_locale_language_index: usize,
    pub draft_locale_encoding_index: usize,

    // Command line (vim-like) in decision menu
    pub cmdline_open: bool,
    pub cmdline_buffer: String,

    // Disk Partitioning screen state
    pub disks_focus_index: usize, // 0..=2 items + 3 Continue
    pub disks_mode_index: usize,  // selected mode index 0..=2
    pub disks_devices: Vec<disks::DiskDevice>,
    pub disks_selected_device: Option<String>,

    // Mirrors & Repositories screen state
    pub mirrors_focus_index: usize, // 0..=3 items + 4 Continue
    pub mirrors_regions_options: Vec<String>,
    pub mirrors_regions_selected: BTreeSet<usize>,
    pub mirrors_loaded: bool,
    pub optional_repos_options: Vec<String>,
    pub optional_repos_selected: BTreeSet<usize>,
    pub mirrors_custom_servers: Vec<String>,
    pub custom_input_buffer: String,
    pub custom_repos: Vec<CustomRepo>,
    pub draft_repo_name: String,
    pub draft_repo_url: String,
    pub draft_repo_sig_index: usize,
    pub draft_repo_signopt_index: usize,

    // Disk Encryption screen state
    pub diskenc_focus_index: usize,        // fields + Continue
    pub disk_encryption_type_index: usize, // 0: None, 1: LUKS
    pub disk_encryption_password: String,
    pub disk_encryption_password_confirm: String,
    pub disk_encryption_selected_partition: Option<String>,

    // Swap Partition state
    pub swap_focus_index: usize, // 0: toggle, 1: Continue
    pub swap_enabled: bool,

    // Unified Kernel Images state
    pub uki_focus_index: usize, // 0: toggle, 1: Continue
    pub uki_enabled: bool,

    // Bootloader state
    pub bootloader_focus_index: usize, // 0: selector, 1: Continue
    pub bootloader_index: usize,       // 0: systemd-boot, 1: grub, 2: efistub, 3: limine

    // Experience Mode state
    pub experience_focus_index: usize, // 0..=3 items + 4 Continue
    pub experience_mode_index: usize,  // 0: Desktop, 1: Minimal, 2: Server, 3: Xorg
    pub selected_desktop_envs: std::collections::BTreeSet<String>,
    pub selected_server_types: std::collections::BTreeSet<String>,
    pub selected_server_packages:
        std::collections::BTreeMap<String, std::collections::BTreeSet<String>>,
    // Xorg popup: selected types and packages (mirrors Server structure)
    pub selected_xorg_types: std::collections::BTreeSet<String>,
    pub selected_xorg_packages:
        std::collections::BTreeMap<String, std::collections::BTreeSet<String>>,
    // Desktop popup: selected packages per environment
    pub selected_env_packages:
        std::collections::BTreeMap<String, std::collections::BTreeSet<String>>,
    // Desktop popup: focus and selection within packages list
    pub popup_packages_focus: bool,
    pub popup_packages_selected_index: usize,
    // Desktop popup: globally selected login manager (None means no manager)
    pub selected_login_manager: Option<String>,
    pub login_manager_user_set: bool,
    // Desktop popup: focus and selection within login managers list
    pub popup_login_focus: bool,
    pub popup_login_selected_index: usize,

    // Hostname state
    pub hostname_focus_index: usize, // 0 input, 1 Continue
    pub hostname_value: String,

    // Root Password state
    pub rootpass_focus_index: usize, // 0 set, 1 confirm, 2 Continue
    pub root_password: String,
    pub root_password_confirm: String,

    // User Account state
    pub user_focus_index: usize, // 0: Add user, 1: Continue
    pub users: Vec<UserAccount>,
    pub draft_user_username: String,
    pub draft_user_password: String,
    pub draft_user_password_confirm: String,
    pub draft_user_is_sudo: bool,

    // Configuration screen state
    pub config_focus_index: usize, // 0: Save, 1: Load, 2: Continue

    // Load feedback
    pub last_load_missing_sections: Vec<String>,
}

impl AppState {
    pub fn new() -> Self {
        let menu_entries = vec![
            MenuEntry {
                label: "Overview".into(),
                content: "Welcome to archinstall-rs. Use Up/Down, Enter".into(),
                screen: Screen::Overview,
            },
            MenuEntry {
                label: "Locales".into(),
                content: String::new(),
                screen: Screen::Locales,
            },
            MenuEntry {
                label: "Mirrors and Repositories".into(),
                content: String::new(),
                screen: Screen::MirrorsRepos,
            },
            MenuEntry {
                label: "Disk Partitioning".into(),
                content: String::new(),
                screen: Screen::Disks,
            },
            MenuEntry {
                label: "Disk Encryption".into(),
                content: String::new(),
                screen: Screen::DiskEncryption,
            },
            MenuEntry {
                label: "Swap Partition".into(),
                content: String::new(),
                screen: Screen::SwapPartition,
            },
            MenuEntry {
                label: "Bootloader".into(),
                content: String::new(),
                screen: Screen::Bootloader,
            },
            MenuEntry {
                label: "Unified Kernel Images".into(),
                content: String::new(),
                screen: Screen::UnifiedKernelImages,
            },
            MenuEntry {
                label: "Hostname".into(),
                content: String::new(),
                screen: Screen::Hostname,
            },
            MenuEntry {
                label: "Root Password".into(),
                content: String::new(),
                screen: Screen::RootPassword,
            },
            MenuEntry {
                label: "User Account".into(),
                content: String::new(),
                screen: Screen::UserAccount,
            },
            MenuEntry {
                label: "Experience Mode".into(),
                content: String::new(),
                screen: Screen::ExperienceMode,
            },
            MenuEntry {
                label: "Audio".into(),
                content: String::new(),
                screen: Screen::Audio,
            },
            MenuEntry {
                label: "Kernels".into(),
                content: String::new(),
                screen: Screen::Kernels,
            },
            MenuEntry {
                label: "Network Configuration".into(),
                content: "Network setup and status.".into(),
                screen: Screen::NetworkConfiguration,
            },
            MenuEntry {
                label: "Additional Packages".into(),
                content: "Package selection and groups.".into(),
                screen: Screen::AdditionalPackages,
            },
            MenuEntry {
                label: "Timezone".into(),
                content: String::new(),
                screen: Screen::Timezone,
            },
            MenuEntry {
                label: "Automatic Time Sync".into(),
                content: String::new(),
                screen: Screen::AutomaticTimeSync,
            },
            MenuEntry {
                label: "Configuration".into(),
                content: String::new(),
                screen: Screen::SaveConfiguration,
            },
            MenuEntry {
                label: "Install".into(),
                content: "Review and start installation.".into(),
                screen: Screen::Install,
            },
            MenuEntry {
                label: "Abort (Ctrl + C)".into(),
                content: String::new(),
                screen: Screen::Abort,
            },
        ];

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut s = Self {
            menu_entries,
            selected_index: 0,
            info_message: String::new(),
            focus: Focus::Menu,
            last_menu_rect: Rect::default(),
            last_infobox_rect: Rect::default(),
            last_content_rect: Rect::default(),
            list_state,

            locales_focus_index: 0,
            keyboard_layout_options: Vec::new(),
            keyboard_layout_index: 0,
            locale_language_options: Vec::new(),
            locale_language_index: 0,
            locale_encoding_options: Vec::new(),
            locale_encoding_index: 0,
            locales_loaded: false,

            popup_open: false,
            popup_kind: None,
            popup_items: Vec::new(),
            popup_visible_indices: Vec::new(),
            popup_selected_visible: 0,
            popup_in_search: false,
            popup_search_query: String::new(),

            editing_locales: false,
            draft_keyboard_layout_index: 0,
            draft_locale_language_index: 0,
            draft_locale_encoding_index: 0,

            cmdline_open: false,
            cmdline_buffer: String::new(),

            disks_focus_index: 0,
            disks_mode_index: 0,
            disks_devices: Vec::new(),
            disks_selected_device: None,

            mirrors_focus_index: 0,
            mirrors_regions_options: Vec::new(),
            mirrors_regions_selected: BTreeSet::new(),
            mirrors_loaded: false,
            optional_repos_options: vec!["multilib".into(), "testing".into()],
            optional_repos_selected: BTreeSet::new(),
            mirrors_custom_servers: Vec::new(),
            custom_input_buffer: String::new(),
            custom_repos: Vec::new(),
            draft_repo_name: String::new(),
            draft_repo_url: String::new(),
            draft_repo_sig_index: 2,
            draft_repo_signopt_index: 0,

            diskenc_focus_index: 0,
            disk_encryption_type_index: 0,
            disk_encryption_password: String::new(),
            disk_encryption_password_confirm: String::new(),
            disk_encryption_selected_partition: None,

            swap_focus_index: 0,
            swap_enabled: false,

            uki_focus_index: 0,
            uki_enabled: false,

            bootloader_focus_index: 0,
            bootloader_index: 0,

            experience_focus_index: 0,
            experience_mode_index: 0,
            selected_desktop_envs: std::collections::BTreeSet::new(),
            selected_server_types: std::collections::BTreeSet::new(),
            selected_server_packages: std::collections::BTreeMap::new(),
            selected_xorg_types: std::collections::BTreeSet::new(),
            selected_xorg_packages: std::collections::BTreeMap::new(),
            selected_env_packages: std::collections::BTreeMap::new(),
            popup_packages_focus: false,
            popup_packages_selected_index: 0,
            selected_login_manager: None,
            login_manager_user_set: false,
            popup_login_focus: false,
            popup_login_selected_index: 0,

            hostname_focus_index: 0,
            hostname_value: String::new(),

            rootpass_focus_index: 0,
            root_password: String::new(),
            root_password_confirm: String::new(),

            user_focus_index: 0,
            users: Vec::new(),
            draft_user_username: String::new(),
            draft_user_password: String::new(),
            draft_user_password_confirm: String::new(),
            draft_user_is_sudo: false,

            config_focus_index: 0,

            last_load_missing_sections: Vec::new(),
        };
        s.update_unified_kernel_images_visibility();
        s
    }

    pub fn current_screen(&self) -> Screen {
        self.menu_entries[self.selected_index].screen
    }

    // config helpers moved to app/config.rs

    // locales getters moved to app/locales.rs

    // locales popup open moved to app/locales.rs

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
                }
                Some(PopupKind::LocaleLanguage) => {
                    if self.editing_locales {
                        self.draft_locale_language_index = idx;
                    } else {
                        self.locale_language_index = idx;
                    }
                }
                Some(PopupKind::LocaleEncoding) => {
                    if self.editing_locales {
                        self.draft_locale_encoding_index = idx;
                    } else {
                        self.locale_encoding_index = idx;
                    }
                }
                Some(PopupKind::MirrorsRegions) => { /* multi-select handled via spacebar; Enter closes */
                }
                Some(PopupKind::OptionalRepos) => { /* multi-select handled via spacebar; Enter closes */
                }
                Some(PopupKind::MirrorsCustomServerInput) => { /* handled via Enter appending buffer; keep open */
                }
                Some(PopupKind::MirrorsCustomRepoName)
                | Some(PopupKind::MirrorsCustomRepoUrl)
                | Some(PopupKind::MirrorsCustomRepoSig)
                | Some(PopupKind::MirrorsCustomRepoSignOpt) => { /* handled in input flow */ }
                Some(PopupKind::DisksDeviceList) => { /* handled in input flow; no direct selection */
                }
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
                | Some(PopupKind::MinimalClearConfirm) => { /* handled in input flow */ }
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

    // locales helpers moved to app/locales.rs

    // mirrors helpers moved to app/mirrors.rs

    pub fn finalize_custom_repo(&mut self) {
        let sig = match self.draft_repo_sig_index {
            0 => RepoSignature::Never,
            1 => RepoSignature::Optional,
            _ => RepoSignature::Required,
        };
        let sign_option = if self.draft_repo_sig_index == 0 {
            None
        } else {
            Some(match self.draft_repo_signopt_index {
                0 => RepoSignOption::TrustedOnly,
                _ => RepoSignOption::TrustedAll,
            })
        };
        self.custom_repos.push(CustomRepo {
            name: self.draft_repo_name.clone(),
            url: self.draft_repo_url.clone(),
            signature: sig,
            sign_option,
        });
        self.close_popup();
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
        // Single Xorg type for now
        self.popup_items = vec!["Xorg".into()];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        // Reset pane focus to left when opening
        self.popup_packages_focus = false;
        self.popup_packages_selected_index = 0;
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

    pub fn update_unified_kernel_images_visibility(&mut self) {
        // Show UKI for all bootloaders except GRUB (index 1)
        let should_show = self.bootloader_index != 1;
        let uki_pos = self
            .menu_entries
            .iter()
            .position(|e| matches!(e.screen, Screen::UnifiedKernelImages));
        if should_show {
            if uki_pos.is_none()
                && let Some(bl_idx) = self
                    .menu_entries
                    .iter()
                    .position(|e| matches!(e.screen, Screen::Bootloader))
            {
                let insert_at = bl_idx + 1;
                self.menu_entries.insert(
                    insert_at,
                    MenuEntry {
                        label: "Unified Kernel Images".into(),
                        content: String::new(),
                        screen: Screen::UnifiedKernelImages,
                    },
                );
                if self.selected_index >= insert_at {
                    self.selected_index += 1;
                    self.list_state.select(Some(self.selected_index));
                }
            }
        } else if let Some(idx) = uki_pos {
            // If currently selected, move selection away first
            if self.selected_index == idx {
                if idx + 1 < self.menu_entries.len() {
                    self.selected_index += 1;
                } else if idx > 0 {
                    self.selected_index -= 1;
                }
                self.list_state.select(Some(self.selected_index));
            }
            self.menu_entries.remove(idx);
            if self.selected_index > idx {
                self.selected_index = self.selected_index.saturating_sub(1);
                self.list_state.select(Some(self.selected_index));
            }
        }
    }

    // disk helpers moved to app/disks.rs
    // locales helpers moved to app/locales.rs
    // mirrors helpers moved to app/mirrors.rs
}
