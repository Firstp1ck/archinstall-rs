use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use std::collections::BTreeSet;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use crate::core::types::{
    AdditionalPackage, CustomRepo, DiskPartitionSpec, Focus, InstallClickTarget, MenuEntry,
    NetworkInterfaceConfig, PopupKind, Screen, UserAccount,
};

pub struct AppState {
    /// True if a real install completed (not dry-run)
    pub install_completed: bool,
    // Reboot prompt state
    pub reboot_prompt_open: bool,
    pub reboot_confirmed: Option<bool>,
    /// For visual indicator: last seen install_log length
    pub last_install_log_len: Option<usize>,
    pub dry_run: bool,
    /// Enable extra debug logging to debug.log
    pub debug_enabled: bool,
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
    pub disks_devices: Vec<crate::app::disks::DiskDevice>,
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

    // Manual Partitioning: create partition popup state
    pub manual_create_units_index: usize, // 0: B, 1: KiB/KB, 2: MiB/MB, 3: GiB/GB
    pub manual_create_free_start_bytes: u64,
    pub manual_create_free_end_bytes: u64,
    pub manual_create_selected_size_bytes: u64,
    pub manual_create_focus_units: bool,
    pub manual_create_kind_index: usize, // 0: BOOT, 1: SWAP, 2: ROOT, 3: OTHER
    pub manual_create_fs_options: Vec<String>,
    pub manual_create_fs_index: usize,
    pub manual_create_mountpoint: String,
    pub manual_edit_index: Option<usize>,

    // Manual Partitioning: per-row metadata for the table (selection handling)
    pub manual_partition_row_meta: Vec<crate::core::types::ManualPartitionRowMeta>,

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
    // AUR settings
    pub aur_selected: bool,              // true if AUR repo option was chosen
    pub aur_helper_index: Option<usize>, // 0: yay, 1: paru

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
    // Additional Packages: groups
    pub addpkgs_group_focus: bool, // focus within groups vs main
    pub addpkgs_group_names: Vec<String>,
    pub addpkgs_group_index: usize,
    pub addpkgs_group_pkg_index: usize,
    pub addpkgs_group_pkg_selected: std::collections::BTreeSet<String>,
    pub addpkgs_group_accum_selected: std::collections::BTreeSet<String>,
    // Persistent selections per group across popup sessions
    pub addpkgs_group_selected:
        std::collections::BTreeMap<String, std::collections::BTreeSet<String>>,

    // Sections processed
    pub processed_sections: BTreeSet<Screen>,

    // Install flow temp state
    pub pending_wipe_confirm: Option<bool>,

    // Live install logging state
    pub install_running: bool,
    pub install_log: Vec<String>,
    pub install_log_tx: Option<Sender<String>>,
    pub install_log_rx: Option<Receiver<String>>,
    // Install progress (for in-TUI progress view)
    pub install_section_titles: Vec<String>,
    pub install_section_done: Vec<bool>,
    pub install_current_section: Option<usize>,

    // Request to exit TUI and run install in stdout mode
    pub exit_tui_after_install: bool,
    pub pending_install_sections: Option<Vec<(String, Vec<String>)>>,

    // Clickable targets in Install decision menu (computed each render)
    pub install_click_targets: Vec<(ratatui::layout::Rect, InstallClickTarget)>,
    // Keyboard selection within Install decision menu (index into install_click_targets)
    pub install_focus_index: usize,
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
            install_completed: false,
            reboot_prompt_open: false,
            reboot_confirmed: None,
            last_install_log_len: None,
            dry_run,
            debug_enabled: false,
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

            manual_create_units_index: 0,
            manual_create_free_start_bytes: 0,
            manual_create_free_end_bytes: 0,
            manual_create_selected_size_bytes: 0,
            manual_create_focus_units: false,
            manual_create_kind_index: 0,
            manual_create_fs_options: Vec::new(),
            manual_create_fs_index: 0,
            manual_create_mountpoint: String::new(),
            manual_edit_index: None,

            manual_partition_row_meta: Vec::new(),

