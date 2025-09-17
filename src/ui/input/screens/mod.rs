// screens module: modularized from the previous monolithic screens.rs

pub mod popups;
pub mod dispatcher;
pub mod common;
pub mod locales;
pub mod mirrors;
pub mod disks;
pub mod disk_encryption;
pub mod swap;
pub mod bootloader;
pub mod uki;
pub mod ats;
pub mod kernels;
pub mod audio;
pub mod network;
pub mod hostname;
pub mod timezone;
pub mod addpkgs;
pub mod rootpass;
pub mod user;
pub mod save_config;
pub mod experience;

// Re-export functions so existing call sites continue to work
pub(crate) use popups::{popup_move_down, popup_move_up};
pub(crate) use dispatcher::*;

// The rest of the functions will be moved here in subsequent edits and re-exported

