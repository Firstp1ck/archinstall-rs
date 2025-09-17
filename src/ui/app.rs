use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

mod abort;
pub mod additional_packages;
pub mod audio;
pub mod automatic_time_sync;
pub mod bootloader;
pub mod config;
pub mod disk_encryption;
pub mod disks;
pub mod experience_mode;
pub mod hostname;
pub mod install;
pub mod kernels;
pub mod locales;
pub mod mirrors;
pub mod network_configuration;
pub mod root_password;
mod save_configuration;
pub mod swap_partition;
pub mod timezone;
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    UserSelectEdit,
    UserSelectDelete,
    UserEditUsername,
    DesktopEnvSelect,
    ServerTypeSelect,
    XorgTypeSelect,
    Info,
    AbortConfirm,
    MinimalClearConfirm,
    KernelSelect,
    TimezoneSelect,
    AdditionalPackageInput,
    NetworkInterfaces,
    NetworkMode,
    NetworkIP,
    NetworkGateway,
    NetworkDNS,
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
pub struct AdditionalPackage {
    pub name: String,
    pub repo: String,
    pub version: String,
    pub description: String,
}

#[derive(Clone)]
pub enum NetworkConfigMode {
    Dhcp,
    Static,
}

#[derive(Clone)]
pub struct NetworkInterfaceConfig {
    pub interface: String,
    pub mode: NetworkConfigMode,
    pub ip_cidr: Option<String>,
    pub gateway: Option<String>,
    pub dns: Option<String>,
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
    pub password_hash: Option<String>,
    pub is_sudo: bool,
}

// Config structs moved to app/config.rs

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct DiskPartitionSpec {
    pub name: Option<String>,
    pub role: Option<String>,
    pub fs: Option<String>,
    pub start: Option<String>,
    pub size: Option<String>,
    pub flags: Vec<String>,
    pub mountpoint: Option<String>,
    pub mount_options: Option<String>,
    pub encrypt: Option<bool>,
}

pub struct AppState {
    pub dry_run: bool,
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
    pub locale_language_to_encoding: std::collections::BTreeMap<String, String>,
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
    // Cached details of the selected device (for partitioning/mounting)
    pub disks_selected_device_model: Option<String>,
    pub disks_selected_device_devtype: Option<String>,
    pub disks_selected_device_size: Option<String>,
    pub disks_selected_device_freespace: Option<String>,
    pub disks_selected_device_sector_size: Option<String>,
    pub disks_selected_device_read_only: Option<bool>,
    // Extended disk configuration
    pub disks_label: Option<String>,
    pub disks_wipe: bool,
    pub disks_align: Option<String>,
    pub disks_partitions: Vec<DiskPartitionSpec>,

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
    pub diskenc_reopen_after_info: bool,
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

    // Kernels state
    pub kernels_focus_index: usize, // 0: select, 1: Continue
    pub selected_kernels: std::collections::BTreeSet<String>,

    // Audio state
    pub audio_focus_index: usize, // 0..=2 choices + 3 Continue
    pub audio_index: usize,       // 0: None, 1: pipewire, 2: pulseaudio

    // Network Configuration state
    pub network_focus_index: usize, // 0..=2 choices + 3 Continue
    pub network_mode_index: usize,  // 0: Copy ISO, 1: Manual, 2: NetworkManager
    pub network_configs: Vec<NetworkInterfaceConfig>,
    pub network_selected_interface: Option<String>,
    pub network_draft_mode_static: bool,
    pub network_draft_ip_cidr: String,
    pub network_draft_gateway: String,
    pub network_draft_dns: String,
    pub network_reopen_after_info_ip: bool,
    pub network_reopen_after_info_gateway: bool,
    pub network_reopen_after_info_dns: bool,

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
    // Graphic Drivers selection shared by Desktop/Xorg popups
    pub selected_graphic_drivers: std::collections::BTreeSet<String>,
    pub popup_drivers_focus: bool,
    pub popup_drivers_selected_index: usize,
    // Desktop popup: globally selected login manager (None means no manager)
    pub selected_login_manager: Option<String>,
    pub login_manager_user_set: bool,
    // Desktop popup: focus and selection within login managers list
    pub popup_login_focus: bool,
    pub popup_login_selected_index: usize,

    // Hostname state
    pub hostname_focus_index: usize, // 0 input, 1 Continue
    pub hostname_value: String,
    pub hostname_reopen_after_info: bool,

    // Timezone state
    pub timezone_focus_index: usize, // 0: select, 1: Continue
    pub timezone_value: String,