            mirrors_focus_index: 0,
            mirrors_regions_options: Vec::new(),
            mirrors_regions_selected: BTreeSet::new(),
            mirrors_loaded: false,
            optional_repos_options: vec!["multilib".into(), "testing".into(), "AUR".into()],
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
            aur_selected: false,
            aur_helper_index: None,

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
            addpkgs_group_focus: false,
            addpkgs_group_names: vec![
                "Terminals".into(),
                "Shells".into(),
                "Browsers".into(),
                "Text Editors".into(),
                "dotfile Management".into(),
            ],
            addpkgs_group_index: 0,
            addpkgs_group_pkg_index: 0,
            addpkgs_group_pkg_selected: std::collections::BTreeSet::new(),
            addpkgs_group_accum_selected: std::collections::BTreeSet::new(),
            addpkgs_group_selected: std::collections::BTreeMap::new(),

            processed_sections: BTreeSet::new(),

            pending_wipe_confirm: None,

            install_running: false,
            install_log: Vec::new(),
            install_log_tx: None,
            install_log_rx: None,
            install_section_titles: Vec::new(),
            install_section_done: Vec::new(),
            install_current_section: None,

            exit_tui_after_install: false,
            pending_install_sections: None,
            install_click_targets: Vec::new(),
            install_focus_index: 0,
        };
        // Initialize dynamic option lists and apply startup defaults
        let _ = s.load_locales_options();
        // Keyboard: us
        if let Some(idx) = s.keyboard_layout_options.iter().position(|k| k == "us") {
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
        if s.mirrors_regions_selected.is_empty()
            && let Some(idx) = s
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

    pub fn append_install_log_line(&mut self, line: String) {
        // Interpret simple progress markers and update state
        if let Some(rest) = line.strip_prefix("::section_start::") {
            if let Some(idx) = self.install_section_titles.iter().position(|t| t == rest) {
                self.install_current_section = Some(idx);
            }
            return;
        }
        if let Some(rest) = line.strip_prefix("::section_done::") {
            if let Some(idx) = self.install_section_titles.iter().position(|t| t == rest)
                && idx < self.install_section_done.len()
            {
                self.install_section_done[idx] = true;
            }
            return;
        }

        self.install_log.push(line);
        // keep log reasonably small
        // No log line limit: allow install_log to grow as needed
        // Update info popup body if it's open as Info
        let info_open = self.popup_open && matches!(self.popup_kind, Some(PopupKind::Info));
        if info_open {
            let body = self.install_log.join("\n");
            if self.popup_items.is_empty() {
                self.popup_items.push(body);
            } else {
                self.popup_items[0] = body;
            }
        }
    }

    pub fn drain_install_logs(&mut self) {
        let Some(rx) = self.install_log_rx.take() else {
            if self.install_running {
                self.debug_log("drain_install_logs: no install_log_rx");
            }
            return;
        };
        let mut drained: Vec<String> = Vec::new();
        let mut disconnected = false;
        loop {
            match rx.try_recv() {
                Ok(line) => drained.push(line),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    disconnected = true;
                    self.debug_log("drain_install_logs: channel disconnected, will set install_running = false");
                    break;
                }
            }
        }
        for line in drained {
            self.append_install_log_line(line);
        }
        // Update last_install_log_len for visual indicator
        self.last_install_log_len = Some(self.install_log.len());
        if disconnected {
            self.install_running = false;
            self.install_log_rx = None;
            self.install_log_tx = None;
            self.debug_log("drain_install_logs: install_running set to false");
            // Show a message in the TUI log output when channel is disconnected
            self.install_log
                .push("[Install process ended: no more output will be received.]".to_string());
        } else {
            self.install_log_rx = Some(rx);
        }
    }

    fn debug_log(&self, msg: &str) {
        if !self.debug_enabled {
            return;
        }
        let now = chrono::Local::now();
        let ts = now.format("%Y-%m-%d %H:%M:%S");
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .and_then(|mut f| {
                use std::io::Write;
                writeln!(f, "[DEBUG {}] {}", ts, msg)
            });
    }

    // Open the AUR helper selection popup
    pub fn open_aur_helper_popup(&mut self) {
        self.popup_kind = Some(PopupKind::AurHelperSelect);
        self.popup_items = vec![
            "yay — Yet another yogurt. Pacman wrapper and AUR helper written in go.".into(),
            "paru — Feature packed AUR helper written in Rust".into(),
        ];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_open = true;
    }
}
