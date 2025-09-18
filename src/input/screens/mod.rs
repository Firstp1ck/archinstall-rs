// screens module: modularized from the previous monolithic screens.rs

pub mod addpkgs;
pub mod ats;
pub mod audio;
pub mod bootloader;
pub mod common;
pub mod disk_encryption;
pub mod disks;
pub mod dispatcher;
pub mod experience;
pub mod hostname;
pub mod kernels;
pub mod locales;
pub mod mirrors;
pub mod network;
pub mod popups;
pub mod rootpass;
pub mod save_config;
pub mod swap;
pub mod timezone;
pub mod uki;
pub mod user;

// Re-export functions so existing call sites continue to work
pub(crate) use dispatcher::*;
pub(crate) use popups::{popup_move_down, popup_move_up};

// The rest of the functions will be moved here in subsequent edits and re-exported