    // Automatic Time Sync state
    pub ats_focus_index: usize, // 0: Yes, 1: No, 2: Continue
    pub ats_enabled: bool,

    // Root Password state
    pub rootpass_focus_index: usize, // 0 set, 1 confirm, 2 Continue
    pub root_password: String,
    pub root_password_confirm: String,
    pub root_password_hash: Option<String>,
    pub rootpass_reopen_after_info: bool,

    // User Account state
    pub user_focus_index: usize, // 0: Add user, 1: Continue
    pub users: Vec<UserAccount>,
    pub selected_user_index: usize,
    pub draft_user_username: String,
    pub draft_user_password: String,
    pub draft_user_password_confirm: String,
    pub draft_user_is_sudo: bool,
    pub username_reopen_after_info: bool,
    pub userpass_reopen_after_info: bool,
    pub useredit_reopen_after_info: bool,

    // Configuration screen state
    pub config_focus_index: usize, // 0: Save, 1: Load, 2: Continue

    // Load feedback
    pub last_load_missing_sections: Vec<String>,

    // Additional Packages state
    pub addpkgs_focus_index: usize, // 0: Add package input, 1: Continue
    pub additional_packages: Vec<AdditionalPackage>,
    pub addpkgs_selected_index: usize,
    pub addpkgs_selected: std::collections::BTreeSet<usize>,
    pub addpkgs_reopen_after_info: bool,

    // Sections processed
    pub processed_sections: BTreeSet<Screen>,
}

impl AppState {
    pub fn new(dry_run: bool) -> Self {
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
            dry_run,
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
            locale_language_to_encoding: std::collections::BTreeMap::new(),
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
            disks_selected_device_model: None,
            disks_selected_device_devtype: None,
            disks_selected_device_size: None,
            disks_selected_device_freespace: None,
            disks_selected_device_sector_size: None,
            disks_selected_device_read_only: None,
            disks_label: Some("gpt".into()),
            disks_wipe: true,
            disks_align: Some("1MiB".into()),
            disks_partitions: Vec::new(),

            mirrors_focus_index: 0,
            mirrors_regions_options: Vec::new(),
            mirrors_regions_selected: BTreeSet::new(),
            mirrors_loaded: false,
            optional_repos_options: vec!["multilib".into(), "testing".into()],
            optional_repos_selected: {
                let mut s = BTreeSet::new();
                s.insert(0);
                s
            },
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
            diskenc_reopen_after_info: false,
            disk_encryption_selected_partition: None,

            swap_focus_index: 0,
            swap_enabled: true,

            uki_focus_index: 0,
            uki_enabled: false,

            bootloader_focus_index: 0,
            bootloader_index: 0,

            kernels_focus_index: 0,
            selected_kernels: {
                let mut s = std::collections::BTreeSet::new();
                s.insert("linux".into());
                s
            },

            audio_focus_index: 0,
            audio_index: 1,

            network_focus_index: 0,
            network_mode_index: 2,
            network_configs: Vec::new(),
            network_selected_interface: None,
            network_draft_mode_static: false,
            network_draft_ip_cidr: String::new(),
            network_draft_gateway: String::new(),
            network_draft_dns: String::new(),
            network_reopen_after_info_ip: false,
            network_reopen_after_info_gateway: false,
            network_reopen_after_info_dns: false,

            experience_focus_index: 0,
            experience_mode_index: 0,
            selected_desktop_envs: {
                let mut s = std::collections::BTreeSet::new();
                s.insert("KDE Plasma".into());
                s
            },
            selected_server_types: std::collections::BTreeSet::new(),
            selected_server_packages: std::collections::BTreeMap::new(),
            selected_xorg_types: std::collections::BTreeSet::new(),
            selected_xorg_packages: std::collections::BTreeMap::new(),
            selected_env_packages: std::collections::BTreeMap::new(),
            popup_packages_focus: false,
            popup_packages_selected_index: 0,
            selected_graphic_drivers: {
                let mut s = std::collections::BTreeSet::new();
                s.insert("intel-media-driver".into());
                s.insert("libva-intel-driver".into());
                s.insert("mesa".into());
                s.insert("vulkan-intel".into());
                s.insert("vulkan-nouveau".into());
                s.insert("vulkan-radeon".into());
                s.insert("xf86-video-amdgpu".into());
                s.insert("xf86-video-ati".into());
                s.insert("xf86-video-nouveau".into());
                s.insert("xf86-video-vmware".into());
                s.insert("xorg-server".into());
                s.insert("xorg-xinit".into());
                s
            },
            popup_drivers_focus: false,
            popup_drivers_selected_index: 0,
            selected_login_manager: Some("sddm".into()),
            login_manager_user_set: false,
            popup_login_focus: false,
            popup_login_selected_index: 0,

            hostname_focus_index: 0,
            hostname_value: "Archlinux".into(),
            hostname_reopen_after_info: false,

            timezone_focus_index: 0,
            timezone_value: "Europe/London".into(),

            ats_focus_index: 0,
            ats_enabled: true,

            rootpass_focus_index: 0,
            root_password: String::new(),
            root_password_confirm: String::new(),
            root_password_hash: None,
            rootpass_reopen_after_info: false,

            user_focus_index: 0,
            users: Vec::new(),
            selected_user_index: 0,
            draft_user_username: String::new(),
            draft_user_password: String::new(),
            draft_user_password_confirm: String::new(),
            draft_user_is_sudo: false,
            username_reopen_after_info: false,
            userpass_reopen_after_info: false,
            useredit_reopen_after_info: false,

            config_focus_index: 0,

            last_load_missing_sections: Vec::new(),

            addpkgs_focus_index: 0,
            additional_packages: Vec::new(),
            addpkgs_selected_index: 0,
            addpkgs_selected: std::collections::BTreeSet::new(),
            addpkgs_reopen_after_info: false,

            processed_sections: BTreeSet::new(),
        };
        // Initialize dynamic option lists and apply startup defaults
        let _ = s.load_locales_options();
        // Keyboard: us
        if let Some(idx) = s
            .keyboard_layout_options
            .iter()
            .position(|k| k == "us")
        {
            s.keyboard_layout_index = idx;
            s.draft_keyboard_layout_index = idx;
        }
        // Locale language: en_US.UTF-8
        if let Some(idx) = s
            .locale_language_options
            .iter()
            .position(|l| l == "en_US.UTF-8")
        {
            s.locale_language_index = idx;
            s.draft_locale_language_index = idx;
        }
        // Encoding: UTF-8
        if let Some(idx) = s
            .locale_encoding_options
            .iter()
            .position(|e| e.eq_ignore_ascii_case("UTF-8"))
        {
            s.locale_encoding_index = idx;
            s.draft_locale_encoding_index = idx;
        }

