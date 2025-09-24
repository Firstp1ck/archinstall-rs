use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::{AppState, Screen};

// Submodules for each screen-specific info renderer
mod additional_packages;
mod audio;
mod automatic_time_sync;
mod bootloader;
mod default_screen;
mod disk_encryption;
mod disks;
mod experience_mode;
mod hostname;
mod kernels;
mod locales;
mod mirrors_repos;
mod network_configuration;
mod root_password;
mod save_configuration;
mod swap;
mod timezone;
mod unified_kernel_images;
mod user_account;

pub fn draw_info(frame: &mut Frame, app: &mut AppState, area: Rect) {
    match app.current_screen() {
        Screen::Locales => locales::render(frame, app, area),
        Screen::MirrorsRepos => mirrors_repos::render(frame, app, area),
        Screen::DiskEncryption => disk_encryption::render(frame, app, area),
        Screen::Disks => disks::render(frame, app, area),
        Screen::SwapPartition => swap::render(frame, app, area),
        Screen::Bootloader => bootloader::render(frame, app, area),
        Screen::Hostname => hostname::render(frame, app, area),
        Screen::RootPassword => root_password::render(frame, app, area),
        Screen::Timezone => timezone::render(frame, app, area),
        Screen::UnifiedKernelImages => unified_kernel_images::render(frame, app, area),
        Screen::AutomaticTimeSync => automatic_time_sync::render(frame, app, area),
        Screen::Kernels => kernels::render(frame, app, area),
        Screen::AdditionalPackages => additional_packages::render(frame, app, area),
        Screen::Audio => audio::render(frame, app, area),
        Screen::NetworkConfiguration => network_configuration::render(frame, app, area),
        Screen::UserAccount => user_account::render(frame, app, area),
        Screen::ExperienceMode => experience_mode::render(frame, app, area),
        Screen::SaveConfiguration => save_configuration::render(frame, app, area),
        _ => default_screen::render(frame, app, area),
    }
}
