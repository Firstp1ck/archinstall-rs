// Re-export core state and types to preserve crate::app::* paths
pub use crate::core::state::AppState;
pub use crate::core::types::{
    AdditionalPackage, CustomRepo, DiskPartitionSpec, Focus, MenuEntry, NetworkConfigMode,
    NetworkInterfaceConfig, PopupKind, RepoSignOption, RepoSignature, Screen, UserAccount,
};

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
pub const KEYBINDS_WIDTH: u16 = 44;
