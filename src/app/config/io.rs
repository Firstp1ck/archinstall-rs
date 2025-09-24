use std::fs;
use std::path::PathBuf;

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

use crate::app::{
    AdditionalPackage, AppState, CustomRepo, RepoSignOption, RepoSignature, UserAccount,
};

use super::types::*;

#[derive(Debug)]
pub enum ConfigLoadError {
    ReadFile,
    ParseToml,
}

impl std::fmt::Display for ConfigLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigLoadError::ReadFile => write!(f, "failed to read config file"),
            ConfigLoadError::ParseToml => write!(f, "failed to parse config file (TOML)"),
        }
    }
}

impl std::error::Error for ConfigLoadError {}

impl AppState {
    fn config_path() -> PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("archinstall-rs.config.toml")
    }

    fn build_config(&self) -> AppConfig {
        let locales = ConfigLocales {
            keyboard_layout: self.current_keyboard_layout(),
            locale_language: self.current_locale_language(),
            locale_encoding: self.current_locale_encoding(),
        };
        let mirrors = ConfigMirrors {
            regions: self
                .mirrors_regions_selected
                .iter()
                .filter_map(|&i| self.mirrors_regions_options.get(i).cloned())
                .collect(),
            optional_repos: self
                .optional_repos_selected
                .iter()
                .filter_map(|&i| self.optional_repos_options.get(i).cloned())
                .collect(),
            aur_helper: self
                .aur_helper_index
                .map(|i| if i == 1 { "paru".into() } else { "yay".into() }),
            custom_servers: self.mirrors_custom_servers.clone(),
            custom_repos: self
                .custom_repos
                .iter()
                .map(|r: &CustomRepo| CustomRepoConfig {
                    name: r.name.clone(),
                    url: r.url.clone(),
                    signature: match r.signature {
                        RepoSignature::Never => "Never".into(),
                        RepoSignature::Optional => "Optional".into(),
                        RepoSignature::Required => "Required".into(),
                    },
                    sign_option: r.sign_option.map(|s| match s {
                        RepoSignOption::TrustedOnly => "TrustedOnly".into(),
                        RepoSignOption::TrustedAll => "TrustedAll".into(),
                    }),
                })
                .collect(),
        };
        let disks_mode = match self.disks_mode_index {
            0 => "Best-effort partition layout",
            1 => "Manual Partitioning",
            _ => "Pre-mounted configuration",
        };
        let disks = ConfigDisks {
            mode: disks_mode.into(),
            selected_device: self.disks_selected_device.clone(),
            selected_device_model: self.disks_selected_device_model.clone(),
            selected_device_devtype: self.disks_selected_device_devtype.clone(),
            selected_device_size: self.disks_selected_device_size.clone(),
            selected_device_freespace: self.disks_selected_device_freespace.clone(),
            selected_device_sector_size: self.disks_selected_device_sector_size.clone(),
            selected_device_read_only: self.disks_selected_device_read_only,
            label: self.disks_label.clone(),
            wipe: Some(self.disks_wipe),
            align: self.disks_align.clone(),
            partitions: self
                .disks_partitions
                .iter()
                .map(|p| ConfigPartition {
                    name: p.name.clone(),
                    role: p.role.clone(),
                    fs: p.fs.clone(),
                    start: p.start.clone(),
                    size: p.size.clone(),
                    flags: p.flags.clone(),
                    mountpoint: p.mountpoint.clone(),
                    mount_options: p.mount_options.clone(),
                    encrypt: p.encrypt,
                })
                .collect(),
        };
        let encryption_type = match self.disk_encryption_type_index {
            0 => "None",
            _ => "LUKS",
        }
        .to_string();
        let password_hash =
            if self.disk_encryption_type_index == 1 && !self.disk_encryption_password.is_empty() {
                let mut hasher = Sha256::new();
                hasher.update(self.disk_encryption_password.as_bytes());
                let result = hasher.finalize();
                Some(format!("{:x}", result))
            } else {
                None
            };
        let disk_encryption = ConfigDiskEncryption {
            encryption_type,
            partition: self.disk_encryption_selected_partition.clone(),
            password_hash,
        };
        let swap = ConfigSwap {
            enabled: self.swap_enabled,
        };
        let bootloader = ConfigBootloader {
            kind: match self.bootloader_index {
                0 => "systemd-boot".into(),
                1 => "grub".into(),
                2 => "efistub".into(),
                _ => "limine".into(),
            },
        };
        let system = ConfigSystem {
            hostname: self.hostname_value.clone(),
            root_password_hash: self.root_password_hash.clone().or_else(|| {
                if !self.root_password.is_empty()
                    && !self.root_password_confirm.is_empty()
                    && self.root_password == self.root_password_confirm
                {
                    let mut hasher = Sha256::new();
                    hasher.update(self.root_password.as_bytes());
                    let result = hasher.finalize();
                    Some(format!("{:x}", result))
                } else {
                    None
                }
            }),
            automatic_time_sync: self.ats_enabled,
            timezone: self.timezone_value.clone(),
        };
        let unified_kernel_images = ConfigUnifiedKernelImages {
            enabled: self.uki_enabled,
        };

        let experience_mode = match self.experience_mode_index {
            0 => "Desktop",
            1 => "Minimal",
            2 => "Server",
            3 => "Xorg",
            _ => "Desktop",
        }
        .to_string();

        let map_set_to_vec_filtered =
            |m: &std::collections::BTreeMap<String, std::collections::BTreeSet<String>>,
             allowed: &std::collections::BTreeSet<String>| {
                let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
                for (k, set) in m.iter() {
                    if !allowed.contains(k) {
                        continue;
                    }
                    let mut v: Vec<String> = set.iter().cloned().collect();
                    v.sort();
                    out.insert(k.clone(), v);
                }
                out
            };

        let experience = ConfigExperience {
            mode: experience_mode,
            desktop_envs: {
                let mut v: Vec<String> = self.selected_desktop_envs.iter().cloned().collect();
                v.sort();
                v
            },
            server_types: {
                let mut v: Vec<String> = self.selected_server_types.iter().cloned().collect();
                v.sort();
                v
            },
            xorg_types: {
                let mut v: Vec<String> = self.selected_xorg_types.iter().cloned().collect();
                v.sort();
                v
            },
            desktop_env_packages: map_set_to_vec_filtered(
                &self.selected_env_packages,
                &self.selected_desktop_envs,
            ),
            server_packages: map_set_to_vec_filtered(
                &self.selected_server_packages,
                &self.selected_server_types,
            ),
            xorg_packages: map_set_to_vec_filtered(
                &self.selected_xorg_packages,
                &self.selected_xorg_types,
            ),
            login_manager: self.selected_login_manager.clone(),
            login_manager_user_set: self.login_manager_user_set,
            graphic_drivers: {
                // Save no drivers only for Minimal; otherwise persist selection
                if self.experience_mode_index == 1 {
                    Vec::new()
                } else {
                    let mut v: Vec<String> =
                        self.selected_graphic_drivers.iter().cloned().collect();
                    v.sort();
                    v
                }
            },
        };
        let users: Vec<ConfigUser> = self
            .users
            .iter()
            .map(|u: &UserAccount| {
                let hash = if let Some(h) = &u.password_hash {
                    h.clone()
                } else if !u.password.is_empty() {
                    let mut hasher = Sha256::new();
                    hasher.update(u.password.as_bytes());
                    let result = hasher.finalize();
                    format!("{:x}", result)
                } else {
                    String::new()
                };
                ConfigUser {
                    username: u.username.clone(),
                    password_hash: hash,
                    is_sudo: u.is_sudo,
                }
            })
            .collect();
        let kernels = ConfigKernels {
            selected: {
                let mut v: Vec<String> = self.selected_kernels.iter().cloned().collect();
                v.sort();
                v
            },
        };
        let audio = ConfigAudio {
            kind: match self.audio_index {
                0 => "None",
                1 => "pipewire",
                _ => "pulseaudio",
            }
            .to_string(),
        };
        let additional_packages: Vec<ConfigAdditionalPackage> = self
            .additional_packages
            .iter()
            .map(|p: &AdditionalPackage| ConfigAdditionalPackage {
                repo: p.repo.clone(),
                name: p.name.clone(),
                version: p.version.clone(),
                description: p.description.clone(),
            })
            .collect();

        AppConfig {
            locales,
            mirrors,
            disks,
            disk_encryption,
            swap,
            bootloader,
            system,
            unified_kernel_images,
            kernels,
            audio,
            experience,
            users,
            network: ConfigNetwork {
                mode: match self.network_mode_index {
                    0 => "CopyISO".into(),
                    1 => "Manual".into(),
                    _ => "NetworkManager".into(),
                },
            },
            additional_packages,
        }
    }

    pub fn save_config(&self) {
        let cfg = self.build_config();
        let path = Self::config_path();
        // Summary without secrets
        let regions_count = self.mirrors_regions_selected.len();
        let repos_count = self.optional_repos_selected.len();
        let custom_servers = self.mirrors_custom_servers.len();
        let custom_repos = self.custom_repos.len();
        let users_count = self.users.len();
        let sudo_count = self.users.iter().filter(|u| u.is_sudo).count();
        let addpkgs_count = self.additional_packages.len();
        let experience_mode = match self.experience_mode_index {
            0 => "Desktop",
            1 => "Minimal",
            2 => "Server",
            3 => "Xorg",
            _ => "Desktop",
        };
        let bootloader_kind = match self.bootloader_index {
            0 => "systemd-boot",
            1 => "grub",
            2 => "efistub",
            _ => "limine",
        };
        let has_root_hash = self.root_password_hash.is_some()
            || (!self.root_password.is_empty()
                && !self.root_password_confirm.is_empty()
                && self.root_password == self.root_password_confirm);
        let has_diskenc_hash =
            self.disk_encryption_type_index == 1 && !self.disk_encryption_password.is_empty();
        self.debug_log(&format!(
            "save_config: path={} regions={} repos={} custom_servers={} custom_repos={} users={} (sudo={}) addpkgs={} experience={} bootloader={} hashes: root={} diskenc={}",
            path.display(),
            regions_count,
            repos_count,
            custom_servers,
            custom_repos,
            users_count,
            sudo_count,
            addpkgs_count,
            experience_mode,
            bootloader_kind,
            has_root_hash,
            has_diskenc_hash
        ));
        if let Ok(toml) = toml::to_string_pretty(&cfg) {
            let _ = fs::write(&path, toml);
            self.debug_log(&format!("save_config: wrote config to {}", path.display()));
        } else {
            self.debug_log("save_config: failed to serialize config to TOML");
        }
    }

    pub fn load_config(&mut self) -> Result<(), ConfigLoadError> {
        // Ensure option lists are available before mapping names back to indices
        let _ = self.load_locales_options();
        let _ = self.load_mirrors_options();

        let path = Self::config_path();
        self.debug_log(&format!("load_config: path={}", path.display()));
        let text = std::fs::read_to_string(&path).map_err(|e| {
            self.debug_log(&format!("load_config: read error: {}", e));
            ConfigLoadError::ReadFile
        })?;
        let cfg: AppConfig = toml::from_str(&text).map_err(|e| {
            self.debug_log(&format!("load_config: parse error: {}", e));
            ConfigLoadError::ParseToml
        })?;
        self.debug_log("load_config: parse ok");
        self.last_load_missing_sections.clear();

        // Locales
        if !cfg.locales.keyboard_layout.is_empty() {
            if let Some(idx) = self
                .keyboard_layout_options
                .iter()
                .position(|s| s == &cfg.locales.keyboard_layout)
            {
                self.keyboard_layout_index = idx;
            }
        } else {
            self.last_load_missing_sections
                .push("Locales: keyboard_layout".into());
        }
        if !cfg.locales.locale_language.is_empty() {
            if let Some(idx) = self
                .locale_language_options
                .iter()
                .position(|s| s == &cfg.locales.locale_language)
            {
                self.locale_language_index = idx;
            }
        } else {
            self.last_load_missing_sections
                .push("Locales: locale_language".into());
        }
        if !cfg.locales.locale_encoding.is_empty() {
            if let Some(idx) = self
                .locale_encoding_options
                .iter()
                .position(|s| s == &cfg.locales.locale_encoding)
            {
                self.locale_encoding_index = idx;
            }
        } else {
            self.last_load_missing_sections
                .push("Locales: locale_encoding".into());
        }

        // Mirrors (best effort)
        self.mirrors_regions_selected.clear();
        if cfg.mirrors.regions.is_empty() {
            self.last_load_missing_sections
                .push("Mirrors: regions".into());
        }
        for name in cfg.mirrors.regions {
            if let Some(idx) = self.mirrors_regions_options.iter().position(|s| s == &name) {
                self.mirrors_regions_selected.insert(idx);
            }
        }
        self.optional_repos_selected.clear();
        if cfg.mirrors.optional_repos.is_empty() {
            self.last_load_missing_sections
                .push("Mirrors: optional_repos".into());
        }
        for name in cfg.mirrors.optional_repos {
            if let Some(idx) = self.optional_repos_options.iter().position(|s| s == &name) {
                self.optional_repos_selected.insert(idx);
            }
        }
        // Load aur_helper
        if let Some(helper) = cfg.mirrors.aur_helper.clone() {
            self.aur_selected = true;
            self.aur_helper_index = Some(if helper.eq_ignore_ascii_case("paru") {
                1
            } else {
                0
            });
        }
        if cfg.mirrors.custom_servers.is_empty() {
            self.last_load_missing_sections
                .push("Mirrors: custom_servers".into());
        }
        self.mirrors_custom_servers = cfg.mirrors.custom_servers;
        self.custom_repos = cfg
            .mirrors
            .custom_repos
            .into_iter()
            .map(|r| CustomRepo {
                name: r.name,
                url: r.url,
                signature: match r.signature.as_str() {
                    "Never" => RepoSignature::Never,
                    "Optional" => RepoSignature::Optional,
                    _ => RepoSignature::Required,
                },
                sign_option: r.sign_option.as_deref().map(|s| {
                    if s == "TrustedOnly" {
                        RepoSignOption::TrustedOnly
                    } else {
                        RepoSignOption::TrustedAll
                    }
                }),
            })
            .collect();
        if self.custom_repos.is_empty() {
            self.last_load_missing_sections
                .push("Mirrors: custom_repos".into());
        }

        // Disks
        self.load_disks_devices();
        let loaded_selected_device = cfg.disks.selected_device.clone();
        if cfg.disks.mode.is_empty() {
            self.last_load_missing_sections.push("Disks: mode".into());
        }
        self.disks_mode_index = match cfg.disks.mode.as_str() {
            "Best-effort partition layout" => 0,
            "Manual Partitioning" => 1,
            _ => 2,
        };
        self.disks_selected_device = loaded_selected_device.and_then(|p| {
            if self.disks_devices.iter().any(|d| d.path == p) {
                Some(p)
            } else {
                None
            }
        });
        // Populate cached disk details by matching the path
        if let Some(ref sel_path) = self.disks_selected_device {
            if let Some(dev) = self.disks_devices.iter().find(|d| &d.path == sel_path) {
                self.disks_selected_device_model = Some(dev.model.clone());
                self.disks_selected_device_devtype = Some(dev.devtype.clone());
                self.disks_selected_device_size = Some(dev.size.clone());
                self.disks_selected_device_freespace = Some(dev.freespace.clone());
                self.disks_selected_device_sector_size = Some(dev.sector_size.clone());
                self.disks_selected_device_read_only = Some(dev.read_only);
            }
        } else {
            self.disks_selected_device_model = None;
            self.disks_selected_device_devtype = None;
            self.disks_selected_device_size = None;
            self.disks_selected_device_freespace = None;
            self.disks_selected_device_sector_size = None;
            self.disks_selected_device_read_only = None;
        }
        if self.disks_selected_device.is_none() {
            self.last_load_missing_sections
                .push("Disks: selected_device".into());
        }
        // Extended disk configuration
        if let Some(l) = cfg.disks.label.clone() {
            self.disks_label = Some(l);
        }
        if let Some(w) = cfg.disks.wipe {
            self.disks_wipe = w;
        }
        if let Some(a) = cfg.disks.align.clone() {
            self.disks_align = Some(a);
        }
        self.disks_partitions = cfg
            .disks
            .partitions
            .into_iter()
            .map(|p| crate::app::DiskPartitionSpec {
                name: p.name,
                role: p.role,
                fs: p.fs,
                start: p.start,
                size: p.size,
                flags: p.flags,
                mountpoint: p.mountpoint,
                mount_options: p.mount_options,
                encrypt: p.encrypt,
            })
            .collect();

        // Disk encryption
        if cfg.disk_encryption.encryption_type.is_empty() {
            self.last_load_missing_sections
                .push("DiskEncryption: encryption_type".into());
        }
        self.disk_encryption_type_index = if cfg.disk_encryption.encryption_type == "LUKS" {
            1
        } else {
            0
        };
        self.disk_encryption_selected_partition = cfg.disk_encryption.partition;
        if self.disk_encryption_selected_partition.is_none() {
            self.last_load_missing_sections
                .push("DiskEncryption: partition".into());
        }
        // do not load plaintext password; hash in config is not converted back

        // Swap
        self.swap_enabled = cfg.swap.enabled;

        // Bootloader
        if cfg.bootloader.kind.is_empty() {
            self.last_load_missing_sections
                .push("Bootloader: kind".into());
        }
        let bootloader_kind = cfg.bootloader.kind.clone();
        self.bootloader_index = match bootloader_kind.as_str() {
            "systemd-boot" => 0,
            "grub" => 1,
            "efistub" => 2,
            _ => 3,
        };

        // System
        if cfg.system.hostname.is_empty() {
            self.last_load_missing_sections
                .push("System: hostname".into());
        }
        self.hostname_value = cfg.system.hostname;
        self.root_password_hash = cfg.system.root_password_hash; // keep as hash only
        if self.root_password_hash.is_none() {
            self.last_load_missing_sections
                .push("System: root_password_hash".into());
        }
        self.ats_enabled = cfg.system.automatic_time_sync;
        if cfg.system.timezone.is_empty() {
            self.last_load_missing_sections
                .push("System: timezone".into());
        }
        self.timezone_value = cfg.system.timezone;

        // Unified Kernel Images
        self.uki_enabled = cfg.unified_kernel_images.enabled;

        // Kernels
        self.selected_kernels = cfg.kernels.selected.into_iter().collect();
        if self.selected_kernels.is_empty() {
            self.last_load_missing_sections
                .push("Kernels: selected".into());
        }

        // Audio
        self.audio_index = match cfg.audio.kind.as_str() {
            "pipewire" => 1,
            "pulseaudio" => 2,
            _ => 0,
        };

        // Experience
        self.experience_mode_index = match cfg.experience.mode.as_str() {
            "Desktop" => 0,
            "Minimal" => 1,
            "Server" => 2,
            "Xorg" => 3,
            _ => 0,
        };
        self.selected_desktop_envs = cfg.experience.desktop_envs.into_iter().collect();
        self.selected_server_types = cfg.experience.server_types.into_iter().collect();
        self.selected_xorg_types = cfg.experience.xorg_types.into_iter().collect();
        self.selected_env_packages = cfg
            .experience
            .desktop_env_packages
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();
        self.selected_server_packages = cfg
            .experience
            .server_packages
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();
        self.selected_xorg_packages = cfg
            .experience
            .xorg_packages
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();
        self.selected_login_manager = cfg.experience.login_manager;
        self.login_manager_user_set = cfg.experience.login_manager_user_set;
        self.selected_graphic_drivers = if self.experience_mode_index == 1 {
            // Minimal mode: do not load any graphics drivers from config
            std::collections::BTreeSet::new()
        } else {
            cfg.experience.graphic_drivers.into_iter().collect()
        };

        // Users
        self.users = cfg
            .users
            .into_iter()
            .map(|u| UserAccount {
                username: u.username,
                password: String::new(), // do not load plaintext
                password_hash: if u.password_hash.is_empty() {
                    None
                } else {
                    Some(u.password_hash)
                },
                is_sudo: u.is_sudo,
            })
            .collect();
        if self.users.is_empty() {
            self.last_load_missing_sections.push("Users: list".into());
        }

        // Network
        self.network_mode_index = match cfg.network.mode.as_str() {
            "CopyISO" => 0,
            "Manual" => 1,
            _ => 2,
        };

        // Additional packages
        self.additional_packages = cfg
            .additional_packages
            .into_iter()
            .map(|p| AdditionalPackage {
                name: p.name,
                repo: p.repo,
                version: p.version,
                description: p.description,
            })
            .collect();

        // Final summary
        self.debug_log(&format!(
            "load_config: summary regions={} repos={} custom_servers={} custom_repos={} users={} addpkgs={} missing_sections={}",
            self.mirrors_regions_selected.len(),
            self.optional_repos_selected.len(),
            self.mirrors_custom_servers.len(),
            self.custom_repos.len(),
            self.users.len(),
            self.additional_packages.len(),
            self.last_load_missing_sections.len()
        ));
        if !self.last_load_missing_sections.is_empty() {
            self.debug_log(&format!(
                "load_config: missing sections -> [{}]",
                self.last_load_missing_sections.join(", ")
            ));
        }

        Ok(())
    }
}