        let _ = s.load_mirrors_options();
        if s.mirrors_regions_selected.is_empty() {
            if let Some(idx) = s
                .mirrors_regions_options
                .iter()
                .position(|c| c == "United States          US   186")
                .or_else(|| {
                    s.mirrors_regions_options
                        .iter()
                        .position(|c| c.contains("United States"))
                })
            {
                s.mirrors_regions_selected.insert(idx);
            }
        }

        s.update_unified_kernel_images_visibility();
        // Seed default Desktop Environment packages for preselected environments
        if s.selected_desktop_envs.contains("KDE Plasma")
            && !s.selected_env_packages.contains_key("KDE Plasma")
        {
            let defaults: Vec<&str> = vec![
                "ark",
                "dolphin",
                "kate",
                "konsole",
                "plasma-meta",
                "plasma-workspace",
                // Common tools
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
            ];
            let set: std::collections::BTreeSet<String> =
                defaults.into_iter().map(|s| s.to_string()).collect();
            s.selected_env_packages.insert("KDE Plasma".into(), set);
        }
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
                    // Auto-select encoding based on selected locale language
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
                                // Heuristic: if language token includes UTF-8, prefer UTF-8
                                if lang.to_uppercase().contains("UTF-8") {
                                    Some("UTF-8".to_string())
                                } else {
                                    None
                                }
                            });
                        if let Some(enc_name) = desired {
                            // Exact match first
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
                                // Fallbacks
                                let upper = enc_name.to_uppercase();
                                // If ISO hinted, pick first ISO available
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
                                    &&
                                    let Some(eidx) = self
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
                Some(PopupKind::MirrorsRegions) => { /* multi-select handled via spacebar; Enter closes */
                }
                Some(PopupKind::OptionalRepos) => { /* multi-select handled via spacebar; Enter closes */
                }
                Some(PopupKind::KernelSelect) => { /* multi-select handled via spacebar; Enter closes */
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
                Some(_) => { /* unhandled popup: ignore selection */ }
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

    pub fn is_ascii_only(input: &str) -> bool { input.is_ascii() }

    pub fn is_ascii_lowercase_only(input: &str) -> bool {
        input.chars().all(|c| c.is_ascii_lowercase())
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
        // Single Xorg type for now
        self.popup_items = vec!["Xorg".into()];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        // Reset pane focus to left when opening
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
