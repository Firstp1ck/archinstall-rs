use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigLocales {
    pub keyboard_layout: String,
    pub locale_language: String,
    pub locale_encoding: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigMirrors {
    pub regions: Vec<String>,
    pub optional_repos: Vec<String>,
    pub custom_servers: Vec<String>,
    pub custom_repos: Vec<CustomRepoConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct CustomRepoConfig {
    pub name: String,
    pub url: String,
    pub signature: String,
    pub sign_option: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigDisks {
    pub mode: String,
    pub selected_device: Option<String>,
    pub selected_device_model: Option<String>,
    pub selected_device_devtype: Option<String>,
    pub selected_device_size: Option<String>,
    pub selected_device_freespace: Option<String>,
    pub selected_device_sector_size: Option<String>,
    pub selected_device_read_only: Option<bool>,
    // Extended disk configuration for explicit partitioning
    pub label: Option<String>, // "gpt" | "msdos"
    pub wipe: Option<bool>,    // wipe the disk before partitioning
    pub align: Option<String>, // e.g. "1MiB"
    pub partitions: Vec<ConfigPartition>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct ConfigPartition {
    pub name: Option<String>,
    pub role: Option<String>, // e.g. efi, bios_boot, root, swap, home, var, ...
    pub fs: Option<String>,   // fat32, ext4, btrfs, xfs, swap, ...
    pub start: Option<String>, // e.g. 1MiB, 513MiB
    pub size: Option<String>, // e.g. 512MiB, 8GiB, 100%
    pub flags: Vec<String>,   // esp, boot, legacy_boot, ...
    pub mountpoint: Option<String>,
    pub mount_options: Option<String>,
    pub encrypt: Option<bool>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigDiskEncryption {
    pub encryption_type: String,
    pub partition: Option<String>,
    pub password_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigSwap {
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigBootloader {
    pub kind: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigSystem {
    pub hostname: String,
    pub root_password_hash: Option<String>,
    pub automatic_time_sync: bool,
    pub timezone: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigUnifiedKernelImages {
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigNetwork {
    pub mode: String, // "CopyISO" | "Manual" | "NetworkManager"
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigAudio {
    pub kind: String, // "None" | "pipewire" | "pulseaudio"
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigExperience {
    pub mode: String, // "Desktop" | "Minimal" | "Server" | "Xorg"
    pub desktop_envs: Vec<String>,
    pub server_types: Vec<String>,
    pub xorg_types: Vec<String>,
    pub desktop_env_packages: BTreeMap<String, Vec<String>>, // env -> packages
    pub server_packages: BTreeMap<String, Vec<String>>,      // type -> packages
    pub xorg_packages: BTreeMap<String, Vec<String>>,        // type -> packages
    pub login_manager: Option<String>,
    pub login_manager_user_set: bool,
    pub graphic_drivers: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppConfig {
    pub locales: ConfigLocales,
    pub mirrors: ConfigMirrors,
    pub disks: ConfigDisks,
    pub disk_encryption: ConfigDiskEncryption,
    pub swap: ConfigSwap,
    pub bootloader: ConfigBootloader,
    pub system: ConfigSystem,
    pub unified_kernel_images: ConfigUnifiedKernelImages,
    pub kernels: ConfigKernels,
    pub audio: ConfigAudio,
    pub experience: ConfigExperience,
    pub users: Vec<ConfigUser>,
    pub network: ConfigNetwork,
    pub additional_packages: Vec<ConfigAdditionalPackage>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigUser {
    pub username: String,
    pub password_hash: String,
    pub is_sudo: bool,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigKernels {
    pub selected: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigAdditionalPackage {
    pub repo: String,
    pub name: String,
    pub version: String,
    pub description: String,
}
